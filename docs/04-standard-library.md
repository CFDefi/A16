# A16 Standard Library Design

> **Version:** 0.1 | **Status:** Draft | **Date:** 2026-01-19

---

## Module Overview

```
a16/
├── ai/                    # AI-First Modules
│   ├── agent.a16          # Agent definitions and execution
│   ├── model.a16          # Model adapters and invocation
│   ├── memory.a16         # Memory systems
│   ├── tools.a16          # Tool registry and execution
│   ├── retrieval.a16      # RAG primitives
│   ├── prompt.a16         # Prompt management
│   └── eval.a16           # Evaluation and benchmarks
│
├── sys/                   # System Modules
│   ├── concurrent.a16     # Async runtime and concurrency
│   ├── io.a16             # File, network, subprocess
│   ├── time.a16           # Time and scheduling
│   └── os.a16             # OS interaction
│
├── core/                  # Core Types and Utilities
│   ├── types.a16          # Built-in type extensions
│   ├── collections.a16    # Advanced collections
│   ├── result.a16         # Result/Option types
│   ├── iter.a16           # Iterators
│   └── math.a16           # Math utilities
│
├── data/                  # Data Processing
│   ├── json.a16           # JSON handling
│   ├── yaml.a16           # YAML handling
│   ├── csv.a16            # CSV handling
│   └── schema.a16         # Schema validation
│
├── http/                  # HTTP Client/Server
│   ├── client.a16         # HTTP client
│   └── server.a16         # HTTP server
│
└── pkg/                   # Package Management
    ├── manager.a16        # Package manager
    └── registry.a16       # Registry client
```

---

## 1. a16.ai.agent

### 1.1 Core Types

```a16
from a16.ai.agent import Agent, Task, Role, Team, Message

### Agent is the fundamental AI execution unit ###
class Agent:
    """Base class for all agents."""
    
    # Configuration
    name: Str
    model: Model
    memory: List[Memory]
    tools: List[Tool]
    budget: Budget
    policy: Policy
    
    # Lifecycle
    fn __init__(self, **config)
    async fn start(self) -> None
    async fn stop(self) -> None
    
    # Execution
    async fn run(self, task: Task) -> Any
    async fn chat(self, message: Message) -> Message
    async fn step(self) -> StepResult
    
    # Introspection
    fn state(self) -> AgentState
    fn trace(self) -> Trace
    fn metrics(self) -> Metrics

class Task:
    """A unit of work for an agent."""
    id: Str
    description: Str
    inputs: Dict[Str, Any]
    expected_output: Optional[Type]
    timeout: Optional[Duration]
    budget: Optional[Budget]

class Role:
    """Defines agent behavior and constraints."""
    name: Str
    system_prompt: Prompt
    allowed_tools: List[Str]
    policies: List[Policy]

class Team:
    """A coordinated group of agents."""
    agents: List[Agent]
    topology: Topology  # pipeline, mesh, star, custom
    coordinator: Optional[Agent]
    
    async fn run(self, task: Task) -> Any
    async fn broadcast(self, message: Message) -> List[Message]
```

### 1.2 Agent Decorators

```a16
# Simple agent
@agent
class SimpleAssistant:
    model: gpt4
    
    task answer(question: Str) -> Str:
        return model.generate(question)

# Configured agent
@agent(
    model=claude3,
    memory=[short_term(10), long_term()],
    budget=Budget(tokens=10000, cost=1.0)
)
class ResearchAgent:
    tools: [web_search, file_read]
    
    task research(topic: Str) -> Report:
        ...

# Role-based agent
@agent(role=roles.Analyst)
class DataAnalyst:
    ...
```

### 1.3 Examples

```a16
from a16.ai.agent import Agent, Task
from a16.ai.model import gpt4
from a16.ai.memory import ShortTermMemory
from a16.ai.tools import web_search

# Define agent
agent researcher = Agent(
    name="Researcher",
    model=gpt4,
    memory=[ShortTermMemory(window=10)],
    tools=[web_search],
    budget=Budget(tokens=5000)
)

# Run task
async fn main():
    task = Task(
        description="Research recent AI developments",
        expected_output=Summary
    )
    result = await researcher.run(task)
    print(result)
```

