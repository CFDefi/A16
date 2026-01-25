//! Statement type checking

use crate::context::TypeContext;
use crate::error::TypeError;
use crate::types::Type;
use a16_ast::*;

impl TypeContext {
    /// Type check a statement
    pub fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(expr_stmt) => {
                self.check_expr(&expr_stmt.expr);
            }
            Stmt::Let(let_stmt) => self.check_let(let_stmt),
            Stmt::Assign(assign_stmt) => self.check_assign(assign_stmt),
            Stmt::AugAssign(aug_assign) => self.check_aug_assign(aug_assign),
            Stmt::Return(return_stmt) => self.check_return(return_stmt),
            Stmt::If(if_stmt) => self.check_if(if_stmt),
            Stmt::For(for_stmt) => self.check_for(for_stmt),
            Stmt::While(while_stmt) => self.check_while(while_stmt),
            Stmt::Match(match_stmt) => self.check_match(match_stmt),
            Stmt::Try(try_stmt) => self.check_try(try_stmt),
            Stmt::With(with_stmt) => self.check_with(with_stmt),
            Stmt::Raise(raise_stmt) => self.check_raise(raise_stmt),
            Stmt::Assert(assert_stmt) => self.check_assert(assert_stmt),
            Stmt::Break(_) => self.check_break(),
            Stmt::Continue(_) => self.check_continue(),
            Stmt::Pass(_) => { /* Pass is always valid */ }
            Stmt::Async(async_block) => self.check_async_block(async_block),
        }
    }
    
    /// Check a block of statements
    pub fn check_block(&mut self, block: &Block) {
        for stmt in &block.stmts {
            self.check_stmt(stmt);
        }
    }
    
    /// Check a let statement
    fn check_let(&mut self, let_stmt: &LetStmt) {
        // Get the initializer type if present
        let init_ty = if let Some(ref value) = let_stmt.value {
            self.check_expr(value)
        } else {
            self.fresh_type_var()
        };
        
        // Get declared type if present
        let declared_ty = if let Some(ref ty_expr) = let_stmt.ty {
            let ty = self.resolve_type_expr(ty_expr);
            if let_stmt.value.is_some() {
                let span = let_stmt.span;
                self.unify(&ty, &init_ty, span)
            } else {
                ty
            }
        } else {
            init_ty
        };
        
        // Bind the pattern
        let mutable = let_stmt.is_mutable && !let_stmt.is_const;
        self.bind_let_pattern(&let_stmt.pattern, &declared_ty, mutable);
    }
    
    /// Bind a let pattern to a type
    fn bind_let_pattern(&mut self, pattern: &Pattern, ty: &Type, mutable: bool) {
        match pattern {
            Pattern::Ident(ident) => {
                self.define(ident.name.clone(), ty.clone(), mutable, ident.span);
            }
            Pattern::Tuple(patterns, _) => {
                if let Type::Tuple(elem_types) = ty {
                    for (pat, elem_ty) in patterns.iter().zip(elem_types.iter()) {
                        self.bind_let_pattern(pat, elem_ty, mutable);
                    }
                } else {
                    for pat in patterns {
                        self.bind_let_pattern(pat, &Type::Any, mutable);
                    }
                }
            }
            Pattern::List(patterns, _) => {
                let elem_ty = ty.element_type().unwrap_or(Type::Any);
                for pat in patterns {
                    self.bind_let_pattern(pat, &elem_ty, mutable);
                }
            }
            Pattern::Rest(ident, _) => {
                self.define(ident.name.clone(), Type::List(Box::new(ty.clone())), mutable, ident.span);
            }
            Pattern::Wildcard(_) => {
                // No binding
            }
            Pattern::Literal(_) => {
                // Pattern matching on literal - no binding
            }
            Pattern::Class(class_pat) => {
                for pat in &class_pat.args {
                    self.bind_let_pattern(pat, &Type::Any, mutable);
                }
            }
            Pattern::Or(patterns, _) => {
                for pat in patterns {
                    self.bind_let_pattern(pat, ty, mutable);
                }
            }
        }
    }
    
    /// Check an assignment statement
    fn check_assign(&mut self, assign_stmt: &AssignStmt) {
        let value_ty = self.check_expr(&assign_stmt.value);
        
        // Check the target expression
        match &assign_stmt.target {
            Expr::Ident(ident) => {
                if let Some(sym) = self.lookup(&ident.name) {
                    if !sym.mutable {
                        self.error(TypeError::ImmutableAssign {
                            name: ident.name.to_string(),
                            def_span: miette::SourceSpan::new(
                                (sym.span.start as usize).into(),
                                (sym.span.end.saturating_sub(sym.span.start)) as usize,
                            ),
                            assign_span: miette::SourceSpan::new(
                                (ident.span.start as usize).into(),
                                (ident.span.end.saturating_sub(ident.span.start)) as usize,
                            ),
                        });
                    } else {
                        let sym_ty = sym.ty.clone();
                        self.unify(&sym_ty, &value_ty, assign_stmt.span);
                    }
                } else {
                    self.error(TypeError::undefined_var(&ident.name, ident.span));
                }
            }
            Expr::Attribute(attr) => {
                let _ = self.check_expr(&Expr::Attribute(attr.clone()));
            }
            Expr::Subscript(subscript) => {
                let _ = self.check_expr(&Expr::Subscript(subscript.clone()));
            }
            Expr::Tuple(tuple) => {
                // Tuple unpacking assignment
                for elem in &tuple.elements {
                    self.check_assign_target(elem, &value_ty);
                }
            }
            _ => {
                // Other expressions - just check them
                self.check_expr(&assign_stmt.target);
            }
        }
    }
    
    /// Check an assignment target expression
    fn check_assign_target(&mut self, target: &Expr, _value_ty: &Type) {
        match target {
            Expr::Ident(ident) => {
                if let Some(sym) = self.lookup(&ident.name) {
                    if !sym.mutable {
                        self.error(TypeError::ImmutableAssign {
                            name: ident.name.to_string(),
                            def_span: miette::SourceSpan::new(
                                (sym.span.start as usize).into(),
                                (sym.span.end.saturating_sub(sym.span.start)) as usize,
                            ),
                            assign_span: miette::SourceSpan::new(
                                (ident.span.start as usize).into(),
                                (ident.span.end.saturating_sub(ident.span.start)) as usize,
                            ),
                        });
                    }
                } else {
                    self.error(TypeError::undefined_var(&ident.name, ident.span));
                }
            }
            Expr::Attribute(attr) => {
                self.check_expr(&Expr::Attribute(attr.clone()));
            }
            Expr::Subscript(subscript) => {
                self.check_expr(&Expr::Subscript(subscript.clone()));
            }
            Expr::Tuple(tuple) => {
                for elem in &tuple.elements {
                    self.check_assign_target(elem, _value_ty);
                }
            }
            _ => {
                self.check_expr(target);
            }
        }
    }
    
    /// Check an augmented assignment (+=, -= etc)
    fn check_aug_assign(&mut self, aug_assign: &AugAssignStmt) {
        let target_ty = self.check_expr(&aug_assign.target);
        let value_ty = self.check_expr(&aug_assign.value);
        
        // Check mutability
        if let Expr::Ident(ident) = &aug_assign.target {
            if let Some(sym) = self.lookup(&ident.name) {
                if !sym.mutable {
                    self.error(TypeError::ImmutableAssign {
                        name: ident.name.to_string(),
                        def_span: miette::SourceSpan::new(
                            (sym.span.start as usize).into(),
                            (sym.span.end.saturating_sub(sym.span.start)) as usize,
                        ),
                        assign_span: miette::SourceSpan::new(
                            (ident.span.start as usize).into(),
                            (ident.span.end.saturating_sub(ident.span.start)) as usize,
                        ),
                    });
                }
            }
        }
        
        // Check that the operation is valid
        let span = aug_assign.span;
        match aug_assign.op {
            AugOp::Add => { self.check_add(&target_ty, &value_ty, span); }
            AugOp::Sub | AugOp::Mul => {
                self.check_numeric_op(&target_ty, &value_ty, &BinaryOp::Sub, span);
            }
            AugOp::Div => { self.check_div(&target_ty, &value_ty, span); }
            AugOp::FloorDiv | AugOp::Mod => {
                self.check_int_op(&target_ty, &value_ty, &BinaryOp::FloorDiv, span);
            }
            AugOp::Pow => { self.check_pow(&target_ty, &value_ty, span); }
            AugOp::BitAnd | AugOp::BitOr | AugOp::BitXor | AugOp::Shl | AugOp::Shr => {
                self.check_bitwise_op(&target_ty, &value_ty, &BinaryOp::BitAnd, span);
            }
        }
    }
    
    /// Check a return statement
    fn check_return(&mut self, return_stmt: &ReturnStmt) {
        let return_ty = if let Some(ref value) = return_stmt.value {
            self.check_expr(value)
        } else {
            Type::None
        };
        
        // Check against expected return type (clone to avoid borrow conflict)
        if let Some(expected) = self.expected_return_type().cloned() {
            let span = return_stmt.span;
            self.unify(&expected, &return_ty, span);
        }
    }
    
    /// Check an if statement
    fn check_if(&mut self, if_stmt: &IfStmt) {
        // Check condition
        self.check_expr(&if_stmt.condition);
        
        // Check then branch
        self.enter_scope();
        self.check_block(&if_stmt.then_block);
        self.exit_scope();
        
        // Check elif branches
        for (cond, block) in &if_stmt.elif_blocks {
            self.check_expr(cond);
            self.enter_scope();
            self.check_block(block);
            self.exit_scope();
        }
        
        // Check else branch
        if let Some(ref else_block) = if_stmt.else_block {
            self.enter_scope();
            self.check_block(else_block);
            self.exit_scope();
        }
    }
    
    /// Check a for loop
    fn check_for(&mut self, for_stmt: &ForStmt) {
        // Check the iterator expression
        let iter_ty = self.check_expr(&for_stmt.iter);
        let span = for_stmt.iter.span();
        
        // Get the element type
        let elem_ty = match &iter_ty {
            Type::List(elem) | Type::Set(elem) | Type::Iterator(elem) => (**elem).clone(),
            Type::Dict(key, _) => (**key).clone(),
            Type::Str => Type::Str,
            Type::Tuple(elems) if !elems.is_empty() => {
                if elems.len() == 1 {
                    elems[0].clone()
                } else {
                    Type::Union(elems.clone())
                }
            }
            Type::Any | Type::Unknown => Type::Any,
            Type::Error => Type::Error,
            _ => {
                self.error(TypeError::not_iterable(&iter_ty.to_string(), span));
                Type::Error
            }
        };
        
        // Enter loop scope and bind target
        self.enter_loop_scope();
        
        // Bind the loop variable pattern
        self.bind_pattern(&for_stmt.target, &elem_ty);
        
        // Check the body
        self.check_block(&for_stmt.body);
        
        self.exit_scope();
        
        // Check else block if present
        if let Some(ref else_block) = for_stmt.else_block {
            self.enter_scope();
            self.check_block(else_block);
            self.exit_scope();
        }
    }
    
    /// Check a while loop
    fn check_while(&mut self, while_stmt: &WhileStmt) {
        // Check condition
        self.check_expr(&while_stmt.condition);
        
        // Enter loop scope
        self.enter_loop_scope();
        self.check_block(&while_stmt.body);
        self.exit_scope();
        
        // Check else block if present
        if let Some(ref else_block) = while_stmt.else_block {
            self.enter_scope();
            self.check_block(else_block);
            self.exit_scope();
        }
    }
    
    /// Check a match statement
    fn check_match(&mut self, match_stmt: &MatchStmt) {
        let subject_ty = self.check_expr(&match_stmt.subject);
        
        for arm in &match_stmt.arms {
            self.enter_scope();
            
            // Check the pattern and bind pattern variables
            self.check_pattern(&arm.pattern, &subject_ty);
            
            // Check guard if present
            if let Some(ref guard) = arm.guard {
                self.check_expr(guard);
            }
            
            // Check the body
            self.check_block(&arm.body);
            
            self.exit_scope();
        }
    }
    
    /// Check a pattern and bind any pattern variables
    fn check_pattern(&mut self, pattern: &Pattern, expected_ty: &Type) {
        match pattern {
            Pattern::Wildcard(_) => {
                // Matches anything, no binding
            }
            Pattern::Ident(ident) => {
                self.define_let(ident.name.clone(), expected_ty.clone(), ident.span);
            }
            Pattern::Literal(lit) => {
                // Check the literal
                self.check_expr(lit);
            }
            Pattern::Tuple(patterns, _) => {
                if let Type::Tuple(elem_types) = expected_ty {
                    if patterns.len() == elem_types.len() {
                        for (pat, ty) in patterns.iter().zip(elem_types.iter()) {
                            self.check_pattern(pat, ty);
                        }
                    }
                }
            }
            Pattern::List(patterns, _) => {
                let elem_ty = expected_ty.element_type().unwrap_or(Type::Any);
                for pat in patterns {
                    self.check_pattern(pat, &elem_ty);
                }
            }
            Pattern::Rest(ident, _) => {
                self.define_let(ident.name.clone(), Type::List(Box::new(expected_ty.clone())), ident.span);
            }
            Pattern::Class(class_pat) => {
                for arg in &class_pat.args {
                    self.check_pattern(arg, &Type::Any);
                }
            }
            Pattern::Or(patterns, _) => {
                for pat in patterns {
                    self.check_pattern(pat, expected_ty);
                }
            }
        }
    }
    
    /// Check a try statement
    fn check_try(&mut self, try_stmt: &TryStmt) {
        // Check the try block
        self.enter_scope();
        self.check_block(&try_stmt.body);
        self.exit_scope();
        
        // Check each except handler
        for handler in &try_stmt.handlers {
            self.enter_scope();
            
            if let Some(ref exc_type) = handler.ty {
                let exc_ty = self.check_expr(exc_type);
                if let Some(ref name) = handler.name {
                    self.define_let(name.name.clone(), exc_ty, name.span);
                }
            }
            
            self.check_block(&handler.body);
            self.exit_scope();
        }
        
        // Check else block
        if let Some(ref else_block) = try_stmt.else_block {
            self.enter_scope();
            self.check_block(else_block);
            self.exit_scope();
        }
        
        // Check finally block
        if let Some(ref finally_block) = try_stmt.finally_block {
            self.enter_scope();
            self.check_block(finally_block);
            self.exit_scope();
        }
    }
    
    /// Check a with statement
    fn check_with(&mut self, with_stmt: &WithStmt) {
        self.enter_scope();
        
        for item in &with_stmt.items {
            let ctx_ty = self.check_expr(&item.context);
            
            if let Some(ref alias) = item.alias {
                self.define_let(alias.name.clone(), ctx_ty, alias.span);
            }
        }
        
        self.check_block(&with_stmt.body);
        self.exit_scope();
    }
    
    /// Check a raise statement
    fn check_raise(&mut self, raise_stmt: &RaiseStmt) {
        if let Some(ref exc) = raise_stmt.exception {
            self.check_expr(exc);
        }
        if let Some(ref cause) = raise_stmt.cause {
            self.check_expr(cause);
        }
    }
    
    /// Check an assert statement
    fn check_assert(&mut self, assert_stmt: &AssertStmt) {
        self.check_expr(&assert_stmt.test);
        if let Some(ref msg) = assert_stmt.msg {
            self.check_expr(msg);
        }
    }
    
    /// Check break statement (must be in loop)
    fn check_break(&mut self) {
        if !self.in_loop() {
            // Warning: break outside loop
        }
    }
    
    /// Check continue statement (must be in loop)
    fn check_continue(&mut self) {
        if !self.in_loop() {
            // Warning: continue outside loop
        }
    }
    
    /// Check an async block
    fn check_async_block(&mut self, async_block: &AsyncBlock) {
        self.enter_scope();
        
        for stmt in &async_block.stmts {
            self.check_stmt(stmt);
        }
        
        self.exit_scope();
    }
}
