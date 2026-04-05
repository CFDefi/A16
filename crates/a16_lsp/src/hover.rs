//! Hover information
//!
//! Provides hover tooltips for keywords, built-in functions, and types.

/// Hover information
#[derive(Debug, Clone)]
pub struct HoverInfo {
    pub label: String,
    pub detail: String,
    pub documentation: Option<String>,
}

/// Compute hover info for a word
pub fn compute_hover(word: &str) -> Option<HoverInfo> {
    // Check keywords
    if let Some(info) = keyword_hover(word) {
        return Some(info);
    }

    // Check built-in functions
    if let Some(info) = builtin_hover(word) {
        return Some(info);
    }

    // Check types
    if let Some(info) = type_hover(word) {
        return Some(info);
    }

    None
}

fn keyword_hover(word: &str) -> Option<HoverInfo> {
    let (label, detail, doc) = match word {
        "fn" => ("fn", "Function definition", "Defines a function.\n\n```a16\nfn name(params) -> ReturnType:\n    body\n```"),
        "let" => ("let", "Variable binding", "Binds a value to a name.\n\n```a16\nlet x = 42\nlet name: Type = value\n```"),
        "const" => ("const", "Constant definition", "Defines an immutable constant.\n\n```a16\nconst PI: Float = 3.14159\n```"),
        "if" => ("if", "Conditional statement", "Executes code conditionally.\n\n```a16\nif condition:\n    then_body\nelif other:\n    elif_body\nelse:\n    else_body\n```"),
        "for" => ("for", "For loop", "Iterates over a sequence.\n\n```a16\nfor item in iterable:\n    body\n```"),
        "while" => ("while", "While loop", "Loops while condition is true.\n\n```a16\nwhile condition:\n    body\n```"),
        "agent" => ("agent", "Agent definition", "Defines an AI agent with model, tools, and tasks.\n\n```a16\nagent MyAgent:\n    model: \"gpt-4\"\n    task main():\n        ...\n```"),
        "struct" => ("struct", "Struct definition", "Defines a data structure.\n\n```a16\nstruct Point:\n    x: Float\n    y: Float\n```"),
        "async" => ("async", "Async function modifier", "Makes a function asynchronous.\n\n```a16\nasync fn fetch(url: Str) -> Str:\n    ...\n```"),
        "await" => ("await", "Await expression", "Waits for an async result.\n\n```a16\nlet result = await fetch(url)\n```"),
        "extern" => ("extern", "Extern block", "Declares foreign functions.\n\n```a16\nextern \"lib_name\":\n    fn func(param: Type) -> RetType\n```"),
        "match" => ("match", "Pattern matching", "Matches a value against patterns.\n\n```a16\nmatch value:\n    case pattern:\n        body\n```"),
        "return" => ("return", "Return statement", "Returns a value from a function."),
        "import" => ("import", "Import statement", "Imports a module.\n\n```a16\nimport module.path\nfrom module import name\n```"),
        _ => return None,
    };

    Some(HoverInfo {
        label: label.to_string(),
        detail: detail.to_string(),
        documentation: Some(doc.to_string()),
    })
}

fn builtin_hover(word: &str) -> Option<HoverInfo> {
    let (sig, doc) = match word {
        "print" => ("fn print(value: Any)", "Prints a value to stdout without newline."),
        "println" => ("fn println(value: Any)", "Prints a value to stdout with newline."),
        "len" => ("fn len(value: Any) -> Int", "Returns the length of a collection."),
        "type" => ("fn type(value: Any) -> Str", "Returns the type name of a value."),
        "str" => ("fn str(value: Any) -> Str", "Converts a value to string."),
        "int" => ("fn int(value: Any) -> Int", "Converts a value to integer."),
        "float" => ("fn float(value: Any) -> Float", "Converts a value to float."),
        "range" => ("fn range(stop: Int) -> List[Int]", "Creates a range from 0 to stop."),
        "abs" => ("fn abs(value: Int | Float) -> Int | Float", "Returns the absolute value."),
        "sorted" => ("fn sorted(list: List) -> List", "Returns a sorted copy of the list."),
        "reversed" => ("fn reversed(list: List) -> List", "Returns a reversed copy of the list."),
        "map" => ("fn map(func: Callable, iter: List) -> List", "Applies a function to each element."),
        "filter" => ("fn filter(func: Callable, iter: List) -> List", "Filters elements by predicate."),
        "ffi_load" => ("fn ffi_load(lib: Str, path: Str | None) -> Bool", "Loads a native library for FFI."),
        "ffi_list" => ("fn ffi_list(lib: Str) -> List[Str]", "Lists functions in a loaded FFI library."),
        _ => return None,
    };

    Some(HoverInfo {
        label: word.to_string(),
        detail: sig.to_string(),
        documentation: Some(doc.to_string()),
    })
}

fn type_hover(word: &str) -> Option<HoverInfo> {
    let (detail, doc) = match word {
        "Int" => ("Built-in integer type", "64-bit signed integer."),
        "Float" => ("Built-in floating-point type", "64-bit IEEE 754 double."),
        "Str" => ("Built-in string type", "UTF-8 string (immutable)."),
        "Bool" => ("Built-in boolean type", "True or False."),
        "List" => ("Built-in list type", "Ordered, mutable collection.\n\n```a16\nlet items: List[Int] = [1, 2, 3]\n```"),
        "Dict" => ("Built-in dictionary type", "Key-value mapping.\n\n```a16\nlet map: Dict[Str, Int] = {\"a\": 1}\n```"),
        "Tensor" => ("AI tensor type", "Multi-dimensional numeric array for ML."),
        "Embedding" => ("AI embedding type", "Vector representation for semantic search."),
        "Any" => ("Dynamic type", "Accepts any value (gradual typing)."),
        "None" => ("None type", "The absence of a value."),
        _ => return None,
    };

    Some(HoverInfo {
        label: word.to_string(),
        detail: detail.to_string(),
        documentation: Some(doc.to_string()),
    })
}
