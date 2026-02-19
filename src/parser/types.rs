use crate::ast::*;
use crate::error::LyraError;
use crate::lexer::token::TokenKind;
use crate::span::Spanned;

use super::Parser;

impl Parser {
    /// Parse a type annotation (e.g., after `:` in `let x : Int = ...`)
    pub fn parse_type_annotation(&mut self) -> Result<SpannedTypeAnn, LyraError> {
        let lhs = self.parse_type_atom()?;

        // Arrow type: A -> B
        if matches!(self.peek(), TokenKind::Arrow) {
            self.advance();
            let rhs = self.parse_type_annotation()?; // right-associative
            let span = lhs.span.merge(rhs.span);
            return Ok(Spanned::new(
                TypeAnnotation::Arrow(Box::new(lhs), Box::new(rhs)),
                span,
            ));
        }

        Ok(lhs)
    }

    fn parse_type_atom(&mut self) -> Result<SpannedTypeAnn, LyraError> {
        let tok = self.peek_token().clone();
        match &tok.kind {
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.advance();

                // Uppercase = named type, lowercase = type variable
                let base = if name.starts_with(|c: char| c.is_uppercase()) {
                    Spanned::new(TypeAnnotation::Named(name), tok.span)
                } else {
                    Spanned::new(TypeAnnotation::Var(name), tok.span)
                };

                // Type application: Option Int, List a
                // Collect arguments while we see type atoms that start valid type expressions
                let mut args = Vec::new();
                loop {
                    match self.peek() {
                        TokenKind::Ident(_) | TokenKind::LParen | TokenKind::LBracket => {
                            // Only consume if the base is a Named type (not a var)
                            if matches!(base.node, TypeAnnotation::Named(_)) {
                                args.push(self.parse_type_atom()?);
                            } else {
                                break;
                            }
                        }
                        _ => break,
                    }
                }

                if args.is_empty() {
                    Ok(base)
                } else {
                    let end = args.last().unwrap().span;
                    let span = tok.span.merge(end);
                    Ok(Spanned::new(TypeAnnotation::App(Box::new(base), args), span))
                }
            }

            // Parenthesized type or unit or tuple
            TokenKind::LParen => {
                let start = tok.span;
                self.advance();

                if matches!(self.peek(), TokenKind::RParen) {
                    self.advance();
                    let span = start.merge(self.previous_span());
                    return Ok(Spanned::new(TypeAnnotation::Unit, span));
                }

                let first = self.parse_type_annotation()?;

                if matches!(self.peek(), TokenKind::Comma) {
                    let mut elems = vec![first];
                    while self.match_token(&TokenKind::Comma) {
                        elems.push(self.parse_type_annotation()?);
                    }
                    self.expect(&TokenKind::RParen)?;
                    let span = start.merge(self.previous_span());
                    return Ok(Spanned::new(TypeAnnotation::Tuple(elems), span));
                }

                self.expect(&TokenKind::RParen)?;
                Ok(first)
            }

            // List type: [Int]
            TokenKind::LBracket => {
                let start = tok.span;
                self.advance();
                let inner = self.parse_type_annotation()?;
                self.expect(&TokenKind::RBracket)?;
                let span = start.merge(self.previous_span());
                Ok(Spanned::new(TypeAnnotation::List(Box::new(inner)), span))
            }

            _ => Err(LyraError::UnexpectedToken {
                expected: "type".to_string(),
                found: tok.kind.describe().to_string(),
                span: tok.span,
            }),
        }
    }
}
