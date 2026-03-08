# Temporal Plane Plan v3

**Status:** planning only  
**Date:** March 8, 2026  
**Canonical plan:** this document supersedes v2 for ongoing planning.

---

## 1. Product Definition

Temporal Plane is a standalone, local-first memory layer for AI coding agents and related tooling.

It is designed to be:

- lightweight
- serverless for local usage
- easy to embed
- easy to inspect by humans
- optimized for episodic and distilled memory
- version-aware and time-travel capable

Temporal Plane is **not** tied to one agent framework. It should be usable from multiple hosts, with AI DX Toolkit as only the first adapter.

---

## 2. Product Goals

Temporal Plane v1 should provide:

1. durable local memory storage
2. fast text-first retrieval
3. explicit pinned context vs archival memory
4. progressive disclosure for context loading
5. human-readable inspection and stats
6. time-travel through memory history
7. stable checkpoints
8. import/export portability
9. low operational complexity
10. a clean library core with adapter surfaces

---

## 3. Non-Goals

Temporal Plane v1 should **not** attempt to be:

- a hosted memory platform
- a full agent runtime
- a graph-memory system
- an orchestration framework
- a daemon/service that must always be running
- a UI-first product
- a multi-tenant shared memory cloud
- a background autonomous reflection swarm

These may become adjacent opportunities later, but they are not the right starting scope.

---

## 4. Product Positioning

Temporal Plane should position itself as:

> a lightweight, inspectable, local-first memory engine for coding agents with built-in version history and practical time-travel.

This differentiates it from:

- **Mem0**, which is strong on managed memory workflows, extraction, and platform features but heavier operationally
- **Letta**, which is strong on stateful agent abstractions and memory architecture, but broader and more runtime-centric than needed for Temporal Plane v1

Temporal Plane should borrow the best product ideas from both while staying materially simpler.

---

## 5. Key Learnings from Mem0

Useful ideas worth adopting:

- scoped memory organization
- distilled memory rather than raw transcript dumping
- metadata-first filtering and search
- explicit memory lifecycle operations
- observability and inspection primitives
- practical APIs for add/search/update/delete

Ideas to avoid in v1:

- managed-service assumptions
- heavy platform coupling
- graph-first complexity
- broad enterprise/team feature sprawl

---

## 6. Key Learnings from Letta

Useful ideas worth adopting:

- strong distinction between always-available memory and archival memory
- progressive disclosure of memory into the active context window
- editable, inspectable, stateful memory concepts
- portable memory/state representation
- maintenance workflows for consolidation and cleanup

Ideas to avoid in v1:

- full runtime/runtime-state complexity
- background sleep-time agent orchestration
- broad filesystem/runtime environment abstraction as a platform requirement
- product surface area that exceeds “memory layer” scope

---

## 7. Core Product Principles

### 7.1 Distilled over raw

Temporal Plane should primarily store durable, useful memories, not unbounded raw transcripts.

### 7.2 Local-first

Local embedded usage should be the default, not a reduced mode.

### 7.3 Human-inspectable

Users must be able to see what is stored, when it was stored, why it matched, and what changed.

### 7.4 Time-aware

Memory state over time is a core product feature.

### 7.5 Progressive disclosure

Only the most relevant, compact memory should be loaded by default. Deeper context should be fetched on demand.

### 7.6 Safe by default

Retention, restore, cleanup, and optimization must preserve recoverability unless the user explicitly chooses otherwise.

---

## 8. Language and Implementation Strategy

### 8.1 Core language choice

Rust should be the primary implementation language.

### Why Rust

- good fit for a reusable embedded engine
- strong performance with explicit control
- good match for Arrow-native storage systems
- clean FFI/binding story for Python and others
- avoids re-implementing core storage semantics in multiple languages
- supports a future CLI, library API, and adapters from one source of truth

### Why not Python as the core

Python is excellent for bindings, prototyping, and convenience, but weaker as the single source of truth for a durable embedded engine intended to be reused across host environments.

