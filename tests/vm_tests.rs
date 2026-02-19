use lyra::compiler::compile;
use lyra::eval::env::Env;
use lyra::eval::value::Value;
use lyra::lexer::tokenize;
use lyra::parser::parse;
use lyra::stdlib::{register_stdlib, register_vm_stdlib};
use lyra::types::env::TypeEnv;
use lyra::types::infer::Inferencer;
use lyra::types::TypeVarGen;
use lyra::vm::VM;

/// Compile and run source code on the VM, returning the last value on the stack.
fn vm_run(source: &str) -> Result<Value, String> {
    let tokens = tokenize(source).map_err(|errs| format!("{:?}", errs))?;
    let decls = parse(tokens).map_err(|e| format!("{:?}", e))?;

    // Type check first
    let mut type_env = TypeEnv::new();
    let runtime_env = Env::new();
    let mut gen = TypeVarGen::new();
    let mut inferencer = Inferencer::new();
    register_stdlib(&mut type_env, &runtime_env, &mut gen);
    for decl in &decls {
        inferencer
            .infer_decl(&mut type_env, decl)
            .map_err(|e| format!("{:?}", e))?;
    }

    // Compile
    let proto = compile(&decls).map_err(|e| e.to_string())?;

    // Run
    let mut vm = VM::new();
    register_vm_stdlib(&mut vm);
    vm.run(proto).map_err(|e| format!("{:?}", e))
}

// ── Basic values ──

#[test]
fn vm_integer() {
    assert_eq!(vm_run("42").unwrap(), Value::Int(42));
}

#[test]
fn vm_float() {
    assert_eq!(vm_run("3.14").unwrap(), Value::Float(3.14));
}

#[test]
fn vm_bool() {
    assert_eq!(vm_run("true").unwrap(), Value::Bool(true));
}

#[test]
fn vm_string() {
    assert_eq!(vm_run("\"hello\"").unwrap(), Value::String("hello".to_string()));
}

#[test]
fn vm_unit() {
    assert_eq!(vm_run("()").unwrap(), Value::Unit);
}

// ── Arithmetic ──

#[test]
fn vm_addition() {
    assert_eq!(vm_run("1 + 2").unwrap(), Value::Int(3));
}

#[test]
fn vm_subtraction() {
    assert_eq!(vm_run("10 - 3").unwrap(), Value::Int(7));
}

#[test]
fn vm_multiplication() {
    assert_eq!(vm_run("4 * 5").unwrap(), Value::Int(20));
}

#[test]
fn vm_division() {
    assert_eq!(vm_run("10 / 3").unwrap(), Value::Int(3));
}

#[test]
fn vm_modulo() {
    assert_eq!(vm_run("10 % 3").unwrap(), Value::Int(1));
}

#[test]
fn vm_complex_arithmetic() {
    assert_eq!(vm_run("(2 + 3) * (4 - 1)").unwrap(), Value::Int(15));
}

// ── Comparison and logic ──

#[test]
fn vm_equality() {
    assert_eq!(vm_run("1 == 1").unwrap(), Value::Bool(true));
    assert_eq!(vm_run("1 == 2").unwrap(), Value::Bool(false));
}

#[test]
fn vm_comparison() {
    assert_eq!(vm_run("1 < 2").unwrap(), Value::Bool(true));
    assert_eq!(vm_run("2 > 1").unwrap(), Value::Bool(true));
    assert_eq!(vm_run("1 <= 1").unwrap(), Value::Bool(true));
    assert_eq!(vm_run("1 >= 2").unwrap(), Value::Bool(false));
}

#[test]
fn vm_boolean_logic() {
    assert_eq!(vm_run("true && false").unwrap(), Value::Bool(false));
    assert_eq!(vm_run("true || false").unwrap(), Value::Bool(true));
    assert_eq!(vm_run("!true").unwrap(), Value::Bool(false));
}

// ── Let bindings ──

