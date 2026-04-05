//! A16 Package Manager
//!
//! Provides manifest parsing, lockfile management, and dependency resolution
//! for A16 projects.

mod manifest;
mod lockfile;

pub use manifest::{Manifest, Dependency, SemVer};
pub use lockfile::{Lockfile, LockedDependency};

#[cfg(test)]
mod tests;
