# Tasks: Policy Runner Guided And Required Memory Workflows

## Workstream Goal

Keep policy runner work organized as one durable workstream where the planning
slice is complete, foundational implementation is shipped, and the remaining
follow-on issues are tracked as active execution slices.

## Execution Slices

### Planning And Foundation

- [x] Convert the plan-only policy runner issue `#74` into a repo-native
  workstream
- [x] Land `CodingAgentAdapter` policy helpers and writeback classification via
  issue `#65` / PR `#66`
- [x] Land the initial policy runner surface via issue `#76` / PR `#79`

### State Lifecycle

- [ ] Implement issue `#83`: clear and cleanup flows for policy evidence,
  lifecycle handling, and TTL-aware behavior

### Enforcement Integrations

- [ ] Implement issue `#82`: reference enforcement adapters and examples for
  hooks, CI, and wrapper flows

### Coding-Agent Composition

- [ ] Implement issue `#84`: higher-level composition between the policy runner
  and `CodingAgentAdapter`

### MCP Interoperability

- [ ] Implement or explicitly defer issue `#81`: MCP-facing policy surface and
  rationale

## Validation Checklist

- [ ] Keep `STATUS.md` aligned with shipped slices and remaining issue backlog
- [ ] Keep follow-on GitHub issue status in sync as slices land
- [ ] Verify new policy surfaces through targeted Rust, Python, and adapter
  tests
- [ ] Preserve the host-side enforcement model when adding examples or MCP

## Notes

- Issue `#74` was a completed planning task, not the durable execution
  container. This workstream replaces it as the canonical planning artifact.
