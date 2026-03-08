# LanceDB + Lance Rust SDK Agent Guide

> Internal research guide for Temporal Plane planning.

**Purpose:** document what LanceDB and the lower-level Lance Rust SDK can realistically provide for Temporal Plane, with emphasis on local-first usage, time-travel, tags, branching, and operational caveats.

---

## 1. Executive Summary

Rust is a viable primary backend interface for Temporal Plane.

The strongest conclusion from the research is:

- `lancedb` in Rust is sufficient for the core local database lifecycle Temporal Plane needs.
- `lance` adds the lower-level dataset and version-control primitives that matter for advanced Temporal Plane workflows.
- Rust is strongest when the design is:
  - local-first,
  - async-friendly,
  - Arrow-native,
  - comfortable with structured query and stream-style result handling.
- Python still has convenience advantages, especially for ad hoc data manipulation and result materialization, but it is not required for feature coverage.

### Planning conclusion

For Temporal Plane:

- **Use Rust as the primary implementation language.**
- **Use `lancedb` for most application-level storage workflows.**
- **Use `lance` selectively for advanced dataset/version-control workflows** such as branching, shallow clone, and deeper lifecycle control.

---

## 2. Recommended Mental Model

Treat the stack as two related layers.

### `lancedb`

Application-facing database layer:

- connect to local/object-store-backed databases
- create/open tables
- add/update/delete rows
- query/filter/search
- create indexes
- inspect versions
- checkout/restore tables
- manage tags/checkpoints

### `lance`

Lower-level dataset control layer:

- dataset internals
- tags and branches at the dataset level
- shallow clone / deep clone
- explicit lineage and layout behaviors
- lower-level cleanup and metadata operations
- more storage-aware and less convenience-oriented APIs

### Temporal Plane implication

Temporal Plane should not assume one crate covers all needs elegantly.

The best design is likely:

- `lancedb` for the normal product path
- `lance` for the advanced version-control path

---

## 3. Why LanceDB Fits Temporal Plane

LanceDB/Lance aligns strongly with the product goals:

- **local-first embedded operation**
- **no server required for OSS/local use**
- **Arrow-native storage** for efficient columnar reads
- **built-in versioning**
- **full-text and vector retrieval in one stack**
- **explicit restore, checkout, and tags**
- **direct object storage support later if needed**

For Temporal Plane, this means the storage substrate already provides:

- time-travel,
- auditability,
- reproducibility,
- search,
- indexing,
- schema evolution,
- and the beginnings of Git-like dataset workflows.

---

## 4. Capability Matrix

| Need | Rust support | Notes |
|---|---|---|
| Local path connection | Supported | Strong fit for embedded local-first workflows |
| Object storage connection | Supported | `s3://`, `gs://`, `az://`, etc. |
| Explicit schema | Supported | Arrow schema-first workflows fit Rust well |
| Create/open tables | Supported | Core table lifecycle is covered |
| Append/add data | Supported | Arrow-native ingestion is a strong path |
| Update/delete | Supported | Standard mutating operations available |
| Filtering/querying | Supported | Query builders and execution APIs exist |
| Full-text search | Supported | FTS index support exists in Rust |
| Scalar indexes | Supported | `BTree`, `Bitmap`, `LabelList` |
| Vector indexes | Supported | IVF and HNSW family variants available |
| Hybrid search | Supported with caveats | Less polished ergonomically than Python docs imply |
| Versioning | Supported | Read version, list versions, checkout, restore |
| Tags/checkpoints | Supported | Strong fit for Temporal Plane milestones |
| Branching | Supported at lower level | More naturally a `lance` concern than casual `lancedb` workflow |
| Shallow clone | Supported at lower level | Advanced but highly relevant |
| Deep clone | Supported at lower level | Useful for archival/export workflows |
| Optimize/compaction | Supported | Operationally important, somewhat sharp-edged |
| Cleanup/pruning | Supported | Must be handled conservatively |
| Import/export | Partially supported | Core data access yes; convenience ergonomics weaker than Python |
| Cloud/remote workflows | Supported with caveats | Native/local and remote/cloud behavior can differ |

---

## 5. Local-First Usage

Temporal Plane is explicitly local-first, and LanceDB fits that very well.

### What the stack supports

- direct local filesystem databases
- in-process operation
- no database daemon for OSS/local usage
- portable filesystem-backed table state
- later promotion to object storage if needed

### Why this matters

Temporal Plane should treat local datasets as the default source of truth for:

- development use
- local agent sessions
- debugging
- test fixtures
- backup/export
- human inspection

### Planning implication

The product should not be designed cloud-first and then forced back into local workflows. LanceDB already supports the desired local mode natively.

---

