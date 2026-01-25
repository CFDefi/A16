//! Tests for VM execution

use a16_parser::parse;
use a16_hir::lower_module;
use a16_codegen::compile;
use crate::VM;
use crate::value::Value;

fn run_code(source: &str, func_name: &str) -> Value {
    let module = parse(source).expect("parse failed");
    let hir = lower_module(&module);
    let bytecode = compile(&hir);
    let mut vm = VM::new(bytecode);
    vm.run(func_name).expect("execution failed")
}

#[test]
fn test_simple_return() {
    let result = run_code(r#"
fn answer():
    return 42
"#, "answer");
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_arithmetic() {
    let result = run_code(r#"
fn calc():
    return 2 + 3 * 4
"#, "calc");
    assert_eq!(result, Value::Int(14));
}

#[test]
fn test_comparison() {
    let result = run_code(r#"
fn gt():
    return 5 > 3
"#, "gt");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_if_else() {
    let result = run_code(r#"
fn check():
    let a = 10
    let b = 5
    if a > b:
        return a
    else:
        return b
"#, "check");
    assert_eq!(result, Value::Int(10));
}

#[test]
fn test_local_variables() {
    let result = run_code(r#"
fn compute():
    let x = 10
    let y = 20
    return x + y
"#, "compute");
    assert_eq!(result, Value::Int(30));
}

#[test]
fn test_string_concat() {
    let result = run_code(r#"
fn greet():
    let name = "World"
    return "Hello, " + name
"#, "greet");
    if let Value::Str(s) = result {
        assert!(s.contains("Hello"));
    } else {
        panic!("Expected string");
    }
}

#[test]
fn test_list_creation() {
    let result = run_code(r#"
fn nums():
    return [1, 2, 3]
"#, "nums");
    if let Value::List(list) = result {
        assert_eq!(list.borrow().len(), 3);
    } else {
        panic!("Expected list");
    }
}
