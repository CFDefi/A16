//! Type Marshaling between A16 and Native Types
//!
//! Provides conversion between A16 runtime Values (via FfiValue)
//! and C-compatible types for FFI calls.

use smol_str::SmolStr;

/// C-compatible type representation
#[derive(Debug, Clone, PartialEq)]
pub enum FfiType {
    Void,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float32,
    Float64,
    Pointer,
    String,
    Bool,
}

impl FfiType {
    /// Parse an FFI type from a type name string
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "Void" | "void" | "None" => Some(FfiType::Void),
            "Int" | "int" | "i64" | "Int64" => Some(FfiType::Int64),
            "Int32" | "i32" => Some(FfiType::Int32),
            "Int16" | "i16" => Some(FfiType::Int16),
            "Int8" | "i8" => Some(FfiType::Int8),
            "UInt64" | "u64" => Some(FfiType::UInt64),
            "UInt32" | "u32" => Some(FfiType::UInt32),
            "UInt16" | "u16" => Some(FfiType::UInt16),
            "UInt8" | "u8" | "Byte" => Some(FfiType::UInt8),
            "Float" | "float" | "f64" | "Float64" => Some(FfiType::Float64),
            "Float32" | "f32" => Some(FfiType::Float32),
            "Ptr" | "ptr" | "Pointer" => Some(FfiType::Pointer),
            "Str" | "str" | "String" => Some(FfiType::String),
            "Bool" | "bool" => Some(FfiType::Bool),
            _ => None,
        }
    }

    /// Size in bytes of this type
    pub fn size(&self) -> usize {
        match self {
            FfiType::Void => 0,
            FfiType::Int8 | FfiType::UInt8 | FfiType::Bool => 1,
            FfiType::Int16 | FfiType::UInt16 => 2,
            FfiType::Int32 | FfiType::UInt32 | FfiType::Float32 => 4,
            FfiType::Int64 | FfiType::UInt64 | FfiType::Float64 | FfiType::Pointer | FfiType::String => 8,
        }
    }
}

impl std::fmt::Display for FfiType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FfiType::Void => write!(f, "Void"),
            FfiType::Int8 => write!(f, "Int8"),
            FfiType::Int16 => write!(f, "Int16"),
            FfiType::Int32 => write!(f, "Int32"),
            FfiType::Int64 => write!(f, "Int64"),
            FfiType::UInt8 => write!(f, "UInt8"),
            FfiType::UInt16 => write!(f, "UInt16"),
            FfiType::UInt32 => write!(f, "UInt32"),
            FfiType::UInt64 => write!(f, "UInt64"),
            FfiType::Float32 => write!(f, "Float32"),
            FfiType::Float64 => write!(f, "Float64"),
            FfiType::Pointer => write!(f, "Pointer"),
            FfiType::String => write!(f, "String"),
            FfiType::Bool => write!(f, "Bool"),
        }
    }
}

/// An FFI value for marshaling between A16 and native code
#[derive(Debug, Clone)]
pub enum FfiValue {
    Void,
    Int(i64),
    UInt(u64),
    Float(f64),
    Str(SmolStr),
    Bool(bool),
    Pointer(usize),
}

impl FfiValue {
    /// Get the FFI type of this value
    pub fn ffi_type(&self) -> FfiType {
        match self {
            FfiValue::Void => FfiType::Void,
            FfiValue::Int(_) => FfiType::Int64,
            FfiValue::UInt(_) => FfiType::UInt64,
            FfiValue::Float(_) => FfiType::Float64,
            FfiValue::Str(_) => FfiType::String,
            FfiValue::Bool(_) => FfiType::Bool,
            FfiValue::Pointer(_) => FfiType::Pointer,
        }
    }

    /// Convert to i64 (for integral types)
    pub fn as_int(&self) -> Option<i64> {
        match self {
            FfiValue::Int(n) => Some(*n),
            FfiValue::UInt(n) => Some(*n as i64),
            FfiValue::Bool(b) => Some(if *b { 1 } else { 0 }),
            FfiValue::Float(f) => Some(*f as i64),
            _ => None,
        }
    }

