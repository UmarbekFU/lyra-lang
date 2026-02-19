pub mod bytecode;
pub mod locals;

use crate::ast::*;
use crate::eval::value::Value;
use crate::span::Span;

use bytecode::{Chunk, FunctionProto, Op, UpvalueRef};
use locals::ScopeTracker;

/// Compiler state for one function scope.
struct CompilerFrame {
    proto: FunctionProto,
    scope: ScopeTracker,
}

pub struct Compiler {
    frames: Vec<CompilerFrame>,
}

impl Compiler {
    pub fn new() -> Self {
        let main_frame = CompilerFrame {
            proto: FunctionProto {
                name: "<main>".to_string(),
                arity: 0,
                chunk: Chunk::new(),
                upvalue_count: 0,
            },
            scope: ScopeTracker::new(),
        };
        Compiler {
            frames: vec![main_frame],
        }
    }

    fn current(&mut self) -> &mut CompilerFrame {
        self.frames.last_mut().unwrap()
    }

    fn emit(&mut self, op: Op, span: Span) -> usize {
        self.current().proto.chunk.emit(op, span)
    }

    fn add_constant(&mut self, value: Value) -> usize {
        self.current().proto.chunk.add_constant(value)
    }

    fn patch_jump(&mut self, offset: usize) {
        self.current().proto.chunk.patch_jump(offset);
    }

    #[allow(dead_code)]
    fn current_offset(&self) -> usize {
        self.frames.last().unwrap().proto.chunk.current_offset()
    }

    /// Compile a full program (list of declarations).
    pub fn compile_program(mut self, decls: &[Decl]) -> Result<FunctionProto, String> {
        let last_idx = decls.len().saturating_sub(1);
        for (i, decl) in decls.iter().enumerate() {
            // For the last declaration, if it's an expression, keep its value on the stack
            if i == last_idx {
                if let Decl::Expr(expr) = decl {
                    self.compile_expr(expr)?;
                    self.emit(Op::Return, expr.span);
                    let frame = self.frames.pop().unwrap();
                    return Ok(frame.proto);
                }
            }
            self.compile_decl(decl)?;
        }
        self.emit(Op::Unit, Span::default());
        self.emit(Op::Return, Span::default());

        let frame = self.frames.pop().unwrap();
        Ok(frame.proto)
    }

    fn compile_decl(&mut self, decl: &Decl) -> Result<(), String> {
        match decl {
            Decl::Let {
                name,
                recursive,
                body,
                ..
            } => {
                if *recursive {
                    // For recursive functions: define the global first, then compile
                    self.emit(Op::Unit, name.span);
                    self.emit(Op::DefineGlobal(name.node.clone()), name.span);
                    self.compile_expr(body)?;
                    self.emit(Op::DefineGlobal(name.node.clone()), name.span);
                } else {
                    self.compile_expr(body)?;
                    self.emit(Op::DefineGlobal(name.node.clone()), name.span);
                }
                Ok(())
            }

            Decl::Type { variants, .. } => {
                // Register constructors as globals
                for variant in variants {
                    let name = &variant.name.node;
                    let arity = variant.fields.len();
                    if arity == 0 {
                        // Nullary: just an ADT value
                        self.emit(Op::MakeAdt(name.clone(), 0), variant.span);
                        self.emit(Op::DefineGlobal(name.clone()), variant.span);
                    } else {
                        // Constructor with fields: compile a wrapper function
                        self.compile_constructor_fn(name, arity, variant.span)?;
                        self.emit(Op::DefineGlobal(name.clone()), variant.span);
                    }
                }
                Ok(())
            }

            Decl::Expr(expr) => {
                self.compile_expr(expr)?;
                self.emit(Op::Pop, expr.span);
                Ok(())
            }

            _ => Ok(()),
        }
    }

    fn compile_constructor_fn(
        &mut self,
        name: &str,
        arity: usize,
        span: Span,
    ) -> Result<(), String> {
        // Create a function that takes `arity` args and builds an ADT
        let mut proto = FunctionProto {
            name: name.to_string(),
            arity: arity as u8,
            chunk: Chunk::new(),
            upvalue_count: 0,
        };

        // The function body: push all params, then MakeAdt
        for i in 0..arity {
            proto.chunk.emit(Op::GetLocal(i), span);
        }
        proto.chunk.emit(Op::MakeAdt(name.to_string(), arity), span);
        proto.chunk.emit(Op::Return, span);

        let const_idx = self.add_constant(Value::Function(proto));
        self.emit(Op::Closure(const_idx, vec![]), span);
        Ok(())
    }