### Why not Go as the core

Go would be viable for a CLI-first tool, but weaker than Rust for an embedded library-first engine with strong data layout control and future cross-language bindings.

---

## 9. Storage Direction

Temporal Plane should use **LanceDB + Lance** as its storage foundation.

### Roles

- `lancedb`: normal database/table workflows
- `lance`: lower-level version-control and dataset workflows when needed

### Why this stack fits

- local embedded operation
- filesystem-backed persistence
- Arrow-native schema and storage
- FTS and indexing
- built-in versions and restore
- tags/checkpoints
- future branch and clone workflows
- upgrade path to object storage if ever needed

---

## 10. Time-Travel and Versioning Philosophy

Versioning is a product feature, not just backend behavior.

Temporal Plane should make it easy to answer:

- What did the memory store contain at a past moment?
- What changed between then and now?
- Can I safely return to a prior state?
- Can I checkpoint before maintenance, import, or cleanup?

### Required v1 versioning capabilities

- list versions
- inspect version history
- checkout historical state for read/inspection
- restore historical state as a new head state
- create human-readable checkpoints

### Important semantic distinction

- `checkout` = view prior state
- `restore` = create a new current state from prior state

Temporal Plane should preserve that distinction clearly in its API and CLI.

---

## 11. New Lance Version-Control Findings and Their Impact

Recent Lance capabilities materially strengthen the Temporal Plane roadmap.

### 11.1 Tags should be first-class in v1

Lance supports immutable, named references to versions.

Temporal Plane should use this capability for:

- checkpoints
- safe restore points
- import guards
- pre-optimize snapshots
- session milestones
- export provenance

This is the cleanest way to give users understandable and stable state references.

### 11.2 Branch-aware architecture should exist from the start

Lance now supports Git-style branching at the dataset layer.

Temporal Plane should not ignore this in the architecture, even if branch UX is deferred.

The internal design should assume future support for:

- alternate memory timelines
- safe experimentation on memory state
- import staging branches
- cleanup/consolidation rehearsal
- “what-if” memory workflows

### 11.3 Branch UX should likely be deferred beyond v1

Although branching is strategically important, it is also an advanced feature.

Reasons to defer explicit branch UX:

- higher complexity for users
- need for strong guardrails
- need to define how pins, summaries, and disclosure behave across branches
- need to define cleanup and branch lifecycle semantics cleanly

### 11.4 Shallow clone is strategically important but not required in v1

Shallow clone opens the door to cheap derived working copies and experiments.

Temporal Plane should keep this in mind for future:

- lightweight previews
- branch sandboxing
- derived test datasets
- debug copies

But it should not be required to ship the core product.

### 11.5 Retention policy must be conservative

Lance optimization and cleanup can prune old recoverable state.

Therefore Temporal Plane should:

- default to conservative retention
- protect tagged/checkpointed versions
- warn before destructive cleanup
- treat cleanup as a recoverability decision

---

## 12. Memory Model

Temporal Plane should store durable memory objects rather than unstructured transcript logs.

### Primary memory categories

- `episodic` — notable events, actions, outcomes
- `semantic` — generalized facts or stable project knowledge
- `preference` — user or agent preferences
- `procedural` — workflows, commands, playbooks, heuristics
- `summary` — distilled rollups of sessions/scopes
- `pinned` — explicitly elevated always-available context

### Important rule

`pinned` is not merely a priority flag. It reflects a distinct operational role in recall and disclosure.

---

## 13. Memory Scopes and Layers

Temporal Plane should support scoped memory boundaries.

### Recommended scope dimensions

- workspace/project
- repository
- branch/worktree context later
- user profile
- agent/tool identity
- session
- topic/tag set

### Recommended memory layers

1. **Pinned context**
   - always small
   - always high signal
   - loaded first

2. **Working memory summaries**
   - compact rollups
   - recent high-value context
   - candidate for default recall

