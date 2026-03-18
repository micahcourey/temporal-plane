# Python

The `mnemix` Python package is a thin, typed client for Mnemix. It wraps the CLI's `--json` interface instead of re-implementing product logic in Python.

That keeps the public behavior aligned across terminal and Python usage while giving agent frameworks a clean programmatic interface.

## Requirements

- Python 3.11 or later
- a `mnemix` binary available either from the wheel, from `PATH`, or via `MNEMIX_BINARY`

## Installation

Install from PyPI:

```bash
pip install mnemix
```

If you mainly want the CLI as a standalone tool, install it with `pipx`:

```bash
pipx install mnemix
```

On supported platforms, the wheel bundles the `mnemix` CLI binary. If your platform does not have a bundled binary, install the CLI separately or point `MNEMIX_BINARY` at an existing binary.

## Quick start

```python
from pathlib import Path
from mnemix import Mnemix, RememberRequest

client = Mnemix(store=Path(".mnemix"))
client.init()

client.remember(RememberRequest(
    id="memory:first-note",
    scope="repo:mnemix",
    kind="observation",
    title="Initial store created",
    summary="Confirmed the Python client can initialize and write to the store.",
    detail="This record verifies the CLI-backed Python workflow end to end.",
    tags=["python", "smoke-test"],
))

results = client.search("Python client", scope="repo:mnemix")
for memory in results:
    print(memory.id, memory.title)

context = client.recall()
print(context.count)
```

## API surface

The client exposes the same core operations as the CLI:

| Method | Purpose |
|---|---|
| `init()` | Initialize a store |
| `remember(request)` | Persist a memory record |
| `show(memory_id)` | Return one memory in full detail |
| `search(text, *, scope, limit)` | Search stored memories |
| `recall(request=None)` | Return layered recall results |
| `pins(*, scope, limit)` | List pinned memories |
| `history(*, scope, limit)` | Inspect recent history |
| `checkpoint(request)` | Create a named checkpoint |
| `versions(*, limit)` | List store versions |
| `restore(request)` | Restore by checkpoint or version |
| `optimize(request=None)` | Compact and optionally prune old history |
| `stats(*, scope)` | Return store statistics |
| `export(destination)` | Export a store |
| `import_store(source)` | Stage an import from an exported store |

## Errors

The package exposes a small explicit exception hierarchy:

| Exception | Meaning |
|---|---|
| `MnemixError` | Base error for the Python package |
| `MnemixCommandError` | The CLI returned a structured failure |
| `MnemixBinaryNotFoundError` | No `mnemix` binary could be resolved |
| `MnemixDecodeError` | CLI output could not be decoded into the expected structure |

## Binary resolution

The client resolves the CLI binary in this order:

1. `MNEMIX_BINARY`
2. a bundled wheel binary, if present
3. `mnemix` on `PATH`

This makes it easy to use published wheels, local development builds, or custom binaries in tests and integrations.

## pip vs pipx

- Use `pip install mnemix` when you want the Python package as a library dependency.
- Use `pipx install mnemix` when you mainly want the `mnemix` CLI available on your shell.

## Design notes

- The Python package is a wrapper layer, not a second implementation.
- JSON mode is the compatibility boundary between Python and the CLI.
- The typed request and response models mirror the CLI response contract so changes stay explicit.

## Host integrations

For real agent hosts, the Python client is the foundation rather than the full
integration surface. Host-specific policy should live in an adapter layer that
decides:

- when to recall memory
- which memory kinds to store
- when to checkpoint
- when to skip writeback

See [Host Adapters](/guide/host-adapters) for the workflow-specific adapter
patterns currently included in the repository.

See [Policy Runner](/guide/policy-runner) for the layer that decides when
memory actions are recommended or required at workflow checkpoints.
