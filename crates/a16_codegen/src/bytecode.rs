//! Bytecode Definitions
//!
//! Stack-based bytecode for the A16 VM.

use smol_str::SmolStr;

/// Compiled bytecode module
#[derive(Debug, Clone)]
pub struct BytecodeModule {
    /// Constant pool
    pub constants: Vec<Constant>,
    /// Function table
    pub functions: Vec<BytecodeFunction>,
    /// Global variable names
    pub globals: Vec<SmolStr>,
    /// Entry point function index (if any)
    pub entry: Option<u16>,
}

/// A compiled function
#[derive(Debug, Clone)]
pub struct BytecodeFunction {
    pub name: SmolStr,
    pub arity: u8,
    pub locals: u8,
    pub code: Vec<u8>,
}

/// Constant values
#[derive(Debug, Clone)]
pub enum Constant {
    Int(i64),
    Float(f64),
    Str(SmolStr),
    None,
    Bool(bool),
}

/// Bytecode opcodes
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    // Stack operations
    Nop = 0x00,
    Pop = 0x01,
    Dup = 0x02,
    Swap = 0x03,
    
    // Constants
    PushConst = 0x10,      // u16 index
    PushTrue = 0x11,
    PushFalse = 0x12,
    PushNone = 0x13,
    PushInt0 = 0x14,
    PushInt1 = 0x15,
    
    // Locals
    LoadLocal = 0x20,      // u8 slot
    StoreLocal = 0x21,     // u8 slot
    
    // Globals
    LoadGlobal = 0x22,       // u16 index (in globals table)
    StoreGlobal = 0x23,      // u16 index
    LoadGlobalByName = 0x24, // u16 name constant index
    
    // Arithmetic
    Add = 0x30,
    Sub = 0x31,
    Mul = 0x32,
    Div = 0x33,
    FloorDiv = 0x34,
    Mod = 0x35,
    Pow = 0x36,
    Neg = 0x37,
    
    // Bitwise
    BitAnd = 0x40,
    BitOr = 0x41,
    BitXor = 0x42,
    BitNot = 0x43,
    Shl = 0x44,
    Shr = 0x45,
    
    // Comparison
    Eq = 0x50,
    Ne = 0x51,
    Lt = 0x52,
    Le = 0x53,
    Gt = 0x54,
    Ge = 0x55,
    
    // Logical
    Not = 0x58,
    
    // Control flow
    Jump = 0x60,           // i16 offset
    JumpIfTrue = 0x61,     // i16 offset
    JumpIfFalse = 0x62,    // i16 offset
    
    // Functions
    Call = 0x70,           // u8 argc
    Return = 0x71,
    
    // Objects
    GetAttr = 0x80,        // u16 name index
    SetAttr = 0x81,        // u16 name index
    GetIndex = 0x82,
    SetIndex = 0x83,
    
    // Constructors
    BuildList = 0x90,      // u16 count
    BuildDict = 0x91,      // u16 count (pairs)
    BuildTuple = 0x92,     // u16 count
    
    // Closures
    MakeClosure = 0x95,    // u16 func_idx, u8 upvalue_count, then upvalue_count × (u8 is_local, u8 index)
    GetUpvalue = 0x96,     // u8 upvalue_index
    SetUpvalue = 0x97,     // u8 upvalue_index
    
    // AI Operations
    ModelInvoke = 0xA0,
    ToolDispatch = 0xA1,
    MemoryStore = 0xA2,
    MemoryRetrieve = 0xA3,
    
    // Async
    Await = 0xB0,
    Spawn = 0xB1,
    SpawnTask = 0xB2,      // u16 func_idx, u8 argc — spawn function as concurrent task
    JoinAll = 0xB3,        // u8 count — wait for N futures, push results as list
    Yield = 0xB4,          // cooperative yield point
    ChannelCreate = 0xB5,  // u16 capacity — create bounded channel
    ChannelSend = 0xB6,    // send value into channel (stack: [channel, value])
    ChannelRecv = 0xB7,    // receive value from channel (stack: [channel])
    
    // Iteration
    GetIter = 0xC0,
    ForIter = 0xC1,        // i16 offset (jump when exhausted)
    
    // FFI
    FfiCall = 0xD0,        // u16 ffi_func_idx, u8 argc — call foreign function
    FfiLoad = 0xD1,        // u16 lib_name_const_idx — load/register foreign library
    
    // Halt
    Halt = 0xFF,
}

