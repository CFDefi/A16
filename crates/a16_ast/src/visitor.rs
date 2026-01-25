//! Visitor pattern for AST traversal

use crate::*;

/// Visitor trait for traversing the AST
pub trait Visitor {
    type Result;
    
    fn visit_module(&mut self, module: &Module) -> Self::Result;
    fn visit_item(&mut self, item: &Item) -> Self::Result;
    fn visit_function(&mut self, func: &FunctionDef) -> Self::Result;
    fn visit_class(&mut self, class: &ClassDef) -> Self::Result;
    fn visit_agent(&mut self, agent: &AgentDef) -> Self::Result;
    fn visit_tool(&mut self, tool: &ToolDef) -> Self::Result;
    fn visit_stmt(&mut self, stmt: &Stmt) -> Self::Result;
    fn visit_expr(&mut self, expr: &Expr) -> Self::Result;
    fn visit_pattern(&mut self, pattern: &Pattern) -> Self::Result;
    fn visit_type(&mut self, ty: &TypeExpr) -> Self::Result;
}

/// Default implementation that visits all children
#[allow(dead_code)]
pub trait VisitorMut {
    fn visit_module(&mut self, module: &mut Module) {
        for item in &mut module.items {
            self.visit_item(item);
        }
    }
    
    fn visit_item(&mut self, item: &mut Item) {
        match item {
            Item::Function(f) => self.visit_function(f),
            Item::Class(c) => self.visit_class(c),
            Item::Agent(a) => self.visit_agent(a),
            Item::Tool(t) => self.visit_tool(t),
            Item::Stmt(s) => self.visit_stmt(s),
            _ => {}
        }
    }
    
    fn visit_function(&mut self, func: &mut FunctionDef) {
        self.visit_block(&mut func.body);
    }
    
    fn visit_class(&mut self, _class: &mut ClassDef) {}
    fn visit_agent(&mut self, _agent: &mut AgentDef) {}
    fn visit_tool(&mut self, _tool: &mut ToolDef) {}
    
    fn visit_block(&mut self, block: &mut Block) {
        for stmt in &mut block.stmts {
            self.visit_stmt(stmt);
        }
    }
    
    fn visit_stmt(&mut self, stmt: &mut Stmt) {
        match stmt {
            Stmt::Expr(e) => self.visit_expr(&mut e.expr),
            Stmt::Let(l) => {
                if let Some(v) = &mut l.value {
                    self.visit_expr(v);
                }
            }
            Stmt::Assign(a) => {
                self.visit_expr(&mut a.target);
                self.visit_expr(&mut a.value);
            }
            Stmt::Return(r) => {
                if let Some(v) = &mut r.value {
                    self.visit_expr(v);
                }
            }
            Stmt::If(i) => {
                self.visit_expr(&mut i.condition);
                self.visit_block(&mut i.then_block);
                for (cond, block) in &mut i.elif_blocks {
                    self.visit_expr(cond);
                    self.visit_block(block);
                }
                if let Some(b) = &mut i.else_block {
                    self.visit_block(b);
                }
            }
            Stmt::For(f) => {
                self.visit_expr(&mut f.iter);
                self.visit_block(&mut f.body);
            }
            Stmt::While(w) => {
                self.visit_expr(&mut w.condition);
                self.visit_block(&mut w.body);
            }
            _ => {}
        }
    }
    
    fn visit_expr(&mut self, expr: &mut Expr) {
        match expr {
            Expr::Binary(b) => {
                self.visit_expr(&mut b.left);
                self.visit_expr(&mut b.right);
            }
            Expr::Unary(u) => {
                self.visit_expr(&mut u.operand);
            }
            Expr::Call(c) => {
                self.visit_expr(&mut c.func);
                for arg in &mut c.args {
                    self.visit_expr(&mut arg.value);
                }
            }
            Expr::Attribute(a) => {
                self.visit_expr(&mut a.value);
            }
            Expr::Subscript(s) => {
                self.visit_expr(&mut s.value);
                self.visit_expr(&mut s.index);
            }
            Expr::List(l) => {
                for e in &mut l.elements {
                    self.visit_expr(e);
                }
            }
            Expr::Await(a) => {
                self.visit_expr(&mut a.value);
            }
            _ => {}
        }
    }
}
