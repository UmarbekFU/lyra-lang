use std::fmt;

use crate::ast::*;

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::IntLit(n) => write!(f, "{}", n),
            Expr::FloatLit(n) => write!(f, "{}", n),
            Expr::BoolLit(b) => write!(f, "{}", b),
            Expr::StringLit(s) => write!(f, "\"{}\"", s),
            Expr::UnitLit => write!(f, "()"),
            Expr::Var(name) => write!(f, "{}", name),
            Expr::ListLit(elems) => {
                write!(f, "[")?;
                for (i, e) in elems.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", e.node)?;
                }
                write!(f, "]")
            }
            Expr::TupleLit(elems) => {
                write!(f, "(")?;
                for (i, e) in elems.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", e.node)?;
                }
                write!(f, ")")
            }
            Expr::Lambda { params, body } => {
                write!(f, "fn (")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p.name.node)?;
                }
                write!(f, ") -> {}", body.node)
            }
            Expr::App { func, args } => {
                write!(f, "{}(", func.node)?;
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", a.node)?;
                }
                write!(f, ")")
            }
            Expr::BinOp { op, lhs, rhs } => {
                write!(f, "({} {} {})", lhs.node, op.as_str(), rhs.node)
            }
            Expr::UnaryOp { op, operand } => match op {
                UnaryOp::Neg => write!(f, "(-{})", operand.node),
                UnaryOp::Not => write!(f, "(!{})", operand.node),
            },
            Expr::Pipe { lhs, rhs } => {
                write!(f, "{} |> {}", lhs.node, rhs.node)
            }
            Expr::If {
                cond,
                then_branch,
                else_branch,
            } => {
                write!(
                    f,
                    "if {} then {} else {}",
                    cond.node, then_branch.node, else_branch.node
                )
            }
            Expr::Let {
                name,
                recursive,
                value,
                body,
                ..
            } => {
                if *recursive {
                    write!(f, "let rec {} = {} in {}", name.node, value.node, body.node)
                } else {
                    write!(f, "let {} = {} in {}", name.node, value.node, body.node)
                }
            }
            Expr::Match { scrutinee, arms } => {
                write!(f, "match {} with", scrutinee.node)?;
                for arm in arms {
                    write!(f, " | {} -> {}", arm.pattern.node, arm.body.node)?;
                }
                Ok(())
            }

            Expr::Interpolation(parts) => {
                write!(f, "\"")?;
                for part in parts {
                    match part {
                        crate::ast::InterpolationPart::Literal(s) => write!(f, "{}", s)?,
                        crate::ast::InterpolationPart::Expr(e) => write!(f, "{{{}}}", e.node)?,
                    }
                }
                write!(f, "\"")
            }

            Expr::Record(fields) => {
                write!(f, "{{ ")?;
                for (i, (name, val)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", name, val.node)?;
                }
                write!(f, " }}")
            }

            Expr::FieldAccess { expr, field } => {
                write!(f, "{}.{}", expr.node, field)
            }
        }
    }
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Pattern::Wildcard => write!(f, "_"),
            Pattern::Var(name) => write!(f, "{}", name),
            Pattern::IntLit(n) => write!(f, "{}", n),
            Pattern::FloatLit(n) => write!(f, "{}", n),
            Pattern::StringLit(s) => write!(f, "\"{}\"", s),
            Pattern::BoolLit(b) => write!(f, "{}", b),
            Pattern::UnitLit => write!(f, "()"),
            Pattern::Tuple(pats) => {
                write!(f, "(")?;
                for (i, p) in pats.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p.node)?;
                }
                write!(f, ")")
            }
            Pattern::List(pats) => {
                write!(f, "[")?;
                for (i, p) in pats.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p.node)?;
                }
                write!(f, "]")
            }
            Pattern::Cons(head, tail) => {
                write!(f, "{} :: {}", head.node, tail.node)
            }
            Pattern::Constructor { name, args } => {
                write!(f, "{}", name)?;
                if !args.is_empty() {
                    write!(f, "(")?;
                    for (i, a) in args.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", a.node)?;
                    }
                    write!(f, ")")?;
                }
                Ok(())
            }
        }
    }
}

impl fmt::Display for TypeAnnotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeAnnotation::Named(name) => write!(f, "{}", name),
            TypeAnnotation::Var(name) => write!(f, "{}", name),
            TypeAnnotation::Arrow(from, to) => write!(f, "{} -> {}", from.node, to.node),
            TypeAnnotation::App(base, args) => {
                write!(f, "{}", base.node)?;
                for a in args {
                    write!(f, " {}", a.node)?;
                }
                Ok(())
            }
            TypeAnnotation::Tuple(elems) => {
                write!(f, "(")?;
                for (i, e) in elems.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", e.node)?;
                }
                write!(f, ")")
            }
            TypeAnnotation::List(inner) => write!(f, "[{}]", inner.node),
            TypeAnnotation::Unit => write!(f, "()"),
        }
    }
}

impl fmt::Display for Decl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Decl::Let {
                name,
                recursive,
                body,
                ..
            } => {
                if *recursive {
                    write!(f, "let rec {} = {}", name.node, body.node)
                } else {
                    write!(f, "let {} = {}", name.node, body.node)
                }
            }
            Decl::Type {
                name,
                type_params,
                variants,
            } => {
                write!(f, "type {}", name.node)?;
                for p in type_params {
                    write!(f, " {}", p.node)?;
                }
                write!(f, " =")?;
                for (i, v) in variants.iter().enumerate() {
                    if i > 0 {
                        write!(f, " |")?;
                    }
                    write!(f, " {}", v.name.node)?;
                    for field in &v.fields {
                        write!(f, " {}", field.node)?;
                    }
                }
                Ok(())
            }
            Decl::Expr(expr) => write!(f, "{}", expr.node),

            Decl::Import { path, .. } => write!(f, "import \"{}\"", path),
        }
    }
}
