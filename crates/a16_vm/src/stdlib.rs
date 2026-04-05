//! Standard Library - Native Functions

use smol_str::SmolStr;
use std::rc::Rc;
use std::cell::RefCell;
use indexmap::IndexMap;

use crate::value::{Value, NativeFunc};
use crate::agent::{AgentRuntime, ModelHandle};
use crate::tool_registry::ToolRegistry;
use a16_tensor::{Tensor, MLP, Activation};
use a16_vector::{HNSWIndex, Embedder, TfIdfEmbedder};

/// Register all standard library functions
pub fn register_stdlib() -> IndexMap<SmolStr, Value> {
    let mut globals = IndexMap::new();
    
    // I/O functions
    globals.insert(SmolStr::new("print"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("print"),
        arity: 1,
        func: native_print,
    }));
    
    globals.insert(SmolStr::new("println"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("println"),
        arity: 1,
        func: native_println,
    }));
    
    globals.insert(SmolStr::new("input"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("input"),
        arity: 1,
        func: native_input,
    }));
    
    // Type conversion
    globals.insert(SmolStr::new("str"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("str"),
        arity: 1,
        func: native_str,
    }));
    
    globals.insert(SmolStr::new("int"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("int"),
        arity: 1,
        func: native_int,
    }));
    
    globals.insert(SmolStr::new("float"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("float"),
        arity: 1,
        func: native_float,
    }));
    
    globals.insert(SmolStr::new("bool"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("bool"),
        arity: 1,
        func: native_bool,
    }));
    
    // Collection functions
    globals.insert(SmolStr::new("len"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("len"),
        arity: 1,
        func: native_len,
    }));
    
    globals.insert(SmolStr::new("range"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("range"),
        arity: 1,
        func: native_range,
    }));
    
    globals.insert(SmolStr::new("list"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("list"),
        arity: 1,
        func: native_list,
    }));
    
    // Utility functions
    globals.insert(SmolStr::new("type"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("type"),
        arity: 1,
        func: native_type,
    }));
    
    globals.insert(SmolStr::new("abs"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("abs"),
        arity: 1,
        func: native_abs,
    }));
    
    globals.insert(SmolStr::new("min"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("min"),
        arity: 2,
        func: native_min,
    }));
    
    globals.insert(SmolStr::new("max"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("max"),
        arity: 2,
        func: native_max,
    }));
    
    // List methods (as functions for now)
    globals.insert(SmolStr::new("append"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("append"),
        arity: 2,
        func: native_append,
    }));
    
    globals.insert(SmolStr::new("push"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("push"),
        arity: 2,
        func: native_append,
    }));
    
    globals.insert(SmolStr::new("pop"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("pop"),
        arity: 1,
        func: native_pop,
    }));
    
    // String functions
    globals.insert(SmolStr::new("upper"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("upper"),
        arity: 1,
        func: native_upper,
    }));
    
    globals.insert(SmolStr::new("lower"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("lower"),
        arity: 1,
        func: native_lower,
    }));
    
    globals.insert(SmolStr::new("split"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("split"),
        arity: 2,
        func: native_split,
    }));
    
    globals.insert(SmolStr::new("join"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("join"),
        arity: 2,
        func: native_join,
    }));
    
    globals.insert(SmolStr::new("strip"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("strip"),
        arity: 1,
        func: native_strip,
    }));
    
    // ===============================
    // AI Runtime Functions
    // ===============================
    
    // Tensor creation
    globals.insert(SmolStr::new("tensor_zeros"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("tensor_zeros"),
        arity: 1,
        func: native_tensor_zeros,
    }));
    
    globals.insert(SmolStr::new("tensor_ones"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("tensor_ones"),
        arity: 1,
        func: native_tensor_ones,
    }));
    
    globals.insert(SmolStr::new("tensor_randn"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("tensor_randn"),
        arity: 1,
        func: native_tensor_randn,
    }));
    
    // Tensor operations
    globals.insert(SmolStr::new("tensor_add"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("tensor_add"),
        arity: 2,
        func: native_tensor_add,
    }));
    
    globals.insert(SmolStr::new("tensor_mul"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("tensor_mul"),
        arity: 2,
        func: native_tensor_mul,
    }));
    
    globals.insert(SmolStr::new("tensor_matmul"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("tensor_matmul"),
        arity: 2,
        func: native_tensor_matmul,
    }));
    
    globals.insert(SmolStr::new("tensor_relu"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("tensor_relu"),
        arity: 1,
        func: native_tensor_relu,
    }));
    
    globals.insert(SmolStr::new("tensor_mean"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("tensor_mean"),
        arity: 1,
        func: native_tensor_mean,
    }));
    
    globals.insert(SmolStr::new("tensor_backward"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("tensor_backward"),
        arity: 1,
        func: native_tensor_backward,
    }));
    
    globals.insert(SmolStr::new("tensor_grad"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("tensor_grad"),
        arity: 1,
        func: native_tensor_grad,
    }));
    
    globals.insert(SmolStr::new("tensor_item"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("tensor_item"),
        arity: 1,
        func: native_tensor_item,
    }));
    
    // Additional tensor APIs
    globals.insert(SmolStr::new("tensor_shape"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("tensor_shape"),
        arity: 1,
        func: native_tensor_shape,
    }));
    
    globals.insert(SmolStr::new("tensor_data"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("tensor_data"),
        arity: 1,
        func: native_tensor_data,
    }));
    
    globals.insert(SmolStr::new("tensor_from_list"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("tensor_from_list"),
        arity: 2,
        func: native_tensor_from_list,
    }));
    
    globals.insert(SmolStr::new("tensor_sub"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("tensor_sub"),
        arity: 2,
        func: native_tensor_sub,
    }));
    
    // Embedding
    globals.insert(SmolStr::new("embed"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("embed"),
        arity: 1,
        func: native_embed,
    }));
    
    // Memory (Vector Index)
    globals.insert(SmolStr::new("memory_new"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("memory_new"),
        arity: 1,
        func: native_memory_new,
    }));
    
    globals.insert(SmolStr::new("memory_store"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("memory_store"),
        arity: 2,
        func: native_memory_store,
    }));
    
    globals.insert(SmolStr::new("memory_retrieve"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("memory_retrieve"),
        arity: 2,
        func: native_memory_retrieve,
    }));
    
    // ===============================
    // M8: Agent & Model Functions
    // ===============================
    
    globals.insert(SmolStr::new("agent_create"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("agent_create"),
        arity: 2,
        func: native_agent_create,
    }));
    
    globals.insert(SmolStr::new("agent_run"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("agent_run"),
        arity: 1,
        func: native_agent_run,
    }));
    
    globals.insert(SmolStr::new("agent_step"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("agent_step"),
        arity: 1,
        func: native_agent_step,
    }));
    
    globals.insert(SmolStr::new("agent_status"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("agent_status"),
        arity: 1,
        func: native_agent_status,
    }));
    
    globals.insert(SmolStr::new("model_create"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("model_create"),
        arity: 1,
        func: native_model_create,
    }));
    
    globals.insert(SmolStr::new("model_generate"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("model_generate"),
        arity: 2,
        func: native_model_generate,
    }));
    
    globals.insert(SmolStr::new("tool_list"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("tool_list"),
        arity: 0,
        func: native_tool_list,
    }));
    
    globals.insert(SmolStr::new("tool_call"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("tool_call"),
        arity: 2,
        func: native_tool_call,
    }));
    
    globals.insert(SmolStr::new("mlp_create"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("mlp_create"),
        arity: 2,
        func: native_mlp_create,
    }));
    
    globals.insert(SmolStr::new("mlp_forward"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("mlp_forward"),
        arity: 2,
        func: native_mlp_forward,
    }));
    
    globals.insert(SmolStr::new("mlp_train_step"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("mlp_train_step"),
        arity: 2,
        func: native_mlp_train_step,
    }));
    // ===============================
    // M11: Async & Concurrency Functions
    // ===============================
    
    globals.insert(SmolStr::new("channel_create"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("channel_create"),
        arity: 1,
        func: native_channel_create,
    }));
    
    globals.insert(SmolStr::new("channel_send"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("channel_send"),
        arity: 2,
        func: native_channel_send,
    }));
    
    globals.insert(SmolStr::new("channel_recv"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("channel_recv"),
        arity: 1,
        func: native_channel_recv,
    }));
    
    globals.insert(SmolStr::new("channel_close"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("channel_close"),
        arity: 1,
        func: native_channel_close,
    }));
    
    globals.insert(SmolStr::new("task_status"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("task_status"),
        arity: 1,
        func: native_task_status,
    }));
    
    globals.insert(SmolStr::new("task_cancel"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("task_cancel"),
        arity: 1,
        func: native_task_cancel,
    }));
    
    globals.insert(SmolStr::new("sleep"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("sleep"),
        arity: 1,
        func: native_sleep,
    }));
    
    globals.insert(SmolStr::new("spawn_fn"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("spawn_fn"),
        arity: 1,
        func: native_spawn_fn,
    }));
    
    globals.insert(SmolStr::new("await_result"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("await_result"),
        arity: 1,
        func: native_await_result,
    }));
    
    globals.insert(SmolStr::new("parallel_exec"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("parallel_exec"),
        arity: 1,
        func: native_parallel_exec,
    }));
    
    // ===============================
    // M12: FFI & Native Extension Functions
    // ===============================
    
    globals.insert(SmolStr::new("ffi_load"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("ffi_load"),
        arity: 2,
        func: native_ffi_load,
    }));
    
    globals.insert(SmolStr::new("ffi_call"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("ffi_call"),
        arity: 2,
        func: native_ffi_call,
    }));
    
    globals.insert(SmolStr::new("ffi_list"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("ffi_list"),
        arity: 1,
        func: native_ffi_list,
    }));
    
    globals.insert(SmolStr::new("ffi_unload"), Value::NativeFunc(NativeFunc {
        name: SmolStr::new("ffi_unload"),
        arity: 1,
        func: native_ffi_unload,
    }));
    
    globals
}

