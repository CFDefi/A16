//! HIR Node Definitions
//!
//! Desugared representation of A16 programs.

use smol_str::SmolStr;
use indexmap::IndexMap;

/// Unique identifier for variables/bindings
pub type VarId = u32;

/// Unique identifier for functions
pub type FuncId = u32;

/// HIR Module - top-level compilation unit
#[derive(Debug, Clone)]
pub struct HirModule {
    pub functions: Vec<HirFunction>,
    pub globals: Vec<HirGlobal>,
    pub agents: Vec<HirAgent>,
    pub tools: Vec<HirTool>,
}

/// HIR Function definition
#[derive(Debug, Clone)]
pub struct HirFunction {
    pub id: FuncId,
    pub name: SmolStr,
    pub params: Vec<HirParam>,
    pub body: HirBlock,
    pub is_async: bool,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct HirParam {
    pub id: VarId,
    pub name: SmolStr,
}

/// Global variable
#[derive(Debug, Clone)]
pub struct HirGlobal {
    pub id: VarId,
    pub name: SmolStr,
    pub init: Option<HirExpr>,
}

/// Agent definition (lowered)
#[derive(Debug, Clone)]
pub struct HirAgent {
    pub name: SmolStr,
    pub model: Option<HirExpr>,
    pub tools: Vec<HirExpr>,
    pub tasks: Vec<HirFunction>,
}

/// Tool definition (lowered)
#[derive(Debug, Clone)]
pub struct HirTool {
    pub name: SmolStr,
    pub permissions: Vec<SmolStr>,
    pub execute: Option<HirFunction>,
}

/// Block of statements
#[derive(Debug, Clone)]
pub struct HirBlock {
    pub stmts: Vec<HirStmt>,
}

/// HIR Statement
#[derive(Debug, Clone)]
pub enum HirStmt {
    /// Variable binding
    Let {
        id: VarId,
        name: SmolStr,
        value: Option<HirExpr>,
    },
    /// Assignment
    Assign {
        target: HirExpr,
        value: HirExpr,
    },
    /// Expression statement
    Expr(HirExpr),
    /// Return
    Return(Option<HirExpr>),
    /// If statement (desugared: no elif)
    If {
        condition: HirExpr,
        then_block: HirBlock,
        else_block: Option<HirBlock>,
    },
    /// Loop (for/while unified)
    Loop {
        init: Option<Box<HirStmt>>,
        condition: Option<HirExpr>,
        update: Option<Box<HirStmt>>,
        body: HirBlock,
    },
    /// For-each loop
    ForEach {
        var: VarId,
        name: SmolStr,
        iter: HirExpr,
        body: HirBlock,
    },
    /// Match statement
    Match {
        subject: HirExpr,
        arms: Vec<HirMatchArm>,
    },
    /// Break
    Break,
    /// Continue
    Continue,
}

/// Match arm
#[derive(Debug, Clone)]
pub struct HirMatchArm {
    pub pattern: HirPattern,
    pub guard: Option<HirExpr>,
    pub body: HirBlock,
}

/// Upvalue capture descriptor for closures
#[derive(Debug, Clone)]
pub struct Upvalue {
    /// Index of the captured variable in the enclosing scope
    pub index: u8,
    /// True if captured from the immediately enclosing function's locals,
    /// false if captured from an outer closure's upvalues
    pub is_local: bool,
    /// Name of the captured variable (for debugging)
    pub name: SmolStr,
}

/// Pattern for matching
#[derive(Debug, Clone)]
pub enum HirPattern {
    Wildcard,
    Var(VarId, SmolStr),
    Literal(HirLiteral),
    Tuple(Vec<HirPattern>),
    /// Constructor pattern: ClassName(sub-patterns)
    Constructor(SmolStr, Vec<HirPattern>),
    /// Or-pattern: p1 | p2
    Or(Vec<HirPattern>),
}

/// HIR Expression
#[derive(Debug, Clone)]
pub enum HirExpr {
    /// Literal values
    Literal(HirLiteral),
    /// Variable reference
    Var(VarId),
    /// Global reference (by ID)
    Global(VarId),
    /// Global reference by name (for stdlib/runtime lookup)
    GlobalRef(SmolStr),
    /// Binary operation
    Binary {
        op: HirBinaryOp,
        left: Box<HirExpr>,
        right: Box<HirExpr>,
    },
    /// Unary operation
    Unary {
        op: HirUnaryOp,
        operand: Box<HirExpr>,
    },
    /// Function call
    Call {
        func: Box<HirExpr>,
        args: Vec<HirExpr>,
    },
    /// Attribute access
    Attr {
        object: Box<HirExpr>,
        name: SmolStr,
    },
    /// Index access
    Index {
        object: Box<HirExpr>,
        index: Box<HirExpr>,
    },
    /// List literal
    List(Vec<HirExpr>),
    /// Dict literal
    Dict(Vec<(HirExpr, HirExpr)>),
    /// Tuple literal
    Tuple(Vec<HirExpr>),
    /// Lambda/closure (inline expression)
    Lambda {
        params: Vec<HirParam>,
        body: Box<HirExpr>,
    },
    /// Full closure with upvalue captures
    Closure {
        func_idx: u16,
        upvalues: Vec<Upvalue>,
    },
    /// Conditional expression
    IfExpr {
        condition: Box<HirExpr>,
        then_expr: Box<HirExpr>,
        else_expr: Box<HirExpr>,
    },
    /// Await expression
    Await(Box<HirExpr>),
    /// Spawn async task
    Spawn(Box<HirExpr>),
    
    // === AI Operations ===
    
    /// Model invocation
    ModelInvoke {
        model: Box<HirExpr>,
        prompt: Box<HirExpr>,
        config: IndexMap<SmolStr, HirExpr>,
    },
    /// Tool dispatch
    ToolDispatch {
        tool: Box<HirExpr>,
        args: Vec<HirExpr>,
    },
    /// Memory store
    MemoryStore {
        memory: Box<HirExpr>,
        content: Box<HirExpr>,
        metadata: Option<Box<HirExpr>>,
    },
    /// Memory retrieve
    MemoryRetrieve {
        memory: Box<HirExpr>,
        query: Box<HirExpr>,
        k: Option<Box<HirExpr>>,
    },
}

/// Literal values
#[derive(Debug, Clone)]
pub enum HirLiteral {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(SmolStr),
    None,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HirBinaryOp {
    // Arithmetic
    Add, Sub, Mul, Div, FloorDiv, Mod, Pow,
    // Bitwise
    BitAnd, BitOr, BitXor, Shl, Shr,
    // Logical
    And, Or,
    // Comparison
    Eq, Ne, Lt, Le, Gt, Ge,
    // Containment
    In, NotIn,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HirUnaryOp {
    Neg,    // -x
    Not,    // not x
    BitNot, // ~x
}
