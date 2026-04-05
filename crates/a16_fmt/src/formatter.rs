//! AST-based source code formatter

use a16_ast::*;
use crate::config::FormatConfig;

/// A16 source formatter
pub struct Formatter {
    config: FormatConfig,
    buf: String,
    indent: usize,
}

impl Formatter {
    /// Create a new formatter
    pub fn new(config: FormatConfig) -> Self {
        Self {
            config,
            buf: String::new(),
            indent: 0,
        }
    }

    /// Get the formatted output
    pub fn output(self) -> String {
        let mut out = self.buf;
        if self.config.trailing_newline && !out.ends_with('\n') {
            out.push('\n');
        }
        out
    }

    /// Write indentation
    fn write_indent(&mut self) {
        for _ in 0..self.indent * self.config.indent_size {
            self.buf.push(' ');
        }
    }

    /// Write a line with indentation
    fn write_line(&mut self, text: &str) {
        self.write_indent();
        self.buf.push_str(text);
        self.buf.push('\n');
    }

    /// Write text without indent or newline
    fn write(&mut self, text: &str) {
        self.buf.push_str(text);
    }

    /// Format a complete module
    pub fn format_module(&mut self, module: &Module) {
        for (i, item) in module.items.iter().enumerate() {
            if i > 0 {
                self.buf.push('\n');
            }
            self.format_item(item);
        }
    }

    /// Format a top-level item
    fn format_item(&mut self, item: &Item) {
        match item {
            Item::Function(func) => self.format_function(func),
            Item::Class(class) => self.format_class(class),
            Item::Agent(agent) => self.format_agent(agent),
            Item::Struct(s) => self.format_struct(s),
            Item::Enum(e) => self.format_enum(e),
            Item::Import(import) => self.format_import(import),
            Item::Const(c) => self.format_const(c),
            Item::Extern(ext) => self.format_extern(ext),
            Item::Stmt(stmt) => self.format_stmt(stmt),
            _ => {
                // Tool, Memory, Prompt — emit as comments for now
                self.write_line("# <unformatted item>");
            }
        }
    }

    /// Format a function definition
    fn format_function(&mut self, func: &FunctionDef) {
        self.write_indent();
        if func.is_async {
            self.write("async ");
        }
        self.write("fn ");
        self.write(func.name.name.as_str());
        self.write("(");
        self.format_params(&func.params);
        self.write(")");
        if let Some(ref ret) = func.return_type {
            self.write(" -> ");
            self.format_type_expr(ret);
        }
        self.write(":\n");
        self.indent += 1;
        self.format_block(&func.body);
        self.indent -= 1;
    }

