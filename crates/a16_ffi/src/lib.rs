//! A16 Foreign Function Interface
//!
//! Provides dynamic library loading, type marshaling between A16 and native types,
//! an FFI registry for managing extern function declarations, and a built-in
//! extension API for registering Rust callbacks.

mod dynlib;
mod marshal;
mod registry;

pub use dynlib::{DynLib, FfiFunc, FfiError};
pub use marshal::{FfiType, FfiValue, marshal_to_ffi, marshal_from_ffi, marshal_args};
pub use registry::{FfiRegistry, FfiCallback, FuncEntry, LibEntry, RegistryError};

#[cfg(test)]
mod tests;
