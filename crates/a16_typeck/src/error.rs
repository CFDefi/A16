//! Type error definitions
//! 
//! Struct fields are used in error message templates via thiserror/miette derives.
#![allow(unused_assignments)]

use a16_ast::Span;
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

/// Convert an A16 Span to a miette SourceSpan
fn to_source_span(span: Span) -> SourceSpan {
    SourceSpan::new(
        (span.start as usize).into(),
        (span.end.saturating_sub(span.start)) as usize,
    )
}

/// Type errors that can occur during type checking
#[derive(Debug, Clone, Error, Diagnostic)]
pub enum TypeError {
    /// Type mismatch between expected and actual types
    #[error("Type mismatch: expected `{expected}`, found `{found}`")]
    #[diagnostic(code(typeck::mismatch))]
    TypeMismatch {
        expected: String,
        found: String,
        #[label("expected `{expected}` here")]
        span: SourceSpan,
        #[help]
        help: Option<String>,
    },
    
    /// Undefined variable
    #[error("Undefined variable `{name}`")]
    #[diagnostic(code(typeck::undefined_var))]
    UndefinedVariable {
        name: String,
        #[label("not found in this scope")]
        span: SourceSpan,
        #[help]
        suggestion: Option<String>,
    },
    
    /// Undefined type
    #[error("Undefined type `{name}`")]
    #[diagnostic(code(typeck::undefined_type))]
    UndefinedType {
        name: String,
        #[label("type not defined")]
        span: SourceSpan,
    },
    
    /// Type is not callable
    #[error("Type `{ty}` is not callable")]
    #[diagnostic(code(typeck::not_callable))]
    NotCallable {
        ty: String,
        #[label("cannot call this type")]
        span: SourceSpan,
    },
    
    /// Wrong number of arguments
    #[error("Wrong number of arguments: expected {expected}, found {found}")]
    #[diagnostic(code(typeck::wrong_args))]
    WrongArgs {
        expected: usize,
        found: usize,
        #[label("expected {expected} argument(s)")]
        span: SourceSpan,
    },
    
    /// Type is not iterable
    #[error("Type `{ty}` is not iterable")]
    #[diagnostic(code(typeck::not_iterable))]
    NotIterable {
        ty: String,
        #[label("cannot iterate over this type")]
        span: SourceSpan,
    },
    
    /// Type is not indexable
    #[error("Type `{ty}` cannot be indexed with `{index_ty}`")]
    #[diagnostic(code(typeck::not_indexable))]
    NotIndexable {
        ty: String,
        index_ty: String,
        #[label("cannot index this type")]
        span: SourceSpan,
    },
    
    /// No member on type
    #[error("Type `{ty}` has no member `{member}`")]
    #[diagnostic(code(typeck::no_member))]
    NoMember {
        ty: String,
        member: String,
        #[label("member not found")]
        span: SourceSpan,
    },
    
    /// Invalid binary operation
    #[error("Operator `{op}` cannot be applied to types `{lhs}` and `{rhs}`")]
    #[diagnostic(code(typeck::invalid_binop))]
    InvalidBinOp {
        op: String,
        lhs: String,
        rhs: String,
        #[label("invalid operation")]
        span: SourceSpan,
    },
    
    /// Invalid unary operation
    #[error("Operator `{op}` cannot be applied to type `{operand}`")]
    #[diagnostic(code(typeck::invalid_unop))]
    InvalidUnOp {
        op: String,
        operand: String,
        #[label("invalid operation")]
        span: SourceSpan,
    },
    
    /// Assignment to immutable binding
    #[error("Cannot assign to immutable variable `{name}`")]
    #[diagnostic(code(typeck::immutable_assign))]
    ImmutableAssign {
        name: String,
        #[label("variable defined here")]
        def_span: SourceSpan,
        #[label("cannot assign here")]
        assign_span: SourceSpan,
    },
    
    /// Missing return statement
    #[error("Function `{name}` is missing a return statement")]
    #[diagnostic(code(typeck::missing_return), help("add a return statement or change return type to None"))]
    MissingReturn {
        name: String,
        #[label("function defined here")]
        span: SourceSpan,
    },
    
    /// Missing required field in struct initialization
    #[error("Missing required field `{field}` in `{struct_name}` initialization")]
    #[diagnostic(code(typeck::missing_field))]
    MissingField {
        struct_name: String,
        field: String,
        #[label("missing field `{field}`")]
        span: SourceSpan,
    },
    
    /// Duplicate definition
    #[error("Duplicate definition of `{name}`")]
    #[diagnostic(code(typeck::duplicate_def))]
    DuplicateDefinition {
        name: String,
        #[label("first defined here")]
        first_span: SourceSpan,
        #[label("redefined here")]
        second_span: SourceSpan,
    },
    
    /// Agent missing required clause
    #[error("Agent `{name}` is missing required `{missing}` clause")]
    #[diagnostic(code(typeck::agent_missing))]
    AgentMissingClause {
        name: String,
        missing: String,
        #[label("agent defined here")]
        span: SourceSpan,
    },
    
