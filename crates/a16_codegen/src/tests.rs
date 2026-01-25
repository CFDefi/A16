//! Tests for bytecode compiler

use a16_parser::parse;
use a16_hir::lower_module;
use crate::compile;

#[test]
fn test_compile_simple_function() {
    let source = r#"
fn add(a, b):
    return a + b
"#;
    let module = parse(source).expect("parse failed");
    let hir = lower_module(&module);
    let bytecode = compile(&hir);
    
    assert_eq!(bytecode.functions.len(), 1);
    assert_eq!(bytecode.functions[0].name.as_str(), "add");
    assert_eq!(bytecode.functions[0].arity, 2);
    assert!(!bytecode.functions[0].code.is_empty());
}

#[test]
fn test_compile_if_else() {
    let source = r#"
fn max(a, b):
    if a > b:
        return a
    else:
        return b
"#;
    let module = parse(source).expect("parse failed");
    let hir = lower_module(&module);
    let bytecode = compile(&hir);
    
    assert_eq!(bytecode.functions.len(), 1);
}

#[test]
fn test_compile_while_loop() {
    let source = r#"
fn count_to(n):
    let i = 0
    while i < n:
        i += 1
    return i
"#;
    let module = parse(source).expect("parse failed");
    let hir = lower_module(&module);
    let bytecode = compile(&hir);
    
    assert_eq!(bytecode.functions.len(), 1);
}

#[test]
fn test_compile_literals() {
    let source = r#"
fn literals():
    let a = 42
    let b = 3.14
    let c = "hello"
    let d = true
    let e = false
    return a
"#;
    let module = parse(source).expect("parse failed");
    let hir = lower_module(&module);
    let bytecode = compile(&hir);
    
    assert_eq!(bytecode.functions.len(), 1);
    // Check constants were added
    assert!(bytecode.constants.len() >= 2); // 42, 3.14, "hello"
}

#[test]
fn test_compile_list_and_dict() {
    let source = r#"
fn collections():
    let nums = [1, 2, 3]
    let data = {"a": 1, "b": 2}
    return nums
"#;
    let module = parse(source).expect("parse failed");
    let hir = lower_module(&module);
    let bytecode = compile(&hir);
    
    assert_eq!(bytecode.functions.len(), 1);
}
