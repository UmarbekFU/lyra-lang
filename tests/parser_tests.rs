use lyra::ast::*;
use lyra::lexer::tokenize;
use lyra::parser::parse;

fn parse_source(source: &str) -> Vec<Decl> {
    let tokens = tokenize(source).expect("lexer should succeed");
    parse(tokens).expect("parser should succeed")
}

#[test]
fn parse_integer_expression() {
    let decls = parse_source("42");
    assert_eq!(decls.len(), 1);
    match &decls[0] {
        Decl::Expr(expr) => assert!(matches!(expr.node, Expr::IntLit(42))),
        _ => panic!("expected Expr decl"),
    }
}

#[test]
fn parse_let_binding() {
    let decls = parse_source("let x = 42");
    assert_eq!(decls.len(), 1);
    match &decls[0] {
        Decl::Let { name, recursive, .. } => {
            assert_eq!(name.node, "x");
            assert!(!recursive);
        }
        _ => panic!("expected Let decl"),
    }
}

#[test]
fn parse_recursive_let() {
    let decls = parse_source("let rec f = fn (x) -> x");
    match &decls[0] {
        Decl::Let { name, recursive, .. } => {
            assert_eq!(name.node, "f");
            assert!(recursive);
        }
        _ => panic!("expected Let decl"),
    }
}

#[test]
fn parse_lambda() {
    let decls = parse_source("fn (x, y) -> x + y");
    match &decls[0] {
        Decl::Expr(expr) => match &expr.node {
            Expr::Lambda { params, .. } => {
                assert_eq!(params.len(), 2);
                assert_eq!(params[0].name.node, "x");
                assert_eq!(params[1].name.node, "y");
            }
            _ => panic!("expected Lambda"),
        },
        _ => panic!("expected Expr decl"),
    }
}

#[test]
fn parse_function_application() {
    let decls = parse_source("f(1, 2)");
    match &decls[0] {
        Decl::Expr(expr) => match &expr.node {
            Expr::App { func, args } => {
                assert!(matches!(func.node, Expr::Var(ref name) if name == "f"));
                assert_eq!(args.len(), 2);
            }
            _ => panic!("expected App, got {:?}", expr.node),
        },
        _ => panic!("expected Expr decl"),
    }
}

#[test]
fn parse_if_expression() {
    let decls = parse_source("if true then 1 else 2");
    match &decls[0] {
        Decl::Expr(expr) => {
            assert!(matches!(expr.node, Expr::If { .. }));
        }
        _ => panic!("expected Expr decl"),
    }
}

#[test]
fn parse_match_expression() {
    let decls = parse_source("match x with | 0 -> true | _ -> false");
    match &decls[0] {
        Decl::Expr(expr) => match &expr.node {
            Expr::Match { arms, .. } => {
                assert_eq!(arms.len(), 2);
            }
            _ => panic!("expected Match"),
        },
        _ => panic!("expected Expr decl"),
    }
}

#[test]
fn parse_list_literal() {
    let decls = parse_source("[1, 2, 3]");
    match &decls[0] {
        Decl::Expr(expr) => match &expr.node {
            Expr::ListLit(elems) => assert_eq!(elems.len(), 3),
            _ => panic!("expected ListLit"),
        },
        _ => panic!("expected Expr decl"),
    }
}

#[test]
fn parse_tuple_literal() {
    let decls = parse_source("(1, 2, 3)");
    match &decls[0] {
        Decl::Expr(expr) => match &expr.node {
            Expr::TupleLit(elems) => assert_eq!(elems.len(), 3),
            _ => panic!("expected TupleLit"),
        },
        _ => panic!("expected Expr decl"),
    }
}

#[test]
fn parse_pipe_operator() {
    let decls = parse_source("[1,2,3] |> length");
    match &decls[0] {
        Decl::Expr(expr) => {
            assert!(matches!(expr.node, Expr::Pipe { .. }));
        }
        _ => panic!("expected Expr decl"),
    }
}

#[test]
fn parse_binary_operators() {
    let decls = parse_source("1 + 2 * 3");
    match &decls[0] {
        Decl::Expr(expr) => match &expr.node {
            Expr::BinOp { op, .. } => {
                assert_eq!(*op, BinOp::Add); // + is lower precedence, so top-level
            }
            _ => panic!("expected BinOp"),
        },
        _ => panic!("expected Expr decl"),
    }
}

