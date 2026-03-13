"""Shared utilities for host-specific Mnemix adapters."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from mnemix import Mnemix
from mnemix.models import (
    CheckpointRequest,
    CheckpointResult,
    MemoryDetail,
    RecallEntry,
    RecallRequest,
    StoreStats,
)


@dataclass(frozen=True)
class ContextBundle:
    """Prompt-ready context returned by host adapter recall helpers."""

    scope: str
    query: str | None
    prompt_preamble: str
    entries: list[RecallEntry]


@dataclass(frozen=True)
class CiRunContext:
    """Context bundle for CI runs, including an optional checkpoint."""

    bundle: ContextBundle
    checkpoint: CheckpointResult | None


class BaseAdapter:
    """Shared host-adapter utilities built on the public Mnemix client."""

    def __init__(self, store: Path | str = Path(".mnemix")) -> None:
        self._client = Mnemix(store=Path(store))

    def ensure_store(self) -> None:
        """Initialise the store if it does not already exist."""
        self._client.init()

    def get_stats(self, *, scope: str | None = None) -> StoreStats:
        """Return store statistics, optionally scoped to a single namespace."""
        return self._client.stats(scope=scope)

    def create_checkpoint(
        self,
        name: str,
        *,
        description: str | None = None,
    ) -> CheckpointResult:
        """Create a named checkpoint."""
        return self._client.checkpoint(
            CheckpointRequest(name=name, description=description)
        )

    def _context_bundle(
        self,
        *,
        scope: str,
        query: str | None,
        heading: str,
        limit: int,
        disclosure_depth: str = "summary_then_pinned",
    ) -> ContextBundle:
        result = self._client.recall(
            RecallRequest(
                text=query,
                scope=scope,
                disclosure_depth=disclosure_depth,
                limit=limit,
            )
        )
        entries = [*result.pinned_context, *result.summaries, *result.archival]
        return ContextBundle(
            scope=scope,
            query=query,
            prompt_preamble=self._render_preamble(heading, entries),
            entries=entries,
        )

    def _render_preamble(self, heading: str, entries: list[RecallEntry]) -> str:
        if not entries:
            return f"{heading}\n- No relevant memory recalled."

        lines = [heading]
        for entry in entries:
            lines.append(
                f"- [{entry.layer}] {entry.memory.title}: {entry.memory.summary}"
            )
        return "\n".join(lines)

    def _remember(
        self,
        *,
        memory_id: str,
        scope: str,
        kind: str,
        title: str,
        summary: str,
        detail: str,
        importance: int,
        tags: list[str] | None = None,
        pin_reason: str | None = None,
        source_tool: str | None = None,
    ) -> MemoryDetail:
        from mnemix.models import RememberRequest

        return self._client.remember(
            RememberRequest(
                id=memory_id,
                scope=scope,
                kind=kind,
                title=title,
                summary=summary,
                detail=detail,
                importance=importance,
                tags=tags or [],
                pin_reason=pin_reason,
                source_tool=source_tool,
            )
        )
