use crate::ast::{Pattern, SpannedPattern};

use super::value::Value;

/// Try to match a value against a pattern.
/// Returns Some(bindings) on success, None on failure.
pub fn match_pattern(pattern: &SpannedPattern, value: &Value) -> Option<Vec<(String, Value)>> {
    match (&pattern.node, value) {
        // Wildcard matches anything
        (Pattern::Wildcard, _) => Some(vec![]),

        // Variable binds the value
        (Pattern::Var(name), val) => Some(vec![(name.clone(), val.clone())]),

        // Literal patterns
        (Pattern::IntLit(a), Value::Int(b)) if *a == *b => Some(vec![]),
        (Pattern::FloatLit(a), Value::Float(b)) if *a == *b => Some(vec![]),
        (Pattern::StringLit(a), Value::String(b)) if a == b => Some(vec![]),
        (Pattern::BoolLit(a), Value::Bool(b)) if *a == *b => Some(vec![]),
        (Pattern::UnitLit, Value::Unit) => Some(vec![]),

        // Tuple pattern
        (Pattern::Tuple(pats), Value::Tuple(vals)) if pats.len() == vals.len() => {
            let mut bindings = Vec::new();
            for (pat, val) in pats.iter().zip(vals.iter()) {
                match match_pattern(pat, val) {
                    Some(b) => bindings.extend(b),
                    None => return None,
                }
            }
            Some(bindings)
        }

        // List pattern (exact length)
        (Pattern::List(pats), Value::List(vals)) if pats.len() == vals.len() => {
            let mut bindings = Vec::new();
            for (pat, val) in pats.iter().zip(vals.iter()) {
                match match_pattern(pat, val) {
                    Some(b) => bindings.extend(b),
                    None => return None,
                }
            }
            Some(bindings)
        }

        // Cons pattern: hd :: tl
        (Pattern::Cons(head, tail), Value::List(list)) if !list.is_empty() => {
            let mut bindings = Vec::new();
            match match_pattern(head, &list[0]) {
                Some(b) => bindings.extend(b),
                None => return None,
            }
            let tail_val = Value::List(list[1..].to_vec());
            match match_pattern(tail, &tail_val) {
                Some(b) => bindings.extend(b),
                None => return None,
            }
            Some(bindings)
        }

        // Constructor pattern
        (
            Pattern::Constructor { name: pname, args },
            Value::Adt {
                constructor,
                fields,
            },
        ) if pname == constructor && args.len() == fields.len() => {
            let mut bindings = Vec::new();
            for (pat, val) in args.iter().zip(fields.iter()) {
                match match_pattern(pat, val) {
                    Some(b) => bindings.extend(b),
                    None => return None,
                }
            }
            Some(bindings)
        }

        _ => None,
    }
}
