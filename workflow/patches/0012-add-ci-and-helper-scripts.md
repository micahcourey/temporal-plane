---
status: completed
summary: "Added the initial engineering automation."
updated: "2026-03-08"
---

# Patch: Add CI and helper scripts

## Summary

Create the initial engineering automation for Milestone 0. Scope: add GitHub workflow files for fmt/clippy/test/doc checks, plus minimal helper scripts if useful for local verification. Keep CI aligned with the roadmap and Rust best practices: formatting, linting, testing, and documentation generation from day one. Acceptance criteria: CI files exist, commands map to the workspace layout, and the automation is suitable for validating the empty scaffold before feature work begins.

## Reason

This patch preserves legacy Dex history after the repository moved to repo-native workflow tracking with `mnemix-workflow`.

## Scope

- Preserve the original Dex task details for `7rv7tddw`.
- Keep the tracked status aligned with the final Dex state: `completed`.
- Avoid reinterpreting the historical task beyond the recorded Dex description and outcome.

## Implementation Notes

- Imported from legacy Dex tracking during the 2026-03-30 workflow migration.
- Original Dex task ID: `7rv7tddw`
- Created: 2026-03-08
- Started: 2026-03-08
- Completed: 2026-03-08
- Recorded outcome:
  Added the initial engineering automation. Created GitHub workflow files for CI, docs presence checks, and release-preparation verification, plus scripts/check.sh and scripts/release.sh. Key decisions: make fmt, clippy, test, doc, and cargo-deny part of the baseline from day one; keep release automation conservative and verification-oriented at this stage

## Validation

- Dex verification notes: workflow files were created against the actual workspace layout and helper scripts were made executable.
- Migration check: imported into `workflow/patches/` for repo-native tracking.

## References

- Original source: `.dex/tasks.jsonl`
- Parent Dex task: `iiu4lrsm`
- Blocked by: `3211xb7o`
- Blocks: `vybahklg`