3. **Archival memory**
   - broader historical memory corpus
   - retrieved on demand
   - searchable and inspectable

This layered model should anchor both API design and retrieval behavior.

---

## 14. Progressive Disclosure

Progressive disclosure should be a defining feature of Temporal Plane.

### Basic rule

Do not dump the entire memory store into the active prompt.

Instead:

1. load pinned context
2. load compact summaries
3. surface top relevant episodic/semantic memory
4. allow deeper inspection or expansion on demand

### Benefits

- lower token usage
- cleaner context windows
- more predictable agent behavior
- better user trust because context loading is understandable

### v1 expectation

Temporal Plane should expose not just retrieval results, but also enough metadata to explain:

- why an item was surfaced
- when it was created or updated
- whether it is pinned
- what scope it belongs to

---

## 15. Retrieval Model

v1 retrieval should be text-first, practical, and explainable.

### Retrieval components

- lexical/FTS search
- metadata filtering
- scoring/ranking layer
- recency and importance adjustments
- pinning-aware prioritization
- summary-first recall paths

### Future-compatible but deferred

- embedding-heavy retrieval
- hybrid dense + sparse retrieval
- graph traversal retrieval
- agent-driven background reranking loops

### Key principle

Retrieval should favor explainability over sophistication in v1.

---

## 16. Human-Friendly Inspection Requirements

A core product promise is that stored memory is inspectable.

### v1 inspection requirements

Users should be able to:

- search memory in plain language or keyword form
- filter by timestamps/date ranges
- inspect a specific memory item
- inspect history and versions
- inspect pinned items
- inspect tags/checkpoints
- inspect counts and distributions
- inspect why an item matched

### Human-readable output requirements

The CLI and adapter-facing surfaces should support:

- readable default output
- `--json` output for machine consumption
- timestamps in understandable form
- concise summaries plus expandable detail

---

## 17. Export and Import

Export and import should be included in v1.

### Why export matters

- backup
- portability
- inspection
- migration
- debugging
- reproducible issue reports

### Why import matters

- restore from backup
- bootstrap from external systems
- migration from prior memory tools
- testing and fixture setup

### Planning expectations

- export format should be explicit and documented
- import should support validation and dry-run modes later
- export/import operations should create or encourage checkpoints

---

## 18. Default Operational Model

Temporal Plane v1 should default to:

- embedded local database
- one primary memory store per configured scope root
- text-first indexing
- conservative retention
- explicit optimize command
- explicit checkpoint command
- transparent history inspection

This model keeps the system understandable and lightweight.

---

## 19. High-Level Architecture

### Layers

1. **Core domain engine**
   - memory model
   - retrieval semantics
   - pinning/disclosure semantics
   - checkpoint/version abstractions

2. **Storage backend**
   - LanceDB table operations
   - Lance advanced version-control operations when needed
   - schema/index lifecycle
   - optimize/cleanup/export helpers

3. **CLI**
   - human-facing workflows
   - inspection and administration
   - JSON output mode

4. **Bindings**
   - Python first
   - others later if needed

5. **Adapters**
   - AI DX Toolkit first
   - host-specific translation without changing core semantics

---

## 20. Repository and Module Layout

A likely repo shape:

- `crates/temporal-plane-core`
- `crates/temporal-plane-lancedb`
- `crates/temporal-plane-cli`
- `python/temporal_plane`
- `adapters/ai-dx-toolkit`
- `docs/`

### Core crate modules

- `memory.rs`
- `query.rs`
- `ranking.rs`
- `pinning.rs`
- `disclosure.rs`
- `summaries.rs`
- `checkpoints.rs`
- `store.rs`
- `schema.rs`
- `retention.rs`
- `inspection.rs`
- `export.rs`
- `import.rs`
- `config.rs`
- `errors.rs`

### LanceDB backend crate modules

