//! Diagnostic computation
//!
//! Converts parser errors, type errors, and lint warnings to LSP diagnostics.

/// Diagnostic severity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

/// A source range (0-indexed line/column)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

/// A diagnostic message
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub source: String,
    pub code: Option<String>,
}

/// Compute diagnostics for A16 source
pub fn compute_diagnostics(source: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Phase 1: Parse errors
    let module = match a16_parser::parse(source) {
        Ok(m) => m,
        Err(e) => {
            diagnostics.push(Diagnostic {
                range: Range { start_line: 0, start_col: 0, end_line: 0, end_col: 1 },
                severity: DiagnosticSeverity::Error,
                message: format!("{:?}", e),
                source: "a16-parser".to_string(),
                code: None,
            });
            return diagnostics;
        }
    };

    // Phase 2: Type errors
    let type_errors = a16_typeck::check(&module);
    for err in &type_errors {
        diagnostics.push(Diagnostic {
            range: Range { start_line: 0, start_col: 0, end_line: 0, end_col: 1 },
            severity: DiagnosticSeverity::Error,
            message: format!("{}", err),
            source: "a16-typeck".to_string(),
            code: None,
        });
    }

    // Phase 3: Lint warnings
    let lint_warnings = a16_lint::lint_module(&module);
    for warn in &lint_warnings {
        let severity = match warn.severity {
            a16_lint::LintSeverity::Warning => DiagnosticSeverity::Warning,
            a16_lint::LintSeverity::Info => DiagnosticSeverity::Information,
            a16_lint::LintSeverity::Hint => DiagnosticSeverity::Hint,
        };

        // Convert span to approximate line/col
        let (line, col) = offset_to_line_col(source, warn.span.start as usize);

        diagnostics.push(Diagnostic {
            range: Range {
                start_line: line,
                start_col: col,
                end_line: line,
                end_col: col + 1,
            },
            severity,
            message: warn.message.clone(),
            source: "a16-lint".to_string(),
            code: Some(warn.rule.code().to_string()),
        });
    }

    diagnostics
}

/// Convert a byte offset to (line, column), both 0-indexed
fn offset_to_line_col(source: &str, offset: usize) -> (u32, u32) {
    let mut line = 0u32;
    let mut col = 0u32;
    for (i, c) in source.char_indices() {
        if i >= offset {
            break;
        }
        if c == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    (line, col)
}
