//! Parser error types

use a16_ast::Span;
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

pub type ParseResult<T> = Result<T, ParseError>;

/// Convert an A16 Span to a miette SourceSpan
fn to_source_span(span: Span) -> SourceSpan {
    SourceSpan::new(
        (span.start as usize).into(),
        (span.end.saturating_sub(span.start)) as usize,
    )
}

#[derive(Error, Debug, Clone)]
pub enum ParseError {
    #[error("Unexpected token: expected {expected}, found {found}")]
    UnexpectedToken {
        expected: String,
        found: String,
        span: Span,
    },
    
    #[error("Unexpected end of file")]
    UnexpectedEof {
        span: Span,
    },
    
    #[error("Invalid syntax: {message}")]
    InvalidSyntax {
        message: String,
        span: Span,
    },
    
    #[error("Invalid indentation")]
    InvalidIndentation {
        span: Span,
    },
    
    #[error("Expected expression")]
    ExpectedExpression {
        span: Span,
    },
    
    #[error("Expected statement")]
    ExpectedStatement {
        span: Span,
    },
    
    #[error("Expected identifier")]
    ExpectedIdentifier {
        span: Span,
    },
    
    #[error("Expected type")]
    ExpectedType {
        span: Span,
    },
    
    #[error("Invalid number literal: {message}")]
    InvalidNumber {
        message: String,
        span: Span,
    },
    
    #[error("Unterminated string")]
    UnterminatedString {
        span: Span,
    },
}

impl Diagnostic for ParseError {
    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        let (label, span) = match self {
            ParseError::UnexpectedToken { span, .. } => ("here", *span),
            ParseError::UnexpectedEof { span } => ("expected more input", *span),
            ParseError::InvalidSyntax { message, span } => (message.as_str(), *span),
            ParseError::InvalidIndentation { span } => ("unexpected indentation", *span),
            ParseError::ExpectedExpression { span } => ("expected expression here", *span),
            ParseError::ExpectedStatement { span } => ("expected statement here", *span),
            ParseError::ExpectedIdentifier { span } => ("expected identifier", *span),
            ParseError::ExpectedType { span } => ("expected type", *span),
            ParseError::InvalidNumber { span, .. } => ("invalid number", *span),
            ParseError::UnterminatedString { span } => ("string not closed", *span),
        };
        Some(Box::new(std::iter::once(
            miette::LabeledSpan::new_with_span(Some(label.to_string()), to_source_span(span))
        )))
    }
}

impl ParseError {
    pub fn span(&self) -> Span {
        match self {
            ParseError::UnexpectedToken { span, .. } => *span,
            ParseError::UnexpectedEof { span } => *span,
            ParseError::InvalidSyntax { span, .. } => *span,
            ParseError::InvalidIndentation { span } => *span,
            ParseError::ExpectedExpression { span } => *span,
            ParseError::ExpectedStatement { span } => *span,
            ParseError::ExpectedIdentifier { span } => *span,
            ParseError::ExpectedType { span } => *span,
            ParseError::InvalidNumber { span, .. } => *span,
            ParseError::UnterminatedString { span } => *span,
        }
    }
}
