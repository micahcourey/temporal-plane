---
status: completed
summary: "Completed the full project rebrand from Temporal Plane to Mnemix."
updated: "2026-03-10"
---

# Patch: Rebrand Temporal Plane to Mnemix

## Summary

Rebrand the project from Temporal Plane to Mnemix across the repo's public surfaces, package names, CLI names, crate names, docs, examples, workflows, and AI context. Acceptance criteria: README/docs use Mnemix and mnemix, the CLI/package/import/crate names are updated coherently, and validation passes for the fully renamed public surfaces with no remaining Temporal Plane identifiers in repo-owned paths or content.

## Reason

This patch preserves legacy Dex history after the repository moved to repo-native workflow tracking with `mnemix-workflow`.

## Scope

- Preserve the original Dex task details for `az0qh4l1`.
- Keep the tracked status aligned with the final Dex state: `completed`.
- Avoid reinterpreting the historical task beyond the recorded Dex description and outcome.

## Implementation Notes

- Imported from legacy Dex tracking during the 2026-03-30 workflow migration.
- Original Dex task ID: `az0qh4l1`
- Created: 2026-03-10
- Started: 2026-03-10
- Completed: 2026-03-10
- Recorded outcome:
  Completed the full project rebrand from Temporal Plane to Mnemix. Renamed the Rust workspace crates, CLI binary, Python package and imports, adapter module, canonical docs, examples, workflows, AGENTS/instruction files, AI context, and repository metadata. Removed Temporal Plane identifiers from repo-owned paths and content, updated the local git remote to the renamed GitHub repository, and validated the result with the full Python test suite, adapter tests, and ./scripts/check.sh.

## Validation

- No separate verification section was recorded in Dex beyond the task result.
- Migration check: imported into `workflow/patches/` for repo-native tracking.

## References

- GitHub issue: #26 (micahcourey/mnemix) - https://github.com/micahcourey/mnemix/issues/26
- Original source: `.dex/tasks.jsonl`
