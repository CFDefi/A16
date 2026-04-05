//! Runtime Values

use smol_str::SmolStr;
use indexmap::IndexMap;
use std::rc::Rc;
use std::cell::RefCell;
use a16_tensor::Tensor;
use a16_vector::HNSWIndex;
use a16_runtime::Channel;

use crate::agent::{AgentRuntime, ModelHandle};
use crate::tool_registry::ToolDef;

/// Runtime value
#[derive(Debug, Clone)]
pub enum Value {
    None,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(SmolStr),
    List(Rc<RefCell<Vec<Value>>>),
    Dict(Rc<RefCell<IndexMap<SmolStr, Value>>>),
    Tuple(Rc<Vec<Value>>),
    Function(u16), // Index into function table
    NativeFunc(NativeFunc),
    Iterator(Box<ValueIterator>),
    // AI Runtime types
    Tensor(Rc<RefCell<Tensor>>),
    Vector(Rc<Vec<f32>>),
    MemoryIndex(Rc<RefCell<HNSWIndex>>),
    // M8: Agent & Model types
    Agent(Rc<RefCell<AgentRuntime>>),
    ModelVal(Rc<RefCell<ModelHandle>>),
    ToolHandle(Rc<ToolDef>),
    // M9: Closure
    Closure { func_idx: u16, upvalues: Vec<Value> },
    // M11: Async primitives
    Future(u64),  // TaskId from executor
    Channel(Rc<RefCell<Channel>>),
}

/// Native function wrapper
#[derive(Debug, Clone)]
pub struct NativeFunc {
    pub name: SmolStr,
    pub arity: u8,
    pub func: fn(&mut Vec<Value>) -> Value,
}

/// Iterator state
#[derive(Debug, Clone)]
pub struct ValueIterator {
    values: Vec<Value>,
    index: usize,
}

impl ValueIterator {
    pub fn new(values: Vec<Value>) -> Self {
        Self { values, index: 0 }
    }
    
    pub fn next(&mut self) -> Option<Value> {
        if self.index < self.values.len() {
            let val = self.values[self.index].clone();
            self.index += 1;
            Some(val)
        } else {
            None
        }
    }
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::None => false,
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Float(n) => *n != 0.0,
            Value::Str(s) => !s.is_empty(),
            Value::List(list) => !list.borrow().is_empty(),
            Value::Dict(dict) => !dict.borrow().is_empty(),
            Value::Tuple(tuple) => !tuple.is_empty(),
            Value::Function(_) => true,
            Value::NativeFunc(_) => true,
            Value::Iterator(_) => true,
            Value::Tensor(_) => true,
            Value::Vector(v) => !v.is_empty(),
            Value::MemoryIndex(_) => true,
            Value::Agent(_) => true,
            Value::ModelVal(_) => true,
            Value::ToolHandle(_) => true,
            Value::Closure { .. } => true,
            Value::Future(_) => true,
            Value::Channel(_) => true,
        }
    }
    
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::None => "None",
            Value::Bool(_) => "Bool",
            Value::Int(_) => "Int",
            Value::Float(_) => "Float",
            Value::Str(_) => "Str",
            Value::List(_) => "List",
            Value::Dict(_) => "Dict",
            Value::Tuple(_) => "Tuple",
            Value::Function(_) => "Function",
            Value::NativeFunc(_) => "NativeFunc",
            Value::Iterator(_) => "Iterator",
            Value::Tensor(_) => "Tensor",
            Value::Vector(_) => "Vector",
            Value::MemoryIndex(_) => "MemoryIndex",
            Value::Agent(_) => "Agent",
            Value::ModelVal(_) => "Model",
            Value::ToolHandle(_) => "Tool",
            Value::Closure { .. } => "Closure",
            Value::Future(_) => "Future",
            Value::Channel(_) => "Channel",
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::None, Value::None) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Int(a), Value::Float(b)) => (*a as f64) == *b,
            (Value::Float(a), Value::Int(b)) => *a == (*b as f64),
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Future(a), Value::Future(b)) => a == b,
            _ => false,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::None => write!(f, "None"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::Str(s) => write!(f, "{}", s),
            Value::List(list) => {
                write!(f, "[")?;
                let borrowed = list.borrow();
                for (i, v) in borrowed.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Dict(dict) => {
                write!(f, "{{")?;
                let borrowed = dict.borrow();
                for (i, (k, v)) in borrowed.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "\"{}\": {}", k, v)?;
                }
                write!(f, "}}")
            }
            Value::Tuple(tuple) => {
                write!(f, "(")?;
                for (i, v) in tuple.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", v)?;
                }
                write!(f, ")")
            }
            Value::Function(idx) => write!(f, "<function {}>", idx),
            Value::NativeFunc(nf) => write!(f, "<native {}>", nf.name),
            Value::Iterator(_) => write!(f, "<iterator>"),
            Value::Tensor(t) => write!(f, "<Tensor shape={:?}>", t.borrow().shape()),
            Value::Vector(v) => write!(f, "<Vector len={}>", v.len()),
            Value::MemoryIndex(idx) => write!(f, "<MemoryIndex len={}>", idx.borrow().len()),
            Value::Agent(a) => write!(f, "<Agent '{}'>", a.borrow().name),
            Value::ModelVal(m) => write!(f, "<Model '{}'>", m.borrow().name),
            Value::ToolHandle(t) => write!(f, "<Tool '{}'>", t.name),
            Value::Closure { func_idx, upvalues } => write!(f, "<Closure func={} upvalues={}>", func_idx, upvalues.len()),
            Value::Future(id) => write!(f, "<Future task={}>", id),
            Value::Channel(ch) => write!(f, "<Channel '{}'>", ch.borrow().name),
        }
    }
}
