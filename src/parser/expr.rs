use crate::ast::*;
use crate::error::LyraError;
use crate::lexer::token::{InterpPart, TokenKind};
use crate::span::{Span, Spanned};

use super::{infix_binding_power, token_to_binop, Parser};

impl Parser {
    /// Parse an expression with Pratt binding power.
    pub fn parse_expr(&mut self) -> Result<SpannedExpr, LyraError> {
        self.parse_expr_bp(0)
    }

    pub(crate) fn parse_expr_bp(&mut self, min_bp: u8) -> Result<SpannedExpr, LyraError> {
        let mut lhs = self.parse_prefix()?;

        loop {
            // Check for field access: expr.field (highest precedence postfix)
            if matches!(self.peek(), TokenKind::Dot) && min_bp <= 19 {
                self.advance(); // consume '.'
                let field_tok = self.advance().clone();
                let field = match field_tok.kind {
                    TokenKind::Ident(name) => name,
                    _ => {
                        return Err(LyraError::UnexpectedToken {
                            expected: "field name".to_string(),
                            found: field_tok.kind.describe().to_string(),
                            span: field_tok.span,
                        })
                    }
                };
                let span = lhs.span.merge(field_tok.span);
                lhs = Spanned::new(
                    Expr::FieldAccess {
                        expr: Box::new(lhs),
                        field,
                    },
                    span,
                );
                continue;
            }

            // Check for function application: expr(args)
            if matches!(self.peek(), TokenKind::LParen)
                && !matches!(
                    lhs.node,
                    Expr::IntLit(_)
                        | Expr::FloatLit(_)
                        | Expr::BoolLit(_)
                        | Expr::StringLit(_)
                        | Expr::Interpolation(_)
                        | Expr::UnitLit
                )
            {
                // Application binds tighter than any infix op
                if min_bp <= 17 {
                    lhs = self.parse_call(lhs)?;
                    continue;
                }
            }

            let op_kind = self.peek().clone();
            if let Some((l_bp, r_bp)) = infix_binding_power(&op_kind) {
                if l_bp < min_bp {
                    break;
                }
                self.advance(); // consume operator
                let rhs = self.parse_expr_bp(r_bp)?;
                let span = lhs.span.merge(rhs.span);

                lhs = match op_kind {
                    TokenKind::PipeRight => Spanned::new(
                        Expr::Pipe {
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        },
                        span,
                    ),
                    _ => Spanned::new(
                        Expr::BinOp {
                            op: token_to_binop(&op_kind),
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        },
                        span,
                    ),
                };
            } else {
                break;
            }
        }

        Ok(lhs)
    }

    /// Parse prefix / atom expressions (NUD position).
    fn parse_prefix(&mut self) -> Result<SpannedExpr, LyraError> {
        let tok = self.peek_token().clone();
        match &tok.kind {
            // Literals
            TokenKind::IntLit(n) => {
                let n = *n;
                self.advance();
                Ok(Spanned::new(Expr::IntLit(n), tok.span))
            }
            TokenKind::FloatLit(n) => {
                let n = *n;
                self.advance();
                Ok(Spanned::new(Expr::FloatLit(n), tok.span))
            }
            TokenKind::StringLit(s) => {
                let s = s.clone();
                self.advance();
                Ok(Spanned::new(Expr::StringLit(s), tok.span))
            }
            TokenKind::InterpolatedString(parts) => {
                let parts = parts.clone();
                let span = tok.span;
                self.advance();
                self.parse_interpolated_string(parts, span)
            }
            TokenKind::BoolLit(b) => {
                let b = *b;
                self.advance();
                Ok(Spanned::new(Expr::BoolLit(b), tok.span))
            }

            // Variable or Constructor
            TokenKind::Ident(_) => {
                let ident = self.expect_ident()?;
                Ok(Spanned::new(Expr::Var(ident.node), ident.span))
            }

            // Unary minus
            TokenKind::Minus => {
                let start = tok.span;
                self.advance();
                let operand = self.parse_expr_bp(17)?; // highest precedence
                let span = start.merge(operand.span);
                Ok(Spanned::new(
                    Expr::UnaryOp {
                        op: UnaryOp::Neg,
                        operand: Box::new(operand),
                    },
                    span,
                ))
            }

            // Unary not
            TokenKind::Not => {
                let start = tok.span;
                self.advance();
                let operand = self.parse_expr_bp(17)?;
                let span = start.merge(operand.span);
                Ok(Spanned::new(
                    Expr::UnaryOp {
                        op: UnaryOp::Not,
                        operand: Box::new(operand),
                    },
                    span,
                ))
            }

            // Parenthesized expr, unit, or tuple
            TokenKind::LParen => self.parse_paren_expr(),

            // List literal
            TokenKind::LBracket => self.parse_list_literal(),

            // Record literal: { field: expr, ... }
            TokenKind::LBrace => self.parse_record_literal(),

            // Lambda: fn (params) -> body
            TokenKind::Fn => self.parse_lambda(),

            // If expression
            TokenKind::If => self.parse_if(),

            // Let expression: let x = e1 in e2
            TokenKind::Let => self.parse_let_expr(),

            // Match expression
            TokenKind::Match => self.parse_match(),

            _ => Err(LyraError::ExpectedExpression {
                found: tok.kind.describe().to_string(),
                span: tok.span,
            }),
        }
    }

