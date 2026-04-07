"""Reference coding-agent wrapper that composes with the policy runner."""

from __future__ import annotations

from pathlib import Path

from adapters import CodingAgentAdapter, CodingOutcome


def main() -> None:
    adapter = CodingAgentAdapter(store=Path(".mnemix"))
    adapter.ensure_store()

    workflow_key = "task-policy-runner-demo"
    scope = adapter.repo_scope("mnemix")

    task = adapter.start_task(
        scope=scope,
        task_title="Implement policy lifecycle commands",
        mode="normal",
        workflow_key=workflow_key,
        task_kind="feature",
        paths=["crates/mnemix-cli/src/cmd/policy.rs"],
    )

    print(task.prompt_preamble)
    if task.policy_decision is not None:
        print(f"policy decision after recall: {task.policy_decision.decision}")

    result = adapter.store_outcome(
        CodingOutcome(
            memory_id="policy-runner-lifecycle-demo",
            scope=scope,
            title="Added policy lifecycle support",
            summary="The workflow can now clear and cleanup policy evidence.",
            detail=(
                "Hosts can clear one workflow or cleanup expired task/session "
                "evidence without pushing policy state into storage."
            ),
            reusable=True,
            tags=["policy"],
        ),
        workflow_key=workflow_key,
    )
    print(f"stored: {result.stored}")
    print(f"recorded policy actions: {result.policy_recorded_actions}")


if __name__ == "__main__":
    main()
