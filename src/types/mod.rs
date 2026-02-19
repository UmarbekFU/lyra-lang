pub mod env;
pub mod exhaustiveness;
pub mod infer;
pub mod subst;
pub mod unify;

use std::collections::{BTreeMap, HashSet};
use std::fmt;

/// Unique identifier for type variables.
pub type TypeVar = u64;

/// Monomorphic types (no quantifiers).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonoType {
    Var(TypeVar),
    Int,
    Float,
    Bool,
    String,
    Unit,
    Arrow(Box<MonoType>, Box<MonoType>),
    List(Box<MonoType>),
    Tuple(Vec<MonoType>),
    Con(String, Vec<MonoType>),
    Record(BTreeMap<String, MonoType>),
}

impl MonoType {
    pub fn free_vars(&self) -> HashSet<TypeVar> {
        match self {
            MonoType::Var(v) => {
                let mut s = HashSet::new();
                s.insert(*v);
                s
            }
            MonoType::Arrow(a, b) => {
                let mut s = a.free_vars();
                s.extend(b.free_vars());
                s
            }
            MonoType::List(inner) => inner.free_vars(),
            MonoType::Tuple(elems) => {
                let mut s = HashSet::new();
                for e in elems {
                    s.extend(e.free_vars());
                }
                s
            }
            MonoType::Con(_, args) => {
                let mut s = HashSet::new();
                for a in args {
                    s.extend(a.free_vars());
                }
                s
            }
            MonoType::Record(fields) => {
                let mut s = HashSet::new();
                for ty in fields.values() {
                    s.extend(ty.free_vars());
                }
                s
            }
            _ => HashSet::new(),
        }
    }

    /// Build a curried arrow type from params to return type.
    pub fn curried_arrow(params: Vec<MonoType>, ret: MonoType) -> MonoType {
        params
            .into_iter()
            .rev()
            .fold(ret, |acc, p| MonoType::Arrow(Box::new(p), Box::new(acc)))
    }
}

impl fmt::Display for MonoType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MonoType::Var(v) => write!(f, "t{}", v),
            MonoType::Int => write!(f, "Int"),
            MonoType::Float => write!(f, "Float"),
            MonoType::Bool => write!(f, "Bool"),
            MonoType::String => write!(f, "String"),
            MonoType::Unit => write!(f, "()"),
            MonoType::Arrow(a, b) => {
                let needs_parens = matches!(a.as_ref(), MonoType::Arrow(_, _));
                if needs_parens {
                    write!(f, "({}) -> {}", a, b)
                } else {
                    write!(f, "{} -> {}", a, b)
                }
            }
            MonoType::List(inner) => write!(f, "[{}]", inner),
            MonoType::Tuple(elems) => {
                write!(f, "(")?;
                for (i, e) in elems.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", e)?;
                }
                write!(f, ")")
            }
            MonoType::Con(name, args) => {
                write!(f, "{}", name)?;
                for a in args {
                    write!(f, " {}", a)?;
                }
                Ok(())
            }
            MonoType::Record(fields) => {
                write!(f, "{{ ")?;
                for (i, (name, ty)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", name, ty)?;
                }
                write!(f, " }}")
            }
        }
    }
}

/// Polymorphic type scheme: forall a b . T
#[derive(Debug, Clone)]
pub struct TypeScheme {
    pub vars: Vec<TypeVar>,
    pub ty: MonoType,
}

impl TypeScheme {
    pub fn mono(ty: MonoType) -> Self {
        TypeScheme {
            vars: vec![],
            ty,
        }
    }

    pub fn free_vars(&self) -> HashSet<TypeVar> {
        let mut s = self.ty.free_vars();
        for v in &self.vars {
            s.remove(v);
        }
        s
    }
}

impl fmt::Display for TypeScheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.vars.is_empty() {
            write!(f, "{}", self.ty)
        } else {
            write!(f, "forall")?;
            for v in &self.vars {
                write!(f, " t{}", v)?;
            }
            write!(f, ". {}", self.ty)
        }
    }
}

/// Generates unique type variables.
pub struct TypeVarGen {
    next: TypeVar,
}

impl TypeVarGen {
    pub fn new() -> Self {
        TypeVarGen { next: 0 }
    }

    pub fn fresh(&mut self) -> TypeVar {
        let v = self.next;
        self.next += 1;
        v
    }

    pub fn fresh_type(&mut self) -> MonoType {
        MonoType::Var(self.fresh())
    }
}