// === Native Function Implementations ===

fn native_print(args: &mut Vec<Value>) -> Value {
    if let Some(val) = args.first() {
        print!("{}", val);
    }
    Value::None
}

fn native_println(args: &mut Vec<Value>) -> Value {
    if let Some(val) = args.first() {
        println!("{}", val);
    } else {
        println!();
    }
    Value::None
}

fn native_input(args: &mut Vec<Value>) -> Value {
    if let Some(Value::Str(prompt)) = args.first() {
        print!("{}", prompt);
        use std::io::Write;
        std::io::stdout().flush().ok();
    }
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok();
    Value::Str(SmolStr::new(input.trim()))
}

fn native_str(args: &mut Vec<Value>) -> Value {
    if let Some(val) = args.first() {
        Value::Str(SmolStr::new(format!("{}", val)))
    } else {
        Value::Str(SmolStr::new(""))
    }
}

fn native_int(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(Value::Int(n)) => Value::Int(*n),
        Some(Value::Float(n)) => Value::Int(*n as i64),
        Some(Value::Str(s)) => {
            s.parse::<i64>().map(Value::Int).unwrap_or(Value::None)
        }
        Some(Value::Bool(b)) => Value::Int(if *b { 1 } else { 0 }),
        _ => Value::None,
    }
}

