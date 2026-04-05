//! Dynamic Library Loading (stubs + extension API)
//!
//! Provides the DynLib, FfiFunc, and FfiError types.
//! Actual dynamic loading via libloading is deferred;
//! the primary mechanism is the registered extensions model in registry.rs.

use smol_str::SmolStr;
use thiserror::Error;

use crate::marshal::{FfiType, FfiValue};

/// Errors from FFI operations
#[derive(Debug, Error, Clone)]
pub enum FfiError {
    #[error("Library not found: {0}")]
    LibraryNotFound(String),

    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),

    #[error("Type mismatch: expected {expected}, got {got}")]
    TypeMismatch {
        expected: String,
        got: String,
    },

    #[error("Call failed: {0}")]
    CallFailed(String),

    #[error("Marshaling error: {0}")]
    MarshalError(String),
}

/// Represents a loaded dynamic library (stub implementation)
#[derive(Debug, Clone)]
pub struct DynLib {
    /// Library name
    pub name: SmolStr,
    /// Optional path to the library file
    pub path: Option<String>,
    /// Whether the library is loaded
    pub loaded: bool,
}

impl DynLib {
    /// Create a new library reference (stub: does not actually load)
    pub fn new(name: &str, path: Option<&str>) -> Self {
        Self {
            name: SmolStr::new(name),
            path: path.map(|p| p.to_string()),
            loaded: false,
        }
    }

    /// Mark the library as loaded (for built-in extensions)
    pub fn mark_loaded(&mut self) {
        self.loaded = true;
    }

    /// Check if the library is loaded
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }
}

/// Represents a foreign function declaration
#[derive(Debug, Clone)]
pub struct FfiFunc {
    /// Function name
    pub name: SmolStr,
    /// Library this function belongs to
    pub lib_name: SmolStr,
    /// Parameter types
    pub param_types: Vec<FfiType>,
    /// Return type
    pub return_type: FfiType,
}

impl FfiFunc {
    /// Create a new FFI function declaration
    pub fn new(
        name: &str,
        lib_name: &str,
        param_types: Vec<FfiType>,
        return_type: FfiType,
    ) -> Self {
        Self {
            name: SmolStr::new(name),
            lib_name: SmolStr::new(lib_name),
            param_types,
            return_type,
        }
    }

    /// Validate argument count
    pub fn validate_args(&self, args: &[FfiValue]) -> Result<(), FfiError> {
        if args.len() != self.param_types.len() {
            return Err(FfiError::CallFailed(format!(
                "{}::{} expects {} arguments, got {}",
                self.lib_name, self.name, self.param_types.len(), args.len()
            )));
        }
        Ok(())
    }
}
