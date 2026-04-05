//! A16 Linter
//!
//! Rule-based source code linter producing warnings and suggestions.

mod rules;

pub use rules::{LintWarning, LintSeverity, LintRule};

/// Lint A16 source code and return warnings
pub fn lint(source: &str) -> Vec<LintWarning> {
    let module = match a16_parser::parse(source) {
        Ok(m) => m,
        Err(_) => return vec![],  // Parser errors are handled separately
    };

    let mut warnings = Vec::new();
    rules::check_all(&module, &mut warnings);
    warnings
}

/// Lint an already-parsed module
pub fn lint_module(module: &a16_ast::Module) -> Vec<LintWarning> {
    let mut warnings = Vec::new();
    rules::check_all(module, &mut warnings);
    warnings
}

#[cfg(test)]
mod tests;