    fn compile_expr(&mut self, expr: &SpannedExpr) -> Result<(), String> {
        let span = expr.span;
        match &expr.node {
            Expr::IntLit(n) => {
                let idx = self.add_constant(Value::Int(*n));
                self.emit(Op::Constant(idx), span);
            }
            Expr::FloatLit(n) => {
                let idx = self.add_constant(Value::Float(*n));
                self.emit(Op::Constant(idx), span);
            }
            Expr::BoolLit(true) => {
                self.emit(Op::True, span);
            }
            Expr::BoolLit(false) => {
                self.emit(Op::False, span);
            }
            Expr::StringLit(s) => {
                let idx = self.add_constant(Value::String(s.clone()));
                self.emit(Op::Constant(idx), span);
            }
            Expr::UnitLit => {
                self.emit(Op::Unit, span);
            }

            Expr::Var(name) => {
                self.compile_var_access(name, span);
            }

            Expr::ListLit(elems) => {
                for elem in elems {
                    self.compile_expr(elem)?;
                }
                self.emit(Op::MakeList(elems.len()), span);
            }

            Expr::TupleLit(elems) => {
                for elem in elems {
                    self.compile_expr(elem)?;
                }
                self.emit(Op::MakeTuple(elems.len()), span);
            }

            Expr::Lambda { params, body } => {
                self.compile_lambda(params, body, None, span)?;
            }

            Expr::App { func, args } => {
                self.compile_expr(func)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit(Op::Call(args.len() as u8), span);
            }

            Expr::BinOp { op, lhs, rhs } => {
                // Short-circuit for && and ||
                match op {
                    BinOp::And => {
                        self.compile_expr(lhs)?;
                        let jump = self.emit(Op::JumpIfFalse(0), span);
                        self.emit(Op::Pop, span);
                        self.compile_expr(rhs)?;
                        self.patch_jump(jump);
                        return Ok(());
                    }
                    BinOp::Or => {
                        self.compile_expr(lhs)?;
                        // If true, skip rhs
                        let else_jump = self.emit(Op::JumpIfFalse(0), span);
                        let end_jump = self.emit(Op::Jump(0), span);
                        self.patch_jump(else_jump);
                        self.emit(Op::Pop, span);
                        self.compile_expr(rhs)?;
                        self.patch_jump(end_jump);
                        return Ok(());
                    }
                    _ => {}
                }

                self.compile_expr(lhs)?;
                self.compile_expr(rhs)?;
                match op {
                    BinOp::Add => self.emit(Op::Add, span),
                    BinOp::Sub => self.emit(Op::Sub, span),
                    BinOp::Mul => self.emit(Op::Mul, span),
                    BinOp::Div => self.emit(Op::Div, span),
                    BinOp::Mod => self.emit(Op::Mod, span),
                    BinOp::Eq => self.emit(Op::Equal, span),
                    BinOp::NotEq => self.emit(Op::NotEqual, span),
                    BinOp::Lt => self.emit(Op::Less, span),
                    BinOp::Gt => self.emit(Op::Greater, span),
                    BinOp::Le => self.emit(Op::LessEqual, span),
                    BinOp::Ge => self.emit(Op::GreaterEqual, span),
                    BinOp::Cons => self.emit(Op::Cons, span),
                    BinOp::And | BinOp::Or => unreachable!(),
                };
            }

            Expr::UnaryOp { op, operand } => {
                self.compile_expr(operand)?;
                match op {
                    UnaryOp::Neg => self.emit(Op::Negate, span),
                    UnaryOp::Not => self.emit(Op::Not, span),
                };
            }

            Expr::Pipe { lhs, rhs } => {
                // a |> f  compiles to  f(a)
                self.compile_expr(rhs)?;
                self.compile_expr(lhs)?;
                self.emit(Op::Call(1), span);
            }

            Expr::If {
                cond,
                then_branch,
                else_branch,
            } => {
                self.compile_expr(cond)?;
                let else_jump = self.emit(Op::JumpIfFalse(0), span);
                self.emit(Op::Pop, span);
                self.compile_expr(then_branch)?;
                let end_jump = self.emit(Op::Jump(0), span);
                self.patch_jump(else_jump);
                self.emit(Op::Pop, span);
                self.compile_expr(else_branch)?;
                self.patch_jump(end_jump);
            }

            Expr::Let {
                name,
                recursive,
                value,
                body,
                ..
            } => {
                self.current().scope.begin_scope();

                if *recursive {
                    // Placeholder for recursive reference
                    self.emit(Op::Unit, span);
                    let local_idx = self.current().scope.add_local(name.node.clone());
                    self.compile_expr(value)?;
                    self.emit(Op::SetLocal(local_idx), span);
                } else {
                    self.compile_expr(value)?;
                    self.current().scope.add_local(name.node.clone());
                }

                self.compile_expr(body)?;

                // Stack: [... local_value body_result]
                // Pop the local from under the result.
                let pops = self.current().scope.end_scope();
                if pops > 0 {
                    self.emit(Op::PopUnder(pops), span);
                }
            }

            Expr::Match { scrutinee, arms } => {
                self.compile_match(scrutinee, arms, span)?;
            }

            Expr::Interpolation(parts) => {
                for (i, part) in parts.iter().enumerate() {
                    match part {
                        InterpolationPart::Literal(s) => {
                            let idx = self.add_constant(Value::String(s.clone()));
                            self.emit(Op::Constant(idx), span);
                        }
                        InterpolationPart::Expr(expr) => {
                            self.compile_expr(expr)?;
                            self.emit(Op::ToString, span);
                        }
                    }
                    if i > 0 {
                        self.emit(Op::StringConcat, span);
                    }
                }
                if parts.is_empty() {
                    let idx = self.add_constant(Value::String(String::new()));
                    self.emit(Op::Constant(idx), span);
                }
            }

            Expr::Record(fields) => {
                let names: Vec<String> = fields.iter().map(|(n, _)| n.clone()).collect();
                for (_, val) in fields {
                    self.compile_expr(val)?;
                }
                self.emit(Op::MakeRecord(names), span);
            }

            Expr::FieldAccess { expr: obj, field } => {
                self.compile_expr(obj)?;
                self.emit(Op::GetField(field.clone()), span);
            }
        }
        Ok(())
    }

