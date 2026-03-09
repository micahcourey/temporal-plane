# Branch lifecycle

**Status:** advanced workflow

Temporal Plane keeps branch-aware storage as an advanced capability.
The v1 product model does not require branch UX in the CLI, but the storage layer can use branches for isolated experiments, staged imports, and clone-oriented recovery flows.

## Branches vs checkpoints

Use a **checkpoint** when you want a stable named version on the current line of history.

Use a **branch** when you want an isolated line of work that may diverge from the current head.

| Use case | Prefer | Why |
|---|---|---|
| Mark a restore point before cleanup or restore | Checkpoint | Stable name on the current timeline |
| Stage an import for review | Branch | Keeps imported rows away from `main` |
| Try an alternate ranking or retrieval experiment | Branch | Lets the experiment diverge safely |
| Protect a meaningful release-like revision | Checkpoint | Fits the existing v1 inspection model |

## Branch creation semantics

Advanced storage backends may create a branch from:

- the latest visible version, or
- an explicitly selected base version.

Branch names are validated and must not use unsafe path traversal patterns.

A branch is represented as:

- a dedicated Lance branch dataset for the memories table, and
- branch metadata describing the parent version.

## Branch deletion semantics

Branch deletion is conservative.

A branch should only be deleted when it still matches its parent version. If the branch has staged changes beyond its base version, deletion should be refused until the caller explicitly handles that divergence.

This keeps experimental and staged work from disappearing silently.

## Import staging workflow

Import staging is the main branch-oriented workflow in Milestone 7.

1. choose or generate a staging branch name
2. create the branch from the current main version
3. read the source store
4. append importable records onto the staging branch only
5. return a result describing the branch and staged record count

This workflow preserves the current main branch while still giving callers a place to inspect imported data.

## Shallow vs deep clone

| Clone kind | Behavior | Best for |
|---|---|---|
| Shallow clone | Copies manifests and lineage cheaply while reusing source data files where supported | Fast local export, temporary analysis, derived workspaces |
| Deep clone | Copies the full table datasets into a new store | Durable backups, offline transfer, isolated recovery rehearsal |

Both clone types are advanced operations and should be surfaced carefully.

## Cleanup and retention

Branches are not part of routine v1 retention cleanup.

That means:

- checkpoint retention rules still protect named checkpoints
- normal optimize and prune flows should stay conservative
- abandoned branches require explicit operator cleanup

## Failure modes

Advanced callers should expect these failure classes:

- duplicate branch names
- invalid branch names
- missing base versions
- branch deletion blocked by staged changes
- clone destination conflicts
- source import path failures

## Relationship to v1 semantics

Milestone 7 does **not** change the stable v1 product model.

Instead it adds internal storage capabilities that support future advanced workflows while keeping:

- storage-specific details out of `temporal-plane-core`
- branch UX out of normal CLI help
- restore and checkpoint semantics unchanged for standard users
