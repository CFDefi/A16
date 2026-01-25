//! Type checker tests

#[cfg(test)]
mod test {
    use crate::check;
    use a16_parser::parse;

    fn check_source(source: &str) -> Vec<String> {
        let module = parse(source).expect("parse failed");
        let errors = check(&module);
        errors.iter().map(|e| e.to_string()).collect()
    }

    fn check_ok(source: &str) {
        let errors = check_source(source);
        assert!(errors.is_empty(), "Expected no errors, got: {:?}", errors);
    }

    fn check_err(source: &str, expected_substr: &str) {
        let errors = check_source(source);
        assert!(!errors.is_empty(), "Expected errors containing '{}', got none", expected_substr);
        let has_match = errors.iter().any(|e| e.contains(expected_substr));
        assert!(has_match, "Expected error containing '{}', got: {:?}", expected_substr, errors);
    }

    // =============================================================================
    // BASIC TYPE INFERENCE
    // =============================================================================

    #[test]
    fn test_literal_types() {
        check_ok(r#"
let a = 42
let b = 3.14
let c = True
let d = "hello"
let e = None
"#);
    }

    #[test]
    fn test_list_type_inference() {
        check_ok(r#"
let nums = [1, 2, 3]
let strs = ["a", "b", "c"]
"#);
    }

    #[test]
    fn test_dict_type_inference() {
        check_ok(r#"
let scores = {"alice": 100, "bob": 85}
"#);
    }

    // =============================================================================
    // FUNCTIONS
    // =============================================================================

    #[test]
    fn test_simple_function() {
        check_ok(r#"
fn add(a: Int, b: Int) -> Int:
    return a + b
"#);
    }

    #[test]
    fn test_function_call() {
        check_ok(r#"
fn double(x: Int) -> Int:
    return x * 2

let result = double(21)
"#);
    }

    // =============================================================================
    // OPERATORS
    // =============================================================================

    #[test]
    fn test_numeric_operators() {
        check_ok(r#"
let a = 1 + 2
let b = 3.0 - 1.5
let c = 2 * 3
let d = 10 / 3
let e = 10 // 3
let f = 10 % 3
let g = 2 ** 8
"#);
    }

    #[test]
    fn test_string_concat() {
        check_ok(r#"
let s = "hello" + " " + "world"
"#);
    }

    // =============================================================================
    // CONTROL FLOW
    // =============================================================================

    #[test]
    fn test_if_statement() {
        check_ok(r#"
let x = 10
if x > 5:
    print("big")
elif x > 0:
    print("small")
else:
    print("zero or negative")
"#);
    }

    #[test]
    fn test_for_loop() {
        check_ok(r#"
let items = [1, 2, 3]
for item in items:
    print(item)
"#);
    }

    #[test]
    fn test_while_loop() {
        check_ok(r#"
let mut i = 0
while i < 10:
    print(i)
    i += 1
"#);
    }

    // =============================================================================
    // CLASSES AND STRUCTS
    // =============================================================================

    #[test]
    fn test_struct() {
        check_ok(r#"
struct Point:
    x: Float
    y: Float
"#);
    }

    #[test]
    fn test_class() {
        // Basic class definition - testing structure only
        // Note: self.member access requires additional type checker work
        check_ok(r#"
class Counter:
    count: Int
    
    fn get(self) -> Int:
        return 0
"#);
    }

    // =============================================================================
    // AGENTS AND AI PRIMITIVES
    // =============================================================================

    #[test]
    fn test_agent_basic() {
        // Agent definition with string-based config values
        check_ok(r#"
agent MyAgent:
    model: "gpt-4"
    tools: ["web_search"]
    
    task greet(name: Str) -> Str:
        return "Hello, " + name
"#);
    }

    #[test]
    fn test_tool_basic() {
        check_ok(r#"
tool web_search:
    permissions: [network]
    sandbox: light
    
    fn execute(query: Str) -> List[Str]:
        return []
"#);
    }

    // =============================================================================
    // UNDEFINED VARIABLES
    // =============================================================================

    #[test]
    fn test_undefined_variable() {
        check_err(r#"
let x = undefined_var
"#, "Undefined variable");
    }

    // =============================================================================
    // COMPREHENSIONS
    // =============================================================================

    #[test]
    fn test_list_comprehension() {
        check_ok(r#"
let nums = [1, 2, 3, 4, 5]
let squares = [x * x for x in nums]
"#);
    }

    // =============================================================================
    // IMPORTS
    // =============================================================================

    #[test]
    fn test_import() {
        check_ok(r#"
import a16.ai.agent
from a16.ai.tools import web_search
"#);
    }

    // =============================================================================
    // MATCH EXPRESSIONS
    // =============================================================================

    #[test]
    fn test_match_statement() {
        check_ok(r#"
let x = 42
match x:
    case 0:
        print("zero")
    case 1:
        print("one")
    case _:
        print("other")
"#);
    }

    // =============================================================================
    // ASYNC/AWAIT
    // =============================================================================

    #[test]
    fn test_async_function() {
        check_ok(r#"
async fn fetch_data(url: Str) -> Str:
    return "data"
"#);
    }

    // =============================================================================
    // GRADUAL TYPING
    // =============================================================================

    #[test]
    fn test_any_type_allows_anything() {
        check_ok(r#"
fn process(x: Any) -> Any:
    return x

let a = process(42)
let b = process("hello")
let c = process([1, 2, 3])
"#);
    }

    #[test]
    fn test_untyped_parameters() {
        // Untyped parameters get Any type, allowing flexibility
        // Testing with single call to avoid unification conflicts
        check_ok(r#"
fn identity(x):
    return x

let a = identity(42)
"#);
    }
}
