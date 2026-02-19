use crate::eval::env::Env;
use crate::eval::builtins::all_builtins;
use crate::eval::register_hof_builtins;
use crate::types::env::TypeEnv;
use crate::types::{MonoType, TypeScheme, TypeVarGen};
use crate::vm::VM;

/// Register all standard library functions into the type and runtime environments.
pub fn register_stdlib(type_env: &mut TypeEnv, runtime_env: &Env, gen: &mut TypeVarGen) {
    // Register plain builtins
    for (name, value) in all_builtins() {
        runtime_env.set(name, value);
    }

    // Register higher-order builtins (map, filter, fold, etc.)
    register_hof_builtins(runtime_env);

    // Register types for builtins
    register_builtin_types(type_env, gen);
}

fn register_builtin_types(env: &mut TypeEnv, gen: &mut TypeVarGen) {
    // IO
    let a = gen.fresh();
    env.insert("print".to_string(), TypeScheme {
        vars: vec![a],
        ty: MonoType::Arrow(Box::new(MonoType::Var(a)), Box::new(MonoType::Unit)),
    });
    let a = gen.fresh();
    env.insert("println".to_string(), TypeScheme {
        vars: vec![a],
        ty: MonoType::Arrow(Box::new(MonoType::Var(a)), Box::new(MonoType::Unit)),
    });

    // to_string : a -> String
    let a = gen.fresh();
    env.insert("to_string".to_string(), TypeScheme {
        vars: vec![a],
        ty: MonoType::Arrow(Box::new(MonoType::Var(a)), Box::new(MonoType::String)),
    });

    // String functions
    env.insert("str_length".to_string(), TypeScheme::mono(
        MonoType::Arrow(Box::new(MonoType::String), Box::new(MonoType::Int)),
    ));
    env.insert("str_concat".to_string(), TypeScheme::mono(
        MonoType::Arrow(
            Box::new(MonoType::String),
            Box::new(MonoType::Arrow(Box::new(MonoType::String), Box::new(MonoType::String))),
        ),
    ));
    env.insert("str_contains".to_string(), TypeScheme::mono(
        MonoType::Arrow(
            Box::new(MonoType::String),
            Box::new(MonoType::Arrow(Box::new(MonoType::String), Box::new(MonoType::Bool))),
        ),
    ));
    env.insert("str_split".to_string(), TypeScheme::mono(
        MonoType::Arrow(
            Box::new(MonoType::String),
            Box::new(MonoType::Arrow(
                Box::new(MonoType::String),
                Box::new(MonoType::List(Box::new(MonoType::String))),
            )),
        ),
    ));
    env.insert("str_chars".to_string(), TypeScheme::mono(
        MonoType::Arrow(
            Box::new(MonoType::String),
            Box::new(MonoType::List(Box::new(MonoType::String))),
        ),
    ));

    // List functions
    let a = gen.fresh();
    env.insert("length".to_string(), TypeScheme {
        vars: vec![a],
        ty: MonoType::Arrow(
            Box::new(MonoType::List(Box::new(MonoType::Var(a)))),
            Box::new(MonoType::Int),
        ),
    });
    let a = gen.fresh();
    env.insert("head".to_string(), TypeScheme {
        vars: vec![a],
        ty: MonoType::Arrow(
            Box::new(MonoType::List(Box::new(MonoType::Var(a)))),
            Box::new(MonoType::Var(a)),
        ),
    });
    let a = gen.fresh();
    env.insert("tail".to_string(), TypeScheme {
        vars: vec![a],
        ty: MonoType::Arrow(
            Box::new(MonoType::List(Box::new(MonoType::Var(a)))),
            Box::new(MonoType::List(Box::new(MonoType::Var(a)))),
        ),
    });
    let a = gen.fresh();
    env.insert("reverse".to_string(), TypeScheme {
        vars: vec![a],
        ty: MonoType::Arrow(
            Box::new(MonoType::List(Box::new(MonoType::Var(a)))),
            Box::new(MonoType::List(Box::new(MonoType::Var(a)))),
        ),
    });
    let a = gen.fresh();
    env.insert("append".to_string(), TypeScheme {
        vars: vec![a],
        ty: MonoType::Arrow(
            Box::new(MonoType::List(Box::new(MonoType::Var(a)))),
            Box::new(MonoType::Arrow(
                Box::new(MonoType::List(Box::new(MonoType::Var(a)))),
                Box::new(MonoType::List(Box::new(MonoType::Var(a)))),
            )),
        ),
    });
    env.insert("range".to_string(), TypeScheme::mono(
        MonoType::Arrow(
            Box::new(MonoType::Int),
            Box::new(MonoType::Arrow(
                Box::new(MonoType::Int),
                Box::new(MonoType::List(Box::new(MonoType::Int))),
            )),
        ),
    ));
    let a = gen.fresh();
    env.insert("nth".to_string(), TypeScheme {
        vars: vec![a],
        ty: MonoType::Arrow(
            Box::new(MonoType::List(Box::new(MonoType::Var(a)))),
            Box::new(MonoType::Arrow(
                Box::new(MonoType::Int),
                Box::new(MonoType::Var(a)),
            )),
        ),
    });

    // map : (a -> b) -> [a] -> [b]
    let a = gen.fresh();
    let b = gen.fresh();
    env.insert("map".to_string(), TypeScheme {
        vars: vec![a, b],
        ty: MonoType::Arrow(
            Box::new(MonoType::Arrow(Box::new(MonoType::Var(a)), Box::new(MonoType::Var(b)))),
            Box::new(MonoType::Arrow(
                Box::new(MonoType::List(Box::new(MonoType::Var(a)))),
                Box::new(MonoType::List(Box::new(MonoType::Var(b)))),
            )),
        ),
    });

    // filter : (a -> Bool) -> [a] -> [a]
    let a = gen.fresh();
    env.insert("filter".to_string(), TypeScheme {
        vars: vec![a],
        ty: MonoType::Arrow(
            Box::new(MonoType::Arrow(Box::new(MonoType::Var(a)), Box::new(MonoType::Bool))),
            Box::new(MonoType::Arrow(
                Box::new(MonoType::List(Box::new(MonoType::Var(a)))),
                Box::new(MonoType::List(Box::new(MonoType::Var(a)))),
            )),
        ),
    });

    // fold : b -> (b -> a -> b) -> [a] -> b
    let a = gen.fresh();
    let b = gen.fresh();
    env.insert("fold".to_string(), TypeScheme {
        vars: vec![a, b],
        ty: MonoType::Arrow(
            Box::new(MonoType::Var(b)),
            Box::new(MonoType::Arrow(
                Box::new(MonoType::Arrow(
                    Box::new(MonoType::Var(b)),
                    Box::new(MonoType::Arrow(Box::new(MonoType::Var(a)), Box::new(MonoType::Var(b)))),
                )),
                Box::new(MonoType::Arrow(
                    Box::new(MonoType::List(Box::new(MonoType::Var(a)))),
                    Box::new(MonoType::Var(b)),
                )),
            )),
        ),
    });

    // zip : [a] -> [b] -> [(a, b)]
    let a = gen.fresh();
    let b = gen.fresh();
    env.insert("zip".to_string(), TypeScheme {
        vars: vec![a, b],
        ty: MonoType::Arrow(
            Box::new(MonoType::List(Box::new(MonoType::Var(a)))),
            Box::new(MonoType::Arrow(
                Box::new(MonoType::List(Box::new(MonoType::Var(b)))),
                Box::new(MonoType::List(Box::new(MonoType::Tuple(vec![MonoType::Var(a), MonoType::Var(b)])))),
            )),
        ),
    });

    // any : (a -> Bool) -> [a] -> Bool
    let a = gen.fresh();
    env.insert("any".to_string(), TypeScheme {
        vars: vec![a],
        ty: MonoType::Arrow(
            Box::new(MonoType::Arrow(Box::new(MonoType::Var(a)), Box::new(MonoType::Bool))),
            Box::new(MonoType::Arrow(
                Box::new(MonoType::List(Box::new(MonoType::Var(a)))),
                Box::new(MonoType::Bool),
            )),
        ),
    });

    // all : (a -> Bool) -> [a] -> Bool
    let a = gen.fresh();
    env.insert("all".to_string(), TypeScheme {
        vars: vec![a],
        ty: MonoType::Arrow(
            Box::new(MonoType::Arrow(Box::new(MonoType::Var(a)), Box::new(MonoType::Bool))),
            Box::new(MonoType::Arrow(
                Box::new(MonoType::List(Box::new(MonoType::Var(a)))),
                Box::new(MonoType::Bool),
            )),
        ),
    });

    // sort : [Int] -> [Int]
    env.insert("sort".to_string(), TypeScheme::mono(
        MonoType::Arrow(
            Box::new(MonoType::List(Box::new(MonoType::Int))),
            Box::new(MonoType::List(Box::new(MonoType::Int))),
        ),
    ));

    // take : Int -> [a] -> [a]
    let a = gen.fresh();
    env.insert("take".to_string(), TypeScheme {
        vars: vec![a],
        ty: MonoType::Arrow(
            Box::new(MonoType::Int),
            Box::new(MonoType::Arrow(
                Box::new(MonoType::List(Box::new(MonoType::Var(a)))),
                Box::new(MonoType::List(Box::new(MonoType::Var(a)))),
            )),
        ),
    });

    // drop : Int -> [a] -> [a]
    let a = gen.fresh();
    env.insert("drop".to_string(), TypeScheme {
        vars: vec![a],
        ty: MonoType::Arrow(
            Box::new(MonoType::Int),
            Box::new(MonoType::Arrow(
                Box::new(MonoType::List(Box::new(MonoType::Var(a)))),
                Box::new(MonoType::List(Box::new(MonoType::Var(a)))),
            )),
        ),
    });

    // flatten : [[a]] -> [a]
    let a = gen.fresh();
    env.insert("flatten".to_string(), TypeScheme {
        vars: vec![a],
        ty: MonoType::Arrow(
            Box::new(MonoType::List(Box::new(MonoType::List(Box::new(MonoType::Var(a)))))),
            Box::new(MonoType::List(Box::new(MonoType::Var(a)))),
        ),
    });

    // sum : [Int] -> Int
    env.insert("sum".to_string(), TypeScheme::mono(
        MonoType::Arrow(
            Box::new(MonoType::List(Box::new(MonoType::Int))),
            Box::new(MonoType::Int),
        ),
    ));

    // product : [Int] -> Int
    env.insert("product".to_string(), TypeScheme::mono(
        MonoType::Arrow(
            Box::new(MonoType::List(Box::new(MonoType::Int))),
            Box::new(MonoType::Int),
        ),
    ));

    // string_to_int : String -> Int
    env.insert("string_to_int".to_string(), TypeScheme::mono(
        MonoType::Arrow(Box::new(MonoType::String), Box::new(MonoType::Int)),
    ));

    // int_to_string : Int -> String
    env.insert("int_to_string".to_string(), TypeScheme::mono(
        MonoType::Arrow(Box::new(MonoType::Int), Box::new(MonoType::String)),
    ));

    // str_trim : String -> String
    env.insert("str_trim".to_string(), TypeScheme::mono(
        MonoType::Arrow(Box::new(MonoType::String), Box::new(MonoType::String)),
    ));

    // str_uppercase : String -> String
    env.insert("str_uppercase".to_string(), TypeScheme::mono(
        MonoType::Arrow(Box::new(MonoType::String), Box::new(MonoType::String)),
    ));

    // str_lowercase : String -> String
    env.insert("str_lowercase".to_string(), TypeScheme::mono(
        MonoType::Arrow(Box::new(MonoType::String), Box::new(MonoType::String)),
    ));

    // str_replace : String -> String -> String -> String
    env.insert("str_replace".to_string(), TypeScheme::mono(
        MonoType::Arrow(
            Box::new(MonoType::String),
            Box::new(MonoType::Arrow(
                Box::new(MonoType::String),
                Box::new(MonoType::Arrow(Box::new(MonoType::String), Box::new(MonoType::String))),
            )),
        ),
    ));

    // str_starts_with : String -> String -> Bool
    env.insert("str_starts_with".to_string(), TypeScheme::mono(
        MonoType::Arrow(
            Box::new(MonoType::String),
            Box::new(MonoType::Arrow(Box::new(MonoType::String), Box::new(MonoType::Bool))),
        ),
    ));

    // str_ends_with : String -> String -> Bool
    env.insert("str_ends_with".to_string(), TypeScheme::mono(
        MonoType::Arrow(
            Box::new(MonoType::String),
            Box::new(MonoType::Arrow(Box::new(MonoType::String), Box::new(MonoType::Bool))),
        ),
    ));

    // str_substring : String -> Int -> Int -> String
    env.insert("str_substring".to_string(), TypeScheme::mono(
        MonoType::Arrow(
            Box::new(MonoType::String),
            Box::new(MonoType::Arrow(
                Box::new(MonoType::Int),
                Box::new(MonoType::Arrow(Box::new(MonoType::Int), Box::new(MonoType::String))),
            )),
        ),
    ));

    // Math functions
    env.insert("abs".to_string(), TypeScheme::mono(
        MonoType::Arrow(Box::new(MonoType::Int), Box::new(MonoType::Int)),
    ));
    env.insert("min".to_string(), TypeScheme::mono(
        MonoType::Arrow(
            Box::new(MonoType::Int),
            Box::new(MonoType::Arrow(Box::new(MonoType::Int), Box::new(MonoType::Int))),
        ),
    ));
    env.insert("max".to_string(), TypeScheme::mono(
        MonoType::Arrow(
            Box::new(MonoType::Int),
            Box::new(MonoType::Arrow(Box::new(MonoType::Int), Box::new(MonoType::Int))),
        ),
    ));
    env.insert("pow".to_string(), TypeScheme::mono(
        MonoType::Arrow(
            Box::new(MonoType::Int),
            Box::new(MonoType::Arrow(Box::new(MonoType::Int), Box::new(MonoType::Int))),
        ),
    ));
    env.insert("float_of_int".to_string(), TypeScheme::mono(
        MonoType::Arrow(Box::new(MonoType::Int), Box::new(MonoType::Float)),
    ));
    env.insert("int_of_float".to_string(), TypeScheme::mono(
        MonoType::Arrow(Box::new(MonoType::Float), Box::new(MonoType::Int)),
    ));
}

