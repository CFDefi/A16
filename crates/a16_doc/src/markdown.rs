//! Markdown rendering for A16 documentation

use crate::*;

/// Render a documented module to Markdown
pub fn render_markdown(doc: &DocModule) -> String {
    let mut out = String::new();

    out.push_str(&format!("# Module: {}\n\n", doc.name));

    for item in &doc.items {
        match item {
            DocItem::Function(func) => render_function(&mut out, func),
            DocItem::Class(class) => render_class(&mut out, class),
            DocItem::Struct(s) => render_struct(&mut out, s),
            DocItem::Agent(agent) => render_agent(&mut out, agent),
            DocItem::Const(c) => render_const(&mut out, c),
            DocItem::Extern(ext) => render_extern(&mut out, ext),
        }
    }

    out
}

fn render_function(out: &mut String, func: &DocFunction) {
    let async_prefix = if func.is_async { "async " } else { "" };
    out.push_str(&format!("## {}fn {}\n\n", async_prefix, func.name));
    out.push_str("```a16\n");
    out.push_str(&format!("{}fn {}(", async_prefix, func.name));
    for (i, param) in func.params.iter().enumerate() {
        if i > 0 { out.push_str(", "); }
        out.push_str(&param.name);
        if let Some(ref ty) = param.type_name {
            out.push_str(&format!(": {}", ty));
        }
    }
    out.push(')');
    if let Some(ref ret) = func.return_type {
        out.push_str(&format!(" -> {}", ret));
    }
    out.push_str("\n```\n\n");

    if !func.params.is_empty() {
        out.push_str("**Parameters:**\n\n");
        for param in &func.params {
            let ty_str = param.type_name.as_deref().unwrap_or("Any");
            out.push_str(&format!("- `{}`: `{}`\n", param.name, ty_str));
        }
        out.push('\n');
    }
}

fn render_class(out: &mut String, class: &DocClass) {
    out.push_str(&format!("## class {}\n\n", class.name));

    if !class.fields.is_empty() {
        out.push_str("**Fields:**\n\n");
        out.push_str("| Field | Type |\n|-------|------|\n");
        for field in &class.fields {
            out.push_str(&format!("| `{}` | `{}` |\n", field.name, field.type_name));
        }
        out.push('\n');
    }

    if !class.methods.is_empty() {
        out.push_str("**Methods:**\n\n");
        for method in &class.methods {
            render_function(out, method);
        }
    }
}

fn render_struct(out: &mut String, s: &DocStruct) {
    out.push_str(&format!("## struct {}\n\n", s.name));

    if !s.fields.is_empty() {
        out.push_str("| Field | Type |\n|-------|------|\n");
        for field in &s.fields {
            out.push_str(&format!("| `{}` | `{}` |\n", field.name, field.type_name));
        }
        out.push('\n');
    }
}

fn render_agent(out: &mut String, agent: &DocAgent) {
    out.push_str(&format!("## agent {}\n\n", agent.name));

    if !agent.tasks.is_empty() {
        out.push_str("**Tasks:**\n\n");
        for task in &agent.tasks {
            render_function(out, task);
        }
    }
}

fn render_const(out: &mut String, c: &DocConst) {
    let ty_str = c.type_name.as_deref().unwrap_or("inferred");
    out.push_str(&format!("## const {}\n\nType: `{}`\n\n", c.name, ty_str));
}

fn render_extern(out: &mut String, ext: &DocExtern) {
    out.push_str(&format!("## extern \"{}\"\n\n", ext.lib_name));
    for func in &ext.functions {
        render_function(out, func);
    }
}
