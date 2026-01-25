//! Standard Library - Native Functions

use smol_str::SmolStr;
use std::rc::Rc;
use std::cell::RefCell;
use indexmap::IndexMap;

use crate::value::{Value, NativeFunc};
use a16_tensor::Tensor;
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

