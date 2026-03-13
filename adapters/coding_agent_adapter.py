"""Mnemix adapter for coding-agent workflows."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Literal

from mnemix.models import OptimizeRequest, RestoreRequest

from ._adapter_base import BaseAdapter, ContextBundle

TaskMode = Literal["quick", "normal", "deep"]


@dataclass(frozen=True)
class CodingTaskContext:
    """Expanded task-start context for coding agents."""

    recall: ContextBundle
    pins: list
    recent_history: list
    prompt_preamble: str
    mode: TaskMode


class CodingAgentAdapter(BaseAdapter):
    """Workflow helpers for coding agents working on implementation tasks."""

    def start_task(
        self,
        *,
        scope: str,
        task_title: str,
        mode: TaskMode = "normal",
        recall_limit: int | None = None,
        pin_limit: int = 10,
        history_limit: int = 5,
    ) -> CodingTaskContext:
        disclosure_depth, default_limit = self._task_mode_config(mode)
        recall = self._context_bundle(
            scope=scope,
            query=task_title,
            heading="Relevant project memory for this coding task:",
            limit=recall_limit or default_limit,
            disclosure_depth=disclosure_depth,
        )
        pins = self.list_pins(scope=scope, limit=pin_limit)
        recent_history = self.review_recent_memory(scope=scope, limit=history_limit)
        return CodingTaskContext(
            recall=recall,
            pins=pins,
            recent_history=recent_history,
            prompt_preamble=self._render_task_briefing(
                recall=recall,
                pins=pins,
                recent_history=recent_history,
            ),
            mode=mode,
        )

    def search_memory(
        self,
        *,
        text: str,
        scope: str | None = None,
        limit: int = 10,
    ):
        return self._client.search(text, scope=scope, limit=limit)

    def load_memory(self, memory_id: str):
        return self._client.show(memory_id)

    def list_pins(
        self,
        *,
        scope: str | None = None,
        limit: int = 20,
    ):
        return self._client.pins(scope=scope, limit=limit)

    def review_recent_memory(
        self,
        *,
        scope: str | None = None,
        limit: int = 20,
    ):
        return self._client.history(scope=scope, limit=limit)

    def checkpoint_before_risky_change(
        self,
        *,
        task_id: str,
        description: str | None = None,
    ):
        return self.create_checkpoint(
            f"task-{task_id}",
            description=description or "Checkpoint before risky coding-agent change",
        )

    def list_versions(self, *, limit: int = 20):
        return self._client.versions(limit=limit)

    def restore_checkpoint(self, checkpoint: str):
        return self._client.restore(RestoreRequest(checkpoint=checkpoint))

    def restore_version(self, version: int):
        return self._client.restore(RestoreRequest(version=version))

    def optimize_store(
        self,
        *,
        prune: bool = False,
        older_than_days: int = 30,
    ):
        return self._client.optimize(
            OptimizeRequest(prune=prune, older_than_days=older_than_days)
        )

    def export_snapshot(self, destination):
        self._client.export(destination)

    def stage_import(self, source):
        self._client.import_store(source)

    def store_decision(
        self,
        *,
        memory_id: str,
        scope: str,
        title: str,
        summary: str,
        detail: str,
        pin_reason: str | None = None,
    ):
        return self._remember(
            memory_id=memory_id,
            scope=scope,
            kind="decision",
            title=title,
            summary=summary,
            detail=detail,
            importance=90,
            pin_reason=pin_reason,
            tags=["coding", "decision"],
            source_tool="coding-agent",
        )

    def store_summary(
        self,
        *,
        memory_id: str,
        scope: str,
        title: str,
        summary: str,
        detail: str,
    ):
        return self._remember(
            memory_id=memory_id,
            scope=scope,
            kind="summary",
            title=title,
            summary=summary,
            detail=detail,
            importance=75,
            tags=["coding", "summary"],
            source_tool="coding-agent",
        )

    def store_fact(
        self,
        *,
        memory_id: str,
        scope: str,
        title: str,
        summary: str,
        detail: str,
    ):
        return self._remember(
            memory_id=memory_id,
            scope=scope,
            kind="fact",
            title=title,
            summary=summary,
            detail=detail,
            importance=70,
            tags=["coding", "fact"],
            source_tool="coding-agent",
        )

    def store_procedure(
        self,
        *,
        memory_id: str,
        scope: str,
        title: str,
        summary: str,
        detail: str,
    ):
        return self._remember(
            memory_id=memory_id,
            scope=scope,
            kind="procedure",
            title=title,
            summary=summary,
            detail=detail,
            importance=80,
            tags=["coding", "procedure"],
            source_tool="coding-agent",
        )

    def store_pitfall(
        self,
        *,
        memory_id: str,
        scope: str,
        title: str,
        summary: str,
        detail: str,
    ):
        return self._remember(
            memory_id=memory_id,
            scope=scope,
            kind="warning",
            title=title,
            summary=summary,
            detail=detail,
            importance=85,
            tags=["coding", "pitfall"],
            source_tool="coding-agent",
        )

    def _task_mode_config(self, mode: TaskMode) -> tuple[str, int]:
        if mode == "quick":
            return ("summary_only", 6)
        if mode == "deep":
            return ("full", 12)
        return ("summary_then_pinned", 8)

    def _render_task_briefing(self, *, recall, pins, recent_history) -> str:
        lines = [recall.prompt_preamble]
        if pins:
            lines.append("")
            lines.append("Pinned memories relevant to the repo:")
            for memory in pins:
                lines.append(f"- {memory.title}: {memory.summary}")
        if recent_history:
            lines.append("")
            lines.append("Recent memory activity:")
            for memory in recent_history:
                lines.append(f"- {memory.title}: {memory.summary}")
        return "\n".join(lines)
