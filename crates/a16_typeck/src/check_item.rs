//! Top-level item type checking

use crate::context::TypeContext;
use crate::error::TypeError;
use crate::types::{Type, FieldInfo, StructInfo, ClassInfo, AgentInfo, ToolInfo};
use a16_ast::*;
use smol_str::SmolStr;

impl TypeContext {
    /// Type check a top-level item
    pub fn check_item(&mut self, item: &Item) {
        match item {
            Item::Function(func) => self.check_function(func),
            Item::Class(class) => self.check_class(class),
            Item::Agent(agent) => self.check_agent(agent),
            Item::Tool(tool) => self.check_tool(tool),
            Item::Memory(memory) => self.check_memory(memory),
            Item::Prompt(prompt) => self.check_prompt(prompt),
            Item::Struct(struct_def) => self.check_struct(struct_def),
            Item::Enum(enum_def) => self.check_enum(enum_def),
            Item::Import(import) => self.check_import(import),
            Item::Const(const_def) => self.check_const(const_def),
            Item::Stmt(stmt) => self.check_stmt(stmt),
        }
    }
    
    /// Check a function definition
    fn check_function(&mut self, func: &FunctionDef) {
        // Build parameter types
        let mut param_types = Vec::new();
        
        for param in &func.params {
            let param_ty = if let Some(ref ty_expr) = param.ty {
                self.resolve_type_expr(ty_expr)
            } else {
                self.fresh_type_var()
            };
            param_types.push(param_ty);
        }
        
        // Get return type
        let return_ty = if let Some(ref ty_expr) = func.return_type {
            self.resolve_type_expr(ty_expr)
        } else {
            self.fresh_type_var()
        };
        
        // Enter function scope
        let is_async = func.is_async;
        self.enter_function_scope(return_ty.clone(), is_async);
        
        // Bind parameters
        for (param, ty) in func.params.iter().zip(param_types.iter()) {
            self.define_let(param.name.name.clone(), ty.clone(), param.name.span);
        }
        
        // Check the body
        self.check_block(&func.body);
        
        // Exit function scope
        self.exit_scope();
    }
    
    /// Check a class definition
    fn check_class(&mut self, class: &ClassDef) {
        self.enter_scope();
        
        // Define 'self' as the class type
        let class_ty = Type::Class(class.name.name.clone());
        self.define_const(SmolStr::new("self"), class_ty.clone(), class.name.span);
        
        // Check each member
        for member in &class.body {
            match member {
                ClassMember::Field(field) => {
                    let field_ty = self.resolve_type_expr(&field.ty);
                    
                    // Check default value if present
                    if let Some(ref default) = field.default {
                        let default_ty = self.check_expr(default);
                        self.unify(&field_ty, &default_ty, default.span());
                    }
                }
                ClassMember::Method(method) => {
                    self.check_function(method);
                }
                ClassMember::Class(nested) => {
                    self.check_class(nested);
                }
            }
        }
        
        self.exit_scope();
    }
    
    /// Check an agent definition
    fn check_agent(&mut self, agent: &AgentDef) {
        self.enter_scope();
        
        // Define 'self' as the agent type
        let agent_ty = Type::Agent(agent.name.name.clone());
        self.define_const(SmolStr::new("self"), agent_ty.clone(), agent.name.span);
        
        let mut has_model = false;
        
        // Check config items
        for config in &agent.config {
            match config {
                AgentConfig::Model(expr) => {
                    self.check_expr(expr);
                    has_model = true;
                }
                AgentConfig::Memory(exprs) => {
                    for expr in exprs {
                        self.check_expr(expr);
                    }
                }
                AgentConfig::Tools(exprs) => {
                    for expr in exprs {
                        self.check_expr(expr);
                    }
                }
                AgentConfig::Budget(items) => {
                    for item in items {
                        self.check_expr(&item.value);
                    }
                }
                AgentConfig::Policy(exprs) => {
                    for expr in exprs {
                        self.check_expr(expr);
                    }
                }
            }
        }
        
        // Check each member
        for member in &agent.members {
            match member {
                AgentMember::Config(config) => {
                    match config {
                        AgentConfig::Model(expr) => {
                            self.check_expr(expr);
                            has_model = true;
                        }
                        AgentConfig::Memory(exprs) => {
                            for expr in exprs {
                                self.check_expr(expr);
                            }
                        }
                        AgentConfig::Tools(exprs) => {
                            for expr in exprs {
                                self.check_expr(expr);
                            }
                        }
                        AgentConfig::Budget(items) => {
                            for item in items {
                                self.check_expr(&item.value);
                            }
                        }
                        AgentConfig::Policy(exprs) => {
                            for expr in exprs {
                                self.check_expr(expr);
                            }
                        }
                    }
                }
                AgentMember::Task(task) => {
                    self.check_task(task);
                }
                AgentMember::Method(method) => {
                    self.check_function(method);
                }
                AgentMember::OnEvent(on_event) => {
                    self.check_on_event(on_event);
                }
            }
        }
        
        // Agents should have a model defined
        if !has_model {
            self.error(TypeError::AgentMissingClause {
                name: agent.name.name.to_string(),
                missing: "model".to_string(),
                span: miette::SourceSpan::new(
                    (agent.name.span.start as usize).into(),
                    (agent.name.span.end.saturating_sub(agent.name.span.start)) as usize,
                ),
            });
        }
        
        self.exit_scope();
    }
    
