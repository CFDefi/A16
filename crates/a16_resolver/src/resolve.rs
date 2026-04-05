//! Module Path Resolution
//!
//! Resolves import statements to actual source file paths.

use smol_str::SmolStr;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Resolved module path
#[derive(Debug, Clone)]
pub struct ModulePath {
    /// The fully qualified module name
    pub name: SmolStr,
    /// The resolved file system path
    pub path: PathBuf,
    /// Whether this is a stdlib module
    pub is_stdlib: bool,
}

/// Module resolver
pub struct Resolver {
    /// Root directory of the project
    project_root: PathBuf,
    /// Standard library path
    stdlib_path: Option<PathBuf>,
    /// Additional search paths
    search_paths: Vec<PathBuf>,
}

impl Resolver {
    /// Create a new resolver for a project
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            project_root,
            stdlib_path: None,
            search_paths: Vec::new(),
        }
    }

    /// Set the standard library path
    pub fn with_stdlib(mut self, path: PathBuf) -> Self {
        self.stdlib_path = Some(path);
        self
    }

    /// Add a search path
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    /// Resolve a module import to a file path
    ///
    /// Resolution order:
    /// 1. Relative to the importing file (for relative imports)
    /// 2. Project root
    /// 3. Search paths
    /// 4. Standard library
    pub fn resolve(
        &self,
        module_name: &str,
        from_file: Option<&Path>,
    ) -> Result<ModulePath, ResolveError> {
        let parts: Vec<&str> = module_name.split('.').collect();

        // Check for relative imports
        if module_name.starts_with('.') {
            if let Some(from) = from_file {
                return self.resolve_relative(&parts, from);
            }
            return Err(ResolveError::RelativeImportNoContext {
                module: module_name.to_string(),
            });
        }

        // Check project root
        if let Some(path) = self.find_module(&parts, &self.project_root) {
            return Ok(ModulePath {
                name: SmolStr::new(module_name),
                path,
                is_stdlib: false,
            });
        }

        // Check search paths
        for search_path in &self.search_paths {
            if let Some(path) = self.find_module(&parts, search_path) {
                return Ok(ModulePath {
                    name: SmolStr::new(module_name),
                    path,
                    is_stdlib: false,
                });
            }
        }

        // Check stdlib
        if let Some(ref stdlib) = self.stdlib_path {
            if let Some(path) = self.find_module(&parts, stdlib) {
                return Ok(ModulePath {
                    name: SmolStr::new(module_name),
                    path,
                    is_stdlib: true,
                });
            }
        }

        // Check if it's a built-in std module name
        if parts.first() == Some(&"std") {
            return Ok(ModulePath {
                name: SmolStr::new(module_name),
                path: PathBuf::from(format!("<stdlib>/{}.a16", parts[1..].join("/"))),
                is_stdlib: true,
            });
        }

        Err(ResolveError::ModuleNotFound {
            module: module_name.to_string(),
        })
    }

    /// Resolve a relative import
    fn resolve_relative(&self, parts: &[&str], from_file: &Path) -> Result<ModulePath, ResolveError> {
        let parent = from_file.parent().unwrap_or(Path::new("."));

        // Count leading dots for relative depth
        let mut depth = 0;
        for part in parts {
            if *part == "" {
                depth += 1;
            } else {
                break;
            }
        }

        let actual_parts = &parts[depth..];
        let mut base = parent.to_path_buf();
        for _ in 1..depth {
            base = base.parent().unwrap_or(Path::new(".")).to_path_buf();
        }

        if let Some(path) = self.find_module(actual_parts, &base) {
            let name = actual_parts.join(".");
            return Ok(ModulePath {
                name: SmolStr::new(name),
                path,
                is_stdlib: false,
            });
        }

        Err(ResolveError::ModuleNotFound {
            module: parts.join("."),
        })
    }

    /// Find a module file in a directory
    fn find_module(&self, parts: &[&str], base: &Path) -> Option<PathBuf> {
        if parts.is_empty() {
            return None;
        }

        // Try as a file: base/part1/part2/...partN.a16
        let mut file_path = base.to_path_buf();
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                file_path.push(format!("{}.a16", part));
            } else {
                file_path.push(part);
            }
        }

        if file_path.exists() {
            return Some(file_path);
        }

        // Try as a package: base/part1/part2/...partN/__init__.a16
        let mut pkg_path = base.to_path_buf();
        for part in parts {
            pkg_path.push(part);
        }
        pkg_path.push("__init__.a16");

        if pkg_path.exists() {
            return Some(pkg_path);
        }

        None
    }

    /// Get the project root
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }
}

/// Module resolution errors
#[derive(Debug, Error)]
pub enum ResolveError {
    #[error("Module not found: `{module}`")]
    ModuleNotFound { module: String },

    #[error("Relative import requires a source context: `{module}`")]
    RelativeImportNoContext { module: String },

    #[error("Circular dependency: {cycle}")]
    CircularDependency { cycle: String },
}
