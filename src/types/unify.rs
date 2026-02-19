use crate::error::LyraError;
use crate::span::Span;

use super::subst::Subst;
use super::{MonoType, TypeVar};

/// Unify two types, returning a substitution that makes them equal.
pub fn unify(t1: &MonoType, t2: &MonoType, span: Span) -> Result<Subst, LyraError> {
    match (t1, t2) {
        // Identical primitives
        (MonoType::Int, MonoType::Int)
        | (MonoType::Float, MonoType::Float)
        | (MonoType::Bool, MonoType::Bool)
        | (MonoType::String, MonoType::String)
        | (MonoType::Unit, MonoType::Unit) => Ok(Subst::new()),

        // Same type variable
        (MonoType::Var(a), MonoType::Var(b)) if a == b => Ok(Subst::new()),

        // Var on left
        (MonoType::Var(v), t) => bind(*v, t, span),

        // Var on right
        (t, MonoType::Var(v)) => bind(*v, t, span),

        // Arrow types
        (MonoType::Arrow(a1, b1), MonoType::Arrow(a2, b2)) => {
            let s1 = unify(a1, a2, span)?;
            let s2 = unify(&s1.apply(b1), &s1.apply(b2), span)?;
            Ok(s2.compose(&s1))
        }

        // List types
        (MonoType::List(a), MonoType::List(b)) => unify(a, b, span),

        // Tuple types
        (MonoType::Tuple(a), MonoType::Tuple(b)) if a.len() == b.len() => {
            unify_many(a, b, span)
        }

        // Constructor types
        (MonoType::Con(n1, a1), MonoType::Con(n2, a2)) if n1 == n2 && a1.len() == a2.len() => {
            unify_many(a1, a2, span)
        }

        // Record types — structural: unify common fields, allow extra fields on either side
        (MonoType::Record(f1), MonoType::Record(f2)) => {
            let mut subst = Subst::new();
            for (name, ty1) in f1 {
                if let Some(ty2) = f2.get(name) {
                    let s = unify(&subst.apply(ty1), &subst.apply(ty2), span)?;
                    subst = s.compose(&subst);
                }
            }
            Ok(subst)
        }

        _ => Err(LyraError::TypeMismatch {
            expected: t1.to_string(),
            found: t2.to_string(),
            span,
        }),
    }
}

fn bind(var: TypeVar, ty: &MonoType, span: Span) -> Result<Subst, LyraError> {
    if let MonoType::Var(v) = ty {
        if *v == var {
            return Ok(Subst::new());
        }
    }
    if occurs(var, ty) {
        return Err(LyraError::InfiniteType {
            var: format!("t{}", var),
            ty: ty.to_string(),
            span,
        });
    }
    Ok(Subst::single(var, ty.clone()))
}

fn occurs(var: TypeVar, ty: &MonoType) -> bool {
    match ty {
        MonoType::Var(v) => *v == var,
        MonoType::Arrow(a, b) => occurs(var, a) || occurs(var, b),
        MonoType::List(inner) => occurs(var, inner),
        MonoType::Tuple(elems) => elems.iter().any(|e| occurs(var, e)),
        MonoType::Con(_, args) => args.iter().any(|a| occurs(var, a)),
        MonoType::Record(fields) => fields.values().any(|t| occurs(var, t)),
        _ => false,
    }
}

fn unify_many(a: &[MonoType], b: &[MonoType], span: Span) -> Result<Subst, LyraError> {
    let mut subst = Subst::new();
    for (t1, t2) in a.iter().zip(b.iter()) {
        let s = unify(&subst.apply(t1), &subst.apply(t2), span)?;
        subst = s.compose(&subst);
    }
    Ok(subst)
}