fn native_float(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(Value::Float(n)) => Value::Float(*n),
        Some(Value::Int(n)) => Value::Float(*n as f64),
        Some(Value::Str(s)) => {
            s.parse::<f64>().map(Value::Float).unwrap_or(Value::None)
        }
        _ => Value::None,
    }
}

fn native_bool(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(val) => Value::Bool(val.is_truthy()),
        None => Value::Bool(false),
    }
}

fn native_len(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(Value::Str(s)) => Value::Int(s.len() as i64),
        Some(Value::List(list)) => Value::Int(list.borrow().len() as i64),
        Some(Value::Dict(dict)) => Value::Int(dict.borrow().len() as i64),
        Some(Value::Tuple(tuple)) => Value::Int(tuple.len() as i64),
        _ => Value::Int(0),
    }
}

fn native_range(args: &mut Vec<Value>) -> Value {
    if let Some(Value::Int(n)) = args.first() {
        let values: Vec<Value> = (0..*n).map(Value::Int).collect();
        Value::List(Rc::new(RefCell::new(values)))
    } else {
        Value::List(Rc::new(RefCell::new(vec![])))
    }
}

fn native_list(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(Value::Tuple(tuple)) => {
            Value::List(Rc::new(RefCell::new(tuple.as_ref().clone())))
        }
        Some(Value::List(list)) => {
            Value::List(Rc::new(RefCell::new(list.borrow().clone())))
        }
        Some(Value::Str(s)) => {
            let chars: Vec<Value> = s.chars()
                .map(|c| Value::Str(SmolStr::new(c.to_string())))
                .collect();
            Value::List(Rc::new(RefCell::new(chars)))
        }
        _ => Value::List(Rc::new(RefCell::new(vec![]))),
    }
}

fn native_type(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(val) => Value::Str(SmolStr::new(val.type_name())),
        None => Value::Str(SmolStr::new("None")),
    }
}

fn native_abs(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(Value::Int(n)) => Value::Int(n.abs()),
        Some(Value::Float(n)) => Value::Float(n.abs()),
        _ => Value::None,
    }
}

fn native_min(args: &mut Vec<Value>) -> Value {
    if args.len() < 2 {
        return Value::None;
    }
    match (&args[0], &args[1]) {
        (Value::Int(a), Value::Int(b)) => Value::Int(*a.min(b)),
        (Value::Float(a), Value::Float(b)) => Value::Float(a.min(*b)),
        _ => Value::None,
    }
}

fn native_max(args: &mut Vec<Value>) -> Value {
    if args.len() < 2 {
        return Value::None;
    }
    match (&args[0], &args[1]) {
        (Value::Int(a), Value::Int(b)) => Value::Int(*a.max(b)),
        (Value::Float(a), Value::Float(b)) => Value::Float(a.max(*b)),
        _ => Value::None,
    }
}

fn native_append(args: &mut Vec<Value>) -> Value {
    if args.len() < 2 {
        return Value::None;
    }
    if let Value::List(list) = &args[0] {
        list.borrow_mut().push(args[1].clone());
    }
    Value::None
}

fn native_pop(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(Value::List(list)) => {
            list.borrow_mut().pop().unwrap_or(Value::None)
        }
        _ => Value::None,
    }
}

fn native_upper(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(Value::Str(s)) => Value::Str(SmolStr::new(s.to_uppercase())),
        _ => Value::None,
    }
}

fn native_lower(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(Value::Str(s)) => Value::Str(SmolStr::new(s.to_lowercase())),
        _ => Value::None,
    }
}

fn native_split(args: &mut Vec<Value>) -> Value {
    if args.len() < 2 {
        return Value::None;
    }
    match (&args[0], &args[1]) {
        (Value::Str(s), Value::Str(sep)) => {
            let parts: Vec<Value> = s.split(sep.as_str())
                .map(|p| Value::Str(SmolStr::new(p)))
                .collect();
            Value::List(Rc::new(RefCell::new(parts)))
        }
        _ => Value::None,
    }
}

fn native_join(args: &mut Vec<Value>) -> Value {
    if args.len() < 2 {
        return Value::None;
    }
    match (&args[0], &args[1]) {
        (Value::Str(sep), Value::List(list)) => {
            let parts: Vec<String> = list.borrow().iter()
                .map(|v| format!("{}", v))
                .collect();
            Value::Str(SmolStr::new(parts.join(sep.as_str())))
        }
        _ => Value::None,
    }
}

fn native_strip(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(Value::Str(s)) => Value::Str(SmolStr::new(s.trim())),
        _ => Value::None,
    }
}

// ===============================
// AI Runtime Function Implementations
// ===============================

// Helper: Extract shape from a list of ints
fn list_to_shape(val: &Value) -> Option<Vec<usize>> {
    if let Value::List(list) = val {
        let borrowed = list.borrow();
        let mut shape = Vec::with_capacity(borrowed.len());
        for v in borrowed.iter() {
            if let Value::Int(n) = v {
                shape.push(*n as usize);
            } else {
                return None;
            }
        }
        Some(shape)
    } else {
        None
    }
}

// Error reporting helper
fn runtime_error(func: &str, msg: &str) -> Value {
    eprintln!("Runtime error in {}: {}", func, msg);
    Value::None
}