    /// Convert to f64 (for floating point types)
    pub fn as_float(&self) -> Option<f64> {
        match self {
            FfiValue::Float(f) => Some(*f),
            FfiValue::Int(n) => Some(*n as f64),
            FfiValue::UInt(n) => Some(*n as f64),
            _ => None,
        }
    }

    /// Convert to string
    pub fn as_str(&self) -> Option<&str> {
        match self {
            FfiValue::Str(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

impl std::fmt::Display for FfiValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FfiValue::Void => write!(f, "void"),
            FfiValue::Int(n) => write!(f, "{}", n),
            FfiValue::UInt(n) => write!(f, "{}", n),
            FfiValue::Float(n) => write!(f, "{}", n),
            FfiValue::Str(s) => write!(f, "{}", s),
            FfiValue::Bool(b) => write!(f, "{}", b),
            FfiValue::Pointer(p) => write!(f, "0x{:x}", p),
        }
    }
}

/// Marshal an FfiValue to a target FfiType
pub fn marshal_to_ffi(value: &FfiValue, target: &FfiType) -> Result<FfiValue, String> {
    match (value, target) {
        // Identity
        (FfiValue::Void, FfiType::Void) => Ok(FfiValue::Void),
        (FfiValue::Int(n), FfiType::Int64) => Ok(FfiValue::Int(*n)),
        (FfiValue::Int(n), FfiType::Int32) => Ok(FfiValue::Int(*n as i32 as i64)),
        (FfiValue::Int(n), FfiType::Int16) => Ok(FfiValue::Int(*n as i16 as i64)),
        (FfiValue::Int(n), FfiType::Int8) => Ok(FfiValue::Int(*n as i8 as i64)),
        (FfiValue::UInt(n), FfiType::UInt64) => Ok(FfiValue::UInt(*n)),
        (FfiValue::UInt(n), FfiType::UInt32) => Ok(FfiValue::UInt(*n as u32 as u64)),
        (FfiValue::Float(f), FfiType::Float64) => Ok(FfiValue::Float(*f)),
        (FfiValue::Float(f), FfiType::Float32) => Ok(FfiValue::Float(*f as f32 as f64)),
        (FfiValue::Str(s), FfiType::String) => Ok(FfiValue::Str(s.clone())),
        (FfiValue::Bool(b), FfiType::Bool) => Ok(FfiValue::Bool(*b)),
        (FfiValue::Pointer(p), FfiType::Pointer) => Ok(FfiValue::Pointer(*p)),

        // Conversions
        (FfiValue::Int(n), FfiType::Float64) => Ok(FfiValue::Float(*n as f64)),
        (FfiValue::Int(n), FfiType::Float32) => Ok(FfiValue::Float(*n as f32 as f64)),
        (FfiValue::Float(f), FfiType::Int64) => Ok(FfiValue::Int(*f as i64)),
        (FfiValue::Bool(b), FfiType::Int64) => Ok(FfiValue::Int(if *b { 1 } else { 0 })),
        (FfiValue::Int(n), FfiType::Bool) => Ok(FfiValue::Bool(*n != 0)),
        (FfiValue::Int(n), FfiType::UInt64) => Ok(FfiValue::UInt(*n as u64)),
        (FfiValue::UInt(n), FfiType::Int64) => Ok(FfiValue::Int(*n as i64)),

        // Error
        _ => Err(format!(
            "Cannot marshal {:?} to {}",
            value.ffi_type(), target
        )),
    }
}

/// Marshal an FfiValue from a source FfiType (back to A16-compatible)
pub fn marshal_from_ffi(value: &FfiValue, source: &FfiType) -> Result<FfiValue, String> {
    // For now, marshal_from_ffi is the same as marshal_to_ffi
    // In a real implementation, this would handle pointer cleanup, string copying, etc.
    marshal_to_ffi(value, source)
}

/// Marshal multiple arguments
pub fn marshal_args(
    args: &[FfiValue],
    types: &[FfiType],
) -> Result<Vec<FfiValue>, String> {
    if args.len() != types.len() {
        return Err(format!(
            "Argument count mismatch: {} args, {} types",
            args.len(), types.len()
        ));
    }
    args.iter()
        .zip(types.iter())
        .map(|(val, ty)| marshal_to_ffi(val, ty))
        .collect()
}
