"""Mnemix adapter for coding-agent workflows."""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from typing import Literal

from mnemix.models import OptimizeRequest, RestoreRequest

from ._adapter_base import BaseAdapter, ContextBundle

TaskMode = Literal["quick", "normal", "deep"]
OutcomeKind = Literal["skip", "decision", "procedure", "summary", "fact", "warning"]
MemoryOutcomeKind = Literal["decision", "procedure", "summary", "fact", "warning"]


@dataclass(frozen=True)
class CodingTaskContext:
    """Expanded task-start context for coding agents."""

    recall: ContextBundle
    pins: list
    recent_history: list
    prompt_preamble: str
    mode: TaskMode


@dataclass(frozen=True)
class CodingOutcome:
    """Durable outcome candidate produced by a coding-agent task."""

    memory_id: str
    scope: str
    title: str
    summary: str
    detail: str
    kind_hint: OutcomeKind | None = None
    reusable: bool = False
    architecture_relevant: bool = False
    recurring_failure: bool = False
    stable_fact: bool = False
    session_summary: bool = False
    low_signal: bool = False
    should_pin: bool = False
    pin_reason: str | None = None
    importance: int | None = None
    confidence: int = 100
    tags: list[str] = field(default_factory=list)
    entities: list[str] = field(default_factory=list)
    source_tool: str | None = None
    source_session_id: str | None = None
    source_ref: str | None = None
    metadata: dict[str, str] = field(default_factory=dict)


@dataclass(frozen=True)
class OutcomeClassification:
    """Classification result for a coding outcome."""

    kind: OutcomeKind
    reason: str
    importance: int | None = None
    pin_reason: str | None = None
    tags: list[str] = field(default_factory=list)


@dataclass(frozen=True)
class StoredOutcomeResult:
    """Result of attempting to store a classified coding outcome."""

    stored: bool
    classification: OutcomeClassification
    memory: object | None = None


class CodingAgentAdapter(BaseAdapter):
    """Workflow helpers for coding agents working on implementation tasks."""

    @staticmethod
    def repo_scope(repo: str) -> str:
        return f"repo:{CodingAgentAdapter._slug(repo)}"

    @staticmethod
    def workspace_scope(workspace: str) -> str:
        return f"workspace:{CodingAgentAdapter._slug(workspace)}"

    @staticmethod
    def session_scope(session_id: str) -> str:
        return f"session:{CodingAgentAdapter._slug(session_id)}"

    @staticmethod
    def task_scope(repo: str, task_id: str) -> str:
        return f"task:{CodingAgentAdapter._slug(repo)}-{CodingAgentAdapter._slug(task_id)}"

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

    def classify_outcome(self, outcome: CodingOutcome) -> OutcomeClassification:
        if outcome.kind_hint is not None:
            return self._classification_for_explicit_hint(outcome)

        if outcome.low_signal:
            return OutcomeClassification(
                kind="skip",
                reason="Outcome marked as low-signal and should not be stored.",
            )

        if outcome.architecture_relevant:
            return OutcomeClassification(
                kind="decision",
                reason="Architecture-relevant outcomes should be stored as decisions.",
                importance=outcome.importance or 90,
                pin_reason=outcome.pin_reason or self._default_pin_reason(outcome, "decision"),
                tags=["coding", "decision", *outcome.tags],
            )

        if outcome.recurring_failure:
            return OutcomeClassification(
                kind="warning",
                reason="Recurring failures should be stored as warnings.",
                importance=outcome.importance or 85,
                pin_reason=outcome.pin_reason if outcome.should_pin else None,
                tags=["coding", "pitfall", *outcome.tags],
            )

        if outcome.session_summary:
            return OutcomeClassification(
                kind="summary",
                reason="Session rollups should be stored as summaries.",
                importance=outcome.importance or 75,
                pin_reason=outcome.pin_reason if outcome.should_pin else None,
                tags=["coding", "summary", *outcome.tags],
            )

        if outcome.stable_fact:
            return OutcomeClassification(
                kind="fact",
                reason="Stable project facts should be stored as facts.",
                importance=outcome.importance or 70,
                pin_reason=outcome.pin_reason if outcome.should_pin else None,
                tags=["coding", "fact", *outcome.tags],
            )

        if outcome.reusable:
            return OutcomeClassification(
                kind="procedure",
                reason="Reusable implementation steps should be stored as procedures.",
                importance=outcome.importance or 80,
                pin_reason=outcome.pin_reason if outcome.should_pin else None,
                tags=["coding", "procedure", *outcome.tags],
            )

        return OutcomeClassification(
            kind="skip",
            reason="Outcome did not indicate durable signal worth storing.",
        )

    def store_outcome(self, outcome: CodingOutcome) -> StoredOutcomeResult:
        classification = self.classify_outcome(outcome)
        if classification.kind == "skip":
            return StoredOutcomeResult(stored=False, classification=classification)

        memory = self._remember(
            memory_id=outcome.memory_id,
            scope=outcome.scope,
            kind=classification.kind,
            title=outcome.title,
            summary=outcome.summary,
            detail=outcome.detail,
            importance=classification.importance or 50,
            confidence=outcome.confidence,
            tags=classification.tags,
            entities=outcome.entities,
            pin_reason=classification.pin_reason,
            source_tool=outcome.source_tool or "coding-agent",
            source_session_id=outcome.source_session_id,
            source_ref=outcome.source_ref,
            metadata=outcome.metadata,
        )
        return StoredOutcomeResult(
            stored=True,
            classification=classification,
            memory=memory,
        )

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

    def _classification_for_explicit_hint(
        self, outcome: CodingOutcome
    ) -> OutcomeClassification:
        if outcome.kind_hint == "skip":
            return OutcomeClassification(
                kind="skip",
                reason="Outcome explicitly marked to skip writeback.",
            )

        base_tags = {
            "decision": ["coding", "decision"],
            "procedure": ["coding", "procedure"],
            "summary": ["coding", "summary"],
            "fact": ["coding", "fact"],
            "warning": ["coding", "pitfall"],
        }[outcome.kind_hint]
        default_importance = {
            "decision": 90,
            "procedure": 80,
            "summary": 75,
            "fact": 70,
            "warning": 85,
        }[outcome.kind_hint]
        pin_reason = outcome.pin_reason if outcome.should_pin or outcome.pin_reason else None
        if outcome.kind_hint == "decision" and outcome.should_pin:
            pin_reason = outcome.pin_reason or self._default_pin_reason(outcome, "decision")

        return OutcomeClassification(
            kind=outcome.kind_hint,
            reason=f"Outcome explicitly classified as {outcome.kind_hint}.",
            importance=outcome.importance or default_importance,
            pin_reason=pin_reason,
            tags=[*base_tags, *outcome.tags],
        )

    def _task_mode_config(self, mode: TaskMode) -> tuple[str, int]:
        if mode == "quick":
            return ("summary_only", 6)
        if mode == "deep":
            return ("full", 12)
        return ("summary_then_pinned", 8)

    def _default_pin_reason(
        self, outcome: CodingOutcome, kind: MemoryOutcomeKind
    ) -> str | None:
        if not outcome.should_pin and outcome.pin_reason is None:
            return None
        if kind == "decision":
            return outcome.pin_reason or "Pinned coding-agent decision"
        return outcome.pin_reason or "Pinned coding-agent memory"

    @staticmethod
    def _slug(value: str) -> str:
        slug = re.sub(r"[^a-z0-9-]+", "-", value.lower())
        slug = re.sub(r"-{2,}", "-", slug).strip("-")
        return slug or "default"

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
