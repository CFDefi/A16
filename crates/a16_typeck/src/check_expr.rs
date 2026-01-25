//! Expression type checking

use crate::context::TypeContext;
use crate::error::TypeError;
use crate::types::Type;
use a16_ast::*;
use smol_str::SmolStr;

impl TypeContext {
    /// Type check an expression and return its type
    pub fn check_expr(&mut self, expr: &Expr) -> Type {
        match expr {
            // Literals
            Expr::Int(_, _) => Type::Int,
            Expr::Float(_, _) => Type::Float,
            Expr::Bool(_, _) => Type::Bool,
            Expr::String(_, _) => Type::Str,
            Expr::FString(fstring) => self.check_fstring(fstring),
            Expr::None(_) => Type::None,
            
            // Identifiers
            Expr::Ident(ident) => self.check_ident(ident),
            
            // Operators
            Expr::Binary(binary) => self.check_binary(binary),
            Expr::Unary(unary) => self.check_unary(unary),
            Expr::Compare(compare) => self.check_compare(compare),
            
            // Conditionals
            Expr::IfExpr(if_expr) => self.check_if_expr(if_expr),
            
            // Collections
            Expr::List(list) => self.check_list(list),
            Expr::Dict(dict) => self.check_dict(dict),
            Expr::Set(set) => self.check_set(set),
            Expr::Tuple(tuple) => self.check_tuple(tuple),
            
            // Access
            Expr::Call(call) => self.check_call(call),
            Expr::Attribute(attr) => self.check_attribute(attr),
            Expr::Subscript(subscript) => self.check_subscript(subscript),
            
            // Lambda
            Expr::Lambda(lambda) => self.check_lambda(lambda),
            
            // Async
            Expr::Await(await_expr) => self.check_await(await_expr),
            Expr::Yield(yield_expr) => self.check_yield(yield_expr),
            Expr::Spawn(spawn_expr) => self.check_spawn(spawn_expr),
            
            // Comprehensions
            Expr::ListComp(comp) => self.check_list_comp(comp),
            Expr::DictComp(comp) => self.check_dict_comp(comp),
            Expr::SetComp(comp) => self.check_set_comp(comp),
            Expr::GeneratorExpr(comp) => self.check_generator(comp),
        }
    }
    
    /// Check an identifier reference
    fn check_ident(&mut self, ident: &Ident) -> Type {
        if let Some(sym) = self.lookup(&ident.name) {
            sym.ty.clone()
        } else {
            self.error(TypeError::undefined_var(&ident.name, ident.span));
            Type::Error
        }
    }
    
    /// Check an f-string
    fn check_fstring(&mut self, fstring: &FStringExpr) -> Type {
        // Check all interpolated expressions
        for part in &fstring.parts {
            if let FStringPart::Expr(expr) = part {
                self.check_expr(expr);
            }
        }
        Type::Str
    }
    
    /// Check a binary expression
    fn check_binary(&mut self, binary: &BinaryExpr) -> Type {
        let lhs_ty = self.check_expr(&binary.left);
        let rhs_ty = self.check_expr(&binary.right);
        let span = binary.span;
        
        // Propagate errors
        if lhs_ty.is_error() || rhs_ty.is_error() {
            return Type::Error;
        }
        
        match binary.op {
            // Arithmetic operators
            BinaryOp::Add => self.check_add(&lhs_ty, &rhs_ty, span),
            BinaryOp::Sub | BinaryOp::Mul => self.check_numeric_op(&lhs_ty, &rhs_ty, &binary.op, span),
            BinaryOp::Div => self.check_div(&lhs_ty, &rhs_ty, span),
            BinaryOp::FloorDiv | BinaryOp::Mod => self.check_int_op(&lhs_ty, &rhs_ty, &binary.op, span),
            BinaryOp::Pow => self.check_pow(&lhs_ty, &rhs_ty, span),
            
            // Bitwise operators
            BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor | 
            BinaryOp::Shl | BinaryOp::Shr => self.check_bitwise_op(&lhs_ty, &rhs_ty, &binary.op, span),
            
            // Logical operators
            BinaryOp::And | BinaryOp::Or => self.check_logical_op(&lhs_ty, &rhs_ty, span),
        }
    }
    
