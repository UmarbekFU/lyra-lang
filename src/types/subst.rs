use std::collections::HashMap;

use super::{MonoType, TypeScheme, TypeVar};

/// A substitution maps type variables to mono types.
#[derive(Debug, Clone, Default)]
pub struct Subst {
    pub map: HashMap<TypeVar, MonoType>,
}

impl Subst {
    pub fn new() -> Self {
        Subst {
            map: HashMap::new(),
        }
    }

    pub fn single(tv: TypeVar, ty: MonoType) -> Self {
        let mut map = HashMap::new();
        map.insert(tv, ty);
        Subst { map }
    }

    /// Apply this substitution to a MonoType.
    pub fn apply(&self, ty: &MonoType) -> MonoType {
        match ty {
            MonoType::Var(v) => {
                if let Some(replacement) = self.map.get(v) {
                    self.apply(replacement)
                } else {
                    ty.clone()
                }
            }
            MonoType::Arrow(a, b) => {
                MonoType::Arrow(Box::new(self.apply(a)), Box::new(self.apply(b)))
            }
            MonoType::List(inner) => MonoType::List(Box::new(self.apply(inner))),
            MonoType::Tuple(elems) => {
                MonoType::Tuple(elems.iter().map(|e| self.apply(e)).collect())
            }
            MonoType::Con(name, args) => {
                MonoType::Con(name.clone(), args.iter().map(|a| self.apply(a)).collect())
            }
            MonoType::Record(fields) => {
                MonoType::Record(
                    fields.iter().map(|(k, v)| (k.clone(), self.apply(v))).collect(),
                )
            }
            _ => ty.clone(),
        }
    }

    /// Apply to a type scheme (substitute free variables only).
    pub fn apply_scheme(&self, scheme: &TypeScheme) -> TypeScheme {
        // Remove quantified variables from the substitution temporarily
        let mut filtered = self.clone();
        for v in &scheme.vars {
            filtered.map.remove(v);
        }
        TypeScheme {
            vars: scheme.vars.clone(),
            ty: filtered.apply(&scheme.ty),
        }
    }

    /// Compose two substitutions: (self ∘ other)(t) = self(other(t))
    pub fn compose(&self, other: &Subst) -> Subst {
        let mut result = Subst::new();
        // Apply self to all of other's mappings
        for (v, ty) in &other.map {
            result.map.insert(*v, self.apply(ty));
        }
        // Add self's mappings (don't override)
        for (v, ty) in &self.map {
            result.map.entry(*v).or_insert_with(|| ty.clone());
        }
        result
    }
}
