---
status: open
summary: "Evaluate whether MCP adds value as an interoperability layer for the policy runner, then implement the initial MCP surf\u2026"
updated: "2026-03-21"
---

# Patch: Evaluate and expose policy runner via MCP

## Summary

Evaluate whether MCP adds value as an interoperability layer for the policy runner, then implement the initial MCP surface if justified. Scope: define the MCP tool contract for policy check/record/explain, keep enforcement host-side, and document the relationship between MCP transport and local policy config/evidence. Acceptance criteria: the repo either ships an initial MCP policy surface or documents a rejected/deferred decision with reasoning, and the result is consistent with the policy-runner design doc.

## Reason

This patch preserves legacy Dex history after the repository moved to repo-native workflow tracking with `mnemix-workflow`.

## Scope

- Preserve the original Dex task details for `3108zh39`.
- Keep the tracked status aligned with the final Dex state: `open`.
- Avoid reinterpreting the historical task beyond the recorded Dex description and outcome.

## Implementation Notes

- Imported from legacy Dex tracking during the 2026-03-30 workflow migration.
- Original Dex task ID: `3108zh39`
- Created: 2026-03-21
- Started: Not recorded
- Completed: Not recorded

## Validation

- This task was still open in Dex when the history was imported.
- Migration check: imported into `workflow/patches/` for repo-native tracking.

## References

- GitHub issue: #81 (micahcourey/mnemix) - https://github.com/micahcourey/mnemix/issues/81
- Original source: `.dex/tasks.jsonl`