fn native_tensor_zeros(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(val) => match list_to_shape(val) {
            Some(shape) if !shape.is_empty() => {
                Value::Tensor(Rc::new(RefCell::new(Tensor::zeros(&shape))))
            }
            Some(_) => runtime_error("tensor_zeros", "shape cannot be empty"),
            None => runtime_error("tensor_zeros", "expected list of integers for shape"),
        },
        None => runtime_error("tensor_zeros", "missing shape argument"),
    }
}

fn native_tensor_ones(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(val) => match list_to_shape(val) {
            Some(shape) if !shape.is_empty() => {
                Value::Tensor(Rc::new(RefCell::new(Tensor::ones(&shape))))
            }
            Some(_) => runtime_error("tensor_ones", "shape cannot be empty"),
            None => runtime_error("tensor_ones", "expected list of integers for shape"),
        },
        None => runtime_error("tensor_ones", "missing shape argument"),
    }
}

fn native_tensor_randn(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(val) => match list_to_shape(val) {
            Some(shape) if !shape.is_empty() => {
                Value::Tensor(Rc::new(RefCell::new(Tensor::randn(&shape))))
            }
            Some(_) => runtime_error("tensor_randn", "shape cannot be empty"),
            None => runtime_error("tensor_randn", "expected list of integers for shape"),
        },
        None => runtime_error("tensor_randn", "missing shape argument"),
    }
}

fn native_tensor_add(args: &mut Vec<Value>) -> Value {
    if args.len() < 2 {
        return runtime_error("tensor_add", "requires 2 arguments");
    }
    match (&args[0], &args[1]) {
        (Value::Tensor(a), Value::Tensor(b)) => {
            let a_ref = a.borrow();
            let b_ref = b.borrow();
            // Check shape compatibility
            if a_ref.shape() != b_ref.shape() && a_ref.size() != 1 && b_ref.size() != 1 {
                return runtime_error("tensor_add", 
                    &format!("shape mismatch: {:?} vs {:?}", a_ref.shape(), b_ref.shape()));
            }
            let result = a_ref.add(&b_ref);
            Value::Tensor(Rc::new(RefCell::new(result)))
        }
        (Value::Tensor(_), other) => runtime_error("tensor_add", 
            &format!("second argument must be Tensor, got {}", other.type_name())),
        (other, _) => runtime_error("tensor_add", 
            &format!("first argument must be Tensor, got {}", other.type_name())),
    }
}

fn native_tensor_mul(args: &mut Vec<Value>) -> Value {
    if args.len() < 2 {
        return runtime_error("tensor_mul", "requires 2 arguments");
    }
    match (&args[0], &args[1]) {
        (Value::Tensor(a), Value::Tensor(b)) => {
            let a_ref = a.borrow();
            let b_ref = b.borrow();
            if a_ref.shape() != b_ref.shape() && a_ref.size() != 1 && b_ref.size() != 1 {
                return runtime_error("tensor_mul", 
                    &format!("shape mismatch: {:?} vs {:?}", a_ref.shape(), b_ref.shape()));
            }
            let result = a_ref.mul(&b_ref);
            Value::Tensor(Rc::new(RefCell::new(result)))
        }
        (Value::Tensor(_), other) => runtime_error("tensor_mul", 
            &format!("second argument must be Tensor, got {}", other.type_name())),
        (other, _) => runtime_error("tensor_mul", 
            &format!("first argument must be Tensor, got {}", other.type_name())),
    }
}

fn native_tensor_matmul(args: &mut Vec<Value>) -> Value {
    if args.len() < 2 {
        return runtime_error("tensor_matmul", "requires 2 arguments");
    }
    match (&args[0], &args[1]) {
        (Value::Tensor(a), Value::Tensor(b)) => {
            let a_ref = a.borrow();
            let b_ref = b.borrow();
            // Check matmul dimension compatibility
            let a_cols = a_ref.shape().last().copied().unwrap_or(0);
            let b_rows = if b_ref.ndim() >= 2 { 
                b_ref.shape()[b_ref.ndim() - 2] 
            } else { 
                b_ref.shape().first().copied().unwrap_or(0) 
            };
            if a_cols != b_rows {
                return runtime_error("tensor_matmul", 
                    &format!("dimension mismatch: {} columns vs {} rows", a_cols, b_rows));
            }
            let result = a_ref.matmul(&b_ref);
            Value::Tensor(Rc::new(RefCell::new(result)))
        }
        (Value::Tensor(_), other) => runtime_error("tensor_matmul", 
            &format!("second argument must be Tensor, got {}", other.type_name())),
        (other, _) => runtime_error("tensor_matmul", 
            &format!("first argument must be Tensor, got {}", other.type_name())),
    }
}

fn native_tensor_relu(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(Value::Tensor(t)) => {
            let result = t.borrow().relu();
            Value::Tensor(Rc::new(RefCell::new(result)))
        }
        Some(other) => runtime_error("tensor_relu", 
            &format!("expected Tensor, got {}", other.type_name())),
        None => runtime_error("tensor_relu", "missing argument"),
    }
}

fn native_tensor_mean(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(Value::Tensor(t)) => {
            if t.borrow().size() == 0 {
                return runtime_error("tensor_mean", "cannot compute mean of empty tensor");
            }
            let result = t.borrow().mean();
            Value::Tensor(Rc::new(RefCell::new(result)))
        }
        Some(other) => runtime_error("tensor_mean", 
            &format!("expected Tensor, got {}", other.type_name())),
        None => runtime_error("tensor_mean", "missing argument"),
    }
}

fn native_tensor_backward(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(Value::Tensor(t)) => {
            t.borrow().backward();
            Value::None
        }
        Some(other) => runtime_error("tensor_backward", 
            &format!("expected Tensor, got {}", other.type_name())),
        None => runtime_error("tensor_backward", "missing argument"),
    }
}

