# Temporal Plane Python Binding

A thin, typed Python client for [Temporal Plane](../README.md) — a local-first
memory layer for AI coding agents.

All product logic lives in the Rust `temporal-plane` CLI binary.  This package
wraps its `--json` output surface; no core behavior is duplicated here.

## Requirements

- Python 3.11 or later
- The `temporal-plane` binary on `PATH` (install from the workspace root via
  `cargo install --path crates/temporal-plane-cli`) or set
  `TP_BINARY=/path/to/temporal-plane`

## Installation (development)

```bash
cd python
pip install -e ".[dev]"
```

## Quick Start

```python
from pathlib import Path
from temporal_plane import TemporalPlane, RememberRequest

tp = TemporalPlane(store=Path(".temporal-plane"))
tp.init()

tp.remember(RememberRequest(
    id="mem-001",
    scope="my-project",
    kind="observation",
    title="Initial scaffolding complete",
    summary="Rust project scaffold created with workspace layout.",
    detail="Added Cargo.toml workspace, core, lancedb, cli, and types crates.",
    tags=["scaffolding"],
))

results = tp.search("scaffolding", scope="my-project")
for m in results:
    print(f"{m.id}: {m.title}")

context = tp.recall()
for entry in context.pinned_context:
    print(f"[pinned] {entry.memory.title}")

stats = tp.stats()
print(f"Total memories: {stats.total_memories}")
```

## API Overview

| Method | Purpose |
|--------|---------|
| `init()` | Initialise the store (idempotent) |
| `remember(request)` | Persist a memory record |
| `show(memory_id)` | Retrieve full detail for a memory |
| `search(text, *, scope, limit)` | Full-text search |
| `recall(request)` | Layered context recall |
| `pins(*, scope, limit)` | List pinned memories |
| `history(*, scope, limit)` | List recent memories |
| `checkpoint(request)` | Create a named checkpoint |
| `versions(*, limit)` | List store versions |
| `restore(request)` | Restore to a checkpoint or version |
| `optimize(request)` | Compact and optionally prune old versions |
| `stats(*, scope)` | Get store statistics |
| `export(destination)` | Export the store |
| `import_store(source)` | Import a store archive |

## Errors

| Exception | When raised |
|-----------|-------------|
| `TemporalPlaneError` | Base class for all errors |
| `TemporalPlaneCommandError` | CLI returned a non-zero exit or error JSON |
| `TemporalPlaneBinaryNotFoundError` | `temporal-plane` binary not found on PATH |
| `TemporalPlaneDecodeError` | CLI output could not be decoded |

## Running Tests

```bash
cd python
pip install -e ".[dev]"
pytest
```

## Binding Strategy

The current binding uses the CLI `--json` surface as the execution boundary.
Direct FFI via PyO3 is deferred until a dedicated stable Rust application API
exists and there is a concrete need that the CLI boundary cannot satisfy.
