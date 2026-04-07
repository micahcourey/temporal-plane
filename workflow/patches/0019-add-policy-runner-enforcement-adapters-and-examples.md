---
status: completed
summary: "Shipped reference policy-runner enforcement examples for wrappers, local hooks, and CI/PR flows."
updated: "2026-04-05"
---

# Patch: Add policy runner enforcement adapters and examples

## Summary

Implement the first enforcement-oriented policy-runner integrations. Scope: add reference git-hook integration, wrapper CLI examples, and CI/PR policy examples that honor policy check results; keep enforcement host-side rather than in storage or MCP. Acceptance criteria: the repo includes documented reference integrations for local hooks and CI checkpoints, examples show how block/require_action decisions are handled, and verification covers the expected command flow for each example.

## Reason

This patch preserves legacy Dex history after the repository moved to repo-native workflow tracking with `mnemix-workflow`.

## Scope

- Add a reference coding-agent wrapper flow
- Add a local pre-commit style enforcement example
- Add a CI/PR policy verification example
- Keep enforcement host-side instead of moving it into storage or MCP

## Implementation Notes

- Added `examples/policy-runner/README.md`
- Added `examples/policy-runner/coding_agent_wrapper.py`
- Added `examples/policy-runner/pre_commit_policy.sh`
- Added `examples/policy-runner/ci_policy_check.sh`
- Updated the policy-runner and host-adapter guides to point at the new
  enforcement examples

## Validation

- Verified policy-related Python and adapter tests through the repo virtualenv
- Kept the example flows host-side and based on `policy check`, `policy
  explain`, and `policy record`

## References

- Original source: `.dex/tasks.jsonl`
