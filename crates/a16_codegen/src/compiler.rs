//! HIR to Bytecode Compiler

use a16_hir::*;
use smol_str::SmolStr;
use indexmap::IndexMap;

use crate::bytecode::*;

/// Compiler context
struct Compiler {
    module: BytecodeModule,
    current_code: Vec<u8>,
    current_locals: IndexMap<VarId, u8>,
    next_local: u8,
    loop_starts: Vec<usize>,
    loop_ends: Vec<Vec<usize>>,
}

impl Compiler {
    fn new() -> Self {
        Self {
            module: BytecodeModule::new(),
            current_code: Vec::new(),
            current_locals: IndexMap::new(),
            next_local: 0,
            loop_starts: Vec::new(),
            loop_ends: Vec::new(),
        }
    }
    
    fn emit(&mut self, op: Opcode) {
        self.current_code.push(op as u8);
    }
    
    fn emit_u8(&mut self, value: u8) {
        self.current_code.push(value);
    }
    
    fn emit_u16(&mut self, value: u16) {
        self.current_code.push((value >> 8) as u8);
        self.current_code.push((value & 0xFF) as u8);
    }
    
    fn emit_i16(&mut self, value: i16) {
        self.emit_u16(value as u16);
    }
    
    fn current_offset(&self) -> usize {
        self.current_code.len()
    }
    
    fn patch_jump(&mut self, offset: usize) {
        let target = self.current_offset() as i16 - offset as i16 - 2;
        self.current_code[offset] = (target >> 8) as u8;
        self.current_code[offset + 1] = (target & 0xFF) as u8;
    }
    
    fn add_constant(&mut self, constant: Constant) -> u16 {
        let idx = self.module.constants.len() as u16;
        self.module.constants.push(constant);
        idx
    }
    
    fn add_string_constant(&mut self, s: SmolStr) -> u16 {
        // Check if already exists
        for (i, c) in self.module.constants.iter().enumerate() {
            if let Constant::Str(ref existing) = c {
                if existing == &s {
                    return i as u16;
                }
            }
        }
        self.add_constant(Constant::Str(s))
    }
    
    fn get_local(&mut self, id: VarId) -> u8 {
        if let Some(&slot) = self.current_locals.get(&id) {
            slot
        } else {
            let slot = self.next_local;
            self.next_local += 1;
            self.current_locals.insert(id, slot);
            slot
        }
    }
}

/// Compile HIR module to bytecode
pub fn compile(hir: &HirModule) -> BytecodeModule {
    let mut compiler = Compiler::new();
    
    // Compile all functions
    for func in &hir.functions {
        compile_function(&mut compiler, func);
    }
    
    // Compile agent tasks
    for agent in &hir.agents {
        for task in &agent.tasks {
            compile_function(&mut compiler, task);
        }
    }
    
    // Compile tool execute functions
    for tool in &hir.tools {
        if let Some(ref execute) = tool.execute {
            compile_function(&mut compiler, execute);
        }
    }
    
    compiler.module
}

fn compile_function(compiler: &mut Compiler, func: &HirFunction) {
    // Reset per-function state
    compiler.current_code = Vec::new();
    compiler.current_locals = IndexMap::new();
    compiler.next_local = 0;
    
    // Allocate locals for parameters
    for param in &func.params {
        compiler.get_local(param.id);
    }
    
    // Compile body
    compile_block(compiler, &func.body);
    
    // Ensure we return
    if compiler.current_code.last() != Some(&(Opcode::Return as u8)) {
        compiler.emit(Opcode::PushNone);
        compiler.emit(Opcode::Return);
    }
    
    compiler.module.functions.push(BytecodeFunction {
        name: func.name.clone(),
        arity: func.params.len() as u8,
        locals: compiler.next_local,
        code: std::mem::take(&mut compiler.current_code),
    });
}

fn compile_block(compiler: &mut Compiler, block: &HirBlock) {
    for stmt in &block.stmts {
        compile_stmt(compiler, stmt);
    }
}

