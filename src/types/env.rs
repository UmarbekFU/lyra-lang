use std::collections::{HashMap, HashSet};

use super::subst::Subst;
use super::{TypeScheme, TypeVar};

/// Type environment: maps names to type schemes.
#[derive(Debug, Clone)]
pub struct TypeEnv {
    bindings: HashMap<String, TypeScheme>,
}

impl TypeEnv {
    pub fn new() -> Self {
        TypeEnv {
            bindings: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: String, scheme: TypeScheme) {
        self.bindings.insert(name, scheme);
    }

    pub fn lookup(&self, name: &str) -> Option<&TypeScheme> {
        self.bindings.get(name)
    }

    pub fn remove(&mut self, name: &str) {
        self.bindings.remove(name);
    }

    pub fn names(&self) -> Vec<&str> {
        self.bindings.keys().map(|s| s.as_str()).collect()
    }

    pub fn free_vars(&self) -> HashSet<TypeVar> {
        let mut s = HashSet::new();
        for scheme in self.bindings.values() {
            s.extend(scheme.free_vars());
        }
        s
    }

    /// Apply a substitution to all type schemes in the environment.
    pub fn apply_subst(&self, subst: &Subst) -> TypeEnv {
        TypeEnv {
            bindings: self
                .bindings
                .iter()
                .map(|(k, v)| (k.clone(), subst.apply_scheme(v)))
                .collect(),
        }
    }
}
