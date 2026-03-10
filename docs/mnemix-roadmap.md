# Mnemix Roadmap

**Status:** planning only  
**Date:** March 8, 2026

This roadmap turns the canonical plan into an implementation sequence, repo scaffold, and milestone breakdown.

## 1. Planning Inputs and Guardrails

This roadmap is derived from:

- [docs/mnemix-plan-v3.md](mnemix-plan-v3.md)
- [docs/lancedb-rust-sdk-agent-guide.md](lancedb-rust-sdk-agent-guide.md)
- [docs/mnemix-coding-guidelines.md](mnemix-coding-guidelines.md)

### Architecture guardrails from the Lance guide

The implementation roadmap assumes:

- Rust is the source of truth.
- `lancedb` handles the main application storage path.
- `lance` is used selectively for lower-level version-control and future advanced workflows.
- tags/checkpoints are required in v1.
- branch awareness must exist in the architecture even if branch UX is deferred.
- retention and cleanup must be conservative and explicit.

### API and repo guardrails from the coding guidelines

The roadmap also assumes these Rust best-practice defaults:

- prefer strong domain types over raw strings, `bool`, or loosely typed option bags
- public structs should keep fields private where possible
- complex construction should use builders
- library crates should use typed errors; binary crates may use app-level aggregation
- public APIs should be documented and example-driven
- names should follow Rust conventions consistently
- public types should implement expected common traits where appropriate
- tests should act as living documentation
- workspace linting and Clippy discipline should be present from the start

---

## 2. Delivery Strategy

The repo should be built in layers, not all at once.

### Build order

1. workspace skeleton and engineering guardrails
2. core domain crate
3. LanceDB backend crate
4. CLI crate
5. Python binding crate/package
6. AI DX Toolkit adapter
7. advanced version-control and branch-aware workflows

### Why this order

This sequence keeps the public architecture clean:

- product semantics are defined before storage details leak outward
- the storage backend can evolve behind traits and typed interfaces
- the CLI validates the product model early
- bindings and adapters wrap the core instead of re-implementing it

---

## 3. Recommended Repo Scaffold

## 3.1 Top-level layout

```text
mnemix/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── rustfmt.toml
├── clippy.toml
├── .editorconfig
├── .gitignore
├── LICENSE
├── README.md
├── CHANGELOG.md
├── deny.toml
├── .github/
│   └── workflows/
│       ├── ci.yml
│       ├── docs.yml
│       └── release.yml
├── crates/
│   ├── mnemix-core/
│   ├── mnemix-lancedb/
│   ├── mnemix-cli/
│   ├── mnemix-types/
│   └── mnemix-test-support/
├── python/
│   ├── pyproject.toml
│   ├── README.md
│   └── mnemix/
├── adapters/
│   └── ai-dx-toolkit/
├── examples/
│   ├── basic-recall/
│   ├── checkpoints/
│   └── import-export/
├── tests/
│   ├── integration/
│   ├── fixtures/
│   └── snapshots/
├── docs/
│   ├── mnemix-plan-v3.md
│   ├── mnemix-roadmap.md
│   ├── lancedb-rust-sdk-agent-guide.md
│   ├── memory-model.md
│   ├── versioning-and-restore.md
│   ├── progressive-disclosure.md
│   ├── export-import.md
│   └── mnemix-coding-guidelines.md
└── scripts/
    ├── check.sh
    └── release.sh
```

## 3.2 Why add `mnemix-types`

A small `types` crate is useful if the project needs:

- shared newtypes and value objects
- serialization contracts shared across CLI, Python, and adapters
- reduced circular dependencies between core and backend

If that feels premature, it can start folded into `mnemix-core` and be extracted later.

## 3.3 Why add `mnemix-test-support`

A dedicated test-support crate keeps:

- fixture creation helpers
- temporary store bootstrapping
- deterministic timestamps/IDs
- snapshot formatting helpers
- shared integration assertions

out of the production crates.

---

## 4. Workspace Configuration Standards

The workspace should be initialized with guardrails immediately.

### Required root files

- `Cargo.toml` with workspace members and shared lint/profile policy
- `rust-toolchain.toml` to pin the toolchain
- `rustfmt.toml` for formatting consistency
- `clippy.toml` for lint expectations
- `deny.toml` for license/advisory/dependency checks
- `.editorconfig` for editor consistency
- `CHANGELOG.md` because release notes are part of the API guideline expectations