fn compile_stmt(compiler: &mut Compiler, stmt: &HirStmt) {
    match stmt {
        HirStmt::Let { id, value, .. } => {
            if let Some(init) = value {
                compile_expr(compiler, init);
            } else {
                compiler.emit(Opcode::PushNone);
            }
            let slot = compiler.get_local(*id);
            compiler.emit(Opcode::StoreLocal);
            compiler.emit_u8(slot);
        }
        
        HirStmt::Assign { target, value } => {
            compile_expr(compiler, value);
            match target {
                HirExpr::Var(id) => {
                    let slot = compiler.get_local(*id);
                    compiler.emit(Opcode::StoreLocal);
                    compiler.emit_u8(slot);
                }
                HirExpr::Attr { object, name } => {
                    compile_expr(compiler, object);
                    let idx = compiler.add_string_constant(name.clone());
                    compiler.emit(Opcode::SetAttr);
                    compiler.emit_u16(idx);
                }
                HirExpr::Index { object, index } => {
                    compile_expr(compiler, object);
                    compile_expr(compiler, index);
                    compiler.emit(Opcode::SetIndex);
                }
                _ => {}
            }
        }
        
        HirStmt::Expr(expr) => {
            compile_expr(compiler, expr);
            compiler.emit(Opcode::Pop);
        }
        
        HirStmt::Return(value) => {
            if let Some(expr) = value {
                compile_expr(compiler, expr);
            } else {
                compiler.emit(Opcode::PushNone);
            }
            compiler.emit(Opcode::Return);
        }
        
        HirStmt::If { condition, then_block, else_block } => {
            compile_expr(compiler, condition);
            compiler.emit(Opcode::JumpIfFalse);
            let else_jump = compiler.current_offset();
            compiler.emit_i16(0); // Placeholder
            compiler.emit(Opcode::Pop); // Pop condition value
            
            compile_block(compiler, then_block);
            
            if let Some(else_blk) = else_block {
                compiler.emit(Opcode::Jump);
                let end_jump = compiler.current_offset();
                compiler.emit_i16(0); // Placeholder
                
                compiler.patch_jump(else_jump);
                compiler.emit(Opcode::Pop); // Pop condition value in else branch
                compile_block(compiler, else_blk);
                compiler.patch_jump(end_jump);
            } else {
                compiler.patch_jump(else_jump);
                compiler.emit(Opcode::Pop); // Pop condition value when no else
            }
        }
        
        HirStmt::Loop { condition, body, .. } => {
            let loop_start = compiler.current_offset();
            compiler.loop_starts.push(loop_start);
            compiler.loop_ends.push(Vec::new());
            
            if let Some(cond) = condition {
                compile_expr(compiler, cond);
                compiler.emit(Opcode::JumpIfFalse);
                let end_jump = compiler.current_offset();
                compiler.emit_i16(0);
                compiler.loop_ends.last_mut().unwrap().push(end_jump);
            }
            
            compile_block(compiler, body);
            
            // Jump back to start
            compiler.emit(Opcode::Jump);
            let back_offset = compiler.current_offset() as i16 - loop_start as i16 - 2;
            compiler.emit_i16(-back_offset - 3);
            
            // Patch all break jumps
            let end_jumps = compiler.loop_ends.pop().unwrap();
            for jump in end_jumps {
                compiler.patch_jump(jump);
            }
            compiler.loop_starts.pop();
        }
        
        HirStmt::ForEach { var, iter, body, .. } => {
            // Compile iterator
            compile_expr(compiler, iter);
            compiler.emit(Opcode::GetIter);
            
            let slot = compiler.get_local(*var);
            
            let loop_start = compiler.current_offset();
            compiler.loop_starts.push(loop_start);
            compiler.loop_ends.push(Vec::new());
            
            // ForIter pushes next value or jumps to end
            compiler.emit(Opcode::ForIter);
            let end_jump = compiler.current_offset();
            compiler.emit_i16(0);
            
            // Store in loop variable
            compiler.emit(Opcode::StoreLocal);
            compiler.emit_u8(slot);
            
            compile_block(compiler, body);
            
            // Jump back to ForIter
            compiler.emit(Opcode::Jump);
            let current = compiler.current_offset();
            // Calculate relative offset: jump from after the i16 to loop_start
            let offset = loop_start as i16 - (current as i16 + 2);
            compiler.emit_i16(offset);
            
            // Patch end jump
            compiler.patch_jump(end_jump);
            let end_jumps = compiler.loop_ends.pop().unwrap();
            for jump in end_jumps {
                compiler.patch_jump(jump);
            }
            compiler.loop_starts.pop();
            
            // Pop iterator
            compiler.emit(Opcode::Pop);
        }
        
        HirStmt::Match { subject, arms } => {
            compile_expr(compiler, subject);
            let mut end_jumps = Vec::new();
            
            for arm in arms {
                compiler.emit(Opcode::Dup);
                // TODO: Compile pattern matching properly
                // For now, just compare with literal patterns
                match &arm.pattern {
                    HirPattern::Literal(lit) => {
                        compile_literal(compiler, lit);
                        compiler.emit(Opcode::Eq);
                    }
                    HirPattern::Var(id, _) => {
                        let slot = compiler.get_local(*id);
                        compiler.emit(Opcode::StoreLocal);
                        compiler.emit_u8(slot);
                        compiler.emit(Opcode::PushTrue);
                    }
                    HirPattern::Tuple(sub_pats) => {
                        // Nested tuple destructuring in match
                        // For now, treat as wildcard match
                        let _ = sub_pats;
                        compiler.emit(Opcode::PushTrue);
                    }
                    HirPattern::Constructor(name, _sub_pats) => {
                        // Constructor match: compare type name
                        let idx = compiler.add_string_constant(name.clone());
                        compiler.emit(Opcode::PushConst);
                        compiler.emit_u16(idx);
                        compiler.emit(Opcode::Eq);
                    }
                    HirPattern::Or(alternatives) => {
                        // Or-pattern: try each alternative
                        // For first alternative, check literal if it is one
                        if let Some(first) = alternatives.first() {
                            if let HirPattern::Literal(lit) = first {
                                compile_literal(compiler, lit);
                                compiler.emit(Opcode::Eq);
                            } else {
                                compiler.emit(Opcode::PushTrue);
                            }
                        } else {
                            compiler.emit(Opcode::PushTrue);
                        }
                    }
                    _ => {
                        compiler.emit(Opcode::PushTrue);
                    }
                }
                
                compiler.emit(Opcode::JumpIfFalse);
                let skip = compiler.current_offset();
                compiler.emit_i16(0);
                
                compiler.emit(Opcode::Pop); // Pop subject
                compile_block(compiler, &arm.body);
                compiler.emit(Opcode::Jump);
                end_jumps.push(compiler.current_offset());
                compiler.emit_i16(0);
                
                compiler.patch_jump(skip);
            }
            
            compiler.emit(Opcode::Pop); // Pop subject if no match
            
            for jump in end_jumps {
                compiler.patch_jump(jump);
            }
        }
        
        HirStmt::Break => {
            compiler.emit(Opcode::Jump);
            let jump = compiler.current_offset();
            compiler.emit_i16(0);
            if let Some(breaks) = compiler.loop_ends.last_mut() {
                breaks.push(jump);
            }
        }
        
        HirStmt::Continue => {
            if let Some(&start) = compiler.loop_starts.last() {
                compiler.emit(Opcode::Jump);
                let back_offset = compiler.current_offset() as i16 - start as i16 - 2;
                compiler.emit_i16(-back_offset - 3);
            }
        }
    }
}

