# Mnemix Versioning and Restore

**Status:** draft implementation guidance

## Core semantics

- `history` and `versions` are inspection tools.
- `checkpoint` creates a stable named reference to the current memories-table version.
- `checkout` is an internal storage operation for viewing historical state.
- `restore` creates a new current head version from a historical version or checkpoint.

Mnemix treats that final distinction as product-critical:

- `checkout` does not redefine current history semantics.
- `restore` does create a new current state and leaves prior versions inspectable.

Restore is not a single atomic storage transaction. Mnemix resolves the target, performs the restore, and then refreshes the latest table handle so normal reads continue against the new head.

## User-facing restore contract

Restore accepts either:

- a checkpoint name
- a raw version number

Before restore runs, the conservative default policy creates an automatic pre-restore checkpoint for the current head when that head is not already checkpointed.

Restore results report:

- the previous current version
- the historical version that was restored
- the new current head version created by the restore
- any automatic pre-restore checkpoint that was created

## Checkpoints and tags

Mnemix implements checkpoints on top of Lance tags for the memories table.

That means checkpoints are:

- human-readable
- stable references to historical state
- part of retention safety rules

Checkpoint metadata is also persisted in the dedicated checkpoints table so CLI and adapter surfaces can render descriptions consistently.

## Safety defaults

- restore auto-creates a pre-restore checkpoint by default
- optimize auto-creates a pre-optimize checkpoint by default
- prune behavior is opt-in
- tagged historical versions are protected from routine cleanup

## CLI guidance

Use `history` or `versions` before destructive operations.

Recommended flow:

1. inspect versions
2. create an explicit checkpoint for a user-meaningful milestone when needed
3. run restore or optimize
4. inspect the new head version after the operation