    /// Format parameters
    fn format_params(&mut self, params: &[Param]) {
        for (i, param) in params.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            self.write(param.name.name.as_str());
            if let Some(ref ty) = param.ty {
                self.write(": ");
                self.format_type_expr(ty);
            }
            if let Some(ref default) = param.default {
                self.write(" = ");
                self.format_expr(default);
            }
        }
    }

    /// Format a type expression
    fn format_type_expr(&mut self, ty: &TypeExpr) {
        match ty {
            TypeExpr::Name(ident) => self.write(ident.name.as_str()),
            TypeExpr::Generic(g) => {
                self.write(g.name.name.as_str());
                self.write("[");
                for (i, arg) in g.args.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.format_type_expr(arg);
                }
                self.write("]");
            }
            TypeExpr::Union(types, _) => {
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        self.write(" | ");
                    }
                    self.format_type_expr(ty);
                }
            }
            TypeExpr::Optional(inner, _) => {
                self.write("Optional[");
                self.format_type_expr(inner);
                self.write("]");
            }
            TypeExpr::Callable(c) => {
                self.write("(");
                for (i, p) in c.params.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.format_type_expr(p);
                }
                self.write(") -> ");
                self.format_type_expr(&c.return_type);
            }
            TypeExpr::Tuple(types, _) => {
                self.write("Tuple[");
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.format_type_expr(ty);
                }
                self.write("]");
            }
        }
    }

    /// Format a block of statements
    fn format_block(&mut self, block: &Block) {
        if block.stmts.is_empty() {
            self.write_line("pass");
        } else {
            for stmt in &block.stmts {
                self.format_stmt(stmt);
            }
        }
    }

    /// Format a statement
    fn format_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let(let_stmt) => {
                self.write_indent();
                self.write("let ");
                self.format_pattern(&let_stmt.pattern);
                if let Some(ref ty) = let_stmt.ty {
                    self.write(": ");
                    self.format_type_expr(ty);
                }
                if let Some(ref val) = let_stmt.value {
                    self.write(" = ");
                    self.format_expr(val);
                }
                self.write("\n");
            }
            Stmt::Return(ret) => {
                self.write_indent();
                self.write("return");
                if let Some(ref val) = ret.value {
                    self.write(" ");
                    self.format_expr(val);
                }
                self.write("\n");
            }
            Stmt::Expr(expr_stmt) => {
                self.write_indent();
                self.format_expr(&expr_stmt.expr);
                self.write("\n");
            }
            Stmt::If(if_stmt) => {
                self.write_indent();
                self.write("if ");
                self.format_expr(&if_stmt.condition);
                self.write(":\n");
                self.indent += 1;
                self.format_block(&if_stmt.then_block);
                self.indent -= 1;
                if let Some(ref else_block) = if_stmt.else_block {
                    self.write_indent();
                    self.write("else:\n");
                    self.indent += 1;
                    self.format_block(else_block);
                    self.indent -= 1;
                }
            }
            Stmt::For(for_stmt) => {
                self.write_indent();
                self.write("for ");
                self.format_pattern(&for_stmt.target);
                self.write(" in ");
                self.format_expr(&for_stmt.iter);
                self.write(":\n");
                self.indent += 1;
                self.format_block(&for_stmt.body);
                self.indent -= 1;
            }
            Stmt::While(while_stmt) => {
                self.write_indent();
                self.write("while ");
                self.format_expr(&while_stmt.condition);
                self.write(":\n");
                self.indent += 1;
                self.format_block(&while_stmt.body);
                self.indent -= 1;
            }
            Stmt::Assign(assign) => {
                self.write_indent();
                self.format_expr(&assign.target);
                self.write(" = ");
                self.format_expr(&assign.value);
                self.write("\n");
            }
            Stmt::Break(_) => self.write_line("break"),
            Stmt::Continue(_) => self.write_line("continue"),
            Stmt::Pass(_) => self.write_line("pass"),
            _ => self.write_line("# <unformatted statement>"),
        }
    }

    /// Format a pattern
    fn format_pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Ident(ident) => self.write(ident.name.as_str()),
            Pattern::Tuple(pats, _) => {
                self.write("(");
                for (i, p) in pats.iter().enumerate() {
                    if i > 0 { self.write(", "); }
                    self.format_pattern(p);
                }
                self.write(")");
            }
            _ => self.write("_"),
        }
    }

    /// Format an expression
    fn format_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Int(n, _) => self.write(&n.to_string()),
            Expr::Float(n, _) => self.write(&format!("{}", n)),
            Expr::Bool(b, _) => self.write(if *b { "True" } else { "False" }),
            Expr::String(s, _) => {
                self.write("\"");
                self.write(s.as_str());
                self.write("\"");
            }
            Expr::None(_) => self.write("None"),
            Expr::Ident(ident) => self.write(ident.name.as_str()),
            Expr::Binary(bin) => {
                self.format_expr(&bin.left);
                let op_str = match bin.op {
                    BinaryOp::Add => "+",
                    BinaryOp::Sub => "-",
                    BinaryOp::Mul => "*",
                    BinaryOp::Div => "/",
                    BinaryOp::FloorDiv => "//",
                    BinaryOp::Mod => "%",
                    BinaryOp::Pow => "**",
                    BinaryOp::BitAnd => "&",
                    BinaryOp::BitOr => "|",
                    BinaryOp::BitXor => "^",
                    BinaryOp::Shl => "<<",
                    BinaryOp::Shr => ">>",
                    BinaryOp::And => "and",
                    BinaryOp::Or => "or",
                };
                self.write(&format!(" {} ", op_str));
                self.format_expr(&bin.right);
            }
            Expr::Unary(un) => {
                let op_str = match un.op {
                    UnaryOp::Neg => "-",
                    UnaryOp::Pos => "+",
                    UnaryOp::Not => "not ",
                    UnaryOp::BitNot => "~",
                };
                self.write(op_str);
                self.format_expr(&un.operand);
            }
            Expr::Call(call) => {
                self.format_expr(&call.func);
                self.write("(");
                for (i, arg) in call.args.iter().enumerate() {
                    if i > 0 { self.write(", "); }
                    self.format_expr(&arg.value);
                }
                self.write(")");
            }
            Expr::Attribute(attr) => {
                self.format_expr(&attr.value);
                self.write(".");
                self.write(attr.attr.name.as_str());
            }
            Expr::Subscript(sub) => {
                self.format_expr(&sub.value);
                self.write("[");
                self.format_expr(&sub.index);
                self.write("]");
            }
            Expr::List(list) => {
                self.write("[");
                for (i, elem) in list.elements.iter().enumerate() {
                    if i > 0 { self.write(", "); }
                    self.format_expr(elem);
                }
                self.write("]");
            }
            Expr::Dict(dict) => {
                self.write("{");
                for (i, (k, v)) in dict.pairs.iter().enumerate() {
                    if i > 0 { self.write(", "); }
                    self.format_expr(k);
                    self.write(": ");
                    self.format_expr(v);
                }
                self.write("}");
            }
            Expr::Await(await_expr) => {
                self.write("await ");
                self.format_expr(&await_expr.value);
            }
            _ => self.write("<expr>"),
        }
    }

    /// Format a class definition
    fn format_class(&mut self, class: &ClassDef) {
        self.write_indent();
        self.write("class ");
        self.write(class.name.name.as_str());
        if !class.bases.is_empty() {
            self.write("(");
            for (i, base) in class.bases.iter().enumerate() {
                if i > 0 { self.write(", "); }
                self.format_expr(base);
            }
            self.write(")");
        }
        self.write(":\n");
        self.indent += 1;
        for member in &class.body {
            match member {
                ClassMember::Field(field) => {
                    self.write_indent();
                    self.write(field.name.name.as_str());
                    self.write(": ");
                    self.format_type_expr(&field.ty);
                    self.write("\n");
                }
                ClassMember::Method(method) => {
                    self.format_function(method);
                }
                ClassMember::Class(nested) => {
                    self.format_class(nested);
                }
            }
        }
        self.indent -= 1;
    }

    /// Format an agent definition
    fn format_agent(&mut self, agent: &AgentDef) {
        self.write_indent();
        self.write("agent ");
        self.write(agent.name.name.as_str());
        self.write(":\n");
        self.indent += 1;
        if agent.members.is_empty() {
            self.write_line("pass");
        }
        for member in &agent.members {
            match member {
                AgentMember::Task(task) => {
                    self.write_indent();
                    self.write("task ");
                    self.write(task.name.name.as_str());
                    self.write("(");
                    self.format_params(&task.params);
                    self.write("):\n");
                    self.indent += 1;
                    self.format_block(&task.body);
                    self.indent -= 1;
                }
                AgentMember::Method(func) => self.format_function(func),
                _ => self.write_line("# <config>"),
            }
        }
        self.indent -= 1;
    }

    /// Format a struct definition
    fn format_struct(&mut self, s: &StructDef) {
        self.write_indent();
        self.write("struct ");
        self.write(s.name.name.as_str());
        self.write(":\n");
        self.indent += 1;
        for field in &s.fields {
            self.write_indent();
            self.write(field.name.name.as_str());
            self.write(": ");
            self.format_type_expr(&field.ty);
            self.write("\n");
        }
        self.indent -= 1;
    }

    /// Format an enum definition
    fn format_enum(&mut self, e: &EnumDef) {
        self.write_indent();
        self.write("enum ");
        self.write(e.name.name.as_str());
        self.write(":\n");
        self.indent += 1;
        for variant in &e.variants {
            self.write_line(variant.name.name.as_str());
        }
        self.indent -= 1;
    }

    /// Format an import statement
    fn format_import(&mut self, import: &ImportStmt) {
        self.write_indent();
        match &import.kind {
            ImportKind::Module { path, alias } => {
                self.write("import ");
                let path_str: Vec<&str> = path.parts.iter().map(|p| p.name.as_str()).collect();
                self.write(&path_str.join("."));
                if let Some(alias) = alias {
                    self.write(" as ");
                    self.write(alias.name.as_str());
                }
            }
            ImportKind::From { path, items } => {
                self.write("from ");
                let path_str: Vec<&str> = path.parts.iter().map(|p| p.name.as_str()).collect();
                self.write(&path_str.join("."));
                self.write(" import ");
                for (i, item) in items.iter().enumerate() {
                    if i > 0 { self.write(", "); }
                    self.write(item.name.name.as_str());
                    if let Some(ref alias) = item.alias {
                        self.write(" as ");
                        self.write(alias.name.as_str());
                    }
                }
            }
        }
        self.write("\n");
    }

    /// Format a const definition
    fn format_const(&mut self, c: &ConstDef) {
        self.write_indent();
        self.write("const ");
        self.write(c.name.name.as_str());
        if let Some(ref ty) = c.ty {
            self.write(": ");
            self.format_type_expr(ty);
        }
        self.write(" = ");
        self.format_expr(&c.value);
        self.write("\n");
    }

    /// Format an extern block
    fn format_extern(&mut self, ext: &ExternBlock) {
        self.write_indent();
        self.write("extern \"");
        self.write(ext.lib_name.as_str());
        self.write("\":\n");
        self.indent += 1;
        for func in &ext.functions {
            self.write_indent();
            self.write("fn ");
            self.write(func.name.name.as_str());
            self.write("(");
            self.format_params(&func.params);
            self.write(")");
            if let Some(ref ret) = func.return_type {
                self.write(" -> ");
                self.format_type_expr(ret);
            }
            self.write("\n");
        }
        self.indent -= 1;
    }
}