---

## 2. a16.ai.model

### 2.1 Core Types

```a16
from a16.ai.model import Model, ModelConfig, Response, Message

class Model:
    """Abstract base for all model adapters."""
    
    # Invocation
    async fn generate(self, prompt: Prompt, **kwargs) -> Response
    async fn stream(self, prompt: Prompt, **kwargs) -> AsyncIterator[Chunk]
    async fn structured(self, prompt: Prompt, schema: Type[T], **kwargs) -> T
    
    # Tool use
    async fn with_tools(self, tools: List[Tool]) -> Self
    async fn invoke_with_tools(self, prompt: Prompt) -> ToolCallResponse
    
    # Batch
    async fn batch(self, prompts: List[Prompt]) -> List[Response]
    
    # Info
    fn context_length(self) -> Int
    fn token_price(self) -> TokenPrice

class Response:
    content: Str
    tokens_in: Int
    tokens_out: Int
    finish_reason: FinishReason
    model: Str
    latency_ms: Int
    tool_calls: Optional[List[ToolCall]]

# Built-in model references
const gpt4 = OpenAIModel("gpt-4o")
const gpt4_mini = OpenAIModel("gpt-4o-mini")
const claude3 = AnthropicModel("claude-3-5-sonnet")
const claude3_haiku = AnthropicModel("claude-3-5-haiku")
const llama3 = OllamaModel("llama3.1:70b")
```

### 2.2 Structured Outputs

```a16
from a16.ai.model import gpt4

struct SearchQuery:
    query: Str
    filters: List[Str]
    max_results: Int = 10

struct AnalysisResult:
    summary: Str
    key_points: List[Str]
    confidence: Float
    sources: List[Str]

# Guaranteed to match schema
let query: SearchQuery = await gpt4.structured(
    prompt=p"Extract a search query from: {user_input}",
    schema=SearchQuery
)

let analysis: AnalysisResult = await gpt4.structured(
    prompt=p"Analyze the following data: {data}",
    schema=AnalysisResult
)
```

### 2.3 Streaming

```a16
from a16.ai.model import gpt4

async fn stream_response(prompt: Str):
    async for chunk in gpt4.stream(prompt):
        print(chunk.text, end="", flush=True)
        
        # Access partial structured data
        if chunk.partial_json:
            update_ui(chunk.partial_json)
    
    print()  # Final newline
```

---

## 3. a16.ai.memory

### 3.1 Memory Types

```a16
from a16.ai.memory import (
    Memory, ShortTermMemory, LongTermMemory, 
    EpisodicMemory, SemanticMemory, WorkingMemory
)

class Memory:
    """Base class for all memory types."""
    
    async fn store(self, content: Any, metadata: Dict = {}) -> Id
    async fn retrieve(self, query: Any, k: Int = 5) -> List[MemoryItem]
    async fn update(self, id: Id, content: Any) -> None
    async fn forget(self, filter: Dict) -> Int
    async fn compress(self, strategy: Str = "summarize") -> None
    
    fn export(self) -> Bytes
    fn import_(self, data: Bytes) -> None

class ShortTermMemory(Memory):
    """Sliding window memory (last N items)."""
    window: Int = 10
    
class LongTermMemory(Memory):
    """Vector-indexed semantic memory."""
    index_type: Str = "hnsw"
    dimensions: Int = 1536
    
class EpisodicMemory(Memory):
    """Event-based memory with temporal structure."""
    retention: Duration = 90d
    compression: Str = "summarize"

class SemanticMemory(Memory):
    """Fact and knowledge storage."""
    dedup: Bool = True
    
class WorkingMemory(Memory):
    """Fast, temporary computation memory."""
    max_size: Int = 1000
```

### 3.2 Memory Composition

```a16
# Combine multiple memory types
memory composite = CompositeMemory([
    ShortTermMemory(window=20),        # Recent interactions
    LongTermMemory(dimensions=1536),   # Persistent knowledge
    EpisodicMemory(retention=30d)      # Summarized history
])

# Retrieve from all, ranked by relevance
results = await composite.retrieve("user preferences", k=10)
```

