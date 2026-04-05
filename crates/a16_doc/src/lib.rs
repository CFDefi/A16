//! A16 Documentation Generator
//!
//! Extracts documentation from A16 source code and renders it to Markdown.

mod markdown;

pub use markdown::render_markdown;

/// A documented module
#[derive(Debug, Clone)]
pub struct DocModule {
    pub name: String,
    pub items: Vec<DocItem>,
}

/// A documented item
#[derive(Debug, Clone)]
pub enum DocItem {
    Function(DocFunction),
    Class(DocClass),
    Struct(DocStruct),
    Agent(DocAgent),
    Const(DocConst),
    Extern(DocExtern),
}

/// A documented function
#[derive(Debug, Clone)]
pub struct DocFunction {
    pub name: String,
    pub params: Vec<DocParam>,
    pub return_type: Option<String>,
    pub is_async: bool,
}

/// A documented parameter
#[derive(Debug, Clone)]
pub struct DocParam {
    pub name: String,
    pub type_name: Option<String>,
    pub has_default: bool,
}

/// A documented class
#[derive(Debug, Clone)]
pub struct DocClass {
    pub name: String,
    pub fields: Vec<DocField>,
    pub methods: Vec<DocFunction>,
}

/// A documented field
#[derive(Debug, Clone)]
pub struct DocField {
    pub name: String,
    pub type_name: String,
}

/// A documented struct
#[derive(Debug, Clone)]
pub struct DocStruct {
    pub name: String,
    pub fields: Vec<DocField>,
}

/// A documented agent
#[derive(Debug, Clone)]
pub struct DocAgent {
    pub name: String,
    pub tasks: Vec<DocFunction>,
}

/// A documented constant
#[derive(Debug, Clone)]
pub struct DocConst {
    pub name: String,
    pub type_name: Option<String>,
}

/// A documented extern block
#[derive(Debug, Clone)]
pub struct DocExtern {
    pub lib_name: String,
    pub functions: Vec<DocFunction>,
}

/// Generate documentation from A16 source code
pub fn generate_docs(source: &str, module_name: &str) -> Result<DocModule, String> {
    let module = a16_parser::parse(source)
        .map_err(|e| format!("Parse error: {:?}", e))?;

    let mut items = Vec::new();

    for item in &module.items {
        match item {
            a16_ast::Item::Function(func) => {
                items.push(DocItem::Function(extract_function(func)));
            }
            a16_ast::Item::Class(class) => {
                items.push(DocItem::Class(extract_class(class)));
            }
            a16_ast::Item::Struct(s) => {
                items.push(DocItem::Struct(extract_struct(s)));
            }
            a16_ast::Item::Agent(agent) => {
                items.push(DocItem::Agent(extract_agent(agent)));
            }
            a16_ast::Item::Const(c) => {
                items.push(DocItem::Const(DocConst {
                    name: c.name.name.to_string(),
                    type_name: c.ty.as_ref().map(|t| format_type_expr(t)),
                }));
            }
            a16_ast::Item::Extern(ext) => {
                items.push(DocItem::Extern(DocExtern {
                    lib_name: ext.lib_name.to_string(),
                    functions: ext.functions.iter().map(|f| DocFunction {
                        name: f.name.name.to_string(),
                        params: f.params.iter().map(|p| DocParam {
                            name: p.name.name.to_string(),
                            type_name: p.ty.as_ref().map(|t| format_type_expr(t)),
                            has_default: p.default.is_some(),
                        }).collect(),
                        return_type: f.return_type.as_ref().map(|t| format_type_expr(t)),
                        is_async: false,
                    }).collect(),
                }));
            }
            _ => {}
        }
    }

    Ok(DocModule {
        name: module_name.to_string(),
        items,
    })
}

fn extract_function(func: &a16_ast::FunctionDef) -> DocFunction {
    DocFunction {
        name: func.name.name.to_string(),
        params: func.params.iter().map(|p| DocParam {
            name: p.name.name.to_string(),
            type_name: p.ty.as_ref().map(|t| format_type_expr(t)),
            has_default: p.default.is_some(),
        }).collect(),
        return_type: func.return_type.as_ref().map(|t| format_type_expr(t)),
        is_async: func.is_async,
    }
}

fn extract_class(class: &a16_ast::ClassDef) -> DocClass {
    let mut fields = Vec::new();
    let mut methods = Vec::new();

    for member in &class.body {
        match member {
            a16_ast::ClassMember::Field(field) => {
                fields.push(DocField {
                    name: field.name.name.to_string(),
                    type_name: format_type_expr(&field.ty),
                });
            }
            a16_ast::ClassMember::Method(method) => {
                methods.push(extract_function(method));
            }
            _ => {}
        }
    }

    DocClass {
        name: class.name.name.to_string(),
        fields,
        methods,
    }
}

fn extract_struct(s: &a16_ast::StructDef) -> DocStruct {
    DocStruct {
        name: s.name.name.to_string(),
        fields: s.fields.iter().map(|f| DocField {
            name: f.name.name.to_string(),
            type_name: format_type_expr(&f.ty),
        }).collect(),
    }
}

fn extract_agent(agent: &a16_ast::AgentDef) -> DocAgent {
    let mut tasks = Vec::new();
    for member in &agent.members {
        if let a16_ast::AgentMember::Task(task) = member {
            tasks.push(DocFunction {
                name: task.name.name.to_string(),
                params: task.params.iter().map(|p| DocParam {
                    name: p.name.name.to_string(),
                    type_name: p.ty.as_ref().map(|t| format_type_expr(t)),
                    has_default: p.default.is_some(),
                }).collect(),
                return_type: task.return_type.as_ref().map(|t| format_type_expr(t)),
                is_async: task.is_async,
            });
        }
    }
    DocAgent {
        name: agent.name.name.to_string(),
        tasks,
    }
}

fn format_type_expr(ty: &a16_ast::TypeExpr) -> String {
    match ty {
        a16_ast::TypeExpr::Name(ident) => ident.name.to_string(),
        a16_ast::TypeExpr::Generic(g) => {
            let args: Vec<String> = g.args.iter().map(|a| format_type_expr(a)).collect();
            format!("{}[{}]", g.name.name, args.join(", "))
        }
        a16_ast::TypeExpr::Union(types, _) => {
            let parts: Vec<String> = types.iter().map(|t| format_type_expr(t)).collect();
            parts.join(" | ")
        }
        a16_ast::TypeExpr::Optional(inner, _) => format!("Optional[{}]", format_type_expr(inner)),
        a16_ast::TypeExpr::Tuple(types, _) => {
            let parts: Vec<String> = types.iter().map(|t| format_type_expr(t)).collect();
            format!("Tuple[{}]", parts.join(", "))
        }
        a16_ast::TypeExpr::Callable(c) => {
            let params: Vec<String> = c.params.iter().map(|p| format_type_expr(p)).collect();
            format!("({}) -> {}", params.join(", "), format_type_expr(&c.return_type))
        }
    }
}

#[cfg(test)]
mod tests;
