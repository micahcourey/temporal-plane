# Feature Spec: Human Memory Browser Tui

## Summary

Add a browse-first interactive terminal mode to `mnemix` so humans can search,
filter, and inspect an agent's memory store without stitching together one-off
CLI commands.

## Problem

`mnemix` already has useful static commands such as `search`, `show`, `pins`,
and `history`, but the current experience is still command-by-command and
record-by-record. That makes it slower for a human operator to scan recent
memories, pivot across scopes, inspect pinned context, and drill into full
memory details while debugging or reviewing an agent's memory state.

The `mnemix-workflow` TUI showed that a narrow, browse-first terminal UI can
make a file-backed system feel much more approachable without replacing the
existing CLI. `mnemix` needs the same kind of human-facing surface for local
memory stores.

## Users

- Primary persona: a human operator or developer inspecting an agent's local
  Mnemix store during coding, debugging, or workflow review
- Secondary persona: a maintainer validating memory quality, pinning behavior,
  and searchability across scopes

## Goals

- Add a full-screen interactive mode such as `mnemix ui`
- Let users browse memories without needing a text query every time
- Make search, pinned-memory inspection, and full-detail review faster than the
  current static commands alone
- Keep v1 read-focused and keyboard-first
- Reuse the existing Rust CLI foundation and product semantics instead of
  creating a separate app

## Non-Goals

- Replacing the existing static CLI commands
- Turning v1 into a full editor for memory records
- Building a terminal chat shell around Mnemix
- Shipping a web UI or Studio surface as part of this workstream
- Reworking retrieval ranking or storage internals beyond what the TUI needs

## User Value

Humans get a practical terminal-native way to understand what an agent has
stored: they can scan recent or pinned memories, run searches, switch scopes,
and inspect full memory records and metadata in one place.

## Functional Requirements

- The CLI should expose an interactive mode such as `mnemix ui`
- The TUI should support at least three read flows in v1:
  - browse recent memories
  - browse pinned memories
  - run text search and inspect matching results
- The search flow should support separate date fields for narrowing results,
  such as `from` and `to`, instead of requiring natural-language date parsing
- The TUI should let the user filter or pivot by scope where the store supports
  it
- The TUI should show a readable detail view for the selected memory, including
  title, kind, scope, summary, detail, tags, pin state, timestamps, and source
  metadata when present
- The TUI should support lightweight keyboard-driven navigation between browse
  modes, result lists, and detail panes
- The TUI should reuse existing Rust-side memory loading and search logic
  instead of shelling out to the CLI binary
- If a browse-first memory listing contract is missing in shared product
  surfaces, this workstream should add one without leaking LanceDB-specific
  details into `mnemix-core`
- The TUI should degrade clearly for empty stores, missing scopes, or invalid
  store paths

## Constraints

- `mnemix-core` must remain storage-agnostic
- The TUI should live inside the Rust CLI surface, not as a separate product
- Python remains a wrapper layer and should not become a second TUI
- The interface must remain understandable without color-only cues
- V1 should stay browse-first so the feature ships as a focused human tool, not
  an overgrown terminal platform

## Success Criteria

- A user can launch an interactive UI against a local store and browse recent
  and pinned memories without typing raw CLI commands repeatedly
- A user can run a text search and inspect full memory details in the same
  session, including optional date-range filtering through separate date fields
- The TUI makes scope-aware store inspection clearly faster and easier than the
  current static command flow
- The implementation leaves room for future actions such as pinning, opening
  history, or launching restore/checkpoint flows without forcing those into v1

## Risks

- The TUI could accidentally depend on backend-only helpers instead of a stable
  product contract for browse/list behavior
- Terminal input and large-detail rendering could expand scope if search-entry
  and scrolling are not kept simple
- V1 could become an editing surface too early and delay a useful browse-first
  release

## References

- `mnemix-workflow/workflow/workstreams/005-interactive-tui-mode/spec.md`
- `crates/mnemix-cli/src/cli.rs`
- `crates/mnemix-lancedb/src/backend.rs`