    /// Check addition (also handles string/list concatenation)
    pub fn check_add(&mut self, lhs: &Type, rhs: &Type, span: Span) -> Type {
        match (lhs, rhs) {
            (Type::Int, Type::Int) => Type::Int,
            (Type::Float, Type::Float) => Type::Float,
            (Type::Int, Type::Float) | (Type::Float, Type::Int) => Type::Float,
            (Type::Str, Type::Str) => Type::Str,
            (Type::List(e1), Type::List(e2)) => {
                let elem = self.unify(e1, e2, span);
                Type::List(Box::new(elem))
            }
            (Type::Any, _) | (_, Type::Any) => Type::Any,
            (Type::Unknown, _) | (_, Type::Unknown) => Type::Unknown,
            _ => {
                self.error(TypeError::invalid_binop("+", &lhs.to_string(), &rhs.to_string(), span));
                Type::Error
            }
        }
    }
    
    /// Check numeric operations (sub, mul)
    pub fn check_numeric_op(&mut self, lhs: &Type, rhs: &Type, op: &BinaryOp, span: Span) -> Type {
        match (lhs, rhs) {
            (Type::Int, Type::Int) => Type::Int,
            (Type::Float, Type::Float) => Type::Float,
            (Type::Int, Type::Float) | (Type::Float, Type::Int) => Type::Float,
            (Type::Any, _) | (_, Type::Any) => Type::Any,
            (Type::Unknown, _) | (_, Type::Unknown) => Type::Unknown,
            // String * Int for repetition
            (Type::Str, Type::Int) | (Type::Int, Type::Str) if matches!(op, BinaryOp::Mul) => Type::Str,
            // List * Int for repetition
            (Type::List(e), Type::Int) | (Type::Int, Type::List(e)) if matches!(op, BinaryOp::Mul) => {
                Type::List(e.clone())
            }
            _ => {
                let op_str = match op {
                    BinaryOp::Sub => "-",
                    BinaryOp::Mul => "*",
                    _ => "?",
                };
                self.error(TypeError::invalid_binop(op_str, &lhs.to_string(), &rhs.to_string(), span));
                Type::Error
            }
        }
    }
    
    /// Check division (always returns float)
    pub fn check_div(&mut self, lhs: &Type, rhs: &Type, span: Span) -> Type {
        match (lhs, rhs) {
            (Type::Int, Type::Int) | (Type::Float, Type::Float) |
            (Type::Int, Type::Float) | (Type::Float, Type::Int) => Type::Float,
            (Type::Any, _) | (_, Type::Any) => Type::Any,
            (Type::Unknown, _) | (_, Type::Unknown) => Type::Unknown,
            _ => {
                self.error(TypeError::invalid_binop("/", &lhs.to_string(), &rhs.to_string(), span));
                Type::Error
            }
        }
    }
    
    /// Check integer operations (floor div, mod)
    pub fn check_int_op(&mut self, lhs: &Type, rhs: &Type, op: &BinaryOp, span: Span) -> Type {
        match (lhs, rhs) {
            (Type::Int, Type::Int) => Type::Int,
            (Type::Float, Type::Float) | (Type::Int, Type::Float) | (Type::Float, Type::Int) => Type::Float,
            (Type::Any, _) | (_, Type::Any) => Type::Any,
            (Type::Unknown, _) | (_, Type::Unknown) => Type::Unknown,
            _ => {
                let op_str = match op {
                    BinaryOp::FloorDiv => "//",
                    BinaryOp::Mod => "%",
                    _ => "?",
                };
                self.error(TypeError::invalid_binop(op_str, &lhs.to_string(), &rhs.to_string(), span));
                Type::Error
            }
        }
    }
    
    /// Check power operation
    pub fn check_pow(&mut self, lhs: &Type, rhs: &Type, span: Span) -> Type {
        match (lhs, rhs) {
            (Type::Int, Type::Int) => Type::Int,
            (Type::Float, _) | (_, Type::Float) => Type::Float,
            (Type::Any, _) | (_, Type::Any) => Type::Any,
            (Type::Unknown, _) | (_, Type::Unknown) => Type::Unknown,
            _ => {
                self.error(TypeError::invalid_binop("**", &lhs.to_string(), &rhs.to_string(), span));
                Type::Error
            }
        }
    }
    
