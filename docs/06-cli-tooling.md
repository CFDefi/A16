# A16 CLI Tooling Design

> **Version:** 0.1 | **Status:** Draft | **Date:** 2026-01-19

---

## Command Overview

```
a16 <command> [options] [arguments]

Commands:
  run       Execute an A16 program
  repl      Start interactive REPL
  new       Create a new project
  build     Compile to bytecode
  test      Run tests
  bench     Run benchmarks
  fmt       Format source code
  lint      Check code quality
  check     Type-check without running
  pkg       Package management
  doctor    Diagnose issues
  trace     Analyze execution traces
  memory    Memory management utilities
```

---

## 1. a16 run

Execute an A16 program.

```bash
a16 run [options] <file> [-- args...]

Options:
  --watch, -w              Watch for changes and restart
  --debug                  Enable debug mode
  --deterministic          Deterministic execution (reproducible)
  --safe                   Maximum security mode
  --profile                Enable profiling
  --trace <file>           Write execution trace
  --token-budget <n>       Set token budget
  --cost-limit <n>         Set cost limit (dollars)
  --timeout <duration>     Set execution timeout
  --model <name>           Override default model
  --memory <path>          Load memory from path
  --config <file>          Use specific config file
  --env <file>             Load environment from file

Examples:
  a16 run main.a16
  a16 run --watch src/main.a16
  a16 run --debug --trace trace.json main.a16
  a16 run --deterministic --seed=42 test.a16
  a16 run --token-budget=5000 research.a16 -- "AI trends"
```

### Execution Modes Matrix

| Flag | Token Tracking | Sandboxing | Determinism | Tracing |
|------|---------------|------------|-------------|---------|
| (default) | On | Light | Off | Off |
| --debug | On | Light | Off | Full |
| --safe | On | Maximum | On | Full |
| --deterministic | On | Light | On | Off |
| --profile | On | Light | Off | Sampling |

---

## 2. a16 repl

Start an interactive REPL session.

```bash
a16 repl [options]

Options:
  --memory, -m <path>      Persistent memory path
  --model <name>           Model to use
  --tools <list>           Comma-separated tools to enable
  --agent <file>           Load agent definition
  --history <file>         Session history file
  --no-color               Disable colored output

Examples:
  a16 repl
  a16 repl --memory ./session.a16m
  a16 repl --agent src/assistant.a16
  a16 repl --model gpt-4o-mini --tools web_search,calculator
```

### REPL Commands

```
>>> help                    # Show help
>>> clear                   # Clear screen
>>> history                 # Show history
>>> save <file>             # Save session
>>> load <file>             # Load session
>>> memory                  # Show memory state
>>> memory.clear            # Clear memory
>>> tokens                  # Show token usage
>>> trace                   # Show last trace
>>> exit / quit / Ctrl+D    # Exit REPL

# Multi-line input
>>> fn example():
...     return 42
...
>>> example()
42
```

---

## 3. a16 new

Create a new A16 project.

```bash
a16 new [options] <name>

Options:
  --template, -t <name>    Project template
  --git                    Initialize git repository
  --no-git                 Skip git initialization
  --path <dir>             Create in specific directory

Templates:
  minimal                  Bare-bones project
  agent                    Single agent project (default)
  team                     Multi-agent team project
  api                      HTTP API with agents
  cli                      CLI application
  library                  Reusable library

Examples:
  a16 new my-agent
  a16 new --template team my-research-team
  a16 new --template api ai-api-service
```

---

## 4. a16 build

Compile A16 source to bytecode.

```bash
a16 build [options] [target]

Options:
  --release                Optimize for production
  --target <target>        Compilation target (native|wasm|bytecode)
  --out, -o <dir>          Output directory
  --no-cache               Ignore compilation cache
  --check-only             Type-check without emitting

Examples:
  a16 build
  a16 build --release
  a16 build --target wasm --out dist/
```

---

## 5. a16 test

Run tests.

