//! Lockfile Management
//!
//! The lockfile ensures deterministic builds by pinning exact versions
//! of all dependencies.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A lockfile pinning exact dependency versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lockfile {
    /// Version of the lockfile format
    pub version: u32,
    /// Locked dependency entries
    pub packages: BTreeMap<String, LockedDependency>,
}

/// A locked (pinned) dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedDependency {
    /// Exact version
    pub version: String,
    /// Source (registry, git, or path)
    pub source: String,
    /// Integrity hash for verification
    pub integrity: Option<String>,
    /// Transitive dependencies
    #[serde(default)]
    pub dependencies: Vec<String>,
}

impl Lockfile {
    /// Create a new empty lockfile
    pub fn new() -> Self {
        Self {
            version: 1,
            packages: BTreeMap::new(),
        }
    }

    /// Add a locked dependency
    pub fn lock(&mut self, name: String, dep: LockedDependency) {
        self.packages.insert(name, dep);
    }

    /// Check if a package is locked
    pub fn is_locked(&self, name: &str) -> bool {
        self.packages.contains_key(name)
    }

    /// Get a locked dependency
    pub fn get(&self, name: &str) -> Option<&LockedDependency> {
        self.packages.get(name)
    }

    /// Number of locked packages
    pub fn len(&self) -> usize {
        self.packages.len()
    }

    /// Whether the lockfile is empty
    pub fn is_empty(&self) -> bool {
        self.packages.is_empty()
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Load from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl Default for Lockfile {
    fn default() -> Self {
        Self::new()
    }
}
