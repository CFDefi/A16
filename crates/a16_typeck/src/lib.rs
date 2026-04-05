//! A16 Type Checker
//!
//! This crate provides type checking for the A16 programming language,
//! implementing Hindley-Milner style type inference with extensions for
//! gradual typing and AI-specific primitives.

pub mod types;
pub mod error;
pub mod context;
mod unify;
mod builtins;
mod check_expr;
mod check_stmt;
mod check_item;
pub mod exhaustive;
pub mod narrowing;

#[cfg(test)]
mod tests;

pub use types::Type;
pub use error::TypeError;
pub use context::TypeContext;
pub use narrowing::NarrowingContext;

use a16_ast::Module;
use check_item::register_item_type;

/// Type check a module and return any errors found.
///
/// This is the main entry point for type checking an A16 program.
/// It performs the following steps:
/// 1. Creates a type context with built-in functions and types
/// 2. Registers all top-level item types (first pass)
/// 3. Type checks all items in detail (second pass)
/// 4. Returns any type errors found
///
/// # Example
///
/// ```ignore
/// use a16_parser::parse;
/// use a16_typeck::check;
///
/// let source = r#"
/// fn add(a: Int, b: Int) -> Int:
///     return a + b
/// "#;
///
/// let module = parse(source).expect("parse failed");
/// let errors = check(&module);
///
/// if errors.is_empty() {
///     println!("No type errors!");
/// } else {
///     for err in &errors {
///         eprintln!("{}", err);
///     }
/// }
/// ```
pub fn check(module: &Module) -> Vec<TypeError> {
    let mut ctx = TypeContext::new();
    
    // Register built-in functions and types
    builtins::register_all(&mut ctx);
    
    // First pass: register all top-level definitions
    // This allows forward references between items
    for item in &module.items {
        register_item_type(&mut ctx, item);
    }
    
    // Second pass: full type checking of all items
    for item in &module.items {
        ctx.check_item(item);
    }
    
    // Return all accumulated errors
    ctx.take_errors()
}

/// Type check a module and return a result with context.
///
/// This variant returns the full type context along with errors,
/// which can be useful for IDE integration or further analysis.
pub fn check_with_context(module: &Module) -> (TypeContext, Vec<TypeError>) {
    let mut ctx = TypeContext::new();
    
    builtins::register_all(&mut ctx);
    
    for item in &module.items {
        register_item_type(&mut ctx, item);
    }
    
    for item in &module.items {
        ctx.check_item(item);
    }
    
    let errors = ctx.take_errors();
    (ctx, errors)
}
