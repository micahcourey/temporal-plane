# Tasks: Human Memory Browser Tui

## Workstream Goal

Create a focused human-facing TUI for `mnemix` that makes it easy to browse,
search, and inspect a local memory store without replacing the existing static
CLI.

## Execution Slices

### Planning And Scope

- [x] Create a repo-native workstream for the human memory browser TUI
- [x] Confirm the v1 command shape and naming, such as `mnemix ui`
- [x] Decide whether recent-memory browsing requires a new shared core trait or
  can reuse existing boundaries cleanly
  Current decision: promote browse/list behavior behind a shared
  `mnemix-core::BrowseBackend` contract so human-facing product surfaces do not
  depend directly on LanceDB-specific helpers.

### Data Access And Contracts

- [x] Add any storage-agnostic browse/list contract needed for recent-memory
  browsing
- [x] Implement the shared browse/list behavior in the LanceDB backend if
  promoted into product surfaces
- [x] Define the memory-detail payload the TUI needs for summary, detail, tags,
  pin state, and source metadata

### Interactive CLI Surface

- [x] Add the interactive CLI entrypoint and terminal lifecycle handling
- [x] Add pane focus, selection, and keyboard navigation state
- [x] Add empty-store, invalid-store, and missing-scope handling

### Browse, Search, And Detail UX

- [x] Implement recent-memory browsing
- [x] Implement pinned-memory browsing
- [x] Implement text search entry and result navigation
- [x] Implement separate `from` / `to` date fields for search filtering
- [x] Implement the full memory-detail preview with scrolling

### Documentation And Verification

- [x] Document the TUI command, keybindings, and v1 scope
- [x] Add targeted tests for browse modes, search flow, and detail rendering
- [x] Run the relevant CLI and repo verification gates

## Validation Checklist

- [x] Keep `STATUS.md` aligned with the true execution state
- [x] Verify the TUI remains keyboard-first and understandable without color
  alone
- [x] Verify date-filtered search works through explicit date fields rather than
  fuzzy natural-language parsing
- [x] Verify scope filtering and empty-state behavior explicitly
- [x] Preserve the existing static CLI behavior for scripting and automation

## Notes

- This workstream is intentionally browse-first. Inline editing, pin/unpin
  actions, restore flows, and other operator mutations can follow once the
  inspection surface is stable.
- The `mnemix-workflow` TUI is a useful implementation precedent, but `mnemix`
  needs memory-store-specific browse and detail semantics rather than workflow
  artifact semantics.
