use std::collections::BTreeMap;
use std::fmt;

use crate::ast::SpannedExpr;
use crate::compiler::bytecode::FunctionProto;

use super::env::Env;

#[derive(Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Unit,
    List(Vec<Value>),
    Tuple(Vec<Value>),
    Record(BTreeMap<String, Value>),
    Closure {
        params: Vec<String>,
        body: SpannedExpr,
        env: Env,
        recursive_name: Option<String>,
    },
    Builtin {
        name: String,
        arity: usize,
        func: fn(Vec<Value>) -> Result<Value, String>,
    },
    PartialApp {
        func: Box<Value>,
        applied_args: Vec<Value>,
    },
    Adt {
        constructor: String,
        fields: Vec<Value>,
    },
    /// Compiled function (bytecode).
    Function(FunctionProto),
    /// Compiled closure (bytecode + captured values).
    ClosureVal {
        proto: FunctionProto,
        upvalues: Vec<Value>,
    },
}

impl Value {
    pub fn type_name(&self) -> &str {
        match self {
            Value::Int(_) => "Int",
            Value::Float(_) => "Float",
            Value::Bool(_) => "Bool",
            Value::String(_) => "String",
            Value::Unit => "()",
            Value::List(_) => "List",
            Value::Tuple(_) => "Tuple",
            Value::Record(_) => "Record",
            Value::Closure { .. } => "Function",
            Value::Builtin { .. } => "Function",
            Value::PartialApp { .. } => "Function",
            Value::Function { .. } => "Function",
            Value::ClosureVal { .. } => "Function",
            Value::Adt { constructor, .. } => constructor.as_str(),
        }
    }

    /// Display a value for string interpolation (strings without quotes).
    pub fn display_unquoted(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            other => format!("{}", other),
        }
    }

    pub fn total_arity(&self) -> usize {
        match self {
            Value::Closure { params, .. } => params.len(),
            Value::Builtin { arity, .. } => *arity,
            Value::PartialApp { func, applied_args } => {
                func.total_arity() - applied_args.len()
            }
            _ => 0,
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => {
                if *n == n.floor() && n.is_finite() {
                    write!(f, "{:.1}", n)
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::Bool(b) => write!(f, "{}", b),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Unit => write!(f, "()"),
            Value::List(items) => {
                write!(f, "[")?;
                for (i, v) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Tuple(items) => {
                write!(f, "(")?;
                for (i, v) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, ")")
            }
            Value::Record(map) => {
                write!(f, "{{ ")?;
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, " }}")
            }
            Value::Closure { .. } => write!(f, "<function>"),
            Value::Builtin { name, .. } => write!(f, "<builtin:{}>", name),
            Value::PartialApp { .. } => write!(f, "<partial>"),
            Value::Function(proto) => write!(f, "<fn:{}>", proto.name),
            Value::ClosureVal { proto, .. } => write!(f, "<closure:{}>", proto.name),
            Value::Adt {
                constructor,
                fields,
            } => {
                write!(f, "{}", constructor)?;
                if !fields.is_empty() {
                    write!(f, "(")?;
                    for (i, v) in fields.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", v)?;
                    }
                    write!(f, ")")?;
                }
                Ok(())
            }
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Unit, Value::Unit) => true,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Tuple(a), Value::Tuple(b)) => a == b,
            (
                Value::Adt {
                    constructor: c1,
                    fields: f1,
                },
                Value::Adt {
                    constructor: c2,
                    fields: f2,
                },
            ) => c1 == c2 && f1 == f2,
            (Value::Record(a), Value::Record(b)) => a == b,
            _ => false,
        }
    }
}
