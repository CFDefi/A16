# A16 Reference Implementation Plan

> **Version:** 0.1 | **Status:** Draft | **Date:** 2026-01-19

---

## 1. Implementation Language Selection

### Decision: Rust (Core) + A16 (Standard Library)

| Component | Language | Rationale |
|-----------|----------|-----------|
| Lexer | Rust | Performance, zero-cost abstractions |
| Parser | Rust | Memory safety, pattern matching |
| Type Checker | Rust | Algebraic data types, exhaustive matching |
| Bytecode Compiler | Rust | Control over memory layout |
| VM | Rust | No GC pause, predictable performance |
| JIT | Rust + LLVM | Existing infrastructure |
| Token Engine | Rust | High throughput, tight loops |
| Memory Manager | Rust | Manual memory control |
| Scheduler | Rust | Lock-free data structures |
| Standard Library | A16 | Dogfooding, accessibility |
| CLI | Rust | Single binary distribution |
| LSP Server | Rust | Performance for IDE |

### Alternative Considered: C++
Rejected due to: memory safety concerns, slower development velocity, less expressive error handling.

### Alternative Considered: Go
Rejected due to: GC pauses unsuitable for real-time token processing, less control over memory layout.

---

## 2. Repository Structure

```
a16/
├── Cargo.toml               # Workspace root
├── crates/
│   ├── a16_lexer/           # Lexical analysis
│   ├── a16_parser/          # Parsing
│   ├── a16_ast/             # AST definitions
│   ├── a16_hir/             # High-level IR
│   ├── a16_typeck/          # Type checking
│   ├── a16_bytecode/        # Bytecode format
│   ├── a16_compiler/        # Compilation pipeline
│   ├── a16_vm/              # Virtual machine
│   ├── a16_jit/             # JIT compiler
│   ├── a16_runtime/         # Core runtime
│   │   ├── src/
│   │   │   ├── token_engine.rs
│   │   │   ├── memory_manager.rs
│   │   │   ├── scheduler.rs
│   │   │   ├── sandbox.rs
│   │   │   └── cache.rs
│   ├── a16_ai/              # AI runtime components
│   │   ├── src/
│   │   │   ├── agent.rs
│   │   │   ├── model.rs
│   │   │   ├── memory.rs
│   │   │   └── tools.rs
│   ├── a16_stdlib/          # Standard library (A16 source)
│   ├── a16_cli/             # CLI application
│   ├── a16_lsp/             # Language server
│   └── a16_pkg/             # Package manager
├── stdlib/                  # A16 standard library source
│   ├── a16/
│   │   ├── ai/
│   │   ├── sys/
│   │   └── core/
├── tests/                   # Integration tests
│   ├── parser/
│   ├── runtime/
│   ├── benchmarks/
│   └── e2e/
├── docs/                    # Documentation
├── examples/                # Example programs
└── tools/                   # Development tools
```

---

## 3. Compilation Pipeline

```
┌─────────────────────────────────────────────────────────────────┐
│                   A16 Compilation Pipeline                       │
└─────────────────────────────────────────────────────────────────┘

Source (.a16)
     │
     ▼
┌─────────────┐
│   Lexer     │  a16_lexer
│   (Rust)    │  - Hand-written for performance
└─────────────┘  - Unicode support
     │           - Error recovery
     │ Token Stream
     ▼
┌─────────────┐
│   Parser    │  a16_parser
│   (Rust)    │  - Recursive descent + Pratt
└─────────────┘  - Full error recovery
     │           - CST → AST transformation
     │ AST
     ▼
┌─────────────┐
│ Name Resolve│  a16_typeck (phase 1)
│   (Rust)    │  - Scope building
└─────────────┘  - Import resolution
     │           - Symbol table
     │ Scoped AST
     ▼
┌─────────────┐
│ Type Check  │  a16_typeck (phase 2)
│   (Rust)    │  - Type inference (HM + extensions)
└─────────────┘  - Constraint solving
     │           - Error reporting
     │ Typed AST
     ▼
┌─────────────┐
│ HIR Lower   │  a16_hir
│   (Rust)    │  - Desugar syntax
└─────────────┘  - Lower AI primitives
     │           - Insert runtime calls
     │ HIR
     ▼
┌─────────────┐
│ HIR Optimize│  a16_hir (optimizer)
│   (Rust)    │  - AI-specific transforms
└─────────────┘  - Parallelization hints
     │           - Cache hints
     │ Optimized HIR
     ▼
┌─────────────┐
│ Bytecode    │  a16_bytecode
│   Emit      │  - Stack-based instructions
│   (Rust)    │  - Constant pool
└─────────────┘  - Debug info
     │
     │ Bytecode (.a16c)
     ▼
┌─────────────────────────────────────────────────────────────────┐
│                        Execution                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │ Interpreter │  │  JIT Tier 1 │  │  JIT Tier 2 │              │
│  │  (a16_vm)   │  │  (Baseline) │  │ (Optimized) │              │
│  └─────────────┘  └─────────────┘  └─────────────┘              │
└─────────────────────────────────────────────────────────────────┘
```

