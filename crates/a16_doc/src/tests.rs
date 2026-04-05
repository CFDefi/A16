//! Documentation generator tests

use crate::{generate_docs, render_markdown};

#[test]
fn test_doc_function() {
    let source = "fn add(a: Int, b: Int) -> Int:\n    return a\n";
    let docs = generate_docs(source, "test").unwrap();
    assert_eq!(docs.items.len(), 1);
    let md = render_markdown(&docs);
    assert!(md.contains("fn add"));
    assert!(md.contains("Int"));
}

#[test]
fn test_doc_struct() {
    let source = "struct Point:\n    x: Float\n    y: Float\n";
    let docs = generate_docs(source, "geometry").unwrap();
    assert_eq!(docs.items.len(), 1);
    let md = render_markdown(&docs);
    assert!(md.contains("struct Point"));
    assert!(md.contains("Float"));
}

#[test]
fn test_doc_async_function() {
    let source = "async fn fetch(url: Str) -> Str:\n    return url\n";
    let docs = generate_docs(source, "net").unwrap();
    let md = render_markdown(&docs);
    assert!(md.contains("async"));
    assert!(md.contains("fn fetch"));
}

#[test]
fn test_doc_module_name() {
    let source = "fn main():\n    pass\n";
    let docs = generate_docs(source, "my_module").unwrap();
    assert_eq!(docs.name, "my_module");
    let md = render_markdown(&docs);
    assert!(md.contains("Module: my_module"));
}

#[test]
fn test_doc_const() {
    let source = "const PI: Float = 3\n";
    let docs = generate_docs(source, "math").unwrap();
    let md = render_markdown(&docs);
    assert!(md.contains("const PI"));
    assert!(md.contains("Float"));
}
