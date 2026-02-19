pub mod frame;

use std::collections::HashMap;

use crate::compiler::bytecode::{FunctionProto, Op};
use crate::eval::value::Value;
use crate::eval::{apply_function};
use crate::span::Span;
use crate::error::LyraError;

use frame::CallFrame;

const MAX_FRAMES: usize = 256;
#[allow(dead_code)]
const MAX_STACK: usize = 65536;

pub struct VM {
    stack: Vec<Value>,
    frames: Vec<CallFrame>,
    globals: HashMap<String, Value>,
}

impl VM {
    pub fn new() -> Self {
        VM {
            stack: Vec::with_capacity(256),
            frames: Vec::with_capacity(64),
            globals: HashMap::new(),
        }
    }

    pub fn define_global(&mut self, name: String, value: Value) {
        self.globals.insert(name, value);
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().expect("stack underflow")
    }

    fn peek(&self) -> &Value {
        self.stack.last().expect("stack underflow")
    }

    fn frame(&self) -> &CallFrame {
        self.frames.last().expect("no call frame")
    }

    fn frame_mut(&mut self) -> &mut CallFrame {
        self.frames.last_mut().expect("no call frame")
    }

    fn read_op(&mut self) -> Op {
        let frame = self.frames.last_mut().unwrap();
        let op = frame.function.chunk.code[frame.ip].clone();
        frame.ip += 1;
        op
    }

    fn current_span(&self) -> Span {
        let frame = self.frame();
        if frame.ip > 0 && frame.ip - 1 < frame.function.chunk.spans.len() {
            frame.function.chunk.spans[frame.ip - 1]
        } else {
            Span::default()
        }
    }

    /// Execute a single compiled function with arguments (used by apply_function for VM interop).
    pub fn call_function(&mut self, proto: FunctionProto, args: Vec<Value>) -> Result<Value, LyraError> {
        let stack_base = self.stack.len();
        for arg in args {
            self.push(arg);
        }
        let frame = CallFrame::new(proto, stack_base, vec![]);
        self.frames.push(frame);
        self.execute()
    }

    /// Execute a closure with captured upvalues and arguments (used by apply_function for VM interop).
    pub fn call_closure(&mut self, proto: FunctionProto, upvalues: Vec<Value>, args: Vec<Value>) -> Result<Value, LyraError> {
        let stack_base = self.stack.len();
        for arg in args {
            self.push(arg);
        }
        let frame = CallFrame::new(proto, stack_base, upvalues);
        self.frames.push(frame);
        self.execute()
    }

    /// Execute a compiled function prototype.
    pub fn run(&mut self, main: FunctionProto) -> Result<Value, LyraError> {
        let main_frame = CallFrame::new(main, 0, vec![]);
        self.frames.push(main_frame);
        self.execute()
    }

