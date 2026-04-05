//! A16 Module Resolver
//!
//! Resolves import paths to source files and builds a dependency graph.
//! Supports:
//! - Relative imports: `from .utils import helper`
//! - Package imports: `import math`
//! - Standard library resolution: `from std.io import read_file`

mod graph;
mod resolve;

pub use graph::{ModuleGraph, ModuleNode, ModuleId};
pub use resolve::{Resolver, ResolveError, ModulePath};

#[cfg(test)]
mod tests;
