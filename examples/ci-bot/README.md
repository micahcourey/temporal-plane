# CI Bot Example

This example shows how to use [`CiBotAdapter`](/Users/micah/Projects/mnemix/adapters/ci_bot_adapter.py)
for automated CI workflows that need memory about failures, fixes, and safe
pre-run checkpoints.

## Typical flow

1. initialize the store
2. prepare run context for the pipeline
3. create a checkpoint before risky automation
4. run the pipeline
5. store recurring failures or reusable fixes

## Example

```python
from pathlib import Path
from adapters import CiBotAdapter

adapter = CiBotAdapter(store=Path(".mnemix"))
adapter.ensure_store()

run = adapter.prepare_run(
    scope="repo:mnemix",
    run_id="42",
    pipeline="publish-python",
)
print(run.bundle.prompt_preamble)
print(run.checkpoint)

adapter.record_failure(
    memory_id="memory:ci-failure-qemu",
    scope="repo:mnemix",
    title="QEMU-based linux preflight fails on arm macs",
    summary="The emulated preflight path is unreliable on local arm macOS hosts.",
    detail="Prefer skipping that local preflight path when the release workflow already validates the linux build elsewhere.",
)

adapter.record_fix(
    memory_id="memory:ci-fix-preflight",
    scope="repo:mnemix",
    title="Use docker writable copy for linux preflight",
    summary="Build the preflight image from a writable copy to avoid read-only mount issues.",
    detail="The writable-copy workaround avoids permission failures during local release verification.",
)
```
