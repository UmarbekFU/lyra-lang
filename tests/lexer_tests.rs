use lyra::lexer::tokenize;
use lyra::lexer::token::TokenKind;

fn token_kinds(source: &str) -> Vec<TokenKind> {
    let tokens = tokenize(source).expect("lexer should succeed");
    tokens.into_iter().map(|t| t.kind).collect()
}

#[test]
fn lex_integer_literals() {
    let kinds = token_kinds("0 42 1000");
    assert_eq!(kinds, vec![
        TokenKind::IntLit(0),
        TokenKind::IntLit(42),
        TokenKind::IntLit(1000),
        TokenKind::Eof,
    ]);
}

#[test]
fn lex_float_literals() {
    let kinds = token_kinds("3.14 0.5");
    assert_eq!(kinds, vec![
        TokenKind::FloatLit(3.14),
        TokenKind::FloatLit(0.5),
        TokenKind::Eof,
    ]);
}

#[test]
fn lex_string_literal() {
    let kinds = token_kinds("\"hello world\"");
    assert_eq!(kinds, vec![
        TokenKind::StringLit("hello world".to_string()),
        TokenKind::Eof,
    ]);
}

#[test]
fn lex_boolean_literals() {
    let kinds = token_kinds("true false");
    assert_eq!(kinds, vec![
        TokenKind::BoolLit(true),
        TokenKind::BoolLit(false),
        TokenKind::Eof,
    ]);
}

#[test]
fn lex_keywords() {
    let kinds = token_kinds("let in fn match with if then else type rec import");
    assert_eq!(kinds, vec![
        TokenKind::Let,
        TokenKind::In,
        TokenKind::Fn,
        TokenKind::Match,
        TokenKind::With,
        TokenKind::If,
        TokenKind::Then,
        TokenKind::Else,
        TokenKind::Type,
        TokenKind::Rec,
        TokenKind::Import,
        TokenKind::Eof,
    ]);
}

#[test]
fn lex_identifiers() {
    let kinds = token_kinds("foo bar Baz");
    assert_eq!(kinds, vec![
        TokenKind::Ident("foo".to_string()),
        TokenKind::Ident("bar".to_string()),
        TokenKind::Ident("Baz".to_string()),
        TokenKind::Eof,
    ]);
}

#[test]
fn lex_operators() {
    let kinds = token_kinds("+ - * / % == != < > <= >= && || ! :: |>");
    assert_eq!(kinds, vec![
        TokenKind::Plus,
        TokenKind::Minus,
        TokenKind::Star,
        TokenKind::Slash,
        TokenKind::Percent,
        TokenKind::EqEq,
        TokenKind::NotEq,
        TokenKind::Lt,
        TokenKind::Gt,
        TokenKind::Le,
        TokenKind::Ge,
        TokenKind::And,
        TokenKind::Or,
        TokenKind::Not,
        TokenKind::ColonColon,
        TokenKind::PipeRight,
        TokenKind::Eof,
    ]);
}

#[test]
fn lex_delimiters() {
    let kinds = token_kinds("( ) [ ] { }");
    assert_eq!(kinds, vec![
        TokenKind::LParen,
        TokenKind::RParen,
        TokenKind::LBracket,
        TokenKind::RBracket,
        TokenKind::LBrace,
        TokenKind::RBrace,
        TokenKind::Eof,
    ]);
}

#[test]
fn lex_arrow_and_pipe() {
    let kinds = token_kinds("-> |> | =");
    assert_eq!(kinds, vec![
        TokenKind::Arrow,
        TokenKind::PipeRight,
        TokenKind::Pipe,
        TokenKind::Eq,
        TokenKind::Eof,
    ]);
}

#[test]
fn lex_comments_are_skipped() {
    let kinds = token_kinds("42 -- this is a comment\n 7");
    assert_eq!(kinds, vec![
        TokenKind::IntLit(42),
        TokenKind::IntLit(7),
        TokenKind::Eof,
    ]);
}

#[test]
fn lex_interpolated_string() {
    let kinds = token_kinds("\"hello {name}!\"");
    assert!(matches!(kinds[0], TokenKind::InterpolatedString(_)));
}

#[test]
fn lex_record_braces() {
    let kinds = token_kinds("{ x: 1 }");
    assert_eq!(kinds[0], TokenKind::LBrace);
    assert_eq!(kinds[4], TokenKind::RBrace);
}

#[test]
fn lex_dot_operator() {
    let kinds = token_kinds("x.y");
    assert_eq!(kinds, vec![
        TokenKind::Ident("x".to_string()),
        TokenKind::Dot,
        TokenKind::Ident("y".to_string()),
        TokenKind::Eof,
    ]);
}

#[test]
fn lex_underscore_wildcard() {
    let kinds = token_kinds("_");
    assert_eq!(kinds, vec![
        TokenKind::Underscore,
        TokenKind::Eof,
    ]);
}

#[test]
fn lex_import_keyword() {
    let kinds = token_kinds("import \"foo\"");
    assert_eq!(kinds, vec![
        TokenKind::Import,
        TokenKind::StringLit("foo".to_string()),
        TokenKind::Eof,
    ]);
}
