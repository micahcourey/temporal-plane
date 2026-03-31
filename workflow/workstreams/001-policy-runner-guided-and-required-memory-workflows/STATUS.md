---
status: open
summary: Policy runner planning is complete and the remaining implementation backlog is tracked under issues #81, #82, #83, and #84.
updated: 2026-03-31
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

The remaining active backlog for this workstream is:

- `#81` evaluate and expose policy runner via MCP
- `#82` add policy runner enforcement adapters and examples
- `#83` add policy runner state lifecycle commands
- `#84` compose policy runner with `CodingAgentAdapter`

## References

- GitHub issue `#74`: design source for this workstream
- GitHub issue `#81`
- GitHub issue `#82`
- GitHub issue `#83`
- GitHub issue `#84`
