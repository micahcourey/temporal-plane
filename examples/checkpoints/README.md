# Checkpoints Example

This example shows how to use checkpoints and restore flows to make memory
changes reversible during larger agent workflows.

Checkpoints are useful before:

- bulk imports
- migration experiments
- agent runs that may write many memories
- cleanup or pruning work

## CLI Walkthrough

Initialize a store and add a baseline memory:

```bash
mnemix --store .mnemix init

mnemix --store .mnemix remember \
  --id memory:baseline \
  --scope repo:mnemix \
  --kind decision \
  --title "Baseline architecture" \
  --summary "The repository keeps storage details behind backend layers." \
  --detail "This baseline entry exists so the later restore step has a stable known state to return to." \
  --importance 90 \
  --pin-reason "Baseline for restore example"
```

Create a checkpoint before an experimental run:

```bash
mnemix --store .mnemix checkpoint \
  --name before-agent-import \
  --description "Safe point before staging agent-written memories"
```

Write an experimental memory that you may want to discard:

```bash
mnemix --store .mnemix remember \
  --id memory:temporary-experiment \
  --scope repo:mnemix \
  --kind observation \
  --title "Temporary experiment" \
  --summary "This entry only exists to demonstrate restore semantics." \
  --detail "If the experiment is not useful, restore the store to the prior checkpoint." \
  --importance 40
```

Inspect available versions:

```bash
mnemix --store .mnemix versions --limit 10
```

Restore the store to the checkpoint:

```bash
mnemix --store .mnemix restore --checkpoint before-agent-import
```

Verify the temporary memory is gone and the baseline memory remains:

```bash
mnemix --store .mnemix show --id memory:baseline
mnemix --store .mnemix show --id memory:temporary-experiment
```

The second `show` call should fail after a successful restore.

## Operational Guidance

Use checkpoints as guardrails, not as a substitute for judgment. The preferred
pattern is:

1. checkpoint before risky or high-volume writes
2. run the agent or import flow
3. inspect the resulting memories
4. restore only if the new state is low-signal or incorrect

This keeps Mnemix safe by default while still allowing aggressive experimentation.
