use super::value::Value;

fn builtin(name: &str, arity: usize, func: fn(Vec<Value>) -> Result<Value, String>) -> (String, Value) {
    (
        name.to_string(),
        Value::Builtin {
            name: name.to_string(),
            arity,
            func,
        },
    )
}

pub fn all_builtins() -> Vec<(String, Value)> {
    vec![
        // IO
        builtin("print", 1, |args| {
            match &args[0] {
                Value::String(s) => print!("{}", s),
                v => print!("{}", v),
            }
            Ok(Value::Unit)
        }),
        builtin("println", 1, |args| {
            match &args[0] {
                Value::String(s) => println!("{}", s),
                v => println!("{}", v),
            }
            Ok(Value::Unit)
        }),

        // String
        builtin("to_string", 1, |args| {
            Ok(Value::String(format!("{}", args[0])))
        }),
        builtin("str_length", 1, |args| {
            match &args[0] {
                Value::String(s) => Ok(Value::Int(s.len() as i64)),
                v => Err(format!("str_length: expected String, got {}", v.type_name())),
            }
        }),
        builtin("str_concat", 2, |args| {
            match (&args[0], &args[1]) {
                (Value::String(a), Value::String(b)) => {
                    Ok(Value::String(format!("{}{}", a, b)))
                }
                _ => Err("str_concat: expected two Strings".to_string()),
            }
        }),
        builtin("str_contains", 2, |args| {
            match (&args[0], &args[1]) {
                (Value::String(haystack), Value::String(needle)) => {
                    Ok(Value::Bool(haystack.contains(needle.as_str())))
                }
                _ => Err("str_contains: expected two Strings".to_string()),
            }
        }),
        builtin("str_split", 2, |args| {
            match (&args[0], &args[1]) {
                (Value::String(s), Value::String(delim)) => {
                    let parts: Vec<Value> = s
                        .split(delim.as_str())
                        .map(|p| Value::String(p.to_string()))
                        .collect();
                    Ok(Value::List(parts))
                }
                _ => Err("str_split: expected two Strings".to_string()),
            }
        }),
        builtin("str_chars", 1, |args| {
            match &args[0] {
                Value::String(s) => {
                    let chars: Vec<Value> = s
                        .chars()
                        .map(|c| Value::String(c.to_string()))
                        .collect();
                    Ok(Value::List(chars))
                }
                _ => Err("str_chars: expected String".to_string()),
            }
        }),

        // List
        builtin("length", 1, |args| {
            match &args[0] {
                Value::List(l) => Ok(Value::Int(l.len() as i64)),
                v => Err(format!("length: expected List, got {}", v.type_name())),
            }
        }),
        builtin("head", 1, |args| {
            match &args[0] {
                Value::List(l) if !l.is_empty() => Ok(l[0].clone()),
                Value::List(_) => Err("head: empty list".to_string()),
                v => Err(format!("head: expected List, got {}", v.type_name())),
            }
        }),
        builtin("tail", 1, |args| {
            match &args[0] {
                Value::List(l) if !l.is_empty() => Ok(Value::List(l[1..].to_vec())),
                Value::List(_) => Err("tail: empty list".to_string()),
                v => Err(format!("tail: expected List, got {}", v.type_name())),
            }
        }),
        builtin("reverse", 1, |args| {
            match &args[0] {
                Value::List(l) => {
                    let mut r = l.clone();
                    r.reverse();
                    Ok(Value::List(r))
                }
                v => Err(format!("reverse: expected List, got {}", v.type_name())),
            }
        }),
        builtin("append", 2, |args| {
            match (&args[0], &args[1]) {
                (Value::List(a), Value::List(b)) => {
                    let mut r = a.clone();
                    r.extend(b.clone());
                    Ok(Value::List(r))
                }
                _ => Err("append: expected two Lists".to_string()),
            }
        }),
        builtin("range", 2, |args| {
            match (&args[0], &args[1]) {
                (Value::Int(start), Value::Int(end)) => {
                    let vals: Vec<Value> = (*start..*end).map(Value::Int).collect();
                    Ok(Value::List(vals))
                }
                _ => Err("range: expected two Ints".to_string()),
            }
        }),
        builtin("nth", 2, |args| {
            match (&args[0], &args[1]) {
                (Value::List(l), Value::Int(i)) => {
                    let idx = *i as usize;
                    if idx < l.len() {
                        Ok(l[idx].clone())
                    } else {
                        Err(format!("nth: index {} out of bounds for length {}", i, l.len()))
                    }
                }
                _ => Err("nth: expected List and Int".to_string()),
            }
        }),

        // Math
        builtin("abs", 1, |args| {
            match &args[0] {
                Value::Int(n) => Ok(Value::Int(n.abs())),
                Value::Float(n) => Ok(Value::Float(n.abs())),
                v => Err(format!("abs: expected number, got {}", v.type_name())),
            }
        }),
        builtin("min", 2, |args| {
            match (&args[0], &args[1]) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(*a.min(b))),
                _ => Err("min: expected two Ints".to_string()),
            }
        }),
        builtin("max", 2, |args| {
            match (&args[0], &args[1]) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(*a.max(b))),
                _ => Err("max: expected two Ints".to_string()),
            }
        }),
        builtin("pow", 2, |args| {
            match (&args[0], &args[1]) {
                (Value::Int(base), Value::Int(exp)) => {
                    Ok(Value::Int(base.pow(*exp as u32)))
                }
                _ => Err("pow: expected two Ints".to_string()),
            }
        }),
        builtin("float_of_int", 1, |args| {
            match &args[0] {
                Value::Int(n) => Ok(Value::Float(*n as f64)),
                v => Err(format!("float_of_int: expected Int, got {}", v.type_name())),
            }
        }),
        builtin("int_of_float", 1, |args| {
            match &args[0] {
                Value::Float(n) => Ok(Value::Int(*n as i64)),
                v => Err(format!("int_of_float: expected Float, got {}", v.type_name())),
            }
        }),

        // List: take, drop, flatten
        builtin("take", 2, |args| {
            match (&args[0], &args[1]) {
                (Value::Int(n), Value::List(l)) => {
                    let n = *n as usize;
                    Ok(Value::List(l.iter().take(n).cloned().collect()))
                }
                _ => Err("take: expected Int and List".to_string()),
            }
        }),
        builtin("drop", 2, |args| {
            match (&args[0], &args[1]) {
                (Value::Int(n), Value::List(l)) => {
                    let n = *n as usize;
                    Ok(Value::List(l.iter().skip(n).cloned().collect()))
                }
                _ => Err("drop: expected Int and List".to_string()),
            }
        }),
        builtin("flatten", 1, |args| {
            match &args[0] {
                Value::List(outer) => {
                    let mut result = Vec::new();
                    for item in outer {
                        match item {
                            Value::List(inner) => result.extend(inner.clone()),
                            _ => return Err("flatten: expected List of Lists".to_string()),
                        }
                    }
                    Ok(Value::List(result))
                }
                v => Err(format!("flatten: expected List, got {}", v.type_name())),
            }
        }),
        builtin("sum", 1, |args| {
            match &args[0] {
                Value::List(l) => {
                    let mut total: i64 = 0;
                    for item in l {
                        match item {
                            Value::Int(n) => total += n,
                            v => return Err(format!("sum: expected Int elements, got {}", v.type_name())),
                        }
                    }
                    Ok(Value::Int(total))
                }
                v => Err(format!("sum: expected List, got {}", v.type_name())),
            }
        }),
        builtin("product", 1, |args| {
            match &args[0] {
                Value::List(l) => {
                    let mut total: i64 = 1;
                    for item in l {
                        match item {
                            Value::Int(n) => total *= n,
                            v => return Err(format!("product: expected Int elements, got {}", v.type_name())),
                        }
                    }
                    Ok(Value::Int(total))
                }
                v => Err(format!("product: expected List, got {}", v.type_name())),
            }
        }),

        // String conversion
        builtin("string_to_int", 1, |args| {
            match &args[0] {
                Value::String(s) => s
                    .parse::<i64>()
                    .map(Value::Int)
                    .map_err(|_| format!("string_to_int: cannot parse \"{}\" as Int", s)),
                v => Err(format!("string_to_int: expected String, got {}", v.type_name())),
            }
        }),
        builtin("int_to_string", 1, |args| {
            match &args[0] {
                Value::Int(n) => Ok(Value::String(n.to_string())),
                v => Err(format!("int_to_string: expected Int, got {}", v.type_name())),
            }
        }),

        // String utilities
        builtin("str_trim", 1, |args| {
            match &args[0] {
                Value::String(s) => Ok(Value::String(s.trim().to_string())),
                v => Err(format!("str_trim: expected String, got {}", v.type_name())),
            }
        }),
        builtin("str_uppercase", 1, |args| {
            match &args[0] {
                Value::String(s) => Ok(Value::String(s.to_uppercase())),
                v => Err(format!("str_uppercase: expected String, got {}", v.type_name())),
            }
        }),
        builtin("str_lowercase", 1, |args| {
            match &args[0] {
                Value::String(s) => Ok(Value::String(s.to_lowercase())),
                v => Err(format!("str_lowercase: expected String, got {}", v.type_name())),
            }
        }),
        builtin("str_replace", 3, |args| {
            match (&args[0], &args[1], &args[2]) {
                (Value::String(s), Value::String(from), Value::String(to)) => {
                    Ok(Value::String(s.replace(from.as_str(), to.as_str())))
                }
                _ => Err("str_replace: expected three Strings".to_string()),
            }
        }),
        builtin("str_starts_with", 2, |args| {
            match (&args[0], &args[1]) {
                (Value::String(s), Value::String(prefix)) => {
                    Ok(Value::Bool(s.starts_with(prefix.as_str())))
                }
                _ => Err("str_starts_with: expected two Strings".to_string()),
            }
        }),
        builtin("str_ends_with", 2, |args| {
            match (&args[0], &args[1]) {
                (Value::String(s), Value::String(suffix)) => {
                    Ok(Value::Bool(s.ends_with(suffix.as_str())))
                }
                _ => Err("str_ends_with: expected two Strings".to_string()),
            }
        }),
        builtin("str_substring", 3, |args| {
            match (&args[0], &args[1], &args[2]) {
                (Value::String(s), Value::Int(start), Value::Int(end)) => {
                    let start = *start as usize;
                    let end = (*end as usize).min(s.len());
                    if start > s.len() {
                        Ok(Value::String(String::new()))
                    } else {
                        Ok(Value::String(s[start..end].to_string()))
                    }
                }
                _ => Err("str_substring: expected String, Int, Int".to_string()),
            }
        }),

        // Higher-order list functions are handled in eval/mod.rs
        // because they need to call back into the evaluator
    ]
}