    /// Check a task definition (inside agent)
    fn check_task(&mut self, task: &TaskDef) {
        let mut param_types = Vec::new();
        
        for param in &task.params {
            let param_ty = if let Some(ref ty_expr) = param.ty {
                self.resolve_type_expr(ty_expr)
            } else {
                self.fresh_type_var()
            };
            param_types.push(param_ty);
        }
        
        let return_ty = if let Some(ref ty_expr) = task.return_type {
            self.resolve_type_expr(ty_expr)
        } else {
            self.fresh_type_var()
        };
        
        // Tasks are always async
        self.enter_function_scope(return_ty, true);
        
        for (param, ty) in task.params.iter().zip(param_types.iter()) {
            self.define_let(param.name.name.clone(), ty.clone(), param.name.span);
        }
        
        self.check_block(&task.body);
        self.exit_scope();
    }
    
    /// Check an on_event handler
    fn check_on_event(&mut self, on_event: &OnEventDef) {
        let mut param_types = Vec::new();
        
        for param in &on_event.params {
            let param_ty = if let Some(ref ty_expr) = param.ty {
                self.resolve_type_expr(ty_expr)
            } else {
                self.fresh_type_var()
            };
            param_types.push(param_ty);
        }
        
        self.enter_function_scope(Type::None, true);
        
        for (param, ty) in on_event.params.iter().zip(param_types.iter()) {
            self.define_let(param.name.name.clone(), ty.clone(), param.name.span);
        }
        
        self.check_block(&on_event.body);
        self.exit_scope();
    }
    
    /// Check a tool definition
    fn check_tool(&mut self, tool: &ToolDef) {
        self.enter_scope();
        
        let mut has_execute = false;
        
        for member in &tool.members {
            match member {
                ToolMember::Permissions(_perms) => {
                    // Just validate permissions are identifiers
                }
                ToolMember::Sandbox(_sandbox) => {
                    // Validate sandbox mode
                }
                ToolMember::RateLimit(expr) => {
                    self.check_expr(expr);
                }
                ToolMember::Audit(_audit) => {
                    // Validate audit level
                }
                ToolMember::Schema(ty_expr) => {
                    self.resolve_type_expr(ty_expr);
                }
                ToolMember::Method(method) => {
                    if method.name.name == "execute" {
                        has_execute = true;
                    }
                    self.check_function(method);
                }
            }
        }
        
        if !has_execute {
            self.error(TypeError::ToolNoExecute {
                name: tool.name.name.to_string(),
                span: miette::SourceSpan::new(
                    (tool.name.span.start as usize).into(),
                    (tool.name.span.end.saturating_sub(tool.name.span.start)) as usize,
                ),
            });
        }
        
        self.exit_scope();
    }
    
    /// Check a memory definition
    fn check_memory(&mut self, memory: &MemoryDef) {
        self.enter_scope();
        
        for member in &memory.members {
            match member {
                MemoryMember::Type(_ty) => {
                    // Validate memory type identifier
                }
                MemoryMember::Capacity(expr) => {
                    self.check_expr(expr);
                }
                MemoryMember::Retention(expr) => {
                    self.check_expr(expr);
                }
                MemoryMember::Compression(_compression) => {
                    // Validate compression mode
                }
                MemoryMember::Index(_index) => {
                    // Validate index type
                }
                MemoryMember::Method(method) => {
                    self.check_function(method);
                }
            }
        }
        
        self.exit_scope();
    }
    
