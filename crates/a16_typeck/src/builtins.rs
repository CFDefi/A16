//! Built-in functions and types registration

use crate::context::TypeContext;
use crate::types::Type;
use a16_ast::Span;
use smol_str::SmolStr;

/// Register all built-in functions and types
pub fn register_all(ctx: &mut TypeContext) {
    register_builtin_functions(ctx);
    register_builtin_types(ctx);
}

/// Register built-in functions
fn register_builtin_functions(ctx: &mut TypeContext) {
    let dummy_span = Span { start: 0, end: 0 };
    
    // I/O functions
    ctx.define(
        SmolStr::new("print"),
        Type::Function {
            params: vec![Type::Any],
            ret: Box::new(Type::None),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("println"),
        Type::Function {
            params: vec![Type::Any],
            ret: Box::new(Type::None),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("input"),
        Type::Function {
            params: vec![Type::Str],
            ret: Box::new(Type::Str),
        },
        false,
        dummy_span,
    );
    
    // String functions
    ctx.define(
        SmolStr::new("upper"),
        Type::Function {
            params: vec![Type::Str],
            ret: Box::new(Type::Str),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("lower"),
        Type::Function {
            params: vec![Type::Str],
            ret: Box::new(Type::Str),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("strip"),
        Type::Function {
            params: vec![Type::Str],
            ret: Box::new(Type::Str),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("split"),
        Type::Function {
            params: vec![Type::Str, Type::Str],
            ret: Box::new(Type::List(Box::new(Type::Str))),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("join"),
        Type::Function {
            params: vec![Type::Str, Type::List(Box::new(Type::Str))],
            ret: Box::new(Type::Str),
        },
        false,
        dummy_span,
    );
    
    // List functions
    ctx.define(
        SmolStr::new("append"),
        Type::Function {
            params: vec![Type::Any, Type::Any],
            ret: Box::new(Type::None),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("push"),
        Type::Function {
            params: vec![Type::Any, Type::Any],
            ret: Box::new(Type::None),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("pop"),
        Type::Function {
            params: vec![Type::Any],
            ret: Box::new(Type::Any),
        },
        false,
        dummy_span,
    );
    
    // Type conversion functions
    ctx.define(
        SmolStr::new("int"),
        Type::Function {
            params: vec![Type::Any],
            ret: Box::new(Type::Int),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("float"),
        Type::Function {
            params: vec![Type::Any],
            ret: Box::new(Type::Float),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("str"),
        Type::Function {
            params: vec![Type::Any],
            ret: Box::new(Type::Str),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("bool"),
        Type::Function {
            params: vec![Type::Any],
            ret: Box::new(Type::Bool),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("bytes"),
        Type::Function {
            params: vec![Type::Any],
            ret: Box::new(Type::Bytes),
        },
        false,
        dummy_span,
    );
    
    // Collection functions
    ctx.define(
        SmolStr::new("list"),
        Type::Function {
            params: vec![Type::Any],
            ret: Box::new(Type::List(Box::new(Type::Any))),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("dict"),
        Type::Function {
            params: vec![],
            ret: Box::new(Type::Dict(Box::new(Type::Any), Box::new(Type::Any))),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("set"),
        Type::Function {
            params: vec![Type::Any],
            ret: Box::new(Type::Set(Box::new(Type::Any))),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("tuple"),
        Type::Function {
            params: vec![Type::Any],
            ret: Box::new(Type::Tuple(vec![])),
        },
        false,
        dummy_span,
    );
    
    // Utility functions
    ctx.define(
        SmolStr::new("len"),
        Type::Function {
            params: vec![Type::Any],
            ret: Box::new(Type::Int),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("range"),
        Type::Function {
            params: vec![Type::Int],
            ret: Box::new(Type::Iterator(Box::new(Type::Int))),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("enumerate"),
        Type::Function {
            params: vec![Type::Any],
            ret: Box::new(Type::Iterator(Box::new(Type::Tuple(vec![Type::Int, Type::Any])))),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("zip"),
        Type::Function {
            params: vec![Type::Any, Type::Any],
            ret: Box::new(Type::Iterator(Box::new(Type::Tuple(vec![Type::Any, Type::Any])))),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("map"),
        Type::Function {
            params: vec![Type::Any, Type::Any],
            ret: Box::new(Type::Iterator(Box::new(Type::Any))),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("filter"),
        Type::Function {
            params: vec![Type::Any, Type::Any],
            ret: Box::new(Type::Iterator(Box::new(Type::Any))),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("reduce"),
        Type::Function {
            params: vec![Type::Any, Type::Any, Type::Any],
            ret: Box::new(Type::Any),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("sorted"),
        Type::Function {
            params: vec![Type::Any],
            ret: Box::new(Type::List(Box::new(Type::Any))),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("reversed"),
        Type::Function {
            params: vec![Type::Any],
            ret: Box::new(Type::Iterator(Box::new(Type::Any))),
        },
        false,
        dummy_span,
    );
    
    // Math functions
    ctx.define(
        SmolStr::new("abs"),
        Type::Function {
            params: vec![Type::Any],
            ret: Box::new(Type::Any),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("min"),
        Type::Function {
            params: vec![Type::Any],
            ret: Box::new(Type::Any),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("max"),
        Type::Function {
            params: vec![Type::Any],
            ret: Box::new(Type::Any),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("sum"),
        Type::Function {
            params: vec![Type::Any],
            ret: Box::new(Type::Any),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("pow"),
        Type::Function {
            params: vec![Type::Any, Type::Any],
            ret: Box::new(Type::Any),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("sqrt"),
        Type::Function {
            params: vec![Type::Float],
            ret: Box::new(Type::Float),
        },
        false,
        dummy_span,
    );
    
    // Type checking functions
    ctx.define(
        SmolStr::new("type"),
        Type::Function {
            params: vec![Type::Any],
            ret: Box::new(Type::Str),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("isinstance"),
        Type::Function {
            params: vec![Type::Any, Type::Any],
            ret: Box::new(Type::Bool),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("hasattr"),
        Type::Function {
            params: vec![Type::Any, Type::Str],
            ret: Box::new(Type::Bool),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("getattr"),
        Type::Function {
            params: vec![Type::Any, Type::Str],
            ret: Box::new(Type::Any),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("setattr"),
        Type::Function {
            params: vec![Type::Any, Type::Str, Type::Any],
            ret: Box::new(Type::None),
        },
        false,
        dummy_span,
    );
    
    // String utilities
    ctx.define(
        SmolStr::new("repr"),
        Type::Function {
            params: vec![Type::Any],
            ret: Box::new(Type::Str),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("format"),
        Type::Function {
            params: vec![Type::Str, Type::Any],
            ret: Box::new(Type::Str),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("hash"),
        Type::Function {
            params: vec![Type::Any],
            ret: Box::new(Type::Int),
        },
        false,
        dummy_span,
    );
    
    // Async utilities
    ctx.define(
        SmolStr::new("sleep"),
        Type::AsyncFunction {
            params: vec![Type::Float],
            ret: Box::new(Type::None),
        },
        false,
        dummy_span,
    );
    
    // AI-specific builtins
    ctx.define(
        SmolStr::new("embed"),
        Type::AsyncFunction {
            params: vec![Type::Any],
            ret: Box::new(Type::Embedding),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("token_count"),
        Type::Function {
            params: vec![Type::Str],
            ret: Box::new(Type::Int),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("now"),
        Type::Function {
            params: vec![],
            ret: Box::new(Type::Float),
        },
        false,
        dummy_span,
    );
    
    // ===============================
    // Tensor Functions (Synchronous)
    // ===============================
    
    ctx.define(
        SmolStr::new("tensor_zeros"),
        Type::Function {
            params: vec![Type::List(Box::new(Type::Int))],
            ret: Box::new(Type::Any),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("tensor_ones"),
        Type::Function {
            params: vec![Type::List(Box::new(Type::Int))],
            ret: Box::new(Type::Any),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("tensor_randn"),
        Type::Function {
            params: vec![Type::List(Box::new(Type::Int))],
            ret: Box::new(Type::Any),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("tensor_add"),
        Type::Function {
            params: vec![Type::Any, Type::Any],
            ret: Box::new(Type::Any),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("tensor_mul"),
        Type::Function {
            params: vec![Type::Any, Type::Any],
            ret: Box::new(Type::Any),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("tensor_matmul"),
        Type::Function {
            params: vec![Type::Any, Type::Any],
            ret: Box::new(Type::Any),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("tensor_relu"),
        Type::Function {
            params: vec![Type::Any],
            ret: Box::new(Type::Any),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("tensor_mean"),
        Type::Function {
            params: vec![Type::Any],
            ret: Box::new(Type::Any),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("tensor_backward"),
        Type::Function {
            params: vec![Type::Any],
            ret: Box::new(Type::None),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("tensor_grad"),
        Type::Function {
            params: vec![Type::Any],
            ret: Box::new(Type::Any),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("tensor_item"),
        Type::Function {
            params: vec![Type::Any],
            ret: Box::new(Type::Float),
        },
        false,
        dummy_span,
    );
    
    // ===============================
    // Memory Functions (Synchronous)
    // ===============================
    
    ctx.define(
        SmolStr::new("memory_new"),
        Type::Function {
            params: vec![Type::Int],
            ret: Box::new(Type::Any),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("memory_store"),
        Type::Function {
            params: vec![Type::Any, Type::Any],
            ret: Box::new(Type::None),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("memory_retrieve"),
        Type::Function {
            params: vec![Type::Any, Type::Any],
            ret: Box::new(Type::Any),
        },
        false,
        dummy_span,
    );
    
    // Additional tensor APIs
    ctx.define(
        SmolStr::new("tensor_shape"),
        Type::Function {
            params: vec![Type::Any],
            ret: Box::new(Type::List(Box::new(Type::Int))),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("tensor_data"),
        Type::Function {
            params: vec![Type::Any],
            ret: Box::new(Type::List(Box::new(Type::Float))),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("tensor_from_list"),
        Type::Function {
            params: vec![Type::List(Box::new(Type::Float)), Type::List(Box::new(Type::Int))],
            ret: Box::new(Type::Any),
        },
        false,
        dummy_span,
    );
    
    ctx.define(
        SmolStr::new("tensor_sub"),
        Type::Function {
            params: vec![Type::Any, Type::Any],
            ret: Box::new(Type::Any),
        },
        false,
        dummy_span,
    );
}

/// Register built-in type aliases and constants
fn register_builtin_types(ctx: &mut TypeContext) {
    let dummy_span = Span { start: 0, end: 0 };
    
    // Boolean constants
    ctx.define(SmolStr::new("True"), Type::Bool, false, dummy_span);
    ctx.define(SmolStr::new("False"), Type::Bool, false, dummy_span);
    ctx.define(SmolStr::new("None"), Type::None, false, dummy_span);
    
    // Mathematical constants
    ctx.define(SmolStr::new("PI"), Type::Float, false, dummy_span);
    ctx.define(SmolStr::new("E"), Type::Float, false, dummy_span);
    ctx.define(SmolStr::new("INF"), Type::Float, false, dummy_span);
    ctx.define(SmolStr::new("NAN"), Type::Float, false, dummy_span);
}