    /// Compile an expression in tail position — emits TailCall for App nodes.
    fn compile_expr_tail(&mut self, expr: &SpannedExpr) -> Result<(), String> {
        let span = expr.span;
        match &expr.node {
            // App in tail position → TailCall
            Expr::App { func, args } => {
                self.compile_expr(func)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit(Op::TailCall(args.len() as u8), span);
                Ok(())
            }

            // If: propagate tail position into both branches
            Expr::If {
                cond,
                then_branch,
                else_branch,
            } => {
                self.compile_expr(cond)?;
                let else_jump = self.emit(Op::JumpIfFalse(0), span);
                self.emit(Op::Pop, span);
                self.compile_expr_tail(then_branch)?;
                let end_jump = self.emit(Op::Jump(0), span);
                self.patch_jump(else_jump);
                self.emit(Op::Pop, span);
                self.compile_expr_tail(else_branch)?;
                self.patch_jump(end_jump);
                Ok(())
            }

            // Let: propagate tail position into body
            Expr::Let {
                name,
                recursive,
                value,
                body,
                ..
            } => {
                self.current().scope.begin_scope();
                if *recursive {
                    self.emit(Op::Unit, span);
                    let local_idx = self.current().scope.add_local(name.node.clone());
                    self.compile_expr(value)?;
                    self.emit(Op::SetLocal(local_idx), span);
                } else {
                    self.compile_expr(value)?;
                    self.current().scope.add_local(name.node.clone());
                }
                self.compile_expr_tail(body)?;
                let pops = self.current().scope.end_scope();
                if pops > 0 {
                    self.emit(Op::PopUnder(pops), span);
                }
                Ok(())
            }

            // Match: propagate tail position into arm bodies
            Expr::Match { scrutinee, arms } => {
                // Reuse the existing match compilation but with tail calls in bodies
                // For simplicity, fall back to non-tail compilation
                // (full TCO through match would require duplicating compile_match)
                self.compile_match(scrutinee, arms, span)
            }

            // Everything else: compile normally (not in tail position)
            _ => self.compile_expr(expr),
        }
    }

