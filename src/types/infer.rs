use std::collections::HashMap;

use crate::ast::*;
use crate::error::LyraError;
use crate::span::Span;

use super::env::TypeEnv;
use super::subst::Subst;
use super::unify::unify;
use super::{MonoType, TypeScheme, TypeVar, TypeVarGen};

pub struct Inferencer {
    gen: TypeVarGen,
    /// Maps constructor names to (type_name, type_params, field_types)
    constructors: HashMap<String, ConstructorInfo>,
}

#[derive(Debug, Clone)]
pub struct ConstructorInfo {
    pub type_name: String,
    pub type_params: Vec<String>,
    pub field_types: Vec<MonoType>,
}

impl Inferencer {
    pub fn new() -> Self {
        Inferencer {
            gen: TypeVarGen::new(),
            constructors: HashMap::new(),
        }
    }

    /// Instantiate a type scheme with fresh type variables.
    fn instantiate(&mut self, scheme: &TypeScheme) -> MonoType {
        let fresh_map: HashMap<TypeVar, MonoType> = scheme
            .vars
            .iter()
            .map(|&v| (v, self.gen.fresh_type()))
            .collect();
        let subst = Subst { map: fresh_map };
        subst.apply(&scheme.ty)
    }

    /// Generalize a type over variables not free in the environment.
    fn generalize(env: &TypeEnv, ty: &MonoType) -> TypeScheme {
        let env_free = env.free_vars();
        let ty_free = ty.free_vars();
        let quantified: Vec<TypeVar> = ty_free.difference(&env_free).copied().collect();
        TypeScheme {
            vars: quantified,
            ty: ty.clone(),
        }
    }

    /// Register constructors from a type declaration.
    pub fn register_type_decl(
        &mut self,
        env: &mut TypeEnv,
        decl: &Decl,
    ) -> Result<(), LyraError> {
        if let Decl::Type {
            name,
            type_params,
            variants,
        } = decl
        {
            // Create a mapping from type param names to type variables
            let param_vars: Vec<(String, TypeVar)> = type_params
                .iter()
                .map(|p| (p.node.clone(), self.gen.fresh()))
                .collect();

            let result_type = if param_vars.is_empty() {
                MonoType::Con(name.node.clone(), vec![])
            } else {
                MonoType::Con(
                    name.node.clone(),
                    param_vars.iter().map(|(_, v)| MonoType::Var(*v)).collect(),
                )
            };

            for variant in variants {
                let field_types: Vec<MonoType> = variant
                    .fields
                    .iter()
                    .map(|f| self.type_ann_to_mono(f, &param_vars))
                    .collect();

                // Constructor type: Field1 -> Field2 -> ... -> ResultType
                let ctor_type = if field_types.is_empty() {
                    result_type.clone()
                } else {
                    MonoType::curried_arrow(field_types.clone(), result_type.clone())
                };

                let scheme = TypeScheme {
                    vars: param_vars.iter().map(|(_, v)| *v).collect(),
                    ty: ctor_type,
                };

                env.insert(variant.name.node.clone(), scheme);

                self.constructors.insert(
                    variant.name.node.clone(),
                    ConstructorInfo {
                        type_name: name.node.clone(),
                        type_params: type_params.iter().map(|p| p.node.clone()).collect(),
                        field_types,
                    },
                );
            }
        }
        Ok(())
    }