/// Register all stdlib functions as VM globals.
pub fn register_vm_stdlib(vm: &mut VM) {
    use crate::eval::value::Value;

    // Register all plain builtins
    for (name, value) in all_builtins() {
        vm.define_global(name, value);
    }

    // Register HOF builtins (map, filter, fold, zip, sort)
    vm.define_global(
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
                    let result = crate::eval::apply_function(
                        func.clone(),
                        vec![item.clone()],
                        crate::span::Span::default(),
                    )
                    .map_err(|e| format!("{}", e))?;
                    results.push(result);
                }
                Ok(Value::List(results))
            },
        },
    );

    vm.define_global(
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
                    let keep = crate::eval::apply_function(
                        func.clone(),
                        vec![item.clone()],
                        crate::span::Span::default(),
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

    vm.define_global(
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
                    acc = crate::eval::apply_function(
                        func.clone(),
                        vec![acc, item.clone()],
                        crate::span::Span::default(),
                    )
                    .map_err(|e| format!("{}", e))?;
                }
                Ok(acc)
            },
        },
    );

    vm.define_global(
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

    vm.define_global(
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
                    let result = crate::eval::apply_function(
                        func.clone(),
                        vec![item.clone()],
                        crate::span::Span::default(),
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

    vm.define_global(
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
                    let result = crate::eval::apply_function(
                        func.clone(),
                        vec![item.clone()],
                        crate::span::Span::default(),
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

    vm.define_global(
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
