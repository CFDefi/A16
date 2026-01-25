# Changelog

All notable changes to A16 will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-01-25

### Added
- **Core Language**
  - Python-like syntax with indentation-based blocks
  - Full type system with inference
  - Functions, classes, and modules
  - Pattern matching and iterators

- **AI Runtime** (NEW)
  - `a16_tensor` crate: N-dimensional tensors with autograd
    - `tensor_zeros`, `tensor_ones`, `tensor_randn` - Creation
    - `tensor_add`, `tensor_sub`, `tensor_mul`, `tensor_matmul` - Operations
    - `tensor_relu`, `tensor_mean` - Activations & reductions
    - `tensor_backward`, `tensor_grad` - Automatic differentiation
    - `tensor_shape`, `tensor_data`, `tensor_from_list` - Introspection
  - `a16_vector` crate: Vector similarity search
    - HNSW index for O(log n) approximate nearest neighbor
    - TF-IDF text embedder
    - `embed`, `memory_new`, `memory_store`, `memory_retrieve`

- **Standard Library**
  - 35+ built-in functions
  - String, list, and dict operations
  - Type conversion utilities

- **Developer Experience**
  - Comprehensive error messages
  - 75 unit tests
  - API documentation

### Infrastructure
- GitHub Actions CI/CD pipelines
- Multi-platform builds (Windows, Linux, macOS)
- Release automation

## [Unreleased]
- GPU acceleration for tensor operations
- Neural network layers (Linear, Conv2d)
- Model serialization
