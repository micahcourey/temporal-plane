# AI DX Toolkit — Mnemix Adapter

This adapter bridges the [AI DX Toolkit](https://github.com/ai-dx-toolkit) memory
event model to the [Mnemix](../../README.md) local-first memory layer.

## Scope

- Maps toolkit memory events (observations, decisions) to Mnemix `remember` calls
- Provides layered context retrieval via `recall` for agent prompt injection
- Exposes session-level checkpointing for safe pre-operation snapshots
- Depends only on the public `mnemix` Python package — no LanceDB internals

## Installation

Install the `mnemix` Python package from the workspace root, then use this
adapter module directly:

```bash
cd python && pip install -e ".[dev]"
```

## Quick Start

```python
from pathlib import Path
from mnemix_adapter import MnemixAdapter

adapter = MnemixAdapter(store=Path(".mnemix"))
adapter.ensure_store()

# Record a toolkit observation
adapter.record_observation(
    id="obs-001",
    scope="my-agent",
    title="Dependency added",
    summary="Added httpx to pyproject.toml.",
    detail="Chose httpx over requests for async support and type safety.",
    tags=["dependencies"],
    source_tool="package-manager",
)

# Pin an important architectural decision
adapter.record_decision(
    id="dec-001",
    scope="my-agent",
    title="Use Rust for core logic",
    summary="Decided to implement core storage logic in Rust.",
    detail="Full rationale: performance, safety, and LanceDB SDK alignment.",
    pin_reason="Core architectural choice — should persist across sessions.",
)

# Retrieve layered context before a new agent session
context = adapter.fetch_context(scope="my-agent", query="dependencies")
for entry in context:
    print(f"[{entry.layer}] {entry.memory.title}")

# Create a checkpoint before a bulk operation
checkpoint = adapter.create_session_checkpoint(
    "sess-20260308",
    description="Pre-import checkpoint",
)
print(f"Checkpoint '{checkpoint.name}' at version {checkpoint.version}")
```

## API

| Method | Purpose |
|--------|---------|
| `ensure_store()` | Initialise the store (idempotent) |
| `record_observation(...)` | Persist a toolkit observation memory |
| `record_decision(...)` | Persist a decision, optionally pinned |
| `fetch_context(scope, query, limit)` | Retrieve layered context for prompt injection |
| `create_session_checkpoint(session_id, description)` | Create a named checkpoint |
| `get_stats(scope)` | Get store statistics |

## Tests

```bash
cd adapters/ai-dx-toolkit
pip install pytest pytest-mock
pytest tests/
```

## Design Principles

- The adapter uses only public `mnemix` package APIs.
- No LanceDB or lance-specific assumptions are made here.
- Core semantics (pinning, layered recall, checkpoints) stay aligned with the CLI surface.
- Defer direct FFI or PyO3 bindings until a dedicated stable Rust application API exists.
