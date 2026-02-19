pub mod builtins;
pub mod env;
pub mod pattern;
pub mod value;

use std::cell::RefCell;
use std::collections::HashMap;

use crate::ast::*;
use crate::error::LyraError;
use crate::span::Span;

use env::Env;
use pattern::match_pattern;
use value::Value;

// Thread-local storage for VM globals, used to pass globals to mini-VMs in callbacks.
thread_local! {
    static VM_GLOBALS: RefCell<Option<HashMap<String, Value>>> = RefCell::new(None);
}

pub fn set_vm_globals(globals: HashMap<String, Value>) {
    VM_GLOBALS.with(|g| {
        *g.borrow_mut() = Some(globals);
    });
}

fn get_vm_globals() -> Option<HashMap<String, Value>> {
    VM_GLOBALS.with(|g| g.borrow().clone())
}

/// Evaluate an expression in the given environment.
pub fn eval(env: &Env, expr: &SpannedExpr) -> Result<Value, LyraError> {
    match &expr.node {
        // ── Literals ──
        Expr::IntLit(n) => Ok(Value::Int(*n)),
        Expr::FloatLit(n) => Ok(Value::Float(*n)),
        Expr::BoolLit(b) => Ok(Value::Bool(*b)),
        Expr::StringLit(s) => Ok(Value::String(s.clone())),
        Expr::UnitLit => Ok(Value::Unit),

        // ── Variable ──
        Expr::Var(name) => env.get(name).ok_or_else(|| LyraError::UndefinedVariable {
            suggestion: None, // env doesn't expose all names easily
            name: name.clone(),
            span: expr.span,
        }),

        // ── List literal ──
        Expr::ListLit(elems) => {
            let vals: Result<Vec<Value>, _> = elems.iter().map(|e| eval(env, e)).collect();
            Ok(Value::List(vals?))
        }

        // ── Tuple literal ──
        Expr::TupleLit(elems) => {
            let vals: Result<Vec<Value>, _> = elems.iter().map(|e| eval(env, e)).collect();
            Ok(Value::Tuple(vals?))
        }

        // ── Lambda ──
        Expr::Lambda { params, body } => Ok(Value::Closure {
            params: params.iter().map(|p| p.name.node.clone()).collect(),
            body: *body.clone(),
            env: env.clone(),
            recursive_name: None,
        }),

        // ── Application ──
        Expr::App { func, args } => {
            let func_val = eval(env, func)?;
            let arg_vals: Result<Vec<Value>, _> = args.iter().map(|a| eval(env, a)).collect();
            apply_function(func_val, arg_vals?, expr.span)
        }

        // ── Binary operation ──
        Expr::BinOp { op, lhs, rhs } => {
            // Short-circuit for && and ||
            if *op == BinOp::And {
                let l = eval(env, lhs)?;
                return match l {
                    Value::Bool(false) => Ok(Value::Bool(false)),
                    Value::Bool(true) => eval(env, rhs),
                    _ => Err(runtime_err("&& requires Bool operands", expr.span)),
                };
            }
            if *op == BinOp::Or {
                let l = eval(env, lhs)?;
                return match l {
                    Value::Bool(true) => Ok(Value::Bool(true)),
                    Value::Bool(false) => eval(env, rhs),
                    _ => Err(runtime_err("|| requires Bool operands", expr.span)),
                };
            }

            let l = eval(env, lhs)?;
            let r = eval(env, rhs)?;
            eval_binop(op, l, r, expr.span)
        }

        // ── Unary operation ──
        Expr::UnaryOp { op, operand } => {
            let val = eval(env, operand)?;
            match (op, &val) {
                (UnaryOp::Neg, Value::Int(n)) => Ok(Value::Int(-n)),
                (UnaryOp::Neg, Value::Float(n)) => Ok(Value::Float(-n)),
                (UnaryOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
                _ => Err(runtime_err(
                    &format!("invalid unary operation on {}", val.type_name()),
                    expr.span,
                )),
            }
        }

        // ── Pipe ──
        Expr::Pipe { lhs, rhs } => {
            let arg = eval(env, lhs)?;
            let func = eval(env, rhs)?;
            apply_function(func, vec![arg], expr.span)
        }

        // ── If expression ──
        Expr::If {
            cond,
            then_branch,
            else_branch,
        } => {
            let cond_val = eval(env, cond)?;
            match cond_val {
                Value::Bool(true) => eval(env, then_branch),
                Value::Bool(false) => eval(env, else_branch),
                _ => Err(runtime_err("if condition must be Bool", cond.span)),
            }
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
                // For recursive let, evaluate the binding and patch self-reference
                let val = eval(env, value)?;
                let val = match val {
                    Value::Closure {
                        params,
                        body: cb,
                        env: cenv,
                        ..
                    } => Value::Closure {
                        params,
                        body: cb,
                        env: cenv,
                        recursive_name: Some(name.node.clone()),
                    },
                    other => other,
                };
                let new_env = env.extend();
                new_env.set(name.node.clone(), val);
                eval(&new_env, body)
            } else {
                let val = eval(env, value)?;
                let new_env = env.extend();
                new_env.set(name.node.clone(), val);
                eval(&new_env, body)
            }
        }

        // ── Match expression ──
        Expr::Match { scrutinee, arms } => {
            let scrut_val = eval(env, scrutinee)?;
            for arm in arms {
                if let Some(bindings) = match_pattern(&arm.pattern, &scrut_val) {
                    let arm_env = env.extend();
                    for (name, val) in bindings {
                        arm_env.set(name, val);
                    }
                    return eval(&arm_env, &arm.body);
                }
            }
            Err(LyraError::MatchFailure { span: expr.span })
        }

        // ── String interpolation ──
        Expr::Interpolation(parts) => {
            let mut result = String::new();
            for part in parts {
                match part {
                    crate::ast::InterpolationPart::Literal(s) => result.push_str(s),
                    crate::ast::InterpolationPart::Expr(e) => {
                        let val = eval(env, e)?;
                        result.push_str(&val.display_unquoted());
                    }
                }
            }
            Ok(Value::String(result))
        }

        // ── Record literal ──
        Expr::Record(fields) => {
            let mut map = std::collections::BTreeMap::new();
            for (name, val_expr) in fields {
                let val = eval(env, val_expr)?;
                map.insert(name.clone(), val);
            }
            Ok(Value::Record(map))
        }

        // ── Field access ──
        Expr::FieldAccess { expr: obj, field } => {
            let val = eval(env, obj)?;
            match val {
                Value::Record(map) => map.get(field).cloned().ok_or_else(|| {
                    runtime_err(&format!("record has no field '{}'", field), expr.span)
                }),
                _ => Err(runtime_err(
                    &format!("cannot access field '{}' on {}", field, val.type_name()),
                    expr.span,
                )),
            }
        }
    }
}

