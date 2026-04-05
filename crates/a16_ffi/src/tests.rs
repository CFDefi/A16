//! FFI Tests

use crate::{
    DynLib, FfiFunc, FfiError, FfiType, FfiValue,
    FfiRegistry, marshal_to_ffi, marshal_from_ffi, marshal_args,
};

// =========================================================================
// DynLib Tests
// =========================================================================

#[test]
fn test_dynlib_construction() {
    let lib = DynLib::new("test_lib", Some("/path/to/lib.so"));
    assert_eq!(lib.name.as_str(), "test_lib");
    assert_eq!(lib.path.as_deref(), Some("/path/to/lib.so"));
    assert!(!lib.is_loaded());
}

#[test]
fn test_dynlib_mark_loaded() {
    let mut lib = DynLib::new("test_lib", None);
    assert!(!lib.is_loaded());
    lib.mark_loaded();
    assert!(lib.is_loaded());
}

// =========================================================================
// FfiFunc Tests
// =========================================================================

#[test]
fn test_ffi_func_construction() {
    let func = FfiFunc::new(
        "sqrt",
        "math",
        vec![FfiType::Float64],
        FfiType::Float64,
    );
    assert_eq!(func.name.as_str(), "sqrt");
    assert_eq!(func.lib_name.as_str(), "math");
    assert_eq!(func.param_types.len(), 1);
}

#[test]
fn test_ffi_func_validate_args() {
    let func = FfiFunc::new(
        "add",
        "math",
        vec![FfiType::Int64, FfiType::Int64],
        FfiType::Int64,
    );

    // Correct count
    assert!(func.validate_args(&[FfiValue::Int(1), FfiValue::Int(2)]).is_ok());

    // Wrong count
    assert!(func.validate_args(&[FfiValue::Int(1)]).is_err());
    assert!(func.validate_args(&[]).is_err());
}

// =========================================================================
// FfiType Tests
// =========================================================================

#[test]
fn test_ffi_type_from_name() {
    assert_eq!(FfiType::from_name("Int"), Some(FfiType::Int64));
    assert_eq!(FfiType::from_name("Float"), Some(FfiType::Float64));
    assert_eq!(FfiType::from_name("Str"), Some(FfiType::String));
    assert_eq!(FfiType::from_name("Bool"), Some(FfiType::Bool));
    assert_eq!(FfiType::from_name("Ptr"), Some(FfiType::Pointer));
    assert_eq!(FfiType::from_name("Void"), Some(FfiType::Void));
    assert_eq!(FfiType::from_name("i32"), Some(FfiType::Int32));
    assert_eq!(FfiType::from_name("f32"), Some(FfiType::Float32));
    assert_eq!(FfiType::from_name("UnknownType"), None);
}

#[test]
fn test_ffi_type_size() {
    assert_eq!(FfiType::Void.size(), 0);
    assert_eq!(FfiType::Int8.size(), 1);
    assert_eq!(FfiType::Int16.size(), 2);
    assert_eq!(FfiType::Int32.size(), 4);
    assert_eq!(FfiType::Int64.size(), 8);
    assert_eq!(FfiType::Float64.size(), 8);
    assert_eq!(FfiType::Pointer.size(), 8);
}

// =========================================================================
// FfiValue Tests
// =========================================================================

#[test]
fn test_ffi_value_accessors() {
    assert_eq!(FfiValue::Int(42).as_int(), Some(42));
    assert_eq!(FfiValue::Float(3.14).as_float(), Some(3.14));
    assert_eq!(FfiValue::Bool(true).as_int(), Some(1));
    assert_eq!(FfiValue::Str(smol_str::SmolStr::new("hello")).as_str(), Some("hello"));
    assert_eq!(FfiValue::Void.as_int(), None);
}

#[test]
fn test_ffi_value_type() {
    assert_eq!(FfiValue::Int(0).ffi_type(), FfiType::Int64);
    assert_eq!(FfiValue::Float(0.0).ffi_type(), FfiType::Float64);
    assert_eq!(FfiValue::Bool(false).ffi_type(), FfiType::Bool);
    assert_eq!(FfiValue::Void.ffi_type(), FfiType::Void);
}

// =========================================================================
// Marshaling Tests
// =========================================================================

#[test]
fn test_marshal_identity() {
    let val = FfiValue::Int(42);
    let result = marshal_to_ffi(&val, &FfiType::Int64).unwrap();
    assert_eq!(result.as_int(), Some(42));
}

#[test]
fn test_marshal_int_to_float() {
    let val = FfiValue::Int(42);
    let result = marshal_to_ffi(&val, &FfiType::Float64).unwrap();
    assert_eq!(result.as_float(), Some(42.0));
}

