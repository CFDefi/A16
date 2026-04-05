//! A16 Extended Standard Library
//!
//! Provides modular standard library implementations:
//! - math: Mathematical functions and constants
//! - string: String manipulation utilities
//! - collections: Advanced collection operations
//! - json: JSON serialization/deserialization
//! - io: File I/O operations
//! - os: Operating system utilities

pub mod math;
pub mod string;
pub mod collections;
pub mod json;
pub mod io;
pub mod os;

#[cfg(test)]
mod tests;
