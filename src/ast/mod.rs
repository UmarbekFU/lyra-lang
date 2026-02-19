pub mod pretty;

use crate::span::{Span, Spanned};

pub type SpannedExpr = Spanned<Expr>;
pub type SpannedPattern = Spanned<Pattern>;
pub type SpannedTypeAnn = Spanned<TypeAnnotation>;

/// Top-level declaration.
#[derive(Debug, Clone)]
pub enum Decl {
    /// `let x = expr` or `let rec f = expr`
    Let {
        name: Spanned<String>,
        recursive: bool,
        type_ann: Option<SpannedTypeAnn>,
        body: SpannedExpr,
    },
    /// `type Option a = Some a | None`
    Type {
        name: Spanned<String>,
        type_params: Vec<Spanned<String>>,
        variants: Vec<Variant>,
    },
    /// `import "path"`
    Import {
        path: String,
        span: Span,
    },
    /// Bare expression (for REPL).
    Expr(SpannedExpr),
}

#[derive(Debug, Clone)]
pub struct Variant {
    pub name: Spanned<String>,
    pub fields: Vec<SpannedTypeAnn>,
    pub span: Span,
}

/// Expressions.
#[derive(Debug, Clone)]
pub enum Expr {
    // Literals
    IntLit(i64),
    FloatLit(f64),
    StringLit(String),
    BoolLit(bool),
    UnitLit,

    // Collections
    ListLit(Vec<SpannedExpr>),
    TupleLit(Vec<SpannedExpr>),

    // Variable reference
    Var(String),

    // Function literal: fn (a, b) -> body
    Lambda {
        params: Vec<LambdaParam>,
        body: Box<SpannedExpr>,
    },

    // Function application: f(x, y)
    App {
        func: Box<SpannedExpr>,
        args: Vec<SpannedExpr>,
    },

    // Binary operation
    BinOp {
        op: BinOp,
        lhs: Box<SpannedExpr>,
        rhs: Box<SpannedExpr>,
    },

    // Unary operation
    UnaryOp {
        op: UnaryOp,
        operand: Box<SpannedExpr>,
    },

    // Pipe: expr |> func
    Pipe {
        lhs: Box<SpannedExpr>,
        rhs: Box<SpannedExpr>,
    },

    // If expression
    If {
        cond: Box<SpannedExpr>,
        then_branch: Box<SpannedExpr>,
        else_branch: Box<SpannedExpr>,
    },

    // Let expression: let x = e1 in e2
    Let {
        name: Spanned<String>,
        recursive: bool,
        type_ann: Option<SpannedTypeAnn>,
        value: Box<SpannedExpr>,
        body: Box<SpannedExpr>,
    },

    // Match expression
    Match {
        scrutinee: Box<SpannedExpr>,
        arms: Vec<MatchArm>,
    },

    // String interpolation: "hello {name}"
    Interpolation(Vec<InterpolationPart>),

    // Record literal: { name: "Alice", age: 30 }
    Record(Vec<(String, SpannedExpr)>),

    // Field access: expr.field
    FieldAccess {
        expr: Box<SpannedExpr>,
        field: String,
    },
}

/// Part of a string interpolation.
#[derive(Debug, Clone)]
pub enum InterpolationPart {
    Literal(String),
    Expr(SpannedExpr),
}

#[derive(Debug, Clone)]
pub struct LambdaParam {
    pub name: Spanned<String>,
    pub type_ann: Option<SpannedTypeAnn>,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: SpannedPattern,
    pub body: SpannedExpr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    Cons,
}

impl BinOp {
    pub fn as_str(&self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Mod => "%",
            BinOp::Eq => "==",
            BinOp::NotEq => "!=",
            BinOp::Lt => "<",
            BinOp::Gt => ">",
            BinOp::Le => "<=",
            BinOp::Ge => ">=",
            BinOp::And => "&&",
            BinOp::Or => "||",
            BinOp::Cons => "::",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

/// Patterns for match expressions.
#[derive(Debug, Clone)]
pub enum Pattern {
    Wildcard,
    Var(String),
    IntLit(i64),
    FloatLit(f64),
    StringLit(String),
    BoolLit(bool),
    UnitLit,
    Tuple(Vec<SpannedPattern>),
    List(Vec<SpannedPattern>),
    Cons(Box<SpannedPattern>, Box<SpannedPattern>),
    Constructor {
        name: String,
        args: Vec<SpannedPattern>,
    },
}

/// Type annotations written by the user.
#[derive(Debug, Clone)]
pub enum TypeAnnotation {
    Named(String),                                    // Int, Bool, etc.
    Var(String),                                      // a, b (type variables)
    Arrow(Box<SpannedTypeAnn>, Box<SpannedTypeAnn>),  // Int -> Bool
    App(Box<SpannedTypeAnn>, Vec<SpannedTypeAnn>),    // Option Int
    Tuple(Vec<SpannedTypeAnn>),                       // (Int, String)
    List(Box<SpannedTypeAnn>),                        // [Int]
    Unit,                                             // ()
}
