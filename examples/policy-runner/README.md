# Policy Runner Example

This example bundle shows host-side enforcement patterns built on the policy
runner rather than baking workflow rules into storage.

Included references:

- `coding_agent_wrapper.py`: a Python wrapper that uses `CodingAgentAdapter`
  policy composition for task start and durable writeback
- `pre_commit_policy.sh`: a local commit gate that blocks when `policy check`
  returns `block` or `require_action`
- `ci_policy_check.sh`: a CI-friendly wrapper that evaluates PR or review
  policy before continuing

## Why host-side examples

Mnemix can explain and record workflow evidence, but the host remains the place
that decides whether to stop a commit, fail CI, or continue after guidance.

That keeps:

- storage generic
- policy inspectable
- enforcement explicit

## Typical sequence

1. Build a workflow key such as `commit-<sha>` or `pr-<number>`.
2. Run `mnemix policy check` or `policy explain`.
3. Record evidence like `recall`, `checkpoint`, or `writeback` when it really
   happens.
4. Re-run `policy explain` before the host proceeds.
5. Clear the workflow or cleanup stale task/session evidence later.