#[test]
fn parse_cons_operator() {
    let decls = parse_source("1 :: [2, 3]");
    match &decls[0] {
        Decl::Expr(expr) => match &expr.node {
            Expr::BinOp { op, .. } => {
                assert_eq!(*op, BinOp::Cons);
            }
            _ => panic!("expected BinOp::Cons"),
        },
        _ => panic!("expected Expr decl"),
    }
}

#[test]
fn parse_let_in_expression() {
    // let...in is only valid inside an expression context (e.g. inside a function body)
    let decls = parse_source("fn (n) -> let x = 1 in x + n");
    match &decls[0] {
        Decl::Expr(expr) => match &expr.node {
            Expr::Lambda { body, .. } => {
                assert!(matches!(body.node, Expr::Let { .. }));
            }
            _ => panic!("expected Lambda wrapping Let expression"),
        },
        _ => panic!("expected Expr decl"),
    }
}

#[test]
fn parse_type_declaration() {
    let decls = parse_source("type Color = Red | Green | Blue");
    match &decls[0] {
        Decl::Type { name, variants, type_params } => {
            assert_eq!(name.node, "Color");
            assert_eq!(type_params.len(), 0);
            assert_eq!(variants.len(), 3);
            assert_eq!(variants[0].name.node, "Red");
            assert_eq!(variants[1].name.node, "Green");
            assert_eq!(variants[2].name.node, "Blue");
        }
        _ => panic!("expected Type decl"),
    }
}

#[test]
fn parse_type_with_fields() {
    let decls = parse_source("type Shape = Circle Int | Rectangle Int Int");
    match &decls[0] {
        Decl::Type { variants, .. } => {
            assert_eq!(variants[0].fields.len(), 1);
            assert_eq!(variants[1].fields.len(), 2);
        }
        _ => panic!("expected Type decl"),
    }
}

#[test]
fn parse_record_literal() {
    let decls = parse_source("{ name: \"Alice\", age: 30 }");
    match &decls[0] {
        Decl::Expr(expr) => match &expr.node {
            Expr::Record(fields) => {
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].0, "name");
                assert_eq!(fields[1].0, "age");
            }
            _ => panic!("expected Record"),
        },
        _ => panic!("expected Expr decl"),
    }
}

#[test]
fn parse_field_access() {
    let decls = parse_source("person.name");
    match &decls[0] {
        Decl::Expr(expr) => match &expr.node {
            Expr::FieldAccess { field, .. } => {
                assert_eq!(field, "name");
            }
            _ => panic!("expected FieldAccess"),
        },
        _ => panic!("expected Expr decl"),
    }
}

#[test]
fn parse_import_declaration() {
    let decls = parse_source("import \"utils\"");
    match &decls[0] {
        Decl::Import { path, .. } => {
            assert_eq!(path, "utils");
        }
        _ => panic!("expected Import decl"),
    }
}

#[test]
fn parse_interpolated_string() {
    let decls = parse_source("\"hello {name}\"");
    match &decls[0] {
        Decl::Expr(expr) => {
            assert!(matches!(expr.node, Expr::Interpolation(_)));
        }
        _ => panic!("expected Expr decl"),
    }
}

#[test]
fn parse_unary_negation() {
    let decls = parse_source("-42");
    match &decls[0] {
        Decl::Expr(expr) => match &expr.node {
            Expr::UnaryOp { op, .. } => {
                assert_eq!(*op, UnaryOp::Neg);
            }
            _ => panic!("expected UnaryOp"),
        },
        _ => panic!("expected Expr decl"),
    }
}

#[test]
fn parse_unary_not() {
    let decls = parse_source("!true");
    match &decls[0] {
        Decl::Expr(expr) => match &expr.node {
            Expr::UnaryOp { op, .. } => {
                assert_eq!(*op, UnaryOp::Not);
            }
            _ => panic!("expected UnaryOp"),
        },
        _ => panic!("expected Expr decl"),
    }
}

#[test]
fn parse_multiple_declarations() {
    let decls = parse_source("let x = 1\nlet y = 2\nx + y");
    assert_eq!(decls.len(), 3);
    assert!(matches!(decls[0], Decl::Let { .. }));
    assert!(matches!(decls[1], Decl::Let { .. }));
    assert!(matches!(decls[2], Decl::Expr(_)));
}

#[test]
fn parse_nested_field_access() {
    let decls = parse_source("a.b.c");
    match &decls[0] {
        Decl::Expr(expr) => match &expr.node {
            Expr::FieldAccess { expr: inner, field } => {
                assert_eq!(field, "c");
                assert!(matches!(inner.node, Expr::FieldAccess { .. }));
            }
            _ => panic!("expected FieldAccess"),
        },
        _ => panic!("expected Expr decl"),
    }
}
