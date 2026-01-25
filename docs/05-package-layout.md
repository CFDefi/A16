# A16 File/Package Layout

> **Version:** 0.1 | **Status:** Draft | **Date:** 2026-01-19

---

## 1. File Types

| Extension | Description | Purpose |
|-----------|-------------|---------|
| `.a16` | A16 source file | Main source code |
| `.a16i` | A16 interface file | Type stubs, FFI declarations |
| `.a16c` | A16 compiled bytecode | Cached compilation |
| `.a16m` | A16 memory snapshot | Serialized agent memory |
| `a16.toml` | Package manifest | Project configuration |
| `a16.lock` | Dependency lock file | Reproducible builds |

---

## 2. Project Structure

### 2.1 Minimal Project

```
my_project/
├── a16.toml           # Package manifest
└── main.a16           # Entry point
```

### 2.2 Standard Project

```
my_project/
├── a16.toml           # Package manifest
├── a16.lock           # Dependency lock
├── src/
│   ├── __init__.a16   # Package root
│   ├── main.a16       # Entry point
│   ├── agent.a16      # Agent definitions
│   └── utils.a16      # Utilities
├── tests/
│   ├── __init__.a16
│   ├── test_agent.a16
│   └── fixtures/
│       └── data.json
├── prompts/
│   ├── system.prompt  # Prompt templates
│   └── tasks.prompt
├── memory/
│   └── .gitkeep       # Persistent memory storage
├── docs/
│   └── README.md
└── .a16/
    ├── cache/         # Compilation cache
    └── memory/        # Local memory store
```

### 2.3 Multi-Agent Project

```
ai_team/
├── a16.toml
├── src/
│   ├── __init__.a16
│   ├── team.a16           # Team coordinator
│   ├── agents/
│   │   ├── __init__.a16
│   │   ├── researcher.a16
│   │   ├── writer.a16
│   │   └── critic.a16
│   ├── tools/
│   │   ├── __init__.a16
│   │   ├── web.a16
│   │   └── database.a16
│   └── memory/
│       ├── __init__.a16
│       └── shared.a16
├── prompts/
│   ├── researcher/
│   │   └── system.prompt
│   ├── writer/
│   │   └── system.prompt
│   └── critic/
│       └── system.prompt
├── tests/
│   ├── unit/
│   └── integration/
└── config/
    ├── dev.toml
    ├── staging.toml
    └── prod.toml
```

---

## 3. Package Manifest (a16.toml)

```toml
[package]
name = "my-ai-agent"
version = "0.1.0"
description = "An intelligent research assistant"
authors = ["Your Name <you@example.com>"]
license = "MIT"
repository = "https://github.com/you/my-ai-agent"
documentation = "https://docs.example.com/my-ai-agent"
readme = "README.md"
keywords = ["ai", "agent", "research"]

# Minimum A16 version required
a16 = ">=0.1.0"

[dependencies]
a16-openai = "^1.0.0"
a16-anthropic = "^1.0.0"
a16-web = "^0.5.0"

[dev-dependencies]
a16-test = "^0.1.0"
a16-bench = "^0.1.0"

[build]
entry = "src/main.a16"
target = "native"  # native | wasm | bytecode

[runtime]
workers = "auto"           # Number of async workers
memory_limit = "4GB"       # Max memory usage
token_budget = 100000      # Default token budget
model_default = "gpt-4o"   # Default model

[models]
# Model configurations
[models.gpt4]
provider = "openai"
model = "gpt-4o"
api_key_env = "OPENAI_API_KEY"

[models.claude]
provider = "anthropic"
model = "claude-3-5-sonnet"
api_key_env = "ANTHROPIC_API_KEY"

[models.local]
provider = "ollama"
model = "llama3.1:70b"
endpoint = "http://localhost:11434"

[tools]
# Tool configurations
sandbox_default = "light"
rate_limit_default = "60/minute"

[memory]
# Memory configuration
backend = "sqlite"         # sqlite | postgres | redis
path = ".a16/memory"
vector_dimensions = 1536

[logging]
level = "info"             # debug | info | warn | error
format = "json"            # json | text
output = "stdout"          # stdout | file | both

[security]
# Security policies
allow_network = true
allow_filesystem = true
allowed_domains = ["*"]
sandbox_level = "light"

[scripts]
# Custom scripts
dev = "a16 run --watch src/main.a16"
test = "a16 test tests/"
bench = "a16 bench benchmarks/"
lint = "a16 lint src/"
deploy = "./scripts/deploy.sh"
```

---

## 4. Module System

### 4.1 Module Resolution

```
Import: from my_package.agents.researcher import Researcher

Resolution order:
1. Check local src/ directory
   → src/agents/researcher.a16
   
2. Check installed packages
   → .a16/packages/my_package/agents/researcher.a16
   
3. Check standard library
   → $A16_HOME/lib/...
```

### 4.2 Package Initialization

```a16
# src/__init__.a16

### 
MyPackage
=========
Top-level package initialization.
###

# Re-export key items
from .agent import MyAgent
from .utils import helper

# Package-level constants
const VERSION = "0.1.0"

# Export list
export [MyAgent, helper, VERSION]
```

### 4.3 Relative Imports

```a16
# In src/agents/researcher.a16

# Same directory
from .utils import helper

# Parent directory
from ..tools import web_search

# Package root
from my_package import VERSION
```

---

## 5. Configuration Hierarchy

```
Priority (highest to lowest):
1. Command-line flags        (--token-budget=5000)
2. Environment variables     (A16_TOKEN_BUDGET=5000)
3. Local config file         (.a16/config.toml)
4. Project config            (a16.toml)
5. User config               (~/.a16/config.toml)
6. System config             (/etc/a16/config.toml)
7. Built-in defaults
```

---

## 6. Lock File (a16.lock)

```toml
# Auto-generated, do not edit
# a16 lock version 1

[[package]]
name = "a16-openai"
version = "1.0.0"
source = "registry+https://packages.a16.dev"
checksum = "sha256:abc123..."

[[package]]
name = "a16-anthropic"
version = "1.0.0"
source = "registry+https://packages.a16.dev"
checksum = "sha256:def456..."
dependencies = [
    "a16-http ^0.5.0"
]
```

---

## 7. Prompt Files (.prompt)

```
# prompts/system.prompt

---
name: research_system
version: 1
variables:
  - topic: string
  - style: string = "academic"
---

You are a research assistant specializing in {topic}.

Guidelines:
- Use {style} writing style
- Cite all sources
- Be concise but thorough

{% if context %}
Previous relevant information:
{% for item in context %}
- {{ item }}
{% endfor %}
{% endif %}
```

---

## 8. Published Package Structure

```
package-name-1.0.0.a16pkg (compressed archive)
├── a16.toml              # Manifest
├── README.md             # Documentation
├── LICENSE               # License file
├── src/                  # Source code
│   └── ...
├── lib/                  # Pre-compiled bytecode
│   └── *.a16c
└── types/                # Type definitions
    └── *.a16i
```

---

*Next: [CLI Tooling Design](./06-cli-tooling.md)*
