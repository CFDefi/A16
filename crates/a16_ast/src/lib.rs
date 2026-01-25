//! A16 Abstract Syntax Tree
//!
//! Defines all AST node types for the A16 language.

mod nodes;
mod visitor;

pub use nodes::*;
pub use visitor::Visitor;

use smol_str::SmolStr;

/// Source location
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }
    
    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

/// Identifier with span
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident {
    pub name: SmolStr,
    pub span: Span,
}

impl Ident {
    pub fn new(name: impl Into<SmolStr>, span: Span) -> Self {
        Self { name: name.into(), span }
    }
}

/// A complete A16 module (source file)
#[derive(Debug, Clone)]
pub struct Module {
    pub items: Vec<Item>,
    pub span: Span,
}
