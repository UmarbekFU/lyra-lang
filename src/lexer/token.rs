use crate::span::Span;

/// A part of an interpolated string at the token level.
#[derive(Debug, Clone, PartialEq)]
pub enum InterpPart {
    Literal(String),
    Tokens(Vec<Token>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    IntLit(i64),
    FloatLit(f64),
    StringLit(String),
    BoolLit(bool),
    InterpolatedString(Vec<InterpPart>),

    // Identifier
    Ident(String),

    // Keywords
    Let,
    In,
    Fn,
    Match,
    With,
    If,
    Then,
    Else,
    Type,
    Rec,
    Import,

    // Symbols
    Eq,         // =
    Arrow,      // ->
    Pipe,       // |
    PipeRight,  // |>
    Plus,       // +
    Minus,      // -
    Star,       // *
    Slash,      // /
    Percent,    // %
    Lt,         // <
    Gt,         // >
    Le,         // <=
    Ge,         // >=
    EqEq,       // ==
    NotEq,      // !=
    And,        // &&
    Or,         // ||
    Not,        // !
    Colon,      // :
    ColonColon, // ::
    Comma,      // ,
    Dot,        // .
    Underscore, // _

    // Delimiters
    LParen,   // (
    RParen,   // )
    LBracket, // [
    RBracket, // ]
    LBrace,   // {
    RBrace,   // }

    // Special
    Eof,
}

impl TokenKind {
    pub fn describe(&self) -> &str {
        match self {
            TokenKind::IntLit(_) => "integer",
            TokenKind::FloatLit(_) => "float",
            TokenKind::StringLit(_) => "string",
            TokenKind::InterpolatedString(_) => "interpolated string",
            TokenKind::BoolLit(_) => "boolean",
            TokenKind::Ident(_) => "identifier",
            TokenKind::Let => "'let'",
            TokenKind::In => "'in'",
            TokenKind::Fn => "'fn'",
            TokenKind::Match => "'match'",
            TokenKind::With => "'with'",
            TokenKind::If => "'if'",
            TokenKind::Then => "'then'",
            TokenKind::Else => "'else'",
            TokenKind::Type => "'type'",
            TokenKind::Rec => "'rec'",
            TokenKind::Import => "'import'",
            TokenKind::Eq => "'='",
            TokenKind::Arrow => "'->'",
            TokenKind::Pipe => "'|'",
            TokenKind::PipeRight => "'|>'",
            TokenKind::Plus => "'+'",
            TokenKind::Minus => "'-'",
            TokenKind::Star => "'*'",
            TokenKind::Slash => "'/'",
            TokenKind::Percent => "'%'",
            TokenKind::Lt => "'<'",
            TokenKind::Gt => "'>'",
            TokenKind::Le => "'<='",
            TokenKind::Ge => "'>='",
            TokenKind::EqEq => "'=='",
            TokenKind::NotEq => "'!='",
            TokenKind::And => "'&&'",
            TokenKind::Or => "'||'",
            TokenKind::Not => "'!'",
            TokenKind::Colon => "':'",
            TokenKind::ColonColon => "'::'",
            TokenKind::Comma => "','",
            TokenKind::Dot => "'.'",
            TokenKind::Underscore => "'_'",
            TokenKind::LParen => "'('",
            TokenKind::RParen => "')'",
            TokenKind::LBracket => "'['",
            TokenKind::RBracket => "']'",
            TokenKind::LBrace => "'{'",
            TokenKind::RBrace => "'}'",
            TokenKind::Eof => "end of input",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Token { kind, span }
    }
}