---

## 4. Key Implementation Details

### 4.1 Lexer (a16_lexer)

```rust
// Token structure
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub text: SmolStr,  // Small string optimization
}

pub enum TokenKind {
    // Keywords
    Fn, Class, Agent, Tool, Memory, ...
    
    // Literals
    Int(i64), Float(f64), String(SmolStr), ...
    
    // Operators
    Plus, Minus, Star, Slash, ...
    
    // Delimiters
    LParen, RParen, LBracket, RBracket, ...
    
    // Special
    Newline, Indent, Dedent, Eof, Error,
}

// Indentation handling
struct IndentTracker {
    stack: Vec<usize>,
    pending_dedents: usize,
}
```

### 4.2 Parser (a16_parser)

```rust
// Parser with error recovery
pub struct Parser<'src> {
    lexer: Lexer<'src>,
    current: Token,
    peek: Token,
    errors: Vec<ParseError>,
}

impl Parser<'_> {
    // Pratt parser for expressions
    fn parse_expr(&mut self, min_bp: u8) -> Expr {
        let mut lhs = self.parse_prefix();
        
        while let Some((l_bp, r_bp)) = infix_binding_power(self.current.kind) {
            if l_bp < min_bp { break; }
            let op = self.advance();
            let rhs = self.parse_expr(r_bp);
            lhs = Expr::Binary(Box::new(lhs), op, Box::new(rhs));
        }
        
        lhs
    }
    
    // Agent definition
    fn parse_agent_def(&mut self) -> AgentDef {
        self.expect(TokenKind::Agent);
        let name = self.parse_ident();
        self.expect(TokenKind::Colon);
        self.expect(TokenKind::Newline);
        self.expect(TokenKind::Indent);
        
        let mut members = vec![];
        while !self.check(TokenKind::Dedent) {
            members.push(self.parse_agent_member());
        }
        
        AgentDef { name, members }
    }
}
```

### 4.3 Virtual Machine (a16_vm)

```rust
pub struct VM {
    frames: Vec<Frame>,
    stack: Vec<Value>,
    heap: Heap,
    globals: HashMap<Symbol, Value>,
    
    // AI components
    token_engine: TokenEngine,
    memory_manager: MemoryManager,
    scheduler: Scheduler,
    tool_sandbox: ToolSandbox,
}

impl VM {
    pub fn execute(&mut self, bytecode: &Bytecode) -> Result<Value> {
        loop {
            let op = self.fetch();
            match op {
                Op::PushConst(idx) => {
                    let value = self.const_pool[idx].clone();
                    self.stack.push(value);
                }
                Op::Call(argc) => {
                    let func = self.stack.pop().unwrap();
                    let args = self.stack.drain(..argc).collect();
                    self.call_function(func, args)?;
                }
                Op::ModelInvoke(model_id) => {
                    let prompt = self.stack.pop().unwrap();
                    let result = self.invoke_model(model_id, prompt).await?;
                    self.stack.push(result);
                }
                Op::Return => {
                    if self.frames.len() == 1 {
                        return Ok(self.stack.pop().unwrap_or(Value::None));
                    }
                    self.pop_frame();
                }
                // ... more opcodes
            }
        }
    }
}
```

### 4.4 Token Intelligence Engine

```rust
pub struct TokenEngine {
    tokenizers: HashMap<ModelId, Box<dyn Tokenizer>>,
    dedup_cache: LruCache<u64, TokenizedContent>,
    compression_engine: CompressionEngine,
    budget_tracker: BudgetTracker,
}

impl TokenEngine {
    pub fn count(&self, model: ModelId, text: &str) -> usize {
        let tokenizer = self.tokenizers.get(&model).unwrap();
        tokenizer.encode(text).len()
    }
    
    pub fn deduplicate(&mut self, contexts: &[Context]) -> Vec<Context> {
        let mut seen = HashSet::new();
        let mut result = vec![];
        
        for ctx in contexts {
            let hash = ctx.semantic_hash();
            if seen.insert(hash) {
                result.push(ctx.clone());
            }
        }
        
        result
    }
    
    pub fn compress(&self, context: &Context, target_tokens: usize) -> Context {
        self.compression_engine.compress(context, target_tokens)
    }
}
```