    /// Check bitwise operations
    pub fn check_bitwise_op(&mut self, lhs: &Type, rhs: &Type, op: &BinaryOp, span: Span) -> Type {
        match (lhs, rhs) {
            (Type::Int, Type::Int) => Type::Int,
            (Type::Bool, Type::Bool) => Type::Bool,
            (Type::Any, _) | (_, Type::Any) => Type::Any,
            (Type::Unknown, _) | (_, Type::Unknown) => Type::Unknown,
            _ => {
                let op_str = match op {
                    BinaryOp::BitAnd => "&",
                    BinaryOp::BitOr => "|",
                    BinaryOp::BitXor => "^",
                    BinaryOp::Shl => "<<",
                    BinaryOp::Shr => ">>",
                    _ => "?",
                };
                self.error(TypeError::invalid_binop(op_str, &lhs.to_string(), &rhs.to_string(), span));
                Type::Error
            }
        }
    }
    
    /// Check logical operations
    fn check_logical_op(&mut self, lhs: &Type, rhs: &Type, span: Span) -> Type {
        match (lhs, rhs) {
            (Type::Bool, Type::Bool) => Type::Bool,
            (Type::Any, _) | (_, Type::Any) => Type::Any,
            (Type::Unknown, _) | (_, Type::Unknown) => Type::Unknown,
            _ => {
                // Allow any type for truthiness, return the unified type
                self.unify(lhs, rhs, span)
            }
        }
    }
    
    /// Check unary expressions
    fn check_unary(&mut self, unary: &UnaryExpr) -> Type {
        let operand_ty = self.check_expr(&unary.operand);
        let span = unary.span;
        
        if operand_ty.is_error() {
            return Type::Error;
        }
        
        match unary.op {
            UnaryOp::Neg => match &operand_ty {
                Type::Int => Type::Int,
                Type::Float => Type::Float,
                Type::Any | Type::Unknown => operand_ty,
                _ => {
                    self.error(TypeError::invalid_unop("-", &operand_ty.to_string(), span));
                    Type::Error
                }
            },
            UnaryOp::Pos => match &operand_ty {
                Type::Int => Type::Int,
                Type::Float => Type::Float,
                Type::Any | Type::Unknown => operand_ty,
                _ => {
                    self.error(TypeError::invalid_unop("+", &operand_ty.to_string(), span));
                    Type::Error
                }
            },
            UnaryOp::Not => Type::Bool,
            UnaryOp::BitNot => match &operand_ty {
                Type::Int => Type::Int,
                Type::Any | Type::Unknown => Type::Int,
                _ => {
                    self.error(TypeError::invalid_unop("~", &operand_ty.to_string(), span));
                    Type::Error
                }
            },
        }
    }
    
    /// Check comparison expressions
    fn check_compare(&mut self, compare: &CompareExpr) -> Type {
        let _lhs_ty = self.check_expr(&compare.left);
        
        for (_, rhs) in &compare.comparisons {
            let _rhs_ty = self.check_expr(rhs);
        }
        
        Type::Bool
    }
    
    /// Check if expression (ternary)
    fn check_if_expr(&mut self, if_expr: &IfExprNode) -> Type {
        let _cond_ty = self.check_expr(&if_expr.condition);
        let then_ty = self.check_expr(&if_expr.then_expr);
        let else_ty = self.check_expr(&if_expr.else_expr);
        
        let span = if_expr.span;
        
        // Unify the two branches
        self.unify(&then_ty, &else_ty, span)
    }
    
    /// Check list literal
    fn check_list(&mut self, list: &ListExpr) -> Type {
        if list.elements.is_empty() {
            Type::List(Box::new(self.fresh_type_var()))
        } else {
            let mut elem_ty = self.check_expr(&list.elements[0]);
            let span = list.span;
            
            for elem in &list.elements[1..] {
                let ty = self.check_expr(elem);
                elem_ty = self.unify(&elem_ty, &ty, span);
            }
            
            Type::List(Box::new(elem_ty))
        }
    }
    
    /// Check dict literal
    fn check_dict(&mut self, dict: &DictExpr) -> Type {
        if dict.pairs.is_empty() {
            Type::Dict(Box::new(self.fresh_type_var()), Box::new(self.fresh_type_var()))
        } else {
            let mut key_ty = self.fresh_type_var();
            let mut val_ty = self.fresh_type_var();
            let span = dict.span;
            
            for (key, value) in &dict.pairs {
                let k_ty = self.check_expr(key);
                let v_ty = self.check_expr(value);
                key_ty = self.unify(&key_ty, &k_ty, span);
                val_ty = self.unify(&val_ty, &v_ty, span);
            }
            
            Type::Dict(Box::new(key_ty), Box::new(val_ty))
        }
    }
    
