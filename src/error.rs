use std::fmt;

use crate::span::Span;

#[derive(Debug, Clone)]
pub enum LyraError {
    // Lexer errors
    UnexpectedChar { ch: char, span: Span },
    UnterminatedString { span: Span },

    // Parser errors
    UnexpectedToken { expected: String, found: String, span: Span },
    ExpectedExpression { found: String, span: Span },

    // Type errors
    TypeMismatch { expected: String, found: String, span: Span },
    InfiniteType { var: String, ty: String, span: Span },
    UndefinedVariable { name: String, suggestion: Option<String>, span: Span },
    UndefinedType { name: String, span: Span },
    UndefinedConstructor { name: String, span: Span },
    NonExhaustivePatterns { missing: Vec<String>, span: Span },
    ArityMismatch { name: String, expected: usize, found: usize, span: Span },

    // Runtime errors
    DivisionByZero { span: Span },
    IndexOutOfBounds { index: i64, length: usize, span: Span },
    NotCallable { span: Span },
    MatchFailure { span: Span },
    RuntimeError { message: String, span: Span },

    // IO errors
    FileNotFound { path: String },
    IoError { msg: String },
}

impl LyraError {
    pub fn span(&self) -> Option<Span> {
        match self {
            LyraError::UnexpectedChar { span, .. }
            | LyraError::UnterminatedString { span, .. }
            | LyraError::UnexpectedToken { span, .. }
            | LyraError::ExpectedExpression { span, .. }
            | LyraError::TypeMismatch { span, .. }
            | LyraError::InfiniteType { span, .. }
            | LyraError::UndefinedVariable { span, .. }
            | LyraError::UndefinedType { span, .. }
            | LyraError::UndefinedConstructor { span, .. }
            | LyraError::NonExhaustivePatterns { span, .. }
            | LyraError::ArityMismatch { span, .. }
            | LyraError::DivisionByZero { span, .. }
            | LyraError::IndexOutOfBounds { span, .. }
            | LyraError::NotCallable { span, .. }
            | LyraError::MatchFailure { span, .. }
            | LyraError::RuntimeError { span, .. } => Some(*span),
            LyraError::FileNotFound { .. } | LyraError::IoError { .. } => None,
        }
    }

    fn message(&self) -> String {
        match self {
            LyraError::UnexpectedChar { ch, .. } => {
                format!("unexpected character '{}'", ch)
            }
            LyraError::UnterminatedString { .. } => "unterminated string literal".to_string(),
            LyraError::UnexpectedToken {
                expected, found, ..
            } => {
                format!("expected {}, found {}", expected, found)
            }
            LyraError::ExpectedExpression { found, .. } => {
                format!("expected expression, found {}", found)
            }
            LyraError::TypeMismatch {
                expected, found, ..
            } => {
                format!("type mismatch: expected {}, found {}", expected, found)
            }
            LyraError::InfiniteType { var, ty, .. } => {
                format!("infinite type: {} occurs in {}", var, ty)
            }
            LyraError::UndefinedVariable { name, suggestion, .. } => {
                if let Some(s) = suggestion {
                    format!("undefined variable '{}'. Did you mean '{}'?", name, s)
                } else {
                    format!("undefined variable '{}'", name)
                }
            }
            LyraError::UndefinedType { name, .. } => {
                format!("undefined type '{}'", name)
            }
            LyraError::UndefinedConstructor { name, .. } => {
                format!("undefined constructor '{}'", name)
            }
            LyraError::NonExhaustivePatterns { missing, .. } => {
                format!("non-exhaustive patterns: missing {}", missing.join(", "))
            }
            LyraError::ArityMismatch {
                name,
                expected,
                found,
                ..
            } => {
                format!(
                    "'{}' expects {} arguments, but got {}",
                    name, expected, found
                )
            }
            LyraError::DivisionByZero { .. } => "division by zero".to_string(),
            LyraError::IndexOutOfBounds { index, length, .. } => {
                format!("index {} out of bounds for length {}", index, length)
            }
            LyraError::NotCallable { .. } => "value is not callable".to_string(),
            LyraError::MatchFailure { .. } => "no matching pattern found".to_string(),
            LyraError::RuntimeError { message, .. } => message.clone(),
            LyraError::FileNotFound { path } => format!("file not found: {}", path),
            LyraError::IoError { msg } => format!("IO error: {}", msg),
        }
    }