impl Opcode {
    pub fn from_u8(byte: u8) -> Option<Opcode> {
        match byte {
            0x00 => Some(Opcode::Nop),
            0x01 => Some(Opcode::Pop),
            0x02 => Some(Opcode::Dup),
            0x03 => Some(Opcode::Swap),
            0x10 => Some(Opcode::PushConst),
            0x11 => Some(Opcode::PushTrue),
            0x12 => Some(Opcode::PushFalse),
            0x13 => Some(Opcode::PushNone),
            0x14 => Some(Opcode::PushInt0),
            0x15 => Some(Opcode::PushInt1),
            0x20 => Some(Opcode::LoadLocal),
            0x21 => Some(Opcode::StoreLocal),
            0x22 => Some(Opcode::LoadGlobal),
            0x23 => Some(Opcode::StoreGlobal),
            0x24 => Some(Opcode::LoadGlobalByName),
            0x30 => Some(Opcode::Add),
            0x31 => Some(Opcode::Sub),
            0x32 => Some(Opcode::Mul),
            0x33 => Some(Opcode::Div),
            0x34 => Some(Opcode::FloorDiv),
            0x35 => Some(Opcode::Mod),
            0x36 => Some(Opcode::Pow),
            0x37 => Some(Opcode::Neg),
            0x40 => Some(Opcode::BitAnd),
            0x41 => Some(Opcode::BitOr),
            0x42 => Some(Opcode::BitXor),
            0x43 => Some(Opcode::BitNot),
            0x44 => Some(Opcode::Shl),
            0x45 => Some(Opcode::Shr),
            0x50 => Some(Opcode::Eq),
            0x51 => Some(Opcode::Ne),
            0x52 => Some(Opcode::Lt),
            0x53 => Some(Opcode::Le),
            0x54 => Some(Opcode::Gt),
            0x55 => Some(Opcode::Ge),
            0x58 => Some(Opcode::Not),
            0x60 => Some(Opcode::Jump),
            0x61 => Some(Opcode::JumpIfTrue),
            0x62 => Some(Opcode::JumpIfFalse),
            0x70 => Some(Opcode::Call),
            0x71 => Some(Opcode::Return),
            0x80 => Some(Opcode::GetAttr),
            0x81 => Some(Opcode::SetAttr),
            0x82 => Some(Opcode::GetIndex),
            0x83 => Some(Opcode::SetIndex),
            0x90 => Some(Opcode::BuildList),
            0x91 => Some(Opcode::BuildDict),
            0x92 => Some(Opcode::BuildTuple),
            0x95 => Some(Opcode::MakeClosure),
            0x96 => Some(Opcode::GetUpvalue),
            0x97 => Some(Opcode::SetUpvalue),
            0xA0 => Some(Opcode::ModelInvoke),
            0xA1 => Some(Opcode::ToolDispatch),
            0xA2 => Some(Opcode::MemoryStore),
            0xA3 => Some(Opcode::MemoryRetrieve),
            0xB0 => Some(Opcode::Await),
            0xB1 => Some(Opcode::Spawn),
            0xB2 => Some(Opcode::SpawnTask),
            0xB3 => Some(Opcode::JoinAll),
            0xB4 => Some(Opcode::Yield),
            0xB5 => Some(Opcode::ChannelCreate),
            0xB6 => Some(Opcode::ChannelSend),
            0xB7 => Some(Opcode::ChannelRecv),
            0xC0 => Some(Opcode::GetIter),
            0xC1 => Some(Opcode::ForIter),
            0xD0 => Some(Opcode::FfiCall),
            0xD1 => Some(Opcode::FfiLoad),
            0xFF => Some(Opcode::Halt),
            _ => None,
        }
    }
}

impl BytecodeModule {
    pub fn new() -> Self {
        Self {
            constants: Vec::new(),
            functions: Vec::new(),
            globals: Vec::new(),
            entry: None,
        }
    }
}

impl Default for BytecodeModule {
    fn default() -> Self {
        Self::new()
    }
}
