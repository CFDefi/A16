//! Type representations for A16

use smol_str::SmolStr;
use std::fmt;

/// Unique identifier for type variables during inference
pub type TypeVarId = u32;

/// Represents an A16 type
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // =========================================================================
    // Primitive Types
    // =========================================================================
    
    /// Arbitrary precision integer
    Int,
    /// 64-bit floating point
    Float,
    /// Boolean
    Bool,
    /// Unicode string
    Str,
    /// Byte sequence
    Bytes,
    /// Null type
    None,
    
    // =========================================================================
    // Generic Container Types
    // =========================================================================
    
    /// Mutable list: List[T]
    List(Box<Type>),
    /// Dictionary: Dict[K, V]
    Dict(Box<Type>, Box<Type>),
    /// Mutable set: Set[T]
    Set(Box<Type>),
    /// Immutable tuple: Tuple[T1, T2, ...]
    Tuple(Vec<Type>),
    /// Optional type: Optional[T] = T | None
    Optional(Box<Type>),
    /// Union type: Union[T1, T2, ...]
    Union(Vec<Type>),
    
    // =========================================================================
    // Callable Types
    // =========================================================================
    
    /// Function type: (P1, P2, ...) -> R
    Function {
        params: Vec<Type>,
        ret: Box<Type>,
    },
    
    /// Async function type: async (P1, P2, ...) -> R
    AsyncFunction {
        params: Vec<Type>,
        ret: Box<Type>,
    },
    
    // =========================================================================
    // AI-Specific Types
    // =========================================================================
    
    /// Named agent type
    Agent(SmolStr),
    /// Named tool type
    Tool(SmolStr),
    /// Named memory type
    Memory(SmolStr),
    /// Named prompt type
    Prompt(SmolStr),
    /// Chat message
    Message,
    /// Execution context
    Context,
    /// Token budget allocation
    TokenBudget,
    /// Model response
    ModelResponse,
    /// Vector embedding
    Embedding,
    /// Structured output schema
    Schema(Box<Type>),
    
    // =========================================================================
    // User-Defined Types
    // =========================================================================
    
    /// Class type
    Class(SmolStr),
    /// Struct type
    Struct(SmolStr),
    /// Enum type
    Enum(SmolStr),
    
    // =========================================================================
    // Type Inference Support
    // =========================================================================
    
    /// Unification type variable (for inference)
    TypeVar(TypeVarId),
    /// Unknown type (for gradual typing - accepts anything)
    Unknown,
    /// Dynamic type (explicit Any annotation)
    Any,
    /// Error placeholder (for recovery after type errors)
    Error,
    
    // =========================================================================
    // Special Types
    // =========================================================================
    
    /// Never type (for functions that don't return)
    Never,
    /// Iterator type
    Iterator(Box<Type>),
    /// Result type: Result[T, E]
    Result(Box<Type>, Box<Type>),
    
    // =========================================================================
    // FFI Types
    // =========================================================================
    
    /// Raw pointer type (for FFI)
    Ptr,
    /// Extern function declaration type
    ExternFunc {
        lib_name: SmolStr,
        params: Vec<Type>,
        ret: Box<Type>,
    },
}