/// Apply a function value to arguments.
pub fn apply_function(func: Value, args: Vec<Value>, span: Span) -> Result<Value, LyraError> {
    match func {
        Value::Closure {
            params,
            body,
            env,
            recursive_name,
        } => {
            if args.len() < params.len() {
                // Partial application
                return Ok(Value::PartialApp {
                    func: Box::new(Value::Closure {
                        params,
                        body,
                        env,
                        recursive_name,
                    }),
                    applied_args: args,
                });
            }

            let call_env = env.extend();

            // Self-reference for recursive functions
            if let Some(ref rec_name) = recursive_name {
                call_env.set(
                    rec_name.clone(),
                    Value::Closure {
                        params: params.clone(),
                        body: body.clone(),
                        env: env.clone(),
                        recursive_name: recursive_name.clone(),
                    },
                );
            }

            for (param, arg) in params.iter().zip(args.iter()) {
                call_env.set(param.clone(), arg.clone());
            }

            // If more args than params, apply rest to the result (currying)
            let result = eval(&call_env, &body)?;
            if args.len() > params.len() {
                apply_function(result, args[params.len()..].to_vec(), span)
            } else {
                Ok(result)
            }
        }

        Value::Builtin {
            func: f,
            arity,
            name,
        } => {
            if args.len() < arity {
                return Ok(Value::PartialApp {
                    func: Box::new(Value::Builtin {
                        name,
                        arity,
                        func: f,
                    }),
                    applied_args: args,
                });
            }
            let (call_args, rest) = args.split_at(arity);
            let result = f(call_args.to_vec()).map_err(|msg| LyraError::RuntimeError {
                message: msg,
                span,
            })?;
            if rest.is_empty() {
                Ok(result)
            } else {
                apply_function(result, rest.to_vec(), span)
            }
        }

        Value::PartialApp {
            func,
            mut applied_args,
        } => {
            applied_args.extend(args);
            apply_function(*func, applied_args, span)
        }

        // ADT constructors can be applied like functions
        Value::Adt {
            constructor,
            fields,
        } if fields.is_empty() && !args.is_empty() => Ok(Value::Adt {
            constructor,
            fields: args,
        }),

        // VM compiled functions — execute via mini VM with globals from calling VM
        Value::Function(proto) => {
            let arity = proto.arity as usize;
            if args.len() < arity {
                return Ok(Value::PartialApp {
                    func: Box::new(Value::Function(proto)),
                    applied_args: args,
                });
            }
            let mut vm = crate::vm::VM::new();
            crate::stdlib::register_vm_stdlib(&mut vm);
            if let Some(globals) = get_vm_globals() {
                for (name, val) in globals {
                    vm.define_global(name, val);
                }
            }
            let result = vm.call_function(proto, args[..arity].to_vec())?;
            if args.len() > arity {
                apply_function(result, args[arity..].to_vec(), span)
            } else {
                Ok(result)
            }
        }

        Value::ClosureVal { proto, upvalues } => {
            let arity = proto.arity as usize;
            if args.len() < arity {
                return Ok(Value::PartialApp {
                    func: Box::new(Value::ClosureVal { proto, upvalues }),
                    applied_args: args,
                });
            }
            let mut vm = crate::vm::VM::new();
            crate::stdlib::register_vm_stdlib(&mut vm);
            if let Some(globals) = get_vm_globals() {
                for (name, val) in globals {
                    vm.define_global(name, val);
                }
            }
            let result = vm.call_closure(proto, upvalues, args[..arity].to_vec())?;
            if args.len() > arity {
                apply_function(result, args[arity..].to_vec(), span)
            } else {
                Ok(result)
            }
        }

        _ => Err(LyraError::NotCallable { span }),
    }
}

