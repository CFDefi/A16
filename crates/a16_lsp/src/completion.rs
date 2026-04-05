//! Completion items
//!
//! Provides keyword completions, built-in function completions, and snippet templates.

/// Completion item kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    Keyword,
    Function,
    Snippet,
}

/// A completion item
#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionKind,
    pub detail: Option<String>,
    pub insert_text: Option<String>,
}

/// Compute completions for the current context
pub fn compute_completions(_source: &str) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    // Keywords
    for kw in KEYWORDS {
        items.push(CompletionItem {
            label: kw.to_string(),
            kind: CompletionKind::Keyword,
            detail: Some("keyword".to_string()),
            insert_text: None,
        });
    }

    // Built-in functions
    for (name, sig) in BUILTIN_FUNCTIONS {
        items.push(CompletionItem {
            label: name.to_string(),
            kind: CompletionKind::Function,
            detail: Some(sig.to_string()),
            insert_text: Some(format!("{}($0)", name)),
        });
    }

    // Snippets
    items.push(CompletionItem {
        label: "fn".to_string(),
        kind: CompletionKind::Snippet,
        detail: Some("Function definition".to_string()),
        insert_text: Some("fn ${1:name}(${2:params}):\n    ${0:pass}".to_string()),
    });

    items.push(CompletionItem {
        label: "agent".to_string(),
        kind: CompletionKind::Snippet,
        detail: Some("Agent definition".to_string()),
        insert_text: Some("agent ${1:Name}:\n    model: ${2:\"gpt-4\"}\n    task ${3:main}():\n        ${0:pass}".to_string()),
    });

    items.push(CompletionItem {
        label: "struct".to_string(),
        kind: CompletionKind::Snippet,
        detail: Some("Struct definition".to_string()),
        insert_text: Some("struct ${1:Name}:\n    ${2:field}: ${3:Type}\n".to_string()),
    });

    items.push(CompletionItem {
        label: "for".to_string(),
        kind: CompletionKind::Snippet,
        detail: Some("For loop".to_string()),
        insert_text: Some("for ${1:item} in ${2:iterable}:\n    ${0:pass}".to_string()),
    });

    items.push(CompletionItem {
        label: "if".to_string(),
        kind: CompletionKind::Snippet,
        detail: Some("If statement".to_string()),
        insert_text: Some("if ${1:condition}:\n    ${0:pass}".to_string()),
    });

    items
}

/// All A16 keywords
const KEYWORDS: &[&str] = &[
    "fn", "let", "const", "return", "if", "elif", "else",
    "for", "while", "break", "continue", "pass",
    "class", "struct", "enum", "import", "from", "as",
    "agent", "task", "tool", "memory", "prompt",
    "async", "await", "spawn", "True", "False", "None",
    "match", "case", "extern",
];

/// Built-in function signatures
const BUILTIN_FUNCTIONS: &[(&str, &str)] = &[
    ("print", "fn print(value: Any)"),
    ("println", "fn println(value: Any)"),
    ("len", "fn len(value: Any) -> Int"),
    ("type", "fn type(value: Any) -> Str"),
    ("str", "fn str(value: Any) -> Str"),
    ("int", "fn int(value: Any) -> Int"),
    ("float", "fn float(value: Any) -> Float"),
    ("range", "fn range(stop: Int) -> List[Int]"),
    ("input", "fn input(prompt: Str) -> Str"),
    ("abs", "fn abs(value: Int | Float) -> Int | Float"),
    ("min", "fn min(a: Any, b: Any) -> Any"),
    ("max", "fn max(a: Any, b: Any) -> Any"),
    ("sorted", "fn sorted(list: List) -> List"),
    ("reversed", "fn reversed(list: List) -> List"),
    ("enumerate", "fn enumerate(iter: List) -> List[Tuple[Int, Any]]"),
    ("zip", "fn zip(a: List, b: List) -> List"),
    ("map", "fn map(func: Callable, iter: List) -> List"),
    ("filter", "fn filter(func: Callable, iter: List) -> List"),
    ("ffi_load", "fn ffi_load(lib: Str, path: Str | None) -> Bool"),
    ("ffi_list", "fn ffi_list(lib: Str) -> List[Str]"),
    ("ffi_unload", "fn ffi_unload(lib: Str) -> Bool"),
];