impl Type {
    /// Check if this type is a numeric type (Int or Float)
    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::Int | Type::Float)
    }
    
    /// Check if this type is a primitive type
    pub fn is_primitive(&self) -> bool {
        matches!(self, Type::Int | Type::Float | Type::Bool | Type::Str | Type::Bytes | Type::None)
    }
    
    /// Check if this type is callable
    pub fn is_callable(&self) -> bool {
        matches!(self, Type::Function { .. } | Type::AsyncFunction { .. })
    }
    
    /// Check if this type is iterable
    pub fn is_iterable(&self) -> bool {
        matches!(self, Type::List(_) | Type::Set(_) | Type::Dict(_, _) | Type::Str | Type::Tuple(_) | Type::Iterator(_))
    }
    
    /// Get the element type if this is a container
    pub fn element_type(&self) -> Option<Type> {
        match self {
            Type::List(elem) => Some((**elem).clone()),
            Type::Set(elem) => Some((**elem).clone()),
            Type::Iterator(elem) => Some((**elem).clone()),
            Type::Str => Some(Type::Str),
            _ => None,
        }
    }
    
    /// Check if this type allows None values
    pub fn is_nullable(&self) -> bool {
        matches!(self, Type::Optional(_) | Type::None | Type::Any | Type::Unknown)
    }
    
    /// Check if this type represents an error state
    pub fn is_error(&self) -> bool {
        matches!(self, Type::Error)
    }
    
    /// Check if this type is a type variable
    pub fn is_type_var(&self) -> bool {
        matches!(self, Type::TypeVar(_))
    }
    
    /// Get the return type if this is a function
    pub fn return_type(&self) -> Option<&Type> {
        match self {
            Type::Function { ret, .. } | Type::AsyncFunction { ret, .. } => Some(ret),
            _ => None,
        }
    }
    
    /// Get the parameter types if this is a function
    pub fn param_types(&self) -> Option<&[Type]> {
        match self {
            Type::Function { params, .. } | Type::AsyncFunction { params, .. } => Some(params),
            _ => None,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Primitives
            Type::Int => write!(f, "Int"),
            Type::Float => write!(f, "Float"),
            Type::Bool => write!(f, "Bool"),
            Type::Str => write!(f, "Str"),
            Type::Bytes => write!(f, "Bytes"),
            Type::None => write!(f, "None"),
            
            // Containers
            Type::List(elem) => write!(f, "List[{}]", elem),
            Type::Dict(key, val) => write!(f, "Dict[{}, {}]", key, val),
            Type::Set(elem) => write!(f, "Set[{}]", elem),
            Type::Tuple(elems) => {
                write!(f, "Tuple[")?;
                for (i, elem) in elems.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", elem)?;
                }
                write!(f, "]")
            }
            Type::Optional(inner) => write!(f, "Optional[{}]", inner),
            Type::Union(types) => {
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 { write!(f, " | ")?; }
                    write!(f, "{}", ty)?;
                }
                Ok(())
            }
            
            // Callables
            Type::Function { params, ret } => {
                write!(f, "(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", ret)
            }
            Type::AsyncFunction { params, ret } => {
                write!(f, "async (")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", ret)
            }
            
            // AI types
            Type::Agent(name) => write!(f, "Agent[{}]", name),
            Type::Tool(name) => write!(f, "Tool[{}]", name),
            Type::Memory(name) => write!(f, "Memory[{}]", name),
            Type::Prompt(name) => write!(f, "Prompt[{}]", name),
            Type::Message => write!(f, "Message"),
            Type::Context => write!(f, "Context"),
            Type::TokenBudget => write!(f, "TokenBudget"),
            Type::ModelResponse => write!(f, "ModelResponse"),
            Type::Embedding => write!(f, "Embedding"),
            Type::Schema(inner) => write!(f, "Schema[{}]", inner),
            
            // User-defined
            Type::Class(name) => write!(f, "{}", name),
            Type::Struct(name) => write!(f, "{}", name),
            Type::Enum(name) => write!(f, "{}", name),
            
            // Inference
            Type::TypeVar(id) => write!(f, "T{}", id),
            Type::Unknown => write!(f, "?"),
            Type::Any => write!(f, "Any"),
            Type::Error => write!(f, "<error>"),
            
            // Special
            Type::Never => write!(f, "Never"),
            Type::Iterator(elem) => write!(f, "Iterator[{}]", elem),
            Type::Result(ok, err) => write!(f, "Result[{}, {}]", ok, err),
            
            // FFI
            Type::Ptr => write!(f, "Ptr"),
            Type::ExternFunc { lib_name, params, ret } => {
                write!(f, "extern[{}](", lib_name)?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", ret)
            }
        }
    }
}

/// Type information for a struct field
#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: SmolStr,
    pub ty: Type,
    pub has_default: bool,
}

/// Type information for a defined struct
#[derive(Debug, Clone)]
pub struct StructInfo {
    pub name: SmolStr,
    pub fields: Vec<FieldInfo>,
}

/// Type information for a defined class
#[derive(Debug, Clone)]
pub struct ClassInfo {
    pub name: SmolStr,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<(SmolStr, Type)>,
    pub base: Option<SmolStr>,
}

/// Type information for an agent
#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub name: SmolStr,
    pub model_type: Option<Type>,
    pub tools: Vec<SmolStr>,
    pub tasks: Vec<(SmolStr, Type)>,
}

/// Type information for a tool
#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: SmolStr,
    pub permissions: Vec<SmolStr>,
    pub execute_type: Option<Type>,
}