fn native_tensor_grad(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(Value::Tensor(t)) => {
            match t.borrow().grad() {
                Some(grad) => Value::Tensor(Rc::new(RefCell::new(grad))),
                None => Value::None, // No gradient is valid
            }
        }
        Some(other) => runtime_error("tensor_grad", 
            &format!("expected Tensor, got {}", other.type_name())),
        None => runtime_error("tensor_grad", "missing argument"),
    }
}

fn native_tensor_item(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(Value::Tensor(t)) => {
            let tensor = t.borrow();
            if tensor.size() != 1 {
                return runtime_error("tensor_item", 
                    &format!("tensor must have exactly 1 element, got {}", tensor.size()));
            }
            Value::Float(tensor.item() as f64)
        }
        Some(other) => runtime_error("tensor_item", 
            &format!("expected Tensor, got {}", other.type_name())),
        None => runtime_error("tensor_item", "missing argument"),
    }
}

fn native_tensor_shape(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(Value::Tensor(t)) => {
            let shape: Vec<Value> = t.borrow().shape()
                .iter()
                .map(|&d| Value::Int(d as i64))
                .collect();
            Value::List(Rc::new(RefCell::new(shape)))
        }
        Some(other) => runtime_error("tensor_shape", 
            &format!("expected Tensor, got {}", other.type_name())),
        None => runtime_error("tensor_shape", "missing argument"),
    }
}

fn native_tensor_data(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(Value::Tensor(t)) => {
            let data: Vec<Value> = t.borrow().data()
                .iter()
                .map(|&v| Value::Float(v as f64))
                .collect();
            Value::List(Rc::new(RefCell::new(data)))
        }
        Some(other) => runtime_error("tensor_data", 
            &format!("expected Tensor, got {}", other.type_name())),
        None => runtime_error("tensor_data", "missing argument"),
    }
}

fn native_tensor_from_list(args: &mut Vec<Value>) -> Value {
    if args.len() < 2 {
        return runtime_error("tensor_from_list", "requires 2 arguments (data, shape)");
    }
    // Extract data
    let data: Option<Vec<f32>> = match &args[0] {
        Value::List(list) => {
            let borrowed = list.borrow();
            let mut result = Vec::with_capacity(borrowed.len());
            for v in borrowed.iter() {
                match v {
                    Value::Float(f) => result.push(*f as f32),
                    Value::Int(i) => result.push(*i as f32),
                    _ => return runtime_error("tensor_from_list", 
                        "data list must contain only numbers"),
                }
            }
            Some(result)
        }
        _ => None,
    };
    
    // Extract shape
    let shape = list_to_shape(&args[1]);
    
    match (data, shape) {
        (Some(d), Some(s)) => {
            let expected_size: usize = s.iter().product();
            if d.len() != expected_size {
                return runtime_error("tensor_from_list",
                    &format!("data length {} != shape product {}", d.len(), expected_size));
            }
            Value::Tensor(Rc::new(RefCell::new(Tensor::from_data(d, s))))
        }
        (None, _) => runtime_error("tensor_from_list", "first argument must be list of numbers"),
        (_, None) => runtime_error("tensor_from_list", "second argument must be list of integers"),
    }
}

fn native_tensor_sub(args: &mut Vec<Value>) -> Value {
    if args.len() < 2 {
        return runtime_error("tensor_sub", "requires 2 arguments");
    }
    match (&args[0], &args[1]) {
        (Value::Tensor(a), Value::Tensor(b)) => {
            let a_ref = a.borrow();
            let b_ref = b.borrow();
            if a_ref.shape() != b_ref.shape() && a_ref.size() != 1 && b_ref.size() != 1 {
                return runtime_error("tensor_sub", 
                    &format!("shape mismatch: {:?} vs {:?}", a_ref.shape(), b_ref.shape()));
            }
            let result = a_ref.sub(&b_ref);
            Value::Tensor(Rc::new(RefCell::new(result)))
        }
        (Value::Tensor(_), other) => runtime_error("tensor_sub", 
            &format!("second argument must be Tensor, got {}", other.type_name())),
        (other, _) => runtime_error("tensor_sub", 
            &format!("first argument must be Tensor, got {}", other.type_name())),
    }
}

// Default embedder dimension
const EMBED_DIM: usize = 64;

fn native_embed(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(Value::Str(text)) => {
            if text.is_empty() {
                return runtime_error("embed", "cannot embed empty string");
            }
            let embedder = TfIdfEmbedder::new(EMBED_DIM);
            let vec = embedder.embed(text.as_str());
            Value::Vector(Rc::new(vec))
        }
        Some(other) => runtime_error("embed", 
            &format!("expected Str, got {}", other.type_name())),
        None => runtime_error("embed", "missing argument"),
    }
}

fn native_memory_new(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(Value::Int(dim)) if *dim > 0 => {
            let index = HNSWIndex::new(*dim as usize);
            Value::MemoryIndex(Rc::new(RefCell::new(index)))
        }
        Some(Value::Int(_)) => runtime_error("memory_new", "dimension must be positive"),
        Some(other) => runtime_error("memory_new", 
            &format!("expected Int dimension, got {}", other.type_name())),
        None => {
            // Default to embed dimension
            let index = HNSWIndex::new(EMBED_DIM);
            Value::MemoryIndex(Rc::new(RefCell::new(index)))
        }
    }
}