#[test]
fn vm_let_binding() {
    assert_eq!(vm_run("let x = 42\nx").unwrap(), Value::Int(42));
}

#[test]
fn vm_let_in_expression() {
    // let...in is only valid inside expression context, so wrap in a function call
    assert_eq!(
        vm_run("let f = fn (n) -> let x = 10 in let y = 20 in x + y + n\nf(0)").unwrap(),
        Value::Int(30)
    );
}

// ── Functions ──

#[test]
fn vm_lambda_and_call() {
    assert_eq!(
        vm_run("let f = fn (x) -> x * 2\nf(21)").unwrap(),
        Value::Int(42)
    );
}

#[test]
fn vm_multi_arg_function() {
    assert_eq!(
        vm_run("let add = fn (a, b) -> a + b\nadd(3, 4)").unwrap(),
        Value::Int(7)
    );
}

#[test]
fn vm_recursive_function() {
    assert_eq!(
        vm_run("let rec fact = fn (n) -> if n <= 1 then 1 else n * fact(n - 1)\nfact(10)").unwrap(),
        Value::Int(3628800)
    );
}

#[test]
fn vm_closure() {
    assert_eq!(
        vm_run("let make_adder = fn (n) -> fn (x) -> x + n\nlet add5 = make_adder(5)\nadd5(10)").unwrap(),
        Value::Int(15)
    );
}

// ── If expression ──

#[test]
fn vm_if_true() {
    assert_eq!(
        vm_run("if true then 42 else 0").unwrap(),
        Value::Int(42)
    );
}

#[test]
fn vm_if_false() {
    assert_eq!(
        vm_run("if false then 42 else 0").unwrap(),
        Value::Int(0)
    );
}

// ── Lists ──

#[test]
fn vm_list_literal() {
    assert_eq!(
        vm_run("[1, 2, 3]").unwrap(),
        Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
    );
}

#[test]
fn vm_cons_operator() {
    assert_eq!(
        vm_run("0 :: [1, 2]").unwrap(),
        Value::List(vec![Value::Int(0), Value::Int(1), Value::Int(2)])
    );
}

#[test]
fn vm_list_length() {
    assert_eq!(vm_run("length([1, 2, 3])").unwrap(), Value::Int(3));
}

#[test]
fn vm_list_head_tail() {
    assert_eq!(vm_run("head([1, 2, 3])").unwrap(), Value::Int(1));
    assert_eq!(
        vm_run("tail([1, 2, 3])").unwrap(),
        Value::List(vec![Value::Int(2), Value::Int(3)])
    );
}

// ── Tuples ──

#[test]
fn vm_tuple() {
    assert_eq!(
        vm_run("(1, true)").unwrap(),
        Value::Tuple(vec![Value::Int(1), Value::Bool(true)])
    );
}

// ── Match expressions ──

#[test]
fn vm_match_int() {
    assert_eq!(
        vm_run("match 2 with | 1 -> 10 | 2 -> 20 | _ -> 0").unwrap(),
        Value::Int(20)
    );
}

#[test]
fn vm_match_wildcard() {
    assert_eq!(
        vm_run("match 99 with | 1 -> 10 | _ -> 0").unwrap(),
        Value::Int(0)
    );
}

#[test]
fn vm_match_list() {
    assert_eq!(
        vm_run("match [1, 2, 3] with | [] -> 0 | hd :: _ -> hd").unwrap(),
        Value::Int(1)
    );
}

// ── ADTs ──

#[test]
fn vm_adt_construction() {
    let result = vm_run(
        "type Color = Red | Green | Blue\nlet c = Red\nc"
    ).unwrap();
    match result {
        Value::Adt { constructor, fields } => {
            assert_eq!(constructor, "Red");
            assert!(fields.is_empty());
        }
        _ => panic!("expected ADT value"),
    }
}

