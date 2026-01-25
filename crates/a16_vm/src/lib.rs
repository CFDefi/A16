//! A16 Virtual Machine
//!
//! Stack-based bytecode interpreter for A16 programs.

mod value;
mod vm;
mod stdlib;

pub use value::Value;
pub use vm::VM;
pub use stdlib::register_stdlib;

#[cfg(test)]
mod tests;
