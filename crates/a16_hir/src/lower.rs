//! AST to HIR Lowering
//!
//! Converts the abstract syntax tree to the high-level intermediate representation.

use a16_ast::*;
use smol_str::SmolStr;
use indexmap::IndexMap;

use crate::nodes::*;

/// Context for lowering
pub struct LoweringContext {
    next_var_id: VarId,
    next_func_id: FuncId,
    var_map: IndexMap<SmolStr, VarId>,
    scopes: Vec<IndexMap<SmolStr, VarId>>,
}

impl LoweringContext {
    pub fn new() -> Self {
        Self {
            next_var_id: 0,
            next_func_id: 0,
            var_map: IndexMap::new(),
            scopes: vec![IndexMap::new()],
        }
    }
    
    fn fresh_var(&mut self, name: SmolStr) -> VarId {
        let id = self.next_var_id;
        self.next_var_id += 1;
        self.scopes.last_mut().unwrap().insert(name.clone(), id);
        self.var_map.insert(name, id);
        id
    }
    
    fn fresh_func(&mut self) -> FuncId {
        let id = self.next_func_id;
        self.next_func_id += 1;
        id
    }
    
    fn lookup_var(&self, name: &str) -> Option<VarId> {
        for scope in self.scopes.iter().rev() {
            if let Some(&id) = scope.get(name) {
                return Some(id);
            }
        }
        None
    }
    
    fn enter_scope(&mut self) {
        self.scopes.push(IndexMap::new());
    }
    
    fn exit_scope(&mut self) {
        self.scopes.pop();
    }
}

impl Default for LoweringContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Lower a module from AST to HIR
pub fn lower_module(module: &Module) -> HirModule {
    let mut ctx = LoweringContext::new();
    let mut functions = Vec::new();
    let mut globals = Vec::new();
    let mut agents = Vec::new();
    let mut tools = Vec::new();
    
    for item in &module.items {
        match item {
            Item::Function(f) => {
                functions.push(lower_function(&mut ctx, f));
            }
            Item::Agent(a) => {
                agents.push(lower_agent(&mut ctx, a));
            }
            Item::Tool(t) => {
                tools.push(lower_tool(&mut ctx, t));
            }
            Item::Stmt(Stmt::Let(let_stmt)) => {
                if let Pattern::Ident(ref ident) = let_stmt.pattern {
                    let id = ctx.fresh_var(ident.name.clone());
                    globals.push(HirGlobal {
                        id,
                        name: ident.name.clone(),
                        init: let_stmt.value.as_ref().map(|e| lower_expr(&mut ctx, e)),
                    });
                }
            }
            _ => {
                // Other items handled later
            }
        }
    }
    
    HirModule {
        functions,
        globals,
        agents,
        tools,
    }
}

fn lower_function(ctx: &mut LoweringContext, func: &FunctionDef) -> HirFunction {
    let id = ctx.fresh_func();
    ctx.enter_scope();
    
    let params: Vec<HirParam> = func.params.iter().map(|p| {
        let var_id = ctx.fresh_var(p.name.name.clone());
        HirParam {
            id: var_id,
            name: p.name.name.clone(),
        }
    }).collect();
    
    let body = lower_block(ctx, &func.body);
    
    ctx.exit_scope();
    
    HirFunction {
        id,
        name: func.name.name.clone(),
        params,
        body,
        is_async: func.is_async,
    }
}

fn lower_agent(ctx: &mut LoweringContext, agent: &AgentDef) -> HirAgent {
    let mut model = None;
    let mut tools_exprs = Vec::new();
    let mut tasks = Vec::new();
    
    for member in &agent.members {
        match member {
            AgentMember::Config(config) => {
                match config {
                    AgentConfig::Model(expr) => {
                        model = Some(lower_expr(ctx, expr));
                    }
                    AgentConfig::Tools(exprs) => {
                        tools_exprs = exprs.iter().map(|e| lower_expr(ctx, e)).collect();
                    }
                    _ => {}
                }
            }
            AgentMember::Task(task) => {
                // Lower task as function
                ctx.enter_scope();
                let params: Vec<HirParam> = task.params.iter().map(|p| {
                    let var_id = ctx.fresh_var(p.name.name.clone());
                    HirParam {
                        id: var_id,
                        name: p.name.name.clone(),
                    }
                }).collect();
                let body = lower_block(ctx, &task.body);
                ctx.exit_scope();
                
                tasks.push(HirFunction {
                    id: ctx.fresh_func(),
                    name: task.name.name.clone(),
                    params,
                    body,
                    is_async: task.is_async,
                });
            }
            _ => {}
        }
    }
    
    HirAgent {
        name: agent.name.name.clone(),
        model,
        tools: tools_exprs,
        tasks,
    }
}