fn eval_binop(op: &BinOp, lhs: Value, rhs: Value, span: Span) -> Result<Value, LyraError> {
    match (op, &lhs, &rhs) {
        // Int arithmetic
        (BinOp::Add, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
        (BinOp::Sub, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
        (BinOp::Mul, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
        (BinOp::Div, Value::Int(_), Value::Int(0)) => Err(LyraError::DivisionByZero { span }),
        (BinOp::Div, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a / b)),
        (BinOp::Mod, Value::Int(_), Value::Int(0)) => Err(LyraError::DivisionByZero { span }),
        (BinOp::Mod, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a % b)),

        // Float arithmetic
        (BinOp::Add, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
        (BinOp::Sub, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
        (BinOp::Mul, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
        (BinOp::Div, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
        (BinOp::Mod, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a % b)),

        // String concatenation via +
        (BinOp::Add, Value::String(a), Value::String(b)) => {
            Ok(Value::String(format!("{}{}", a, b)))
        }

        // Int comparison
        (BinOp::Lt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
        (BinOp::Gt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
        (BinOp::Le, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
        (BinOp::Ge, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),

        // Float comparison
        (BinOp::Lt, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
        (BinOp::Gt, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
        (BinOp::Le, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
        (BinOp::Ge, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),

        // String comparison
        (BinOp::Lt, Value::String(a), Value::String(b)) => Ok(Value::Bool(a < b)),
        (BinOp::Gt, Value::String(a), Value::String(b)) => Ok(Value::Bool(a > b)),

        // Equality (polymorphic)
        (BinOp::Eq, a, b) => Ok(Value::Bool(a == b)),
        (BinOp::NotEq, a, b) => Ok(Value::Bool(a != b)),

        // Cons
        (BinOp::Cons, val, Value::List(list)) => {
            let mut new_list = vec![val.clone()];
            new_list.extend(list.clone());
            Ok(Value::List(new_list))
        }

        _ => Err(runtime_err(
            &format!(
                "invalid operation {} {} {}",
                lhs.type_name(),
                op.as_str(),
                rhs.type_name()
            ),
            span,
        )),
    }
}

fn runtime_err(message: &str, span: Span) -> LyraError {
    LyraError::RuntimeError {
        message: message.to_string(),
        span,
    }
}

/// Evaluate a top-level declaration, updating the environment.
pub fn eval_decl(env: &Env, decl: &Decl) -> Result<Option<Value>, LyraError> {
    match decl {
        Decl::Let {
            name,
            recursive,
            body,
            ..
        } => {
            let val = eval(env, body)?;
            let val = if *recursive {
                match val {
                    Value::Closure {
                        params,
                        body: cb,
                        env: cenv,
                        ..
                    } => Value::Closure {
                        params,
                        body: cb,
                        env: cenv,
                        recursive_name: Some(name.node.clone()),
                    },
                    other => other,
                }
            } else {
                val
            };
            env.set(name.node.clone(), val);
            Ok(None)
        }

        Decl::Type { variants, .. } => {
            // Register constructor functions
            for variant in variants {
                let ctor_name = variant.name.node.clone();
                let arity = variant.fields.len();
                if arity == 0 {
                    // Nullary constructor — just a value
                    env.set(
                        ctor_name.clone(),
                        Value::Adt {
                            constructor: ctor_name,
                            fields: vec![],
                        },
                    );
                } else {
                    // Constructor with fields — stored as empty ADT,
                    // apply_function handles filling in fields
                    env.set(
                        ctor_name.clone(),
                        Value::Adt {
                            constructor: ctor_name.clone(),
                            fields: vec![],
                        },
                    );
                }
            }
            Ok(None)
        }

        Decl::Expr(expr) => {
            let val = eval(env, expr)?;
            Ok(Some(val))
        }

        Decl::Import { path, span } => {
            Err(LyraError::RuntimeError {
                message: format!("import not yet supported: \"{}\"", path),
                span: *span,
            })
        }
    }
}

// ── Higher-order stdlib functions (need eval access) ──

pub fn register_hof_builtins(env: &Env) {
    // map: (a -> b) -> [a] -> [b]
    env.set(
        "map".to_string(),
        Value::Builtin {
            name: "map".to_string(),
            arity: 2,
            func: |args| {
                let func = &args[0];
                let list = match &args[1] {
                    Value::List(l) => l,
                    v => return Err(format!("map: expected List, got {}", v.type_name())),
                };
                let mut results = Vec::new();
                for item in list {
                    let result = apply_function(
                        func.clone(),
                        vec![item.clone()],
                        Span::default(),
                    )
                    .map_err(|e| format!("{}", e))?;
                    results.push(result);
                }
                Ok(Value::List(results))
            },
        },
    );

    // filter: (a -> Bool) -> [a] -> [a]
    env.set(
        "filter".to_string(),
        Value::Builtin {
            name: "filter".to_string(),
            arity: 2,
            func: |args| {
                let func = &args[0];
                let list = match &args[1] {
                    Value::List(l) => l,
                    v => return Err(format!("filter: expected List, got {}", v.type_name())),
                };
                let mut results = Vec::new();
                for item in list {
                    let keep = apply_function(
                        func.clone(),
                        vec![item.clone()],
                        Span::default(),
                    )
                    .map_err(|e| format!("{}", e))?;
                    if matches!(keep, Value::Bool(true)) {
                        results.push(item.clone());
                    }
                }
                Ok(Value::List(results))
            },
        },
    );

    // fold: b -> (b -> a -> b) -> [a] -> b
    env.set(
        "fold".to_string(),
        Value::Builtin {
            name: "fold".to_string(),
            arity: 3,
            func: |args| {
                let mut acc = args[0].clone();
                let func = &args[1];
                let list = match &args[2] {
                    Value::List(l) => l,
                    v => return Err(format!("fold: expected List, got {}", v.type_name())),
                };
                for item in list {
                    acc = apply_function(
                        func.clone(),
                        vec![acc, item.clone()],
                        Span::default(),
                    )
                    .map_err(|e| format!("{}", e))?;
                }
                Ok(acc)
            },
        },
    );

    // zip: [a] -> [b] -> [(a, b)]
    env.set(
        "zip".to_string(),
        Value::Builtin {
            name: "zip".to_string(),
            arity: 2,
            func: |args| {
                let a = match &args[0] {
                    Value::List(l) => l,
                    v => return Err(format!("zip: expected List, got {}", v.type_name())),
                };
                let b = match &args[1] {
                    Value::List(l) => l,
                    v => return Err(format!("zip: expected List, got {}", v.type_name())),
                };
                let pairs: Vec<Value> = a
                    .iter()
                    .zip(b.iter())
                    .map(|(x, y)| Value::Tuple(vec![x.clone(), y.clone()]))
                    .collect();
                Ok(Value::List(pairs))
            },
        },
    );

    // any: (a -> Bool) -> [a] -> Bool
    env.set(
        "any".to_string(),
        Value::Builtin {
            name: "any".to_string(),
            arity: 2,
            func: |args| {
                let func = &args[0];
                let list = match &args[1] {
                    Value::List(l) => l,
                    v => return Err(format!("any: expected List, got {}", v.type_name())),
                };
                for item in list {
                    let result = apply_function(
                        func.clone(),
                        vec![item.clone()],
                        Span::default(),
                    )
                    .map_err(|e| format!("{}", e))?;
                    if matches!(result, Value::Bool(true)) {
                        return Ok(Value::Bool(true));
                    }
                }
                Ok(Value::Bool(false))
            },
        },
    );

    // all: (a -> Bool) -> [a] -> Bool
    env.set(
        "all".to_string(),
        Value::Builtin {
            name: "all".to_string(),
            arity: 2,
            func: |args| {
                let func = &args[0];
                let list = match &args[1] {
                    Value::List(l) => l,
                    v => return Err(format!("all: expected List, got {}", v.type_name())),
                };
                for item in list {
                    let result = apply_function(
                        func.clone(),
                        vec![item.clone()],
                        Span::default(),
                    )
                    .map_err(|e| format!("{}", e))?;
                    if matches!(result, Value::Bool(false)) {
                        return Ok(Value::Bool(false));
                    }
                }
                Ok(Value::Bool(true))
            },
        },
    );

    // sort: [Int] -> [Int]
    env.set(
        "sort".to_string(),
        Value::Builtin {
            name: "sort".to_string(),
            arity: 1,
            func: |args| {
                let list = match &args[0] {
                    Value::List(l) => l.clone(),
                    v => return Err(format!("sort: expected List, got {}", v.type_name())),
                };
                let mut ints: Vec<i64> = list
                    .iter()
                    .map(|v| match v {
                        Value::Int(n) => Ok(*n),
                        v => Err(format!("sort: expected Int elements, got {}", v.type_name())),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                ints.sort();
                Ok(Value::List(ints.into_iter().map(Value::Int).collect()))
            },
        },
    );
}