    fn compile_var_access(&mut self, name: &str, span: Span) {
        // Check locals first
        if let Some(idx) = self.current().scope.resolve_local(name) {
            self.emit(Op::GetLocal(idx), span);
            return;
        }

        // Check upvalues (for closures)
        if self.frames.len() > 1 {
            if let Some(idx) = self.resolve_upvalue(self.frames.len() - 1, name) {
                self.emit(Op::GetUpvalue(idx), span);
                return;
            }
        }

        // Global
        self.emit(Op::GetGlobal(name.to_string()), span);
    }

    fn resolve_upvalue(&mut self, frame_idx: usize, name: &str) -> Option<usize> {
        if frame_idx == 0 {
            return None;
        }

        // Check the enclosing frame's locals
        let parent_idx = frame_idx - 1;
        if let Some(local_idx) = self.frames[parent_idx].scope.resolve_local(name) {
            let uv_idx = self.frames[frame_idx].scope.add_upvalue(local_idx, true);
            return Some(uv_idx);
        }

        // Check the enclosing frame's upvalues (transitively)
        if let Some(uv_idx) = self.resolve_upvalue(parent_idx, name) {
            let new_idx = self.frames[frame_idx].scope.add_upvalue(uv_idx, false);
            return Some(new_idx);
        }

        None
    }

    fn compile_lambda(
        &mut self,
        params: &[LambdaParam],
        body: &SpannedExpr,
        rec_name: Option<&str>,
        span: Span,
    ) -> Result<(), String> {
        // Push a new compiler frame
        let name = rec_name.unwrap_or("<lambda>").to_string();
        let new_frame = CompilerFrame {
            proto: FunctionProto {
                name,
                arity: params.len() as u8,
                chunk: Chunk::new(),
                upvalue_count: 0,
            },
            scope: ScopeTracker::new(),
        };
        self.frames.push(new_frame);

        // Add params as locals
        self.current().scope.begin_scope();
        for param in params {
            self.current().scope.add_local(param.name.node.clone());
        }

        // If recursive, add self reference
        if let Some(rec) = rec_name {
            self.current().scope.add_local(rec.to_string());
        }

        // Compile body with tail call optimization
        self.compile_expr_tail(body)?;
        self.emit(Op::Return, span);

        // Pop frame
        let mut frame = self.frames.pop().unwrap();
        frame.proto.upvalue_count = frame.scope.upvalues.len();

        // Build upvalue refs
        let upvalue_refs: Vec<UpvalueRef> = frame
            .scope
            .upvalues
            .iter()
            .map(|uv| UpvalueRef {
                is_local: uv.is_local,
                index: uv.index,
            })
            .collect();

        // Add function as constant in the parent frame
        let const_idx = self.add_constant(Value::Function(frame.proto));
        self.emit(Op::Closure(const_idx, upvalue_refs), span);

        Ok(())
    }