    /// Check set literal
    fn check_set(&mut self, set: &SetExpr) -> Type {
        if set.elements.is_empty() {
            Type::Set(Box::new(self.fresh_type_var()))
        } else {
            let mut elem_ty = self.check_expr(&set.elements[0]);
            let span = set.span;
            
            for elem in &set.elements[1..] {
                let ty = self.check_expr(elem);
                elem_ty = self.unify(&elem_ty, &ty, span);
            }
            
            Type::Set(Box::new(elem_ty))
        }
    }
    
    /// Check tuple literal
    fn check_tuple(&mut self, tuple: &TupleExpr) -> Type {
        let elem_types: Vec<Type> = tuple.elements.iter()
            .map(|e| self.check_expr(e))
            .collect();
        Type::Tuple(elem_types)
    }
    
    /// Check function call
    fn check_call(&mut self, call: &CallExpr) -> Type {
        let callee_ty = self.check_expr(&call.func);
        let span = call.span;
        
        // Get positional arguments
        let args: Vec<&Expr> = call.args.iter().map(|a| &a.value).collect();
        
        match callee_ty {
            Type::Function { params, ret } => {
                self.check_call_args(&params, &args, span);
                *ret
            }
            Type::AsyncFunction { params, ret } => {
                self.check_call_args(&params, &args, span);
                *ret
            }
            Type::Class(name) | Type::Struct(name) => {
                Type::Class(name)
            }
            Type::Agent(name) => {
                Type::Agent(name)
            }
            Type::Any | Type::Unknown => Type::Any,
            Type::Error => Type::Error,
            _ => {
                self.error(TypeError::not_callable(&callee_ty.to_string(), span));
                Type::Error
            }
        }
    }
    
    /// Check function call arguments
    fn check_call_args(&mut self, params: &[Type], args: &[&Expr], span: Span) {
        if args.len() != params.len() {
            if !params.iter().any(|p| matches!(p, Type::Any)) {
                self.error(TypeError::wrong_args(params.len(), args.len(), span));
            }
        }
        
        for (param, arg) in params.iter().zip(args.iter()) {
            let arg_ty = self.check_expr(arg);
            self.unify(param, &arg_ty, arg.span());
        }
    }
    
    /// Check attribute access (obj.attr)
    fn check_attribute(&mut self, attr: &AttributeExpr) -> Type {
        let obj_ty = self.check_expr(&attr.value);
        let member = &attr.attr.name;
        let span = attr.span;
        
        match &obj_ty {
            Type::Class(name) | Type::Struct(name) => {
                if let Some(class_info) = self.lookup_class(name) {
                    for (method_name, method_ty) in &class_info.methods {
                        if method_name == member {
                            return method_ty.clone();
                        }
                    }
                    for field in &class_info.fields {
                        if &field.name == member {
                            return field.ty.clone();
                        }
                    }
                }
                if let Some(struct_info) = self.lookup_struct(name) {
                    for field in &struct_info.fields {
                        if &field.name == member {
                            return field.ty.clone();
                        }
                    }
                }
                self.error(TypeError::no_member(&obj_ty.to_string(), member, span));
                Type::Error
            }
            Type::Agent(name) => {
                if let Some(agent_info) = self.lookup_agent(name) {
                    for (task_name, task_ty) in &agent_info.tasks {
                        if task_name == member {
                            return task_ty.clone();
                        }
                    }
                }
                Type::Any
            }
            Type::List(_) => self.check_list_method(member, &obj_ty, span),
            Type::Dict(_, _) => self.check_dict_method(member, &obj_ty, span),
            Type::Str => self.check_str_method(member, span),
            Type::Any | Type::Unknown => Type::Any,
            Type::Error => Type::Error,
            _ => {
                self.error(TypeError::no_member(&obj_ty.to_string(), member, span));
                Type::Error
            }
        }
    }
    
