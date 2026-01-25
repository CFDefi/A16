//! Parser tests

use crate::parse;

#[test]
fn test_parse_simple_function() {
    let source = r#"
fn hello(name: Str) -> Str:
    return "Hello, " + name
"#;
    
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    
    let module = result.unwrap();
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_agent() {
    let source = r#"
agent MyAgent:
    model: gpt4
    tools: [web_search, file_read]
    
    task greet(name: Str) -> Str:
        return "Hello, " + name
"#;
    
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_struct() {
    let source = r#"
struct Point:
    x: Float
    y: Float
    z: Float = 0.0
"#;
    
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_expressions() {
    let source = r#"
let a = 1 + 2 * 3
let b = (1 + 2) * 3
let c = x if condition else y
let d = [1, 2, 3]
let e = {"a": 1, "b": 2}
let f = await fetch(url)
"#;
    
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_control_flow() {
    let source = r#"
if x > 0:
    print("positive")
elif x < 0:
    print("negative")
else:
    print("zero")

for item in items:
    print(item)

while condition:
    do_something()
"#;
    
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_imports() {
    let source = r#"
import a16.ai.agent
from a16.ai.model import gpt4, claude
from a16.ai.tools import web_search as search
"#;
    
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_tool() {
    let source = r#"
tool web_search:
    permissions: [network]
    sandbox: light
    rate_limit: 60
    
    fn execute(query: Str) -> List[Result]:
        return search(query)
"#;
    
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}
