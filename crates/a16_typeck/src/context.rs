//! Type checking context and symbol table

use crate::error::TypeError;
use crate::types::{Type, TypeVarId, StructInfo, ClassInfo, AgentInfo, ToolInfo};
use a16_ast::{Span, Module, Item};
use indexmap::IndexMap;
use smol_str::SmolStr;

/// Information about a symbol in scope
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Symbol name
    pub name: SmolStr,
    /// Symbol type
    pub ty: Type,
    /// Whether the symbol is mutable (let vs const)
    pub mutable: bool,
    /// Where the symbol was defined
    pub span: Span,
}

/// A single lexical scope
#[derive(Debug, Clone)]
struct Scope {
    /// Symbols defined in this scope
    symbols: IndexMap<SmolStr, Symbol>,
    /// Expected return type (for function scopes)
    return_type: Option<Type>,
    /// Whether this is a loop scope (for break/continue)
    is_loop: bool,
    /// Whether this is an async scope
    is_async: bool,
}

impl Scope {
    fn new() -> Self {
        Scope {
            symbols: IndexMap::new(),
            return_type: None,
            is_loop: false,
            is_async: false,
        }
    }
    
    fn with_return_type(ret: Type) -> Self {
        Scope {
            symbols: IndexMap::new(),
            return_type: Some(ret),
            is_loop: false,
            is_async: false,
        }
    }
}

/// Type checking context
/// 
/// Maintains the symbol table, type variable substitutions,
/// and accumulated errors during type checking.
pub struct TypeContext {
    /// Stack of lexical scopes
    scopes: Vec<Scope>,
    
    /// Type variable substitutions for unification
    substitutions: IndexMap<TypeVarId, Type>,
    
    /// Counter for generating fresh type variables
    next_type_var: TypeVarId,
    
    /// Registered struct types
    structs: IndexMap<SmolStr, StructInfo>,
    
    /// Registered class types
    classes: IndexMap<SmolStr, ClassInfo>,
    
    /// Registered agent types
    agents: IndexMap<SmolStr, AgentInfo>,
    
    /// Registered tool types
    tools: IndexMap<SmolStr, ToolInfo>,
    
    /// Accumulated type errors
    errors: Vec<TypeError>,
}

impl TypeContext {
    /// Create a new type checking context with builtins registered
    pub fn new() -> Self {
        let mut ctx = TypeContext {
            scopes: vec![Scope::new()],
            substitutions: IndexMap::new(),
            next_type_var: 0,
            structs: IndexMap::new(),
            classes: IndexMap::new(),
            agents: IndexMap::new(),
            tools: IndexMap::new(),
            errors: Vec::new(),
        };
        ctx.register_builtins();
        ctx
    }
    
    /// Register built-in functions and types
    fn register_builtins(&mut self) {
        use crate::builtins::register_all;
        register_all(self);
    }
    
    // =========================================================================
    // SCOPE MANAGEMENT
    // =========================================================================
    
    /// Enter a new lexical scope
    pub fn enter_scope(&mut self) {
        self.scopes.push(Scope::new());
    }
    
    /// Enter a function scope with expected return type
    pub fn enter_function_scope(&mut self, return_type: Type, is_async: bool) {
        let mut scope = Scope::with_return_type(return_type);
        scope.is_async = is_async;
        self.scopes.push(scope);
    }
    
    /// Enter a loop scope
    pub fn enter_loop_scope(&mut self) {
        let mut scope = Scope::new();
        scope.is_loop = true;
        self.scopes.push(scope);
    }
    