```bash
a16 test [options] [filter]

Options:
  --watch, -w              Watch for changes
  --coverage               Generate coverage report
  --parallel, -j <n>       Parallel test jobs
  --timeout <duration>     Test timeout
  --filter <pattern>       Filter tests by name
  --verbose, -v            Verbose output
  --fail-fast              Stop on first failure
  --mock-models            Use mock model responses
  --snapshot               Update snapshots

Examples:
  a16 test
  a16 test tests/agent_test.a16
  a16 test --filter "test_research*"
  a16 test --coverage --parallel=4
  a16 test --mock-models  # Deterministic, no API calls
```

### Test Output

```
a16 test

Running tests...

tests/test_agent.a16
  ✓ test_agent_initialization (0.05s)
  ✓ test_memory_persistence (0.12s)
  ✓ test_tool_execution (0.34s, 150 tokens)
  ✗ test_budget_enforcement (0.22s)
    Expected: BudgetExceeded
    Got: Response with 1200 tokens

tests/test_memory.a16
  ✓ test_store_retrieve (0.08s)
  ✓ test_compression (0.44s)

Results: 5 passed, 1 failed, 0 skipped
Time: 1.25s
Tokens: 450 (est. $0.02)
Coverage: 78.5%
```

---

## 6. a16 bench

Run benchmarks.

```bash
a16 bench [options] [filter]

Options:
  --baseline <file>        Compare against baseline
  --save <file>            Save results as baseline
  --iterations <n>         Number of iterations
  --warmup <n>             Warmup iterations
  --output <format>        Output format (table|json|csv)

Examples:
  a16 bench
  a16 bench benchmarks/model_bench.a16
  a16 bench --save baseline.json
  a16 bench --baseline baseline.json  # Compare
```

### Benchmark Output

```
a16 bench

Benchmark Results
═════════════════════════════════════════════════════════

Name                     │ Time      │ Tokens │ Cost
─────────────────────────┼───────────┼────────┼─────────
simple_query             │ 234ms     │ 120    │ $0.001
complex_research         │ 4.5s      │ 2,340  │ $0.047
multi_agent_team         │ 12.3s     │ 8,920  │ $0.178

vs Baseline (Python + LangChain):
─────────────────────────────────────────────────────────
simple_query             │ 2.1x faster  │ 35% fewer tokens
complex_research         │ 3.4x faster  │ 52% fewer tokens
multi_agent_team         │ 4.8x faster  │ 61% fewer tokens
```

---

## 7. a16 fmt

Format source code.

```bash
a16 fmt [options] [files...]

Options:
  --check                  Check formatting without modifying
  --diff                   Show diff of changes
  --config <file>          Use specific formatter config
  --stdin                  Read from stdin
  --exclude <patterns>     Exclude patterns

Examples:
  a16 fmt                  # Format all files
  a16 fmt src/main.a16     # Format specific file
  a16 fmt --check          # CI check
  a16 fmt --diff           # Preview changes
```

### Formatter Configuration (.a16fmt.toml)

```toml
indent_size = 4
max_line_length = 100
trailing_comma = true
sort_imports = true
group_imports = true
blank_lines_after_imports = 2
```

---

## 8. a16 lint

Check code quality.

```bash
a16 lint [options] [files...]

Options:
  --fix                    Auto-fix issues where possible
  --strict                 Enable strict checks
  --config <file>          Use specific linter config
  --format <format>        Output format (text|json|sarif)
  --rules <rules>          Enable specific rules

Examples:
  a16 lint
  a16 lint --fix
  a16 lint --strict src/
  a16 lint --format sarif > report.sarif
```

### Lint Rules

```
Token Rules:
  T001  Unbounded token usage
  T002  Missing token budget
  T003  Inefficient prompt pattern

Safety Rules:
  S001  Unsandboxed tool
  S002  Missing permissions
  S003  Hardcoded secrets

Style Rules:
  E001  Line too long
  E002  Improper indentation
  E003  Missing docstring

Performance Rules:
  P001  Sequential model calls (should be parallel)
  P002  Uncached expensive operation
  P003  Inefficient memory access pattern
```

---

## 9. a16 check

Type-check without running.

```bash
a16 check [options] [files...]

Options:
  --strict                 Strict type checking
  --all                    Check all files, not just modified
  --explain                Explain type errors in detail
  --json                   JSON output

Examples:
  a16 check
  a16 check --strict src/
  a16 check --explain
```

