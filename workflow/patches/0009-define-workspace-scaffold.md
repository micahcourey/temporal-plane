---
status: completed
summary: "Defined the initial repository scaffold and documented it in docs/repo-scaffold-spec.md."
updated: "2026-03-08"
---

# Patch: Define workspace scaffold

## Summary

Break down Milestone 0 into a concrete repository structure aligned with docs/mnemix-roadmap.md. Scope: confirm the top-level workspace layout, root files, crate boundaries, test-support placement, docs placement, examples layout, and scripts folder. Reference docs/mnemix-plan-v3.md, docs/mnemix-roadmap.md, docs/lancedb-rust-sdk-agent-guide.md, docs/coding-guidlines/rust-api-guidelines/checklist.md, and docs/coding-guidlines/rust-best-practices/README.md. Acceptance criteria: the intended scaffold is explicit enough to implement without redesign during file creation, crate boundaries are clear, and the layout stays Rust-workspace-first and local-first.

## Reason

This patch preserves legacy Dex history after the repository moved to repo-native workflow tracking with `mnemix-workflow`.

## Scope

- Preserve the original Dex task details for `9fs1hq2y`.
- Keep the tracked status aligned with the final Dex state: `completed`.
- Avoid reinterpreting the historical task beyond the recorded Dex description and outcome.

## Implementation Notes

- Imported from legacy Dex tracking during the 2026-03-30 workflow migration.
- Original Dex task ID: `9fs1hq2y`
- Created: 2026-03-08
- Started: 2026-03-08
- Completed: 2026-03-08
- Recorded outcome:
  Defined the initial repository scaffold and documented it in docs/repo-scaffold-spec.md. Locked the top-level workspace shape, crate boundaries, placeholder policy, and non-Rust scaffold directories so milestone 0 implementation could proceed without repo redesign. Key decisions: keep a Rust-workspace-first layout, include mnemix-types from the start, and scaffold Python/adapters/examples early without giving them implementation ownership

## Validation

- Dex verification notes: scaffold spec written and aligned with docs/mnemix-roadmap.md, docs/mnemix-plan-v3.md, and docs/lancedb-rust-sdk-agent-guide.md.
- Migration check: imported into `workflow/patches/` for repo-native tracking.

## References

- Original source: `.dex/tasks.jsonl`
- Parent Dex task: `iiu4lrsm`
- Blocks: `86cpw02k`