    /// Tool missing execute function
    #[error("Tool `{name}` must have an `execute` function")]
    #[diagnostic(code(typeck::tool_no_execute))]
    ToolNoExecute {
        name: String,
        #[label("tool defined here")]
        span: SourceSpan,
    },
    
    /// Async operation outside async context
    #[error("Cannot use `await` outside of an async function")]
    #[diagnostic(code(typeck::await_outside))]
    AwaitOutsideAsync {
        #[label("await used here")]
        span: SourceSpan,
    },
    
    /// Recursive type detected
    #[error("Recursive type detected")]
    #[diagnostic(code(typeck::recursive_type))]
    RecursiveType {
        #[label("type references itself")]
        span: SourceSpan,
    },
    
    /// Type unification failed
    #[error("Cannot unify types `{t1}` and `{t2}`")]
    #[diagnostic(code(typeck::unify_failed))]
    UnificationFailed {
        t1: String,
        t2: String,
        #[label("types are incompatible")]
        span: SourceSpan,
    },
    
    // M9: Pattern exhaustiveness
    
    /// Non-exhaustive match patterns
    #[error("Non-exhaustive match: missing patterns: {patterns}")]
    #[diagnostic(code(typeck::non_exhaustive), help("add a wildcard `_` arm or cover all cases"))]
    NonExhaustiveMatch {
        patterns: String,
        #[label("match expression here")]
        span: SourceSpan,
    },
    
    /// Unreachable match arm
    #[error("Unreachable match arm: previous patterns already cover this case")]
    #[diagnostic(code(typeck::unreachable_arm))]
    UnreachableArm {
        #[label("this arm is unreachable")]
        span: SourceSpan,
    },
    
    /// Custom error (for extensibility)
    #[error("{message}")]
    #[diagnostic(code(typeck::custom))]
    Custom {
        message: String,
        #[label("here")]
        span: SourceSpan,
    },
}

impl TypeError {
    /// Create a type mismatch error
    pub fn type_mismatch(expected: &str, found: &str, span: Span) -> Self {
        TypeError::TypeMismatch {
            expected: expected.to_string(),
            found: found.to_string(),
            span: to_source_span(span),
            help: None,
        }
    }
    
    /// Create an undefined variable error
    pub fn undefined_var(name: &str, span: Span) -> Self {
        TypeError::UndefinedVariable {
            name: name.to_string(),
            span: to_source_span(span),
            suggestion: None,
        }
    }
    
    /// Create a not callable error
    pub fn not_callable(ty: &str, span: Span) -> Self {
        TypeError::NotCallable {
            ty: ty.to_string(),
            span: to_source_span(span),
        }
    }
    
    /// Create a wrong args error
    pub fn wrong_args(expected: usize, found: usize, span: Span) -> Self {
        TypeError::WrongArgs {
            expected,
            found,
            span: to_source_span(span),
        }
    }
    
    /// Create a not iterable error
    pub fn not_iterable(ty: &str, span: Span) -> Self {
        TypeError::NotIterable {
            ty: ty.to_string(),
            span: to_source_span(span),
        }
    }
    
    /// Create a not indexable error
    pub fn not_indexable(ty: &str, index_ty: &str, span: Span) -> Self {
        TypeError::NotIndexable {
            ty: ty.to_string(),
            index_ty: index_ty.to_string(),
            span: to_source_span(span),
        }
    }
    
    /// Create a no member error
    pub fn no_member(ty: &str, member: &str, span: Span) -> Self {
        TypeError::NoMember {
            ty: ty.to_string(),
            member: member.to_string(),
            span: to_source_span(span),
        }
    }
    
    /// Create an invalid binary operator error
    pub fn invalid_binop(op: &str, lhs: &str, rhs: &str, span: Span) -> Self {
        TypeError::InvalidBinOp {
            op: op.to_string(),
            lhs: lhs.to_string(),
            rhs: rhs.to_string(),
            span: to_source_span(span),
        }
    }
    
    /// Create an invalid unary operator error
    pub fn invalid_unop(op: &str, operand: &str, span: Span) -> Self {
        TypeError::InvalidUnOp {
            op: op.to_string(),
            operand: operand.to_string(),
            span: to_source_span(span),
        }
    }
    
    /// Create a unification failed error
    pub fn unification_failed(t1: &str, t2: &str, span: Span) -> Self {
        TypeError::UnificationFailed {
            t1: t1.to_string(),
            t2: t2.to_string(),
            span: to_source_span(span),
        }
    }
    
    /// Create a custom error
    pub fn custom(message: String, span: Span) -> Self {
        TypeError::Custom {
            message,
            span: to_source_span(span),
        }
    }
    
    /// Create a non-exhaustive match error
    pub fn non_exhaustive_match(patterns: &[String], span: Span) -> Self {
        TypeError::NonExhaustiveMatch {
            patterns: patterns.join(", "),
            span: to_source_span(span),
        }
    }
    
    /// Create an unreachable arm warning
    pub fn unreachable_arm(span: Span) -> Self {
        TypeError::UnreachableArm {
            span: to_source_span(span),
        }
    }
}
