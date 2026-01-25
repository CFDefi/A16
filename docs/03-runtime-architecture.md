# A16 Runtime + VM/JIT Architecture

> **Version:** 0.1 | **Status:** Draft | **Date:** 2026-01-19

---

## Table of Contents
1. [Architecture Overview](#1-architecture-overview)
2. [Compilation Pipeline](#2-compilation-pipeline)
3. [Virtual Machine Design](#3-virtual-machine-design)
4. [JIT Compiler](#4-jit-compiler)
5. [Token Intelligence Engine](#5-token-intelligence-engine)
6. [Memory Manager](#6-memory-manager)
7. [Async Scheduler](#7-async-scheduler)
8. [Tool Sandbox](#8-tool-sandbox)
9. [Caching Layer](#9-caching-layer)
10. [Debugging & Tracing](#10-debugging--tracing)

---

## 1. Architecture Overview

### 1.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              A16 Application                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                           A16 Standard Library                               │
│    ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐        │
│    │ a16.ai.  │ │ a16.ai.  │ │ a16.ai.  │ │ a16.ai.  │ │ a16.sys. │        │
│    │  agent   │ │  model   │ │  memory  │ │  tools   │ │concurrent│        │
│    └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘        │
├─────────────────────────────────────────────────────────────────────────────┤
│                              A16 Core Runtime                                │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐           │
│  │   Token     │ │   Memory    │ │   Async     │ │    Tool     │           │
│  │  Engine     │ │  Manager    │ │ Scheduler   │ │  Sandbox    │           │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘           │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐           │
│  │   Cache     │ │   Policy    │ │   Trace     │ │   Metric    │           │
│  │   Layer     │ │  Enforcer   │ │  Collector  │ │  Collector  │           │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘           │
├─────────────────────────────────────────────────────────────────────────────┤
│                           A16 Execution Engine                               │
│  ┌─────────────────────────────────────────────────────────────────┐       │
│  │                     Adaptive Executor                            │       │
│  │  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐        │       │
│  │  │  Interpreter  │  │  JIT Tier 1   │  │  JIT Tier 2   │        │       │
│  │  │  (Bytecode)   │  │  (Baseline)   │  │  (Optimized)  │        │       │
│  │  └───────────────┘  └───────────────┘  └───────────────┘        │       │
│  └─────────────────────────────────────────────────────────────────┘       │
├─────────────────────────────────────────────────────────────────────────────┤
│                           A16 Compiler Frontend                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐      │
│  │  Lexer   │→ │  Parser  │→ │   AST    │→ │   HIR    │→ │Bytecode  │      │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘  └──────────┘      │
├─────────────────────────────────────────────────────────────────────────────┤
│                          Operating System + Hardware                         │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 Design Principles

1. **AI-Aware Optimization**: The runtime understands AI workloads (model calls, tool execution, memory retrieval) and optimizes for them.

2. **Progressive Compilation**: Interpret first for fast startup; JIT-compile hot paths for performance.

3. **Automatic Parallelism**: The scheduler identifies independent operations and runs them concurrently.

4. **Token-Centric**: Every token is tracked, cached, and optimized. Token cost is a first-class metric.

5. **Secure by Default**: Tools run sandboxed. Permissions are explicit. Actions are auditable.

---

## 2. Compilation Pipeline

### 2.1 Pipeline Stages

```
Source Code (.a16)
       │
       ▼
┌─────────────────┐
│     LEXER       │  → Token Stream
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│     PARSER      │  → Concrete Syntax Tree (CST)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   AST Builder   │  → Abstract Syntax Tree (AST)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Name Resolver  │  → Scoped AST
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Type Checker   │  → Typed AST
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  HIR Generator  │  → High-Level IR
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  HIR Optimizer  │  → Optimized HIR
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Bytecode Emitter│  → A16 Bytecode
└────────┬────────┘
         │
         ▼
   Execution │ JIT
```

### 2.2 Intermediate Representations

#### AST (Abstract Syntax Tree)
- Direct representation of source structure
- Preserves source locations for error reporting
- Immutable after construction

#### HIR (High-Level Intermediate Representation)
- Desugared (no syntactic sugar)
- Explicit control flow
- Type-annotated
- AI primitives lowered to core operations

```
HIR Node Types:
├── Module, Function, Class
├── Agent, Tool, Memory, Prompt (AI primitives)
├── Block, If, Loop, Match
├── Call, Invoke, Await, Spawn
├── Let, Assign, Return
├── Binary, Unary, Literal
├── TokenBudget, CacheHint, ParallelHint
└── TracePoint, PolicyCheck
```

#### Bytecode
- Stack-based virtual machine instructions
- Compact binary format
- Versioned for compatibility

### 2.3 AI-Specific Optimizations

| Optimization | Description | Benefit |
|--------------|-------------|---------|
| **Prompt Canonicalization** | Normalize prompts to canonical form | Cache hit rate ↑ |
| **Context Deduplication** | Detect and eliminate repeated context | Token usage ↓ |
| **Parallel Inference** | Identify independent model calls | Latency ↓ |
| **Speculative Scheduling** | Pre-schedule likely next operations | Latency ↓ |
| **Memory Prefetch** | Load relevant memories before needed | Latency ↓ |
| **Tool Batching** | Combine independent tool calls | API calls ↓ |

---

## 3. Virtual Machine Design

### 3.1 VM Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         A16 Virtual Machine                      │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │ Instruction │  │   Stack     │  │   Heap      │              │
│  │  Decoder    │  │  Machine    │  │  Manager    │              │
│  └─────────────┘  └─────────────┘  └─────────────┘              │
│                                                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │  Register   │  │   Frame     │  │  Exception  │              │
│  │   File      │  │   Stack     │  │   Handler   │              │
│  └─────────────┘  └─────────────┘  └─────────────┘              │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    AI Extension Unit                      │   │
│  │  ┌────────┐  ┌────────┐  ┌────────┐  ┌────────┐          │   │
│  │  │ Token  │  │ Memory │  │  Tool  │  │ Model  │          │   │
│  │  │Counter │  │Access  │  │Dispatch│  │ Invoke │          │   │
│  │  └────────┘  └────────┘  └────────┘  └────────┘          │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 Instruction Set

#### Core Instructions (Standard VM)
```
; Stack manipulation
PUSH_CONST    <idx>       ; Push constant from pool
PUSH_LOCAL    <slot>      ; Push local variable
PUSH_GLOBAL   <idx>       ; Push global variable
POP                       ; Discard top of stack
DUP                       ; Duplicate top of stack
SWAP                      ; Swap top two elements

; Arithmetic
ADD, SUB, MUL, DIV, MOD
NEG, NOT, BITNOT
AND, OR, XOR, SHL, SHR

; Comparison
EQ, NE, LT, LE, GT, GE
IS, IN

; Control flow
JUMP          <offset>    ; Unconditional jump
JUMP_IF_TRUE  <offset>    ; Conditional jump
JUMP_IF_FALSE <offset>
CALL          <argc>      ; Function call
RETURN                    ; Return from function
RAISE                     ; Raise exception
CATCH         <handler>   ; Set exception handler

; Object operations
GET_ATTR      <name>      ; Get attribute
SET_ATTR      <name>      ; Set attribute
GET_INDEX                 ; Get index (a[i])
SET_INDEX                 ; Set index
BUILD_LIST    <count>     ; Build list from stack
BUILD_DICT    <count>     ; Build dict from stack
BUILD_TUPLE   <count>     ; Build tuple

; Closures
MAKE_CLOSURE  <func_idx>  ; Create closure
LOAD_FREE     <idx>       ; Load from closure
STORE_FREE    <idx>       ; Store to closure
```

#### AI Extension Instructions
```
; Token operations
TOKEN_BUDGET_ENTER <limit>  ; Enter token budget scope
TOKEN_BUDGET_EXIT           ; Exit token budget scope
TOKEN_COUNT                 ; Push current token count
TOKEN_COMPRESS <strategy>   ; Compress context

; Memory operations
MEMORY_STORE    <mem_id>    ; Store to memory
MEMORY_RETRIEVE <mem_id>    ; Retrieve from memory
MEMORY_FORGET   <mem_id>    ; Delete from memory

; Model operations
MODEL_INVOKE    <model_id>  ; Invoke model
MODEL_STREAM    <model_id>  ; Invoke with streaming
MODEL_STRUCTURED <schema>   ; Invoke with schema

; Tool operations
TOOL_DISPATCH   <tool_id>   ; Dispatch tool call
TOOL_AWAIT                  ; Await tool result
TOOL_BATCH_BEGIN            ; Start tool batch
TOOL_BATCH_END              ; Execute batched tools

; Agent operations
AGENT_SPAWN     <agent_id>  ; Spawn agent
AGENT_SEND      <agent_id>  ; Send message to agent
AGENT_RECEIVE               ; Receive message
AGENT_TERMINATE             ; Terminate agent

; Concurrency
ASYNC_SPAWN     <func_idx>  ; Spawn async task
ASYNC_AWAIT                 ; Await task
PARALLEL_BEGIN              ; Start parallel block
PARALLEL_END                ; End parallel block

; Tracing/Debugging
TRACE_EMIT      <event>     ; Emit trace event
BREAKPOINT                  ; Debugger breakpoint
```

### 3.3 Object Representation

```
┌─────────────────────────────────────────┐
│              A16 Object Header          │
├─────────────────────────────────────────┤
│  Type Tag        (8 bits)               │
│  Flags           (8 bits)               │
│    - Immutable                          │
│    - AI-Tracked                         │
│    - Token-Counted                      │
│    - Cached                             │
│  Reference Count (48 bits)              │
├─────────────────────────────────────────┤
│  Type-Specific Data                     │
│  (varies by type)                       │
└─────────────────────────────────────────┘
```

**Type Tags:**
- 0x00-0x0F: Primitives (Int, Float, Bool, None)
- 0x10-0x1F: Strings and Bytes
- 0x20-0x2F: Collections (List, Dict, Set, Tuple)
- 0x30-0x3F: Functions and Closures
- 0x40-0x4F: Classes and Instances
- 0x50-0x5F: AI Types (Agent, Tool, Memory, Prompt)
- 0x60-0x6F: Async Types (Task, Channel, Future)
- 0xF0-0xFF: Special (Error, Undefined, Deleted)

---

## 4. JIT Compiler

### 4.1 Tiered Compilation Strategy

```
┌─────────────────────────────────────────────────────────────────┐
│                      Execution Profile                           │
│                                                                  │
│  Cold Code                    Hot Code                           │
│  (executed once)              (executed often)                   │
│       │                             │                            │
│       ▼                             ▼                            │
│  ┌──────────────┐             ┌──────────────┐                  │
│  │ Interpreter  │──count>50──▶│  JIT Tier 1  │                  │
│  │  (Bytecode)  │             │  (Baseline)  │                  │
│  └──────────────┘             └──────┬───────┘                  │
│                                      │                           │
│                                 count>1000                       │
│                                      │                           │
│                                      ▼                           │
│                               ┌──────────────┐                  │
│                               │  JIT Tier 2  │                  │
│                               │ (Optimized)  │                  │
│                               └──────────────┘                  │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 Tier 1: Baseline JIT
- Quick compilation (< 1ms per function)
- Direct bytecode-to-machine-code translation
- Minimal optimization
- Collects profiling data for Tier 2

### 4.3 Tier 2: Optimizing JIT

**Standard Optimizations:**
- Inlining (aggressive for small functions)
- Dead code elimination
- Constant propagation
- Loop-invariant code motion
- Register allocation (linear scan)
- Escape analysis (stack allocation)

**AI-Specific Optimizations:**

| Optimization | Trigger | Action |
|--------------|---------|--------|
| **Model Call Fusion** | Multiple model calls with same config | Batch into single call |
| **Memory Prefetch** | Memory retrieve before model call | Start retrieval early |
| **Context Caching** | Repeated context patterns | Cache tokenized context |
| **Speculative Tool** | Tool call + conditional | Start tool speculatively |
| **Stream Fusion** | Chained streaming operations | Fuse into single stream |

### 4.4 Deoptimization
When assumptions are violated:
1. Save current state
2. Map machine state to interpreter state
3. Continue in interpreter
4. Re-profile and re-compile if needed

---

## 5. Token Intelligence Engine

### 5.1 Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Token Intelligence Engine                     │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                     Token Counter                         │   │
│  │  ┌───────────┐  ┌───────────┐  ┌───────────┐             │   │
│  │  │ Tokenizer │  │  Budget   │  │   Cost    │             │   │
│  │  │   Cache   │  │  Tracker  │  │ Estimator │             │   │
│  │  └───────────┘  └───────────┘  └───────────┘             │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                   Context Optimizer                       │   │
│  │  ┌───────────┐  ┌───────────┐  ┌───────────┐             │   │
│  │  │ Dedup     │  │ Compress  │  │ Canonicalize│            │   │
│  │  │ Engine    │  │  Engine   │  │  Engine   │             │   │
│  │  └───────────┘  └───────────┘  └───────────┘             │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    Response Cache                         │   │
│  │  ┌───────────┐  ┌───────────┐  ┌───────────┐             │   │
│  │  │  Exact    │  │ Semantic  │  │  Partial  │             │   │
│  │  │   Match   │  │   Match   │  │   Match   │             │   │
│  │  └───────────┘  └───────────┘  └───────────┘             │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

### 5.2 Token Counting

```a16
# Runtime tracks tokens automatically
model.invoke(prompt)
# Internally:
#   1. Tokenize prompt → count input tokens
#   2. Invoke model → count output tokens
#   3. Update budget tracker
#   4. Emit token metrics
```

**Tokenizer Support:**
- GPT (tiktoken cl100k_base)
- Claude (claude tokenizer)
- Llama (sentencepiece)
- Custom (plugin interface)

### 5.3 Deduplication Engine

Identifies and eliminates repeated content:

```
Before (Naive):
  System: "You are a helpful assistant. [500 tokens]"
  User: "What is Python?"
  → Total: 520 tokens

  System: "You are a helpful assistant. [500 tokens]"
  User: "What is Rust?"
  → Total: 520 tokens

After (A16):
  System: [cached: ID=0x1A2B] "You are a helpful assistant."
  User: "What is Python?"
  → Cost: 20 tokens (system cached)

  System: [cache hit: ID=0x1A2B]
  User: "What is Rust?"
  → Cost: 18 tokens (system cached)
```

### 5.4 Compression Strategies

| Strategy | When | How | Ratio |
|----------|------|-----|-------|
| **Summarize** | Long context (>4K tokens) | LLM summarization | 5-10x |
| **Truncate** | Sliding window | Keep last N items | Variable |
| **Extract** | Information retrieval | Keep relevant only | 3-5x |
| **Symbolic** | Repeated patterns | Replace with symbols | 2-3x |
| **Delta** | Sequential messages | Store only changes | 2-4x |

### 5.5 Response Caching

**Exact Match Cache:**
```python
cache_key = hash(model_id, prompt_hash, params_hash)
if cache_key in exact_cache:
    return exact_cache[cache_key]
```

**Semantic Match Cache:**
```python
query_embedding = embed(prompt)
similar = semantic_index.search(query_embedding, threshold=0.95)
if similar:
    return adapt_response(similar.response, prompt)
```

**Partial Match Cache:**
```python
# Cache intermediate results
if prefix in partial_cache:
    return extend_from(partial_cache[prefix], remaining)
```

---

## 6. Memory Manager

### 6.1 Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                       Memory Manager                             │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    Object Heap                            │   │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐      │   │
│  │  │ Young   │  │  Old    │  │ Large   │  │ Pinned  │      │   │
│  │  │  Gen    │  │  Gen    │  │ Object  │  │  Objects│      │   │
│  │  └─────────┘  └─────────┘  └─────────┘  └─────────┘      │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    AI Memory Store                        │   │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐      │   │
│  │  │ Vector  │  │ Episodic│  │ Working │  │ Skill   │      │   │
│  │  │  Index  │  │  Store  │  │ Memory  │  │  Store  │      │   │
│  │  └─────────┘  └─────────┘  └─────────┘  └─────────┘      │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    Persistent Store                       │   │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐                   │   │
│  │  │ SQLite  │  │ RocksDB │  │  Qdrant │                   │   │
│  │  │ (meta)  │  │ (KV)    │  │ (vector)│                   │   │
│  │  └─────────┘  └─────────┘  └─────────┘                   │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

### 6.2 Object Heap (Runtime Memory)

**Generational GC:**
- **Young Generation**: Bump allocation, frequent minor GC
- **Old Generation**: Compacting GC, infrequent major GC
- **Large Objects**: Separate allocation, immediate reclaim
- **Pinned Objects**: No movement (for FFI, caching)

**GC Triggers:**
- Young gen 80% full → Minor GC
- Old gen 70% full → Major GC
- Memory pressure signal from OS

### 6.3 AI Memory Store

**Vector Index:**
- HNSW index for semantic search
- Hybrid: sparse + dense vectors
- Automatic index updates
- Configurable dimensions (768, 1024, 1536)

**Episodic Store:**
- Event-based storage
- Automatic summarization
- Temporal indexing
- Configurable retention

**Working Memory:**
- Sliding window
- Fast access (in-memory)
- Automatic eviction

**Skill Store:**
- Learned procedures
- Function extraction from traces
- Reusable tool chains

### 6.4 Memory Operations API

```a16
# Store
let id = memory.store(
    content="User prefers concise answers",
    metadata={"type": "preference", "confidence": 0.9}
)

# Retrieve
let results = memory.retrieve(
    query="What does the user prefer?",
    k=5,
    filter={"type": "preference"},
    threshold=0.7
)

# Update
memory.update(id, content="User prefers very concise answers")

# Forget
memory.forget(filter={"older_than": "90d"})

# Compress
memory.compress(strategy="summarize", threshold=1000)

# Export/Import
let snapshot = memory.export()
memory.import(snapshot)
```

---

## 7. Async Scheduler

### 7.1 Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                       Async Scheduler                            │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    Task Manager                           │   │
│  │  ┌─────────────────┐  ┌─────────────────┐                │   │
│  │  │   Task Queue    │  │   Priority      │                │   │
│  │  │ (Lock-free MPMC)│  │     Heap        │                │   │
│  │  └─────────────────┘  └─────────────────┘                │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    Worker Pool                            │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐         │   │
│  │  │Worker 1 │ │Worker 2 │ │Worker 3 │ │Worker N │         │   │
│  │  │ (Core 0)│ │ (Core 1)│ │ (Core 2)│ │(Core N-1│         │   │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘         │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    I/O Reactor                            │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │   │
│  │  │  Network    │  │    File     │  │   Timer     │       │   │
│  │  │   Poller    │  │   Watcher   │  │   Wheel     │       │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘       │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

### 7.2 Task Model

```
Task States:
  Created → Ready → Running → (Suspended | Completed | Failed)
                        ↑           │
                        └───────────┘

Task Structure:
  ┌─────────────────────────────────┐
  │           Task                   │
  ├─────────────────────────────────┤
  │ ID: u64                         │
  │ State: TaskState                │
  │ Priority: u8                    │
  │ Coroutine: Frame                │
  │ Context: ExecutionContext       │
  │ Parent: Option<TaskId>          │
  │ Children: Vec<TaskId>           │
  │ WaitingOn: Option<Future>       │
  │ TokenBudget: Option<Budget>     │
  │ Trace: TraceBuffer              │
  └─────────────────────────────────┘
```

### 7.3 Scheduling Policies

| Policy | Description | Use Case |
|--------|-------------|----------|
| **Fair** | Round-robin with time slices | Default for most workloads |
| **Priority** | Higher priority runs first | Critical agent tasks |
| **Latency** | Minimize response time | Interactive agents |
| **Throughput** | Maximize tasks/second | Batch processing |
| **Token-Aware** | Prioritize by token budget | Cost optimization |

### 7.4 Work Stealing

Workers steal from other workers when idle:
1. Check local queue
2. If empty, try stealing from random worker
3. If all local queues empty, check global queue
4. If global empty, park and wait for wakeup

### 7.5 Parallel Primitives

```a16
# Parallel block (all run concurrently)
parallel:
    let a = fetch_data(url1)
    let b = fetch_data(url2)
    let c = fetch_data(url3)
# Continues after all complete

# Spawn (fire and forget)
let handle = spawn long_running_task()
# Continue immediately

# Select (first to complete wins)
match select:
    case response = await model_call():
        handle_response(response)
    case timeout = await sleep(30s):
        handle_timeout()

# Fan-out / fan-in
let results = await parallel_map(urls, fetch_data)
```

---

## 8. Tool Sandbox

### 8.1 Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Tool Sandbox                              │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                  Permission Manager                       │   │
│  │  ┌───────────┐  ┌───────────┐  ┌───────────┐             │   │
│  │  │  Policy   │  │  Checker  │  │  Auditor  │             │   │
│  │  │   Store   │  │           │  │           │             │   │
│  │  └───────────┘  └───────────┘  └───────────┘             │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                  Isolation Layer                          │   │
│  │  ┌───────────┐  ┌───────────┐  ┌───────────┐             │   │
│  │  │  Process  │  │   WASM    │  │  Container│             │   │
│  │  │ Sandbox   │  │  Runtime  │  │  Runtime  │             │   │
│  │  └───────────┘  └───────────┘  └───────────┘             │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                  Resource Limiter                         │   │
│  │  ┌───────────┐  ┌───────────┐  ┌───────────┐             │   │
│  │  │   CPU     │  │  Memory   │  │  Network  │             │   │
│  │  │  Quota    │  │   Limit   │  │ Bandwidth │             │   │
│  │  └───────────┘  └───────────┘  └───────────┘             │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

### 8.2 Permission Model

```a16
# Permission types
enum Permission:
    Network(domains: List[Str])
    FileSystem(paths: List[Path], mode: Mode)
    Process(commands: List[Str])
    Environment(vars: List[Str])
    Memory(limit: Bytes)
    CPU(limit: Duration)
    GPU(allowed: Bool)

# Tool declaration
tool web_search:
    permissions: [
        Network(domains=["*.google.com", "*.bing.com"]),
        Memory(limit=100MB),
        CPU(limit=5s)
    ]
    
    fn execute(query: Str) -> List[Result]:
        # Can only access allowed domains
        # Killed if exceeds resource limits
        ...
```

### 8.3 Sandbox Levels

| Level | Isolation | Performance | Use Case |
|-------|-----------|-------------|----------|
| **None** | Same process | 100% | Trusted stdlib tools |
| **Light** | Capability-restricted | 95% | Internal tools |
| **Medium** | Separate process | 80% | External tools |
| **Heavy** | WASM | 60% | Untrusted code |
| **Maximum** | Container | 40% | High-risk tools |

### 8.4 Audit Logging

```a16
# Every tool call is logged
AuditLog:
    timestamp: DateTime
    agent_id: Str
    tool_id: Str
    action: Str
    params: Dict
    result_summary: Str
    duration_ms: Int
    tokens_used: Int
    policy_checks: List[Check]
    violations: List[Violation]
```

---

## 9. Caching Layer

### 9.1 Cache Hierarchy

```
┌─────────────────────────────────────────────────────────┐
│                     L1: Hot Cache                        │
│              (In-Memory, Per-Worker, <100μs)             │
├─────────────────────────────────────────────────────────┤
│                    L2: Warm Cache                        │
│              (Shared Memory, <1ms)                       │
├─────────────────────────────────────────────────────────┤
│                    L3: Cold Cache                        │
│              (SSD/Disk, <10ms)                           │
├─────────────────────────────────────────────────────────┤
│                   L4: Remote Cache                       │
│              (Network/Redis, <50ms)                      │
└─────────────────────────────────────────────────────────┘
```

### 9.2 Cache Types

| Cache | Keys | Values | TTL |
|-------|------|--------|-----|
| **Token Cache** | Text hash | Token IDs | Infinite |
| **Embedding Cache** | Text hash | Vector | 24h |
| **Response Cache** | (Model, Prompt, Params) | Response | 1h |
| **Memory Cache** | Query embedding | Results | 5m |
| **Tool Cache** | (Tool, Args) | Result | Varies |

### 9.3 Cache Policies

```a16
@cache(
    ttl=3600,                    # Time to live
    max_size=1000,               # Max entries
    eviction="lru",              # LRU/LFU/TTL
    key_fn=(args) => hash(args), # Custom key function
    condition=(result) => result.success, # Only cache if true
    refresh_on_hit=True          # Extend TTL on hit
)
fn expensive_operation(query: Str) -> Result:
    ...
```

---

## 10. Debugging & Tracing

### 10.1 Trace System

```
┌─────────────────────────────────────────────────────────────────┐
│                       Trace System                               │
├─────────────────────────────────────────────────────────────────┤
│  Trace Event Types:                                              │
│  ├── Function Enter/Exit                                        │
│  ├── Model Invoke/Response                                       │
│  ├── Tool Dispatch/Complete                                      │
│  ├── Memory Store/Retrieve                                       │
│  ├── Token Budget Update                                         │
│  ├── Agent Message Send/Receive                                  │
│  ├── Exception Raise/Catch                                       │
│  └── Custom User Events                                          │
├─────────────────────────────────────────────────────────────────┤
│  Output Formats:                                                 │
│  ├── JSON Lines (structured)                                    │
│  ├── OpenTelemetry (distributed tracing)                        │
│  ├── Chrome Trace (visualization)                               │
│  └── Console (human-readable)                                   │
└─────────────────────────────────────────────────────────────────┘
```

### 10.2 Debugger Features

```
┌─────────────────────────────────────────────────────────────────┐
│                       A16 Debugger                               │
├─────────────────────────────────────────────────────────────────┤
│  Standard Features:                                              │
│  ├── Breakpoints (line, conditional, exception)                 │
│  ├── Step (in, over, out)                                       │
│  ├── Variable inspection                                        │
│  ├── Call stack navigation                                      │
│  └── Expression evaluation                                       │
├─────────────────────────────────────────────────────────────────┤
│  AI-Specific Features:                                           │
│  ├── Token budget visualization                                  │
│  ├── Memory state inspector                                     │
│  ├── Agent message timeline                                     │
│  ├── Model call replay                                          │
│  ├── Tool execution trace                                       │
│  └── Prompt/Response diff                                       │
├─────────────────────────────────────────────────────────────────┤
│  Time-Travel Debugging:                                          │
│  ├── Record execution                                           │
│  ├── Replay forward/backward                                    │
│  ├── State snapshots                                            │
│  └── Branching ("what if")                                      │
└─────────────────────────────────────────────────────────────────┘
```

### 10.3 Profiler

```a16
# Enable profiling
with profile() as p:
    result = agent.run(task)

# Report
print(p.report())
# Output:
#   Total time: 2.34s
#   Token usage: 4,521 (input: 3,200, output: 1,321)
#   Model calls: 3 (avg: 450ms)
#   Tool calls: 5 (avg: 120ms)
#   Memory operations: 12 (avg: 5ms)
#   Cache hits: 8/15 (53%)
#   
#   Hot spots:
#     1. model.invoke (web_search_prompt) - 1.2s, 2100 tokens
#     2. tool.execute (web_search) - 0.6s
#     3. memory.retrieve - 0.3s
```

### 10.4 Observability Integration

**Metrics (Prometheus-compatible):**
- `a16_tokens_total{type="input|output", model="..."}`
- `a16_model_latency_seconds{model="..."}`
- `a16_tool_latency_seconds{tool="..."}`
- `a16_cache_hits_total{cache="..."}`
- `a16_errors_total{type="..."}`

**Tracing (OpenTelemetry):**
- Distributed trace context propagation
- Span attributes for AI metadata
- Automatic instrumentation

**Logging (Structured):**
- JSON-formatted logs
- Correlation IDs
- Severity levels with AI-aware categories

---

## Appendix: Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Cold start | < 50ms | First instruction execution |
| Function call | < 50ns | Overhead per call |
| Model invoke | < 5ms | Overhead (excluding network) |
| Tool dispatch | < 1ms | Sandbox setup |
| Memory store | < 100μs | In-memory |
| Memory retrieve | < 10ms | Vector search |
| Cache lookup | < 10μs | L1 hit |
| Context switch | < 1μs | Async task switch |

---

*Next: [Standard Library Design](./04-standard-library.md)*