### Cargo workspace policy

Recommended defaults:

- shared edition/version/license/repository metadata
- shared lint settings where supported
- consistent crate categories/keywords/description fields
- minimal public dependency surface

### CI baseline

The first CI should run:

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo doc --workspace --no-deps`
- dependency/license checks

This follows the Rust best-practices guidance around linting and testing discipline.

---

## 5. Crate Responsibilities

## 5.1 `mnemix-core`

This crate owns product semantics.

### Responsibilities

- domain models
- memory scopes and layers
- recall/search request and result types
- ranking and disclosure policies
- pinning rules
- checkpoint abstractions
- retention policy types
- export/import contracts at the product layer
- traits for storage backend capabilities

### Must not own

- raw LanceDB table wiring
- CLI formatting
- Python binding glue
- host-specific adapter behavior

### Design guidance

Per the Rust API guidelines, this crate should favor:

- newtypes for identifiers and domain meaning
- private fields with constructors/builders
- predictable method naming
- typed errors using `thiserror`
- serde support for portable domain objects where appropriate

## 5.2 `mnemix-lancedb`

This crate owns the concrete local storage implementation.

### Responsibilities

- database connection management
- table creation/open/migration
- query translation
- FTS/index management
- version inspection and restore plumbing
- tag/checkpoint implementation
- optimize and cleanup implementation
- import/export storage translation
- future `lance` advanced integration points

### Design guidance

This crate should present a clean backend implementation of core traits, while isolating:

- SDK churn
- schema evolution details
- query builder specifics
- advanced branch mechanics

## 5.3 `mnemix-cli`

This crate is the human-facing operational surface.

### Responsibilities

- command parsing
- rich terminal output
- JSON output mode
- interactive-safe restore/checkpoint prompts later
- stats and inspection views
- snapshot-friendly formatting for tests

### Design guidance

Use `anyhow` or equivalent only at the binary boundary if needed. Keep library errors typed below the CLI layer.

## 5.4 `python/mnemix`

This package is the first language binding.

### Responsibilities

- wrap stable Rust core APIs
- expose ergonomic Python entry points
- support scripting and adapter integration
- avoid re-implementing business logic

## 5.5 `adapters/ai-dx-toolkit`

This adapter should translate host concepts into Mnemix concepts without creating a second memory system.

---

## 6. Initial Internal Module Scaffold

## 6.1 `mnemix-core/src/`

```text
lib.rs
config.rs
errors.rs
ids.rs
memory.rs
scope.rs
query.rs
ranking.rs
pinning.rs
disclosure.rs
summaries.rs
checkpoints.rs
history.rs
retention.rs
inspection.rs
export.rs
import.rs
traits.rs
```

### Notes

- `ids.rs` should hold newtypes such as `MemoryId`, `ScopeId`, `CheckpointName`, and `VersionId`.
- `traits.rs` should define backend-facing traits so the core is not coupled directly to `lancedb`.
- `history.rs` should model inspection and diff-friendly concepts before the CLI layer exists.

## 6.2 `mnemix-lancedb/src/`

```text
lib.rs
backend.rs
connection.rs
schema.rs
migrations.rs
models.rs
queries.rs
indexing.rs
versions.rs
tags.rs
optimize.rs
serialization.rs
export.rs
import.rs
branches.rs
```

### Notes

- `branches.rs` can be mostly placeholder in v1 but should exist so the internal design stays branch-aware.
- `models.rs` should isolate storage-row representations from core domain models.
- `serialization.rs` should centralize schema encoding/decoding and avoid scattering row transforms.

## 6.3 `mnemix-cli/src/`

```text
main.rs
cli.rs
errors.rs
output/
  human.rs
  json.rs
cmd/
  init.rs
  remember.rs
  recall.rs
  search.rs
  show.rs
  pins.rs
  history.rs
  checkpoint.rs
  versions.rs
  restore.rs
  stats.rs
  export.rs
  import.rs
  optimize.rs