    /// Check a prompt definition
    fn check_prompt(&mut self, prompt: &PromptDef) {
        self.enter_scope();
        
        // Bind parameters
        for param in &prompt.params {
            let param_ty = if let Some(ref ty_expr) = param.ty {
                self.resolve_type_expr(ty_expr)
            } else {
                Type::Any
            };
            self.define_let(param.name.name.clone(), param_ty, param.name.span);
        }
        
        // Check prompt sections
        for section in &prompt.sections {
            match &section.content {
                PromptContent::String(_) => {}
                PromptContent::Template(_) => {}
                PromptContent::Block(block) => {
                    self.check_block(block);
                }
            }
        }
        
        self.exit_scope();
    }
    
    /// Check a struct definition
    fn check_struct(&mut self, struct_def: &StructDef) {
        for field in &struct_def.fields {
            self.resolve_type_expr(&field.ty);
            
            if let Some(ref default) = field.default {
                let default_ty = self.check_expr(default);
                let field_ty = self.resolve_type_expr(&field.ty);
                self.unify(&field_ty, &default_ty, default.span());
            }
        }
    }
    
    /// Check an enum definition
    fn check_enum(&mut self, enum_def: &EnumDef) {
        // Enum variants don't need much checking beyond registration
        for _variant in &enum_def.variants {
            // Variant fields would be checked if needed
        }
    }
    
    /// Check an import statement
    fn check_import(&mut self, import: &ImportStmt) {
        match &import.kind {
            ImportKind::Module { path, alias } => {
                let name = alias.as_ref()
                    .unwrap_or_else(|| path.parts.last().unwrap());
                self.define_const(name.name.clone(), Type::Any, name.span);
            }
            ImportKind::From { path: _, items } => {
                for item in items {
                    let name = item.alias.as_ref().unwrap_or(&item.name);
                    self.define_const(name.name.clone(), Type::Any, name.span);
                }
            }
        }
    }
    
    /// Check a const definition
    fn check_const(&mut self, const_def: &ConstDef) {
        let value_ty = self.check_expr(&const_def.value);
        
        let const_ty = if let Some(ref ty_expr) = const_def.ty {
            let declared = self.resolve_type_expr(ty_expr);
            self.unify(&declared, &value_ty, const_def.value.span())
        } else {
            value_ty
        };
        
        self.define_const(const_def.name.name.clone(), const_ty, const_def.name.span);
    }
}