    /// Exit the current lexical scope
    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }
    
    /// Get the expected return type for the current function scope
    pub fn expected_return_type(&self) -> Option<&Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(ref ret) = scope.return_type {
                return Some(ret);
            }
        }
        None
    }
    
    /// Check if we're in a loop scope
    pub fn in_loop(&self) -> bool {
        self.scopes.iter().any(|s| s.is_loop)
    }
    
    /// Check if we're in an async scope
    pub fn in_async(&self) -> bool {
        self.scopes.iter().any(|s| s.is_async)
    }
    
    // =========================================================================
    // SYMBOL TABLE
    // =========================================================================
    
    /// Define a new symbol in the current scope
    pub fn define(&mut self, name: SmolStr, ty: Type, mutable: bool, span: Span) {
        let symbol = Symbol { name: name.clone(), ty, mutable, span };
        if let Some(scope) = self.scopes.last_mut() {
            scope.symbols.insert(name, symbol);
        }
    }
    
    /// Define a const (immutable) binding
    pub fn define_const(&mut self, name: SmolStr, ty: Type, span: Span) {
        self.define(name, ty, false, span);
    }
    
    /// Define a let (mutable) binding
    pub fn define_let(&mut self, name: SmolStr, ty: Type, span: Span) {
        self.define(name, ty, true, span);
    }
    
    /// Look up a symbol by name, searching from innermost to outermost scope
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.symbols.get(name) {
                return Some(sym);
            }
        }
        None
    }
    
    /// Check if a symbol is defined in the current scope (not parent scopes)
    pub fn is_defined_locally(&self, name: &str) -> bool {
        self.scopes.last()
            .map(|s| s.symbols.contains_key(name))
            .unwrap_or(false)
    }
    
    // =========================================================================
    // TYPE REGISTRATION
    // =========================================================================
    
    /// Register a struct type
    pub fn register_struct(&mut self, info: StructInfo) {
        self.structs.insert(info.name.clone(), info);
    }
    
    /// Register a class type
    pub fn register_class(&mut self, info: ClassInfo) {
        self.classes.insert(info.name.clone(), info);
    }
    
    /// Register an agent type
    pub fn register_agent(&mut self, info: AgentInfo) {
        self.agents.insert(info.name.clone(), info);
    }
    
    /// Register a tool type
    pub fn register_tool(&mut self, info: ToolInfo) {
        self.tools.insert(info.name.clone(), info);
    }
    
    /// Look up a struct by name
    pub fn lookup_struct(&self, name: &str) -> Option<&StructInfo> {
        self.structs.get(name)
    }
    
    /// Look up a class by name
    pub fn lookup_class(&self, name: &str) -> Option<&ClassInfo> {
        self.classes.get(name)
    }
    
    /// Look up an agent by name
    pub fn lookup_agent(&self, name: &str) -> Option<&AgentInfo> {
        self.agents.get(name)
    }
    
    /// Look up a tool by name
    pub fn lookup_tool(&self, name: &str) -> Option<&ToolInfo> {
        self.tools.get(name)
    }
    
    // =========================================================================
    // TYPE VARIABLES
    // =========================================================================
    
    /// Generate a fresh type variable
    pub fn fresh_type_var(&mut self) -> Type {
        let id = self.next_type_var;
        self.next_type_var += 1;
        Type::TypeVar(id)
    }
    
    /// Get the substitution for a type variable, if any
    pub fn get_substitution(&self, id: TypeVarId) -> Option<&Type> {
        self.substitutions.get(&id)
    }
    
    /// Set the substitution for a type variable
    pub fn set_substitution(&mut self, id: TypeVarId, ty: Type) {
        self.substitutions.insert(id, ty);
    }
    
    // =========================================================================
    // ERROR HANDLING
    // =========================================================================
    
    /// Record a type error
    pub fn error(&mut self, err: TypeError) {
        self.errors.push(err);
    }
    
    /// Check if any errors have been recorded
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    /// Get all recorded errors
    pub fn errors(&self) -> &[TypeError] {
        &self.errors
    }
    
    /// Take all recorded errors, clearing the internal list
    pub fn take_errors(&mut self) -> Vec<TypeError> {
        std::mem::take(&mut self.errors)
    }
    
    // =========================================================================
    // MODULE CHECKING
    // =========================================================================
    
    /// Type check a complete module
    pub fn check_module(&mut self, module: &Module) {
        // First pass: collect all type definitions
        for item in &module.items {
            self.register_item(item);
        }
        
        // Second pass: check all items
        for item in &module.items {
            self.check_item(item);
        }
    }
    
    /// Register an item's type without full checking (first pass)
    fn register_item(&mut self, item: &Item) {
        use crate::check_item::register_item_type;
        register_item_type(self, item);
    }
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}
