//! FFI Registry
//!
//! Central registry for loaded libraries and registered extern functions.
//! Supports both dynamic library functions (future) and built-in Rust extensions.

use smol_str::SmolStr;
use indexmap::IndexMap;
use thiserror::Error;

use crate::{FfiType, FfiValue, FfiError};

/// Errors from the FFI registry
#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Library '{0}' not found")]
    LibraryNotFound(String),

    #[error("Function '{0}' not found")]
    FunctionNotFound(String),

    #[error("Function '{0}' already registered")]
    AlreadyRegistered(String),

    #[error("Argument count mismatch: expected {expected}, got {got}")]
    ArgCountMismatch { expected: usize, got: usize },

    #[error("FFI error: {0}")]
    Ffi(#[from] FfiError),
}

/// A registered library entry
#[derive(Debug, Clone)]
pub struct LibEntry {
    /// Library name
    pub name: SmolStr,
    /// Optional path to the dynamic library file
    pub path: Option<String>,
    /// Whether the library is loaded
    pub loaded: bool,
}

/// Type alias for FFI callback functions
pub type FfiCallback = fn(&[FfiValue]) -> Result<FfiValue, FfiError>;

/// A registered function entry
#[derive(Debug, Clone)]
pub struct FuncEntry {
    /// Library this function belongs to
    pub lib_name: SmolStr,
    /// Function name
    pub name: SmolStr,
    /// Parameter types
    pub param_types: Vec<FfiType>,
    /// Return type
    pub return_type: FfiType,
    /// Optional native callback (for built-in extensions)
    pub callback: Option<FfiCallback>,
}

/// FFI Registry — central manager for foreign function declarations
#[derive(Debug)]
pub struct FfiRegistry {
    /// Registered libraries
    libraries: IndexMap<SmolStr, LibEntry>,
    /// Registered functions (full qualified: "lib::func")
    functions: IndexMap<SmolStr, FuncEntry>,
    /// Function index for bytecode references
    func_index: Vec<SmolStr>,
}

impl FfiRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            libraries: IndexMap::new(),
            functions: IndexMap::new(),
            func_index: Vec::new(),
        }
    }

    /// Register a library
    pub fn register_library(&mut self, name: &str, path: Option<&str>) -> Result<(), RegistryError> {
        let entry = LibEntry {
            name: SmolStr::new(name),
            path: path.map(|p| p.to_string()),
            loaded: path.is_none(), // Built-in libs are always "loaded"
        };
        self.libraries.insert(SmolStr::new(name), entry);
        Ok(())
    }

    /// Register a function signature (declaration only, no callback)
    pub fn register_function(
        &mut self,
        lib_name: &str,
        func_name: &str,
        param_types: Vec<FfiType>,
        return_type: FfiType,
    ) -> Result<u16, RegistryError> {
        let key = SmolStr::new(format!("{}::{}", lib_name, func_name));

        let entry = FuncEntry {
            lib_name: SmolStr::new(lib_name),
            name: SmolStr::new(func_name),
            param_types,
            return_type,
            callback: None,
        };

        self.functions.insert(key.clone(), entry);
        let idx = self.func_index.len() as u16;
        self.func_index.push(key);
        Ok(idx)
    }

    /// Register a function with a native callback (built-in extension)
    pub fn register_extension(
        &mut self,
        lib_name: &str,
        func_name: &str,
        param_types: Vec<FfiType>,
        return_type: FfiType,
        callback: FfiCallback,
    ) -> Result<u16, RegistryError> {
        let key = SmolStr::new(format!("{}::{}", lib_name, func_name));

        let entry = FuncEntry {
            lib_name: SmolStr::new(lib_name),
            name: SmolStr::new(func_name),
            param_types,
            return_type,
            callback: Some(callback),
        };

        self.functions.insert(key.clone(), entry);
        let idx = self.func_index.len() as u16;
        self.func_index.push(key);
        Ok(idx)
    }

    /// Look up a function by qualified name (lib::func)
    pub fn lookup(&self, lib_name: &str, func_name: &str) -> Option<&FuncEntry> {
        let key = SmolStr::new(format!("{}::{}", lib_name, func_name));
        self.functions.get(&key)
    }

    /// Look up a function by index (for bytecode)
    pub fn lookup_by_index(&self, idx: u16) -> Option<&FuncEntry> {
        self.func_index.get(idx as usize)
            .and_then(|key| self.functions.get(key))
    }

    /// Call a registered function by name
    pub fn call(
        &self,
        lib_name: &str,
        func_name: &str,
        args: &[FfiValue],
    ) -> Result<FfiValue, RegistryError> {
        let func = self.lookup(lib_name, func_name)
            .ok_or_else(|| RegistryError::FunctionNotFound(
                format!("{}::{}", lib_name, func_name)
            ))?;

        // Check argument count
        if args.len() != func.param_types.len() {
            return Err(RegistryError::ArgCountMismatch {
                expected: func.param_types.len(),
                got: args.len(),
            });
        }

        // Call callback if available
        match func.callback {
            Some(cb) => cb(args).map_err(RegistryError::Ffi),
            None => Err(RegistryError::FunctionNotFound(format!(
                "{}::{} has no implementation (dynamic loading not available)",
                lib_name, func_name
            ))),
        }
    }

    /// Call a function by index
    pub fn call_by_index(
        &self,
        idx: u16,
        args: &[FfiValue],
    ) -> Result<FfiValue, RegistryError> {
        let func = self.lookup_by_index(idx)
            .ok_or_else(|| RegistryError::FunctionNotFound(
                format!("ffi_func_{}", idx)
            ))?;

        if args.len() != func.param_types.len() {
            return Err(RegistryError::ArgCountMismatch {
                expected: func.param_types.len(),
                got: args.len(),
            });
        }

        match func.callback {
            Some(cb) => cb(args).map_err(RegistryError::Ffi),
            None => Err(RegistryError::FunctionNotFound(format!(
                "{}::{} has no implementation",
                func.lib_name, func.name
            ))),
        }
    }

    /// Get all function names in a library
    pub fn list_functions(&self, lib_name: &str) -> Vec<&str> {
        self.functions.values()
            .filter(|f| f.lib_name.as_str() == lib_name)
            .map(|f| f.name.as_str())
            .collect()
    }

    /// Get all registered library names
    pub fn list_libraries(&self) -> Vec<&str> {
        self.libraries.keys().map(|k| k.as_str()).collect()
    }

    /// Total number of registered functions
    pub fn function_count(&self) -> usize {
        self.functions.len()
    }

    /// Total number of registered libraries
    pub fn library_count(&self) -> usize {
        self.libraries.len()
    }

    /// Check if a library is registered
    pub fn has_library(&self, name: &str) -> bool {
        self.libraries.contains_key(name)
    }
}

impl Default for FfiRegistry {
    fn default() -> Self {
        Self::new()
    }
}