#[test]
fn test_marshal_bool_to_int() {
    let val = FfiValue::Bool(true);
    let result = marshal_to_ffi(&val, &FfiType::Int64).unwrap();
    assert_eq!(result.as_int(), Some(1));
}

#[test]
fn test_marshal_invalid() {
    let val = FfiValue::Str(smol_str::SmolStr::new("hello"));
    assert!(marshal_to_ffi(&val, &FfiType::Int64).is_err());
}

#[test]
fn test_marshal_args_batch() {
    let args = vec![FfiValue::Int(1), FfiValue::Float(2.0)];
    let types = vec![FfiType::Int64, FfiType::Float64];
    let result = marshal_args(&args, &types).unwrap();
    assert_eq!(result.len(), 2);
}

#[test]
fn test_marshal_args_mismatch_count() {
    let args = vec![FfiValue::Int(1)];
    let types = vec![FfiType::Int64, FfiType::Float64];
    assert!(marshal_args(&args, &types).is_err());
}

// =========================================================================
// Registry Tests
// =========================================================================

#[test]
fn test_registry_empty() {
    let reg = FfiRegistry::new();
    assert_eq!(reg.library_count(), 0);
    assert_eq!(reg.function_count(), 0);
}

#[test]
fn test_registry_register_library() {
    let mut reg = FfiRegistry::new();
    reg.register_library("math", None).unwrap();
    assert!(reg.has_library("math"));
    assert_eq!(reg.library_count(), 1);
}

#[test]
fn test_registry_register_function() {
    let mut reg = FfiRegistry::new();
    reg.register_library("math", None).unwrap();
    let idx = reg.register_function(
        "math", "sqrt",
        vec![FfiType::Float64],
        FfiType::Float64,
    ).unwrap();
    assert_eq!(idx, 0);
    assert_eq!(reg.function_count(), 1);
    assert!(reg.lookup("math", "sqrt").is_some());
}

#[test]
fn test_registry_register_extension() {
    let mut reg = FfiRegistry::new();
    reg.register_library("builtin", None).unwrap();

    fn native_add(args: &[FfiValue]) -> Result<FfiValue, FfiError> {
        let a = args[0].as_int().unwrap_or(0);
        let b = args[1].as_int().unwrap_or(0);
        Ok(FfiValue::Int(a + b))
    }

    let idx = reg.register_extension(
        "builtin", "add",
        vec![FfiType::Int64, FfiType::Int64],
        FfiType::Int64,
        native_add,
    ).unwrap();

    let result = reg.call("builtin", "add", &[FfiValue::Int(3), FfiValue::Int(4)]).unwrap();
    assert_eq!(result.as_int(), Some(7));
}

#[test]
fn test_registry_call_by_index() {
    let mut reg = FfiRegistry::new();
    reg.register_library("builtin", None).unwrap();

    fn native_negate(args: &[FfiValue]) -> Result<FfiValue, FfiError> {
        let n = args[0].as_int().unwrap_or(0);
        Ok(FfiValue::Int(-n))
    }

    let idx = reg.register_extension(
        "builtin", "negate",
        vec![FfiType::Int64],
        FfiType::Int64,
        native_negate,
    ).unwrap();

    let result = reg.call_by_index(idx, &[FfiValue::Int(42)]).unwrap();
    assert_eq!(result.as_int(), Some(-42));
}

#[test]
fn test_registry_list_functions() {
    let mut reg = FfiRegistry::new();
    reg.register_library("math", None).unwrap();
    reg.register_function("math", "sin", vec![FfiType::Float64], FfiType::Float64).unwrap();
    reg.register_function("math", "cos", vec![FfiType::Float64], FfiType::Float64).unwrap();

    let funcs = reg.list_functions("math");
    assert_eq!(funcs.len(), 2);
    assert!(funcs.contains(&"sin"));
    assert!(funcs.contains(&"cos"));
}

#[test]
fn test_registry_function_not_found() {
    let reg = FfiRegistry::new();
    let result = reg.call("math", "sqrt", &[FfiValue::Float(4.0)]);
    assert!(result.is_err());
}

#[test]
fn test_registry_arg_count_mismatch() {
    let mut reg = FfiRegistry::new();
    reg.register_library("builtin", None).unwrap();

    fn native_id(args: &[FfiValue]) -> Result<FfiValue, FfiError> {
        Ok(args[0].clone())
    }

    reg.register_extension(
        "builtin", "id",
        vec![FfiType::Int64],
        FfiType::Int64,
        native_id,
    ).unwrap();

    // Too many args
    let result = reg.call("builtin", "id", &[FfiValue::Int(1), FfiValue::Int(2)]);
    assert!(result.is_err());
}