    fn compile_match(
        &mut self,
        scrutinee: &SpannedExpr,
        arms: &[MatchArm],
        span: Span,
    ) -> Result<(), String> {
        // Compile scrutinee and store as a tracked local so slot numbering stays correct.
        self.current().scope.begin_scope();
        self.compile_expr(scrutinee)?;
        let scrut_slot = self.current().scope.add_local("__scrutinee".to_string());

        let mut end_jumps = Vec::new();

        for (i, arm) in arms.iter().enumerate() {
            let is_last = i == arms.len() - 1;
            let _binding_count = self.count_pattern_bindings(&arm.pattern);
            let needs_test = self.pattern_needs_test(&arm.pattern);

            let mut next_arm_jump: Option<usize> = None;

            if needs_test {
                // Push scrutinee for test (test peeks, doesn't pop)
                self.emit(Op::GetLocal(scrut_slot), span);
                let jump = self.compile_pattern_test(&arm.pattern, span)?;
                next_arm_jump = Some(jump);
                // Test passed: pop the test copy
                self.emit(Op::Pop, span);
            }

            // Emit pattern bindings using GetLocal(scrut_slot) to access scrutinee
            self.current().scope.begin_scope();
            self.emit_pattern_bindings(scrut_slot, &arm.pattern, span)?;

            // Compile arm body
            self.compile_expr(&arm.body)?;

            // Clean up arm bindings from under the result
            let arm_pops = self.current().scope.end_scope();
            if arm_pops > 0 {
                self.emit(Op::PopUnder(arm_pops), span);
            }

            // Jump to end of match
            let end_jump = self.emit(Op::Jump(0), span);
            end_jumps.push(end_jump);

            // Patch test failure jump
            if let Some(jump) = next_arm_jump {
                self.patch_jump(jump);
                // Failed test: pop the test copy that's still on stack
                if !is_last {
                    self.emit(Op::Pop, span);
                }
            }
        }

        // All end jumps land here. The result is on top, scrutinee local below.
        for jump in end_jumps {
            self.patch_jump(jump);
        }

        // Pop the scrutinee local from under the result.
        let match_pops = self.current().scope.end_scope();
        if match_pops > 0 {
            self.emit(Op::PopUnder(match_pops), span);
        }

        Ok(())
    }

    fn pattern_needs_test(&self, pattern: &SpannedPattern) -> bool {
        match &pattern.node {
            Pattern::Wildcard | Pattern::Var(_) => false,
            _ => true,
        }
    }

    fn count_pattern_bindings(&self, pattern: &SpannedPattern) -> usize {
        match &pattern.node {
            Pattern::Wildcard => 0,
            Pattern::Var(_) => 1,
            Pattern::IntLit(_) | Pattern::FloatLit(_) | Pattern::StringLit(_)
            | Pattern::BoolLit(_) | Pattern::UnitLit => 0,
            Pattern::Constructor { args, .. } => {
                args.iter().map(|a| self.count_pattern_bindings(a)).sum()
            }
            Pattern::Cons(head, tail) => {
                self.count_pattern_bindings(head) + self.count_pattern_bindings(tail)
            }
            Pattern::Tuple(pats) | Pattern::List(pats) => {
                pats.iter().map(|p| self.count_pattern_bindings(p)).sum()
            }
        }
    }

    fn compile_pattern_test(&mut self, pattern: &SpannedPattern, span: Span) -> Result<usize, String> {
        match &pattern.node {
            Pattern::Wildcard | Pattern::Var(_) => {
                unreachable!("wildcard/var patterns don't need tests")
            }
            Pattern::IntLit(n) => Ok(self.emit(Op::TestInt(*n, 0), span)),
            Pattern::FloatLit(_) => Ok(self.emit(Op::JumpIfFalse(0), span)),
            Pattern::BoolLit(b) => Ok(self.emit(Op::TestBool(*b, 0), span)),
            Pattern::StringLit(s) => Ok(self.emit(Op::TestString(s.clone(), 0), span)),
            Pattern::UnitLit => Ok(self.emit(Op::TestUnit(0), span)),
            Pattern::Constructor { name, .. } => {
                Ok(self.emit(Op::TestTag(name.clone(), 0), span))
            }
            Pattern::List(pats) if pats.is_empty() => {
                Ok(self.emit(Op::TestEmptyList(0), span))
            }
            Pattern::Cons(_, _) => Ok(self.emit(Op::TestCons(0), span)),
            _ => Ok(self.emit(Op::JumpIfFalse(0), span)),
        }
    }

