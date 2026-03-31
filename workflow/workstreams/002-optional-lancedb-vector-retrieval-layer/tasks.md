# Tasks: Optional Lancedb Vector Retrieval Layer

## Workstream Goal

Preserve the completed vector retrieval implementation plan as a real
workstream, then use that workstream to track the actual implementation backlog
that still remains across the open follow-on GitHub issues.

## Execution Slices

### Planning

- [x] Convert the plan-only vector retrieval issue `#75` into a repo-native
  workstream
- [x] Preserve the detailed implementation plan in
  `.plans/lancedb-vector-retrieval-implementation-plan.md`

### Contracts And Schema

- [ ] Implement issue `#77`: vector retrieval contracts
- [ ] Implement issue `#78`: vector-capable LanceDB schema scaffolding

### Embedding Pipeline

- [ ] Implement issue `#80`: embedding provider plumbing
- [ ] Implement issue `#85`: vector enablement and backfill flows
- [ ] Implement issue `#86`: embedding write-path support

### Retrieval And Ranking

- [ ] Implement issue `#87`: semantic candidate retrieval
- [ ] Implement issue `#89`: fixed-size embedding schema groundwork
- [ ] Implement issue `#90`: write fixed-size embeddings for schema-v4 stores

### Operator Visibility

- [ ] Implement issue `#88`: vector status and index readiness reporting

## Validation Checklist

- [ ] Keep `STATUS.md` aligned with the true implementation backlog
- [ ] Update issue state as slices land instead of relying on stale Dex markers
- [ ] Validate retrieval mode, fallback behavior, schema migration, and
  readiness output with targeted tests
- [ ] Preserve lexical-only behavior for stores without vectors

## Notes

- Issue `#75` is a completed planning artifact, not the execution backlog
  itself.
- Several follow-on issues contain stale `dex:task:completed:true` markers in
  their GitHub bodies. This workstream treats those items as active until their
  actual GitHub issue state and merged code say otherwise.
