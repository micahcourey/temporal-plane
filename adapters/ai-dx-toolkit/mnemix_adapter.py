"""AI DX Toolkit — Mnemix adapter.

This adapter bridges the AI DX Toolkit memory event model to the Mnemix
Python client.  It depends only on the public ``mnemix`` package API and
makes no assumptions about LanceDB internals.

Usage::

    from pathlib import Path
    from mnemix_adapter import MnemixAdapter

    adapter = MnemixAdapter(store=Path(".mnemix"))
    adapter.ensure_store()

    adapter.record_observation(
        id="obs-001",
        scope="my-agent",
        title="Dependency added",
        summary="Added httpx to pyproject.toml.",
        detail="Full context of the change and rationale.",
        tags=["dependencies"],
    )

    context = adapter.fetch_context(scope="my-agent", query="httpx dependency")
    for entry in context:
        print(entry.memory.title)
"""

from __future__ import annotations

from pathlib import Path

from mnemix import Mnemix
from mnemix.models import (
    CheckpointRequest,
    CheckpointResult,
    MemoryDetail,
    RecallEntry,
    RecallRequest,
    RememberRequest,
    StoreStats,
)


class MnemixAdapter:
    """Adapter that maps AI DX Toolkit memory events to Mnemix operations.

    This is the proof-of-concept first adapter.  It exposes a small, stable
    surface suitable for integration into higher-level agent frameworks.

    All operations delegate to :class:`~mnemix.Mnemix`; no
    product logic is duplicated in this adapter.

    Args:
        store: Path to the Mnemix store directory.
    """

    def __init__(self, store: Path | str = Path(".mnemix")) -> None:
        self._client = Mnemix(store=Path(store))

    # ------------------------------------------------------------------
    # Store lifecycle
    # ------------------------------------------------------------------

    def ensure_store(self) -> None:
        """Initialise the store if it does not already exist.

        Idempotent — safe to call on an existing store.
        """
        self._client.init()

    # ------------------------------------------------------------------
    # Memory recording helpers
    # ------------------------------------------------------------------

    def record_observation(
        self,
        *,
        id: str,
        scope: str,
        title: str,
        summary: str,
        detail: str,
        importance: int = 50,
        tags: list[str] | None = None,
        entities: list[str] | None = None,
        source_tool: str | None = None,
        source_session_id: str | None = None,
        metadata: dict[str, str] | None = None,
    ) -> MemoryDetail:
        """Record an observation memory from a toolkit event.

        Args:
            id: Unique memory identifier.
            scope: Scope identifier (e.g. agent name or project).
            title: Short, human-readable title.
            summary: Concise one-paragraph summary.
            detail: Full detail text.
            importance: Importance score 0-100 (default 50).
            tags: Optional list of tag strings.
            entities: Optional list of entity strings.
            source_tool: Name of the tool that generated this observation.
            source_session_id: Optional session identifier.
            metadata: Optional free-form metadata key-value pairs.

        Returns:
            The persisted :class:`~mnemix.models.MemoryDetail`.
        """
        return self._client.remember(
            RememberRequest(
                id=id,
                scope=scope,
                kind="observation",
                title=title,
                summary=summary,
                detail=detail,
                importance=importance,
                tags=tags or [],
                entities=entities or [],
                source_tool=source_tool,
                source_session_id=source_session_id,
                metadata=metadata or {},
            )
        )

    def record_decision(
        self,
        *,
        id: str,
        scope: str,
        title: str,
        summary: str,
        detail: str,
        importance: int = 70,
        pin_reason: str | None = None,
        tags: list[str] | None = None,
        source_tool: str | None = None,
    ) -> MemoryDetail:
        """Record a decision memory, optionally pinning it for persistent context.

        Args:
            id: Unique memory identifier.
            scope: Scope identifier.
            title: Short decision title.
            summary: One-paragraph decision summary.
            detail: Full rationale and context.
            importance: Importance score 0-100 (default 70).
            pin_reason: If provided, the decision is pinned with this reason.
            tags: Optional list of tag strings.
            source_tool: Name of the tool that generated this decision.

        Returns:
            The persisted :class:`~mnemix.models.MemoryDetail`.
        """
        return self._client.remember(
            RememberRequest(
                id=id,
                scope=scope,
                kind="decision",
                title=title,
                summary=summary,
                detail=detail,
                importance=importance,
                pin_reason=pin_reason,
                tags=tags or [],
                source_tool=source_tool,
            )
        )

    # ------------------------------------------------------------------
    # Context retrieval
    # ------------------------------------------------------------------

    def fetch_context(
        self,
        *,
        scope: str,
        query: str | None = None,
        limit: int = 10,
    ) -> list[RecallEntry]:
        """Retrieve layered context for an agent session.

        Returns pinned context first, then summaries.  Suitable for injecting
        into an agent prompt preamble.

        Args:
            scope: Scope to retrieve context for.
            query: Optional text query to focus the recall.
            limit: Maximum number of results.

        Returns:
            A flat list of :class:`~mnemix.models.RecallEntry` objects
            ordered by layer (pinned → summary → archival).
        """
        result = self._client.recall(
            RecallRequest(
                text=query,
                scope=scope,
                disclosure_depth="summary_then_pinned",
                limit=limit,
            )
        )
        return [*result.pinned_context, *result.summaries, *result.archival]

    # ------------------------------------------------------------------
    # Safety and lifecycle
    # ------------------------------------------------------------------

    def create_session_checkpoint(
        self,
        session_id: str,
        *,
        description: str | None = None,
    ) -> CheckpointResult:
        """Create a named checkpoint for the current agent session.

        Call this at the start of a significant operation or before a bulk
        memory import so the state can be safely restored.

        Args:
            session_id: A stable session identifier used as the checkpoint name.
            description: Optional human-readable description.

        Returns:
            The created :class:`~mnemix.models.CheckpointResult`.
        """
        return self._client.checkpoint(
            CheckpointRequest(name=session_id, description=description)
        )

    def get_stats(self, *, scope: str | None = None) -> StoreStats:
        """Return store statistics, optionally scoped to a single agent namespace.

        Args:
            scope: Optional scope filter.

        Returns:
            A :class:`~mnemix.models.StoreStats` snapshot.
        """
        return self._client.stats(scope=scope)