    fn execute(&mut self) -> Result<Value, LyraError> {
        loop {
            if self.frames.is_empty() {
                return Ok(self.stack.pop().unwrap_or(Value::Unit));
            }

            let (frame_done, frame_base) = {
                let frame = self.frames.last().unwrap();
                (frame.ip >= frame.function.chunk.code.len(), frame.stack_base)
            };
            if frame_done {
                // End of function
                let result = self.pop();
                let base = frame_base;
                self.frames.pop();
                self.stack.truncate(base);
                if self.frames.is_empty() {
                    return Ok(result);
                }
                self.push(result);
                continue;
            }

            let op = self.read_op();

            match op {
                Op::Constant(idx) => {
                    let val = self.frame().function.chunk.constants[idx].clone();
                    self.push(val);
                }
                Op::Unit => self.push(Value::Unit),
                Op::True => self.push(Value::Bool(true)),
                Op::False => self.push(Value::Bool(false)),
                Op::Pop => {
                    self.pop();
                }
                Op::Dup => {
                    let val = self.peek().clone();
                    self.push(val);
                }

                // ── Variables ──
                Op::GetLocal(slot) => {
                    let base = self.frame().stack_base;
                    let val = self.stack[base + slot].clone();
                    self.push(val);
                }
                Op::SetLocal(slot) => {
                    let base = self.frame().stack_base;
                    let val = self.peek().clone();
                    self.stack[base + slot] = val;
                }
                Op::GetUpvalue(idx) => {
                    let val = self.frame().upvalues[idx].clone();
                    self.push(val);
                }
                Op::GetGlobal(name) => {
                    let val = self.globals.get(&name).cloned().ok_or_else(|| {
                        let candidates: Vec<&str> =
                            self.globals.keys().map(|s| s.as_str()).collect();
                        LyraError::UndefinedVariable {
                            suggestion: crate::error::suggest_similar(&name, &candidates),
                            name: name.clone(),
                            span: self.current_span(),
                        }
                    })?;
                    self.push(val);
                }
                Op::DefineGlobal(name) => {
                    let val = self.pop();
                    self.globals.insert(name, val);
                }

                // ── Arithmetic ──
                Op::Add => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = match (&a, &b) {
                        (Value::Int(x), Value::Int(y)) => Value::Int(x + y),
                        (Value::Float(x), Value::Float(y)) => Value::Float(x + y),
                        (Value::String(x), Value::String(y)) => {
                            Value::String(format!("{}{}", x, y))
                        }
                        _ => {
                            return Err(LyraError::RuntimeError {
                                message: format!(
                                    "cannot add {} and {}",
                                    a.type_name(),
                                    b.type_name()
                                ),
                                span: self.current_span(),
                            })
                        }
                    };
                    self.push(result);
                }
                Op::Sub => self.binary_arith(|a, b| a - b, |a, b| a - b)?,
                Op::Mul => self.binary_arith(|a, b| a * b, |a, b| a * b)?,
                Op::Div => {
                    let b = self.pop();
                    let a = self.pop();
                    match (&a, &b) {
                        (Value::Int(_), Value::Int(0)) => {
                            return Err(LyraError::DivisionByZero {
                                span: self.current_span(),
                            })
                        }
                        (Value::Int(x), Value::Int(y)) => self.push(Value::Int(x / y)),
                        (Value::Float(x), Value::Float(y)) => self.push(Value::Float(x / y)),
                        _ => {
                            return Err(LyraError::RuntimeError {
                                message: "invalid division operands".to_string(),
                                span: self.current_span(),
                            })
                        }
                    }
                }
                Op::Mod => {
                    let b = self.pop();
                    let a = self.pop();
                    match (&a, &b) {
                        (Value::Int(_), Value::Int(0)) => {
                            return Err(LyraError::DivisionByZero {
                                span: self.current_span(),
                            })
                        }
                        (Value::Int(x), Value::Int(y)) => self.push(Value::Int(x % y)),
                        (Value::Float(x), Value::Float(y)) => self.push(Value::Float(x % y)),
                        _ => {
                            return Err(LyraError::RuntimeError {
                                message: "invalid mod operands".to_string(),
                                span: self.current_span(),
                            })
                        }
                    }
                }
                Op::Negate => {
                    let val = self.pop();
                    match val {
                        Value::Int(n) => self.push(Value::Int(-n)),
                        Value::Float(n) => self.push(Value::Float(-n)),
                        _ => {
                            return Err(LyraError::RuntimeError {
                                message: "cannot negate non-number".to_string(),
                                span: self.current_span(),
                            })
                        }
                    }
                }

                // ── Comparison ──
                Op::Equal => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(a == b));
                }
                Op::NotEqual => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(a != b));
                }
                Op::Less => self.binary_cmp(|a, b| a < b, |a, b| a < b)?,
                Op::Greater => self.binary_cmp(|a, b| a > b, |a, b| a > b)?,
                Op::LessEqual => self.binary_cmp(|a, b| a <= b, |a, b| a <= b)?,
                Op::GreaterEqual => self.binary_cmp(|a, b| a >= b, |a, b| a >= b)?,

                // ── Logic ──
                Op::Not => {
                    let val = self.pop();
                    match val {
                        Value::Bool(b) => self.push(Value::Bool(!b)),
                        _ => {
                            return Err(LyraError::RuntimeError {
                                message: "cannot negate non-boolean".to_string(),
                                span: self.current_span(),
                            })
                        }
                    }
                }

                // ── Control flow ──
                Op::Jump(offset) => {
                    self.frame_mut().ip += offset;
                }
                Op::JumpIfFalse(offset) => {
                    if let Value::Bool(false) = self.peek() {
                        self.frame_mut().ip += offset;
                    }
                }
                Op::Loop(offset) => {
                    self.frame_mut().ip -= offset;
                }

                // ── Functions ──
                Op::Call(arg_count) => {
                    let argc = arg_count as usize;
                    let func_idx = self.stack.len() - argc - 1;
                    let func = self.stack[func_idx].clone();

                    match func {
                        Value::Function(proto) => {
                            if self.frames.len() >= MAX_FRAMES {
                                return Err(LyraError::RuntimeError {
                                    message: "stack overflow".to_string(),
                                    span: self.current_span(),
                                });
                            }
                            let frame = CallFrame::new(proto, func_idx + 1, vec![]);
                            self.frames.push(frame);
                        }
                        Value::ClosureVal { proto, upvalues } => {
                            if self.frames.len() >= MAX_FRAMES {
                                return Err(LyraError::RuntimeError {
                                    message: "stack overflow".to_string(),
                                    span: self.current_span(),
                                });
                            }
                            let frame = CallFrame::new(proto, func_idx + 1, upvalues);
                            self.frames.push(frame);
                        }
                        // Fall back to tree-walking for builtins and partial app
                        Value::Builtin { .. } | Value::Closure { .. } | Value::PartialApp { .. } => {
                            let args: Vec<Value> =
                                self.stack.drain(func_idx + 1..).collect();
                            self.stack.pop(); // pop the function
                            // Save globals so callbacks can access them via mini-VM
                            crate::eval::set_vm_globals(self.globals.clone());
                            let result =
                                apply_function(func, args, self.current_span())?;
                            self.push(result);
                        }
                        _ => {
                            return Err(LyraError::NotCallable {
                                span: self.current_span(),
                            })
                        }
                    }
                }

                Op::TailCall(arg_count) => {
                    let argc = arg_count as usize;
                    let func_idx = self.stack.len() - argc - 1;
                    let func = self.stack[func_idx].clone();

                    match func {
                        Value::Function(proto) | Value::ClosureVal { proto, .. } => {
                            // Move args to the current frame's base
                            let base = self.frame().stack_base;
                            let args: Vec<Value> =
                                self.stack.drain(func_idx + 1..).collect();
                            self.stack.truncate(base);
                            for arg in args {
                                self.push(arg);
                            }
                            // Reuse frame
                            let frame = self.frame_mut();
                            frame.function = proto;
                            frame.ip = 0;
                        }
                        _ => {
                            // Fall back to regular call
                            let args: Vec<Value> =
                                self.stack.drain(func_idx + 1..).collect();
                            self.stack.pop();
                            crate::eval::set_vm_globals(self.globals.clone());
                            let result =
                                apply_function(func, args, self.current_span())?;
                            self.push(result);
                        }
                    }
                }

                Op::Return => {
                    let result = self.pop();
                    let base = self.frame().stack_base;
                    self.frames.pop();
                    self.stack.truncate(base.saturating_sub(1).max(0)); // pop function + locals
                    if self.frames.is_empty() {
                        return Ok(result);
                    }
                    self.push(result);
                }

                Op::Closure(const_idx, upvalue_refs) => {
                    let proto = match self.frame().function.chunk.constants[const_idx].clone() {
                        Value::Function(p) => p,
                        _ => panic!("closure constant is not a function"),
                    };

                    let mut upvalues = Vec::new();
                    for uv_ref in &upvalue_refs {
                        if uv_ref.is_local {
                            let base = self.frame().stack_base;
                            upvalues.push(self.stack[base + uv_ref.index].clone());
                        } else {
                            upvalues.push(self.frame().upvalues[uv_ref.index].clone());
                        }
                    }

                    if upvalues.is_empty() {
                        self.push(Value::Function(proto));
                    } else {
                        self.push(Value::ClosureVal { proto, upvalues });
                    }
                }

                // ── Data structures ──
                Op::MakeList(n) => {
                    let start = self.stack.len() - n;
                    let items: Vec<Value> = self.stack.drain(start..).collect();
                    self.push(Value::List(items));
                }
                Op::MakeTuple(n) => {
                    let start = self.stack.len() - n;
                    let items: Vec<Value> = self.stack.drain(start..).collect();
                    self.push(Value::Tuple(items));
                }
                Op::MakeAdt(tag, n) => {
                    let start = self.stack.len() - n;
                    let fields: Vec<Value> = self.stack.drain(start..).collect();
                    self.push(Value::Adt {
                        constructor: tag,
                        fields,
                    });
                }
                Op::Cons => {
                    let tail = self.pop();
                    let head = self.pop();
                    match tail {
                        Value::List(mut list) => {
                            list.insert(0, head);
                            self.push(Value::List(list));
                        }
                        _ => {
                            return Err(LyraError::RuntimeError {
                                message: ":: requires a list on the right".to_string(),
                                span: self.current_span(),
                            })
                        }
                    }
                }

                // ── Pattern matching helpers ──
                Op::TestTag(tag, offset) => {
                    if let Value::Adt { constructor, .. } = self.peek() {
                        if constructor != &tag {
                            self.frame_mut().ip += offset;
                        }
                    } else {
                        self.frame_mut().ip += offset;
                    }
                }
                Op::TestInt(n, offset) => {
                    if let Value::Int(v) = self.peek() {
                        if *v != n {
                            self.frame_mut().ip += offset;
                        }
                    } else {
                        self.frame_mut().ip += offset;
                    }
                }
                Op::TestBool(b, offset) => {
                    if let Value::Bool(v) = self.peek() {
                        if *v != b {
                            self.frame_mut().ip += offset;
                        }
                    } else {
                        self.frame_mut().ip += offset;
                    }
                }
                Op::TestString(s, offset) => {
                    if let Value::String(v) = self.peek() {
                        if v != &s {
                            self.frame_mut().ip += offset;
                        }
                    } else {
                        self.frame_mut().ip += offset;
                    }
                }
                Op::TestUnit(offset) => {
                    if !matches!(self.peek(), Value::Unit) {
                        self.frame_mut().ip += offset;
                    }
                }
                Op::TestEmptyList(offset) => {
                    if let Value::List(l) = self.peek() {
                        if !l.is_empty() {
                            self.frame_mut().ip += offset;
                        }
                    } else {
                        self.frame_mut().ip += offset;
                    }
                }
                Op::TestCons(offset) => {
                    if let Value::List(l) = self.peek() {
                        if l.is_empty() {
                            self.frame_mut().ip += offset;
                        }
                    } else {
                        self.frame_mut().ip += offset;
                    }
                }
                Op::TestTuple(n, offset) => {
                    if let Value::Tuple(t) = self.peek() {
                        if t.len() != n {
                            self.frame_mut().ip += offset;
                        }
                    } else {
                        self.frame_mut().ip += offset;
                    }
                }
                Op::GetAdtField(idx) => {
                    let val = self.peek().clone();
                    if let Value::Adt { fields, .. } = val {
                        self.push(fields[idx].clone());
                    }
                }
                Op::GetListHead => {
                    let val = self.peek().clone();
                    if let Value::List(l) = val {
                        self.push(l[0].clone());
                    }
                }
                Op::GetListTail => {
                    let val = self.peek().clone();
                    if let Value::List(l) = val {
                        self.push(Value::List(l[1..].to_vec()));
                    }
                }
                Op::GetTupleField(idx) => {
                    let val = self.peek().clone();
                    if let Value::Tuple(t) = val {
                        self.push(t[idx].clone());
                    }
                }
                Op::PopMatch => {
                    self.pop();
                }
                Op::Swap => {
                    let len = self.stack.len();
                    self.stack.swap(len - 1, len - 2);
                }
                Op::PopUnder(n) => {
                    let top = self.pop();
                    for _ in 0..n {
                        self.pop();
                    }
                    self.push(top);
                }

                // ── Records ──
                Op::MakeRecord(names) => {
                    let start = self.stack.len() - names.len();
                    let values: Vec<Value> = self.stack.drain(start..).collect();
                    let mut map = std::collections::BTreeMap::new();
                    for (name, val) in names.into_iter().zip(values) {
                        map.insert(name, val);
                    }
                    self.push(Value::Record(map));
                }
                Op::GetField(name) => {
                    let val = self.pop();
                    if let Value::Record(map) = val {
                        if let Some(field_val) = map.get(&name) {
                            self.push(field_val.clone());
                        } else {
                            return Err(LyraError::RuntimeError {
                                message: format!("no field '{}' in record", name),
                                span: self.current_span(),
                            });
                        }
                    } else {
                        return Err(LyraError::RuntimeError {
                            message: "field access on non-record".to_string(),
                            span: self.current_span(),
                        });
                    }
                }

                // ── String ops ──
                Op::ToString => {
                    let val = self.pop();
                    self.push(Value::String(val.display_unquoted()));
                }
                Op::StringConcat => {
                    let b = self.pop();
                    let a = self.pop();
                    match (a, b) {
                        (Value::String(a), Value::String(b)) => {
                            self.push(Value::String(format!("{}{}", a, b)));
                        }
                        _ => {
                            return Err(LyraError::RuntimeError {
                                message: "string concat requires strings".to_string(),
                                span: self.current_span(),
                            })
                        }
                    }
                }

                Op::Print => {
                    let val = self.pop();
                    match &val {
                        Value::String(s) => println!("{}", s),
                        v => println!("{}", v),
                    }
                    self.push(Value::Unit);
                }
                Op::PrintRaw => {
                    let val = self.pop();
                    match &val {
                        Value::String(s) => print!("{}", s),
                        v => print!("{}", v),
                    }
                    self.push(Value::Unit);
                }
            }
        }
    }

    fn binary_arith(
        &mut self,
        int_op: fn(i64, i64) -> i64,
        float_op: fn(f64, f64) -> f64,
    ) -> Result<(), LyraError> {
        let b = self.pop();
        let a = self.pop();
        match (&a, &b) {
            (Value::Int(x), Value::Int(y)) => self.push(Value::Int(int_op(*x, *y))),
            (Value::Float(x), Value::Float(y)) => self.push(Value::Float(float_op(*x, *y))),
            _ => {
                return Err(LyraError::RuntimeError {
                    message: format!(
                        "arithmetic on {} and {}",
                        a.type_name(),
                        b.type_name()
                    ),
                    span: self.current_span(),
                })
            }
        }
        Ok(())
    }

    fn binary_cmp(
        &mut self,
        int_op: fn(&i64, &i64) -> bool,
        float_op: fn(&f64, &f64) -> bool,
    ) -> Result<(), LyraError> {
        let b = self.pop();
        let a = self.pop();
        match (&a, &b) {
            (Value::Int(x), Value::Int(y)) => self.push(Value::Bool(int_op(x, y))),
            (Value::Float(x), Value::Float(y)) => self.push(Value::Bool(float_op(x, y))),
            _ => {
                return Err(LyraError::RuntimeError {
                    message: format!(
                        "comparison on {} and {}",
                        a.type_name(),
                        b.type_name()
                    ),
                    span: self.current_span(),
                })
            }
        }
        Ok(())
    }
}