fn lower_tool(ctx: &mut LoweringContext, tool: &ToolDef) -> HirTool {
    let mut permissions = Vec::new();
    let mut execute = None;
    
    for member in &tool.members {
        match member {
            ToolMember::Permissions(perms) => {
                permissions = perms.iter().map(|p| p.name.clone()).collect();
            }
            ToolMember::Method(func) => {
                if func.name.name.as_str() == "execute" {
                    execute = Some(lower_function(ctx, func));
                }
            }
            _ => {}
        }
    }
    
    HirTool {
        name: tool.name.name.clone(),
        permissions,
        execute,
    }
}

fn lower_block(ctx: &mut LoweringContext, block: &Block) -> HirBlock {
    let stmts = block.stmts.iter().map(|s| lower_stmt(ctx, s)).collect();
    HirBlock { stmts }
}

fn lower_stmt(ctx: &mut LoweringContext, stmt: &Stmt) -> HirStmt {
    match stmt {
        Stmt::Let(let_stmt) => {
            if let Pattern::Ident(ref ident) = let_stmt.pattern {
                let id = ctx.fresh_var(ident.name.clone());
                HirStmt::Let {
                    id,
                    name: ident.name.clone(),
                    value: let_stmt.value.as_ref().map(|e| lower_expr(ctx, e)),
                }
            } else {
                // TODO: Handle pattern destructuring
                HirStmt::Expr(HirExpr::Literal(HirLiteral::None))
            }
        }
        Stmt::Assign(assign) => {
            HirStmt::Assign {
                target: lower_expr(ctx, &assign.target),
                value: lower_expr(ctx, &assign.value),
            }
        }
        Stmt::AugAssign(aug) => {
            // Desugar x += y to x = x + y
            let target = lower_expr(ctx, &aug.target);
            let value = lower_expr(ctx, &aug.value);
            let op = match aug.op {
                AugOp::Add => HirBinaryOp::Add,
                AugOp::Sub => HirBinaryOp::Sub,
                AugOp::Mul => HirBinaryOp::Mul,
                AugOp::Div => HirBinaryOp::Div,
                AugOp::FloorDiv => HirBinaryOp::FloorDiv,
                AugOp::Mod => HirBinaryOp::Mod,
                AugOp::Pow => HirBinaryOp::Pow,
                AugOp::BitAnd => HirBinaryOp::BitAnd,
                AugOp::BitOr => HirBinaryOp::BitOr,
                AugOp::BitXor => HirBinaryOp::BitXor,
                AugOp::Shl => HirBinaryOp::Shl,
                AugOp::Shr => HirBinaryOp::Shr,
            };
            HirStmt::Assign {
                target: target.clone(),
                value: HirExpr::Binary {
                    op,
                    left: Box::new(target),
                    right: Box::new(value),
                },
            }
        }
        Stmt::Expr(expr_stmt) => {
            HirStmt::Expr(lower_expr(ctx, &expr_stmt.expr))
        }
        Stmt::Return(ret) => {
            HirStmt::Return(ret.value.as_ref().map(|e| lower_expr(ctx, e)))
        }
        Stmt::If(if_stmt) => {
            // Desugar elif chains into nested if-else
            let else_block = lower_elif_chain(ctx, &if_stmt.elif_blocks, &if_stmt.else_block);
            HirStmt::If {
                condition: lower_expr(ctx, &if_stmt.condition),
                then_block: lower_block(ctx, &if_stmt.then_block),
                else_block,
            }
        }
        Stmt::For(for_stmt) => {
            if let Pattern::Ident(ref ident) = for_stmt.target {
                let var_id = ctx.fresh_var(ident.name.clone());
                HirStmt::ForEach {
                    var: var_id,
                    name: ident.name.clone(),
                    iter: lower_expr(ctx, &for_stmt.iter),
                    body: lower_block(ctx, &for_stmt.body),
                }
            } else {
                // TODO: Handle pattern destructuring
                HirStmt::Expr(HirExpr::Literal(HirLiteral::None))
            }
        }
        Stmt::While(while_stmt) => {
            HirStmt::Loop {
                init: None,
                condition: Some(lower_expr(ctx, &while_stmt.condition)),
                update: None,
                body: lower_block(ctx, &while_stmt.body),
            }
        }
        Stmt::Match(match_stmt) => {
            let arms = match_stmt.arms.iter().map(|arm| {
                ctx.enter_scope();
                let pattern = lower_pattern(ctx, &arm.pattern);
                let guard = arm.guard.as_ref().map(|e| lower_expr(ctx, e));
                let body = lower_block(ctx, &arm.body);
                ctx.exit_scope();
                HirMatchArm { pattern, guard, body }
            }).collect();
            HirStmt::Match {
                subject: lower_expr(ctx, &match_stmt.subject),
                arms,
            }
        }
        Stmt::Break(_) => HirStmt::Break,
        Stmt::Continue(_) => HirStmt::Continue,
        _ => {
            // Pass, Try, With, Raise, Assert, Async - handle later
            HirStmt::Expr(HirExpr::Literal(HirLiteral::None))
        }
    }
}

