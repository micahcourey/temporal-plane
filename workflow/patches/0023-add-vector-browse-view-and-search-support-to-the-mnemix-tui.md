---
status: open
summary: "Add vector-aware browse, detail, and search support to the Mnemix TUI."
updated: "2026-04-02"
---

# Patch: Add vector-aware browse, detail, and search support to the Mnemix TUI

## Summary

Extend `mnemix ui` so the human-first TUI can inspect and use the vector
retrieval layer that landed in the backend and static CLI. Scope: add a
vector-aware search mode selector for lexical, semantic-only, and hybrid
retrieval where supported; surface store vector readiness and coverage inside
the TUI; and show enough result/detail context that an operator can tell why a
memory appeared and whether the store is actually vector-ready. Acceptance
criteria: the TUI can expose vector status without dropping to a separate CLI
command, search can run in vector-capable modes when the store/runtime allows
it, unsupported modes fail clearly instead of silently degrading, and the
detail experience makes semantic or hybrid matches inspectable.

## Reason

The current PR adds vector support to the LanceDB backend and introduces
non-TUI CLI commands such as `vectors show`, `vectors enable`, and
`vectors backfill`, but `mnemix ui` remains lexical-only. The current TUI still
offers only `Recent`, `Pinned`, and `Search` browse modes, and its search path
builds a default lexical `SearchQuery` with no retrieval-mode control. That
creates a product gap: the terminal UI is now the main human-facing browse
surface, but it cannot inspect vector readiness or exercise semantic and hybrid
retrieval even when the backend supports them.

## Scope

- Add vector-aware search controls to the TUI so an operator can select
  `lexical`, `semantic`, or `hybrid` retrieval for search flows
- Surface store-level vector status in the TUI, including:
  vectors enabled, model, dimensions, provider availability, coverage, and
  vector-index readiness
- Make unsupported vector modes explicit in the UI when the current store or
  runtime cannot run them
- Show retrieval provenance in search results and/or detail view so semantic
  and hybrid matches are explainable to a human operator
- Preserve the existing browse-first TUI feel rather than turning the interface
  into a generic configuration dashboard
- Update docs to explain the new TUI behavior and its relationship to the
  static `vectors` commands
- Add targeted tests for mode selection, unavailable-vector states, and
  explanation rendering

## Implementation Notes

- The current TUI implementation lives under `crates/mnemix-cli/src/tui/`
  rather than a separate crate, so this patch should stay inside that existing
  boundary unless a shared contract is clearly needed
- `crates/mnemix-cli/src/tui/data.rs` currently builds search queries with
  `SearchQuery::new(...)`, which defaults to lexical retrieval; this patch will
  likely need to switch to the retrieval-mode-aware constructor path
- `crates/mnemix-cli/src/tui/state.rs` will need explicit UI state for the
  selected retrieval mode and for any status or capability snapshot shown in
  the interface
- `crates/mnemix-cli/src/tui/render.rs` should make vector behavior legible
  without overwhelming the browse-first layout; lightweight badges, status
  lines, or a compact store-status panel are likely a better fit than a large
  new pane
- Reuse existing backend status surfaces such as `vector_status()` instead of
  inventing parallel TUI-only detection logic
- If per-result explanation data is insufficient for the TUI, promote only the
  minimal shared contract needed to keep semantic and hybrid matches
  inspectable
- Preserve a clean no-vector experience for stores that remain purely lexical

## Validation

- `cargo test -p mnemix-cli`
- `cargo test -p mnemix-lancedb semantic_only_search`
- `cargo test -p mnemix-lancedb recall_semantic_only_marks_semantic_matches`
- `cargo test -p mnemix-lancedb recall_hybrid_marks_hybrid_matches`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- Manual TUI verification:
  open `mnemix ui`, confirm vector status is visible, confirm lexical/semantic/
  hybrid search modes behave correctly, and confirm unsupported modes show an
  explicit message instead of silently acting lexical

## References

- `crates/mnemix-cli/src/cmd/ui.rs`
- `crates/mnemix-cli/src/tui/mod.rs`
- `crates/mnemix-cli/src/tui/data.rs`
- `crates/mnemix-cli/src/tui/state.rs`
- `crates/mnemix-cli/src/tui/render.rs`
- `crates/mnemix-cli/src/cmd/vectors.rs`
- `crates/mnemix-core/src/query.rs`
- `crates/mnemix-lancedb/src/backend.rs`
- `workflow/workstreams/003-human-memory-browser-tui/plan.md`
- `workflow/workstreams/002-optional-lancedb-vector-retrieval-layer/plan.md`
