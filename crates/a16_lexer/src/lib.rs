//! A16 Lexer
//! 
//! Tokenizes A16 source code into a stream of tokens.
//! Handles indentation-based blocks (like Python).

mod token;
mod lexer;
mod indent;

pub use token::{Token, TokenKind, Span};
pub use lexer::Lexer;

#[cfg(test)]
mod tests;
