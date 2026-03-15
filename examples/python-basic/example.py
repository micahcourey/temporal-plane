"""Basic Mnemix Python binding example.

Run with:
    mnemix binary on PATH (cargo build --release, then add target/release to PATH)
    pip install -e ../../python
    python example.py
"""

from pathlib import Path

from mnemix import Mnemix, RememberRequest
from mnemix.models import CheckpointRequest

store = Path("/tmp/mnemix-python-basic-example")
tp = Mnemix(store=store)

# 1. Initialise the store
tp.init()
print("Store initialised.")

# 2. Remember a few records
tp.remember(RememberRequest(
    id="mem-001",
    scope="example-project",
    kind="observation",
    title="Python binding works",
    summary="Confirmed the Python binding can init and remember.",
    detail="Created a store at /tmp, called init(), then remember().",
    importance=70,
    tags=["python", "binding"],
))

tp.remember(RememberRequest(
    id="mem-002",
    scope="example-project",
    kind="decision",
    title="Use CLI JSON surface as binding boundary",
    summary="Decided to wrap the CLI --json surface rather than build FFI.",
    detail="FFI would require a stable Rust application API not yet available.",
    importance=90,
    pin_reason="Core architectural decision for all Python consumers.",
))

# 3. Search
results = tp.search("binding", scope="example-project")
print(f"Search returned {len(results)} result(s):")
for m in results:
    print(f"  [{m.kind}] {m.title} (importance={m.importance})")

# 4. Recall with layered context
context = tp.recall()
print(
    f"Recall — pinned={len(context.pinned_context)}, "
    f"summaries={len(context.summaries)}, "
    f"archival={len(context.archival)}"
)

# 5. Checkpoint
cp = tp.checkpoint(CheckpointRequest(name="example-v1", description="End of example run"))
print(f"Checkpoint '{cp.name}' at version {cp.version}")

# 6. Stats
stats = tp.stats()
print(f"Total memories: {stats.total_memories}, versions: {stats.version_count}")
