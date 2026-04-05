//! Linter tests

use crate::{lint, LintRule};

#[test]
fn test_empty_body() {
    let source = "fn unused():\n    pass\n";
    let warnings = lint(source);
    assert!(warnings.iter().any(|w| w.rule == LintRule::EmptyBody));
}

#[test]
fn test_naming_convention_function() {
    let source = "fn MyFunction():\n    return 0\n";
    let warnings = lint(source);
    assert!(warnings.iter().any(|w| w.rule == LintRule::NamingConvention));
}

#[test]
fn test_naming_convention_ok() {
    let source = "fn my_function():\n    return 0\n";
    let warnings = lint(source);
    assert!(!warnings.iter().any(|w| w.rule == LintRule::NamingConvention));
}

#[test]
fn test_missing_return_type() {
    let source = "fn compute():\n    return 42\n";
    let warnings = lint(source);
    assert!(warnings.iter().any(|w| w.rule == LintRule::MissingReturnType));
}

#[test]
fn test_return_type_present() {
    let source = "fn compute() -> Int:\n    return 42\n";
    let warnings = lint(source);
    assert!(!warnings.iter().any(|w| w.rule == LintRule::MissingReturnType));
}

#[test]
fn test_unreachable_code() {
    let source = "fn foo() -> Int:\n    return 1\n    return 2\n";
    let warnings = lint(source);
    assert!(warnings.iter().any(|w| w.rule == LintRule::UnreachableCode));
}

#[test]
fn test_no_unreachable_code() {
    let source = "fn foo() -> Int:\n    let x = 1\n    return x\n";
    let warnings = lint(source);
    assert!(!warnings.iter().any(|w| w.rule == LintRule::UnreachableCode));
}

#[test]
fn test_lint_rule_codes() {
    assert_eq!(LintRule::EmptyBody.code(), "W001");
    assert_eq!(LintRule::NamingConvention.code(), "W002");
    assert_eq!(LintRule::MissingReturnType.code(), "W003");
    assert_eq!(LintRule::AgentMissingModel.code(), "W004");
    assert_eq!(LintRule::UnreachableCode.code(), "W005");
    assert_eq!(LintRule::UnusedImport.code(), "W006");
}

#[test]
fn test_clean_code_no_warnings() {
    let source = "fn main():\n    return 0\n";
    let warnings = lint(source);
    // main is exempt from missing return type
    assert!(warnings.is_empty());
}
