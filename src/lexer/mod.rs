pub mod token;

use crate::error::LyraError;
use crate::span::Span;
use token::{Token, TokenKind};

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    start: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            chars: source.chars().collect(),
            pos: 0,
            start: 0,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, Vec<LyraError>> {
        let mut tokens = Vec::new();
        let mut errors = Vec::new();

        loop {
            self.skip_whitespace_and_comments();
            if self.is_at_end() {
                tokens.push(Token::new(TokenKind::Eof, Span::new(self.pos, self.pos)));
                break;
            }

            self.start = self.pos;
            match self.advance() {
                '(' => tokens.push(self.make_token(TokenKind::LParen)),
                ')' => tokens.push(self.make_token(TokenKind::RParen)),
                '[' => tokens.push(self.make_token(TokenKind::LBracket)),
                ']' => tokens.push(self.make_token(TokenKind::RBracket)),
                '{' => tokens.push(self.make_token(TokenKind::LBrace)),
                '}' => tokens.push(self.make_token(TokenKind::RBrace)),
                ',' => tokens.push(self.make_token(TokenKind::Comma)),
                '.' => tokens.push(self.make_token(TokenKind::Dot)),
                '+' => tokens.push(self.make_token(TokenKind::Plus)),
                '*' => tokens.push(self.make_token(TokenKind::Star)),
                '/' => tokens.push(self.make_token(TokenKind::Slash)),
                '%' => tokens.push(self.make_token(TokenKind::Percent)),

                '-' => {
                    if self.match_char('>') {
                        tokens.push(self.make_token(TokenKind::Arrow));
                    } else {
                        tokens.push(self.make_token(TokenKind::Minus));
                    }
                }

                '|' => {
                    if self.match_char('>') {
                        tokens.push(self.make_token(TokenKind::PipeRight));
                    } else if self.match_char('|') {
                        tokens.push(self.make_token(TokenKind::Or));
                    } else {
                        tokens.push(self.make_token(TokenKind::Pipe));
                    }
                }

                '=' => {
                    if self.match_char('=') {
                        tokens.push(self.make_token(TokenKind::EqEq));
                    } else {
                        tokens.push(self.make_token(TokenKind::Eq));
                    }
                }

                '!' => {
                    if self.match_char('=') {
                        tokens.push(self.make_token(TokenKind::NotEq));
                    } else {
                        tokens.push(self.make_token(TokenKind::Not));
                    }
                }

                '<' => {
                    if self.match_char('=') {
                        tokens.push(self.make_token(TokenKind::Le));
                    } else {
                        tokens.push(self.make_token(TokenKind::Lt));
                    }
                }

                '>' => {
                    if self.match_char('=') {
                        tokens.push(self.make_token(TokenKind::Ge));
                    } else {
                        tokens.push(self.make_token(TokenKind::Gt));
                    }
                }

                '&' => {
                    if self.match_char('&') {
                        tokens.push(self.make_token(TokenKind::And));
                    } else {
                        errors.push(LyraError::UnexpectedChar {
                            ch: '&',
                            span: self.current_span(),
                        });
                    }
                }

                ':' => {
                    if self.match_char(':') {
                        tokens.push(self.make_token(TokenKind::ColonColon));
                    } else {
                        tokens.push(self.make_token(TokenKind::Colon));
                    }
                }

                '_' if !self.peek().is_alphanumeric() && self.peek() != '_' => {
                    tokens.push(self.make_token(TokenKind::Underscore));
                }

                '"' => match self.scan_string() {
                    Ok(tok) => tokens.push(tok),
                    Err(e) => errors.push(e),
                },

                c if c.is_ascii_digit() => {
                    tokens.push(self.scan_number(c));
                }

                c if c.is_alphabetic() || c == '_' => {
                    tokens.push(self.scan_identifier(c));
                }

                c => {
                    errors.push(LyraError::UnexpectedChar {
                        ch: c,
                        span: self.current_span(),
                    });
                }
            }
        }

        if errors.is_empty() {
            Ok(tokens)
        } else {
            Err(errors)
        }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.chars.len()
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.chars[self.pos]
        }
    }

    fn peek_next(&self) -> char {
        if self.pos + 1 >= self.chars.len() {
            '\0'
        } else {
            self.chars[self.pos + 1]
        }
    }

    fn advance(&mut self) -> char {
        let ch = self.chars[self.pos];
        self.pos += 1;
        ch
    }

    fn match_char(&mut self, expected: char) -> bool {
        if !self.is_at_end() && self.chars[self.pos] == expected {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            if self.is_at_end() {
                break;
            }
            match self.peek() {
                ' ' | '\t' | '\r' | '\n' => {
                    self.advance();
                }
                '-' if self.peek_next() == '-' => {
                    // Line comment: skip to end of line
                    while !self.is_at_end() && self.peek() != '\n' {
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }

    fn make_token(&self, kind: TokenKind) -> Token {
        Token::new(kind, Span::new(self.start, self.pos))
    }

    fn current_span(&self) -> Span {
        Span::new(self.start, self.pos)
    }

    fn scan_string(&mut self) -> Result<Token, LyraError> {
        let mut current_lit = String::new();
        let mut parts: Vec<token::InterpPart> = Vec::new();
        let mut has_interpolation = false;

        while !self.is_at_end() && self.peek() != '"' {
            let ch = self.advance();
            if ch == '\\' {
                if self.is_at_end() {
                    return Err(LyraError::UnterminatedString {
                        span: self.current_span(),
                    });
                }
                let escaped = self.advance();
                match escaped {
                    'n' => current_lit.push('\n'),
                    't' => current_lit.push('\t'),
                    'r' => current_lit.push('\r'),
                    '\\' => current_lit.push('\\'),
                    '"' => current_lit.push('"'),
                    '{' => current_lit.push('{'),
                    '}' => current_lit.push('}'),
                    _ => {
                        current_lit.push('\\');
                        current_lit.push(escaped);
                    }
                }
            } else if ch == '{' {
                has_interpolation = true;
                // Save the literal part before this interpolation
                if !current_lit.is_empty() {
                    parts.push(token::InterpPart::Literal(current_lit.clone()));
                    current_lit.clear();
                }
                // Extract the source text inside {...} (tracking brace nesting)
                let mut depth = 1;
                let mut expr_src = String::new();
                while !self.is_at_end() && depth > 0 {
                    let c = self.advance();
                    if c == '{' {
                        depth += 1;
                        expr_src.push(c);
                    } else if c == '}' {
                        depth -= 1;
                        if depth > 0 {
                            expr_src.push(c);
                        }
                    } else {
                        expr_src.push(c);
                    }
                }
                if depth > 0 {
                    return Err(LyraError::UnterminatedString {
                        span: self.current_span(),
                    });
                }
                // Lex the expression source
                let mut inner_lexer = Lexer::new(&expr_src);
                let inner_tokens = inner_lexer.tokenize().map_err(|errs| errs[0].clone())?;
                // Remove the trailing Eof token
                let inner_tokens: Vec<_> = inner_tokens
                    .into_iter()
                    .filter(|t| !matches!(t.kind, TokenKind::Eof))
                    .collect();
                parts.push(token::InterpPart::Tokens(inner_tokens));
            } else {
                current_lit.push(ch);
            }
        }

        if self.is_at_end() {
            return Err(LyraError::UnterminatedString {
                span: self.current_span(),
            });
        }

        self.advance(); // closing "

        if has_interpolation {
            // Push trailing literal if any
            if !current_lit.is_empty() {
                parts.push(token::InterpPart::Literal(current_lit));
            }
            Ok(self.make_token(TokenKind::InterpolatedString(parts)))
        } else {
            Ok(self.make_token(TokenKind::StringLit(current_lit)))
        }
    }

    fn scan_number(&mut self, first: char) -> Token {
        let mut num_str = String::from(first);
        let mut is_float = false;

        while !self.is_at_end() && self.peek().is_ascii_digit() {
            num_str.push(self.advance());
        }

        // Check for decimal point
        if !self.is_at_end() && self.peek() == '.' && self.peek_next().is_ascii_digit() {
            is_float = true;
            num_str.push(self.advance()); // the '.'
            while !self.is_at_end() && self.peek().is_ascii_digit() {
                num_str.push(self.advance());
            }
        }

        if is_float {
            let val: f64 = num_str.parse().unwrap_or(0.0);
            self.make_token(TokenKind::FloatLit(val))
        } else {
            let val: i64 = num_str.parse().unwrap_or(0);
            self.make_token(TokenKind::IntLit(val))
        }
    }

    fn scan_identifier(&mut self, first: char) -> Token {
        let mut ident = String::from(first);
        while !self.is_at_end() && (self.peek().is_alphanumeric() || self.peek() == '_') {
            ident.push(self.advance());
        }

        let kind = match ident.as_str() {
            "let" => TokenKind::Let,
            "in" => TokenKind::In,
            "fn" => TokenKind::Fn,
            "match" => TokenKind::Match,
            "with" => TokenKind::With,
            "if" => TokenKind::If,
            "then" => TokenKind::Then,
            "else" => TokenKind::Else,
            "type" => TokenKind::Type,
            "rec" => TokenKind::Rec,
            "import" => TokenKind::Import,
            "true" => TokenKind::BoolLit(true),
            "false" => TokenKind::BoolLit(false),
            _ => TokenKind::Ident(ident),
        };

        self.make_token(kind)
    }
}

pub fn tokenize(source: &str) -> Result<Vec<Token>, Vec<LyraError>> {
    Lexer::new(source).tokenize()
}