fn native_memory_store(args: &mut Vec<Value>) -> Value {
    if args.len() < 2 {
        return runtime_error("memory_store", "requires 2 arguments (memory, vector)");
    }
    match (&args[0], &args[1]) {
        (Value::MemoryIndex(idx), Value::Vector(vec)) => {
            let index = idx.borrow();
            if vec.len() != index.dim() {
                return runtime_error("memory_store", 
                    &format!("vector dimension {} != memory dimension {}", vec.len(), index.dim()));
            }
            drop(index);
            idx.borrow_mut().insert(vec.as_ref().clone(), None);
            Value::None
        }
        (Value::MemoryIndex(_), other) => runtime_error("memory_store", 
            &format!("second argument must be Vector, got {}", other.type_name())),
        (other, _) => runtime_error("memory_store", 
            &format!("first argument must be MemoryIndex, got {}", other.type_name())),
    }
}

fn native_memory_retrieve(args: &mut Vec<Value>) -> Value {
    if args.len() < 2 {
        return runtime_error("memory_retrieve", "requires 2 arguments (memory, query)");
    }
    match (&args[0], &args[1]) {
        (Value::MemoryIndex(idx), Value::Vector(query)) => {
            let index = idx.borrow();
            if query.len() != index.dim() {
                return runtime_error("memory_retrieve", 
                    &format!("query dimension {} != memory dimension {}", query.len(), index.dim()));
            }
            let results = index.search(query.as_ref(), 5, 50);
            let scores: Vec<Value> = results.iter()
                .map(|r| Value::Float(r.score as f64))
                .collect();
            Value::List(Rc::new(RefCell::new(scores)))
        }
        (Value::MemoryIndex(idx), Value::Int(_k)) => {
            Value::Int(idx.borrow().len() as i64)
        }
        (Value::MemoryIndex(_), other) => runtime_error("memory_retrieve", 
            &format!("second argument must be Vector or Int, got {}", other.type_name())),
        (other, _) => runtime_error("memory_retrieve", 
            &format!("first argument must be MemoryIndex, got {}", other.type_name())),
    }
}

// ===============================
// M8: Agent, Model, Tool Functions
// ===============================

fn native_agent_create(args: &mut Vec<Value>) -> Value {
    if args.len() < 2 {
        return runtime_error("agent_create", "requires 2 arguments (name, model)");
    }
    match (&args[0], &args[1]) {
        (Value::Str(name), Value::ModelVal(model)) => {
            let tools = Rc::new(RefCell::new(ToolRegistry::new()));
            let agent = AgentRuntime::new(
                name.clone(),
                model.borrow().clone(),
                tools,
            );
            Value::Agent(Rc::new(RefCell::new(agent)))
        }
        (Value::Str(name), Value::Str(_model_name)) => {
            // Create with a named model (stub)
            let tools = Rc::new(RefCell::new(ToolRegistry::new()));
            let model = ModelHandle::new(name.as_str());
            let agent = AgentRuntime::new(
                name.clone(),
                model,
                tools,
            );
            Value::Agent(Rc::new(RefCell::new(agent)))
        }
        (other, _) => runtime_error("agent_create",
            &format!("first argument must be Str, got {}", other.type_name())),
    }
}

fn native_agent_run(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(Value::Agent(agent)) => {
            let result = agent.borrow_mut().run();
            Value::Str(result)
        }
        Some(other) => runtime_error("agent_run",
            &format!("expected Agent, got {}", other.type_name())),
        None => runtime_error("agent_run", "missing argument"),
    }
}

fn native_agent_step(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(Value::Agent(agent)) => {
            let state = agent.borrow_mut().step();
            Value::Str(SmolStr::new(state.to_string()))
        }
        Some(other) => runtime_error("agent_step",
            &format!("expected Agent, got {}", other.type_name())),
        None => runtime_error("agent_step", "missing argument"),
    }
}

fn native_agent_status(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(Value::Agent(agent)) => {
            Value::Str(SmolStr::new(agent.borrow().status()))
        }
        Some(other) => runtime_error("agent_status",
            &format!("expected Agent, got {}", other.type_name())),
        None => runtime_error("agent_status", "missing argument"),
    }
}

fn native_model_create(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(Value::Str(name)) => {
            let model = ModelHandle::new(name.clone());
            Value::ModelVal(Rc::new(RefCell::new(model)))
        }
        Some(other) => runtime_error("model_create",
            &format!("expected Str, got {}", other.type_name())),
        None => runtime_error("model_create", "missing argument"),
    }
}

fn native_model_generate(args: &mut Vec<Value>) -> Value {
    if args.len() < 2 {
        return runtime_error("model_generate", "requires 2 arguments (model, input)");
    }
    match (&args[0], &args[1]) {
        (Value::ModelVal(model), Value::Tensor(input)) => {
            let result = model.borrow().generate(&input.borrow());
            Value::Tensor(Rc::new(RefCell::new(result)))
        }
        (Value::ModelVal(_), other) => runtime_error("model_generate",
            &format!("second argument must be Tensor, got {}", other.type_name())),
        (other, _) => runtime_error("model_generate",
            &format!("first argument must be Model, got {}", other.type_name())),
    }
}

fn native_tool_list(args: &mut Vec<Value>) -> Value {
    let _ = args;
    let registry = ToolRegistry::new();
    let names: Vec<Value> = registry.list().iter()
        .map(|n| Value::Str(n.clone()))
        .collect();
    Value::List(Rc::new(RefCell::new(names)))
}

