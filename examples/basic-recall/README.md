# Basic Recall Example

This example shows the smallest useful Mnemix loop for an agent-enabled coding
project:

1. initialize a local store
2. write a few durable memories
3. recall layered context before a new task
4. inspect the exact record that was surfaced

The goal is not to persist every step the agent takes. The goal is to keep only
the project knowledge that should survive beyond the current turn.

## When This Pattern Fits

Use this flow when the agent is starting a non-trivial task and needs prior
project context such as:

- architectural decisions
- recurring commands or workflows
- user preferences
- repeated failure modes

Skip memory writes for trivial edits or information already obvious from the
current diff.

## CLI Walkthrough

Initialize a store:

```bash
mnemix --store .mnemix init
```

Record a pinned decision that should consistently influence future work:

```bash
mnemix --store .mnemix remember \
  --id memory:core-boundary \
  --scope repo:mnemix \
  --kind decision \
  --title "Keep Rust as the source of truth" \
  --summary "Bindings and adapters must wrap Rust behavior rather than reimplement it." \
  --detail "New integrations should call the Rust-owned product surface. Python and host adapters stay thin and should not duplicate product logic." \
  --importance 95 \
  --tag architecture \
  --tag rust \
  --pin-reason "Applies to every integration example"
```

Record a reusable procedure:

```bash
mnemix --store .mnemix remember \
  --id memory:agent-memory-policy \
  --scope repo:mnemix \
  --kind procedure \
  --title "Use recall before multi-step agent tasks" \
  --summary "Run recall before prompt assembly and write back only durable outcomes after the task." \
  --detail "Treat memory as selective project context. Do not store transient chain-of-thought, obvious diff summaries, or one-off tool noise." \
  --importance 85 \
  --tag agents \
  --tag workflow
```

Recall context for a new integration task:

```bash
mnemix --store .mnemix recall \
  --scope repo:mnemix \
  --text "agent integration" \
  --disclosure-depth summary_then_pinned \
  --limit 10
```

Inspect the specific record if the agent or developer wants more detail:

```bash
mnemix --store .mnemix show --id memory:core-boundary
```

## What To Expect

`recall` returns layered results:

- `pinned_context`: small, always-relevant signal
- `summaries`: compact reusable context
- `archival`: broader history when deeper expansion is requested

That matches the product model described in
[`docs/mnemix-plan-v3.md`](/Users/micah/Projects/mnemix/.worktrees/codex-mnemix-examples/docs/mnemix-plan-v3.md):
load the smallest high-value context first, then expand only when needed.

## Integration Hint

In a real agent host, this example becomes:

```text
task starts
-> mnemix recall(scope, task_text)
-> inject pinned + summary context into the agent prompt
-> run the agent
-> persist only durable learnings with remember
```
