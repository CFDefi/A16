//! LSP backend tests

use crate::{LanguageBackend, diagnostics, completion, hover};

#[test]
fn test_backend_new() {
    let backend = LanguageBackend::new();
    assert!(backend.documents.is_empty());
}

#[test]
fn test_backend_update_document() {
    let mut backend = LanguageBackend::new();
    backend.update_document("file:///test.a16", "fn main():\n    pass\n".to_string(), 1);
    assert_eq!(backend.documents.len(), 1);
    assert!(backend.documents.contains_key("file:///test.a16"));
}

#[test]
fn test_backend_close_document() {
    let mut backend = LanguageBackend::new();
    backend.update_document("file:///test.a16", "fn main():\n    pass\n".to_string(), 1);
    backend.close_document("file:///test.a16");
    assert!(backend.documents.is_empty());
}

#[test]
fn test_diagnostics_clean() {
    let diags = diagnostics::compute_diagnostics("fn main():\n    return 0\n");
    // Clean code should have no errors (may have lint hints)
    let errors: Vec<_> = diags.iter()
        .filter(|d| d.severity == diagnostics::DiagnosticSeverity::Error)
        .collect();
    assert!(errors.is_empty());
}

#[test]
fn test_diagnostics_parse_error() {
    let diags = diagnostics::compute_diagnostics("fn ::::");
    assert!(!diags.is_empty());
    assert_eq!(diags[0].severity, diagnostics::DiagnosticSeverity::Error);
}

#[test]
fn test_completions() {
    let items = completion::compute_completions("");
    assert!(!items.is_empty());
    // Should contain keywords
    assert!(items.iter().any(|i| i.label == "fn"));
    assert!(items.iter().any(|i| i.label == "agent"));
    // Should contain built-in functions
    assert!(items.iter().any(|i| i.label == "println"));
    assert!(items.iter().any(|i| i.label == "len"));
}

#[test]
fn test_hover_keyword() {
    let info = hover::compute_hover("fn").unwrap();
    assert_eq!(info.label, "fn");
    assert!(info.documentation.is_some());
}

#[test]
fn test_hover_builtin() {
    let info = hover::compute_hover("println").unwrap();
    assert!(info.detail.contains("fn println"));
}

#[test]
fn test_hover_type() {
    let info = hover::compute_hover("Int").unwrap();
    assert!(info.detail.contains("integer"));
}

#[test]
fn test_hover_unknown() {
    let info = hover::compute_hover("xyzzy_unknown_word");
    assert!(info.is_none());
}

#[test]
fn test_completion_kinds() {
    let items = completion::compute_completions("");
    let keywords: Vec<_> = items.iter()
        .filter(|i| i.kind == completion::CompletionKind::Keyword)
        .collect();
    let functions: Vec<_> = items.iter()
        .filter(|i| i.kind == completion::CompletionKind::Function)
        .collect();
    let snippets: Vec<_> = items.iter()
        .filter(|i| i.kind == completion::CompletionKind::Snippet)
        .collect();
    assert!(keywords.len() >= 25);
    assert!(functions.len() >= 15);
    assert!(snippets.len() >= 3);
}

#[test]
fn test_diagnostics_with_lint_warnings() {
    let source = "fn MyBadName():\n    pass\n";
    let diags = diagnostics::compute_diagnostics(source);
    // Should have lint warnings for naming and empty body
    let warnings: Vec<_> = diags.iter()
        .filter(|d| d.severity == diagnostics::DiagnosticSeverity::Warning 
                  || d.severity == diagnostics::DiagnosticSeverity::Information
                  || d.severity == diagnostics::DiagnosticSeverity::Hint)
        .collect();
    assert!(!warnings.is_empty());
}
