pub mod decl;
pub mod expr;
pub mod pattern;
pub mod types;

use crate::ast::*;
use crate::error::LyraError;
use crate::lexer::token::{Token, TokenKind};
use crate::span::{Span, Spanned};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn parse_program(&mut self) -> Result<Vec<Decl>, LyraError> {
        let mut decls = Vec::new();
        while !self.is_at_end() {
            decls.push(self.parse_decl()?);
        }
        Ok(decls)
    }

    // ── Token navigation ──

    pub(crate) fn peek(&self) -> &TokenKind {
        &self.tokens[self.pos].kind
    }

    pub(crate) fn peek_token(&self) -> &Token {
        &self.tokens[self.pos]
    }

    pub(crate) fn peek_span(&self) -> Span {
        self.tokens[self.pos].span
    }

    pub(crate) fn previous_span(&self) -> Span {
        if self.pos > 0 {
            self.tokens[self.pos - 1].span
        } else {
            Span::default()
        }
    }

    pub(crate) fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos];
        if !self.is_at_end() {
            self.pos += 1;
        }
        tok
    }

    pub(crate) fn is_at_end(&self) -> bool {
        matches!(self.peek(), TokenKind::Eof)
    }

    pub(crate) fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(self.peek()) == std::mem::discriminant(kind)
    }

    pub(crate) fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    pub(crate) fn expect(&mut self, kind: &TokenKind) -> Result<&Token, LyraError> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(LyraError::UnexpectedToken {
                expected: kind.describe().to_string(),
                found: self.peek().describe().to_string(),
                span: self.peek_span(),
            })
        }
    }

    pub(crate) fn expect_ident(&mut self) -> Result<Spanned<String>, LyraError> {
        let tok = self.advance().clone();
        match tok.kind {
            TokenKind::Ident(name) => Ok(Spanned::new(name, tok.span)),
            _ => Err(LyraError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: tok.kind.describe().to_string(),
                span: tok.span,
            }),
        }
    }

    /// Check if next token is an uppercase identifier (constructor).
    #[allow(dead_code)]
    pub(crate) fn peek_is_constructor(&self) -> bool {
        match self.peek() {
            TokenKind::Ident(name) => name.starts_with(|c: char| c.is_uppercase()),
            _ => false,
        }
    }
}

/// Binding power for infix operators (left_bp, right_bp).
pub(crate) fn infix_binding_power(kind: &TokenKind) -> Option<(u8, u8)> {
    match kind {
        TokenKind::PipeRight => Some((1, 2)),
        TokenKind::Or => Some((3, 4)),
        TokenKind::And => Some((5, 6)),
        TokenKind::EqEq | TokenKind::NotEq => Some((7, 8)),
        TokenKind::Lt | TokenKind::Gt | TokenKind::Le | TokenKind::Ge => Some((9, 10)),
        TokenKind::ColonColon => Some((12, 11)), // right-associative
        TokenKind::Plus | TokenKind::Minus => Some((13, 14)),
        TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Some((15, 16)),
        _ => None,
    }
}

pub(crate) fn token_to_binop(kind: &TokenKind) -> BinOp {
    match kind {
        TokenKind::Plus => BinOp::Add,
        TokenKind::Minus => BinOp::Sub,
        TokenKind::Star => BinOp::Mul,
        TokenKind::Slash => BinOp::Div,
        TokenKind::Percent => BinOp::Mod,
        TokenKind::EqEq => BinOp::Eq,
        TokenKind::NotEq => BinOp::NotEq,
        TokenKind::Lt => BinOp::Lt,
        TokenKind::Gt => BinOp::Gt,
        TokenKind::Le => BinOp::Le,
        TokenKind::Ge => BinOp::Ge,
        TokenKind::And => BinOp::And,
        TokenKind::Or => BinOp::Or,
        TokenKind::ColonColon => BinOp::Cons,
        _ => unreachable!("not a binary operator: {:?}", kind),
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Decl>, LyraError> {
    Parser::new(tokens).parse_program()
}
