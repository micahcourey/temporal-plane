---
status: completed
summary: "Reviewed the temporary imported coding guideline sources and distilled the project-relevant rules into docs/coding-guid\u2026"
updated: "2026-03-08"
---

# Patch: Distill agent coding guidelines

## Summary

Review the full docs/coding-guidlines source material, extract only the project-relevant rules for Mnemix, create a strict distilled guideline document for agents, update references to point to the distilled document, and remove the temporary upstream guideline copies before the PR merges. Acceptance criteria: a single distilled guideline document remains as the source of truth, AGENTS.md and planning docs reference it where appropriate, and the temporary coding-guidlines contents are removed from the repo.

## Reason

This patch preserves legacy Dex history after the repository moved to repo-native workflow tracking with `mnemix-workflow`.

## Scope

- Preserve the original Dex task details for `z4bne753`.
- Keep the tracked status aligned with the final Dex state: `completed`.
- Avoid reinterpreting the historical task beyond the recorded Dex description and outcome.

## Implementation Notes

- Imported from legacy Dex tracking during the 2026-03-30 workflow migration.
- Original Dex task ID: `z4bne753`
- Created: 2026-03-08
- Started: 2026-03-08
- Completed: 2026-03-08
- Recorded outcome:
  Reviewed the temporary imported coding guideline sources and distilled the project-relevant rules into docs/coding-guidlines/mnemix-agent-guidelines.md. The distilled guide now captures strict Mnemix-specific rules for architecture boundaries, Rust API design, error handling, testing, docs, dependency hygiene, CI discipline, performance/ownership, Python binding boundaries, and LanceDB leakage prevention. Updated AGENTS.md, docs/mnemix-roadmap.md, docs/repo-scaffold-spec.md, and the Milestone 1 Dex task to point at the distilled guide. Removed the temporary imported contents from docs/coding-guidlines so only the distilled guideline file remains

## Validation

- Dex verification notes: references now point to the new file, docs validation shows no errors, and the directory now contains only docs/coding-guidlines/mnemix-agent-guidelines.md.
- Migration check: imported into `workflow/patches/` for repo-native tracking.

## References

- GitHub issue: #10 (micahcourey/mnemix) - https://github.com/micahcourey/mnemix/issues/10
- Original source: `.dex/tasks.jsonl`
