# Plan: Optional Lancedb Vector Retrieval Layer

## Summary

Use issue `#75` and the local `.plans/lancedb-vector-retrieval-implementation-plan.md`
as the design source, but treat this workstream as the durable execution
container for the full vector retrieval effort. The follow-on GitHub issues are
the real implementation slices and should remain the operational backlog until
they are merged and closed.

## Scope Analysis

### Affected Areas

| Area | Changes Required |
|------|-----------------|
| `mnemix-core` | Retrieval mode contracts, semantic match provenance, capability flags |
| `mnemix-lancedb` | Schema evolution, embedding persistence, vector indexing, hybrid ranking |
| CLI | Retrieval mode flags, vector status, operational commands |
| Python wrapper | Mirror retrieval mode and operational surfaces |
| Tests | Schema migration, backfill, ranking, readiness, and fallback coverage |
| Docs and workflow artifacts | Preserve the plan and track the issue-based rollout clearly |

### Affected Layers

- [x] Documentation and planning artifacts
- [ ] Core query contracts
- [ ] Backend capability surface
- [ ] LanceDB schema and maintenance flows
- [ ] Embedding provider plumbing
- [ ] Search and recall execution
- [ ] CLI surface
- [ ] Python wrapper
- [ ] Tests and operational reporting

## Technical Design

### Canonical Planning Source

The detailed design already exists in:

- GitHub issue `#75`
- `.plans/lancedb-vector-retrieval-implementation-plan.md`

This workstream intentionally does not replace that technical content with a
shorter watered-down version. Instead, it organizes the execution slices and
keeps the plan discoverable in repo-native form.

### Proposed Additions

```text
crates/mnemix-core/src/                            # retrieval mode and capability contracts
crates/mnemix-lancedb/src/                         # schema, provider, backfill, retrieval, status
crates/mnemix-cli/src/                             # CLI flags and vector status/maintenance flows
python/mnemix/                                     # mirrored retrieval and maintenance surfaces
tests / fixtures                                   # deterministic semantic and migration coverage
workflow/workstreams/002-optional-lancedb-vector-retrieval-layer/
```

### Design Constraints

- Keep text-first behavior as the baseline
- Keep LanceDB-specific details out of `mnemix-core`
- Preserve explainability for lexical, semantic, and hybrid results
- Keep vector support optional at both store and query levels
- Treat stale Dex completion markers in issue bodies as historical noise, not
  current execution truth

## Implementation Slices

### Slice 1: Planning

- Completed planning work from issue `#75`

### Slice 2: Product Contracts And Schema

- Issue `#77`: vector retrieval contracts
- Issue `#78`: vector-capable schema scaffolding

### Slice 3: Embedding Infrastructure

- Issue `#80`: embedding provider plumbing
- Issue `#85`: vector enablement and backfill flows
- Issue `#86`: embedding write-path support

### Slice 4: Retrieval Execution

- Issue `#87`: semantic candidate retrieval
- Issue `#89`: fixed-size embedding schema groundwork
- Issue `#90`: fixed-size embedding writes for schema-v4 stores

### Slice 5: Operator Visibility

- Issue `#88`: vector status and index readiness reporting

## Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Open issue bodies claim completion before code lands on main | High | High | Treat GitHub open/closed state and merged code as authoritative, and use this workstream to track the true backlog |
| Vector ranking overwhelms pinning, recency, or importance | High | Medium | Keep hybrid ranking inspectable and bounded |
| Schema evolution becomes brittle | High | Medium | Keep vector fields optional and make enablement/backfill explicit |
| Operational state becomes hard to understand | Medium | Medium | Make readiness, coverage, and index state first-class status outputs |

## References

- GitHub issue `#75`
- GitHub issues `#77`, `#78`, `#80`, `#85`, `#86`, `#87`, `#88`, `#89`, `#90`
- `.plans/lancedb-vector-retrieval-implementation-plan.md`
