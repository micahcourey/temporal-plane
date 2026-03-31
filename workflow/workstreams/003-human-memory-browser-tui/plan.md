# Plan: Human Memory Browser Tui

## Summary

Build a narrow v1 terminal UI on top of the existing Rust CLI and store access
layers. The first release should focus on browsing recent and pinned memories,
running text search, and inspecting full memory details, while keeping the
architecture small enough for later operator actions.

## Scope Analysis

### Affected Areas

| Area | Changes Required |
|------|-----------------|
| `mnemix-core` | Add a storage-agnostic browse/list contract only if the TUI needs shared semantics that do not already exist |
| `mnemix-lancedb` | Wire existing memory-list behavior into any shared browse surface and preserve scope-aware filtering |
| `mnemix-cli` | Add `ui` command, terminal runtime, state model, rendering, and slim search filter inputs |
| Output / data loading | Reuse existing memory formatting and loading logic where possible without shelling out |
| Tests | Cover browse modes, empty states, search flow, and detail rendering |
| Docs and workflow artifacts | Explain the TUI, keybindings, and its relationship to the static CLI |

### Affected Layers

- [x] Documentation and planning artifacts
- [ ] Core browse/list contract
- [ ] Backend browse/list implementation
- [ ] CLI surface
- [ ] TUI runtime and rendering
- [ ] Search and detail interaction model
- [ ] Tests and verification

## Technical Design

### Current Baseline

Today `mnemix` already has:

- `search` for text-first retrieval
- `show` for loading one record by id
- `pins` for pinned-memory listing
- `history` and related store-inspection commands

It does not yet have:

- an interactive terminal UI
- a storage-agnostic browse/list trait for recent-memory browsing
- a unified human flow for switching between browse, pinned, and search views

The LanceDB backend already exposes `list_memories(...)` and
`list_pinned_memories(...)`, which is promising for v1, but the workstream
should not let the TUI hard-code a LanceDB-only contract if that behavior needs
to become product-visible.

### Proposed Additions

```text
crates/mnemix-core/src/traits.rs                   # optional browse/list trait if v1 needs shared semantics
crates/mnemix-core/src/...                         # any supporting query or result types
crates/mnemix-lancedb/src/backend.rs               # implementation of shared browse/list behavior
crates/mnemix-cli/src/cli.rs                       # `ui` subcommand
crates/mnemix-cli/src/cmd/ui.rs                    # command entrypoint
crates/mnemix-cli/src/tui/                         # app state, data loading, rendering, events
crates/mnemix-cli/tests/ or src/output/tests/      # targeted interaction and rendering coverage
README.md / docs/                                  # keyboard usage and positioning
workflow/workstreams/003-human-memory-browser-tui/
```

### Runtime Approach

- Reuse the `mnemix-workflow` precedent and prefer `ratatui` plus `crossterm`
  for layout, rendering, and keyboard input
- Keep the TUI state, data loading, and rendering layers separate so future
  operator actions do not require a rewrite
- Prefer direct Rust API calls over shelling out to existing CLI subcommands

### V1 Layout

- left pane: browse mode and scope selection
- center pane: memory result list for the current mode
- right pane: memory detail preview
- footer: key hints, current scope, and active query or mode

### V1 Modes

- Recent: browse the latest stored memories for a scope or the full store
- Pinned: inspect the highest-signal pinned memories quickly
- Search: enter a text query, optionally apply separate `from` / `to` date
  fields, and browse matching records

### V1 Interactions

- move through panes and result lists from the keyboard
- switch scopes or clear scope filters
- enter and rerun search queries
- set or clear separate search date fields such as `from` and `to`
- scroll through full memory detail content

## Design Constraints

- Keep v1 read-focused and do not require inline editing
- Preserve `mnemix-core` boundaries by adding product contracts only when the
  TUI actually needs them
- Keep the static CLI as the automation and scripting layer
- Prefer readable memory detail rendering over ambitious terminal polish

## Implementation Slices

### Slice 1: Workstream And Data Contract

- Create the durable repo-native workstream for the TUI effort
- Decide whether recent-memory browsing belongs in a shared core trait or can
  stay behind existing CLI/backend boundaries
- Define the v1 browse modes and memory-detail payload requirements

### Slice 2: Interactive Entry Point

- Add `mnemix ui`
- Set up terminal initialization, teardown, and event-loop boundaries
- Define the core TUI app state and pane-focus model

### Slice 3: Browse And Search Flows

- Implement recent-memory browsing
- Implement pinned-memory browsing
- Implement search query entry and result navigation
- Implement separate date fields for narrowing search results
- Implement scope filtering and empty/error states

### Slice 4: Detail View And Readability

- Render full memory details cleanly in a preview pane
- Show metadata such as kind, scope, tags, pin state, timestamps, and source
  fields
- Support scrolling and clear focus cues without relying only on color

### Slice 5: Docs And Validation

- Document the command, keybindings, and v1 scope
- Add targeted tests for browse modes, empty stores, and memory-detail
  inspection
- Run the relevant CLI and repo verification gates before handoff

## Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| The TUI leans on LanceDB-only helpers and bypasses product boundaries | High | Medium | Promote browse/list semantics into `mnemix-core` if the UI needs a stable shared contract |
| Search input handling and layout polish expand the scope | Medium | Medium | Keep v1 controls narrow and keyboard-first |
| Large memory detail payloads become hard to read | Medium | Medium | Prioritize clear sectioning, scrolling, and metadata labeling over fancy rendering |
| The UI duplicates static CLI logic awkwardly | Medium | Medium | Treat the static CLI as the scriptable layer and the TUI as the interactive inspection layer |

## References

- `mnemix-workflow/workflow/workstreams/005-interactive-tui-mode/plan.md`
- `crates/mnemix-cli/src/cli.rs`
- `crates/mnemix-cli/src/cmd/search.rs`
- `crates/mnemix-cli/src/cmd/show.rs`
- `crates/mnemix-lancedb/src/backend.rs`
