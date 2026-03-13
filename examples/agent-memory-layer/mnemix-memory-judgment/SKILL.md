---
name: mnemix-memory-judgment
description: Use this skill when a repository uses Mnemix as a local memory layer and the agent needs to decide when to recall prior context or write back durable project knowledge. Helps with selective memory use for coding agents, project conventions, architecture decisions, reusable procedures, and avoiding noisy low-value memory writes.
license: Apache-2.0
metadata:
  author: mnemix
  version: "0.1"
---

# Mnemix Memory Judgment

Use this skill when working in a repository that treats Mnemix as selective
project memory rather than a task log.

## Workflow

1. Classify the task as trivial or non-trivial.
2. For non-trivial work, recall memory for the repository scope and task topic.
3. Prefer pinned context and summaries before loading broader archival context.
4. Complete the task using recalled context only when it materially affects the work.
5. At the end, decide whether the task produced durable knowledge worth storing.
6. Write back only durable facts, decisions, procedures, preferences, or repeated pitfalls.
7. Skip writeback when the outcome is transient, obvious from the code, or low-signal.

## Recall Guidance

Recall is usually appropriate when the task:

- spans multiple steps or files
- touches architecture, conventions, or recurring workflows
- is likely to repeat later
- has prior decisions or pitfalls that could change the implementation

For trivial edits, recall is optional unless the user explicitly asks for memory
usage.

## Writeback Guidance

Prefer one compact memory over several noisy ones.

Write a memory only if a future agent would benefit from knowing:

- what decision was already made
- what procedure should be reused
- what preference should be preserved
- what mistake should be avoided

Do not store:

- raw transcripts
- command-by-command activity logs
- temporary debugging notes
- speculative ideas that were not adopted
- summaries that are already obvious from the final code or docs

## Resources

See [the reference guide](references/REFERENCE.md) for:

- a more explicit decision rubric
- good and bad memory examples
- suggested recall and writeback prompts