- `backend.rs`
- `schema.rs`
- `migrations.rs`
- `indexing.rs`
- `queries.rs`
- `versions.rs`
- `tags.rs`
- `branches.rs` *(architectural placeholder even if lightly used at first)*
- `optimize.rs`
- `serialization.rs`
- `export.rs`
- `import.rs`

### CLI crate modules

- `main.rs`
- `cmd/init.rs`
- `cmd/remember.rs`
- `cmd/recall.rs`
- `cmd/search.rs`
- `cmd/show.rs`
- `cmd/pins.rs`
- `cmd/history.rs`
- `cmd/checkpoint.rs`
- `cmd/versions.rs`
- `cmd/restore.rs`
- `cmd/stats.rs`
- `cmd/export.rs`
- `cmd/import.rs`
- `cmd/optimize.rs`

---

## 21. Data Model Sketch

Suggested base fields for a memory record:

- `id`
- `scope_id`
- `memory_kind`
- `title`
- `summary`
- `detail`
- `fts_text`
- `importance`
- `confidence`
- `created_at`
- `updated_at`
- `source_session_id`
- `source_tool`
- `source_ref`
- `tags`
- `entities`
- `is_pinned`
- `pin_reason`
- `expires_at` *(optional later)*
- `metadata_json`

### Notes

- `fts_text` should be intentionally composed for retrieval quality.
- `is_pinned` and `pin_reason` are essential for disclosure semantics.
- raw transcript bodies should not be the main storage model.

---

## 22. Versioning and Checkpoint Abstractions

Temporal Plane should define its own product-level abstractions over Lance versioning.

### Recommended core concepts

- **Version** — a concrete table state revision
- **Checkpoint** — a user-meaningful named reference, likely implemented with tags
- **Restore** — create a new current state from a past version/checkpoint
- **History** — inspect what changed over time

### Product requirement

Users should not need to understand raw storage internals to work safely with history.

---

## 23. Branching Strategy

### v1 stance

- architecture: branch-aware
- user-facing feature set: minimal or none
- internal use: possible later for advanced workflows

### v2+ candidate uses

- experimental cleanup passes
- alternate summary generations
- memory compaction experiments
- safe import staging
- “fork this memory state” workflows

### Design rule

Do not build v1 in a way that makes branches impossible later.

---

## 24. Retention and Cleanup Strategy

Retention must balance storage efficiency with recoverability.

### Default policy direction

- keep recent history generously
- protect tagged versions
- do not aggressively prune by default
- make destructive cleanup explicit

### Cleanup product behavior

Before destructive cleanup, Temporal Plane should encourage or create a checkpoint.

### Reason

A memory system loses trust quickly if users cannot recover from a bad summarize, import, or optimize operation.

---

## 25. Observability and Stats

Temporal Plane should expose useful observability without requiring a heavy dashboard.

### v1 stats/inspection targets

- total memory count
- count by kind
- count by scope
- pinned count
- recent activity summary
- version count
- latest checkpoint/tag info
- approximate storage footprint if feasible
- index/optimize status where practical

This should be available in both human-readable and machine-readable formats.

---

## 26. CLI Requirements

The CLI is a major part of the product experience.

### v1 command set

- `init`
- `remember`
- `recall`
- `search`
- `show`
- `pins`
- `history`
- `checkpoint`
- `versions`
- `restore`
- `stats`
- `export`
- `import`
- `optimize`

### Output requirements

- human-readable by default
- `--json` for automation
- stable enough to support adapter integration

### UX requirement

Inspection commands must surface dates/timestamps clearly.

---

## 27. API Surface Expectations

The library API should expose product concepts, not raw storage quirks.

### Good API examples

- `remember(memory)`
- `recall(query)`
- `search(filters)`
- `pin(id)`
- `unpin(id)`
- `checkpoint(name)`
- `history(scope)`
- `restore(checkpoint_or_version)`
- `export(...)`
- `import(...)`

### Bad API direction