fn compile_expr(compiler: &mut Compiler, expr: &HirExpr) {
    match expr {
        HirExpr::Literal(lit) => compile_literal(compiler, lit),
        
        HirExpr::Var(id) => {
            let slot = compiler.get_local(*id);
            compiler.emit(Opcode::LoadLocal);
            compiler.emit_u8(slot);
        }
        
        HirExpr::Global(id) => {
            compiler.emit(Opcode::LoadGlobal);
            compiler.emit_u16(*id as u16);
        }
        
        HirExpr::GlobalRef(name) => {
            // Emit LoadGlobalByName with name as constant
            let idx = compiler.add_string_constant(name.clone());
            compiler.emit(Opcode::LoadGlobalByName);
            compiler.emit_u16(idx);
        }
        
        HirExpr::Binary { op, left, right } => {
            // Handle short-circuit operators specially
            match op {
                HirBinaryOp::And => {
                    compile_expr(compiler, left);
                    compiler.emit(Opcode::JumpIfFalse);
                    let skip = compiler.current_offset();
                    compiler.emit_i16(0);
                    compiler.emit(Opcode::Pop);
                    compile_expr(compiler, right);
                    compiler.patch_jump(skip);
                    return;
                }
                HirBinaryOp::Or => {
                    compile_expr(compiler, left);
                    compiler.emit(Opcode::JumpIfTrue);
                    let skip = compiler.current_offset();
                    compiler.emit_i16(0);
                    compiler.emit(Opcode::Pop);
                    compile_expr(compiler, right);
                    compiler.patch_jump(skip);
                    return;
                }
                _ => {}
            }
            
            // Regular binary operators
            compile_expr(compiler, left);
            compile_expr(compiler, right);
            let opcode = match op {
                HirBinaryOp::Add => Opcode::Add,
                HirBinaryOp::Sub => Opcode::Sub,
                HirBinaryOp::Mul => Opcode::Mul,
                HirBinaryOp::Div => Opcode::Div,
                HirBinaryOp::FloorDiv => Opcode::FloorDiv,
                HirBinaryOp::Mod => Opcode::Mod,
                HirBinaryOp::Pow => Opcode::Pow,
                HirBinaryOp::BitAnd => Opcode::BitAnd,
                HirBinaryOp::BitOr => Opcode::BitOr,
                HirBinaryOp::BitXor => Opcode::BitXor,
                HirBinaryOp::Shl => Opcode::Shl,
                HirBinaryOp::Shr => Opcode::Shr,
                HirBinaryOp::Eq => Opcode::Eq,
                HirBinaryOp::Ne => Opcode::Ne,
                HirBinaryOp::Lt => Opcode::Lt,
                HirBinaryOp::Le => Opcode::Le,
                HirBinaryOp::Gt => Opcode::Gt,
                HirBinaryOp::Ge => Opcode::Ge,
                HirBinaryOp::In | HirBinaryOp::NotIn => Opcode::Eq,
                HirBinaryOp::And | HirBinaryOp::Or => unreachable!(),
            };
            compiler.emit(opcode);
        }
        
        HirExpr::Unary { op, operand } => {
            compile_expr(compiler, operand);
            let opcode = match op {
                HirUnaryOp::Neg => Opcode::Neg,
                HirUnaryOp::Not => Opcode::Not,
                HirUnaryOp::BitNot => Opcode::BitNot,
            };
            compiler.emit(opcode);
        }
        
        HirExpr::Call { func, args } => {
            compile_expr(compiler, func);
            for arg in args {
                compile_expr(compiler, arg);
            }
            compiler.emit(Opcode::Call);
            compiler.emit_u8(args.len() as u8);
        }
        
        HirExpr::Attr { object, name } => {
            compile_expr(compiler, object);
            let idx = compiler.add_string_constant(name.clone());
            compiler.emit(Opcode::GetAttr);
            compiler.emit_u16(idx);
        }
        
        HirExpr::Index { object, index } => {
            compile_expr(compiler, object);
            compile_expr(compiler, index);
            compiler.emit(Opcode::GetIndex);
        }
        
        HirExpr::List(elements) => {
            for elem in elements {
                compile_expr(compiler, elem);
            }
            compiler.emit(Opcode::BuildList);
            compiler.emit_u16(elements.len() as u16);
        }
        
        HirExpr::Dict(pairs) => {
            for (key, value) in pairs {
                compile_expr(compiler, key);
                compile_expr(compiler, value);
            }
            compiler.emit(Opcode::BuildDict);
            compiler.emit_u16(pairs.len() as u16);
        }
        
        HirExpr::Tuple(elements) => {
            for elem in elements {
                compile_expr(compiler, elem);
            }
            compiler.emit(Opcode::BuildTuple);
            compiler.emit_u16(elements.len() as u16);
        }
        
        HirExpr::Lambda { params: _, body } => {
            // Simple lambda: compile body inline (no upvalue capture)
            compile_expr(compiler, body);
        }
        
        HirExpr::Closure { func_idx, upvalues } => {
            // Full closure: emit MakeClosure with upvalue capture list
            compiler.emit(Opcode::MakeClosure);
            compiler.emit_u16(*func_idx);
            compiler.emit_u8(upvalues.len() as u8);
            for uv in upvalues {
                compiler.emit_u8(if uv.is_local { 1 } else { 0 });
                compiler.emit_u8(uv.index);
            }
        }
        
        HirExpr::IfExpr { condition, then_expr, else_expr } => {
            compile_expr(compiler, condition);
            compiler.emit(Opcode::JumpIfFalse);
            let else_jump = compiler.current_offset();
            compiler.emit_i16(0);
            
            compile_expr(compiler, then_expr);
            compiler.emit(Opcode::Jump);
            let end_jump = compiler.current_offset();
            compiler.emit_i16(0);
            
            compiler.patch_jump(else_jump);
            compile_expr(compiler, else_expr);
            compiler.patch_jump(end_jump);
        }
        
        HirExpr::Await(inner) => {
            compile_expr(compiler, inner);
            compiler.emit(Opcode::Await);
        }
        
        HirExpr::Spawn(inner) => {
            compile_expr(compiler, inner);
            compiler.emit(Opcode::Spawn);
        }
        
        HirExpr::ModelInvoke { model, prompt, .. } => {
            compile_expr(compiler, model);
            compile_expr(compiler, prompt);
            compiler.emit(Opcode::ModelInvoke);
        }
        
        HirExpr::ToolDispatch { tool, args } => {
            compile_expr(compiler, tool);
            for arg in args {
                compile_expr(compiler, arg);
            }
            compiler.emit(Opcode::ToolDispatch);
        }
        
        HirExpr::MemoryStore { memory, content, .. } => {
            compile_expr(compiler, memory);
            compile_expr(compiler, content);
            compiler.emit(Opcode::MemoryStore);
        }
        
        HirExpr::MemoryRetrieve { memory, query, .. } => {
            compile_expr(compiler, memory);
            compile_expr(compiler, query);
            compiler.emit(Opcode::MemoryRetrieve);
        }
    }
}

fn compile_literal(compiler: &mut Compiler, lit: &HirLiteral) {
    match lit {
        HirLiteral::None => compiler.emit(Opcode::PushNone),
        HirLiteral::Bool(true) => compiler.emit(Opcode::PushTrue),
        HirLiteral::Bool(false) => compiler.emit(Opcode::PushFalse),
        HirLiteral::Int(0) => compiler.emit(Opcode::PushInt0),
        HirLiteral::Int(1) => compiler.emit(Opcode::PushInt1),
        HirLiteral::Int(n) => {
            let idx = compiler.add_constant(Constant::Int(*n));
            compiler.emit(Opcode::PushConst);
            compiler.emit_u16(idx);
        }
        HirLiteral::Float(n) => {
            let idx = compiler.add_constant(Constant::Float(*n));
            compiler.emit(Opcode::PushConst);
            compiler.emit_u16(idx);
        }
        HirLiteral::Str(s) => {
            let idx = compiler.add_string_constant(s.clone());
            compiler.emit(Opcode::PushConst);
            compiler.emit_u16(idx);
        }
    }
}
