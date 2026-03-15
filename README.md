# Mnemix

> A lightweight, inspectable, local-first memory engine for AI coding agents — with built-in version history and time-travel.

```mermaid
%%{init: {
  "theme": "base",
  "themeVariables": {
    "background": "#ffffff",
    "primaryTextColor": "#24292f",
    "primaryColor": "#f6f8fa",
    "primaryBorderColor": "#0969da",
    "lineColor": "#57606a",
    "secondaryColor": "#ddf4ff",
    "tertiaryColor": "#d8f5d0"
  }
}}%%
mindmap
  root((🧠 Mnemix))
    📝 Remember
      observations
      decisions
      preferences
      facts & procedures
    🔍 Recall
      pinned context first
      summaries on demand
      full archival depth
    🔎 Search
      full-text across scopes
      filter by kind & tags
      ranked by importance
    🛡️ Protect
      named checkpoints
        versioned history
      time-travel restore
    🔬 Inspect
      human-readable CLI
      store stats & diffs
      import / export
```

AI coding agents have no continuous memory. Every session starts cold. Mnemix changes that — giving your agent a structured, local memory store that persists between sessions, supports rich retrieval, and lets you inspect, restore, and version everything.

No cloud, no daemon, no configuration sprawl. Just a local store on your filesystem that your agent can write to and read from, with a clean CLI and Python client sitting on top.

---

## How it works

An agent calls `remember` to persist an observation, decision, or fact. Later sessions call `recall` or `search` to retrieve the most relevant context. The full version history of the store is preserved — you can checkpoint before risky operations, list what changed, and restore to any prior state.

```mermaid
sequenceDiagram
    participant A as 🤖 AI Agent
    participant TP as Mnemix
    participant S as 💾 Local Store

    rect rgb(15, 118, 110)
        Note over A,S: Session 1
        A->>TP: remember(scope, kind, title, summary, detail)
        TP->>S: persist memory → Version N created
    end

    rect rgb(3, 105, 161)
        Note over A,S: Session 2
        A->>TP: recall(scope="my-project")
        TP->>S: query versions + index
        S-->>TP: matching memories
        TP-->>A: pinned_context · summaries · archival
    end
```

Context is returned in **three layers** based on relevance:

| Layer | What it contains | When it loads |
|-------|-----------------|---------------|
| `pinned_context` | Explicitly pinned, always-relevant memories | Always first |
| `summaries` | High-importance and recent distilled memories | By default |
| `archival` | Full historical record | On request |

This **progressive disclosure** pattern means you never flood an agent's context window with noise.

---

## Features

- **Scoped memory** — organize memories by project or context
- **Typed memory kinds** — `observation`, `decision`, `preference`, `summary`, `fact`, `procedure`, `warning`
- **Full-text search** — fast FTS with scope and importance filters
- **Pinned context** — pin critical decisions or preferences to always surface first
- **Version history** — every write creates an immutable version; inspect and browse the full timeline
- **Checkpoints** — named, human-readable stable points in the version history
- **Time-travel restore** — restore the store to any prior version or checkpoint as a new head state
- **Import / export** — portable store archives
- **Store optimization** — compact and prune old data with safety checkpoints built in
- **Human-readable CLI** — inspect the full store state from the terminal
- **Python client** — typed, thin wrapper for use in Python agents and scripts
- **Local-first, no cloud** — runs entirely on your filesystem, zero external dependencies

---

## Quick start

### Install from PyPI

On supported platforms, the PyPI wheel bundles the `mnemix` CLI binary:

```bash
pip install mnemix
```

If you want the CLI as a standalone tool, prefer `pipx`:

```bash
pipx install mnemix
```

If no bundled wheel is available for your platform, install the CLI from source:

```bash
cargo install --path crates/mnemix-cli
```

### Initialize a store

```bash
mnemix --store .mnemix init
```

### Persist a memory

