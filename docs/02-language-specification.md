# A16 Language Specification v0.1

> **Version:** 0.1 | **Status:** Draft | **Date:** 2026-01-19

---

## Table of Contents
1. [Lexical Structure](#1-lexical-structure)
2. [Syntax Grammar (EBNF)](#2-syntax-grammar-ebnf)
3. [Types and Type System](#3-types-and-type-system)
4. [Variables and Bindings](#4-variables-and-bindings)
5. [Functions and Callables](#5-functions-and-callables)
6. [Classes and Objects](#6-classes-and-objects)
7. [AI Primitives](#7-ai-primitives)
8. [Modules and Imports](#8-modules-and-imports)
9. [Error Handling](#9-error-handling)
10. [Concurrency Model](#10-concurrency-model)
11. [Memory Model](#11-memory-model)
12. [Execution Modes](#12-execution-modes)

---

## 1. Lexical Structure

### 1.1 Character Set
A16 source files are UTF-8 encoded. Identifiers support Unicode letters.

### 1.2 Line Structure
- **Indentation-based blocks** (like Python)
- **Indent unit:** 4 spaces (configurable, tabs converted)
- **Line continuation:** trailing `\` or open brackets `([{`
- **Comments:** `#` for line comments, `###` for doc comments

```a16
# This is a line comment

### 
This is a documentation comment.
Supports markdown formatting.
###
fn my_function():
    pass
```

### 1.3 Keywords

**Core Keywords:**
```
fn       class    agent    tool     memory   prompt
if       elif     else     match    case
for      while    loop     break    continue return
try      except   finally  raise    assert
import   from     as       export
async    await    spawn    yield
let      const    type     enum     struct
and      or       not      in       is
with     pass     del      None     True     False
```

**AI-Specific Keywords:**
```
agent    tool     memory   prompt   budget   model
context  retrieve store    compress stream   invoke
sandbox  audit    permit   deny     policy   trace
```

### 1.4 Identifiers
```ebnf
identifier     = letter (letter | digit | "_")*
letter         = "a"..."z" | "A"..."Z" | "_" | <unicode_letter>
digit          = "0"..."9"
private_ident  = "_" identifier
dunder_ident   = "__" identifier "__"
```

### 1.5 Literals

```ebnf
integer        = decimal_int | hex_int | octal_int | binary_int
decimal_int    = digit (digit | "_")*
hex_int        = "0x" hex_digit (hex_digit | "_")*
octal_int      = "0o" octal_digit (octal_digit | "_")*
binary_int     = "0b" ("0" | "1") ("0" | "1" | "_")*

float          = digit+ "." digit+ [exponent]
               | digit+ exponent
exponent       = ("e" | "E") ["+" | "-"] digit+

string         = '"' string_char* '"'
               | "'" string_char* "'"
               | '"""' multiline_char* '"""'
               | "'''" multiline_char* "'''"
               
fstring        = "f" string
raw_string     = "r" string
prompt_string  = "p" '"""' prompt_content '"""'

boolean        = "True" | "False"
none           = "None"
```

---

## 2. Syntax Grammar (EBNF)

### 2.1 Program Structure

```ebnf
program         = statement*

statement       = simple_stmt NEWLINE
                | compound_stmt

simple_stmt     = expr_stmt
                | assign_stmt
                | aug_assign_stmt
                | return_stmt
                | raise_stmt
                | break_stmt
                | continue_stmt
                | pass_stmt
                | import_stmt
                | export_stmt
                | assert_stmt

compound_stmt   = if_stmt
                | for_stmt
                | while_stmt
                | match_stmt
                | try_stmt
                | with_stmt
                | fn_def
                | class_def
                | agent_def
                | tool_def
                | memory_def
                | prompt_def
                | async_block
```

### 2.2 Expressions

```ebnf
expr            = conditional_expr

conditional_expr = or_expr ["if" or_expr "else" conditional_expr]

or_expr         = and_expr ("or" and_expr)*
and_expr        = not_expr ("and" not_expr)*
not_expr        = "not" not_expr | comparison
comparison      = bitor_expr (comp_op bitor_expr)*
comp_op         = "<" | ">" | "==" | ">=" | "<=" | "!=" | "in" | "not" "in" | "is" | "is" "not"

bitor_expr      = xor_expr ("|" xor_expr)*
xor_expr        = bitand_expr ("^" bitand_expr)*
bitand_expr     = shift_expr ("&" shift_expr)*
shift_expr      = arith_expr (("<<" | ">>") arith_expr)*
arith_expr      = term (("+" | "-") term)*
term            = factor (("*" | "/" | "//" | "%" | "@") factor)*
factor          = ("+" | "-" | "~") factor | power
power           = await_expr ["**" factor]
await_expr      = ["await"] primary_expr

primary_expr    = atom trailer*
trailer         = "(" [arguments] ")"
                | "[" subscript "]"
                | "." identifier
                
atom            = identifier
                | literal
                | "(" [expr_list] ")"
                | "[" [list_items] "]"
                | "{" [dict_items] "}"
                | "{" set_items "}"
                | lambda_expr
                | struct_expr
```

### 2.3 Function Definition

```ebnf
fn_def          = [decorators] ["async"] "fn" identifier "(" [params] ")" ["->" type_expr] ":" block

decorators      = ("@" dotted_name ["(" [arguments] ")"] NEWLINE)+

params          = param ("," param)* ["," "/" ["," param ("," param)*]] ["," "*" [param]] ["," kwparam ("," kwparam)*] ["," "**" param]
                | "*" [param] ["," kwparam ("," kwparam)*] ["," "**" param]
                | "**" param

param           = identifier [":" type_expr] ["=" expr]
kwparam         = identifier [":" type_expr] "=" expr

block           = NEWLINE INDENT statement+ DEDENT
                | simple_stmt
```

### 2.4 Class Definition

```ebnf
class_def       = [decorators] "class" identifier ["(" [class_args] ")"] ":" class_block

class_args      = expr ("," expr)*

class_block     = NEWLINE INDENT class_member+ DEDENT

class_member    = fn_def
                | field_def
                | class_def
                | pass_stmt

field_def       = identifier ":" type_expr ["=" expr]
```

### 2.5 Agent Definition (AI Primitive)

```ebnf
agent_def       = [decorators] "agent" identifier ["(" [agent_config] ")"] ":" agent_block

agent_config    = agent_param ("," agent_param)*
agent_param     = identifier "=" expr

agent_block     = NEWLINE INDENT agent_member+ DEDENT

agent_member    = memory_clause
                | tools_clause
                | budget_clause
                | model_clause
                | policy_clause
                | fn_def
                | task_def
                | on_event_def

memory_clause   = "memory" ":" memory_spec
memory_spec     = identifier ("," identifier)*
                | "[" memory_config ("," memory_config)* "]"
memory_config   = identifier ["(" [arguments] ")"]

tools_clause    = "tools" ":" "[" tool_ref ("," tool_ref)* "]"
tool_ref        = identifier ["as" identifier]

budget_clause   = "budget" ":" budget_spec
budget_spec     = budget_item ("," budget_item)*
budget_item     = identifier "=" expr

model_clause    = "model" ":" expr

policy_clause   = "policy" ":" policy_spec
policy_spec     = identifier | "[" identifier ("," identifier)* "]"

task_def        = ["async"] "task" identifier "(" [params] ")" ["->" type_expr] ":" block

on_event_def    = "on" identifier "(" [params] ")" ":" block
```

### 2.6 Tool Definition (AI Primitive)

```ebnf
tool_def        = [decorators] "tool" identifier ":" tool_block

tool_block      = NEWLINE INDENT tool_member+ DEDENT

tool_member     = permissions_clause
                | sandbox_clause
                | rate_limit_clause
                | audit_clause
                | schema_clause
                | fn_def

permissions_clause = "permissions" ":" "[" identifier ("," identifier)* "]"
sandbox_clause     = "sandbox" ":" identifier
rate_limit_clause  = "rate_limit" ":" expr
audit_clause       = "audit" ":" identifier
schema_clause      = "schema" ":" type_expr
```

### 2.7 Memory Definition (AI Primitive)

```ebnf
memory_def      = "memory" identifier ":" memory_block

memory_block    = NEWLINE INDENT memory_member+ DEDENT

memory_member   = type_clause
                | capacity_clause
                | retention_clause
                | compression_clause
                | index_clause
                | retrieval_clause

type_clause         = "type" ":" identifier
capacity_clause     = "capacity" ":" expr
retention_clause    = "retention" ":" expr
compression_clause  = "compression" ":" identifier
index_clause        = "index" ":" identifier
retrieval_clause    = "retrieval" ":" identifier
```

### 2.8 Prompt Definition (AI Primitive)

```ebnf
prompt_def      = "prompt" identifier ["(" [params] ")"] ":" prompt_block

prompt_block    = NEWLINE INDENT prompt_content DEDENT

prompt_content  = prompt_string
                | prompt_section+

prompt_section  = section_header ":" (prompt_string | block)
section_header  = "system" | "user" | "assistant" | "context" | identifier
```

### 2.9 Control Flow

```ebnf
if_stmt         = "if" expr ":" block ("elif" expr ":" block)* ["else" ":" block]

for_stmt        = "for" target_list "in" expr_list ":" block ["else" ":" block]

while_stmt      = "while" expr ":" block ["else" ":" block]

match_stmt      = "match" expr ":" NEWLINE INDENT case_clause+ DEDENT
case_clause     = "case" pattern ["if" expr] ":" block

try_stmt        = "try" ":" block except_clause+ ["else" ":" block] ["finally" ":" block]
                | "try" ":" block "finally" ":" block
except_clause   = "except" [expr ["as" identifier]] ":" block

with_stmt       = "with" with_item ("," with_item)* ":" block
with_item       = expr ["as" identifier]
```

### 2.10 Async and Concurrency

```ebnf
async_block     = "async" ":" async_body

async_body      = NEWLINE INDENT async_stmt+ DEDENT

async_stmt      = spawn_stmt
                | await_expr_stmt
                | parallel_block
                | statement

spawn_stmt      = "spawn" expr ["as" identifier]

parallel_block  = "parallel" ":" NEWLINE INDENT statement+ DEDENT
```

---

## 3. Types and Type System

### 3.1 Type Philosophy
A16 uses **gradual typing**: type hints are optional but enable optimization and safety when provided. The runtime can enforce types in strict mode.

### 3.2 Built-in Types

| Type | Description | Literal Example |
|------|-------------|-----------------|
| `Int` | Arbitrary precision integer | `42`, `0xFF` |
| `Float` | 64-bit floating point | `3.14`, `1e-10` |
| `Bool` | Boolean | `True`, `False` |
| `Str` | Unicode string | `"hello"` |
| `Bytes` | Byte sequence | `b"data"` |
| `None` | Null type | `None` |
| `List[T]` | Mutable list | `[1, 2, 3]` |
| `Dict[K, V]` | Dictionary | `{"a": 1}` |
| `Set[T]` | Mutable set | `{1, 2, 3}` |
| `Tuple[T...]` | Immutable tuple | `(1, "a", True)` |
| `Optional[T]` | T or None | - |
| `Union[T1, T2]` | Either type | - |
| `Any` | Dynamic type | - |

### 3.3 AI-Specific Types

| Type | Description |
|------|-------------|
| `Agent` | Agent instance |
| `Tool` | Tool definition |
| `Memory` | Memory store |
| `Prompt` | Prompt template |
| `Message` | Chat message |
| `Context` | Execution context |
| `TokenBudget` | Token allocation |
| `ModelResponse` | Model output |
| `Embedding` | Vector embedding |
| `Schema[T]` | Structured output schema |

### 3.4 Type Expressions

```ebnf
type_expr       = simple_type
                | generic_type
                | union_type
                | callable_type
                | schema_type

simple_type     = identifier

generic_type    = identifier "[" type_args "]"
type_args       = type_expr ("," type_expr)*

union_type      = type_expr "|" type_expr

callable_type   = "(" [type_args] ")" "->" type_expr

schema_type     = "Schema" "[" struct_type "]"
struct_type     = "{" field_type ("," field_type)* "}"
field_type      = identifier ":" type_expr
```

### 3.5 Struct Types (for Structured Outputs)

```a16
struct SearchResult:
    title: Str
    url: Str
    snippet: Str
    relevance: Float

struct Summary:
    key_points: List[Str]
    conclusion: Str
    confidence: Float = 0.0
```

### 3.6 Type Inference
- Local variables: inferred from initialization
- Function returns: inferred if not annotated
- AI outputs: enforced via Schema types

---

## 4. Variables and Bindings

### 4.1 Variable Declaration

```a16
# Mutable binding (default)
let x = 42
x = 43  # OK

# Immutable binding
const PI = 3.14159
PI = 3  # Error: cannot reassign const

# Type-annotated
let name: Str = "A16"
let items: List[Int] = []
```

### 4.2 Destructuring

```a16
# Tuple destructuring
let (a, b, c) = (1, 2, 3)

# List destructuring
let [first, *rest] = [1, 2, 3, 4]

# Dict destructuring
let {name, age} = {"name": "Alice", "age": 30}

# Pattern matching in assignment
let Point(x, y) = get_point()
```

### 4.3 Scope Rules
- Block-scoped (like Rust/JS let)
- Lexical scoping with closures
- No global keyword pollution (explicit `global` required)

---

## 5. Functions and Callables

### 5.1 Function Definition

```a16
fn add(a: Int, b: Int) -> Int:
    return a + b

# With default values
fn greet(name: Str, greeting: Str = "Hello") -> Str:
    return f"{greeting}, {name}!"

# Variadic
fn sum(*numbers: Int) -> Int:
    return reduce(add, numbers, 0)

# Keyword-only
fn config(*, debug: Bool = False, verbose: Bool = False):
    pass
```

### 5.2 Lambda Expressions

```a16
# Short form
let double = (x) => x * 2

# With types
let parse: (Str) -> Int = (s) => int(s)

# Multi-line
let process = (data) =>:
    cleaned = clean(data)
    return transform(cleaned)
```

### 5.3 Decorators

```a16
@trace
@cache(ttl=3600)
fn expensive_operation(query: Str) -> Result:
    ...

# Decorator definition
fn cache(ttl: Int = 0):
    fn decorator(func):
        let _cache = {}
        fn wrapper(*args, **kwargs):
            key = hash((args, kwargs))
            if key in _cache and not expired(_cache[key], ttl):
                return _cache[key].value
            result = func(*args, **kwargs)
            _cache[key] = CacheEntry(result, now())
            return result
        return wrapper
    return decorator
```

---

## 6. Classes and Objects

### 6.1 Class Definition

```a16
class Point:
    x: Float
    y: Float
    
    fn __init__(self, x: Float, y: Float):
        self.x = x
        self.y = y
    
    fn distance(self, other: Point) -> Float:
        return sqrt((self.x - other.x)**2 + (self.y - other.y)**2)
    
    fn __repr__(self) -> Str:
        return f"Point({self.x}, {self.y})"
```

### 6.2 Inheritance

```a16
class Shape:
    fn area(self) -> Float:
        raise NotImplementedError()

class Circle(Shape):
    radius: Float
    
    fn __init__(self, radius: Float):
        self.radius = radius
    
    fn area(self) -> Float:
        return PI * self.radius ** 2
```

### 6.3 Protocols (Structural Typing)

```a16
protocol Callable:
    fn __call__(self, *args, **kwargs) -> Any

protocol Iterable[T]:
    fn __iter__(self) -> Iterator[T]

# Any class implementing these methods satisfies the protocol
```

---

## 7. AI Primitives

### 7.1 Agent Definition

```a16
agent ResearchAssistant:
    """An agent that researches topics and provides summaries."""
    
    memory: [short_term(window=10), long_term(type=vector)]
    tools: [web_search, file_read, calculator]
    model: gpt4
    budget: tokens=4000, cost=0.10
    policy: [no_code_execution, safe_domains_only]
    
    task summarize(topic: Str) -> Summary:
        """Research and summarize a topic."""
        # Retrieve relevant past context
        context = memory.retrieve(topic, k=5)
        
        # Search for new information
        results = await web_search(topic, count=10)
        
        # Generate structured summary
        return model.generate(
            prompt=prompts.summarize,
            context=context + results,
            output_schema=Summary
        )
    
    on message(msg: Message):
        """Handle incoming messages."""
        memory.store(msg)
        response = await self.process(msg.content)
        return response
```

### 7.2 Tool Definition

```a16
tool web_search:
    """Search the web for information."""
    
    permissions: [network]
    sandbox: isolated
    rate_limit: 60/minute
    audit: full
    
    schema:
        query: Str
        count: Int = 10
        domains: Optional[List[Str]] = None
    
    fn execute(query: Str, count: Int, domains: Optional[List[Str]]) -> List[SearchResult]:
        # Implementation
        url = build_search_url(query, count, domains)
        response = http.get(url)
        return parse_results(response)
```

### 7.3 Memory Definition

```a16
memory ConversationMemory:
    type: episodic
    capacity: 1000
    retention: 30d
    compression: summarize
    index: semantic
    
    fn store(item: Any) -> Id:
        embedded = embed(item)
        return self._store(embedded)
    
    fn retrieve(query: Any, k: Int = 5) -> List[Any]:
        embedded = embed(query)
        return self._search(embedded, k)
```

### 7.4 Prompt Definition

```a16
prompt summarize(topic: Str, context: List[Str]):
    system:
        p"""
        You are a research assistant. Provide accurate, well-structured summaries.
        Always cite sources. Be concise but comprehensive.
        """
    
    context:
        p"""
        Previous relevant information:
        {% for item in context %}
        - {{ item }}
        {% endfor %}
        """
    
    user:
        p"""
        Please research and summarize: {{ topic }}
        
        Provide:
        1. Key points (3-5 bullet points)
        2. A brief conclusion
        3. Confidence level (0-1)
        """
```

### 7.5 Structured Output

```a16
struct TaskPlan:
    goal: Str
    steps: List[Step]
    estimated_tokens: Int
    confidence: Float

struct Step:
    action: Str
    tool: Optional[Str]
    expected_output: Str

# Usage
let plan = model.generate(
    prompt=planning_prompt,
    output_schema=TaskPlan
)
# plan is guaranteed to match TaskPlan structure
```

### 7.6 Token Budgeting

```a16
# Block-level budget
with token_budget(2000) as budget:
    result = agent.run(task)
    print(f"Used {budget.used} of {budget.limit} tokens")

# Agent-level budget (in agent definition)
agent BudgetedAgent:
    budget: tokens=10000, cost=1.00
    
    # Operations that exceed budget raise BudgetExceeded

# Automatic compression when approaching limit
with token_budget(1000, compress=True):
    # Long context automatically compressed
    result = model.generate(prompt, context=large_context)
```

---

## 8. Modules and Imports

### 8.1 Import Syntax

```a16
# Import module
import a16.ai.agent

# Import with alias
import a16.ai.agent as ag

# Import specific items
from a16.ai.agent import Agent, Task

# Import all (discouraged)
from a16.ai.tools import *

# Relative imports
from .utils import helper
from ..common import shared
```

### 8.2 Module Definition

```a16
# my_module.a16

### 
My Module
=========
A description of what this module does.
###

# Module-level constants
const VERSION = "1.0.0"

# Private function (underscore prefix)
fn _internal_helper():
    pass

# Public function
fn public_api():
    pass

# Explicit exports
export [public_api, VERSION]
```

### 8.3 Package Structure

```
my_package/
├── __init__.a16      # Package initialization
├── core.a16          # Core module
├── utils/
│   ├── __init__.a16
│   └── helpers.a16
└── a16.toml          # Package manifest
```

---

## 9. Error Handling

### 9.1 Exception Hierarchy

```
BaseException
├── SystemExit
├── KeyboardInterrupt
└── Exception
    ├── ValueError
    ├── TypeError
    ├── RuntimeError
    ├── IOError
    ├── AIError
    │   ├── BudgetExceeded
    │   ├── ModelError
    │   ├── ToolError
    │   ├── MemoryError
    │   └── PolicyViolation
    └── ...
```

### 9.2 Try/Except

```a16
try:
    result = agent.run(task)
except BudgetExceeded as e:
    log.warning(f"Budget exceeded: {e.used}/{e.limit} tokens")
    result = fallback_result()
except ModelError as e:
    log.error(f"Model failed: {e}")
    raise
except Exception as e:
    log.error(f"Unexpected error: {e}")
    result = None
finally:
    cleanup()
```

### 9.3 Result Type (Rust-style)

```a16
from a16.core import Result, Ok, Err

fn divide(a: Float, b: Float) -> Result[Float, Str]:
    if b == 0:
        return Err("Division by zero")
    return Ok(a / b)

# Pattern matching on result
match divide(10, 2):
    case Ok(value):
        print(f"Result: {value}")
    case Err(msg):
        print(f"Error: {msg}")
```

---

## 10. Concurrency Model

### 10.1 Async/Await

```a16
async fn fetch_data(url: Str) -> Data:
    response = await http.get(url)
    return parse(response)

async fn main():
    data = await fetch_data("https://api.example.com")
    print(data)
```

### 10.2 Parallel Execution

```a16
# Parallel block - all statements run concurrently
async:
    parallel:
        result1 = fetch_data(url1)
        result2 = fetch_data(url2)
        result3 = fetch_data(url3)
    
    # Continues when all complete
    combined = merge(result1, result2, result3)
```

### 10.3 Task Spawning

```a16
async fn background_worker():
    while True:
        task = await queue.get()
        await process(task)

async fn main():
    # Spawn background task
    worker_handle = spawn background_worker()
    
    # Continue with main logic
    await do_work()
    
    # Cancel if needed
    worker_handle.cancel()
```

### 10.4 Channels

```a16
from a16.sys.concurrent import Channel

async fn producer(ch: Channel[Int]):
    for i in range(10):
        await ch.send(i)
    ch.close()

async fn consumer(ch: Channel[Int]):
    async for item in ch:
        print(item)

async fn main():
    let ch = Channel[Int](buffer=10)
    spawn producer(ch)
    await consumer(ch)
```

### 10.5 Agent Concurrency

```a16
# Multi-agent team with message passing
team ResearchTeam:
    agents: [Researcher, Writer, Critic]
    topology: pipeline  # or: mesh, star, custom
    
    async fn run(topic: Str) -> Report:
        # Agents communicate via internal message bus
        research = await Researcher.research(topic)
        draft = await Writer.write(research)
        feedback = await Critic.review(draft)
        final = await Writer.revise(draft, feedback)
        return final
```

---

## 11. Memory Model

### 11.1 Value Semantics
- Primitives (Int, Float, Bool, Str): copied on assignment
- Collections (List, Dict, Set): reference semantics
- Structs: value semantics by default, reference with `ref`

### 11.2 Ownership (Optional Mode)

```a16
# In strict mode, ownership is tracked
fn process(data: owned List[Int]) -> List[Int]:
    # data is moved into this function
    return transform(data)

fn main():
    let items = [1, 2, 3]
    let result = process(items)
    # items is no longer valid here (moved)
```

### 11.3 AI Memory Model

```a16
# Memory types
memory short_term:
    type: sliding_window
    capacity: 10
    # Automatically evicts oldest

memory long_term:
    type: vector
    index: hnsw
    capacity: 1_000_000
    # Semantic retrieval

memory episodic:
    type: compressed
    compression: summarize
    retention: 90d
    # Auto-summarizes old entries

# Memory operations
memory.store(data)                    # Store
memory.retrieve(query, k=5)           # Retrieve k nearest
memory.forget(predicate)              # Selective deletion
memory.compress()                     # Manual compression
memory.export() -> Bytes              # Serialize
memory.import(data: Bytes)            # Deserialize
```

---

## 12. Execution Modes

### 12.1 Standard Mode
Default execution with all features enabled.

```bash
a16 run script.a16
```

### 12.2 Deterministic Mode
For reproducible execution (testing, debugging).

```bash
a16 run --deterministic --seed=42 script.a16
```

- Fixed random seeds
- Deterministic model responses (via caching or mock)
- No external I/O unless explicitly allowed

### 12.3 Safe Mode
Maximum security for untrusted code or tools.

```bash
a16 run --safe script.a16
```

- All tools sandboxed
- Network disabled by default
- Filesystem restricted to working directory
- CPU/memory limits enforced

### 12.4 Debug Mode
Enhanced observability.

```bash
a16 run --debug script.a16
```

- Full tracing enabled
- Token usage logged
- Step-through execution available
- Memory snapshots at each step

### 12.5 Profile Mode
Performance analysis.

```bash
a16 run --profile script.a16
```

- Timing for all operations
- Token usage breakdown
- Memory allocation tracking
- Bottleneck identification

---

## Appendix A: Operator Precedence

| Precedence | Operators | Associativity |
|------------|-----------|---------------|
| 1 (highest) | `**` | Right |
| 2 | `+x`, `-x`, `~x` | Unary |
| 3 | `*`, `/`, `//`, `%`, `@` | Left |
| 4 | `+`, `-` | Left |
| 5 | `<<`, `>>` | Left |
| 6 | `&` | Left |
| 7 | `^` | Left |
| 8 | `\|` | Left |
| 9 | `<`, `<=`, `>`, `>=`, `!=`, `==` | Chained |
| 10 | `not` | Unary |
| 11 | `and` | Left |
| 12 | `or` | Left |
| 13 | `if`-`else` | Ternary |
| 14 | `=>` | Right |
| 15 (lowest) | `,` | Left |

---

## Appendix B: Reserved for Future

The following are reserved for future language versions:
- `macro`, `quote`, `unquote` (metaprogramming)
- `effect`, `handle` (algebraic effects)
- `linear`, `affine` (linear types)
- `gpu`, `tensor` (hardware acceleration)
- `verify`, `prove` (formal verification)

---

*Next: [Runtime + VM/JIT Architecture](./03-runtime-architecture.md)*