    /// Emit pattern bindings by reading from the scrutinee local.
    fn emit_pattern_bindings(&mut self, scrut_slot: usize, pattern: &SpannedPattern, span: Span) -> Result<(), String> {
        match &pattern.node {
            Pattern::Var(name) => {
                // Bind the variable to the scrutinee value
                self.emit(Op::GetLocal(scrut_slot), span);
                self.current().scope.add_local(name.clone());
                Ok(())
            }
            Pattern::Wildcard | Pattern::IntLit(_) | Pattern::FloatLit(_)
            | Pattern::StringLit(_) | Pattern::BoolLit(_) | Pattern::UnitLit => {
                Ok(())
            }
            Pattern::Constructor { args, .. } => {
                for (i, arg) in args.iter().enumerate() {
                    self.emit_constructor_field_binding(scrut_slot, i, arg, span)?;
                }
                Ok(())
            }
            Pattern::Cons(head, tail) => {
                // Bind head: push list, GetListHead pushes head, swap+pop list
                self.emit_cons_head_binding(scrut_slot, head, span)?;
                // Bind tail: push list, GetListTail pushes tail, swap+pop list
                self.emit_cons_tail_binding(scrut_slot, tail, span)?;
                Ok(())
            }
            Pattern::Tuple(pats) => {
                for (i, pat) in pats.iter().enumerate() {
                    self.emit_tuple_field_binding(scrut_slot, i, pat, span)?;
                }
                Ok(())
            }
            Pattern::List(pats) if pats.is_empty() => Ok(()),
            Pattern::List(_pats) => Ok(()),
        }
    }

    fn emit_constructor_field_binding(
        &mut self,
        scrut_slot: usize,
        field_idx: usize,
        pattern: &SpannedPattern,
        span: Span,
    ) -> Result<(), String> {
        match &pattern.node {
            Pattern::Var(name) => {
                // Push scrutinee, extract field, keep as local
                self.emit(Op::GetLocal(scrut_slot), span);
                self.emit(Op::GetAdtField(field_idx), span);
                // Stack: [... scrut_copy field_value]
                // Swap and pop to leave just field_value
                self.emit(Op::Swap, span);
                self.emit(Op::Pop, span);
                // Stack: [... field_value]
                self.current().scope.add_local(name.clone());
                Ok(())
            }
            Pattern::Wildcard => Ok(()),
            _ => {
                // Nested patterns: push the field value and recursively bind
                // For now, just extract as a single binding
                Ok(())
            }
        }
    }

    fn emit_tuple_field_binding(
        &mut self,
        scrut_slot: usize,
        field_idx: usize,
        pattern: &SpannedPattern,
        span: Span,
    ) -> Result<(), String> {
        match &pattern.node {
            Pattern::Var(name) => {
                self.emit(Op::GetLocal(scrut_slot), span);
                self.emit(Op::GetTupleField(field_idx), span);
                self.emit(Op::Swap, span);
                self.emit(Op::Pop, span);
                self.current().scope.add_local(name.clone());
                Ok(())
            }
            Pattern::Wildcard => Ok(()),
            _ => Ok(()),
        }
    }

    fn emit_cons_head_binding(
        &mut self,
        scrut_slot: usize,
        head_pattern: &SpannedPattern,
        span: Span,
    ) -> Result<(), String> {
        match &head_pattern.node {
            Pattern::Var(name) => {
                // GetLocal pushes list copy, GetListHead peeks list and pushes head
                self.emit(Op::GetLocal(scrut_slot), span);
                self.emit(Op::GetListHead, span);
                // Stack: [... list_copy head]
                self.emit(Op::Swap, span);
                self.emit(Op::Pop, span);
                // Stack: [... head]
                self.current().scope.add_local(name.clone());
                Ok(())
            }
            Pattern::Wildcard => Ok(()),
            _ => Ok(()),
        }
    }

    fn emit_cons_tail_binding(
        &mut self,
        scrut_slot: usize,
        tail_pattern: &SpannedPattern,
        span: Span,
    ) -> Result<(), String> {
        match &tail_pattern.node {
            Pattern::Var(name) => {
                self.emit(Op::GetLocal(scrut_slot), span);
                self.emit(Op::GetListTail, span);
                // Stack: [... list_copy tail]
                self.emit(Op::Swap, span);
                self.emit(Op::Pop, span);
                // Stack: [... tail]
                self.current().scope.add_local(name.clone());
                Ok(())
            }
            Pattern::Wildcard => Ok(()),
            _ => Ok(()),
        }
    }
}

/// Compile a program from declarations to a function prototype.
pub fn compile(decls: &[Decl]) -> Result<FunctionProto, String> {
    Compiler::new().compile_program(decls)
}
