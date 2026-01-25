//! Tests for HIR lowering

use a16_parser::parse;
use crate::lower_module;

#[test]
fn test_lower_simple_function() {
    let source = r#"
fn add(a, b):
    return a + b
"#;
    let module = parse(source).expect("parse failed");
    let hir = lower_module(&module);
    
    assert_eq!(hir.functions.len(), 1);
    assert_eq!(hir.functions[0].name.as_str(), "add");
    assert_eq!(hir.functions[0].params.len(), 2);
}

#[test]
fn test_lower_if_elif() {
    let source = r#"
fn classify(x):
    if x > 0:
        return "positive"
    elif x < 0:
        return "negative"
    else:
        return "zero"
"#;
    let module = parse(source).expect("parse failed");
    let hir = lower_module(&module);
    
    assert_eq!(hir.functions.len(), 1);
    // elif is desugared to nested if-else
}

#[test]
fn test_lower_for_loop() {
    let source = r#"
fn sum_list(items):
    let total = 0
    for x in items:
        total += x
    return total
"#;
    let module = parse(source).expect("parse failed");
    let hir = lower_module(&module);
    
    assert_eq!(hir.functions.len(), 1);
}

#[test]
fn test_lower_agent() {
    let source = r#"
agent MyAgent:
    model: "gpt-4"
    tools: ["search"]
    
    task greet(name: Str) -> Str:
        return "Hello, " + name
"#;
    let module = parse(source).expect("parse failed");
    let hir = lower_module(&module);
    
    assert_eq!(hir.agents.len(), 1);
    assert_eq!(hir.agents[0].name.as_str(), "MyAgent");
    assert_eq!(hir.agents[0].tasks.len(), 1);
}

#[test]
fn test_lower_binary_ops() {
    let source = r#"
fn calc():
    let a = 1 + 2 * 3
    let b = a - 4 / 2
    return b
"#;
    let module = parse(source).expect("parse failed");
    let hir = lower_module(&module);
    
    assert_eq!(hir.functions.len(), 1);
}