### 3.3 Examples

```a16
from a16.ai.memory import LongTermMemory

# Create memory
memory = LongTermMemory()

# Store with metadata
await memory.store(
    content="User prefers concise answers with examples",
    metadata={"type": "preference", "confidence": 0.9}
)

# Retrieve
results = await memory.retrieve(
    query="What format does the user like?",
    k=5
)

for item in results:
    print(f"[{item.score:.2f}] {item.content}")

# Forget old entries
deleted = await memory.forget({"older_than": "90d"})
print(f"Cleaned up {deleted} old memories")
```

---

## 4. a16.ai.tools

### 4.1 Tool Definition

```a16
from a16.ai.tools import Tool, tool, Permission, Sandbox

@tool(
    name="web_search",
    description="Search the web for information",
    permissions=[Permission.Network],
    sandbox=Sandbox.Light,
    rate_limit=60/minute
)
async fn web_search(
    query: Str,
    num_results: Int = 10,
    domains: Optional[List[Str]] = None
) -> List[SearchResult]:
    """
    Search the web and return relevant results.
    
    Args:
        query: Search query string
        num_results: Maximum results to return
        domains: Optional domain filter
    
    Returns:
        List of search results with title, url, and snippet
    """
    # Implementation
    url = build_search_url(query, num_results, domains)
    response = await http.get(url)
    return parse_results(response)
```

### 4.2 Tool Registry

```a16
from a16.ai.tools import ToolRegistry

# Global registry
registry = ToolRegistry()

# Register tools
registry.register(web_search)
registry.register(file_read)
registry.register(calculator)

# Get tool by name
tool = registry.get("web_search")

# List all tools
for t in registry.list():
    print(f"{t.name}: {t.description}")

# Generate schema for LLM
schema = registry.openai_schema()
```

### 4.3 Built-in Tools

```a16
from a16.ai.tools.builtin import (
    # Web
    web_search, web_fetch, web_scrape,
    
    # Files
    file_read, file_write, file_list,
    
    # Code
    code_execute, code_analyze,
    
    # Data
    json_parse, json_query, csv_parse,
    
    # Math
    calculator, wolfram_alpha,
    
    # Utility
    datetime_now, uuid_generate
)
```

---

## 5. a16.ai.retrieval

### 5.1 RAG Primitives

```a16
from a16.ai.retrieval import (
    Document, Chunk, Embedder, Index, Retriever
)

class Document:
    id: Str
    content: Str
    metadata: Dict

class Chunk:
    id: Str
    document_id: Str
    content: Str
    embedding: Optional[List[Float]]
    start_idx: Int
    end_idx: Int

class Embedder:
    """Generate embeddings for text."""
    async fn embed(self, text: Str) -> List[Float]
    async fn embed_batch(self, texts: List[Str]) -> List[List[Float]]

class Index:
    """Vector index for similarity search."""
    async fn add(self, id: Str, embedding: List[Float], metadata: Dict = {})
    async fn search(self, query: List[Float], k: Int = 10) -> List[Match]
    async fn delete(self, id: Str)

class Retriever:
    """High-level retrieval pipeline."""
    embedder: Embedder
    index: Index
    reranker: Optional[Reranker]
    
    async fn retrieve(self, query: Str, k: Int = 10) -> List[Document]
```

### 5.2 Chunking Strategies

```a16
from a16.ai.retrieval import (
    chunk_by_tokens, chunk_by_sentences, chunk_by_paragraphs,
    chunk_recursive, chunk_semantic
)

# Token-based (fixed size)
chunks = chunk_by_tokens(document, size=512, overlap=50)

# Sentence-based (natural boundaries)
chunks = chunk_by_sentences(document, max_sentences=5)

# Recursive (hierarchical)
chunks = chunk_recursive(document, max_size=1000, separators=["\n\n", "\n", ". "])

# Semantic (embedding-based boundaries)
chunks = await chunk_semantic(document, embedder, threshold=0.8)
```

