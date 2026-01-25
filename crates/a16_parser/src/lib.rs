//! A16 Parser
//!
//! Recursive descent parser with Pratt expression parsing.

mod parser;
mod error;
mod expr;
mod stmt;
mod item;

pub use parser::Parser;
pub use error::{ParseError, ParseResult};

use a16_ast::Module;

/// Parse A16 source code into an AST
pub fn parse(source: &str) -> ParseResult<Module> {
    let mut parser = Parser::new(source);
    parser.parse_module()
}

#[cfg(test)]
mod tests;
