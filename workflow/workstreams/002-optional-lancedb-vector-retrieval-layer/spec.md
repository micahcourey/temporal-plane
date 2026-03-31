# Feature Spec: Optional Lancedb Vector Retrieval Layer

## Summary

Add optional LanceDB vector retrieval to Mnemix as an additive capability that
improves semantic recall without replacing the current text-first, explainable,
progressive-disclosure retrieval model.

## Problem

Mnemix currently works best when query wording overlaps strongly with stored
memory wording. That preserves explainability, but it misses cases where a query
is semantically related to a stored summary, preference, or procedure without
sharing enough exact lexical overlap for FTS alone.

Issue `#75` captured the implementation plan for solving that problem, but it
was a planning artifact, not the durable execution container. This workstream
becomes the canonical repo-native home for the vector retrieval effort.

## Users

- Primary persona: coding agents and local automation that need better semantic
  memory recall
- Secondary persona: maintainers operating LanceDB-backed stores who need safe
  vector enablement, backfill, and maintenance

## Goals

- Improve recall for semantically related memories while preserving lexical
  retrieval as the baseline
- Keep vector support optional per store and per query
- Keep LanceDB and embedding mechanics contained inside `mnemix-lancedb`
- Preserve explainability and checkpoint-safe migration behavior

## Non-Goals

- Replacing lexical search as the only supported retrieval path
- Requiring cloud-hosted embeddings
- Leaking LanceDB vector schema details into `mnemix-core`
- Treating stale Dex completion metadata as authoritative over current backlog

## User Value

This workstream improves memory recall quality for real coding and research
workflows where semantic similarity matters, while keeping the product local,
inspectable, and safe to operate.

## Functional Requirements

- Core query contracts must represent lexical, semantic, and hybrid retrieval
  semantics without backend leaks
- LanceDB storage must support optional embedding persistence and vector
  configuration
- Operators must be able to enable vectors, backfill embeddings, and inspect
  readiness safely
- CLI and Python surfaces must expose the feature consistently
- Search and recall explanations must distinguish lexical, semantic, and hybrid
  matches

## Constraints

- `mnemix-core` stays storage-agnostic
- Lexical retrieval must remain valid when vectors are disabled or unavailable
- Backfill and migration behavior must be safe and checkpoint-aware
- Python remains a wrapper surface rather than a second implementation

## Success Criteria

- The vector retrieval plan from issue `#75` is preserved in proper workstream
  form
- The active implementation backlog is visible as one cohesive execution plan
- Future contributors can see that `#75` is the umbrella plan and `#77`-`#90`
  are the actual implementation slices

## Risks

- Semantic ranking can unintentionally overpower pinning, recency, or
  importance
- Schema evolution can become fragile if embedding storage is forced too early
- Open issues may drift from actual implementation state if the workstream is
  not updated as slices land

## References

- GitHub issue `#75`
- `.plans/lancedb-vector-retrieval-implementation-plan.md`
- GitHub issues `#77`, `#78`, `#80`, `#85`, `#86`, `#87`, `#88`, `#89`, `#90`
