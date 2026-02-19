use lyra::lexer::tokenize;
use lyra::parser::parse;
use lyra::stdlib::register_stdlib;
use lyra::types::env::TypeEnv;
use lyra::types::infer::Inferencer;
use lyra::types::TypeVarGen;
use lyra::eval::env::Env;

fn typecheck(source: &str) -> Result<(), String> {
    let tokens = tokenize(source).map_err(|errs| format!("{:?}", errs))?;
    let decls = parse(tokens).map_err(|e| format!("{:?}", e))?;

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
    Ok(())
}

fn typecheck_fails(source: &str) -> bool {
    typecheck(source).is_err()
}

// ── Basic type inference ──

#[test]
fn infer_integer() {
    assert!(typecheck("42").is_ok());
}

#[test]
fn infer_float() {
    assert!(typecheck("3.14").is_ok());
}

#[test]
fn infer_string() {
    assert!(typecheck("\"hello\"").is_ok());
}

#[test]
fn infer_bool() {
    assert!(typecheck("true").is_ok());
}

#[test]
fn infer_unit() {
    assert!(typecheck("()").is_ok());
}

// ── Arithmetic ──

#[test]
fn infer_int_arithmetic() {
    assert!(typecheck("1 + 2").is_ok());
    assert!(typecheck("1 - 2").is_ok());
    assert!(typecheck("2 * 3").is_ok());
    assert!(typecheck("6 / 2").is_ok());
    assert!(typecheck("7 % 3").is_ok());
}

#[test]
fn infer_int_string_add_fails() {
    assert!(typecheck_fails("1 + \"hello\""));
}

// ── Let bindings ──

#[test]
fn infer_let_binding() {
    assert!(typecheck("let x = 42").is_ok());
}

#[test]
fn infer_let_with_usage() {
    assert!(typecheck("let x = 42\nto_string(x)").is_ok());
}

#[test]
fn infer_recursive_let() {
    assert!(typecheck("let rec f = fn (x) -> if x <= 0 then 0 else f(x - 1)").is_ok());
}

// ── Functions ──

#[test]
fn infer_lambda() {
    assert!(typecheck("fn (x) -> x + 1").is_ok());
}

#[test]
fn infer_higher_order_function() {
    assert!(typecheck("let apply = fn (f, x) -> f(x)").is_ok());
}

#[test]
fn infer_function_application() {
    assert!(typecheck("let f = fn (x) -> x + 1\nf(42)").is_ok());
}

// ── Lists ──

#[test]
fn infer_list() {
    assert!(typecheck("[1, 2, 3]").is_ok());
}

#[test]
fn infer_heterogeneous_list_fails() {
    assert!(typecheck_fails("[1, \"hello\"]"));
}

#[test]
fn infer_list_operations() {
    assert!(typecheck("head([1, 2, 3])").is_ok());
    assert!(typecheck("tail([1, 2, 3])").is_ok());
    assert!(typecheck("length([1, 2, 3])").is_ok());
}

// ── Tuples ──

#[test]
fn infer_tuple() {
    assert!(typecheck("(1, \"hello\", true)").is_ok());
}

// ── If expression ──

#[test]
fn infer_if_expression() {
    assert!(typecheck("if true then 1 else 2").is_ok());
}

#[test]
fn infer_if_branch_mismatch_fails() {
    assert!(typecheck_fails("if true then 1 else \"hello\""));
}

#[test]
fn infer_if_non_bool_condition_fails() {
    assert!(typecheck_fails("if 42 then 1 else 2"));
}

// ── Match expression ──

#[test]
fn infer_match() {
    assert!(typecheck(
        "match 42 with | 0 -> true | _ -> false"
    ).is_ok());
}

// ── ADTs ──

#[test]
fn infer_adt_definition() {
    assert!(typecheck("type Color = Red | Green | Blue").is_ok());
}

#[test]
fn infer_adt_construction() {
    assert!(typecheck(
        "type Shape = Circle Int | Rect Int Int\nlet s = Circle(5)"
    ).is_ok());
}

#[test]
fn infer_adt_match() {
    assert!(typecheck(
        "type Bool2 = T | F\nmatch T with | T -> 1 | F -> 0"
    ).is_ok());
}

// ── Pipe operator ──

#[test]
fn infer_pipe() {
    assert!(typecheck("[1, 2, 3] |> length").is_ok());
}

#[test]
fn infer_pipe_chain() {
    assert!(typecheck(
        "[1, 2, 3] |> map(fn (x) -> x * 2) |> length"
    ).is_ok());
}

// ── Cons operator ──

#[test]
fn infer_cons() {
    assert!(typecheck("1 :: [2, 3]").is_ok());
}

#[test]
fn infer_cons_type_mismatch_fails() {
    assert!(typecheck_fails("\"hello\" :: [1, 2]"));
}

// ── Records ──

#[test]
fn infer_record_literal() {
    assert!(typecheck("{ name: \"Alice\", age: 30 }").is_ok());
}

#[test]
fn infer_record_field_access() {
    assert!(typecheck(
        "let r = { x: 1, y: 2 }\nr.x + r.y"
    ).is_ok());
}

// ── String interpolation ──

#[test]
fn infer_interpolation() {
    assert!(typecheck(
        "let name = \"world\"\n\"hello {name}\""
    ).is_ok());
}

// ── Stdlib functions ──

#[test]
fn infer_stdlib_map() {
    assert!(typecheck(
        "map(fn (x) -> x * 2, [1, 2, 3])"
    ).is_ok());
}

#[test]
fn infer_stdlib_filter() {
    assert!(typecheck(
        "filter(fn (x) -> x > 0, [1, -2, 3])"
    ).is_ok());
}

#[test]
fn infer_stdlib_fold() {
    assert!(typecheck(
        "fold(0, fn (acc, x) -> acc + x, [1, 2, 3])"
    ).is_ok());
}

#[test]
fn infer_stdlib_take_drop() {
    assert!(typecheck("take(2, [1, 2, 3])").is_ok());
    assert!(typecheck("drop(1, [1, 2, 3])").is_ok());
}

#[test]
fn infer_stdlib_any_all() {
    assert!(typecheck("any(fn (x) -> x > 0, [1, -2, 3])").is_ok());
    assert!(typecheck("all(fn (x) -> x > 0, [1, 2, 3])").is_ok());
}

#[test]
fn infer_stdlib_flatten() {
    assert!(typecheck("flatten([[1, 2], [3, 4]])").is_ok());
}

#[test]
fn infer_stdlib_sum_product() {
    assert!(typecheck("sum([1, 2, 3])").is_ok());
    assert!(typecheck("product([1, 2, 3])").is_ok());
}

#[test]
fn infer_stdlib_string_conversions() {
    assert!(typecheck("string_to_int(\"42\")").is_ok());
    assert!(typecheck("int_to_string(42)").is_ok());
}

#[test]
fn infer_stdlib_str_utilities() {
    assert!(typecheck("str_trim(\" hello \")").is_ok());
    assert!(typecheck("str_uppercase(\"hello\")").is_ok());
    assert!(typecheck("str_lowercase(\"HELLO\")").is_ok());
    assert!(typecheck("str_replace(\"hello\", \"l\", \"r\")").is_ok());
    assert!(typecheck("str_starts_with(\"hello\", \"he\")").is_ok());
    assert!(typecheck("str_ends_with(\"hello\", \"lo\")").is_ok());
    assert!(typecheck("str_substring(\"hello\", 1, 3)").is_ok());
}

// ── Undefined variable ──

#[test]
fn undefined_variable_fails() {
    assert!(typecheck_fails("undefined_var"));
}
