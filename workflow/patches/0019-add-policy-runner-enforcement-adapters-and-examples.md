---
status: open
summary: "Implement the first enforcement-oriented policy-runner integrations."
updated: "2026-03-21"
---

# Patch: Add policy runner enforcement adapters and examples

## Summary

Implement the first enforcement-oriented policy-runner integrations. Scope: add reference git-hook integration, wrapper CLI examples, and CI/PR policy examples that honor policy check results; keep enforcement host-side rather than in storage or MCP. Acceptance criteria: the repo includes documented reference integrations for local hooks and CI checkpoints, examples show how block/require_action decisions are handled, and verification covers the expected command flow for each example.

## Reason

This patch preserves legacy Dex history after the repository moved to repo-native workflow tracking with `mnemix-workflow`.

## Scope

- Preserve the original Dex task details for `wjqmxdqh`.
- Keep the tracked status aligned with the final Dex state: `open`.
- Avoid reinterpreting the historical task beyond the recorded Dex description and outcome.

## Implementation Notes

- Imported from legacy Dex tracking during the 2026-03-30 workflow migration.
- Original Dex task ID: `wjqmxdqh`
- Created: 2026-03-21
- Started: Not recorded
- Completed: Not recorded

## Validation

- This task was still open in Dex when the history was imported.
- Migration check: imported into `workflow/patches/` for repo-native tracking.

## References

- Original source: `.dex/tasks.jsonl`