    fn type_ann_to_mono(
        &mut self,
        ann: &SpannedTypeAnn,
        params: &[(String, TypeVar)],
    ) -> MonoType {
        match &ann.node {
            TypeAnnotation::Named(name) => match name.as_str() {
                "Int" => MonoType::Int,
                "Float" => MonoType::Float,
                "Bool" => MonoType::Bool,
                "String" => MonoType::String,
                _ => MonoType::Con(name.clone(), vec![]),
            },
            TypeAnnotation::Var(name) => {
                if let Some((_, tv)) = params.iter().find(|(n, _)| n == name) {
                    MonoType::Var(*tv)
                } else {
                    MonoType::Var(self.gen.fresh())
                }
            }
            TypeAnnotation::Arrow(from, to) => MonoType::Arrow(
                Box::new(self.type_ann_to_mono(from, params)),
                Box::new(self.type_ann_to_mono(to, params)),
            ),
            TypeAnnotation::List(inner) => {
                MonoType::List(Box::new(self.type_ann_to_mono(inner, params)))
            }
            TypeAnnotation::Tuple(elems) => {
                MonoType::Tuple(elems.iter().map(|e| self.type_ann_to_mono(e, params)).collect())
            }
            TypeAnnotation::App(base, args) => {
                let base_mono = self.type_ann_to_mono(base, params);
                if let MonoType::Con(name, _) = base_mono {
                    MonoType::Con(
                        name,
                        args.iter().map(|a| self.type_ann_to_mono(a, params)).collect(),
                    )
                } else {
                    base_mono
                }
            }
            TypeAnnotation::Unit => MonoType::Unit,
        }
    }