    /// Check list methods
    fn check_list_method(&self, method: &str, list_ty: &Type, _span: Span) -> Type {
        let elem_ty = if let Type::List(e) = list_ty { (**e).clone() } else { Type::Any };
        
        match method {
            "append" => Type::Function {
                params: vec![elem_ty],
                ret: Box::new(Type::None),
            },
            "extend" => Type::Function {
                params: vec![Type::List(Box::new(elem_ty))],
                ret: Box::new(Type::None),
            },
            "pop" => Type::Function {
                params: vec![],
                ret: Box::new(elem_ty),
            },
            "clear" => Type::Function {
                params: vec![],
                ret: Box::new(Type::None),
            },
            "copy" => Type::Function {
                params: vec![],
                ret: Box::new(list_ty.clone()),
            },
            _ => Type::Any,
        }
    }
    
    /// Check dict methods
    fn check_dict_method(&self, method: &str, dict_ty: &Type, _span: Span) -> Type {
        let (key_ty, val_ty) = if let Type::Dict(k, v) = dict_ty { 
            ((**k).clone(), (**v).clone()) 
        } else { 
            (Type::Any, Type::Any) 
        };
        
        match method {
            "get" => Type::Function {
                params: vec![key_ty.clone()],
                ret: Box::new(Type::Optional(Box::new(val_ty.clone()))),
            },
            "keys" => Type::Function {
                params: vec![],
                ret: Box::new(Type::Iterator(Box::new(key_ty))),
            },
            "values" => Type::Function {
                params: vec![],
                ret: Box::new(Type::Iterator(Box::new(val_ty))),
            },
            "items" => Type::Function {
                params: vec![],
                ret: Box::new(Type::Iterator(Box::new(Type::Tuple(vec![key_ty, val_ty])))),
            },
            _ => Type::Any,
        }
    }
    
    /// Check string methods
    fn check_str_method(&self, method: &str, _span: Span) -> Type {
        match method {
            "upper" | "lower" | "strip" | "title" | "capitalize" => {
                Type::Function { params: vec![], ret: Box::new(Type::Str) }
            }
            "split" => Type::Function {
                params: vec![Type::Str],
                ret: Box::new(Type::List(Box::new(Type::Str))),
            },
            "join" => Type::Function {
                params: vec![Type::List(Box::new(Type::Str))],
                ret: Box::new(Type::Str),
            },
            "replace" => Type::Function {
                params: vec![Type::Str, Type::Str],
                ret: Box::new(Type::Str),
            },
            "startswith" | "endswith" => {
                Type::Function { params: vec![Type::Str], ret: Box::new(Type::Bool) }
            }
            "find" => Type::Function {
                params: vec![Type::Str],
                ret: Box::new(Type::Int),
            },
            _ => Type::Any,
        }
    }
    
    /// Check subscript access (obj[index])
    fn check_subscript(&mut self, subscript: &SubscriptExpr) -> Type {
        let obj_ty = self.check_expr(&subscript.value);
        let index_ty = self.check_expr(&subscript.index);
        let span = subscript.span;
        
        match &obj_ty {
            Type::List(elem) => {
                if !matches!(index_ty, Type::Int | Type::Any | Type::Unknown) {
                    self.error(TypeError::not_indexable(&obj_ty.to_string(), &index_ty.to_string(), span));
                }
                (**elem).clone()
            }
            Type::Dict(key, val) => {
                self.unify(key, &index_ty, span);
                (**val).clone()
            }
            Type::Tuple(elems) => {
                if elems.is_empty() {
                    Type::Never
                } else if elems.len() == 1 {
                    elems[0].clone()
                } else {
                    Type::Any
                }
            }
            Type::Str => {
                if !matches!(index_ty, Type::Int | Type::Any | Type::Unknown) {
                    self.error(TypeError::not_indexable(&obj_ty.to_string(), &index_ty.to_string(), span));
                }
                Type::Str
            }
            Type::Any | Type::Unknown => Type::Any,
            Type::Error => Type::Error,
            _ => {
                self.error(TypeError::not_indexable(&obj_ty.to_string(), &index_ty.to_string(), span));
                Type::Error
            }
        }
    }
    
