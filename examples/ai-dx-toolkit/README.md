# Example: AI DX Toolkit Adapter

This example shows how to use the AI DX Toolkit Temporal Plane adapter to record
agent memory events and retrieve layered context.

## Prerequisites

- The `tp` binary on `PATH` (build with `cargo build --release`)
- Python 3.11+
- `temporal-plane` installed: `cd python && pip install -e .`
- The adapter module: available at `adapters/ai-dx-toolkit/temporal_plane_adapter.py`

## Running the Example

```bash
cargo build --release
export PATH="$PWD/target/release:$PATH"

pip install -e python/

cd examples/ai-dx-toolkit
python example.py
```

## example.py

```python
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent.parent / "adapters" / "ai-dx-toolkit"))

from temporal_plane_adapter import TemporalPlaneAdapter

store = Path("/tmp/tp-adapter-example")
adapter = TemporalPlaneAdapter(store=store)

# Initialise
adapter.ensure_store()
print("Store ready.")

# Record observations from a toolkit run
adapter.record_observation(
    id="obs-001",
    scope="my-agent",
    title="Added httpx dependency",
    summary="httpx added to pyproject.toml for async HTTP support.",
    detail="Chose httpx over requests for async-first design and type safety.",
    tags=["dependencies", "http"],
    source_tool="package-manager",
)

# Record a pinned architectural decision
adapter.record_decision(
    id="dec-001",
    scope="my-agent",
    title="Use Rust for core logic",
    summary="Core storage and domain logic implemented in Rust.",
    detail="Full rationale: performance, LanceDB SDK, and type safety.",
    pin_reason="Core architectural choice — survives session resets.",
    importance=95,
)

# Create a session checkpoint before a bulk operation
cp = adapter.create_session_checkpoint(
    "sess-20260308",
    description="Pre-import checkpoint",
)
print(f"Checkpoint '{cp.name}' at version {cp.version}")

# Fetch layered context for prompt injection
context = adapter.fetch_context(scope="my-agent", query="architecture")
print(f"Context entries: {len(context)}")
for entry in context:
    print(f"  [{entry.layer}] {entry.memory.title}")

# Store stats
stats = adapter.get_stats(scope="my-agent")
print(f"Memories: {stats.total_memories}, pinned: {stats.pinned_memories}")
```

## Expected Output

```
Store ready.
Checkpoint 'sess-20260308' at version 3
Context entries: 2
  [pinned_context] Use Rust for core logic
  [summary] Added httpx dependency
Memories: 2, pinned: 1
```

## Design Notes

- The adapter depends only on the `temporal_plane` public Python API.
- No LanceDB-specific assumptions are made in the adapter layer.
- Pinned decisions appear in `pinned_context` and are favored in recall.
- Checkpoints created before bulk operations enable safe restore if needed.