    /// Infer the type of an expression. Returns (substitution, type).
    pub fn infer(
        &mut self,
        env: &TypeEnv,
        expr: &SpannedExpr,
    ) -> Result<(Subst, MonoType), LyraError> {
        match &expr.node {
            // ── Literals ──
            Expr::IntLit(_) => Ok((Subst::new(), MonoType::Int)),
            Expr::FloatLit(_) => Ok((Subst::new(), MonoType::Float)),
            Expr::BoolLit(_) => Ok((Subst::new(), MonoType::Bool)),
            Expr::StringLit(_) => Ok((Subst::new(), MonoType::String)),
            Expr::UnitLit => Ok((Subst::new(), MonoType::Unit)),

            // ── Variable ──
            Expr::Var(name) => match env.lookup(name) {
                Some(scheme) => {
                    let ty = self.instantiate(scheme);
                    Ok((Subst::new(), ty))
                }
                None => {
                    let suggestion =
                        crate::error::suggest_similar(name, &env.names());
                    Err(LyraError::UndefinedVariable {
                        suggestion,
                        name: name.clone(),
                        span: expr.span,
                    })
                }
            },

            // ── List literal ──
            Expr::ListLit(elems) => {
                if elems.is_empty() {
                    let tv = self.gen.fresh_type();
                    Ok((Subst::new(), MonoType::List(Box::new(tv))))
                } else {
                    let (mut subst, first_ty) = self.infer(env, &elems[0])?;
                    for elem in &elems[1..] {
                        let env2 = env.apply_subst(&subst);
                        let (s, ty) = self.infer(&env2, elem)?;
                        subst = s.compose(&subst);
                        let s_u =
                            unify(&subst.apply(&first_ty), &subst.apply(&ty), elem.span)?;
                        subst = s_u.compose(&subst);
                    }
                    Ok((
                        subst.clone(),
                        MonoType::List(Box::new(subst.apply(&first_ty))),
                    ))
                }
            }

            // ── Tuple literal ──
            Expr::TupleLit(elems) => {
                let mut subst = Subst::new();
                let mut types = Vec::new();
                for elem in elems {
                    let env2 = env.apply_subst(&subst);
                    let (s, ty) = self.infer(&env2, elem)?;
                    subst = s.compose(&subst);
                    types.push(subst.apply(&ty));
                }
                Ok((subst, MonoType::Tuple(types)))
            }

            // ── Lambda ──
            Expr::Lambda { params, body } => {
                let param_types: Vec<MonoType> =
                    params.iter().map(|_| self.gen.fresh_type()).collect();

                let mut new_env = env.clone();
                for (param, ty) in params.iter().zip(&param_types) {
                    new_env.insert(param.name.node.clone(), TypeScheme::mono(ty.clone()));
                }

                let (s, body_ty) = self.infer(&new_env, body)?;

                let fn_type = param_types
                    .into_iter()
                    .rev()
                    .fold(body_ty, |acc, param_ty| {
                        MonoType::Arrow(Box::new(s.apply(&param_ty)), Box::new(acc))
                    });

                Ok((s, fn_type))
            }

            // ── Application ──
            Expr::App { func, args } => {
                let (s1, fn_ty) = self.infer(env, func)?;
                let mut subst = s1;
                let mut current_fn_ty = fn_ty;

                for arg in args {
                    let env2 = env.apply_subst(&subst);
                    let (s2, arg_ty) = self.infer(&env2, arg)?;
                    subst = s2.compose(&subst);

                    let ret_ty = self.gen.fresh_type();
                    let expected_fn = MonoType::Arrow(
                        Box::new(subst.apply(&arg_ty)),
                        Box::new(ret_ty.clone()),
                    );
                    let s3 = unify(
                        &subst.apply(&current_fn_ty),
                        &expected_fn,
                        expr.span,
                    )?;
                    subst = s3.compose(&subst);
                    current_fn_ty = subst.apply(&ret_ty);
                }

                Ok((subst, current_fn_ty))
            }

            // ── Binary operation ──
            Expr::BinOp { op, lhs, rhs } => self.infer_binop(env, op, lhs, rhs, expr.span),

            // ── Unary operation ──
            Expr::UnaryOp { op, operand } => {
                let (s, ty) = self.infer(env, operand)?;
                match op {
                    UnaryOp::Neg => {
                        // Allow neg on Int or Float
                        let s2 = unify(&ty, &MonoType::Int, expr.span)
                            .or_else(|_| unify(&ty, &MonoType::Float, expr.span))?;
                        let s = s2.compose(&s);
                        Ok((s.clone(), s.apply(&ty)))
                    }
                    UnaryOp::Not => {
                        let s2 = unify(&ty, &MonoType::Bool, expr.span)?;
                        let s = s2.compose(&s);
                        Ok((s, MonoType::Bool))
                    }
                }
            }

            // ── Pipe ──
            Expr::Pipe { lhs, rhs } => {
                let (s1, lhs_ty) = self.infer(env, lhs)?;
                let env2 = env.apply_subst(&s1);
                let (s2, rhs_ty) = self.infer(&env2, rhs)?;
                let subst = s2.compose(&s1);

                let ret_ty = self.gen.fresh_type();
                let expected_fn = MonoType::Arrow(
                    Box::new(subst.apply(&lhs_ty)),
                    Box::new(ret_ty.clone()),
                );
                let s3 = unify(&subst.apply(&rhs_ty), &expected_fn, expr.span)?;
                let s = s3.compose(&subst);
                Ok((s.clone(), s.apply(&ret_ty)))
            }

            // ── If expression ──
            Expr::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let (s1, cond_ty) = self.infer(env, cond)?;
                let s2 = unify(&cond_ty, &MonoType::Bool, cond.span)?;
                let mut s = s2.compose(&s1);

                let (s3, then_ty) = self.infer(&env.apply_subst(&s), then_branch)?;
                s = s3.compose(&s);
                let (s4, else_ty) = self.infer(&env.apply_subst(&s), else_branch)?;
                s = s4.compose(&s);

                let s5 = unify(&s.apply(&then_ty), &s.apply(&else_ty), expr.span)?;
                s = s5.compose(&s);
                Ok((s.clone(), s.apply(&then_ty)))
            }

            // ── Let expression ──
            Expr::Let {
                name,
                recursive,
                value,
                body,
                ..
            } => {
                if *recursive {
                    let fresh = self.gen.fresh_type();
                    let mut rec_env = env.clone();
                    rec_env.insert(name.node.clone(), TypeScheme::mono(fresh.clone()));

                    let (s1, bind_ty) = self.infer(&rec_env, value)?;
                    let s2 = unify(&s1.apply(&fresh), &bind_ty, expr.span)?;
                    let combined = s2.compose(&s1);

                    let generalized_ty = combined.apply(&bind_ty);
                    let scheme =
                        Self::generalize(&env.apply_subst(&combined), &generalized_ty);

                    let mut body_env = env.apply_subst(&combined);
                    body_env.insert(name.node.clone(), scheme);
                    let (s3, body_ty) = self.infer(&body_env, body)?;
                    Ok((s3.compose(&combined), body_ty))
                } else {
                    let (s1, bind_ty) = self.infer(env, value)?;
                    let scheme = Self::generalize(&env.apply_subst(&s1), &bind_ty);

                    let mut body_env = env.apply_subst(&s1);
                    body_env.insert(name.node.clone(), scheme);
                    let (s2, body_ty) = self.infer(&body_env, body)?;
                    Ok((s2.compose(&s1), body_ty))
                }
            }

            // ── Match expression ──
            Expr::Match { scrutinee, arms } => {
                let (s1, scrut_ty) = self.infer(env, scrutinee)?;
                let result_ty = self.gen.fresh_type();
                let mut subst = s1;

                for arm in arms {
                    let (s_pat, bindings) = self.infer_pattern(
                        &env.apply_subst(&subst),
                        &arm.pattern,
                        &subst.apply(&scrut_ty),
                    )?;
                    subst = s_pat.compose(&subst);

                    let mut arm_env = env.apply_subst(&subst);
                    for (name, ty) in bindings {
                        arm_env.insert(name, TypeScheme::mono(ty));
                    }

                    let (s_body, body_ty) = self.infer(&arm_env, &arm.body)?;
                    subst = s_body.compose(&subst);

                    let s_unify = unify(
                        &subst.apply(&result_ty),
                        &subst.apply(&body_ty),
                        arm.body.span,
                    )?;
                    subst = s_unify.compose(&subst);
                }

                // Check exhaustiveness (emit warning, not error)
                let final_scrut_ty = subst.apply(&scrut_ty);
                let pattern_refs: Vec<_> = arms.iter().map(|a| &a.pattern).collect();
                let missing = super::exhaustiveness::check_exhaustiveness(
                    &pattern_refs,
                    &final_scrut_ty,
                    &self.constructors,
                );
                if !missing.is_empty() {
                    eprintln!(
                        "\x1b[1;33mwarning\x1b[0m: non-exhaustive patterns: missing {}",
                        missing.join(", ")
                    );
                }

                Ok((subst.clone(), subst.apply(&result_ty)))
            }

            // ── String interpolation ──
            Expr::Interpolation(parts) => {
                let mut subst = Subst::new();
                for part in parts {
                    if let InterpolationPart::Expr(e) = part {
                        let (s, _ty) = self.infer(&env.apply_subst(&subst), e)?;
                        subst = s.compose(&subst);
                    }
                }
                Ok((subst, MonoType::String))
            }

            // ── Record literal ──
            Expr::Record(fields) => {
                let mut subst = Subst::new();
                let mut field_types = std::collections::BTreeMap::new();
                for (name, val_expr) in fields {
                    let (s, ty) = self.infer(&env.apply_subst(&subst), val_expr)?;
                    subst = s.compose(&subst);
                    field_types.insert(name.clone(), subst.apply(&ty));
                }
                Ok((subst, MonoType::Record(field_types)))
            }

            // ── Field access ──
            Expr::FieldAccess { expr: obj, field } => {
                let (s1, obj_ty) = self.infer(env, obj)?;
                let result_ty = self.gen.fresh_type();
                // Expect the object to be a record containing this field
                let mut expected_fields = std::collections::BTreeMap::new();
                expected_fields.insert(field.clone(), result_ty.clone());
                let expected = MonoType::Record(expected_fields);
                let s2 = unify(&s1.apply(&obj_ty), &expected, expr.span)?;
                let s = s2.compose(&s1);
                Ok((s.clone(), s.apply(&result_ty)))
            }
        }
    }

    fn infer_binop(
        &mut self,
        env: &TypeEnv,
        op: &BinOp,
        lhs: &SpannedExpr,
        rhs: &SpannedExpr,
        span: Span,
    ) -> Result<(Subst, MonoType), LyraError> {
        let (s1, lhs_ty) = self.infer(env, lhs)?;
        let env2 = env.apply_subst(&s1);
        let (s2, rhs_ty) = self.infer(&env2, rhs)?;
        let mut s = s2.compose(&s1);

        match op {
            // Arithmetic: Int -> Int -> Int (or Float)
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                let s3 = unify(&s.apply(&lhs_ty), &s.apply(&rhs_ty), span)?;
                s = s3.compose(&s);
                let unified_ty = s.apply(&lhs_ty);
                // Must be Int or Float
                let s4 = unify(&unified_ty, &MonoType::Int, span)
                    .or_else(|_| unify(&unified_ty, &MonoType::Float, span))?;
                s = s4.compose(&s);
                Ok((s.clone(), s.apply(&lhs_ty)))
            }

            // Comparison: a -> a -> Bool (for ordered types)
            BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => {
                let s3 = unify(&s.apply(&lhs_ty), &s.apply(&rhs_ty), span)?;
                s = s3.compose(&s);
                Ok((s, MonoType::Bool))
            }

            // Equality: a -> a -> Bool
            BinOp::Eq | BinOp::NotEq => {
                let s3 = unify(&s.apply(&lhs_ty), &s.apply(&rhs_ty), span)?;
                s = s3.compose(&s);
                Ok((s, MonoType::Bool))
            }

            // Logical: Bool -> Bool -> Bool
            BinOp::And | BinOp::Or => {
                let s3 = unify(&s.apply(&lhs_ty), &MonoType::Bool, span)?;
                s = s3.compose(&s);
                let s4 = unify(&s.apply(&rhs_ty), &MonoType::Bool, span)?;
                s = s4.compose(&s);
                Ok((s, MonoType::Bool))
            }

            // Cons: a -> [a] -> [a]
            BinOp::Cons => {
                let list_ty = MonoType::List(Box::new(s.apply(&lhs_ty)));
                let s3 = unify(&s.apply(&rhs_ty), &list_ty, span)?;
                s = s3.compose(&s);
                Ok((s.clone(), s.apply(&rhs_ty)))
            }
        }
    }

    /// Infer types from a pattern, returning bindings introduced.
    fn infer_pattern(
        &mut self,
        env: &TypeEnv,
        pattern: &SpannedPattern,
        expected: &MonoType,
    ) -> Result<(Subst, Vec<(String, MonoType)>), LyraError> {
        match &pattern.node {
            Pattern::Wildcard => Ok((Subst::new(), vec![])),

            Pattern::Var(name) => {
                Ok((Subst::new(), vec![(name.clone(), expected.clone())]))
            }

            Pattern::IntLit(_) => {
                let s = unify(expected, &MonoType::Int, pattern.span)?;
                Ok((s, vec![]))
            }

            Pattern::FloatLit(_) => {
                let s = unify(expected, &MonoType::Float, pattern.span)?;
                Ok((s, vec![]))
            }

            Pattern::StringLit(_) => {
                let s = unify(expected, &MonoType::String, pattern.span)?;
                Ok((s, vec![]))
            }

            Pattern::BoolLit(_) => {
                let s = unify(expected, &MonoType::Bool, pattern.span)?;
                Ok((s, vec![]))
            }

            Pattern::UnitLit => {
                let s = unify(expected, &MonoType::Unit, pattern.span)?;
                Ok((s, vec![]))
            }

            Pattern::Tuple(pats) => {
                let elem_types: Vec<MonoType> =
                    pats.iter().map(|_| self.gen.fresh_type()).collect();
                let tuple_ty = MonoType::Tuple(elem_types.clone());
                let s1 = unify(expected, &tuple_ty, pattern.span)?;

                let mut subst = s1;
                let mut bindings = Vec::new();
                for (pat, ty) in pats.iter().zip(&elem_types) {
                    let (s, b) =
                        self.infer_pattern(env, pat, &subst.apply(ty))?;
                    subst = s.compose(&subst);
                    bindings.extend(b);
                }
                Ok((subst, bindings))
            }

            Pattern::List(pats) => {
                let elem_ty = self.gen.fresh_type();
                let list_ty = MonoType::List(Box::new(elem_ty.clone()));
                let s1 = unify(expected, &list_ty, pattern.span)?;

                let mut subst = s1;
                let mut bindings = Vec::new();
                for pat in pats {
                    let (s, b) = self.infer_pattern(
                        env,
                        pat,
                        &subst.apply(&elem_ty),
                    )?;
                    subst = s.compose(&subst);
                    bindings.extend(b);
                }
                Ok((subst, bindings))
            }

            Pattern::Cons(head, tail) => {
                let elem_ty = self.gen.fresh_type();
                let list_ty = MonoType::List(Box::new(elem_ty.clone()));
                let s1 = unify(expected, &list_ty, pattern.span)?;
                let mut subst = s1;

                let (s2, head_bindings) = self.infer_pattern(
                    env,
                    head,
                    &subst.apply(&elem_ty),
                )?;
                subst = s2.compose(&subst);

                let (s3, tail_bindings) = self.infer_pattern(
                    env,
                    tail,
                    &subst.apply(&list_ty),
                )?;
                subst = s3.compose(&subst);

                let mut bindings = head_bindings;
                bindings.extend(tail_bindings);
                Ok((subst, bindings))
            }

            Pattern::Constructor { name, args } => {
                let info = self.constructors.get(name).cloned().ok_or_else(|| {
                    LyraError::UndefinedConstructor {
                        name: name.clone(),
                        span: pattern.span,
                    }
                })?;

                if args.len() != info.field_types.len() {
                    return Err(LyraError::ArityMismatch {
                        name: name.clone(),
                        expected: info.field_types.len(),
                        found: args.len(),
                        span: pattern.span,
                    });
                }

                // Create fresh type variables for type params
                let fresh_params: Vec<(String, MonoType)> = info
                    .type_params
                    .iter()
                    .map(|p| (p.clone(), self.gen.fresh_type()))
                    .collect();

                let result_ty = if fresh_params.is_empty() {
                    MonoType::Con(info.type_name.clone(), vec![])
                } else {
                    MonoType::Con(
                        info.type_name.clone(),
                        fresh_params.iter().map(|(_, t)| t.clone()).collect(),
                    )
                };

                let s1 = unify(expected, &result_ty, pattern.span)?;
                let mut subst = s1;

                let mut bindings = Vec::new();
                for (arg_pat, field_ty) in args.iter().zip(&info.field_types) {
                    let concrete_field = subst.apply(field_ty);
                    let (s, b) =
                        self.infer_pattern(env, arg_pat, &concrete_field)?;
                    subst = s.compose(&subst);
                    bindings.extend(b);
                }

                Ok((subst, bindings))
            }
        }
    }

    /// Infer the type of a top-level declaration.
    pub fn infer_decl(
        &mut self,
        env: &mut TypeEnv,
        decl: &Decl,
    ) -> Result<Option<MonoType>, LyraError> {
        match decl {
            Decl::Let {
                name,
                recursive,
                body,
                ..
            } => {
                if *recursive {
                    let fresh = self.gen.fresh_type();
                    let mut rec_env = env.clone();
                    rec_env.insert(name.node.clone(), TypeScheme::mono(fresh.clone()));

                    let (s1, bind_ty) = self.infer(&rec_env, body)?;
                    let s2 = unify(&s1.apply(&fresh), &bind_ty, body.span)?;
                    let combined = s2.compose(&s1);

                    let final_ty = combined.apply(&bind_ty);
                    let scheme = Self::generalize(&env.apply_subst(&combined), &final_ty);
                    env.insert(name.node.clone(), scheme);
                    Ok(Some(final_ty))
                } else {
                    let (s, ty) = self.infer(env, body)?;
                    let scheme = Self::generalize(&env.apply_subst(&s), &ty);
                    env.insert(name.node.clone(), scheme);
                    Ok(Some(ty))
                }
            }

            Decl::Type { .. } => {
                self.register_type_decl(env, decl)?;
                Ok(None)
            }

            Decl::Expr(expr) => {
                let (_, ty) = self.infer(env, expr)?;
                Ok(Some(ty))
            }

            Decl::Import { .. } => {
                // Import type checking will be implemented in Phase 4
                Ok(None)
            }
        }
    }
}
