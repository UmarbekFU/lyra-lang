use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::Context;
use rustyline::Helper;
use std::borrow::Cow;

pub struct LyraHelper;

const KEYWORDS: &[&str] = &[
    "let", "in", "fn", "match", "with", "if", "then", "else", "type", "rec", "true", "false",
];

impl Helper for LyraHelper {}

impl Completer for LyraHelper {
    type Candidate = Pair;
    fn complete(
        &self,
        _line: &str,
        _pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        Ok((0, vec![]))
    }
}

impl Hinter for LyraHelper {
    type Hint = String;
    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        None
    }
}

impl Highlighter for LyraHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        let mut result = String::with_capacity(line.len() + 64);
        let chars: Vec<char> = line.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            let ch = chars[i];

            // Comments
            if ch == '-' && i + 1 < len && chars[i + 1] == '-' {
                result.push_str("\x1b[90m");
                while i < len {
                    result.push(chars[i]);
                    i += 1;
                }
                result.push_str("\x1b[0m");
                continue;
            }

            // String literals
            if ch == '"' {
                result.push_str("\x1b[32m");
                result.push(ch);
                i += 1;
                while i < len && chars[i] != '"' {
                    if chars[i] == '\\' && i + 1 < len {
                        result.push(chars[i]);
                        i += 1;
                    }
                    result.push(chars[i]);
                    i += 1;
                }
                if i < len {
                    result.push(chars[i]);
                    i += 1;
                }
                result.push_str("\x1b[0m");
                continue;
            }

            // Numbers
            if ch.is_ascii_digit() {
                result.push_str("\x1b[36m");
                while i < len && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    result.push(chars[i]);
                    i += 1;
                }
                result.push_str("\x1b[0m");
                continue;
            }

            // Identifiers / keywords
            if ch.is_alphabetic() || ch == '_' {
                let mut word = String::new();
                while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    word.push(chars[i]);
                    i += 1;
                }
                if KEYWORDS.contains(&word.as_str()) {
                    result.push_str("\x1b[1;34m");
                    result.push_str(&word);
                    result.push_str("\x1b[0m");
                } else if word.starts_with(|c: char| c.is_uppercase()) {
                    result.push_str("\x1b[33m");
                    result.push_str(&word);
                    result.push_str("\x1b[0m");
                } else {
                    result.push_str(&word);
                }
                continue;
            }

            // Operators
            if "|>-><=!&:+*/%".contains(ch) {
                result.push_str("\x1b[33m");
                result.push(ch);
                result.push_str("\x1b[0m");
                i += 1;
                continue;
            }

            result.push(ch);
            i += 1;
        }

        Cow::Owned(result)
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _forced: bool) -> bool {
        true
    }
}

impl Validator for LyraHelper {
    fn validate(&self, _ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        Ok(ValidationResult::Valid(None))
    }
}