---

## 10. a16 pkg

Package management.

```bash
a16 pkg <subcommand> [options]

Subcommands:
  install     Install packages
  uninstall   Remove packages
  update      Update packages
  list        List installed packages
  search      Search registry
  publish     Publish package
  info        Package information
  outdated    Check for updates

Examples:
  a16 pkg install a16-openai
  a16 pkg install a16-openai@^1.0.0
  a16 pkg install --dev a16-test
  a16 pkg uninstall a16-openai
  a16 pkg update
  a16 pkg search "web scraping"
  a16 pkg publish
  a16 pkg info a16-openai
  a16 pkg outdated
```

### Package Installation Output

```
a16 pkg install a16-openai

Resolving dependencies...
  a16-openai@1.0.0
  └── a16-http@0.5.2
      └── a16-async@0.3.1

Installing:
  ✓ a16-async@0.3.1 (cached)
  ✓ a16-http@0.5.2 (downloading... 45KB)
  ✓ a16-openai@1.0.0 (downloading... 120KB)

Installed 3 packages in 1.2s
```

---

## 11. a16 doctor

Diagnose issues and check environment.

```bash
a16 doctor [options]

Options:
  --fix                    Attempt to fix issues
  --verbose                Detailed output

Examples:
  a16 doctor
  a16 doctor --fix
```

### Doctor Output

```
a16 doctor

A16 Environment Check
═════════════════════════════════════════════════════════

Runtime:
  ✓ A16 version: 0.1.0
  ✓ Platform: Windows 11 x64
  ✓ Memory: 16GB available

Configuration:
  ✓ Config file: ~/.a16/config.toml
  ✓ Package cache: ~/.a16/cache
  ⚠ OPENAI_API_KEY: not set

Models:
  ✓ OpenAI API: connected
  ✗ Anthropic API: key invalid
  ✓ Ollama: running (localhost:11434)

Packages:
  ✓ 12 packages installed
  ⚠ 3 packages have updates

Recommendations:
  1. Set OPENAI_API_KEY environment variable
  2. Update Anthropic API key in config
  3. Run 'a16 pkg update' to update packages
```

---

## 12. a16 trace

Analyze execution traces.

```bash
a16 trace <subcommand> [options]

Subcommands:
  view        View trace file
  analyze     Analyze trace for insights
  compare     Compare two traces
  export      Export trace to format

Examples:
  a16 trace view trace.json
  a16 trace analyze trace.json --hot-spots
  a16 trace compare before.json after.json
  a16 trace export trace.json --format=chrome
```

---

## 13. a16 memory

Memory management utilities.

```bash
a16 memory <subcommand> [options]

Subcommands:
  inspect     View memory contents
  export      Export memory to file
  import      Import memory from file
  compress    Compress memory
  clear       Clear memory
  stats       Memory statistics

Examples:
  a16 memory inspect ./memory.a16m
  a16 memory export ./memory.a16m --format=json
  a16 memory compress ./memory.a16m --strategy=summarize
  a16 memory stats ./memory.a16m
```

---

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `A16_HOME` | A16 installation directory | Platform-specific |
| `A16_CONFIG` | Config file path | `~/.a16/config.toml` |
| `A16_CACHE` | Cache directory | `~/.a16/cache` |
| `A16_LOG_LEVEL` | Log verbosity | `info` |
| `A16_TOKEN_BUDGET` | Default token budget | Unlimited |
| `A16_COST_LIMIT` | Default cost limit | Unlimited |
| `A16_DEFAULT_MODEL` | Default model | `gpt-4o` |
| `OPENAI_API_KEY` | OpenAI API key | - |
| `ANTHROPIC_API_KEY` | Anthropic API key | - |

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Command-line usage error |
| 3 | Compilation error |
| 4 | Runtime error |
| 5 | Test failure |
| 6 | Lint error |
| 7 | Budget exceeded |
| 8 | Timeout |
| 9 | Policy violation |

---

*Next: [Benchmarks + Proof Suite](./07-benchmarks.md)*