---

## 5. Testing Strategy

### 5.1 Test Categories

| Category | Tool | Coverage Target |
|----------|------|-----------------|
| Unit Tests | `cargo test` | 80%+ |
| Integration Tests | Custom harness | All major flows |
| Property Tests | `proptest` | Parser, type checker |
| Fuzz Tests | `cargo-fuzz` | Lexer, parser |
| Benchmark Tests | `criterion` | All hot paths |
| E2E Tests | `a16 test` | User-facing functionality |

### 5.2 Test Infrastructure

```rust
// tests/parser/test_agent.rs
#[test]
fn test_parse_agent_basic() {
    let source = r#"
agent MyAgent:
    model: gpt4
    tools: [search]
    
    task greet(name: Str) -> Str:
        return "Hello, " + name
"#;
    
    let ast = parse(source).unwrap();
    assert_matches!(ast.items[0], Item::Agent(_));
}

// Snapshot testing
#[test]
fn test_parse_snapshot() {
    let source = include_str!("fixtures/complex_agent.a16");
    let ast = parse(source).unwrap();
    insta::assert_snapshot!(format!("{:#?}", ast));
}
```

### 5.3 CI Pipeline

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all
      - run: cargo clippy -- -D warnings
      - run: cargo fmt --check
      
  fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo +nightly fuzz run parser -- -max_total_time=60
      
  bench:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo bench --no-run  # Compile only
```

---

## 6. Build and Distribution

### 6.1 Build Targets

| Platform | Target | Format |
|----------|--------|--------|
| Linux x64 | `x86_64-unknown-linux-gnu` | Binary + .deb + .rpm |
| Linux ARM | `aarch64-unknown-linux-gnu` | Binary |
| macOS x64 | `x86_64-apple-darwin` | Binary + .pkg |
| macOS ARM | `aarch64-apple-darwin` | Binary + .pkg |
| Windows | `x86_64-pc-windows-msvc` | Binary + .msi |
| WASM | `wasm32-unknown-unknown` | .wasm (playground) |

### 6.2 Release Process

```bash
# Release script
#!/bin/bash
set -e

VERSION=$1

# Update version
sed -i "s/version = .*/version = \"$VERSION\"/" Cargo.toml

# Build all platforms
cross build --release --target x86_64-unknown-linux-gnu
cross build --release --target aarch64-unknown-linux-gnu
cross build --release --target x86_64-apple-darwin
cross build --release --target aarch64-apple-darwin
cross build --release --target x86_64-pc-windows-msvc

# Create installers
./scripts/create-deb.sh $VERSION
./scripts/create-msi.sh $VERSION

# Upload to GitHub Releases
gh release create v$VERSION ./dist/* --title "A16 $VERSION"

# Update Homebrew formula
./scripts/update-homebrew.sh $VERSION
```

---

## 7. Dependencies

### Core Dependencies

```toml
[dependencies]
# Parsing
logos = "0.13"          # Lexer generator
rowan = "0.15"          # Syntax trees

# Data structures
smol_str = "0.2"        # Small string optimization
indexmap = "2.0"        # Ordered maps
dashmap = "5.5"         # Concurrent HashMap

# Async
tokio = { version = "1.35", features = ["full"] }
async-trait = "0.1"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "1.3"

# HTTP (for model APIs)
reqwest = { version = "0.11", features = ["json", "stream"] }

# Vector operations
ndarray = "0.15"
simsimd = "0.1"         # SIMD similarity

# Error handling
thiserror = "1.0"
miette = "5.10"         # Pretty diagnostics
```

### Optional Dependencies

```toml
[features]
jit = ["cranelift"]     # JIT compilation
gpu = ["wgpu"]          # GPU acceleration
enterprise = []         # Enterprise features

[dependencies.cranelift]
version = "0.103"
optional = true
```

---

## 8. Performance Considerations

### Hot Path Optimization

1. **Tokenization:** Use tiktoken's Rust port for GPT, custom for Claude
2. **String interning:** All identifiers interned via `lasso` crate
3. **Bytecode dispatch:** Computed goto (via inline assembly) on supported platforms
4. **Memory allocation:** Arena allocators for AST, pool allocators for runtime objects
5. **Cache-friendly:** Struct-of-arrays for AI memory indices

### Profiling

```bash
# CPU profiling
cargo flamegraph --bin a16 -- run benchmark.a16

# Memory profiling
valgrind --tool=massif ./target/release/a16 run benchmark.a16

# Allocation tracking
MALLOC_CONF="prof:true" ./target/release/a16 run benchmark.a16
```

---

*Next: [Example Programs](./10-examples.md)*
