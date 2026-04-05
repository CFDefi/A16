//! Package Manifest (a16.toml)
//!
//! Defines the project metadata, dependencies, and build configuration.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Semantic version
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemVer {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return None;
        }
        Some(Self {
            major: parts[0].parse().ok()?,
            minor: parts[1].parse().ok()?,
            patch: parts[2].parse().ok()?,
        })
    }

    /// Check if this version satisfies a requirement (simple caret range)
    pub fn satisfies(&self, req: &SemVer) -> bool {
        if req.major == 0 {
            self.major == req.major && self.minor == req.minor && self.patch >= req.patch
        } else {
            self.major == req.major && (self.minor > req.minor || (self.minor == req.minor && self.patch >= req.patch))
        }
    }
}

impl std::fmt::Display for SemVer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// A package dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// Version requirement
    pub version: String,
    /// Optional git source
    pub git: Option<String>,
    /// Optional path source (local)
    pub path: Option<String>,
    /// Whether this is a dev dependency
    #[serde(default)]
    pub dev: bool,
}

/// Package manifest (a16.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Package metadata
    pub package: PackageMeta,
    /// Dependencies
    #[serde(default)]
    pub dependencies: BTreeMap<String, Dependency>,
    /// Dev dependencies
    #[serde(default)]
    pub dev_dependencies: BTreeMap<String, Dependency>,
    /// Build configuration
    #[serde(default)]
    pub build: BuildConfig,
}

/// Package metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMeta {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub repository: String,
    #[serde(default = "default_entry")]
    pub entry: String,
}

fn default_entry() -> String {
    "src/main.a16".to_string()
}

/// Build configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    /// Optimization level: 0, 1, 2, 3
    #[serde(default = "default_opt")]
    pub opt_level: u8,
    /// Target output directory
    #[serde(default = "default_out")]
    pub out_dir: String,
}

fn default_opt() -> u8 { 1 }
fn default_out() -> String { "build".to_string() }

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            opt_level: default_opt(),
            out_dir: default_out(),
        }
    }
}

impl Manifest {
    /// Create a minimal manifest for a new project
    pub fn new(name: &str) -> Self {
        Self {
            package: PackageMeta {
                name: name.to_string(),
                version: "0.1.0".to_string(),
                description: String::new(),
                authors: Vec::new(),
                license: String::new(),
                repository: String::new(),
                entry: default_entry(),
            },
            dependencies: BTreeMap::new(),
            dev_dependencies: BTreeMap::new(),
            build: BuildConfig::default(),
        }
    }

    /// Load a manifest from a JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize the manifest to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Check if the manifest has any dependencies
    pub fn has_dependencies(&self) -> bool {
        !self.dependencies.is_empty()
    }

    /// Get all dependency names
    pub fn dependency_names(&self) -> Vec<&str> {
        self.dependencies.keys().map(|s| s.as_str()).collect()
    }
}