    /// Check lambda expression
    fn check_lambda(&mut self, lambda: &LambdaExpr) -> Type {
        self.enter_scope();
        
        let mut param_types = Vec::new();
        for param in &lambda.params {
            let param_ty = if let Some(ty_expr) = &param.ty {
                self.resolve_type_expr(ty_expr)
            } else {
                self.fresh_type_var()
            };
            param_types.push(param_ty.clone());
            self.define_let(param.name.name.clone(), param_ty, param.name.span);
        }
        
        let return_ty = self.check_expr(&lambda.body);
        
        self.exit_scope();
        
        Type::Function {
            params: param_types,
            ret: Box::new(return_ty),
        }
    }
    
    /// Check await expression
    fn check_await(&mut self, await_expr: &AwaitExpr) -> Type {
        let inner_ty = self.check_expr(&await_expr.value);
        inner_ty
    }
    
    /// Check yield expression
    fn check_yield(&mut self, yield_expr: &YieldExpr) -> Type {
        if let Some(ref value) = yield_expr.value {
            self.check_expr(value)
        } else {
            Type::None
        }
    }
    
    /// Check spawn expression
    fn check_spawn(&mut self, spawn_expr: &SpawnExpr) -> Type {
        let _task_ty = self.check_expr(&spawn_expr.value);
        Type::Any
    }
    
    /// Check list comprehension
    fn check_list_comp(&mut self, comp: &ComprehensionExpr) -> Type {
        self.enter_scope();
        
        // Process each generator
        for gen in &comp.generators {
            let iter_ty = self.check_expr(&gen.iter);
            let elem_ty = iter_ty.element_type().unwrap_or(Type::Any);
            self.bind_pattern(&gen.target, &elem_ty);
            
            // Check conditions
            for cond in &gen.conditions {
                self.check_expr(cond);
            }
        }
        
        // Check the element expression
        let result_elem_ty = self.check_expr(&comp.element);
        
        self.exit_scope();
        
        Type::List(Box::new(result_elem_ty))
    }
    
    /// Check dict comprehension
    fn check_dict_comp(&mut self, comp: &DictComprehensionExpr) -> Type {
        self.enter_scope();
        
        for gen in &comp.generators {
            let iter_ty = self.check_expr(&gen.iter);
            let elem_ty = iter_ty.element_type().unwrap_or(Type::Any);
            self.bind_pattern(&gen.target, &elem_ty);
            
            for cond in &gen.conditions {
                self.check_expr(cond);
            }
        }
        
        let key_ty = self.check_expr(&comp.key);
        let value_ty = self.check_expr(&comp.value);
        
        self.exit_scope();
        
        Type::Dict(Box::new(key_ty), Box::new(value_ty))
    }
    
    /// Check set comprehension
    fn check_set_comp(&mut self, comp: &ComprehensionExpr) -> Type {
        self.enter_scope();
        
        for gen in &comp.generators {
            let iter_ty = self.check_expr(&gen.iter);
            let elem_ty = iter_ty.element_type().unwrap_or(Type::Any);
            self.bind_pattern(&gen.target, &elem_ty);
            
            for cond in &gen.conditions {
                self.check_expr(cond);
            }
        }
        
        let result_elem_ty = self.check_expr(&comp.element);
        
        self.exit_scope();
        
        Type::Set(Box::new(result_elem_ty))
    }
    
    /// Check generator expression
    fn check_generator(&mut self, comp: &ComprehensionExpr) -> Type {
        self.enter_scope();
        
        for gen in &comp.generators {
            let iter_ty = self.check_expr(&gen.iter);
            let elem_ty = iter_ty.element_type().unwrap_or(Type::Any);
            self.bind_pattern(&gen.target, &elem_ty);
            
            for cond in &gen.conditions {
                self.check_expr(cond);
            }
        }
        
        let result_elem_ty = self.check_expr(&comp.element);
        
        self.exit_scope();
        
        Type::Iterator(Box::new(result_elem_ty))
    }
    