```bash
mnemix --store .mnemix remember \
  --id mem-001 \
  --scope my-project \
  --kind observation \
  --title "Decided to use LanceDB for local storage" \
  --summary "LanceDB chosen for Arrow-native embedded storage with FTS and versioning." \
  --detail "Evaluated SQLite, DuckDB, and LanceDB. LanceDB wins on versioning and vector support." \
  --importance 80 \
  --tag architecture --tag storage
```

### Recall context

```bash
mnemix --store .mnemix recall --scope my-project
```

### Search

```bash
mnemix --store .mnemix search --text "storage decision" --scope my-project
```

### Checkpoint before a risky operation

```bash
mnemix --store .mnemix checkpoint \
  --name before-refactor \
  --description "Stable state before large codebase restructure"
```

### Restore to a prior state

```bash
mnemix --store .mnemix restore --checkpoint before-refactor
```

---

## Python client

For library use, install from PyPI:

```bash
pip install mnemix
```

For CLI-first use, install with `pipx`:

```bash
pipx install mnemix
```

> On supported platforms, the wheel includes the `mnemix` CLI binary and works without any extra setup. If you are on an unsupported platform or using a source install, install the CLI separately and ensure it is on `PATH`, or set `MNEMIX_BINARY=/path/to/mnemix`.

```python
from pathlib import Path
from mnemix import Mnemix, RememberRequest

client = Mnemix(store=Path(".mnemix"))
client.init()

client.remember(RememberRequest(
    id="mem-001",
    scope="my-project",
    kind="decision",
    title="Use LanceDB for local storage",
    summary="LanceDB chosen for Arrow-native embedded storage with FTS and versioning.",
    detail="Evaluated SQLite, DuckDB, and LanceDB. LanceDB wins on versioning and vector support.",
    importance=80,
    tags=["architecture", "storage"],
))

# Retrieve layered context
context = client.recall()
for entry in context.pinned_context:
    print(f"[pinned] {entry.memory.title}")

# Full-text search
results = client.search("storage decision", scope="my-project")
for m in results:
    print(f"{m.id}: {m.title} (importance={m.importance})")

# Inspect store stats
stats = client.stats()
print(f"Total memories: {stats.total_memories}")
```

---

## Host adapters

Mnemix keeps the base Python client generic and puts workflow policy in
host-specific adapters under [`adapters/`](/Users/micah/Projects/mnemix/adapters).

Available adapters:

- `CodingAgentAdapter` for implementation tasks, decisions, procedures, and pitfalls
- `ChatAssistantAdapter` for user preferences and durable chat context
- `CiBotAdapter` for automated runs, checkpoints, failures, and fixes
- `ReviewToolAdapter` for review rules, conventions, and recurring issues

See:

- [adapters/README.md](/Users/micah/Projects/mnemix/adapters/README.md)
- [docs_site/src/guide/host-adapters.md](/Users/micah/Projects/mnemix/docs_site/src/guide/host-adapters.md)
- [examples/agent-memory-layer/README.md](/Users/micah/Projects/mnemix/examples/agent-memory-layer/README.md)

---

## Memory model

Each memory record has:

| Field | Purpose |
|-------|---------|
| `id` | Stable identifier for this memory |
| `scope` | Project or context namespace |
| `kind` | One of `observation`, `decision`, `preference`, `summary`, `fact`, `procedure`, `warning` |
| `title` | Short human-readable label |
| `summary` | Distilled, compact version of the memory |
| `detail` | Full detail — the complete context |
| `importance` | 0–100 score, controls recall ranking |
| `confidence` | 0–100 score, how certain this memory is |
| `tags` | Free-form labels for filtering and search |
| `entities` | Named entities mentioned in this memory |
| `pin_reason` | If set, this memory is pinned and surfaces first in recall |

---

## Version history and time-travel

Every write creates a new immutable version:

```bash
# List versions
mnemix --store .mnemix versions

# Create a named checkpoint
mnemix --store .mnemix checkpoint --name stable-baseline

# Restore the store to a named checkpoint
mnemix --store .mnemix restore --checkpoint stable-baseline
```

Restore creates a **new head state** from the prior version — it does not discard history.

---

## Repository structure

