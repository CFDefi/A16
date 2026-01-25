# A16

**A performance-first language for fast, stable AI**

## Overview

A16 is a cutting-edge programming language and runtime environment designed specifically for AI applications where performance, stability, and resource efficiency are critical. Built from the ground up to address the unique challenges of modern AI workloads, A16 delivers exceptional speed while maintaining minimal resource consumption.

## Key Features

- **⚡ Blazing Fast Performance** - Optimized bytecode VM with sub-millisecond execution
- **🛡️ Rock-Solid Stability** - Comprehensive type system with 75+ unit tests
- **💾 Low Resource Usage** - Zero-copy tensor operations, efficient memory layout
- **🎯 AI-Optimized** - Built-in tensors, embeddings, and vector memory
- **🔧 Developer-Friendly** - Python-like syntax with autograd support

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

## AI Runtime

### Tensors
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

## Project Structure

```
A16/
├── crates/
│   ├── a16_lexer/      # Tokenization
│   ├── a16_parser/     # AST construction
│   ├── a16_typeck/     # Type checking
│   ├── a16_codegen/    # Bytecode generation
│   ├── a16_vm/         # Virtual machine
│   ├── a16_tensor/     # Tensor ops + autograd
│   └── a16_vector/     # Embeddings + HNSW
├── examples/           # Sample programs
└── docs/               # Documentation
```

## Copyright and License

**© 2026 Ace VanFossen. All rights reserved.**

This is proprietary software. No permission is granted to use, copy, modify, or distribute without written permission from the copyright holder.

For licensing inquiries, contact: vanfossenace@gmail.com
