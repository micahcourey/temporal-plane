"""Minimal host-side wrapper for selective Mnemix usage.

This example is intentionally generic so it can be adapted to tools such as
Codex, Cursor, Claude Code, or a custom agent runner.

The wrapper does three things:

1. recall relevant memory before a non-trivial task
2. build a compact context block for the agent
3. write back only durable learnings after the task
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from adapters import CodingAgentAdapter, CodingOutcome


@dataclass(frozen=True)
class AgentTask:
    repo_name: str
    title: str
    prompt: str
    trivial: bool = False


@dataclass(frozen=True)
class AgentResult:
    response_text: str
    durable_summary: str | None = None
    durable_detail: str | None = None
    should_store: bool = False


class AgentMemoryLayer:
    """Wraps an agent runtime with selective Mnemix recall and writeback."""

    def __init__(self, store: Path | str = Path(".mnemix")) -> None:
        self._adapter = CodingAgentAdapter(store=store)
        self._adapter.ensure_store()

    def run_task(self, task: AgentTask) -> AgentResult:
        context_block = ""
        if not task.trivial:
            context_block = self._build_context_block(task)

        full_prompt = self._compose_prompt(task.prompt, context_block)
        result = self._run_agent(full_prompt)

        if result.should_store:
            self._store_learning(task, result)

        return result

    def _build_context_block(self, task: AgentTask) -> str:
        context = self._adapter.start_task(
            scope=self._adapter.repo_scope(task.repo_name),
            task_title=task.title,
        )
        return context.prompt_preamble

    def _compose_prompt(self, prompt: str, context_block: str) -> str:
        if not context_block:
            return prompt
        return (
            "Relevant project memory:\n"
            f"{context_block}\n\n"
            "Use this context when it materially affects the task.\n\n"
            f"Task:\n{prompt}"
        )

    def _run_agent(self, prompt: str) -> AgentResult:
        # Replace this stub with the actual host tool call.
        #
        # Examples:
        # - Cursor extension hook
        # - Codex task runner
        # - Claude Code wrapper command
        # - internal orchestration service
        del prompt
        return AgentResult(
            response_text="Implement feature X using the recalled project context.",
            durable_summary="Use recall for multi-step agent tasks and write back only durable learnings.",
            durable_detail=(
                "This wrapper keeps memory selective: recall happens before prompt "
                "assembly, while writeback happens only for decisions, procedures, "
                "preferences, or repeated pitfalls that will matter later."
            ),
            should_store=True,
        )

    def _store_learning(self, task: AgentTask, result: AgentResult) -> None:
        if result.durable_summary is None or result.durable_detail is None:
            return

        memory_id = (
            task.title.lower()
            .replace(" ", "-")
            .replace("/", "-")
            .replace(":", "-")
        )

        self._adapter.store_outcome(
            CodingOutcome(
                memory_id=f"memory:{memory_id}",
                scope=self._adapter.repo_scope(task.repo_name),
                title=task.title,
                summary=result.durable_summary,
                detail=result.durable_detail,
                reusable=True,
                tags=["agents", "memory"],
                source_tool="agent-wrapper",
            )
        )


if __name__ == "__main__":
    layer = AgentMemoryLayer()
    result = layer.run_task(
        AgentTask(
            repo_name="mnemix",
            title="Use Mnemix selectively in agent workflows",
            prompt="Draft project guidance for intelligent memory usage.",
        )
    )
    print(result.response_text)
