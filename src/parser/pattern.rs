use crate::ast::*;
use crate::error::LyraError;
use crate::lexer::token::TokenKind;
use crate::span::Spanned;

use super::Parser;

impl Parser {
    pub fn parse_pattern(&mut self) -> Result<SpannedPattern, LyraError> {
        let mut pat = self.parse_pattern_atom()?;

        // Cons pattern: hd :: tl
        if matches!(self.peek(), TokenKind::ColonColon) {
            self.advance();
            let tail = self.parse_pattern()?;
            let span = pat.span.merge(tail.span);
            pat = Spanned::new(
                Pattern::Cons(Box::new(pat), Box::new(tail)),
                span,
            );
        }

        Ok(pat)
    }

    fn parse_pattern_atom(&mut self) -> Result<SpannedPattern, LyraError> {
        let tok = self.peek_token().clone();
        match &tok.kind {
            // Wildcard
            TokenKind::Underscore => {
                self.advance();
                Ok(Spanned::new(Pattern::Wildcard, tok.span))
            }

            // Integer literal pattern
            TokenKind::IntLit(n) => {
                let n = *n;
                self.advance();
                Ok(Spanned::new(Pattern::IntLit(n), tok.span))
            }

            // Float literal pattern
            TokenKind::FloatLit(n) => {
                let n = *n;
                self.advance();
                Ok(Spanned::new(Pattern::FloatLit(n), tok.span))
            }

            // String literal pattern
            TokenKind::StringLit(s) => {
                let s = s.clone();
                self.advance();
                Ok(Spanned::new(Pattern::StringLit(s), tok.span))
            }

            // Bool literal pattern
            TokenKind::BoolLit(b) => {
                let b = *b;
                self.advance();
                Ok(Spanned::new(Pattern::BoolLit(b), tok.span))
            }

            // Identifier: either a constructor (uppercase) or variable binding (lowercase)
            TokenKind::Ident(name) => {
                let name = name.clone();
                if name.starts_with(|c: char| c.is_uppercase()) {
                    // Constructor pattern
                    self.advance();
                    let mut args = Vec::new();
                    if matches!(self.peek(), TokenKind::LParen) {
                        self.advance(); // consume '('
                        if !matches!(self.peek(), TokenKind::RParen) {
                            args.push(self.parse_pattern()?);
                            while self.match_token(&TokenKind::Comma) {
                                args.push(self.parse_pattern()?);
                            }
                        }
                        self.expect(&TokenKind::RParen)?;
                    }
                    let end = self.previous_span();
                    let span = tok.span.merge(end);
                    Ok(Spanned::new(
                        Pattern::Constructor { name, args },
                        span,
                    ))
                } else {
                    // Variable binding pattern
                    self.advance();
                    Ok(Spanned::new(Pattern::Var(name), tok.span))
                }
            }

            // Parenthesized pattern or unit or tuple pattern
            TokenKind::LParen => {
                let start = tok.span;
                self.advance();

                if matches!(self.peek(), TokenKind::RParen) {
                    self.advance();
                    let span = start.merge(self.previous_span());
                    return Ok(Spanned::new(Pattern::UnitLit, span));
                }

                let first = self.parse_pattern()?;

                if matches!(self.peek(), TokenKind::Comma) {
                    let mut pats = vec![first];
                    while self.match_token(&TokenKind::Comma) {
                        pats.push(self.parse_pattern()?);
                    }
                    self.expect(&TokenKind::RParen)?;
                    let span = start.merge(self.previous_span());
                    return Ok(Spanned::new(Pattern::Tuple(pats), span));
                }

                self.expect(&TokenKind::RParen)?;
                Ok(first)
            }

            // List pattern
            TokenKind::LBracket => {
                let start = tok.span;
                self.advance();

                let mut pats = Vec::new();
                if !matches!(self.peek(), TokenKind::RBracket) {
                    pats.push(self.parse_pattern()?);
                    while self.match_token(&TokenKind::Comma) {
                        pats.push(self.parse_pattern()?);
                    }
                }

                self.expect(&TokenKind::RBracket)?;
                let span = start.merge(self.previous_span());
                Ok(Spanned::new(Pattern::List(pats), span))
            }

            // Negative integer literal
            TokenKind::Minus => {
                let start = tok.span;
                self.advance();
                let next = self.peek_token().clone();
                match next.kind {
                    TokenKind::IntLit(n) => {
                        self.advance();
                        let span = start.merge(next.span);
                        Ok(Spanned::new(Pattern::IntLit(-n), span))
                    }
                    TokenKind::FloatLit(n) => {
                        self.advance();
                        let span = start.merge(next.span);
                        Ok(Spanned::new(Pattern::FloatLit(-n), span))
                    }
                    _ => Err(LyraError::ExpectedExpression {
                        found: next.kind.describe().to_string(),
                        span: next.span,
                    }),
                }
            }

            _ => Err(LyraError::ExpectedExpression {
                found: tok.kind.describe().to_string(),
                span: tok.span,
            }),
        }
    }
}