Do not force every consumer to understand raw Lance/LanceDB internals just to use the core product.

---

## 28. Python Binding Strategy

Python should be the first binding layer.

### Why

- many AI tool integrations are Python-based
- simplifies adoption by existing agent tooling
- enables easier scripting and experimentation

### Important constraint

Python should wrap Rust core behavior rather than re-implement it.

---

## 29. Testing Strategy

Testing should explicitly cover both product semantics and storage correctness.

### Test layers

1. **Core domain tests**
   - scoring
   - recall semantics
   - pinning/disclosure behavior
   - checkpoint behavior

2. **Backend integration tests**
   - create/open/add/update/delete
   - FTS/index behavior
   - list versions / checkout / restore
   - tag/checkpoint workflows
   - optimize and retention behavior

3. **CLI tests**
   - human output snapshots
   - JSON output contracts
   - error behavior

4. **Import/export tests**
   - round-trip fidelity
   - partial failure behavior
   - metadata preservation

5. **Future advanced tests**
   - branch workflows
   - shallow clone workflows

---

## 30. Documentation Requirements

The docs set should include:

- product overview
- memory model
- retrieval model
- pinning/progressive disclosure
- versioning and restore guide
- export/import guide
- retention and optimize guide
- adapter integration guide
- LanceDB/Rust backend agent guide

---

## 31. Delivery Phases

### Phase 0 — design freeze

- finalize product vocabulary
- finalize v1 scope
- freeze data model direction
- freeze versioning semantics

### Phase 1 — core local memory engine

- Rust core
- LanceDB backend
- text-first retrieval
- pinning + summaries
- checkpoint/version inspection
- export/import
- CLI inspection commands

### Phase 2 — adapter and binding hardening

- Python binding
- AI DX Toolkit adapter
- better automation surfaces
- ergonomics pass

### Phase 3 — advanced storage workflows

- branch-aware workflows
- shallow/deep clone workflows
- richer maintenance tools
- optional hybrid retrieval

---

## 32. Open Questions

Items still requiring explicit planning decisions:

1. exact public schema and serialization format
2. export format shape and compatibility policy
3. exact CLI JSON contract
4. how much provenance detail should be stored by default
5. whether summaries are stored separately or materialized from memory sets
6. how restore interacts with pins and summaries operationally
7. whether branch support should stay fully hidden in v1 or appear in expert mode
8. what optimize defaults are safe enough for end users

---

## 33. Concrete Recommendations

### Strong decisions already justified

- Rust core is the right primary direction.
- Python should be the first binding, not the core implementation.
- LanceDB + Lance is the right storage foundation.
- Progressive disclosure is a required product principle.
- Pinned vs archival memory must be explicit.
- Human-readable inspection is required in v1.
- Export/import is required in v1.
- Checkpoints/tags are required in v1.

### New version-control recommendations

- Temporal Plane should treat tags/checkpoints as first-class v1 product concepts.
- Temporal Plane should be architecturally branch-aware from day one.
- Temporal Plane should likely defer explicit branch/shallow-clone UX until after v1.
- Temporal Plane should default to conservative retention because cleanup can remove recoverability.

---

## 34. Immediate Next Planning Work

1. freeze the v1 command/API contract
2. freeze the base data schema
3. define checkpoint/tag naming and behavior rules
4. define retention defaults and cleanup safety rules
5. define export/import format and compatibility expectations
6. define how summaries and pins are stored and surfaced
7. create the first repository scaffolding plan

---

## 35. Bottom Line

Temporal Plane should be built as a standalone Rust memory engine backed by LanceDB/Lance, with:

- distilled durable memory
- text-first practical retrieval
- pinned vs archival layering
- progressive disclosure
- strong human inspection
- explicit checkpoints and history
- local-first operation

And after the latest Lance research, one design point is now clear:

**Temporal Plane should ship v1 with serious version-awareness and checkpointing, while staying architecturally ready for true branch-based memory workflows later.**
