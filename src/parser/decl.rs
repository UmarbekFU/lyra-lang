use crate::ast::*;
use crate::error::LyraError;
use crate::lexer::token::TokenKind;
use crate::span::Spanned;

use super::Parser;

impl Parser {
    pub(crate) fn parse_decl(&mut self) -> Result<Decl, LyraError> {
        match self.peek() {
            TokenKind::Let => self.parse_let_decl(),
            TokenKind::Type => self.parse_type_decl(),
            TokenKind::Import => self.parse_import_decl(),
            _ => {
                let expr = self.parse_expr()?;
                Ok(Decl::Expr(expr))
            }
        }
    }

    fn parse_let_decl(&mut self) -> Result<Decl, LyraError> {
        self.advance(); // consume 'let'
        let recursive = self.match_token(&TokenKind::Rec);
        let name = self.expect_ident()?;

        let type_ann = if self.match_token(&TokenKind::Colon) {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        self.expect(&TokenKind::Eq)?;
        let body = self.parse_expr()?;

        Ok(Decl::Let {
            name,
            recursive,
            type_ann,
            body,
        })
    }

    fn parse_type_decl(&mut self) -> Result<Decl, LyraError> {
        self.advance(); // consume 'type'
        let name = self.expect_ident()?;

        // Type parameters (lowercase identifiers)
        let mut type_params = Vec::new();
        while let TokenKind::Ident(p) = self.peek() {
            if p.starts_with(|c: char| c.is_lowercase()) {
                let param = self.expect_ident()?;
                type_params.push(param);
            } else {
                break;
            }
        }

        self.expect(&TokenKind::Eq)?;

        // Parse variants: Variant1 Field1 Field2 | Variant2 | ...
        let mut variants = Vec::new();
        self.match_token(&TokenKind::Pipe); // optional leading |
        variants.push(self.parse_variant()?);
        while self.match_token(&TokenKind::Pipe) {
            variants.push(self.parse_variant()?);
        }

        Ok(Decl::Type {
            name,
            type_params,
            variants,
        })
    }

    fn parse_variant(&mut self) -> Result<Variant, LyraError> {
        let name = self.expect_ident()?;
        let start = name.span;
        let mut fields = Vec::new();

        // Fields are type atoms until we hit | or EOF or a new decl keyword
        loop {
            match self.peek() {
                TokenKind::Pipe
                | TokenKind::Eof
                | TokenKind::Let
                | TokenKind::Type => break,
                TokenKind::Ident(s) if s.starts_with(|c: char| c.is_uppercase()) => {
                    // Could be a field type OR the next variant if preceded by |
                    // Since we break on |, an uppercase ident here is a type field
                    fields.push(self.parse_type_atom_for_variant()?);
                }
                TokenKind::Ident(_) | TokenKind::LParen | TokenKind::LBracket => {
                    fields.push(self.parse_type_atom_for_variant()?);
                }
                _ => break,
            }
        }

        let end = if fields.is_empty() {
            name.span
        } else {
            fields.last().unwrap().span
        };

        Ok(Variant {
            name,
            fields,
            span: start.merge(end),
        })
    }

    fn parse_import_decl(&mut self) -> Result<Decl, LyraError> {
        let start = self.peek_span();
        self.advance(); // consume 'import'
        let tok = self.advance().clone();
        match tok.kind {
            TokenKind::StringLit(path) => {
                let span = start.merge(tok.span);
                Ok(Decl::Import { path, span })
            }
            _ => Err(LyraError::UnexpectedToken {
                expected: "string path".to_string(),
                found: tok.kind.describe().to_string(),
                span: tok.span,
            }),
        }
    }

    /// Parse a single type atom for a variant field.
    fn parse_type_atom_for_variant(&mut self) -> Result<SpannedTypeAnn, LyraError> {
        let tok = self.peek_token().clone();
        match &tok.kind {
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.advance();
                if name.starts_with(|c: char| c.is_uppercase()) {
                    Ok(Spanned::new(TypeAnnotation::Named(name), tok.span))
                } else {
                    Ok(Spanned::new(TypeAnnotation::Var(name), tok.span))
                }
            }
            TokenKind::LParen => {
                let start = tok.span;
                self.advance();
                if matches!(self.peek(), TokenKind::RParen) {
                    self.advance();
                    let span = start.merge(self.previous_span());
                    return Ok(Spanned::new(TypeAnnotation::Unit, span));
                }
                let inner = self.parse_type_annotation()?;
                self.expect(&TokenKind::RParen)?;
                let span = start.merge(self.previous_span());
                Ok(Spanned::new(inner.node, span))
            }
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