### 5.3 Example RAG Pipeline

```a16
from a16.ai.retrieval import Retriever, Index
from a16.ai.model import gpt4, text_embedding_3_small

# Setup
embedder = text_embedding_3_small
index = Index.create("hnsw", dimensions=1536)
retriever = Retriever(embedder=embedder, index=index)

# Ingest documents
async fn ingest(documents: List[Document]):
    for doc in documents:
        chunks = chunk_recursive(doc, max_size=500)
        for chunk in chunks:
            embedding = await embedder.embed(chunk.content)
            await index.add(chunk.id, embedding, {"doc_id": doc.id})

# Query with RAG
async fn query_with_rag(question: Str) -> Str:
    # Retrieve relevant chunks
    chunks = await retriever.retrieve(question, k=5)
    
    # Build context
    context = "\n\n".join(c.content for c in chunks)
    
    # Generate answer
    response = await gpt4.generate(
        prompt=p"""
        Context:
        {context}
        
        Question: {question}
        
        Answer based on the context above:
        """
    )
    return response.content
```

---

## 6. a16.ai.eval

### 6.1 Evaluation Framework

```a16
from a16.ai.eval import (
    Evaluator, Metric, TestCase, TestSuite, BenchmarkResult
)

class Evaluator:
    """Run evaluations on agents or models."""
    
    async fn evaluate(
        self, 
        target: Agent | Model,
        test_suite: TestSuite
    ) -> EvalResult
    
    async fn compare(
        self,
        targets: List[Agent | Model],
        test_suite: TestSuite
    ) -> ComparisonResult

class TestCase:
    name: Str
    input: Any
    expected: Any
    metrics: List[Metric]
    timeout: Duration = 30s

class TestSuite:
    name: Str
    cases: List[TestCase]
    
    @classmethod
    fn from_file(cls, path: Path) -> TestSuite
```

### 6.2 Built-in Metrics

```a16
from a16.ai.eval.metrics import (
    # Accuracy metrics
    ExactMatch,          # Exact string match
    FuzzyMatch,          # Fuzzy string similarity
    SemanticSimilarity,  # Embedding similarity
    ContainsAll,         # Contains required substrings
    
    # Quality metrics
    Coherence,           # Logical flow
    Relevance,           # Answers the question
    Factuality,          # Grounded in context
    
    # Safety metrics
    Toxicity,            # Harmful content detection
    Bias,                # Bias detection
    
    # Performance metrics
    TokenCount,          # Token usage
    Latency,             # Response time
    Cost                 # API cost
)

# Use in test case
test = TestCase(
    name="summarization_quality",
    input="Summarize this article: ...",
    expected="...",
    metrics=[
        SemanticSimilarity(threshold=0.8),
        Coherence(min_score=0.7),
        TokenCount(max=500)
    ]
)
```

### 6.3 Example Evaluation

```a16
from a16.ai.eval import Evaluator, TestSuite
from a16.ai.agent import my_agent

# Load test suite
suite = TestSuite.from_file("tests/qa_benchmark.yaml")

# Run evaluation
evaluator = Evaluator()
results = await evaluator.evaluate(my_agent, suite)

# Report
print(f"Pass rate: {results.pass_rate:.1%}")
print(f"Avg latency: {results.avg_latency_ms}ms")
print(f"Total tokens: {results.total_tokens}")

for case in results.failed:
    print(f"FAILED: {case.name}")
    print(f"  Expected: {case.expected}")
    print(f"  Got: {case.actual}")
```

---

## 7. a16.sys.concurrent

### 7.1 Async Primitives

```a16
from a16.sys.concurrent import (
    spawn, sleep, timeout, select,
    Channel, Mutex, Semaphore, Barrier
)

# Spawn task
async fn background_work():
    while True:
        await do_something()
        await sleep(1s)

handle = spawn background_work()

# Timeout
try:
    result = await timeout(5s, long_operation())
except TimeoutError:
    print("Operation timed out")

# Select (first to complete)
match await select(
    fetch_from_cache(),
    fetch_from_db(),
    sleep(1s)
):
    case (0, result):
        print(f"Cache hit: {result}")
    case (1, result):
        print(f"DB result: {result}")
    case (2, _):
        print("Timeout")
```