fn lower_elif_chain(
    ctx: &mut LoweringContext,
    elif_blocks: &[(Expr, Block)],
    else_block: &Option<Block>,
) -> Option<HirBlock> {
    if elif_blocks.is_empty() {
        return else_block.as_ref().map(|b| lower_block(ctx, b));
    }
    
    let (cond, block) = &elif_blocks[0];
    let nested_else = lower_elif_chain(ctx, &elif_blocks[1..], else_block);
    
    Some(HirBlock {
        stmts: vec![HirStmt::If {
            condition: lower_expr(ctx, cond),
            then_block: lower_block(ctx, block),
            else_block: nested_else,
        }],
    })
}

fn lower_pattern(ctx: &mut LoweringContext, pattern: &Pattern) -> HirPattern {
    match pattern {
        Pattern::Ident(ident) => {
            let id = ctx.fresh_var(ident.name.clone());
            HirPattern::Var(id, ident.name.clone())
        }
        Pattern::Tuple(patterns, _) => {
            HirPattern::Tuple(patterns.iter().map(|p| lower_pattern(ctx, p)).collect())
        }
        Pattern::Literal(expr) => {
            match expr {
                Expr::Int(n, _) => HirPattern::Literal(HirLiteral::Int(*n)),
                Expr::Float(n, _) => HirPattern::Literal(HirLiteral::Float(*n)),
                Expr::Bool(b, _) => HirPattern::Literal(HirLiteral::Bool(*b)),
                Expr::String(s, _) => HirPattern::Literal(HirLiteral::Str(s.clone())),
                Expr::None(_) => HirPattern::Literal(HirLiteral::None),
                _ => HirPattern::Wildcard,
            }
        }
        _ => HirPattern::Wildcard,
    }
}

