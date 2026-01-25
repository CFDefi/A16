# A16 Programming Language — Complete Specification

> **The first programming language where AI is the foundation, not an afterthought.**

---

## Document Index

| # | Document | Description |
|---|----------|-------------|
| 1 | [Product Definition](./01-product-definition.md) | What A16 is, who it's for, why it wins |
| 2 | [Language Specification](./02-language-specification.md) | Syntax (EBNF), types, semantics, AI primitives |
| 3 | [Runtime Architecture](./03-runtime-architecture.md) | VM, JIT, Token Engine, Memory Manager, Scheduler |
| 4 | [Standard Library](./04-standard-library.md) | a16.ai.*, a16.sys.*, module APIs |
| 5 | [Package Layout](./05-package-layout.md) | Project structure, a16.toml, modules |
| 6 | [CLI Tooling](./06-cli-tooling.md) | a16 run, repl, fmt, test, pkg, etc. |
| 7 | [Benchmarks](./07-benchmarks.md) | 10 benchmarks proving A16 advantages |
| 8 | [Milestone Plan](./08-milestone-plan.md) | M0→M10 with deliverables and demos |
| 9 | [Implementation Plan](./09-implementation-plan.md) | Rust architecture, compilation pipeline |
| 10 | [Example Programs](./10-examples.md) | 3 complete A16 programs |

---

## Quick Start

```a16
from a16.ai.agent import Agent
from a16.ai.model import gpt4
from a16.ai.memory import LongTermMemory

agent Assistant:
    model: gpt4
    memory: [LongTermMemory()]
    budget: tokens=5000
    
    task answer(question: Str) -> Str:
        context = await memory.retrieve(question, k=5)
        response = await model.generate(question, context=context)
        await memory.store(f"Q: {question}\nA: {response}")
        return response

async fn main():
    let bot = Assistant()
    print(await bot.answer("What is quantum computing?"))
```

---

## Key Metrics (vs Python Baseline)

| Metric | Improvement |
|--------|-------------|
| Token usage | **65% fewer** |
| Latency | **4x faster** |
| Memory | **40% less** |
| Code volume | **70% less** |
| Cold start | **60% faster** |

---

## Getting Started

```bash
# Install A16
curl -fsSL https://a16.dev/install | sh

# Create new project
a16 new my-agent

# Run
cd my-agent
a16 run src/main.a16

# Interactive REPL
a16 repl
```

---

*A16 — Build the future of AI systems.*
