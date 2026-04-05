//! Lint Rules for A16
//!
//! Each rule checks a specific code quality concern.

use a16_ast::*;

/// Lint severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LintSeverity {
    Warning,
    Info,
    Hint,
}

/// A lint rule identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LintRule {
    /// W001: Empty function body (just pass)
    EmptyBody,
    /// W002: Naming convention violation
    NamingConvention,
    /// W003: Missing return type annotation
    MissingReturnType,
    /// W004: Agent missing model config
    AgentMissingModel,
    /// W005: Unreachable code after return
    UnreachableCode,
    /// W006: Unused import
    UnusedImport,
}

impl LintRule {
    /// Get the rule code (e.g., "W001")
    pub fn code(&self) -> &'static str {
        match self {
            LintRule::EmptyBody => "W001",
            LintRule::NamingConvention => "W002",
            LintRule::MissingReturnType => "W003",
            LintRule::AgentMissingModel => "W004",
            LintRule::UnreachableCode => "W005",
            LintRule::UnusedImport => "W006",
        }
    }
}

/// A lint warning
#[derive(Debug, Clone)]
pub struct LintWarning {
    pub rule: LintRule,
    pub message: String,
    pub span: Span,
    pub severity: LintSeverity,
}

impl std::fmt::Display for LintWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.rule.code(), self.message)
    }
}

/// Run all lint rules on a module
pub fn check_all(module: &Module, warnings: &mut Vec<LintWarning>) {
    for item in &module.items {
        check_item(item, warnings);
    }
}

fn check_item(item: &Item, warnings: &mut Vec<LintWarning>) {
    match item {
        Item::Function(func) => check_function(func, warnings),
        Item::Class(class) => check_class(class, warnings),
        Item::Agent(agent) => check_agent(agent, warnings),
        _ => {}
    }
}

/// W001: Empty function body
/// W002: Naming convention (functions should be snake_case)
/// W003: Missing return type
/// W005: Unreachable code after return
fn check_function(func: &FunctionDef, warnings: &mut Vec<LintWarning>) {
    // W001: Empty body (only a pass statement)
    if func.body.stmts.len() == 1 {
        if let Stmt::Pass(_) = &func.body.stmts[0] {
            warnings.push(LintWarning {
                rule: LintRule::EmptyBody,
                message: format!("Function '{}' has an empty body", func.name.name),
                span: func.name.span,
                severity: LintSeverity::Warning,
            });
        }
    }

    // W002: Naming convention — functions should be snake_case
    let name = func.name.name.as_str();
    if !name.starts_with("__") && name.chars().any(|c| c.is_uppercase()) {
        warnings.push(LintWarning {
            rule: LintRule::NamingConvention,
            message: format!("Function '{}' should use snake_case naming", name),
            span: func.name.span,
            severity: LintSeverity::Info,
        });
    }

    // W003: Missing return type
    if func.return_type.is_none() && func.name.name.as_str() != "main" {
        warnings.push(LintWarning {
            rule: LintRule::MissingReturnType,
            message: format!("Function '{}' is missing a return type annotation", func.name.name),
            span: func.name.span,
            severity: LintSeverity::Hint,
        });
    }

    // W005: Unreachable code after return
    check_unreachable_code(&func.body.stmts, warnings);
}

/// W002: Class naming convention (should be PascalCase)
fn check_class(class: &ClassDef, warnings: &mut Vec<LintWarning>) {
    let name = class.name.name.as_str();
    if !name.is_empty() {
        let first = name.chars().next().unwrap();
        if first.is_lowercase() {
            warnings.push(LintWarning {
                rule: LintRule::NamingConvention,
                message: format!("Class '{}' should use PascalCase naming", name),
                span: class.name.span,
                severity: LintSeverity::Info,
            });
        }
    }

    // Check methods
    for member in &class.body {
        if let ClassMember::Method(method) = member {
            check_function(method, warnings);
        }
    }
}

/// W004: Agent missing model
fn check_agent(agent: &AgentDef, warnings: &mut Vec<LintWarning>) {
    let has_model = agent.config.iter().any(|c| matches!(c, AgentConfig::Model(_)))
        || agent.members.iter().any(|m| matches!(m, AgentMember::Config(AgentConfig::Model(_))));

    if !has_model {
        warnings.push(LintWarning {
            rule: LintRule::AgentMissingModel,
            message: format!("Agent '{}' is missing a 'model' configuration", agent.name.name),
            span: agent.name.span,
            severity: LintSeverity::Warning,
        });
    }
}

/// W005: Check for statements after unconditional return
fn check_unreachable_code(stmts: &[Stmt], warnings: &mut Vec<LintWarning>) {
    let mut found_return = false;
    for stmt in stmts {
        if found_return {
            let span = match stmt {
                Stmt::Return(r) => r.span,
                Stmt::Let(l) => l.span,
                Stmt::Expr(e) => e.span,
                Stmt::If(i) => i.span,
                Stmt::For(f) => f.span,
                Stmt::While(w) => w.span,
                Stmt::Break(s) => *s,
                Stmt::Continue(s) => *s,
                Stmt::Pass(s) => *s,
                _ => Span::default(),
            };
            warnings.push(LintWarning {
                rule: LintRule::UnreachableCode,
                message: "Unreachable code after return statement".to_string(),
                span,
                severity: LintSeverity::Warning,
            });
            break;
        }
        if matches!(stmt, Stmt::Return(_)) {
            found_return = true;
        }
    }
}
