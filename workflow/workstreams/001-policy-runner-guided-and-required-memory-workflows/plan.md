# Plan: Policy Runner Guided And Required Memory Workflows

## Summary

Treat this workstream as the durable home for the policy-runner architecture and
execution backlog. The design slice from issue `#74` is already complete, the
first implementation layers from `#65` and `#76` have shipped, and the
remaining work should extend the existing policy surface rather than redesign
it.

## Scope Analysis

### Affected Areas

| Area | Changes Required |
|------|-----------------|
| `mnemix-core` | Extend policy model only where lifecycle or integration semantics require it |
| `mnemix-cli` | Keep `policy check`, `policy explain`, and `policy record` as the canonical command loop |
| Python wrapper | Mirror any additional lifecycle or integration surfaces |
| Adapters | Compose with the policy runner without duplicating core policy logic |
| Workflow docs | Explain trigger/action/evidence semantics and workflow-key strategy |
| GitHub/workflow artifacts | Track completion and remaining slices cleanly |

### Affected Layers

- [x] Documentation and planning artifacts
- [x] Core policy model
- [x] CLI implementation
- [x] Python wrapper
- [x] Adapter surface
- [ ] Hooks and reference enforcement examples
- [ ] MCP interoperability layer

## Technical Design

### Current Baseline

The following slices are already shipped or represented by merged work:

- issue `#74`: policy runner design and phased rollout definition
- issue `#65` / PR `#66`: `CodingAgentAdapter` scope helpers, classification,
  explicit skip behavior, and `store_outcome(...)`
- issue `#76` / PR `#79`: typed policy config, evidence, evaluation, and CLI /
  Python policy surfaces

### Remaining Additions

```text
crates/mnemix-core/src/policy.rs                  # lifecycle and evidence policy extensions
crates/mnemix-cli/src/cmd/policy.rs               # clear/cleanup or related lifecycle commands
python/mnemix/client.py                           # lifecycle wrappers
python/mnemix/models.py                           # lifecycle request/response models
adapters/coding_agent_adapter.py                  # policy-aware composition entrypoints
examples/                                         # end-to-end guided / strict workflows
docs/ or workflow/workstreams/001-.../            # canonical explanation of the architecture and backlog
```

### Design Constraints

- Keep policy above storage and backend details
- Keep enforcement host-side even if MCP is added as a transport surface
- Reuse the existing policy runner as the single source of workflow judgment
- Do not split CLI, Python, and adapter behavior into separate policy models

## Implementation Slices

### Slice 1: Planning And Baseline Foundation

- Completed design work from issue `#74`
- Completed coding-agent adapter policy helpers from issue `#65`
- Completed initial policy runner implementation from issue `#76`

### Slice 2: State Lifecycle

- Issue `#83`: add evidence cleanup, clear behavior, TTL handling, and related
  CLI/Python surfaces

### Slice 3: Enforcement Examples

- Issue `#82`: add reference git-hook, wrapper CLI, and CI/PR enforcement
  examples

### Slice 4: Adapter Composition

- Issue `#84`: compose policy checks with `CodingAgentAdapter` start, writeback,
  and checkpoint flows

### Slice 5: MCP Evaluation

- Issue `#81`: evaluate whether MCP adds value as an interoperability surface
  and either implement or explicitly defer it

## Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| The repo loses the original planning context from issue `#74` | Medium | Medium | Keep this workstream as the canonical summary of the design and backlog |
| New integrations bypass the existing policy runner and reimplement logic | High | Medium | Require CLI, Python, and adapter flows to compose with the shipped policy model |
| MCP scope expands too early and distracts from core host-side workflows | Medium | Medium | Keep MCP explicitly optional until the host-side lifecycle and adapter slices are settled |

## References

- GitHub issue `#74`
- GitHub issue `#81`
- GitHub issue `#82`
- GitHub issue `#83`
- GitHub issue `#84`
- PR `#66`
- PR `#79`
