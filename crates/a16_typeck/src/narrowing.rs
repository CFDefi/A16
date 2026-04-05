//! Type Narrowing
//!
//! Implements type narrowing for conditional checks.
//! After `if x is not None:`, the type of `x` is narrowed
//! from `Optional[T]` to `T` within the then-block.

use smol_str::SmolStr;
use crate::types::Type;

/// A narrowing context that tracks narrowed types within a scope
#[derive(Debug, Clone)]
pub struct NarrowingContext {
    /// Stack of narrowing scopes: variable name -> narrowed type
    scopes: Vec<Vec<(SmolStr, Type)>>,
}

impl NarrowingContext {
    pub fn new() -> Self {
        Self {
            scopes: vec![Vec::new()],
        }
    }

    /// Enter a new narrowing scope (e.g., inside an if-block)
    pub fn enter_scope(&mut self) {
        self.scopes.push(Vec::new());
    }

    /// Exit the current narrowing scope
    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    /// Add a narrowing for a variable in the current scope
    pub fn narrow(&mut self, name: SmolStr, ty: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.push((name, ty));
        }
    }

    /// Look up the narrowed type for a variable
    /// Returns the most recent (innermost) narrowing
    pub fn get_narrowed(&self, name: &str) -> Option<&Type> {
        for scope in self.scopes.iter().rev() {
            for (n, ty) in scope.iter().rev() {
                if n.as_str() == name {
                    return Some(ty);
                }
            }
        }
        None
    }
}

impl Default for NarrowingContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Attempt to narrow a type based on a condition expression.
///
/// Patterns recognized:
/// - `x is not None` → narrow Optional[T] to T
/// - `x is None` → narrow Optional[T] to None (in else branch, narrow to T)
/// - `type(x) == "Foo"` → narrow to Foo
pub fn narrow_type_from_condition(
    original_ty: &Type,
    is_negated: bool,
) -> Option<Type> {
    match original_ty {
        Type::Optional(inner) => {
            if is_negated {
                // `x is not None` → narrow to T
                Some((**inner).clone())
            } else {
                // `x is None` → narrow to None
                Some(Type::None)
            }
        }
        Type::Union(variants) => {
            if is_negated {
                // Remove None from union
                let filtered: Vec<Type> = variants.iter()
                    .filter(|t| !matches!(t, Type::None))
                    .cloned()
                    .collect();
                match filtered.len() {
                    0 => Some(Type::None),
                    1 => Some(filtered.into_iter().next().unwrap()),
                    _ => Some(Type::Union(filtered)),
                }
            } else {
                Some(Type::None)
            }
        }
        _ => None,
    }
}

/// Check if an expression is a `x is not None` or `x is None` pattern
/// Returns (variable_name, is_negated) if detected
pub fn detect_none_check(condition_desc: &str) -> Option<(String, bool)> {
    // Simple pattern detection from textual representation
    // In practice, this would be done by inspecting the AST directly
    if condition_desc.contains("is not None") {
        let var = condition_desc.split(" is not None").next()?;
        Some((var.trim().to_string(), true))
    } else if condition_desc.contains("is None") {
        let var = condition_desc.split(" is None").next()?;
        Some((var.trim().to_string(), false))
    } else {
        None
    }
}
