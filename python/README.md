# Mnemix Python Binding

A thin, typed Python client for [Mnemix](../README.md) — a local-first
memory layer for AI coding agents.

All product logic lives in the Rust `mnemix` CLI binary.  This package
wraps its `--json` output surface; no core behavior is duplicated here.

## Requirements

- Python 3.11 or later
- On supported platforms, the published wheel bundles the `mnemix`
    binary automatically
- On unsupported platforms or source installs, install the CLI separately via
    `cargo install --path crates/mnemix-cli` or set
    `MNEMIX_BINARY=/path/to/mnemix`

## Installation

Install the Python wrapper from PyPI:

```bash
pip install mnemix
```

On supported platforms, this wheel includes the Rust `mnemix` CLI
binary and should work without any additional setup.

If no bundled wheel is available for your platform, install the CLI separately
and ensure `mnemix` is on `PATH`, or set `MNEMIX_BINARY` to an explicit
binary path.

## Installation (development)

```bash
cd python
pip install -e ".[dev]"
```

## Quick Start

```python
from pathlib import Path
from mnemix import Mnemix, RememberRequest

tp = Mnemix(store=Path(".mnemix"))
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
| `MnemixError` | Base class for all errors |
| `MnemixCommandError` | CLI returned a non-zero exit or error JSON |
| `MnemixBinaryNotFoundError` | `mnemix` binary not found on PATH |
| `MnemixDecodeError` | CLI output could not be decoded |

## Running Tests

```bash
cd python
pip install -e ".[dev]"
pytest
```

## Release Validation

Before publishing a new version, from a local clone of this repository run the
release checks from the repository root:

```bash
# From the repository root (source checkout)
./scripts/check-python-package.sh
```

This runs Python tests, builds the wheel and sdist, and validates package
metadata rendering with `twine check`. It also installs the freshly built wheel
into a clean virtual environment and verifies that `mnemix` imports and
exposes `__version__` correctly.

## Binding Strategy

The current binding uses the CLI `--json` surface as the execution boundary.
Direct FFI via PyO3 is deferred until a dedicated stable Rust application API
exists and there is a concrete need that the CLI boundary cannot satisfy.
