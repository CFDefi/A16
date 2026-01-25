//! High-Level Intermediate Representation for A16
//!
//! The HIR is a desugared, typed representation of A16 programs.
//! It serves as the input to the bytecode compiler.

mod nodes;
mod lower;

pub use nodes::*;
pub use lower::lower_module;

#[cfg(test)]
mod tests;
