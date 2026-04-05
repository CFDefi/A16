# A16

**A performance-first language for fast, stable AI**

## Overview

A16 is a cutting-edge programming language and runtime environment designed specifically for AI applications where performance, stability, and resource efficiency are critical. Built from the ground up to address the unique challenges of modern AI workloads, A16 delivers exceptional speed while maintaining minimal resource consumption.

## Key Features

- **⚡ Blazing Fast Performance** — Optimized bytecode VM with sub-millisecond execution
- **🛡️ Rock-Solid Stability** — Comprehensive type system with 212+ unit tests, zero warnings
- **💾 Low Resource Usage** — Zero-copy tensor operations, efficient memory layout
- **🎯 AI-Optimized** — Built-in tensors, embeddings, vector memory, and agent runtime
- **🔧 Developer-Friendly** — Python-like syntax with autograd support
- **🔌 Extensible** — FFI for native extensions, LSP for IDE integration
- **🏗️ Self-Hosting** — Bootstrap compiler written in A16 itself

## Quick Start

### Installation

```bash
git clone https://github.com/CFDefi/A16.git
cd A16
cargo build --release
cargo run --bin a16 -- run examples/tensor_demo.a16
```

### Your First A16 Program

```a16
fn main():
    let x = tensor_ones([2, 3])
    let w = tensor_randn([3, 2])
    let y = tensor_matmul(x, w)
    let out = tensor_relu(y)
    println(tensor_shape(out))
    return 0
```

## Language Features

### AI Agents
```a16
agent ResearchBot:
    model: "gpt-4"
    task summarize(text: Str) -> Str:
        return generate(text)
```

### Async / Concurrency
```a16
async fn fetch_data(url: Str) -> Str:
    let resp = await http_get(url)
    return resp

fn main():
    let tasks = [fetch_data("a"), fetch_data("b")]
    let results = await join_all(tasks)
```

### FFI / Native Extensions
```a16
extern "mathlib":
    fn fast_sin(x: Float) -> Float
    fn fast_cos(x: Float) -> Float
```

### Pattern Matching
```a16
match value:
    case Some(x):
        println(x)
    case None:
        println("empty")
```

### Tensors & Autograd
```a16
let a = tensor_zeros([4, 4])
let b = tensor_randn([4, 4])
let c = tensor_matmul(a, b)
tensor_backward(c)
```

### Vector Memory
```a16
let mem = memory_new(64)
let v = embed("hello world")
memory_store(mem, v)
let results = memory_retrieve(mem, embed("hi"))
```

## CLI

```bash
a16 run file.a16          # Run a program
a16 check file.a16        # Type check
a16 fmt file.a16          # Format source code
a16 lint file.a16         # Run linter
a16 doc file.a16          # Generate documentation
a16 parse file.a16        # Show AST
a16 lex file.a16          # Show tokens
a16 bootstrap             # Run self-hosting compiler test
a16 repl                  # Interactive REPL
a16 doctor                # Environment check
```

## Project Structure

```
A16/
├── crates/
│   ├── a16_lexer/       # Tokenization (logos-based)
│   ├── a16_ast/         # Abstract syntax tree
│   ├── a16_parser/      # Recursive descent parser
│   ├── a16_typeck/      # Hindley-Milner type checker
│   ├── a16_hir/         # High-level IR (desugaring)
│   ├── a16_codegen/     # Bytecode compiler
│   ├── a16_vm/          # Stack-based virtual machine
│   ├── a16_cli/         # Command-line interface
│   ├── a16_tensor/      # N-D tensors + autograd
│   ├── a16_vector/      # HNSW index + TF-IDF embedder
│   ├── a16_runtime/     # Async executor + channels
│   ├── a16_resolver/    # Module graph + dependency resolver
│   ├── a16_pkg/         # Package manager (manifest, lockfile)
│   ├── a16_ffi/         # FFI registry + type marshaling
│   ├── a16_fmt/         # Source code formatter
│   ├── a16_lint/        # Rule-based linter (6 rules)
│   ├── a16_doc/         # Documentation generator
│   ├── a16_lsp/         # Language server (diagnostics, completion, hover)
│   └── a16_stdlib/      # Extended stdlib (math, string, collections, json, io, os)
├── bootstrap/           # Self-hosting compiler in A16
│   ├── lexer.a16        # Bootstrap tokenizer
│   ├── parser.a16       # Bootstrap parser
│   ├── codegen.a16      # Bootstrap bytecode compiler
│   ├── vm.a16           # Bootstrap virtual machine
│   └── main.a16         # Bootstrap entry point
├── examples/            # Sample programs
└── docs/                # Documentation
```

## Stats

| Metric | Value |
|--------|-------|
| **Crates** | 19 |
| **Tests** | 212 |
| **Test Pass Rate** | 100% |
| **Compiler Warnings** | 0 |
| **Milestones** | 16/16 complete |

## Copyright and License

**© 2026 Ace VanFossen. All rights reserved.**

This is proprietary software. No permission is granted to use, copy, modify, or distribute without written permission from the copyright holder.

For licensing inquiries, contact: vanfossenace@gmail.com
