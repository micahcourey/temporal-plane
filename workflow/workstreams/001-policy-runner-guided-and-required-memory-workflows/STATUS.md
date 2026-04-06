---
status: completed
summary: Policy runner lifecycle commands, enforcement examples, and CodingAgentAdapter composition are now shipped, and MCP is explicitly deferred until a concrete interoperability need appears.
updated: 2026-04-05
prs:
  - 66
  - 79
---

# Status

This workstream is the repo-native home for the policy runner effort that was
previously split across the plan-only GitHub issue `#74` and several follow-on
implementation issues.

The planning/design slice from `#74` is complete, and two foundational
implementation slices already landed:

- `#65` expanded `CodingAgentAdapter` policy-oriented workflow helpers and was
  closed after PR `#66`
- `#76` shipped the initial policy runner surface and was closed after PR `#79`

The final follow-on slices are now represented in the repo:

- `#82` shipped reference enforcement examples for local hooks, wrappers, and
  CI/PR policy checks
- `#83` shipped `policy clear` and `policy cleanup`, lifecycle metadata in
  `policy-state.json`, and TTL-aware task/session evidence handling
- `#84` composed the policy runner with `CodingAgentAdapter` task start,
  checkpoint, and writeback flows
- `#81` is explicitly deferred for now because host-side enforcement paths are
  sufficient and MCP does not yet add enough interoperability value to justify
  another surface area

## References

- GitHub issue `#74`: design source for this workstream
- GitHub issue `#81`
- GitHub issue `#82`
- GitHub issue `#83`
- GitHub issue `#84`