    fn label(&self) -> String {
        match self {
            LyraError::TypeMismatch {
                expected, found, ..
            } => format!("expected {}, found {}", expected, found),
            _ => self.message(),
        }
    }

    fn kind_str(&self) -> &'static str {
        match self {
            LyraError::UnexpectedChar { .. }
            | LyraError::UnterminatedString { .. } => "syntax error",
            LyraError::UnexpectedToken { .. }
            | LyraError::ExpectedExpression { .. } => "parse error",
            LyraError::TypeMismatch { .. }
            | LyraError::InfiniteType { .. }
            | LyraError::UndefinedVariable { .. }
            | LyraError::UndefinedType { .. }
            | LyraError::UndefinedConstructor { .. }
            | LyraError::NonExhaustivePatterns { .. }
            | LyraError::ArityMismatch { .. } => "type error",
            LyraError::DivisionByZero { .. }
            | LyraError::IndexOutOfBounds { .. }
            | LyraError::NotCallable { .. }
            | LyraError::MatchFailure { .. }
            | LyraError::RuntimeError { .. } => "runtime error",
            LyraError::FileNotFound { .. }
            | LyraError::IoError { .. } => "io error",
        }
    }

    /// Render error with source snippet and caret pointing to the span.
    pub fn render(&self, source: &str, filename: &str) -> String {
        let msg = self.message();
        let kind = self.kind_str();

        let span = match self.span() {
            Some(s) => s,
            None => return format!("\x1b[1;31m{}\x1b[0m: {}", kind, msg),
        };

        let (line_num, col, line_text) = locate_in_source(source, span);
        let width = line_num.to_string().len();
        let caret_len = span.len().max(1).min(line_text.len().saturating_sub(col.saturating_sub(1)));
        let label = self.label();

        format!(
            "\x1b[1;31m{kind}\x1b[0m: {msg}\n \x1b[1;34m-->\x1b[0m {file}:{line}:{col}\n{pad} \x1b[1;34m|\x1b[0m\n\x1b[1;34m{line_num:>width$}\x1b[0m \x1b[1;34m|\x1b[0m {line_text}\n{pad} \x1b[1;34m|\x1b[0m {spaces}\x1b[1;31m{carets} {label}\x1b[0m",
            kind = kind,
            msg = msg,
            file = filename,
            line = line_num,
            col = col,
            pad = " ".repeat(width),
            width = width,
            line_text = line_text,
            spaces = " ".repeat(col.saturating_sub(1)),
            carets = "^".repeat(caret_len.max(1)),
            label = label,
        )
    }
}

fn locate_in_source(source: &str, span: Span) -> (usize, usize, &str) {
    let mut line_num = 1;
    let mut line_start = 0;

    for (i, ch) in source.char_indices() {
        if i >= span.start {
            break;
        }
        if ch == '\n' {
            line_num += 1;
            line_start = i + 1;
        }
    }

    let col = span.start - line_start + 1;
    let line_end = source[line_start..]
        .find('\n')
        .map(|i| line_start + i)
        .unwrap_or(source.len());
    let line_text = &source[line_start..line_end];

    (line_num, col, line_text)
}

impl fmt::Display for LyraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind_str(), self.message())
    }
}

impl std::error::Error for LyraError {}

/// Levenshtein edit distance between two strings.
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in 0..=m {
        dp[i][0] = i;
    }
    for j in 0..=n {
        dp[0][j] = j;
    }
    for i in 1..=m {
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }
    dp[m][n]
}

/// Find the closest match to `name` in `candidates` by edit distance.
pub fn suggest_similar(name: &str, candidates: &[&str]) -> Option<String> {
    let threshold = match name.len() {
        0..=2 => 1,
        3..=5 => 2,
        _ => 3,
    };
    candidates
        .iter()
        .filter(|c| {
            let dist = levenshtein(name, c);
            dist > 0 && dist <= threshold
        })
        .min_by_key(|c| levenshtein(name, c))
        .map(|c| c.to_string())
}
