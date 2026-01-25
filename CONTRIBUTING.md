# Contributing to A16

Thank you for your interest in contributing to A16!

## Development Setup

```bash
# Clone the repository
git clone https://github.com/a16-lang/a16.git
cd a16

# Build the project
cargo build --workspace

# Run tests
cargo test --workspace

# Run the CLI
cargo run --bin a16 -- run examples/tensor_demo.a16
```

## Project Structure

```
crates/
├── a16_lexer/     # Tokenization
├── a16_parser/    # AST construction
├── a16_ast/       # Abstract syntax tree
├── a16_hir/       # High-level IR
├── a16_typeck/    # Type checking
├── a16_codegen/   # Bytecode generation
├── a16_vm/        # Virtual machine
├── a16_cli/       # Command-line interface
├── a16_tensor/    # Tensor operations + autograd
└── a16_vector/    # Vector embeddings + HNSW
```

## Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`cargo test --workspace`)
5. Run lints (`cargo clippy --workspace`)
6. Format code (`cargo fmt --all`)
7. Commit your changes (`git commit -m 'Add amazing feature'`)
8. Push to the branch (`git push origin feature/amazing-feature`)
9. Open a Pull Request

## Code Style

- Follow Rust conventions (rustfmt, clippy)
- Add tests for new features
- Document public APIs with doc comments
- Keep error messages helpful and descriptive

## Running Benchmarks

```bash
cargo bench --package a16_tensor
```

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
