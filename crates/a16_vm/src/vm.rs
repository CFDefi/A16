use a16_codegen::{BytecodeModule, Constant, Opcode};
use smol_str::SmolStr;
use indexmap::IndexMap;
use std::rc::Rc;
use std::cell::RefCell;
use a16_runtime::{Executor, Channel};
use a16_ffi::FfiRegistry;

use crate::value::{Value, ValueIterator};

/// Call frame for function execution
#[derive(Debug)]
struct CallFrame {
    func_idx: u16,
    ip: usize,
    base: usize,
    /// Captured upvalues for closures (empty for regular functions)
    upvalues: Vec<Value>,
}

/// A16 Virtual Machine
pub struct VM {
    stack: Vec<Value>,
    frames: Vec<CallFrame>,
    globals: IndexMap<SmolStr, Value>,
    module: BytecodeModule,
    /// Async task executor
    executor: Executor,
    /// Stored results for eagerly-evaluated spawned tasks
    task_results: IndexMap<u64, Value>,
    /// Next task result ID
    next_task_id: u64,
    /// FFI function registry
    ffi_registry: FfiRegistry,
}

/// VM execution result
pub type VMResult<T> = Result<T, VMError>;

/// VM error type
#[derive(Debug, Clone)]
pub enum VMError {
    StackUnderflow,
    InvalidOpcode(u8),
    TypeError(String),
    DivisionByZero,
    IndexOutOfBounds,
    UndefinedVariable(String),
    NotCallable,
    WrongArgCount { expected: u8, got: u8 },
}

impl std::fmt::Display for VMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VMError::StackUnderflow => write!(f, "Stack underflow"),
            VMError::InvalidOpcode(op) => write!(f, "Invalid opcode: 0x{:02X}", op),
            VMError::TypeError(msg) => write!(f, "Type error: {}", msg),
            VMError::DivisionByZero => write!(f, "Division by zero"),
            VMError::IndexOutOfBounds => write!(f, "Index out of bounds"),
            VMError::UndefinedVariable(name) => write!(f, "Undefined variable: {}", name),
            VMError::NotCallable => write!(f, "Value is not callable"),
            VMError::WrongArgCount { expected, got } => {
                write!(f, "Wrong number of arguments: expected {}, got {}", expected, got)
            }
        }
    }
}

impl std::error::Error for VMError {}

impl VM {
    /// Create a new VM with bytecode
    pub fn new(module: BytecodeModule) -> Self {
        let mut globals = crate::stdlib::register_stdlib();
        
        // Register all user-defined functions as globals
        for (idx, func) in module.functions.iter().enumerate() {
            globals.insert(func.name.clone(), Value::Function(idx as u16));
        }
        
        Self {
            stack: Vec::with_capacity(1024),
            frames: Vec::with_capacity(64),
            globals,
            module,
            executor: Executor::new(),
            task_results: IndexMap::new(),
            next_task_id: 1,
            ffi_registry: FfiRegistry::new(),
        }
    }
    
    /// Run the VM starting from a function by name
    pub fn run(&mut self, func_name: &str) -> VMResult<Value> {
        // Find function
        let func_idx = self.module.functions.iter()
            .position(|f| f.name.as_str() == func_name)
            .ok_or_else(|| VMError::UndefinedVariable(func_name.to_string()))?;
        
        self.call_function(func_idx as u16, 0)?;
        self.execute()
    }
    
    fn call_function(&mut self, func_idx: u16, argc: u8) -> VMResult<()> {
        self.call_function_with_upvalues(func_idx, argc, Vec::new())
    }
    
    /// Call a function with arguments already on stack, with optional upvalues
    fn call_function_with_upvalues(&mut self, func_idx: u16, argc: u8, upvalues: Vec<Value>) -> VMResult<()> {
        let func = &self.module.functions[func_idx as usize];
        
        if argc != func.arity {
            return Err(VMError::WrongArgCount {
                expected: func.arity,
                got: argc,
            });
        }
        
        // Base points to first local (arguments)
        let base = self.stack.len() - argc as usize;
        
        // Allocate space for remaining locals
        let extra_locals = func.locals.saturating_sub(argc);
        for _ in 0..extra_locals {
            self.stack.push(Value::None);
        }
        
        self.frames.push(CallFrame {
            func_idx,
            ip: 0,
            base,
            upvalues,
        });
        
        Ok(())
    }
    
