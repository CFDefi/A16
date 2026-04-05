//! Formatter tests

use crate::{format_source, FormatConfig, format_source_with_config};

#[test]
fn test_format_simple_function() {
    let source = "fn main():\n    return 0\n";
    let result = format_source(source).unwrap();
    assert!(result.contains("fn main()"));
    assert!(result.contains("return 0"));
}

#[test]
fn test_format_function_with_params() {
    let source = "fn add(a: Int, b: Int) -> Int:\n    return a\n";
    let result = format_source(source).unwrap();
    assert!(result.contains("fn add(a: Int, b: Int) -> Int:"));
}

#[test]
fn test_format_let_statement() {
    let source = "fn main():\n    let x = 42\n";
    let result = format_source(source).unwrap();
    assert!(result.contains("let x = 42"));
}

#[test]
fn test_format_if_statement() {
    let source = "fn main():\n    if True:\n        return 1\n";
    let result = format_source(source).unwrap();
    assert!(result.contains("if True:"));
    assert!(result.contains("return 1"));
}

#[test]
fn test_format_struct() {
    let source = "struct Point:\n    x: Float\n    y: Float\n";
    let result = format_source(source).unwrap();
    assert!(result.contains("struct Point:"));
    assert!(result.contains("x: Float"));
}

#[test]
fn test_format_trailing_newline() {
    let config = FormatConfig {
        trailing_newline: true,
        ..Default::default()
    };
    let source = "fn main():\n    pass\n";
    let result = format_source_with_config(source, &config).unwrap();
    assert!(result.ends_with('\n'));
}

#[test]
fn test_format_config_defaults() {
    let config = FormatConfig::default();
    assert_eq!(config.indent_size, 4);
    assert_eq!(config.max_line_width, 100);
    assert!(config.trailing_newline);
}

#[test]
fn test_format_import() {
    let source = "import a16.ai.agent\n\nfn main():\n    pass\n";
    let result = format_source(source).unwrap();
    assert!(result.contains("import a16.ai.agent"));
}
