---
status: completed
summary: "Deferred MCP exposure for the policy runner until host-side workflows demonstrate a concrete interoperability need."
updated: "2026-04-05"
---

# Patch: Evaluate and expose policy runner via MCP

## Summary

Evaluate whether MCP adds value as an interoperability layer for the policy runner, then implement the initial MCP surface if justified. Scope: define the MCP tool contract for policy check/record/explain, keep enforcement host-side, and document the relationship between MCP transport and local policy config/evidence. Acceptance criteria: the repo either ships an initial MCP policy surface or documents a rejected/deferred decision with reasoning, and the result is consistent with the policy-runner design doc.

## Reason

This patch preserves legacy Dex history after the repository moved to repo-native workflow tracking with `mnemix-workflow`.

## Scope

- Evaluate whether MCP adds enough value beyond the host-side policy surfaces
- Document the decision in repo-native workflow and user-facing guides
- Preserve the design rule that enforcement remains host-side

## Implementation Notes

- Reviewed the current policy surface after lifecycle commands, enforcement
  examples, and `CodingAgentAdapter` composition landed
- Deferred MCP because wrapper scripts, hooks, CI, and adapter composition now
  cover the primary host-side enforcement paths without adding another contract
  to maintain
- Updated `workflow/workstreams/001-.../STATUS.md`, `tasks.md`, `plan.md`, and
  `docs_site/src/guide/policy-runner.md` to reflect the defer decision

## Validation

- The policy-runner guide now documents MCP as intentionally deferred
- The workstream tracking artifacts mark issue `#81` as resolved by an explicit
  defer decision

## References

- GitHub issue: #81 (micahcourey/mnemix) - https://github.com/micahcourey/mnemix/issues/81
- Original source: `.dex/tasks.jsonl`