fn native_tool_call(args: &mut Vec<Value>) -> Value {
    if args.len() < 2 {
        return runtime_error("tool_call", "requires 2 arguments (name, args)");
    }
    match (&args[0], &args[1]) {
        (Value::Str(name), Value::List(tool_args)) => {
            let registry = ToolRegistry::new();
            let str_args: Vec<SmolStr> = tool_args.borrow().iter()
                .map(|v| SmolStr::new(format!("{}", v)))
                .collect();
            match registry.call(name.as_str(), &str_args) {
                Some(result) => {
                    if result.success {
                        Value::Str(result.output)
                    } else {
                        runtime_error("tool_call",
                            &format!("tool failed: {}", result.error.unwrap_or_default()))
                    }
                }
                None => runtime_error("tool_call",
                    &format!("tool '{}' not found", name)),
            }
        }
        (Value::Str(_), other) => runtime_error("tool_call",
            &format!("second argument must be List, got {}", other.type_name())),
        (other, _) => runtime_error("tool_call",
            &format!("first argument must be Str, got {}", other.type_name())),
    }
}

fn native_mlp_create(args: &mut Vec<Value>) -> Value {
    if args.len() < 2 {
        return runtime_error("mlp_create", "requires 2 arguments (layer_sizes, activation)");
    }
    // Extract layer sizes from list of ints
    let sizes = match list_to_shape(&args[0]) {
        Some(s) if s.len() >= 2 => s,
        _ => return runtime_error("mlp_create", "first argument must be list of at least 2 integers"),
    };
    // Extract activation name
    let activation = match &args[1] {
        Value::Str(s) => match s.as_str() {
            "relu" => Activation::ReLU,
            "sigmoid" => Activation::Sigmoid,
            "tanh" => Activation::Tanh,
            "none" => Activation::None,
            _ => Activation::ReLU,
        },
        _ => Activation::ReLU,
    };
    let mlp = MLP::new(&sizes, activation);
    let model = ModelHandle {
        name: SmolStr::new("mlp"),
        temperature: 0.0,
        max_tokens: 0,
        mlp: Some(mlp),
    };
    Value::ModelVal(Rc::new(RefCell::new(model)))
}

fn native_mlp_forward(args: &mut Vec<Value>) -> Value {
    if args.len() < 2 {
        return runtime_error("mlp_forward", "requires 2 arguments (model, input)");
    }
    match (&args[0], &args[1]) {
        (Value::ModelVal(model), Value::Tensor(input)) => {
            let m = model.borrow();
            match &m.mlp {
                Some(mlp) => {
                    let result = mlp.forward(&input.borrow());
                    Value::Tensor(Rc::new(RefCell::new(result)))
                }
                None => runtime_error("mlp_forward", "model has no MLP"),
            }
        }
        (Value::ModelVal(_), other) => runtime_error("mlp_forward",
            &format!("second argument must be Tensor, got {}", other.type_name())),
        (other, _) => runtime_error("mlp_forward",
            &format!("first argument must be Model, got {}", other.type_name())),
    }
}

fn native_mlp_train_step(args: &mut Vec<Value>) -> Value {
    // Takes model and a list [input_tensor, target_tensor, learning_rate]
    if args.len() < 2 {
        return runtime_error("mlp_train_step", "requires 2 arguments (model, [input, target, lr])");
    }
    match (&args[0], &args[1]) {
        (Value::ModelVal(model), Value::List(params)) => {
            let params_borrowed = params.borrow();
            if params_borrowed.len() < 3 {
                return runtime_error("mlp_train_step", 
                    "second argument must be [input_tensor, target_tensor, learning_rate]");
            }
            
            let input = match &params_borrowed[0] {
                Value::Tensor(t) => t.borrow().clone(),
                _ => return runtime_error("mlp_train_step", "input must be Tensor"),
            };
            let target = match &params_borrowed[1] {
                Value::Tensor(t) => t.borrow().clone(),
                _ => return runtime_error("mlp_train_step", "target must be Tensor"),
            };
            let lr = match &params_borrowed[2] {
                Value::Float(f) => *f as f32,
                Value::Int(i) => *i as f32,
                _ => return runtime_error("mlp_train_step", "learning rate must be Float"),
            };
            
            let m = model.borrow();
            match &m.mlp {
                Some(mlp) => {
                    let mut optimizer = a16_tensor::SGD::new(mlp.parameters(), lr);
                    let loss = a16_tensor::train_step(mlp, &input, &target, &mut optimizer);
                    Value::Float(loss as f64)
                }
                None => runtime_error("mlp_train_step", "model has no MLP"),
            }
        }
        (Value::ModelVal(_), other) => runtime_error("mlp_train_step",
            &format!("second argument must be List, got {}", other.type_name())),
        (other, _) => runtime_error("mlp_train_step",
            &format!("first argument must be Model, got {}", other.type_name())),
    }
}

// ===============================
// M11: Async & Concurrency Function Implementations
// ===============================

fn native_channel_create(args: &mut Vec<Value>) -> Value {
    let capacity = match args.first() {
        Some(Value::Int(n)) => *n as usize,
        _ => 16, // default capacity
    };
    let ch = a16_runtime::Channel::new(SmolStr::new("channel"), capacity);
    Value::Channel(Rc::new(RefCell::new(ch)))
}

fn native_channel_send(args: &mut Vec<Value>) -> Value {
    if args.len() < 2 {
        return runtime_error("channel_send", "requires 2 arguments (channel, value)");
    }
    match &args[0] {
        Value::Channel(ch) => {
            let msg = a16_runtime::ChannelMessage::Text(
                SmolStr::new(format!("{}", args[1]))
            );
            match ch.borrow_mut().send(msg) {
                Ok(()) => Value::Bool(true),
                Err(e) => {
                    eprintln!("channel_send: {}", e);
                    Value::Bool(false)
                }
            }
        }
        other => runtime_error("channel_send",
            &format!("first argument must be Channel, got {}", other.type_name())),
    }
}