```
mnemix/
├── crates/
│   ├── mnemix-core/       # Mnemix core crate source path
│   ├── mnemix-lancedb/    # Mnemix LanceDB backend source path
│   ├── mnemix-cli/        # Mnemix CLI source path
│   ├── mnemix-types/      # Shared value objects and contracts
│   └── mnemix-test-support/
├── python/                        # Python package (mnemix on PyPI)
│   ├── mnemix/
├── adapters/
│   ├── _adapter_base.py           # Shared adapter utilities
│   ├── coding_agent_adapter.py    # Coding-agent workflow adapter
│   ├── chat_assistant_adapter.py  # Chat-assistant workflow adapter
│   ├── ci_bot_adapter.py          # CI-bot workflow adapter
│   ├── review_tool_adapter.py     # Review-tool workflow adapter
│   └── tests/                     # Adapter smoke tests
├── examples/                      # Runnable usage examples
└── docs/                          # Architecture and design documentation
```

### Crate responsibilities

| Crate | Responsibility |
|-------|---------------|
| `mnemix-core` | Domain types, traits, typed errors — no storage-specific code |
| `mnemix-lancedb` | All LanceDB and Lance storage details, behind core traits |
| `mnemix-cli` | CLI parsing, command dispatch, human + JSON output rendering |
| `mnemix-types` | Shared value objects and request/response contracts |
| `mnemix-test-support` | Deterministic test fixtures — non-production only |

The core crate is intentionally storage-agnostic. LanceDB details never leak into it.

---

## Architecture: the binding strategy

The Python package is a thin subprocess wrapper over the CLI's `--json` output surface. No product logic is duplicated in Python — Rust is the single source of truth.

```mermaid
flowchart TD
    A(["🐍 Python Agent"])
    B["mnemix CLI<br/>--json output surface"]
    C["mnemix-lancedb<br/>Rust storage backend"]
    D[("Lance dataset<br/>on filesystem")]

    A -- "subprocess + --json" --> B
    B -- "Rust traits" --> C
    C --> D

    style A fill:#4f46e5,color:#fff,stroke:#4338ca
    style B fill:#0f766e,color:#fff,stroke:#0d9488
    style C fill:#0369a1,color:#fff,stroke:#0284c7
    style D fill:#1e293b,color:#e2e8f0,stroke:#334155
```

Direct FFI via PyO3 is planned for a future milestone once the stable Rust API surface is locked.

---

## Roadmap status

| Milestone | Description | Status |
|-----------|-------------|--------|
| 0 | Workspace and engineering baseline | ✅ Done |
| 1 | Core domain contract freeze | ✅ Done |
| 2 | Local LanceDB backend MVP | ✅ Done |
| 3 | Human-first CLI MVP | ✅ Done |
| 4 | Progressive disclosure and pinning semantics | ✅ Done |
| 5 | Version-aware safety features | ✅ Done |
| 6 | Python binding and first adapter | ✅ Done |
| 7 | Advanced storage workflows (branch-aware internals) | ✅ Done |
| — | First PyPI release | 🚀 In progress |

---

## Contributing and engineering baseline

The workspace is configured with:

- stable Rust toolchain pinning via `rust-toolchain.toml`
- `rustfmt` and `clippy` with strict lint defaults
- `cargo-deny` for dependency policy
- `cargo test --workspace` and `cargo doc --workspace` as CI gates

### Run baseline checks

```bash
./scripts/check.sh
```

### Python packaging validation

```bash
./scripts/check-python-package.sh
```

---

## Documentation

- [Architecture and plan](docs/mnemix-plan-v3.md)
- [Roadmap and milestones](docs/mnemix-roadmap.md)
- [LanceDB Rust SDK agent guide](docs/lancedb-rust-sdk-agent-guide.md)
- [Checkpoint and retention policy](docs/checkpoint-and-retention-policy.md)
- [Versioning and restore](docs/versioning-and-restore.md)
- [Branch lifecycle](docs/branch-lifecycle.md)
- [Python package README](python/README.md)
- [Release checklist](docs/release-checklist.md)

---

## License

MIT — see [LICENSE](LICENSE).