    fn parse_paren_expr(&mut self) -> Result<SpannedExpr, LyraError> {
        let start = self.peek_span();
        self.advance(); // consume '('

        // Unit: ()
        if matches!(self.peek(), TokenKind::RParen) {
            self.advance();
            let span = start.merge(self.previous_span());
            return Ok(Spanned::new(Expr::UnitLit, span));
        }

        let first = self.parse_expr()?;

        // Tuple: (a, b, ...)
        if matches!(self.peek(), TokenKind::Comma) {
            let mut elems = vec![first];
            while self.match_token(&TokenKind::Comma) {
                elems.push(self.parse_expr()?);
            }
            self.expect(&TokenKind::RParen)?;
            let span = start.merge(self.previous_span());
            return Ok(Spanned::new(Expr::TupleLit(elems), span));
        }

        // Parenthesized expression
        self.expect(&TokenKind::RParen)?;
        Ok(first)
    }

    fn parse_list_literal(&mut self) -> Result<SpannedExpr, LyraError> {
        let start = self.peek_span();
        self.advance(); // consume '['

        let mut elems = Vec::new();
        if !matches!(self.peek(), TokenKind::RBracket) {
            elems.push(self.parse_expr()?);
            while self.match_token(&TokenKind::Comma) {
                elems.push(self.parse_expr()?);
            }
        }

        self.expect(&TokenKind::RBracket)?;
        let span = start.merge(self.previous_span());
        Ok(Spanned::new(Expr::ListLit(elems), span))
    }

    fn parse_lambda(&mut self) -> Result<SpannedExpr, LyraError> {
        let start = self.peek_span();
        self.advance(); // consume 'fn'

        self.expect(&TokenKind::LParen)?;
        let mut params = Vec::new();
        if !matches!(self.peek(), TokenKind::RParen) {
            params.push(self.parse_lambda_param()?);
            while self.match_token(&TokenKind::Comma) {
                params.push(self.parse_lambda_param()?);
            }
        }
        self.expect(&TokenKind::RParen)?;
        self.expect(&TokenKind::Arrow)?;

        let body = self.parse_expr()?;
        let span = start.merge(body.span);
        Ok(Spanned::new(
            Expr::Lambda {
                params,
                body: Box::new(body),
            },
            span,
        ))
    }

    fn parse_lambda_param(&mut self) -> Result<LambdaParam, LyraError> {
        let name = self.expect_ident()?;
        let type_ann = if self.match_token(&TokenKind::Colon) {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };
        Ok(LambdaParam { name, type_ann })
    }

    fn parse_if(&mut self) -> Result<SpannedExpr, LyraError> {
        let start = self.peek_span();
        self.advance(); // consume 'if'

        let cond = self.parse_expr()?;
        self.expect(&TokenKind::Then)?;
        let then_branch = self.parse_expr()?;
        self.expect(&TokenKind::Else)?;
        let else_branch = self.parse_expr()?;

        let span = start.merge(else_branch.span);
        Ok(Spanned::new(
            Expr::If {
                cond: Box::new(cond),
                then_branch: Box::new(then_branch),
                else_branch: Box::new(else_branch),
            },
            span,
        ))
    }