fn lower_expr(ctx: &mut LoweringContext, expr: &Expr) -> HirExpr {
    match expr {
        Expr::Int(n, _) => HirExpr::Literal(HirLiteral::Int(*n)),
        Expr::Float(n, _) => HirExpr::Literal(HirLiteral::Float(*n)),
        Expr::Bool(b, _) => HirExpr::Literal(HirLiteral::Bool(*b)),
        Expr::String(s, _) => HirExpr::Literal(HirLiteral::Str(s.clone())),
        Expr::None(_) => HirExpr::Literal(HirLiteral::None),
        
        Expr::Ident(ident) => {
            if let Some(id) = ctx.lookup_var(ident.name.as_str()) {
                HirExpr::Var(id)
            } else {
                // Global reference by name (stdlib, builtins, or runtime-resolved)
                HirExpr::GlobalRef(ident.name.clone())
            }
        }
        
        Expr::Binary(bin) => {
            let op = match bin.op {
                BinaryOp::Add => HirBinaryOp::Add,
                BinaryOp::Sub => HirBinaryOp::Sub,
                BinaryOp::Mul => HirBinaryOp::Mul,
                BinaryOp::Div => HirBinaryOp::Div,
                BinaryOp::FloorDiv => HirBinaryOp::FloorDiv,
                BinaryOp::Mod => HirBinaryOp::Mod,
                BinaryOp::Pow => HirBinaryOp::Pow,
                BinaryOp::BitAnd => HirBinaryOp::BitAnd,
                BinaryOp::BitOr => HirBinaryOp::BitOr,
                BinaryOp::BitXor => HirBinaryOp::BitXor,
                BinaryOp::Shl => HirBinaryOp::Shl,
                BinaryOp::Shr => HirBinaryOp::Shr,
                BinaryOp::And => HirBinaryOp::And,
                BinaryOp::Or => HirBinaryOp::Or,
            };
            HirExpr::Binary {
                op,
                left: Box::new(lower_expr(ctx, &bin.left)),
                right: Box::new(lower_expr(ctx, &bin.right)),
            }
        }
        
        Expr::Unary(un) => {
            let op = match un.op {
                UnaryOp::Neg => HirUnaryOp::Neg,
                UnaryOp::Not => HirUnaryOp::Not,
                UnaryOp::Pos => return lower_expr(ctx, &un.operand),
                UnaryOp::BitNot => HirUnaryOp::BitNot,
            };
            HirExpr::Unary {
                op,
                operand: Box::new(lower_expr(ctx, &un.operand)),
            }
        }
        
        Expr::Call(call) => {
            HirExpr::Call {
                func: Box::new(lower_expr(ctx, &call.func)),
                args: call.args.iter().map(|a| lower_expr(ctx, &a.value)).collect(),
            }
        }
        
        Expr::Attribute(attr) => {
            HirExpr::Attr {
                object: Box::new(lower_expr(ctx, &attr.value)),
                name: attr.attr.name.clone(),
            }
        }
        
        Expr::Subscript(sub) => {
            HirExpr::Index {
                object: Box::new(lower_expr(ctx, &sub.value)),
                index: Box::new(lower_expr(ctx, &sub.index)),
            }
        }
        
        Expr::List(list) => {
            HirExpr::List(list.elements.iter().map(|e| lower_expr(ctx, e)).collect())
        }
        
        Expr::Dict(dict) => {
            HirExpr::Dict(
                dict.pairs.iter()
                    .map(|(k, v)| (lower_expr(ctx, k), lower_expr(ctx, v)))
                    .collect()
            )
        }
        
        Expr::Tuple(tuple) => {
            HirExpr::Tuple(tuple.elements.iter().map(|e| lower_expr(ctx, e)).collect())
        }
        
        Expr::Lambda(lambda) => {
            ctx.enter_scope();
            let params: Vec<HirParam> = lambda.params.iter().map(|p| {
                let id = ctx.fresh_var(p.name.name.clone());
                HirParam { id, name: p.name.name.clone() }
            }).collect();
            let body = lower_expr(ctx, &lambda.body);
            ctx.exit_scope();
            HirExpr::Lambda {
                params,
                body: Box::new(body),
            }
        }
        
        Expr::IfExpr(if_expr) => {
            HirExpr::IfExpr {
                condition: Box::new(lower_expr(ctx, &if_expr.condition)),
                then_expr: Box::new(lower_expr(ctx, &if_expr.then_expr)),
                else_expr: Box::new(lower_expr(ctx, &if_expr.else_expr)),
            }
        }
        
        Expr::Await(await_expr) => {
            HirExpr::Await(Box::new(lower_expr(ctx, &await_expr.value)))
        }
        
        Expr::Spawn(spawn_expr) => {
            HirExpr::Spawn(Box::new(lower_expr(ctx, &spawn_expr.value)))
        }
        
        Expr::Compare(cmp) => {
            // Lower chained comparisons: a < b < c → (a < b) and (b < c)
            if cmp.comparisons.len() == 1 {
                let (op, rhs) = &cmp.comparisons[0];
                HirExpr::Binary {
                    op: compare_op_to_hir(*op),
                    left: Box::new(lower_expr(ctx, &cmp.left)),
                    right: Box::new(lower_expr(ctx, rhs)),
                }
            } else {
                // Chain: a < b < c → (a < b) and (b < c)
                let mut result = {
                    let (op, rhs) = &cmp.comparisons[0];
                    HirExpr::Binary {
                        op: compare_op_to_hir(*op),
                        left: Box::new(lower_expr(ctx, &cmp.left)),
                        right: Box::new(lower_expr(ctx, rhs)),
                    }
                };
                
                for i in 1..cmp.comparisons.len() {
                    let (_prev_op, prev_rhs) = &cmp.comparisons[i - 1];
                    let (op, rhs) = &cmp.comparisons[i];
                    let new_cmp = HirExpr::Binary {
                        op: compare_op_to_hir(*op),
                        left: Box::new(lower_expr(ctx, prev_rhs)),
                        right: Box::new(lower_expr(ctx, rhs)),
                    };
                    result = HirExpr::Binary {
                        op: HirBinaryOp::And,
                        left: Box::new(result),
                        right: Box::new(new_cmp),
                    };
                }
                
                result
            }
        }
        
        _ => {
            // Set, ListComp, DictComp, SetComp, etc. - handle later
            HirExpr::Literal(HirLiteral::None)
        }
    }
}

fn compare_op_to_hir(op: CompareOp) -> HirBinaryOp {
    match op {
        CompareOp::Eq => HirBinaryOp::Eq,
        CompareOp::Ne => HirBinaryOp::Ne,
        CompareOp::Lt => HirBinaryOp::Lt,
        CompareOp::Le => HirBinaryOp::Le,
        CompareOp::Gt => HirBinaryOp::Gt,
        CompareOp::Ge => HirBinaryOp::Ge,
        CompareOp::In => HirBinaryOp::In,
        CompareOp::Is => HirBinaryOp::Eq, // Simplify Is to Eq for now
        CompareOp::NotIn => HirBinaryOp::NotIn,
        CompareOp::IsNot => HirBinaryOp::Ne, // Simplify IsNot to Ne
    }
}