```

### Notes

Keep output rendering separate from command execution logic so it is easy to snapshot test and to keep the command layer small.

---

## 7. Milestone Breakdown

## Milestone 0 — Workspace and engineering baseline

**Goal:** create the repo skeleton and guardrails before product code spreads.

### Deliverables

- cargo workspace scaffold
- root metadata and policy files
- CI workflow with fmt/clippy/test/doc checks
- crate READMEs and crate metadata
- initial docs structure
- test-support crate stub

### Acceptance criteria

- workspace builds with placeholder crates
- CI is green on an empty but valid workspace
- lint and formatting are enforced from day one
- crate docs render without warnings

### Key best-practice ties

- API guideline metadata expectations
- Clippy discipline
- documentation coverage from the start

---

## Milestone 1 — Core domain contract freeze

**Goal:** define the product model independent of storage.

### Deliverables

- typed domain IDs and value objects
- `MemoryRecord` and related domain structs
- `RecallQuery`, `SearchQuery`, `HistoryQuery`, `StatsQuery`
- checkpoint and retention types
- backend capability traits
- crate-level docs and examples
- unit tests for domain invariants

### Acceptance criteria

- no `lancedb` dependency in `mnemix-core`
- public APIs use typed parameters instead of ambiguous flags
- core error types are typed and documented
- docs demonstrate at least one end-to-end in-memory-style flow

### Key best-practice ties

- newtypes over ambiguous strings and `bool`
- builders where construction becomes complex
- private fields for future-proofing
- examples in rustdoc

---

## Milestone 2 — Local LanceDB backend MVP

**Goal:** implement the first real persistent backend for local-first usage.

### Deliverables

- connect/open/init local store
- create/open required tables
- schema version metadata
- remember/search/show primitives
- FTS indexing path
- basic filtering and ranking integration
- version listing
- checkpoint tag creation
- export/import skeletons
- integration test fixtures using temp directories

### Acceptance criteria

- can initialize a local store and persist memory records
- can search by FTS and scope filters
- can create and list checkpoints
- integration tests cover create/open/add/delete/query flows
- backend details remain hidden behind core traits

### Key Lance guide ties

- use `lancedb` for the main storage path
- maintain explicit schema control
- rely on FTS first, not vector complexity
- treat versioning and tags as part of the MVP, not future polish

---

## Milestone 3 — Human-first CLI MVP

**Goal:** make the system usable and inspectable without a custom UI.

### Deliverables

- `init`
- `remember`
- `search`
- `show`
- `pins`
- `history`
- `checkpoint`
- `versions`
- `stats`
- `export`
- `import`
- `--json` output mode
- snapshot tests for CLI output

### Acceptance criteria

- a user can inspect stored memory, timestamps, and checkpoints from the terminal
- all commands support predictable human-readable output
- machine-readable output is stable enough for adapter work
- snapshots protect output regressions

### Key best-practice ties

- tests as living documentation
- avoid hidden behavior
- keep binary-layer error aggregation separate from library error types

---

## Milestone 4 — Progressive disclosure and pinning semantics

**Goal:** implement the product behavior that differentiates Mnemix from a simple searchable store.

### Deliverables

- explicit pin/unpin support
- pinned-context retrieval path
- summary-first recall path
- archival expansion path
- retrieval explanation metadata
- ranking heuristics for recency/importance/pinned state
- tests for disclosure ordering and recall semantics

### Acceptance criteria

- recall can return layered results rather than one flat list
- pinned context is consistently favored but bounded
- summaries and archival items are distinguishable in results
- explanations for surfaced items are available for inspection

### Key plan ties

- progressive disclosure is a v1 requirement
- pinned vs archival distinction is first-class
- explainable retrieval beats over-engineered retrieval in v1

---

## Milestone 5 — Version-aware safety features

**Goal:** make restore, history, and maintenance safe and understandable.

### Deliverables

- historical inspection commands and APIs
- restore flow
- pre-import and pre-optimize checkpoint policy
- retention configuration types
- optimize command with clear warnings
- tag/checkpoint naming policy
- tests for restore and recovery behavior

### Acceptance criteria

- restore creates a new current state rather than silently mutating history semantics
- cleanup paths are conservative by default
- tagged versions are protected from routine cleanup logic
- users can inspect history before destructive operations

### Key Lance guide ties

- preserve distinction between `checkout` and `restore`
- treat cleanup as a recoverability decision
- use tags as named stable references

---

## Milestone 6 — Python binding and first adapter

**Goal:** make the Rust core consumable from the first external host.

### Deliverables

- Python package scaffold
- stable high-level Python entry points
- serialization-safe request/response wrappers
- AI DX Toolkit adapter proof of concept
- usage examples
- binding tests

### Acceptance criteria

- Python wraps Rust behavior without duplicating core logic
- adapter uses public APIs only
- core semantics remain unchanged between CLI and Python

### Key best-practice ties

- keep the core public API clean and stable
- do not leak backend-specific details into the host adapter interface

---

## Milestone 7 — Advanced storage workflows

**Goal:** prepare for expert workflows without destabilizing v1.

### Deliverables

- branch-aware internal extensions
- shallow/deep clone evaluation
- advanced import staging workflow
- branch experiment prototype
- branch lifecycle documentation

### Acceptance criteria

- no breaking changes to the v1 product model
- branch-aware functionality remains clearly marked as advanced
- internal abstractions can support alternate timelines safely

### Key Lance guide ties

- use `lance` selectively where `lancedb` stops being the right abstraction
- keep branch UX guarded until semantics are fully clear

---

## 8. Suggested Milestone Order for Actual Execution

If the project wants the fastest path to a usable v1, the recommended sequence is:

1. Milestone 0
2. Milestone 1
3. Milestone 2
4. Milestone 3
5. Milestone 4
6. Milestone 5
7. Milestone 6
8. Milestone 7

This keeps the first shippable local CLI before Python/adapters and before advanced branch workflows.

---

## 9. Definition of v1

Mnemix v1 should be considered complete when all of the following are true:

- local-first embedded storage works reliably
- text-first search and recall work with scope filtering
- pinned vs archival layering is implemented
- progressive disclosure behavior is implemented
- human-readable CLI inspection is strong
- checkpoints and version history are first-class
- restore is safe and explainable
- export/import are available
- retention defaults are conservative
- Python binding exists for the first integration path

Branch UX is not required for v1.

---

## 10. First Files to Create When Implementation Starts

### Root

- `Cargo.toml`
- `rust-toolchain.toml`
- `clippy.toml`
- `rustfmt.toml`
- `deny.toml`
- `.github/workflows/ci.yml`
- `README.md`
- `CHANGELOG.md`

### Core crate

- `crates/mnemix-core/Cargo.toml`
- `crates/mnemix-core/src/lib.rs`
- `crates/mnemix-core/src/errors.rs`
- `crates/mnemix-core/src/ids.rs`
- `crates/mnemix-core/src/memory.rs`
- `crates/mnemix-core/src/query.rs`
- `crates/mnemix-core/src/traits.rs`

### Backend crate

- `crates/mnemix-lancedb/Cargo.toml`
- `crates/mnemix-lancedb/src/lib.rs`
- `crates/mnemix-lancedb/src/backend.rs`
- `crates/mnemix-lancedb/src/schema.rs`
- `crates/mnemix-lancedb/src/versions.rs`
- `crates/mnemix-lancedb/src/tags.rs`

### CLI crate

- `crates/mnemix-cli/Cargo.toml`
- `crates/mnemix-cli/src/main.rs`
- `crates/mnemix-cli/src/cli.rs`
- `crates/mnemix-cli/src/cmd/init.rs`
- `crates/mnemix-cli/src/cmd/search.rs`
- `crates/mnemix-cli/src/cmd/show.rs`

---

## 11. Architecture Risks to Watch Early

- letting `lancedb` types leak into the core crate
- exposing ambiguous stringly typed APIs too early
- merging CLI presentation concerns into product/domain types
- aggressive cleanup defaults that undermine trust
- skipping checkpoint support until “later”
- re-implementing logic separately in Python
- overbuilding branch workflows before the main product is solid

---

## 12. Recommended Next Planning Artifacts

After this roadmap, the next useful planning documents are:

1. `docs/repo-scaffold-spec.md`
2. `docs/v1-api-contract.md`
3. `docs/schema-spec.md`
4. `docs/checkpoint-and-retention-policy.md`
5. `docs/cli-json-contract.md`

---

## 13. Bottom Line

The implementation roadmap should keep Mnemix disciplined:

- start with workspace and API hygiene
- define the product model before storage details leak upward
- ship a strong local CLI before chasing advanced workflows
- wrap Rust in Python rather than duplicating logic
- treat checkpoints, history, and safe restore as part of the initial product promise

If the repo scaffold and milestones above are followed, the project should stay aligned with both the Rust API guidance and the LanceDB architectural realities already documented in the planning set.