/// First pass: register item types without full checking
pub fn register_item_type(ctx: &mut TypeContext, item: &Item) {
    match item {
        Item::Function(func) => {
            let param_types: Vec<Type> = func.params.iter()
                .map(|p| {
                    if let Some(ref ty) = p.ty {
                        ctx.resolve_type_expr(ty)
                    } else {
                        ctx.fresh_type_var()
                    }
                })
                .collect();
            
            let return_ty = if let Some(ref ty) = func.return_type {
                ctx.resolve_type_expr(ty)
            } else {
                ctx.fresh_type_var()
            };
            
            let func_ty = if func.is_async {
                Type::AsyncFunction {
                    params: param_types,
                    ret: Box::new(return_ty),
                }
            } else {
                Type::Function {
                    params: param_types,
                    ret: Box::new(return_ty),
                }
            };
            
            ctx.define_const(func.name.name.clone(), func_ty, func.name.span);
        }
        
        Item::Class(class) => {
            let mut fields = Vec::new();
            let mut methods = Vec::new();
            
            for member in &class.body {
                match member {
                    ClassMember::Field(field) => {
                        let ty = ctx.resolve_type_expr(&field.ty);
                        fields.push(FieldInfo {
                            name: field.name.name.clone(),
                            ty,
                            has_default: field.default.is_some(),
                        });
                    }
                    ClassMember::Method(method) => {
                        let param_types: Vec<Type> = method.params.iter()
                            .map(|p| {
                                if let Some(ref ty) = p.ty {
                                    ctx.resolve_type_expr(ty)
                                } else {
                                    ctx.fresh_type_var()
                                }
                            })
                            .collect();
                        
                        let return_ty = if let Some(ref ty) = method.return_type {
                            ctx.resolve_type_expr(ty)
                        } else {
                            ctx.fresh_type_var()
                        };
                        
                        let method_ty = Type::Function {
                            params: param_types,
                            ret: Box::new(return_ty),
                        };
                        
                        methods.push((method.name.name.clone(), method_ty));
                    }
                    ClassMember::Class(_) => {}
                }
            }
            
            let base = class.bases.first().and_then(|b| {
                if let Expr::Ident(ident) = b {
                    Some(ident.name.clone())
                } else {
                    None
                }
            });
            
            ctx.register_class(ClassInfo {
                name: class.name.name.clone(),
                fields,
                methods,
                base,
            });
            
            ctx.define_const(
                class.name.name.clone(),
                Type::Class(class.name.name.clone()),
                class.name.span,
            );
        }
        
        Item::Struct(struct_def) => {
            let fields: Vec<FieldInfo> = struct_def.fields.iter()
                .map(|f| {
                    let ty = ctx.resolve_type_expr(&f.ty);
                    FieldInfo {
                        name: f.name.name.clone(),
                        ty,
                        has_default: f.default.is_some(),
                    }
                })
                .collect();
            
            ctx.register_struct(StructInfo {
                name: struct_def.name.name.clone(),
                fields,
            });
            
            ctx.define_const(
                struct_def.name.name.clone(),
                Type::Struct(struct_def.name.name.clone()),
                struct_def.name.span,
            );
        }
        
        Item::Agent(agent) => {
            let mut tools = Vec::new();
            let mut tasks = Vec::new();
            
            // Check config for tools
            for config in &agent.config {
                if let AgentConfig::Tools(exprs) = config {
                    for expr in exprs {
                        if let Expr::Ident(ident) = expr {
                            tools.push(ident.name.clone());
                        }
                    }
                }
            }
            
            for member in &agent.members {
                match member {
                    AgentMember::Config(AgentConfig::Tools(exprs)) => {
                        for expr in exprs {
                            if let Expr::Ident(ident) = expr {
                                tools.push(ident.name.clone());
                            }
                        }
                    }
                    AgentMember::Task(task) => {
                        let param_types: Vec<Type> = task.params.iter()
                            .map(|p| {
                                if let Some(ref ty) = p.ty {
                                    ctx.resolve_type_expr(ty)
                                } else {
                                    ctx.fresh_type_var()
                                }
                            })
                            .collect();
                        
                        let return_ty = if let Some(ref ty) = task.return_type {
                            ctx.resolve_type_expr(ty)
                        } else {
                            ctx.fresh_type_var()
                        };
                        
                        let task_ty = Type::AsyncFunction {
                            params: param_types,
                            ret: Box::new(return_ty),
                        };
                        
                        tasks.push((task.name.name.clone(), task_ty));
                    }
                    _ => {}
                }
            }
            
            ctx.register_agent(AgentInfo {
                name: agent.name.name.clone(),
                model_type: None,
                tools,
                tasks,
            });
            
            ctx.define_const(
                agent.name.name.clone(),
                Type::Agent(agent.name.name.clone()),
                agent.name.span,
            );
        }
        
        Item::Tool(tool) => {
            let mut permissions = Vec::new();
            let mut execute_type = None;
            
            for member in &tool.members {
                match member {
                    ToolMember::Permissions(perms) => {
                        for perm in perms {
                            permissions.push(perm.name.clone());
                        }
                    }
                    ToolMember::Method(method) if method.name.name == "execute" => {
                        let param_types: Vec<Type> = method.params.iter()
                            .map(|p| {
                                if let Some(ref ty) = p.ty {
                                    ctx.resolve_type_expr(ty)
                                } else {
                                    Type::Any
                                }
                            })
                            .collect();
                        
                        let return_ty = if let Some(ref ty) = method.return_type {
                            ctx.resolve_type_expr(ty)
                        } else {
                            Type::Any
                        };
                        
                        execute_type = Some(Type::Function {
                            params: param_types,
                            ret: Box::new(return_ty),
                        });
                    }
                    _ => {}
                }
            }
            
            ctx.register_tool(ToolInfo {
                name: tool.name.name.clone(),
                permissions,
                execute_type,
            });
            
            ctx.define_const(
                tool.name.name.clone(),
                Type::Tool(tool.name.name.clone()),
                tool.name.span,
            );
        }
        
        Item::Memory(memory) => {
            ctx.define_const(
                memory.name.name.clone(),
                Type::Memory(memory.name.name.clone()),
                memory.name.span,
            );
        }
        
        Item::Prompt(prompt) => {
            ctx.define_const(
                prompt.name.name.clone(),
                Type::Prompt(prompt.name.name.clone()),
                prompt.name.span,
            );
        }
        
        Item::Enum(enum_def) => {
            ctx.define_const(
                enum_def.name.name.clone(),
                Type::Enum(enum_def.name.name.clone()),
                enum_def.name.span,
            );
        }
        
        Item::Const(_) | Item::Import(_) | Item::Stmt(_) => {
            // These are handled during check phase
        }
    }
}