fn native_channel_recv(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(Value::Channel(ch)) => {
            match ch.borrow_mut().recv() {
                Ok(msg) => match msg {
                    a16_runtime::ChannelMessage::Text(s) => Value::Str(s),
                    a16_runtime::ChannelMessage::ValueId(_) => Value::None,
                },
                Err(_) => Value::None,
            }
        }
        Some(other) => runtime_error("channel_recv",
            &format!("expected Channel, got {}", other.type_name())),
        None => runtime_error("channel_recv", "missing argument"),
    }
}

fn native_channel_close(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(Value::Channel(ch)) => {
            ch.borrow_mut().close();
            Value::None
        }
        Some(other) => runtime_error("channel_close",
            &format!("expected Channel, got {}", other.type_name())),
        None => runtime_error("channel_close", "missing argument"),
    }
}

fn native_task_status(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(Value::Future(_id)) => {
            // In eager execution mode, all futures are immediately "done"
            Value::Str(SmolStr::new("done"))
        }
        Some(other) => runtime_error("task_status",
            &format!("expected Future, got {}", other.type_name())),
        None => runtime_error("task_status", "missing argument"),
    }
}

fn native_task_cancel(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(Value::Future(_id)) => {
            // In eager execution mode, future is already complete
            Value::Bool(false) // cannot cancel completed task
        }
        Some(other) => runtime_error("task_cancel",
            &format!("expected Future, got {}", other.type_name())),
        None => runtime_error("task_cancel", "missing argument"),
    }
}

fn native_sleep(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(Value::Int(ms)) => {
            // Cooperative sleep: in a real async runtime this would yield.
            // For now, actually sleep (useful for demos).
            if *ms > 0 && *ms < 10000 {
                std::thread::sleep(std::time::Duration::from_millis(*ms as u64));
            }
            Value::None
        }
        Some(Value::Float(secs)) => {
            let ms = (*secs * 1000.0) as u64;
            if ms > 0 && ms < 10000 {
                std::thread::sleep(std::time::Duration::from_millis(ms));
            }
            Value::None
        }
        _ => Value::None,
    }
}

fn native_spawn_fn(args: &mut Vec<Value>) -> Value {
    // Wraps a value as a Future (for use with stdlib, not bytecode)
    match args.first() {
        Some(_val) => {
            // In eager mode, the value is already computed
            // We just wrap it as a "future" that's already resolved
            Value::Future(0) // placeholder ID
        }
        None => Value::None,
    }
}

fn native_await_result(args: &mut Vec<Value>) -> Value {
    // Unwrap a Future value (for use with stdlib, not bytecode)
    match args.first() {
        Some(Value::Future(_id)) => {
            // In eager mode, we'd need access to task_results.
            // Since native funcs don't have VM access, just pass through.
            Value::None
        }
        Some(other) => other.clone(), // pass through non-futures
        None => Value::None,
    }
}

fn native_parallel_exec(args: &mut Vec<Value>) -> Value {
    // Execute a list of values "in parallel" (eagerly, since we're single-threaded)
    match args.first() {
        Some(Value::List(list)) => {
            // In eager mode, the values are already computed
            // Just return them as-is
            Value::List(list.clone())
        }
        Some(other) => runtime_error("parallel_exec",
            &format!("expected List, got {}", other.type_name())),
        None => runtime_error("parallel_exec", "missing argument"),
    }
}

// ===============================
// M12: FFI & Native Extension Function Implementations
// ===============================

fn native_ffi_load(args: &mut Vec<Value>) -> Value {
    if args.len() < 2 {
        return runtime_error("ffi_load", "requires 2 arguments (lib_name, path)");
    }
    match (&args[0], &args[1]) {
        (Value::Str(name), Value::Str(path)) => {
            // In the registered extensions model, we just acknowledge the load request.
            // Actual library loading would happen via the FfiRegistry in the VM.
            println!("[FFI] Registered library '{}' at '{}'", name, path);
            Value::Bool(true)
        }
        (Value::Str(name), Value::None) => {
            println!("[FFI] Registered built-in library '{}'", name);
            Value::Bool(true)
        }
        _ => runtime_error("ffi_load", "expected (Str, Str) or (Str, None)"),
    }
}

fn native_ffi_call(args: &mut Vec<Value>) -> Value {
    if args.len() < 2 {
        return runtime_error("ffi_call", "requires 2 arguments (func_name, args_list)");
    }
    match &args[0] {
        Value::Str(func_name) => {
            // Stub: FFI calls through stdlib require VM access.
            // Real FFI calls go through the FfiCall opcode.
            runtime_error("ffi_call",
                &format!("function '{}' not available via stdlib (use extern blocks)", func_name))
        }
        _ => runtime_error("ffi_call", "first argument must be Str"),
    }
}

fn native_ffi_list(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(Value::Str(lib_name)) => {
            // Stub: would list functions from the FFI registry
            // Returns empty list since we can't access VM state from here
            println!("[FFI] Listing functions in '{}'", lib_name);
            Value::List(Rc::new(RefCell::new(Vec::new())))
        }
        _ => runtime_error("ffi_list", "expected library name (Str)"),
    }
}

fn native_ffi_unload(args: &mut Vec<Value>) -> Value {
    match args.first() {
        Some(Value::Str(lib_name)) => {
            println!("[FFI] Unloaded library '{}'", lib_name);
            Value::Bool(true)
        }
        _ => runtime_error("ffi_unload", "expected library name (Str)"),
    }
}