    /// Bind a pattern to a type
    pub fn bind_pattern(&mut self, pattern: &Pattern, ty: &Type) {
        match pattern {
            Pattern::Ident(ident) => {
                self.define_let(ident.name.clone(), ty.clone(), ident.span);
            }
            Pattern::Tuple(patterns, _) => {
                if let Type::Tuple(elem_types) = ty {
                    for (pat, elem_ty) in patterns.iter().zip(elem_types.iter()) {
                        self.bind_pattern(pat, elem_ty);
                    }
                } else {
                    for pat in patterns {
                        self.bind_pattern(pat, &Type::Any);
                    }
                }
            }
            Pattern::List(patterns, _) => {
                let elem_ty = ty.element_type().unwrap_or(Type::Any);
                for pat in patterns {
                    self.bind_pattern(pat, &elem_ty);
                }
            }
            Pattern::Rest(ident, _) => {
                self.define_let(ident.name.clone(), Type::List(Box::new(ty.clone())), ident.span);
            }
            Pattern::Wildcard(_) => {
                // No binding
            }
            Pattern::Literal(_) => {
                // No binding
            }
            Pattern::Class(class_pat) => {
                for pat in &class_pat.args {
                    self.bind_pattern(pat, &Type::Any);
                }
            }
            Pattern::Or(patterns, _) => {
                // Bind all patterns in the or
                for pat in patterns {
                    self.bind_pattern(pat, ty);
                }
            }
        }
    }
    
    /// Resolve a type expression from the AST to a Type
    pub fn resolve_type_expr(&mut self, type_expr: &TypeExpr) -> Type {
        match type_expr {
            TypeExpr::Name(ident) => self.resolve_simple_type(&ident.name),
            TypeExpr::Generic(generic) => {
                let base_name = &generic.name.name;
                let type_args: Vec<Type> = generic.args.iter()
                    .map(|arg| self.resolve_type_expr(arg))
                    .collect();
                
                match base_name.as_str() {
                    "List" if type_args.len() == 1 => Type::List(Box::new(type_args[0].clone())),
                    "Dict" if type_args.len() == 2 => Type::Dict(
                        Box::new(type_args[0].clone()),
                        Box::new(type_args[1].clone()),
                    ),
                    "Set" if type_args.len() == 1 => Type::Set(Box::new(type_args[0].clone())),
                    "Tuple" => Type::Tuple(type_args),
                    "Optional" if type_args.len() == 1 => Type::Optional(Box::new(type_args[0].clone())),
                    "Result" if type_args.len() == 2 => Type::Result(
                        Box::new(type_args[0].clone()),
                        Box::new(type_args[1].clone()),
                    ),
                    "Schema" if type_args.len() == 1 => Type::Schema(Box::new(type_args[0].clone())),
                    "Iterator" if type_args.len() == 1 => Type::Iterator(Box::new(type_args[0].clone())),
                    _ => Type::Class(base_name.clone()),
                }
            }
            TypeExpr::Union(types, _) => {
                let resolved: Vec<Type> = types.iter()
                    .map(|t| self.resolve_type_expr(t))
                    .collect();
                if resolved.len() == 1 {
                    resolved[0].clone()
                } else {
                    Type::Union(resolved)
                }
            }
            TypeExpr::Optional(inner, _) => {
                Type::Optional(Box::new(self.resolve_type_expr(inner)))
            }
            TypeExpr::Callable(callable) => {
                let param_types: Vec<Type> = callable.params.iter()
                    .map(|p| self.resolve_type_expr(p))
                    .collect();
                let ret_type = self.resolve_type_expr(&callable.return_type);
                Type::Function {
                    params: param_types,
                    ret: Box::new(ret_type),
                }
            }
            TypeExpr::Tuple(types, _) => {
                let resolved: Vec<Type> = types.iter()
                    .map(|t| self.resolve_type_expr(t))
                    .collect();
                Type::Tuple(resolved)
            }
        }
    }
    
    /// Resolve a simple type name
    fn resolve_simple_type(&self, name: &str) -> Type {
        match name {
            "Int" => Type::Int,
            "Float" => Type::Float,
            "Bool" => Type::Bool,
            "Str" => Type::Str,
            "Bytes" => Type::Bytes,
            "None" => Type::None,
            "Any" => Type::Any,
            "Never" => Type::Never,
            "Message" => Type::Message,
            "Context" => Type::Context,
            "TokenBudget" => Type::TokenBudget,
            "ModelResponse" => Type::ModelResponse,
            "Embedding" => Type::Embedding,
            _ => {
                if self.lookup_struct(name).is_some() {
                    Type::Struct(SmolStr::new(name))
                } else if self.lookup_class(name).is_some() {
                    Type::Class(SmolStr::new(name))
                } else if self.lookup_agent(name).is_some() {
                    Type::Agent(SmolStr::new(name))
                } else {
                    Type::Class(SmolStr::new(name))
                }
            }
        }
    }
}
