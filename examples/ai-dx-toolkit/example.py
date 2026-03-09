"""AI DX Toolkit adapter example.

Run with:
    tp binary on PATH (cargo build --release, then add target/release to PATH)
    pip install -e ../../python
    python example.py
"""

import sys
from pathlib import Path

# Add the adapter directory so temporal_plane_adapter is importable
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
