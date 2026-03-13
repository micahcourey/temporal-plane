# Mnemix Usage Instructions

Use Mnemix as a selective memory layer for non-trivial agent work in this
repository.

## When To Recall

Run memory recall before starting a task when any of the following are true:

- the task is multi-step
- prior architectural decisions may affect the implementation
- the repo has reusable commands, workflows, or pitfalls relevant to the task
- user preferences or project conventions are likely to matter

For trivial one-file edits or purely local refactors, recall is optional.

## When To Remember

Write memory only when the result is likely to matter in a future session.

Good candidates:

- durable project facts
- design decisions and tradeoffs
- reusable procedures
- user preferences
- recurring failure modes and their fixes

Do not write memory for:

- obvious facts already visible in code or docs
- transient planning notes
- low-value activity logs
- one-off tool output
- raw chain-of-thought or verbose internal reasoning

## Task Workflow

Before a non-trivial task:

1. recall memory for the repo scope and task topic
2. use pinned and summary context first
3. open deeper archival context only if needed

After completing a task:

1. decide whether any durable learning was produced
2. if yes, write one or a small number of high-signal memories
3. if no, skip writeback

## Quality Bar

Each stored memory should answer at least one of these questions for a future
agent:

- what should I know before changing this area?
- what decision was already made here?
- what procedure should I follow?
- what mistake should I avoid repeating?

If a proposed memory does not clear that bar, do not store it.