## 6. Core Rust API Concepts

The exact method names and shapes may evolve, but the research showed these major concepts are present.

### Connection and database setup

Important concepts include:

- `connect(...)`
- `ConnectBuilder`
- `Connection`
- `create_table(...)`
- `create_table_streaming(...)`
- `create_empty_table(...)`
- `open_table(...)`
- `drop_table(...)`
- namespace and table listing operations

### Table lifecycle

Important concepts include:

- `Table`
- `schema()`
- `count_rows(...)`
- `add(...)`
- `update(...)`
- `delete(...)`
- `merge_insert(...)`

### Querying

Important concepts include:

- `query()`
- projection / select controls
- filters
- limit/offset
- FTS query controls
- vector nearest-neighbor query path
- query execution plan / analysis support

### Indexing

Important concepts include:

- `create_index(...)`
- `Index`
- index stats
- list indices
- wait for index
- prewarm index

### Versioning and recovery

Important concepts include:

- `version()`
- `list_versions()`
- `checkout(...)`
- `checkout_tag(...)`
- `checkout_latest()`
- `restore()`

### Tags / checkpoints

Important concepts include:

- `tags()`
- list tags
- create tag
- update tag
- delete tag
- resolve tag to version

### Maintenance

Important concepts include:

- `optimize(...)`
- compaction
- pruning
- index optimization / reindex behavior

---

## 7. Full-Text Search

Full-text search is highly relevant to Temporal Plane because v1 is text-first.

### What matters

Rust `lancedb` supports FTS indexing and query support.

This is a strong fit for:

- summaries
- details
- tags
- entities
- file names and paths
- durable lessons and preferences

### Planning implication

Temporal Plane should plan around:

- a denormalized `fts_text` field
- FTS for lexical retrieval
- metadata filters for precision
- optional vector retrieval later

### Caveat

Do not assume the Rust FTS ergonomics exactly match Python examples. The capability is there, but the product should validate the exact Rust flow before freezing UX assumptions.

---

## 8. Indexing Model

Temporal Plane will depend heavily on indexing quality for practical local performance.

### Relevant index types

Research surfaced support for:

- `BTree`
- `Bitmap`
- `LabelList`
- `FTS`
- vector index families including IVF and HNSW variants

### Planning implication

For Temporal Plane, the most likely early-value indexes are:

- FTS on searchable text
- scalar index on session/scope/category/importance/timestamps
- maybe label-list style indexing later for tags or entities depending on exact schema

### Operational caveat

Indexes are an explicit maintenance concern.

New data may not be fully represented in optimized indexes until maintenance runs. So Temporal Plane must treat index optimization as a product-level operational feature, not a hidden implementation detail.

---

## 9. Versioning and Time Travel

This is one of the strongest reasons to use Lance.

### Core observations

- mutating operations create new versions
- read-only operations do not
- `checkout` is for viewing historical state
- `checkout_latest` returns to current head state
- `restore` creates a new version equivalent to the restored snapshot

### Why this matters to Temporal Plane

Temporal Plane needs to support:

- reproducible memory state
- auditability
- “what did the agent know then?”
- repair and rollback after bad memory writes
- safe pre-maintenance checkpoints

### Planning implication

Versioning is not just a backend feature. It should be treated as a user-facing product capability.

---

## 10. Tags as First-Class Checkpoints

Tags are one of the best immediate fits for Temporal Plane.

### Why tags matter

Tags are the cleanest way to create stable, human-readable references to meaningful memory states.

### Recommended Temporal Plane uses

- `bootstrap`
- `pre_import`
- `pre_restore`
- `pre_optimize`
- `milestone_*`
- `session_end_*`
- `backup_*`

### Planning implication

For Temporal Plane:

- tags should be a first-class user concept
- checkpoint UX should be built on tags
- retention policy should explicitly preserve tagged versions

---

## 11. Branching

Branching is one of the most important newer Lance capabilities for future Temporal Plane design.

### What the new Lance model provides

- independent version history per branch
- physical separation of branch data
- better isolation than older shared-metadata branch models
- full time travel within branches

### Why it is relevant to Temporal Plane

Potential Temporal Plane branch use cases include:

- alternate pinning strategies
- cleanup experiments
- memory consolidation experiments
- import staging
- safe what-if analysis on a memory store

### Important caution

Branching is powerful, but not yet something to expose casually without product-specific guardrails.

Research indicates branch creation should be treated carefully and may require cleanup logic if operations fail.

### Planning implication

Temporal Plane should:

- be branch-aware in the architecture
- not require branch UX in v1
- consider branch workflows an advanced v2+ capability unless a concrete v1 need appears

---

## 12. Shallow Clone and Deep Clone

