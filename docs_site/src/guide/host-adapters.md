# Host Adapters

Mnemix keeps its product API generic. Host-specific memory policy belongs in
adapters.

That split matters because different hosts need different behavior:

- a coding agent should recall implementation context and store decisions,
  procedures, and pitfalls
- a chat assistant should recall user context and store preferences or stable facts
- a CI bot should checkpoint before risky automation and preserve failures or fixes
- a review tool should recall conventions and store reusable review rules or recurring issues

## Why adapters exist

The Python client is intentionally thin. It exposes generic product operations
such as `remember`, `recall`, `checkpoint`, and `restore`, but it does not
decide:

- when a host should recall memory
- which recall depth fits the workflow
- which memory kinds are appropriate
- what should be pinned
- when to skip writeback

Those are host workflow decisions, not storage concerns.

## Current adapter module

The repository includes a host adapter module at:

```text
adapters/
```

The path is historical, but the module now exposes four workflow-specific
adapters:

| Adapter | Workflow |
|---|---|
| `CodingAgentAdapter` | implementation tasks, architecture decisions, reusable procedures, pitfalls |
| `ChatAssistantAdapter` | conversational context, user preferences, durable facts |
| `CiBotAdapter` | automated runs, pre-run checkpoints, failures, remediation steps |
| `ReviewToolAdapter` | review-time conventions, reusable review rules, recurring issues |

## Coding agent workflow

Coding agents usually need memory at task boundaries.

Typical pattern:

1. start a non-trivial task
2. recall pinned context and summaries for the repo and task topic
3. perform the task
4. write back only durable outcomes

```python
from pathlib import Path
from adapters import CodingAgentAdapter

adapter = CodingAgentAdapter(store=Path(".mnemix"))
adapter.ensure_store()

context = adapter.start_task(
    scope=adapter.repo_scope("mnemix"),
    task_title="Add host-specific adapter docs",
    mode="deep",
)
print(context.prompt_preamble)
```

Use coding-agent writeback for:

- design decisions
- reusable procedures
- repeated implementation pitfalls

The coding adapter also exposes:

- scope helpers with `repo_scope(...)`, `workspace_scope(...)`, `session_scope(...)`, and `task_scope(...)`
- targeted search with `search_memory(...)`
- full memory inspection with `load_memory(...)`
- typed classification with `classify_outcome(...)`
- policy-driven writeback with `store_outcome(...)`
- explicit pin and history review through task-start context assembly
- pre-change checkpoints with `checkpoint_before_risky_change(...)`
- version inspection and restore
- optimize, export, and staged import helpers

### CodingAgentAdapter API

| Method | Purpose |
|---|---|
| `repo_scope(...)` | Build a standard repository scope |
| `workspace_scope(...)` | Build a standard workspace scope |
| `session_scope(...)` | Build a standard session scope |
| `task_scope(...)` | Build a standard task scope |
| `start_task(...)` | Assemble task-start context with recall, pins, and recent history |
| `search_memory(...)` | Run targeted search during implementation work |
| `load_memory(...)` | Inspect one memory in full detail |
| `list_pins(...)` | View pinned memory for the current repo or scope |
| `review_recent_memory(...)` | Inspect recent memory activity |
| `classify_outcome(...)` | Classify a coding outcome as skip/decision/procedure/summary/fact/warning |
| `store_outcome(...)` | Apply classification and write back only durable outcomes |
| `checkpoint_before_risky_change(...)` | Create a safety checkpoint before risky work |
| `list_versions(...)` | Inspect store version history |
| `restore_checkpoint(...)` | Restore to a named checkpoint |
| `restore_version(...)` | Restore to a specific version |
| `optimize_store(...)` | Run compaction and optional pruning |
| `export_snapshot(...)` | Export the current store |
| `stage_import(...)` | Stage an imported store into the current store |
| `store_decision(...)` | Persist a durable design decision |
| `store_procedure(...)` | Persist a reusable coding procedure |
| `store_summary(...)` | Persist a session/task summary |
| `store_fact(...)` | Persist a stable project fact |
| `store_pitfall(...)` | Persist a recurring implementation warning |

## Chat assistant workflow

Chat assistants should usually store much less than coding agents. The main
durable signal is often user preference or stable user-specific facts.

```python
from pathlib import Path
from adapters import ChatAssistantAdapter

adapter = ChatAssistantAdapter(store=Path(".mnemix"))
adapter.ensure_store()

context = adapter.prepare_reply(
    scope="user:demo",
    user_message="Please keep answers concise.",
)
print(context.prompt_preamble)
```

Use chat writeback for:

- user preferences
- stable factual context that should persist across sessions

Avoid storing transient turn-by-turn summaries.

### ChatAssistantAdapter API

| Method | Purpose |
|---|---|
| `prepare_reply(...)` | Recall prompt-ready user and conversation context |
| `store_preference(...)` | Persist a durable user preference |
| `store_fact(...)` | Persist a stable user- or task-relevant fact |

## CI bot workflow

CI bots care more about safe automation and repeatable remediation than about
general conversational memory.

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
```

Use CI writeback for:

- recurring failures
- runbook-quality fixes
- operational warnings that should influence later runs

### CiBotAdapter API

| Method | Purpose |
|---|---|
| `prepare_run(...)` | Recall CI context and optionally create a pre-run checkpoint |
| `record_failure(...)` | Persist a recurring CI failure mode |
| `record_fix(...)` | Persist a reusable remediation procedure |

## Review tool workflow

Review tools need reusable review knowledge rather than generic task history.

```python
from pathlib import Path
from adapters import ReviewToolAdapter

adapter = ReviewToolAdapter(store=Path(".mnemix"))
adapter.ensure_store()

context = adapter.prepare_review(
    scope="repo:mnemix",
    review_topic="adapter verification expectations",
)
print(context.prompt_preamble)
```

Use review writeback for:

- reusable review rules
- recurring quality issues
- project-specific review conventions

### ReviewToolAdapter API

| Method | Purpose |
|---|---|
| `prepare_review(...)` | Recall project conventions and review context |
| `record_review_rule(...)` | Persist a reusable review rule or policy |
| `record_recurring_issue(...)` | Persist a recurring review finding or quality issue |

## Design rule

Keep the base Mnemix API generic. Put timing, judgment, and workflow policy in
the adapter for the specific host.

That gives each host the right memory behavior without turning the product API
into a grab bag of use-case-specific flags.

## Ecosystem template

If you want a more comprehensive coding-agent adapter and reusable memory-policy
template, see the `mnemix-context` universal Mnemix template:

[mnemix-context/templates/universal/mnemix](https://github.com/micahcourey/mnemix-context/tree/main/templates/universal/mnemix)