#[test]
fn vm_adt_with_fields() {
    let result = vm_run(
        "type Shape = Circle Int\nlet s = Circle(5)\ns"
    ).unwrap();
    match result {
        Value::Adt { constructor, fields } => {
            assert_eq!(constructor, "Circle");
            assert_eq!(fields, vec![Value::Int(5)]);
        }
        _ => panic!("expected ADT value"),
    }
}

#[test]
fn vm_adt_match() {
    assert_eq!(
        vm_run(
            "type Shape = Circle Int | Rectangle Int Int\n\
             let area = fn (s) -> match s with | Circle(r) -> r * r * 3 | Rectangle(w, h) -> w * h\n\
             area(Rectangle(4, 5))"
        ).unwrap(),
        Value::Int(20)
    );
}

// ── Pipe operator ──

#[test]
fn vm_pipe() {
    assert_eq!(
        vm_run("[1, 2, 3] |> length").unwrap(),
        Value::Int(3)
    );
}

#[test]
fn vm_pipe_chain() {
    assert_eq!(
        vm_run(
            "[1, 2, 3, 4, 5] |> map(fn (x) -> x * x) |> fold(0, fn (acc, x) -> acc + x)"
        ).unwrap(),
        Value::Int(55)
    );
}

// ── String interpolation ──

#[test]
fn vm_interpolation() {
    assert_eq!(
        vm_run("let name = \"world\"\n\"hello {name}\"").unwrap(),
        Value::String("hello world".to_string())
    );
}

// ── Records ──

#[test]
fn vm_record() {
    let result = vm_run("{ x: 1, y: 2 }").unwrap();
    match result {
        Value::Record(map) => {
            assert_eq!(map.get("x"), Some(&Value::Int(1)));
            assert_eq!(map.get("y"), Some(&Value::Int(2)));
        }
        _ => panic!("expected Record"),
    }
}

#[test]
fn vm_record_field_access() {
    assert_eq!(
        vm_run("let r = { x: 10, y: 20 }\nr.x + r.y").unwrap(),
        Value::Int(30)
    );
}

// ── New stdlib ──

#[test]
fn vm_stdlib_take() {
    assert_eq!(
        vm_run("take(2, [1, 2, 3, 4, 5])").unwrap(),
        Value::List(vec![Value::Int(1), Value::Int(2)])
    );
}

#[test]
fn vm_stdlib_drop() {
    assert_eq!(
        vm_run("drop(3, [1, 2, 3, 4, 5])").unwrap(),
        Value::List(vec![Value::Int(4), Value::Int(5)])
    );
}

#[test]
fn vm_stdlib_sum() {
    assert_eq!(
        vm_run("sum([1, 2, 3, 4, 5])").unwrap(),
        Value::Int(15)
    );
}

#[test]
fn vm_stdlib_product() {
    assert_eq!(
        vm_run("product([1, 2, 3, 4, 5])").unwrap(),
        Value::Int(120)
    );
}

#[test]
fn vm_stdlib_flatten() {
    assert_eq!(
        vm_run("flatten([[1, 2], [3, 4], [5]])").unwrap(),
        Value::List(vec![
            Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4), Value::Int(5)
        ])
    );
}

#[test]
fn vm_stdlib_string_conversions() {
    assert_eq!(
        vm_run("string_to_int(\"42\")").unwrap(),
        Value::Int(42)
    );
    assert_eq!(
        vm_run("int_to_string(42)").unwrap(),
        Value::String("42".to_string())
    );
}

#[test]
fn vm_stdlib_str_utilities() {
    assert_eq!(
        vm_run("str_trim(\"  hello  \")").unwrap(),
        Value::String("hello".to_string())
    );
    assert_eq!(
        vm_run("str_uppercase(\"hello\")").unwrap(),
        Value::String("HELLO".to_string())
    );
    assert_eq!(
        vm_run("str_lowercase(\"HELLO\")").unwrap(),
        Value::String("hello".to_string())
    );
}
