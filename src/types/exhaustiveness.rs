use std::collections::HashMap;

use crate::ast::Pattern;
use crate::span::Spanned;

use super::infer::ConstructorInfo;
use super::MonoType;

/// Check if a set of match patterns is exhaustive for a given scrutinee type.
/// Returns a list of missing pattern descriptions, or empty if exhaustive.
pub fn check_exhaustiveness(
    patterns: &[&Spanned<Pattern>],
    scrut_type: &MonoType,
    constructors: &HashMap<String, ConstructorInfo>,
) -> Vec<String> {
    // Wildcard or variable pattern covers everything
    if patterns.iter().any(|p| is_catch_all(&p.node)) {
        return vec![];
    }

    match scrut_type {
        MonoType::Bool => check_bool_exhaustiveness(patterns),
        MonoType::Con(type_name, _) => {
            check_adt_exhaustiveness(patterns, type_name, constructors)
        }
        MonoType::List(_) => check_list_exhaustiveness(patterns),
        // For other types (Int, String, Float, etc.), we can't check exhaustiveness
        // practically, but we warn if there's no catch-all
        _ => {
            if patterns.is_empty() {
                vec!["_".to_string()]
            } else {
                // Has some specific patterns but no wildcard — warn
                vec!["_ (wildcard)".to_string()]
            }
        }
    }
}

fn is_catch_all(pattern: &Pattern) -> bool {
    matches!(pattern, Pattern::Wildcard | Pattern::Var(_))
}

fn check_bool_exhaustiveness(patterns: &[&Spanned<Pattern>]) -> Vec<String> {
    let mut has_true = false;
    let mut has_false = false;
    for p in patterns {
        match &p.node {
            Pattern::BoolLit(true) => has_true = true,
            Pattern::BoolLit(false) => has_false = true,
            Pattern::Wildcard | Pattern::Var(_) => return vec![],
            _ => {}
        }
    }
    let mut missing = vec![];
    if !has_true {
        missing.push("true".to_string());
    }
    if !has_false {
        missing.push("false".to_string());
    }
    missing
}

fn check_adt_exhaustiveness(
    patterns: &[&Spanned<Pattern>],
    type_name: &str,
    constructors: &HashMap<String, ConstructorInfo>,
) -> Vec<String> {
    // Collect all constructors for this type
    let all_ctors: Vec<String> = constructors
        .iter()
        .filter(|(_, info)| info.type_name == type_name)
        .map(|(name, _)| name.clone())
        .collect();

    if all_ctors.is_empty() {
        return vec![];
    }

    // Check which constructors are covered by the patterns
    let mut covered: std::collections::HashSet<String> = std::collections::HashSet::new();
    for p in patterns {
        match &p.node {
            Pattern::Constructor { name, .. } => {
                covered.insert(name.clone());
            }
            Pattern::Wildcard | Pattern::Var(_) => return vec![],
            _ => {}
        }
    }

    let mut missing: Vec<String> = all_ctors
        .into_iter()
        .filter(|c| !covered.contains(c))
        .collect();
    missing.sort();
    missing
}

fn check_list_exhaustiveness(patterns: &[&Spanned<Pattern>]) -> Vec<String> {
    let mut has_empty = false;
    let mut has_cons = false;
    for p in patterns {
        match &p.node {
            Pattern::List(elems) if elems.is_empty() => has_empty = true,
            Pattern::List(_) => {} // specific length — partial coverage
            Pattern::Cons(_, _) => has_cons = true,
            Pattern::Wildcard | Pattern::Var(_) => return vec![],
            _ => {}
        }
    }
    let mut missing = vec![];
    if !has_empty {
        missing.push("[]".to_string());
    }
    if !has_cons {
        missing.push("(_ :: _)".to_string());
    }
    missing
}