These features are also highly relevant.

### Shallow clone

Useful for:

- cheap experimental copies
- branch-like derived workspaces
- debug sandboxes
- import rehearsal
- review workflows

### Deep clone

Useful for:

- archival backups
- full export-like copies
- hard isolation
- migration and environment handoff

### Planning implication

Temporal Plane should likely distinguish two concepts later:

- cheap derived working copies
- durable fully owned copies

For v1, these are probably advanced features. But the architecture should not block them.

---

## 13. Optimization, Compaction, and Pruning

Optimization is necessary, but dangerous if treated casually.

### Key findings

Optimization and cleanup can:

- improve read performance
- compact fragmented data
- absorb newly added data into better index state
- prune old versions under retention rules

### Important caveat

Once old versions are cleaned up, recoverability can be reduced or lost.

### Temporal Plane implication

Temporal Plane needs explicit retention tiers, such as:

- disposable working history
- protected tagged milestones
- optional archival export/clone

### Product rule

Cleanup must be treated as a governance and recoverability event, not just a storage optimization.

---

## 14. Schema Evolution and Metadata

LanceDB/Lance can support a product that evolves over time.

### Relevant capabilities

- add columns
- drop columns
- alter columns
- metadata updates
- config metadata updates
- merge-style workflows

### Planning implication

Temporal Plane can evolve its schema without rebuilding from scratch all the time, but maintenance and migration planning still matter.

This is useful for:

- later adding pinned metadata
- adding provenance fields
- adding future retrieval features
- evolving export formats

---

## 15. Query Model

Rust support is sufficient for Temporal Plane’s likely query model.

### Likely retrieval layers

Temporal Plane can build layered retrieval on top of:

- scalar filters
- FTS
- vector similarity later
- query analysis and explain-plan support

### Planning implication

Temporal Plane should define its own retrieval contract in the core instead of leaking raw SDK differences upward.

---

## 16. Rust Caveats vs Python

Python still has real convenience advantages.

### Main differences

Python is friendlier for:

- ad hoc scripting
- dataframe workflows
- quick inspection
- easy materialization of result sets
- richer examples in docs

Rust is stronger for:

- core engine implementation
- explicit lifecycle control
- low-level predictability
- avoiding duplicated backend semantics across languages

### Important planning implication

Do not design the Rust implementation by copying Python examples mentally. Validate feature assumptions against actual Rust surfaces.

---

## 17. Remote vs Native Caveat

Research found that not all native/local and remote/cloud table behaviors are identical.

### Why this matters

Temporal Plane is local-first, so this is not a blocker.

But if remote support is added later, the product should not assume all operations behave exactly the same across environments.

### Planning implication

Use local/native behavior as the baseline contract for v1.

---

## 18. Temporal Plane Planning Recommendations

### Strong recommendations

- Use Rust as the primary backend implementation.
- Use `lancedb` for most application storage workflows.
- Use `lance` selectively for advanced version-control operations.
- Treat tags/checkpoints as required in v1.
- Treat historical inspection as required in v1.
- Treat optimize/retention as explicit product features.
- Keep retention conservative by default.

### Architectural recommendations

- Keep the core branch-aware even if branch UX is deferred.
- Design export/import with future deep-clone and archival workflows in mind.
- Separate normal storage workflows from advanced dataset-control workflows in the internal architecture.

### Product recommendations

- human-readable stats and inspection should be first-class in v1
- checkpoint and version inspection should be first-class in v1
- export/import should be included in v1
- branch and shallow-clone workflows should be planned but likely deferred to advanced phases

---

## 19. Key Risks and Caveats

- Rust docs are not always as example-rich as Python docs.
- Some advanced capabilities are split between `lancedb` and `lance`.
- Branching is powerful but should be wrapped in product guardrails.
- Cleanup can reduce recoverability if policy is too aggressive.
- Index freshness and optimization affect real-world performance.

---

## 20. Recommended Source Set for Future Research

High-value sources for ongoing validation:

- LanceDB tables/versioning docs
- LanceDB search and FTS docs
- docs.rs for `lancedb`
- docs.rs or upstream docs for `lance`
- Lance format docs, especially versioning and branch/tag spec
- branching and shallow clone blog posts
- time-travel RAG tutorial

---

## 21. Bottom-Line Recommendation

Temporal Plane should adopt LanceDB + Lance Rust SDK as a local-first, version-aware storage foundation.

The right planning stance is:

- `lancedb` for the main product path
- `lance` for advanced version-control behavior
- Rust as source of truth
- Python as wrapper/binding layer, not storage owner

And most importantly:

**Versioning, tags, and historical inspection should be treated as core Temporal Plane capabilities, not incidental backend details.**
