# A16 AI Runtime API Reference

A16 includes built-in AI capabilities for tensors, embeddings, and vector memory.

## Tensor Operations

### Creation

| Function | Signature | Description |
|----------|-----------|-------------|
| `tensor_zeros(shape)` | `[Int] -> Tensor` | Create tensor filled with zeros |
| `tensor_ones(shape)` | `[Int] -> Tensor` | Create tensor filled with ones |
| `tensor_randn(shape)` | `[Int] -> Tensor` | Create tensor with random values |
| `tensor_from_list(data, shape)` | `[Float], [Int] -> Tensor` | Create tensor from data list |

**Example:**
```a16
let a = tensor_zeros([2, 3])     # 2x3 matrix of zeros
let b = tensor_randn([4, 4])     # 4x4 random matrix
let c = tensor_from_list([1.0, 2.0, 3.0, 4.0], [2, 2])
```

### Arithmetic

| Function | Signature | Description |
|----------|-----------|-------------|
| `tensor_add(a, b)` | `Tensor, Tensor -> Tensor` | Element-wise addition |
| `tensor_sub(a, b)` | `Tensor, Tensor -> Tensor` | Element-wise subtraction |
| `tensor_mul(a, b)` | `Tensor, Tensor -> Tensor` | Element-wise multiplication |
| `tensor_matmul(a, b)` | `Tensor, Tensor -> Tensor` | Matrix multiplication |

**Error Handling:**
- Shape mismatch errors are reported with details
- Dimension compatibility is checked for matmul

### Activations & Reductions

| Function | Signature | Description |
|----------|-----------|-------------|
| `tensor_relu(t)` | `Tensor -> Tensor` | ReLU activation (max(0, x)) |
| `tensor_mean(t)` | `Tensor -> Tensor` | Mean of all elements (scalar tensor) |

### Autograd

| Function | Signature | Description |
|----------|-----------|-------------|
| `tensor_backward(t)` | `Tensor -> None` | Compute gradients via backprop |
| `tensor_grad(t)` | `Tensor -> Tensor?` | Get gradient (None if not computed) |

### Introspection

| Function | Signature | Description |
|----------|-----------|-------------|
| `tensor_shape(t)` | `Tensor -> [Int]` | Get shape as list |
| `tensor_data(t)` | `Tensor -> [Float]` | Get data as flat list |
| `tensor_item(t)` | `Tensor -> Float` | Get scalar value (size must be 1) |

---

## Embeddings

| Function | Signature | Description |
|----------|-----------|-------------|
| `embed(text)` | `Str -> Vector` | Embed text into 64-dim vector (TF-IDF) |

**Example:**
```a16
let v = embed("hello world")
println(v)  # <Vector len=64>
```

---

## Vector Memory (HNSW Index)

| Function | Signature | Description |
|----------|-----------|-------------|
| `memory_new(dim)` | `Int -> MemoryIndex` | Create new vector index |
| `memory_store(mem, vec)` | `MemoryIndex, Vector -> None` | Store vector in index |
| `memory_retrieve(mem, query)` | `MemoryIndex, Vector -> [Float]` | Find similar vectors (returns scores) |
| `memory_retrieve(mem, k)` | `MemoryIndex, Int -> Int` | Get index size |

**Example:**
```a16
let mem = memory_new(64)
let v1 = embed("hello")
let v2 = embed("goodbye")

memory_store(mem, v1)
memory_store(mem, v2)

let query = embed("hi")
let results = memory_retrieve(mem, query)
println(results)  # [0.8, 0.2, ...]
```

---

## Error Messages

The AI runtime provides helpful error messages:

| Error | Example Message |
|-------|-----------------|
| Type mismatch | `expected Tensor, got Str` |
| Shape mismatch | `shape mismatch: [2, 3] vs [4, 5]` |
| Dimension error | `dimension mismatch: 3 columns vs 4 rows` |
| Invalid shape | `shape cannot be empty` |
| Size error | `tensor must have exactly 1 element, got 5` |

---

## Complete Example

```a16
fn main():
    # Create weights
    let w = tensor_randn([3, 2])
    let x = tensor_ones([2, 3])
    
    # Forward pass
    let y = tensor_matmul(x, w)
    let out = tensor_relu(y)
    
    # Inspect
    println(tensor_shape(out))  # [2, 2]
    println(tensor_data(out))   # [values...]
    
    return 0
```