    fn parse_let_expr(&mut self) -> Result<SpannedExpr, LyraError> {
        let start = self.peek_span();
        self.advance(); // consume 'let'

        let recursive = self.match_token(&TokenKind::Rec);
        let name = self.expect_ident()?;

        let type_ann = if self.match_token(&TokenKind::Colon) {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        self.expect(&TokenKind::Eq)?;
        let value = self.parse_expr()?;
        self.expect(&TokenKind::In)?;
        let body = self.parse_expr()?;

        let span = start.merge(body.span);
        Ok(Spanned::new(
            Expr::Let {
                name,
                recursive,
                type_ann,
                value: Box::new(value),
                body: Box::new(body),
            },
            span,
        ))
    }

    fn parse_match(&mut self) -> Result<SpannedExpr, LyraError> {
        let start = self.peek_span();
        self.advance(); // consume 'match'

        let scrutinee = self.parse_expr()?;
        self.expect(&TokenKind::With)?;

        let mut arms = Vec::new();
        // First arm can optionally have |
        self.match_token(&TokenKind::Pipe);
        arms.push(self.parse_match_arm()?);

        while self.match_token(&TokenKind::Pipe) {
            arms.push(self.parse_match_arm()?);
        }

        let last_span = arms.last().map(|a| a.body.span).unwrap_or(start);
        let span = start.merge(last_span);
        Ok(Spanned::new(
            Expr::Match {
                scrutinee: Box::new(scrutinee),
                arms,
            },
            span,
        ))
    }

    fn parse_match_arm(&mut self) -> Result<MatchArm, LyraError> {
        let pattern = self.parse_pattern()?;
        self.expect(&TokenKind::Arrow)?;
        let body = self.parse_expr()?;
        Ok(MatchArm { pattern, body })
    }

    fn parse_call(&mut self, func: SpannedExpr) -> Result<SpannedExpr, LyraError> {
        self.advance(); // consume '('
        let mut args = Vec::new();
        if !matches!(self.peek(), TokenKind::RParen) {
            args.push(self.parse_expr()?);
            while self.match_token(&TokenKind::Comma) {
                args.push(self.parse_expr()?);
            }
        }
        self.expect(&TokenKind::RParen)?;
        let span = func.span.merge(self.previous_span());
        Ok(Spanned::new(
            Expr::App {
                func: Box::new(func),
                args,
            },
            span,
        ))
    }

    fn parse_record_literal(&mut self) -> Result<SpannedExpr, LyraError> {
        let start = self.peek_span();
        self.advance(); // consume '{'

        let mut fields = Vec::new();
        if !matches!(self.peek(), TokenKind::RBrace) {
            // Parse first field: name: expr
            let name = self.expect_ident()?;
            self.expect(&TokenKind::Colon)?;
            let value = self.parse_expr()?;
            fields.push((name.node, value));

            while self.match_token(&TokenKind::Comma) {
                if matches!(self.peek(), TokenKind::RBrace) {
                    break; // allow trailing comma
                }
                let name = self.expect_ident()?;
                self.expect(&TokenKind::Colon)?;
                let value = self.parse_expr()?;
                fields.push((name.node, value));
            }
        }

        self.expect(&TokenKind::RBrace)?;
        let span = start.merge(self.previous_span());
        Ok(Spanned::new(Expr::Record(fields), span))
    }

    fn parse_interpolated_string(
        &mut self,
        parts: Vec<InterpPart>,
        span: Span,
    ) -> Result<SpannedExpr, LyraError> {
        let mut interp_parts = Vec::new();
        for part in parts {
            match part {
                InterpPart::Literal(s) => {
                    interp_parts.push(InterpolationPart::Literal(s));
                }
                InterpPart::Tokens(tokens) => {
                    // Add an Eof token so the sub-parser knows when to stop
                    let mut tokens_with_eof = tokens;
                    tokens_with_eof.push(crate::lexer::token::Token::new(
                        TokenKind::Eof,
                        span,
                    ));
                    let mut sub_parser = super::Parser::new(tokens_with_eof);
                    let expr = sub_parser.parse_expr()?;
                    interp_parts.push(InterpolationPart::Expr(expr));
                }
            }
        }
        Ok(Spanned::new(Expr::Interpolation(interp_parts), span))
    }
}