### 7.2 Channels

```a16
from a16.sys.concurrent import Channel

# Create channel
let ch: Channel[Message] = Channel(buffer=10)

# Producer
async fn producer():
    for i in range(100):
        await ch.send(Message(i))
    ch.close()

# Consumer
async fn consumer():
    async for msg in ch:
        process(msg)

# Fan-out
async fn fan_out(source: Channel, workers: Int) -> List[Channel]:
    outputs = [Channel() for _ in range(workers)]
    
    async fn distribute():
        i = 0
        async for item in source:
            await outputs[i % workers].send(item)
            i += 1
        for out in outputs:
            out.close()
    
    spawn distribute()
    return outputs
```

### 7.3 Parallel Utilities

```a16
from a16.sys.concurrent import parallel_map, parallel_filter, gather

# Parallel map
results = await parallel_map(urls, fetch, max_concurrency=10)

# Parallel filter
valid = await parallel_filter(items, async (x) => await is_valid(x))

# Gather with error handling
results = await gather(
    task1(),
    task2(),
    task3(),
    return_exceptions=True
)
```

---

## 8. a16.sys.io

### 8.1 File Operations

```a16
from a16.sys.io import File, Path

# Read
content = await File.read("data.txt")
lines = await File.read_lines("data.txt")
data = await File.read_json("config.json")

# Write
await File.write("output.txt", content)
await File.write_json("data.json", obj)
await File.append("log.txt", line + "\n")

# Path operations
path = Path("project/src/main.a16")
print(path.name)       # "main.a16"
print(path.stem)       # "main"
print(path.suffix)     # ".a16"
print(path.parent)     # "project/src"
print(path.exists())   # True/False

# Glob
for file in Path("src").glob("**/*.a16"):
    print(file)
```

### 8.2 Network

```a16
from a16.http import Client, Request, Response

# Simple requests
response = await Client.get("https://api.example.com/data")
response = await Client.post("https://api.example.com/submit", json=data)

# Configured client
client = Client(
    base_url="https://api.example.com",
    headers={"Authorization": f"Bearer {token}"},
    timeout=30s,
    retries=3
)

response = await client.get("/users")
users = response.json()
```

---

## 9. a16.data.json / a16.data.schema

### 9.1 JSON Operations

```a16
from a16.data.json import parse, stringify, query

# Parse
data = parse('{"name": "A16", "version": 1}')

# Stringify
json_str = stringify(data, indent=2)

# Query (JSONPath)
values = query(data, "$.users[*].name")
```

### 9.2 Schema Validation

```a16
from a16.data.schema import validate, Schema

schema = Schema({
    "type": "object",
    "properties": {
        "name": {"type": "string"},
        "age": {"type": "integer", "minimum": 0}
    },
    "required": ["name"]
})

# Validate
result = validate(data, schema)
if not result.valid:
    for error in result.errors:
        print(f"Validation error: {error}")
```

---

## 10. a16.pkg

### 10.1 Package Manager API

```a16
from a16.pkg import Manager, Package, Registry

manager = Manager()

# Install
await manager.install("a16-openai", version="^1.0.0")

# Uninstall
await manager.uninstall("a16-openai")

# List installed
for pkg in manager.list():
    print(f"{pkg.name}@{pkg.version}")

# Search registry
results = await Registry.search("openai")
for pkg in results:
    print(f"{pkg.name}: {pkg.description}")
```

---

## Module Import Examples

```a16
# Standard imports
from a16.ai.agent import Agent
from a16.ai.model import gpt4
from a16.ai.memory import LongTermMemory
from a16.ai.tools import web_search
from a16.sys.concurrent import spawn, Channel
from a16.data.json import parse, stringify

# Wildcard (discouraged but supported)
from a16.ai.tools.builtin import *

# Aliased
import a16.ai.agent as ag
import a16.ai.model as mdl
```

---

*Next: [File/Package Layout](./05-package-layout.md)*
