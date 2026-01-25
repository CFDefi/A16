//! Bytecode Compiler for A16
//!
//! Compiles HIR to stack-based bytecode for the VM.

mod bytecode;
mod compiler;

pub use bytecode::*;
pub use compiler::compile;

#[cfg(test)]
mod tests;
