//! AST Node definitions

use crate::{Ident, Span};
use smol_str::SmolStr;

/// Top-level items
#[derive(Debug, Clone)]
pub enum Item {
    Function(FunctionDef),
    Class(ClassDef),
    Agent(AgentDef),
    Tool(ToolDef),
    Memory(MemoryDef),
    Prompt(PromptDef),
    Struct(StructDef),
    Enum(EnumDef),
    Import(ImportStmt),
    Const(ConstDef),
    Extern(ExternBlock),
    Stmt(Stmt),
}

// =============================================================================
// FUNCTIONS
// =============================================================================

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: Ident,
    pub params: Vec<Param>,
    pub return_type: Option<TypeExpr>,
    pub body: Block,
    pub decorators: Vec<Decorator>,
    pub is_async: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: Ident,
    pub ty: Option<TypeExpr>,
    pub default: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Decorator {
    pub name: DottedName,
    pub args: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct DottedName {
    pub parts: Vec<Ident>,
    pub span: Span,
}

// =============================================================================
// CLASSES
// =============================================================================

#[derive(Debug, Clone)]
pub struct ClassDef {
    pub name: Ident,
    pub bases: Vec<Expr>,
    pub body: Vec<ClassMember>,
    pub decorators: Vec<Decorator>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ClassMember {
    Field(FieldDef),
    Method(FunctionDef),
    Class(ClassDef),
}

#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: Ident,
    pub ty: TypeExpr,
    pub default: Option<Expr>,
    pub span: Span,
}

// =============================================================================
// AI PRIMITIVES
// =============================================================================

#[derive(Debug, Clone)]
pub struct AgentDef {
    pub name: Ident,
    pub config: Vec<AgentConfig>,
    pub members: Vec<AgentMember>,
    pub decorators: Vec<Decorator>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum AgentConfig {
    Model(Expr),
    Memory(Vec<Expr>),
    Tools(Vec<Expr>),
    Budget(Vec<BudgetItem>),
    Policy(Vec<Expr>),
}

#[derive(Debug, Clone)]
pub struct BudgetItem {
    pub name: Ident,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub enum AgentMember {
    Config(AgentConfig),
    Task(TaskDef),
    Method(FunctionDef),
    OnEvent(OnEventDef),
}

#[derive(Debug, Clone)]
pub struct TaskDef {
    pub name: Ident,
    pub params: Vec<Param>,
    pub return_type: Option<TypeExpr>,
    pub body: Block,
    pub is_async: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct OnEventDef {
    pub event: Ident,
    pub params: Vec<Param>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ToolDef {
    pub name: Ident,
    pub members: Vec<ToolMember>,
    pub decorators: Vec<Decorator>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ToolMember {
    Permissions(Vec<Ident>),
    Sandbox(Ident),
    RateLimit(Expr),
    Audit(Ident),
    Schema(TypeExpr),
    Method(FunctionDef),
}

#[derive(Debug, Clone)]
pub struct MemoryDef {
    pub name: Ident,
    pub members: Vec<MemoryMember>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum MemoryMember {
    Type(Ident),
    Capacity(Expr),
    Retention(Expr),
    Compression(Ident),
    Index(Ident),
    Method(FunctionDef),
}

#[derive(Debug, Clone)]
pub struct PromptDef {
    pub name: Ident,
    pub params: Vec<Param>,
    pub sections: Vec<PromptSection>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct PromptSection {
    pub name: Ident,
    pub content: PromptContent,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum PromptContent {
    String(SmolStr),
    Template(SmolStr),
    Block(Block),
}

// =============================================================================
// STRUCTS & ENUMS
// =============================================================================

#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: Ident,
    pub fields: Vec<FieldDef>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: Ident,
    pub variants: Vec<EnumVariant>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: Ident,
    pub fields: Option<Vec<FieldDef>>,
    pub span: Span,
}

// =============================================================================
// IMPORTS
// =============================================================================

#[derive(Debug, Clone)]
pub struct ImportStmt {
    pub kind: ImportKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ImportKind {
    /// import module.path
    Module {
        path: DottedName,
        alias: Option<Ident>,
    },
    /// from module.path import item1, item2
    From {
        path: DottedName,
        items: Vec<ImportItem>,
    },
}

#[derive(Debug, Clone)]
pub struct ImportItem {
    pub name: Ident,
    pub alias: Option<Ident>,
}

#[derive(Debug, Clone)]
pub struct ConstDef {
    pub name: Ident,
    pub ty: Option<TypeExpr>,
    pub value: Expr,
    pub span: Span,
}

// =============================================================================
// STATEMENTS
// =============================================================================

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(ExprStmt),
    Let(LetStmt),
    Assign(AssignStmt),
    AugAssign(AugAssignStmt),
    Return(ReturnStmt),
    If(IfStmt),
    For(ForStmt),
    While(WhileStmt),
    Match(MatchStmt),
    Try(TryStmt),
    With(WithStmt),
    Raise(RaiseStmt),
    Assert(AssertStmt),
    Break(Span),
    Continue(Span),
    Pass(Span),
    Async(AsyncBlock),
}

#[derive(Debug, Clone)]
pub struct ExprStmt {
    pub expr: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct LetStmt {
    pub pattern: Pattern,
    pub ty: Option<TypeExpr>,
    pub value: Option<Expr>,
    pub is_const: bool,
    pub is_mutable: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AssignStmt {
    pub target: Expr,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AugAssignStmt {
    pub target: Expr,
    pub op: AugOp,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub enum AugOp {
    Add, Sub, Mul, Div, FloorDiv, Mod, Pow,
    BitAnd, BitOr, BitXor, Shl, Shr,
}

#[derive(Debug, Clone)]
pub struct ReturnStmt {
    pub value: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_block: Block,
    pub elif_blocks: Vec<(Expr, Block)>,
    pub else_block: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub target: Pattern,
    pub iter: Expr,
    pub body: Block,
    pub else_block: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Block,
    pub else_block: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MatchStmt {
    pub subject: Expr,
    pub arms: Vec<MatchArm>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TryStmt {
    pub body: Block,
    pub handlers: Vec<ExceptHandler>,
    pub else_block: Option<Block>,
    pub finally_block: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ExceptHandler {
    pub ty: Option<Expr>,
    pub name: Option<Ident>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct WithStmt {
    pub items: Vec<WithItem>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct WithItem {
    pub context: Expr,
    pub alias: Option<Ident>,
}

#[derive(Debug, Clone)]
pub struct RaiseStmt {
    pub exception: Option<Expr>,
    pub cause: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AssertStmt {
    pub test: Expr,
    pub msg: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AsyncBlock {
    pub stmts: Vec<Stmt>,
    pub is_parallel: bool,
    pub span: Span,
}

// =============================================================================
// EXPRESSIONS
// =============================================================================

#[derive(Debug, Clone)]
pub enum Expr {
    // Literals
    Int(i64, Span),
    Float(f64, Span),
    String(SmolStr, Span),
    FString(FStringExpr),
    Bool(bool, Span),
    None(Span),
    
    // Identifiers
    Ident(Ident),
    
    // Compound
    List(ListExpr),
    Dict(DictExpr),
    Set(SetExpr),
    Tuple(TupleExpr),
    
    // Operations
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    Compare(CompareExpr),
    
    // Access
    Attribute(AttributeExpr),
    Subscript(SubscriptExpr),
    Call(CallExpr),
    
    // Special
    Lambda(LambdaExpr),
    IfExpr(IfExprNode),
    Await(AwaitExpr),
    Yield(YieldExpr),
    Spawn(SpawnExpr),
    
    // Comprehensions
    ListComp(ComprehensionExpr),
    DictComp(DictComprehensionExpr),
    SetComp(ComprehensionExpr),
    GeneratorExpr(ComprehensionExpr),
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Int(_, s) | Expr::Float(_, s) | Expr::String(_, s) |
            Expr::Bool(_, s) | Expr::None(s) => *s,
            Expr::Ident(i) => i.span,
            Expr::FString(e) => e.span,
            Expr::List(e) => e.span,
            Expr::Dict(e) => e.span,
            Expr::Set(e) => e.span,
            Expr::Tuple(e) => e.span,
            Expr::Binary(e) => e.span,
            Expr::Unary(e) => e.span,
            Expr::Compare(e) => e.span,
            Expr::Attribute(e) => e.span,
            Expr::Subscript(e) => e.span,
            Expr::Call(e) => e.span,
            Expr::Lambda(e) => e.span,
            Expr::IfExpr(e) => e.span,
            Expr::Await(e) => e.span,
            Expr::Yield(e) => e.span,
            Expr::Spawn(e) => e.span,
            Expr::ListComp(e) | Expr::SetComp(e) | Expr::GeneratorExpr(e) => e.span,
            Expr::DictComp(e) => e.span,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FStringExpr {
    pub parts: Vec<FStringPart>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum FStringPart {
    Literal(SmolStr),
    Expr(Box<Expr>),
}

#[derive(Debug, Clone)]
pub struct ListExpr {
    pub elements: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct DictExpr {
    pub pairs: Vec<(Expr, Expr)>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct SetExpr {
    pub elements: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TupleExpr {
    pub elements: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub op: BinaryOp,
    pub right: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, FloorDiv, Mod, Pow,
    BitAnd, BitOr, BitXor, Shl, Shr,
    And, Or,
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub operand: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Neg, Pos, Not, BitNot,
}

#[derive(Debug, Clone)]
pub struct CompareExpr {
    pub left: Box<Expr>,
    pub comparisons: Vec<(CompareOp, Expr)>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub enum CompareOp {
    Eq, Ne, Lt, Le, Gt, Ge, In, NotIn, Is, IsNot,
}

#[derive(Debug, Clone)]
pub struct AttributeExpr {
    pub value: Box<Expr>,
    pub attr: Ident,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct SubscriptExpr {
    pub value: Box<Expr>,
    pub index: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct CallExpr {
    pub func: Box<Expr>,
    pub args: Vec<Arg>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Arg {
    pub name: Option<Ident>,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct LambdaExpr {
    pub params: Vec<Param>,
    pub body: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct IfExprNode {
    pub condition: Box<Expr>,
    pub then_expr: Box<Expr>,
    pub else_expr: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AwaitExpr {
    pub value: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct YieldExpr {
    pub value: Option<Box<Expr>>,
    pub is_from: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct SpawnExpr {
    pub value: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ComprehensionExpr {
    pub element: Box<Expr>,
    pub generators: Vec<Generator>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct DictComprehensionExpr {
    pub key: Box<Expr>,
    pub value: Box<Expr>,
    pub generators: Vec<Generator>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Generator {
    pub target: Pattern,
    pub iter: Expr,
    pub conditions: Vec<Expr>,
    pub is_async: bool,
}

// =============================================================================
// PATTERNS
// =============================================================================

#[derive(Debug, Clone)]
pub enum Pattern {
    Ident(Ident),
    Tuple(Vec<Pattern>, Span),
    List(Vec<Pattern>, Span),
    Rest(Ident, Span),
    Literal(Expr),
    Class(ClassPattern),
    Or(Vec<Pattern>, Span),
    Wildcard(Span),
}

#[derive(Debug, Clone)]
pub struct ClassPattern {
    pub name: DottedName,
    pub args: Vec<Pattern>,
    pub span: Span,
}

// =============================================================================
// TYPES
// =============================================================================

#[derive(Debug, Clone)]
pub enum TypeExpr {
    Name(Ident),
    Generic(GenericType),
    Union(Vec<TypeExpr>, Span),
    Optional(Box<TypeExpr>, Span),
    Callable(CallableType),
    Tuple(Vec<TypeExpr>, Span),
}

#[derive(Debug, Clone)]
pub struct GenericType {
    pub name: Ident,
    pub args: Vec<TypeExpr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct CallableType {
    pub params: Vec<TypeExpr>,
    pub return_type: Box<TypeExpr>,
    pub span: Span,
}

impl TypeExpr {
    pub fn span(&self) -> Span {
        match self {
            TypeExpr::Name(ident) => ident.span,
            TypeExpr::Generic(g) => g.span,
            TypeExpr::Union(_, span) => *span,
            TypeExpr::Optional(_, span) => *span,
            TypeExpr::Callable(c) => c.span,
            TypeExpr::Tuple(_, span) => *span,
        }
    }
}

// =============================================================================
// FFI / EXTERN
// =============================================================================

/// An extern block declaring foreign functions from a native library
#[derive(Debug, Clone)]
pub struct ExternBlock {
    /// Library name (e.g., "libc", "math_ext")
    pub lib_name: SmolStr,
    /// Declared extern functions
    pub functions: Vec<ExternFunc>,
    pub span: Span,
}

/// A foreign function declaration (no body)
#[derive(Debug, Clone)]
pub struct ExternFunc {
    /// Function name
    pub name: Ident,
    /// Parameters with types
    pub params: Vec<Param>,
    /// Return type
    pub return_type: Option<TypeExpr>,
    pub span: Span,
}
