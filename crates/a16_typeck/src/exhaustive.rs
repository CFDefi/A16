//! Pattern Exhaustiveness Checker
//!
//! Implements a simplified exhaustiveness analysis for match statements.
//! Based on the approach from Maranget's "Warnings for Pattern Matching"
//! but adapted for A16's type system.

use crate::types::Type;
use crate::error::TypeError;
use a16_ast::*;

/// Result of exhaustiveness analysis
#[derive(Debug)]
pub struct ExhaustivenessResult {
    /// Whether the match is exhaustive
    pub is_exhaustive: bool,
    /// Missing patterns (if not exhaustive)
    pub missing_patterns: Vec<String>,
    /// Unreachable arms (if any)
    pub unreachable_arms: Vec<usize>,
}

/// Check if a match statement's patterns are exhaustive
pub fn check_exhaustiveness(
    subject_ty: &Type,
    arms: &[MatchArm],
) -> ExhaustivenessResult {
    let mut has_wildcard = false;
    let mut has_var_bind = false;
    let mut covered_literals = Vec::new();
    let mut unreachable_arms = Vec::new();

    for (i, arm) in arms.iter().enumerate() {
        if has_wildcard || has_var_bind {
            // Any arm after a wildcard/var is unreachable
            unreachable_arms.push(i);
            continue;
        }

        match &arm.pattern {
            Pattern::Wildcard(_) => {
                has_wildcard = true;
            }
            Pattern::Ident(_) => {
                // Variable binding matches everything
                has_var_bind = true;
            }
            Pattern::Literal(lit) => {
                let lit_str = literal_to_string(lit);
                if covered_literals.contains(&lit_str) {
                    unreachable_arms.push(i);
                } else {
                    covered_literals.push(lit_str);
                }
            }
            Pattern::Tuple(pats, _) => {
                // Tuple pattern: check if all sub-patterns are wildcards
                if pats.iter().all(|p| matches!(p, Pattern::Wildcard(_))) {
                    has_wildcard = true;
                }
            }
            Pattern::Or(pats, _) => {
                // Or-pattern: check sub-patterns
                for pat in pats {
                    if matches!(pat, Pattern::Wildcard(_)) {
                        has_wildcard = true;
                    }
                }
            }
            _ => {
                // Other complex patterns — conservatively mark as non-exhaustive
            }
        }
    }

    // Determine if exhaustive
    let is_exhaustive = has_wildcard || has_var_bind || is_fully_covered(subject_ty, &covered_literals);

    let missing_patterns = if is_exhaustive {
        Vec::new()
    } else {
        compute_missing_patterns(subject_ty, &covered_literals)
    };

    ExhaustivenessResult {
        is_exhaustive,
        missing_patterns,
        unreachable_arms,
    }
}

/// Check if all constructors of a type are covered by the given literals
fn is_fully_covered(ty: &Type, covered: &[String]) -> bool {
    match ty {
        Type::Bool => {
            covered.contains(&"true".to_string()) && covered.contains(&"false".to_string())
        }
        Type::None => {
            covered.contains(&"None".to_string())
        }
        // For numeric and string types, infinite domain — can never be exhaustive
        // without a wildcard
        Type::Int | Type::Float | Type::Str => false,
        // Optional type: need both Some and None
        Type::Optional(_) => {
            covered.contains(&"None".to_string()) && covered.len() >= 2
        }
        _ => false,
    }
}

/// Compute the missing patterns for diagnostics
fn compute_missing_patterns(ty: &Type, covered: &[String]) -> Vec<String> {
    match ty {
        Type::Bool => {
            let mut missing = Vec::new();
            if !covered.contains(&"true".to_string()) {
                missing.push("true".to_string());
            }
            if !covered.contains(&"false".to_string()) {
                missing.push("false".to_string());
            }
            if missing.is_empty() {
                missing.push("_".to_string());
            }
            missing
        }
        Type::Optional(inner) => {
            let mut missing = Vec::new();
            if !covered.contains(&"None".to_string()) {
                missing.push("None".to_string());
            }
            if covered.len() <= 1 {
                missing.push(format!("Some({})", inner));
            }
            missing
        }
        _ => vec!["_".to_string()],
    }
}

/// Convert a literal expression to a string for comparison
fn literal_to_string(expr: &Expr) -> String {
    match expr {
        Expr::Int(v, _) => v.to_string(),
        Expr::Float(v, _) => v.to_string(),
        Expr::String(v, _) => format!("\"{}\"", v),
        Expr::Bool(v, _) => v.to_string(),
        Expr::None(_) => "None".to_string(),
        _ => "_".to_string(),
    }
}

/// Generate a warning for non-exhaustive match patterns
pub fn non_exhaustive_warning(
    missing: &[String],
    span: Span,
) -> TypeError {
    let patterns = missing.join(", ");
    TypeError::custom(
        format!("Non-exhaustive match: missing patterns: {}", patterns),
        span,
    )
}

/// Generate a warning for unreachable match arms
pub fn unreachable_arm_warning(
    arm_index: usize,
    span: Span,
) -> TypeError {
    TypeError::custom(
        format!("Unreachable match arm #{}: previous patterns already cover this case", arm_index + 1),
        span,
    )
}
