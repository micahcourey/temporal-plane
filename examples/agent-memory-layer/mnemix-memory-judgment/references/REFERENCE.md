# Reference Guide

This reference expands the main skill with a more explicit rubric for deciding
when to use Mnemix and what to store.

## Decision Rubric

Use recall when at least one of these is true:

- the task is multi-step
- the task affects architecture or project conventions
- earlier work likely created reusable knowledge
- getting the context wrong would cause repeated future cost

Skip recall when:

- the edit is local and obvious
- no prior project knowledge is needed
- opening memory would add more noise than value

Write memory when the resulting information is:

- durable
- reusable
- non-obvious
- likely to matter in a later session

Skip writeback when the output is:

- temporary
- procedural noise from this single run
- directly inferable from the final diff
- still speculative or unresolved

## Good Memory Examples

Good:

- `Decision: The adapter layer must call the Rust-owned Mnemix surface rather than reproducing product logic in Python.`
- `Procedure: Before bulk memory imports, create a checkpoint so low-signal staged memories can be safely rolled back.`
- `Preference: Prefer pinned architectural decisions and summary memories before opening archival results during agent recall.`

## Bad Memory Examples

Bad:

- `Edited three files and ran tests.`
- `Thought about two approaches and picked one after a short discussion.`
- `Opened README, searched the repo, then fixed a typo.`

## Prompt Patterns

Suggested pre-task recall prompt:

```text
Recall memory for this repository and task topic. Return pinned context and
compact summaries first. Expand deeper only if the recalled context is directly
relevant to the task.
```

Suggested post-task writeback prompt:

```text
Evaluate whether this task produced durable project knowledge. Store only
decisions, procedures, preferences, or repeated pitfalls that would matter in a
future session. Skip low-signal summaries and transient notes.
```