    /// Main execution loop
    fn execute(&mut self) -> VMResult<Value> {
        loop {
            if self.frames.is_empty() {
                return Ok(self.stack.pop().unwrap_or(Value::None));
            }
            
            let frame = self.frames.last_mut().unwrap();
            let func = &self.module.functions[frame.func_idx as usize];
            
            if frame.ip >= func.code.len() {
                // Implicit return None
                self.stack.push(Value::None);
                self.frames.pop();
                continue;
            }
            
            let opcode = func.code[frame.ip];
            frame.ip += 1;
            
            match Opcode::from_u8(opcode) {
                Some(Opcode::Nop) => {}
                
                Some(Opcode::Pop) => {
                    self.stack.pop();
                }
                
                Some(Opcode::Dup) => {
                    let val = self.stack.last().cloned().ok_or(VMError::StackUnderflow)?;
                    self.stack.push(val);
                }
                
                Some(Opcode::Swap) => {
                    let len = self.stack.len();
                    if len < 2 {
                        return Err(VMError::StackUnderflow);
                    }
                    self.stack.swap(len - 1, len - 2);
                }
                
                Some(Opcode::PushConst) => {
                    let idx = self.read_u16()?;
                    let val = self.constant_to_value(idx);
                    self.stack.push(val);
                }
                
                Some(Opcode::PushTrue) => self.stack.push(Value::Bool(true)),
                Some(Opcode::PushFalse) => self.stack.push(Value::Bool(false)),
                Some(Opcode::PushNone) => self.stack.push(Value::None),
                Some(Opcode::PushInt0) => self.stack.push(Value::Int(0)),
                Some(Opcode::PushInt1) => self.stack.push(Value::Int(1)),
                
                Some(Opcode::LoadLocal) => {
                    let slot = self.read_u8()?;
                    let frame = self.frames.last().unwrap();
                    let val = self.stack[frame.base + slot as usize].clone();
                    self.stack.push(val);
                }
                
                Some(Opcode::StoreLocal) => {
                    let slot = self.read_u8()?;
                    let val = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let frame = self.frames.last().unwrap();
                    self.stack[frame.base + slot as usize] = val;
                }
                
                Some(Opcode::LoadGlobal) => {
                    let idx = self.read_u16()?;
                    let name = &self.module.globals[idx as usize];
                    let val = self.globals.get(name).cloned()
                        .ok_or_else(|| VMError::UndefinedVariable(name.to_string()))?;
                    self.stack.push(val);
                }
                
                Some(Opcode::StoreGlobal) => {
                    let idx = self.read_u16()?;
                    let val = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let name = self.module.globals[idx as usize].clone();
                    self.globals.insert(name, val);
                }
                
                Some(Opcode::LoadGlobalByName) => {
                    let idx = self.read_u16()?;
                    let name = self.get_string_constant(idx);
                    let val = self.globals.get(&name).cloned()
                        .ok_or_else(|| VMError::UndefinedVariable(name.to_string()))?;
                    self.stack.push(val);
                }
                
                // Arithmetic
                Some(Opcode::Add) => self.binary_op(|a, b| match (a, b) {
                    (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x + y)),
                    (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x + y)),
                    (Value::Int(x), Value::Float(y)) => Ok(Value::Float(x as f64 + y)),
                    (Value::Float(x), Value::Int(y)) => Ok(Value::Float(x + y as f64)),
                    (Value::Str(x), Value::Str(y)) => Ok(Value::Str(SmolStr::new(format!("{}{}", x, y)))),
                    _ => Err(VMError::TypeError("Cannot add these types".into())),
                })?,
                
                Some(Opcode::Sub) => self.binary_op(|a, b| match (a, b) {
                    (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x - y)),
                    (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x - y)),
                    (Value::Int(x), Value::Float(y)) => Ok(Value::Float(x as f64 - y)),
                    (Value::Float(x), Value::Int(y)) => Ok(Value::Float(x - y as f64)),
                    _ => Err(VMError::TypeError("Cannot subtract these types".into())),
                })?,
                
                Some(Opcode::Mul) => self.binary_op(|a, b| match (a, b) {
                    (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x * y)),
                    (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x * y)),
                    (Value::Int(x), Value::Float(y)) => Ok(Value::Float(x as f64 * y)),
                    (Value::Float(x), Value::Int(y)) => Ok(Value::Float(x * y as f64)),
                    _ => Err(VMError::TypeError("Cannot multiply these types".into())),
                })?,
                
                Some(Opcode::Div) => self.binary_op(|a, b| match (a, b) {
                    (Value::Int(x), Value::Int(y)) if y != 0 => Ok(Value::Float(x as f64 / y as f64)),
                    (Value::Float(x), Value::Float(y)) if y != 0.0 => Ok(Value::Float(x / y)),
                    (Value::Int(x), Value::Float(y)) if y != 0.0 => Ok(Value::Float(x as f64 / y)),
                    (Value::Float(x), Value::Int(y)) if y != 0 => Ok(Value::Float(x / y as f64)),
                    _ => Err(VMError::DivisionByZero),
                })?,
                
                Some(Opcode::FloorDiv) => self.binary_op(|a, b| match (a, b) {
                    (Value::Int(x), Value::Int(y)) if y != 0 => Ok(Value::Int(x / y)),
                    _ => Err(VMError::DivisionByZero),
                })?,
                
                Some(Opcode::Mod) => self.binary_op(|a, b| match (a, b) {
                    (Value::Int(x), Value::Int(y)) if y != 0 => Ok(Value::Int(x % y)),
                    _ => Err(VMError::DivisionByZero),
                })?,
                
                Some(Opcode::Pow) => self.binary_op(|a, b| match (a, b) {
                    (Value::Int(x), Value::Int(y)) if y >= 0 => Ok(Value::Int(x.pow(y as u32))),
                    (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x.powf(y))),
                    _ => Err(VMError::TypeError("Invalid power operation".into())),
                })?,
                
                Some(Opcode::Neg) => {
                    let val = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let result = match val {
                        Value::Int(n) => Value::Int(-n),
                        Value::Float(n) => Value::Float(-n),
                        _ => return Err(VMError::TypeError("Cannot negate".into())),
                    };
                    self.stack.push(result);
                }
                
                // Bitwise
                Some(Opcode::BitAnd) => self.binary_op(|a, b| match (a, b) {
                    (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x & y)),
                    _ => Err(VMError::TypeError("Bitwise AND requires integers".into())),
                })?,
                
                Some(Opcode::BitOr) => self.binary_op(|a, b| match (a, b) {
                    (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x | y)),
                    _ => Err(VMError::TypeError("Bitwise OR requires integers".into())),
                })?,
                
                Some(Opcode::BitXor) => self.binary_op(|a, b| match (a, b) {
                    (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x ^ y)),
                    _ => Err(VMError::TypeError("Bitwise XOR requires integers".into())),
                })?,
                
                Some(Opcode::BitNot) => {
                    let val = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let result = match val {
                        Value::Int(n) => Value::Int(!n),
                        _ => return Err(VMError::TypeError("Bitwise NOT requires integer".into())),
                    };
                    self.stack.push(result);
                }
                
                Some(Opcode::Shl) => self.binary_op(|a, b| match (a, b) {
                    (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x << y)),
                    _ => Err(VMError::TypeError("Shift requires integers".into())),
                })?,
                
                Some(Opcode::Shr) => self.binary_op(|a, b| match (a, b) {
                    (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x >> y)),
                    _ => Err(VMError::TypeError("Shift requires integers".into())),
                })?,
                
                // Comparison
                Some(Opcode::Eq) => self.binary_op(|a, b| Ok(Value::Bool(a == b)))?,
                Some(Opcode::Ne) => self.binary_op(|a, b| Ok(Value::Bool(a != b)))?,
                
                Some(Opcode::Lt) => self.compare_op(|x, y| x < y)?,
                Some(Opcode::Le) => self.compare_op(|x, y| x <= y)?,
                Some(Opcode::Gt) => self.compare_op(|x, y| x > y)?,
                Some(Opcode::Ge) => self.compare_op(|x, y| x >= y)?,
                
                Some(Opcode::Not) => {
                    let val = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    self.stack.push(Value::Bool(!val.is_truthy()));
                }
                
                // Control flow
                Some(Opcode::Jump) => {
                    let offset = self.read_i16()?;
                    let frame = self.frames.last_mut().unwrap();
                    frame.ip = (frame.ip as i32 + offset as i32) as usize;
                }
                
                Some(Opcode::JumpIfTrue) => {
                    let offset = self.read_i16()?;
                    let val = self.stack.last().ok_or(VMError::StackUnderflow)?;
                    if val.is_truthy() {
                        let frame = self.frames.last_mut().unwrap();
                        frame.ip = (frame.ip as i32 + offset as i32) as usize;
                    }
                }
                
                Some(Opcode::JumpIfFalse) => {
                    let offset = self.read_i16()?;
                    let val = self.stack.last().ok_or(VMError::StackUnderflow)?;
                    if !val.is_truthy() {
                        let frame = self.frames.last_mut().unwrap();
                        frame.ip = (frame.ip as i32 + offset as i32) as usize;
                    }
                }
                
                // Functions
                Some(Opcode::Call) => {
                    let argc = self.read_u8()?;
                    let callee = self.stack[self.stack.len() - 1 - argc as usize].clone();
                    
                    match callee {
                        Value::Function(idx) => {
                            // Remove callee from stack
                            self.stack.remove(self.stack.len() - 1 - argc as usize);
                            self.call_function(idx, argc)?;
                        }
                        Value::Closure { func_idx, upvalues } => {
                            // Remove callee from stack
                            self.stack.remove(self.stack.len() - 1 - argc as usize);
                            self.call_function_with_upvalues(func_idx, argc, upvalues)?;
                        }
                        Value::NativeFunc(nf) => {
                            if argc != nf.arity {
                                return Err(VMError::WrongArgCount {
                                    expected: nf.arity,
                                    got: argc,
                                });
                            }
                            // Collect args
                            let mut args: Vec<Value> = self.stack.drain(self.stack.len() - argc as usize..).collect();
                            self.stack.pop(); // Remove callee
                            let result = (nf.func)(&mut args);
                            self.stack.push(result);
                        }
                        _ => return Err(VMError::NotCallable),
                    }
                }
                
                Some(Opcode::Return) => {
                    let result = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let frame = self.frames.pop().unwrap();
                    
                    // Clean up locals
                    let _func = &self.module.functions[frame.func_idx as usize];
                    self.stack.truncate(frame.base);
                    
                    self.stack.push(result);
                }
                
                // Objects
                Some(Opcode::GetAttr) => {
                    let idx = self.read_u16()?;
                    let name = self.get_string_constant(idx);
                    let obj = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    
                    let val = match obj {
                        Value::Dict(dict) => dict.borrow().get(&name).cloned().unwrap_or(Value::None),
                        _ => return Err(VMError::TypeError("Cannot get attribute".into())),
                    };
                    self.stack.push(val);
                }
                
                Some(Opcode::SetAttr) => {
                    let idx = self.read_u16()?;
                    let name = self.get_string_constant(idx);
                    let obj = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let val = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    
                    match obj {
                        Value::Dict(dict) => {
                            dict.borrow_mut().insert(name, val);
                        }
                        _ => return Err(VMError::TypeError("Cannot set attribute".into())),
                    }
                }
                
                Some(Opcode::GetIndex) => {
                    let index = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let obj = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    
                    let val = match (obj, index) {
                        (Value::List(list), Value::Int(i)) => {
                            let borrowed = list.borrow();
                            let idx = if i < 0 { (borrowed.len() as i64 + i) as usize } else { i as usize };
                            borrowed.get(idx).cloned().ok_or(VMError::IndexOutOfBounds)?
                        }
                        (Value::Dict(dict), Value::Str(key)) => {
                            dict.borrow().get(&key).cloned().unwrap_or(Value::None)
                        }
                        (Value::Tuple(tuple), Value::Int(i)) => {
                            let idx = if i < 0 { (tuple.len() as i64 + i) as usize } else { i as usize };
                            tuple.get(idx).cloned().ok_or(VMError::IndexOutOfBounds)?
                        }
                        _ => return Err(VMError::TypeError("Invalid index operation".into())),
                    };
                    self.stack.push(val);
                }
                
                Some(Opcode::SetIndex) => {
                    let val = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let index = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let obj = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    
                    match (obj, index) {
                        (Value::List(list), Value::Int(i)) => {
                            let mut borrowed = list.borrow_mut();
                            let idx = if i < 0 { (borrowed.len() as i64 + i) as usize } else { i as usize };
                            if idx < borrowed.len() {
                                borrowed[idx] = val;
                            } else {
                                return Err(VMError::IndexOutOfBounds);
                            }
                        }
                        (Value::Dict(dict), Value::Str(key)) => {
                            dict.borrow_mut().insert(key, val);
                        }
                        _ => return Err(VMError::TypeError("Invalid index operation".into())),
                    }
                }
                
                // Constructors
                Some(Opcode::BuildList) => {
                    let count = self.read_u16()? as usize;
                    let start = self.stack.len() - count;
                    let elements: Vec<Value> = self.stack.drain(start..).collect();
                    self.stack.push(Value::List(Rc::new(RefCell::new(elements))));
                }
                
                Some(Opcode::BuildDict) => {
                    let count = self.read_u16()? as usize;
                    let start = self.stack.len() - count * 2;
                    let pairs: Vec<Value> = self.stack.drain(start..).collect();
                    let mut dict = IndexMap::new();
                    for chunk in pairs.chunks(2) {
                        if let [Value::Str(key), value] = chunk {
                            dict.insert(key.clone(), value.clone());
                        }
                    }
                    self.stack.push(Value::Dict(Rc::new(RefCell::new(dict))));
                }
                
                Some(Opcode::BuildTuple) => {
                    let count = self.read_u16()? as usize;
                    let start = self.stack.len() - count;
                    let elements: Vec<Value> = self.stack.drain(start..).collect();
                    self.stack.push(Value::Tuple(Rc::new(elements)));
                }
                
                // Iteration
                Some(Opcode::GetIter) => {
                    let val = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let iter = match val {
                        Value::List(list) => ValueIterator::new(list.borrow().clone()),
                        Value::Tuple(tuple) => ValueIterator::new(tuple.as_ref().clone()),
                        _ => return Err(VMError::TypeError("Not iterable".into())),
                    };
                    self.stack.push(Value::Iterator(Box::new(iter)));
                }
                
                Some(Opcode::ForIter) => {
                    let offset = self.read_i16()?;
                    
                    // Iterator should be at top of stack
                    let iter = self.stack.last_mut().ok_or(VMError::StackUnderflow)?;
                    
                    if let Value::Iterator(ref mut it) = iter {
                        if let Some(val) = it.next() {
                            self.stack.push(val);
                        } else {
                            // Jump to end
                            let frame = self.frames.last_mut().unwrap();
                            frame.ip = (frame.ip as i32 + offset as i32) as usize;
                        }
                    } else {
                        return Err(VMError::TypeError("Expected iterator".into()));
                    }
                }
                
                // AI Operations (stubs for now)
                Some(Opcode::ModelInvoke) => {
                    let _prompt = self.stack.pop();
                    let _model = self.stack.pop();
                    // TODO: Integrate with actual model API
                    self.stack.push(Value::Str(SmolStr::new("[model response]")));
                }
                
                Some(Opcode::ToolDispatch) => {
                    let _tool = self.stack.pop();
                    // TODO: Integrate with tool sandbox
                    self.stack.push(Value::Str(SmolStr::new("[tool result]")));
                }
                
                Some(Opcode::MemoryStore) => {
                    let _content = self.stack.pop();
                    let _memory = self.stack.pop();
                    // TODO: Integrate with memory store
                    self.stack.push(Value::Str(SmolStr::new("[stored]")));
                }
                
                Some(Opcode::MemoryRetrieve) => {
                    let _query = self.stack.pop();
                    let _memory = self.stack.pop();
                    // TODO: Integrate with memory retrieval
                    self.stack.push(Value::List(Rc::new(RefCell::new(vec![]))));
                }
                
                // Async Runtime
                Some(Opcode::Await) => {
                    // Resolve a Future: if TOS is Future(id), replace with stored result
                    let val = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    match val {
                        Value::Future(task_id) => {
                            // Look up the eagerly-computed result
                            let result = self.task_results.get(&task_id)
                                .cloned()
                                .unwrap_or(Value::None);
                            self.stack.push(result);
                        }
                        other => {
                            // Not a future — pass through (already resolved)
                            self.stack.push(other);
                        }
                    }
                }
                
                Some(Opcode::Spawn) => {
                    // Legacy spawn: eagerly execute the function call on TOS
                    // The callee should already be on the stack
                    let val = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    match val {
                        Value::Function(idx) => {
                            // Eagerly execute: call function, get result, wrap in Future
                            let task_id = self.next_task_id;
                            self.next_task_id += 1;
                            
                            // Execute synchronously (simplified eager model)
                            self.call_function(idx, 0)?;
                            let result = self.execute_to_return()?;
                            self.task_results.insert(task_id, result);
                            self.stack.push(Value::Future(task_id));
                        }
                        other => {
                            // Not callable — just wrap the value
                            let task_id = self.next_task_id;
                            self.next_task_id += 1;
                            self.task_results.insert(task_id, other);
                            self.stack.push(Value::Future(task_id));
                        }
                    }
                }
                
                Some(Opcode::SpawnTask) => {
                    // Spawn a function as a concurrent task
                    // Reads: u16 func_idx, u8 argc
                    let func_idx = self.read_u16()?;
                    let argc = self.read_u8()?;
                    
                    let task_id = self.next_task_id;
                    self.next_task_id += 1;
                    
                    // Collect arguments from stack
                    let args: Vec<Value> = if argc > 0 {
                        self.stack.drain(self.stack.len() - argc as usize..).collect()
                    } else {
                        Vec::new()
                    };
                    
                    // Push args back, call function, execute eagerly
                    for arg in args {
                        self.stack.push(arg);
                    }
                    self.call_function(func_idx, argc)?;
                    let result = self.execute_to_return()?;
                    self.task_results.insert(task_id, result);
                    
                    // Also register with executor for tracking
                    self.executor.spawn(SmolStr::new(format!("task_{}", task_id)), func_idx);
                    
                    self.stack.push(Value::Future(task_id));
                }
                
                Some(Opcode::JoinAll) => {
                    // Wait for N futures, push results as a list
                    let count = self.read_u8()? as usize;
                    let start = self.stack.len() - count;
                    let futures: Vec<Value> = self.stack.drain(start..).collect();
                    
                    let mut results = Vec::with_capacity(count);
                    for future in futures {
                        match future {
                            Value::Future(task_id) => {
                                let result = self.task_results.get(&task_id)
                                    .cloned()
                                    .unwrap_or(Value::None);
                                results.push(result);
                            }
                            other => results.push(other),
                        }
                    }
                    
                    self.stack.push(Value::List(Rc::new(RefCell::new(results))));
                }
                
                Some(Opcode::Yield) => {
                    // Cooperative yield point — no-op in single-task mode
                    // In a full async runtime, this would suspend the current task
                }
                
                Some(Opcode::ChannelCreate) => {
                    // Create a bounded channel
                    let capacity = self.read_u16()? as usize;
                    let ch_id = self.next_task_id;
                    self.next_task_id += 1;
                    let ch = Channel::new(
                        SmolStr::new(format!("ch_{}", ch_id)),
                        capacity,
                    );
                    self.stack.push(Value::Channel(Rc::new(RefCell::new(ch))));
                }
                
                Some(Opcode::ChannelSend) => {
                    // Stack: [channel, value] -> []
                    let val = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let ch_val = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    
                    match ch_val {
                        Value::Channel(ch) => {
                            let msg = a16_runtime::ChannelMessage::Text(
                                SmolStr::new(format!("{}", val))
                            );
                            ch.borrow_mut().send(msg)
                                .map_err(|e| VMError::TypeError(format!("Channel send failed: {}", e)))?;
                            self.stack.push(Value::None);
                        }
                        _ => return Err(VMError::TypeError("Expected channel".into())),
                    }
                }
                
                Some(Opcode::ChannelRecv) => {
                    // Stack: [channel] -> [value]
                    let ch_val = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    
                    match ch_val {
                        Value::Channel(ch) => {
                            match ch.borrow_mut().recv() {
                                Ok(msg) => {
                                    let val = match msg {
                                        a16_runtime::ChannelMessage::Text(s) => Value::Str(s),
                                        a16_runtime::ChannelMessage::ValueId(id) => {
                                            self.task_results.get(&id)
                                                .cloned()
                                                .unwrap_or(Value::None)
                                        }
                                    };
                                    self.stack.push(val);
                                }
                                Err(_) => {
                                    self.stack.push(Value::None);
                                }
                            }
                        }
                        _ => return Err(VMError::TypeError("Expected channel".into())),
                    }
                }
                
                // FFI
                Some(Opcode::FfiCall) => {
                    let ffi_idx = self.read_u16()?;
                    let argc = self.read_u8()?;
                    
                    // Collect arguments from stack
                    let start = self.stack.len() - argc as usize;
                    let args: Vec<Value> = self.stack.drain(start..).collect();
                    
                    // Marshal args to FFI values
                    let ffi_args: Vec<a16_ffi::FfiValue> = args.iter().map(|v| match v {
                        Value::Int(n) => a16_ffi::FfiValue::Int(*n),
                        Value::Float(f) => a16_ffi::FfiValue::Float(*f),
                        Value::Bool(b) => a16_ffi::FfiValue::Bool(*b),
                        Value::Str(s) => a16_ffi::FfiValue::Str(s.clone()),
                        Value::None => a16_ffi::FfiValue::Void,
                        _ => a16_ffi::FfiValue::Str(SmolStr::new(format!("{}", v))),
                    }).collect();
                    
                    // Call through registry
                    match self.ffi_registry.call_by_index(ffi_idx, &ffi_args) {
                        Ok(result) => {
                            let val = match result {
                                a16_ffi::FfiValue::Int(n) => Value::Int(n),
                                a16_ffi::FfiValue::UInt(n) => Value::Int(n as i64),
                                a16_ffi::FfiValue::Float(f) => Value::Float(f),
                                a16_ffi::FfiValue::Bool(b) => Value::Bool(b),
                                a16_ffi::FfiValue::Str(s) => Value::Str(s),
                                a16_ffi::FfiValue::Void => Value::None,
                                a16_ffi::FfiValue::Pointer(p) => Value::Int(p as i64),
                            };
                            self.stack.push(val);
                        }
                        Err(e) => {
                            return Err(VMError::TypeError(format!("FFI call failed: {}", e)));
                        }
                    }
                }
                
                Some(Opcode::FfiLoad) => {
                    let lib_name_idx = self.read_u16()?;
                    let lib_name = self.get_string_constant(lib_name_idx);
                    self.ffi_registry.register_library(lib_name.as_str(), None)
                        .map_err(|e| VMError::TypeError(format!("FFI load failed: {}", e)))?;
                    self.stack.push(Value::Bool(true));
                }
                
                // Closures
                Some(Opcode::MakeClosure) => {
                    let func_idx = self.read_u16()?;
                    let upvalue_count = self.read_u8()?;
                    let mut upvalues = Vec::with_capacity(upvalue_count as usize);
                    
                    for _ in 0..upvalue_count {
                        let is_local = self.read_u8()? != 0;
                        let index = self.read_u8()?;
                        
                        let val = if is_local {
                            // Capture from current frame's locals
                            let frame = self.frames.last().unwrap();
                            self.stack[frame.base + index as usize].clone()
                        } else {
                            // Capture from enclosing closure's upvalues
                            let frame = self.frames.last().unwrap();
                            frame.upvalues.get(index as usize).cloned().unwrap_or(Value::None)
                        };
                        upvalues.push(val);
                    }
                    
                    self.stack.push(Value::Closure { func_idx, upvalues });
                }
                
                Some(Opcode::GetUpvalue) => {
                    let index = self.read_u8()?;
                    let frame = self.frames.last().unwrap();
                    let val = frame.upvalues.get(index as usize)
                        .cloned()
                        .unwrap_or(Value::None);
                    self.stack.push(val);
                }
                
                Some(Opcode::SetUpvalue) => {
                    let index = self.read_u8()?;
                    let val = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let frame = self.frames.last_mut().unwrap();
                    if (index as usize) < frame.upvalues.len() {
                        frame.upvalues[index as usize] = val;
                    }
                }
                
                Some(Opcode::Halt) => {
                    return Ok(self.stack.pop().unwrap_or(Value::None));
                }
                
                None => return Err(VMError::InvalidOpcode(opcode)),
            }
        }
    }
    /// Execute until the current call frame returns.
    /// Used by spawn/spawn_task for eager execution of tasks.
    fn execute_to_return(&mut self) -> VMResult<Value> {
        let target_depth = self.frames.len() - 1;
        loop {
            if self.frames.len() <= target_depth {
                // The spawned frame has returned
                return Ok(self.stack.pop().unwrap_or(Value::None));
            }

            let frame = self.frames.last_mut().unwrap();
            let func = &self.module.functions[frame.func_idx as usize];

            if frame.ip >= func.code.len() {
                // Implicit return None
                self.stack.push(Value::None);
                let old_frame = self.frames.pop().unwrap();
                self.stack.truncate(old_frame.base);
                self.stack.push(Value::None);
                continue;
            }

            let opcode = func.code[frame.ip];
            frame.ip += 1;

            // Handle Return specially to check depth
            if let Some(Opcode::Return) = Opcode::from_u8(opcode) {
                let result = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                let old_frame = self.frames.pop().unwrap();
                self.stack.truncate(old_frame.base);
                self.stack.push(result.clone());

                if self.frames.len() <= target_depth {
                    return Ok(result);
                }
                continue;
            }

            // Re-process through execute. We need to put IP back and call execute once.
            // Simpler approach: just run the full execute loop but break on depth check
            let frame = self.frames.last_mut().unwrap();
            frame.ip -= 1; // put opcode back
            
            // Single-step: execute one instruction via the main loop
            // We re-enter execute which will process one opcode and return here
            break;
        }

        // Fallback: run the full execute loop  
        self.execute()
    }

    fn read_u8(&mut self) -> VMResult<u8> {
        let frame = self.frames.last_mut().unwrap();
        let func = &self.module.functions[frame.func_idx as usize];
        let val = func.code[frame.ip];
        frame.ip += 1;
        Ok(val)
    }
    
    fn read_u16(&mut self) -> VMResult<u16> {
        let high = self.read_u8()? as u16;
        let low = self.read_u8()? as u16;
        Ok((high << 8) | low)
    }
    
    fn read_i16(&mut self) -> VMResult<i16> {
        Ok(self.read_u16()? as i16)
    }
    
    fn constant_to_value(&self, idx: u16) -> Value {
        match &self.module.constants[idx as usize] {
            Constant::Int(n) => Value::Int(*n),
            Constant::Float(n) => Value::Float(*n),
            Constant::Str(s) => Value::Str(s.clone()),
            Constant::Bool(b) => Value::Bool(*b),
            Constant::None => Value::None,
        }
    }
    
    fn get_string_constant(&self, idx: u16) -> SmolStr {
        match &self.module.constants[idx as usize] {
            Constant::Str(s) => s.clone(),
            _ => SmolStr::new(""),
        }
    }
    
    fn binary_op<F>(&mut self, f: F) -> VMResult<()>
    where
        F: FnOnce(Value, Value) -> VMResult<Value>,
    {
        let b = self.stack.pop().ok_or(VMError::StackUnderflow)?;
        let a = self.stack.pop().ok_or(VMError::StackUnderflow)?;
        let result = f(a, b)?;
        self.stack.push(result);
        Ok(())
    }
    
    fn compare_op<F>(&mut self, f: F) -> VMResult<()>
    where
        F: FnOnce(f64, f64) -> bool,
    {
        let b = self.stack.pop().ok_or(VMError::StackUnderflow)?;
        let a = self.stack.pop().ok_or(VMError::StackUnderflow)?;
        
        let result = match (a, b) {
            (Value::Int(x), Value::Int(y)) => f(x as f64, y as f64),
            (Value::Float(x), Value::Float(y)) => f(x, y),
            (Value::Int(x), Value::Float(y)) => f(x as f64, y),
            (Value::Float(x), Value::Int(y)) => f(x, y as f64),
            _ => return Err(VMError::TypeError("Cannot compare these types".into())),
        };
        
        self.stack.push(Value::Bool(result));
        Ok(())
    }
}
