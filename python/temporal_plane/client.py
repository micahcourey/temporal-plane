"""Temporal Plane Python binding — high-level client.

The :class:`TemporalPlane` client is the primary entry point for Python
consumers.  It wraps the Rust ``tp`` CLI JSON surface; all product logic
remains in the Rust binary.

Example usage::

    from pathlib import Path
    from temporal_plane import TemporalPlane
    from temporal_plane.models import RememberRequest

    tp = TemporalPlane(store=Path(".temporal-plane"))
    tp.init()

    tp.remember(RememberRequest(
        id="mem-001",
        scope="project-alpha",
        kind="observation",
        title="Initial scaffolding complete",
        summary="Rust project scaffold created with workspace layout.",
        detail="Added Cargo.toml workspace, core, lancedb, cli, types crates.",
    ))

    results = tp.search("scaffolding", scope="project-alpha")
    for m in results:
        print(m.title)
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

from . import _runner
from .models import (
    CheckpointRequest,
    CheckpointResult,
    MemoryDetail,
    MemorySummary,
    OptimizeRequest,
    OptimizeResult,
    RecallRequest,
    RecallResult,
    RememberRequest,
    RestoreRequest,
    RestoreResult,
    SearchRequest,
    StoreStats,
    VersionRecord,
)

_DEFAULT_STORE = Path(".temporal-plane")


class TemporalPlane:
    """High-level Python client for a Temporal Plane store.

    All operations delegate to the ``tp`` CLI binary via its ``--json`` output
    mode.  No product logic is duplicated here.

    Args:
        store: Path to the store directory.  Defaults to ``.temporal-plane``
            in the current working directory.
    """

    def __init__(self, store: Path | str = _DEFAULT_STORE) -> None:
        self._store = Path(store)

    # ------------------------------------------------------------------
    # Internal helpers
    # ------------------------------------------------------------------

    def _run(self, subcommand: str, args: list[str]) -> dict[str, Any]:
        return _runner.run(self._store, subcommand, args)

    # ------------------------------------------------------------------
    # Store lifecycle
    # ------------------------------------------------------------------

    def init(self) -> None:
        """Initialise a new Temporal Plane store at :attr:`store`.

        Idempotent — safe to call on an existing store.
        """
        self._run("init", [])

    # ------------------------------------------------------------------
    # Memory operations
    # ------------------------------------------------------------------

    def remember(self, request: RememberRequest) -> MemoryDetail:
        """Store a memory record and return the persisted detail view.

        Args:
            request: A :class:`~temporal_plane.models.RememberRequest`
                describing the memory to persist.

        Returns:
            The fully hydrated :class:`~temporal_plane.models.MemoryDetail`.
        """
        args = [
            "--id", request.id,
            "--scope", request.scope,
            "--kind", request.kind,
            "--title", request.title,
            "--summary", request.summary,
            "--detail", request.detail,
            "--importance", str(request.importance),
            "--confidence", str(request.confidence),
        ]
        for tag in request.tags:
            args += ["--tag", tag]
        for entity in request.entities:
            args += ["--entity", entity]
        if request.pin_reason is not None:
            args += ["--pin-reason", request.pin_reason]
        for key, value in request.metadata.items():
            args += ["--metadata", f"{key}={value}"]
        if request.source_session_id is not None:
            args += ["--source-session-id", request.source_session_id]
        if request.source_tool is not None:
            args += ["--source-tool", request.source_tool]
        if request.source_ref is not None:
            args += ["--source-ref", request.source_ref]

        data = self._run("remember", args)
        return MemoryDetail.from_dict(data["memory"])

    def show(self, memory_id: str) -> MemoryDetail:
        """Return the full detail view for a memory record.

        Args:
            memory_id: The memory identifier string.

        Returns:
            The :class:`~temporal_plane.models.MemoryDetail` for that record.
        """
        data = self._run("show", ["--id", memory_id])
        return MemoryDetail.from_dict(data["memory"])

    def search(
        self,
        text: str,
        *,
        scope: str | None = None,
        limit: int = 10,
    ) -> list[MemorySummary]:
        """Full-text search across memory records.

        Args:
            text: The search query string.
            scope: Optional scope filter.
            limit: Maximum number of results.

        Returns:
            A list of :class:`~temporal_plane.models.MemorySummary` objects.
        """
        req = SearchRequest(text=text, scope=scope, limit=limit)
        args = ["--text", req.text, "--limit", str(req.limit)]
        if req.scope is not None:
            args += ["--scope", req.scope]
        data = self._run("search", args)
        return [MemorySummary.from_dict(m) for m in data.get("memories", [])]

    def recall(self, request: RecallRequest | None = None) -> RecallResult:
        """Return a layered recall result.

        Args:
            request: Optional :class:`~temporal_plane.models.RecallRequest`.
                Defaults to a summary-then-pinned recall with no text filter.

        Returns:
            A :class:`~temporal_plane.models.RecallResult` with layered
            ``pinned_context``, ``summaries``, and ``archival`` entries.
        """
        if request is None:
            request = RecallRequest()
        args = [
            "--disclosure-depth", request.disclosure_depth,
            "--limit", str(request.limit),
        ]
        if request.text is not None:
            args += ["--text", request.text]
        if request.scope is not None:
            args += ["--scope", request.scope]
        data = self._run("recall", args)
        return RecallResult.from_dict(data)

    def pins(
        self,
        *,
        scope: str | None = None,
        limit: int = 20,
    ) -> list[MemorySummary]:
        """Return pinned memory records.

        Args:
            scope: Optional scope filter.
            limit: Maximum number of results.

        Returns:
            A list of pinned :class:`~temporal_plane.models.MemorySummary` objects.
        """
        args = ["--limit", str(limit)]
        if scope is not None:
            args += ["--scope", scope]
        data = self._run("pins", args)
        return [MemorySummary.from_dict(m) for m in data.get("memories", [])]

    def history(
        self,
        *,
        scope: str | None = None,
        limit: int = 20,
    ) -> list[MemorySummary]:
        """Return recent memory records in chronological order.

        Args:
            scope: Optional scope filter.
            limit: Maximum number of results.

        Returns:
            A list of :class:`~temporal_plane.models.MemorySummary` objects.
        """
        args = ["--limit", str(limit)]
        if scope is not None:
            args += ["--scope", scope]
        data = self._run("history", args)
        return [MemorySummary.from_dict(m) for m in data.get("memories", [])]

    # ------------------------------------------------------------------
    # Version and checkpoint operations
    # ------------------------------------------------------------------

    def checkpoint(self, request: CheckpointRequest) -> CheckpointResult:
        """Create a named checkpoint at the current store version.

        Args:
            request: A :class:`~temporal_plane.models.CheckpointRequest`.

        Returns:
            The created :class:`~temporal_plane.models.CheckpointResult`.
        """
        args = ["--name", request.name]
        if request.description is not None:
            args += ["--description", request.description]
        data = self._run("checkpoint", args)
        return CheckpointResult.from_dict(data["checkpoint"])

    def versions(self, *, limit: int = 20) -> list[VersionRecord]:
        """List store versions in descending order.

        Args:
            limit: Maximum number of versions to return.

        Returns:
            A list of :class:`~temporal_plane.models.VersionRecord` objects.
        """
        data = self._run("versions", ["--limit", str(limit)])
        return [VersionRecord.from_dict(v) for v in data.get("versions", [])]

    def restore(self, request: RestoreRequest) -> RestoreResult:
        """Restore the store to a previous checkpoint or version.

        A pre-restore checkpoint is automatically created by the CLI unless
        the store is already at the requested state.

        Args:
            request: A :class:`~temporal_plane.models.RestoreRequest` with
                either ``checkpoint`` or ``version`` set.

        Returns:
            A :class:`~temporal_plane.models.RestoreResult`.
        """
        if request.checkpoint is not None:
            args = ["--checkpoint", request.checkpoint]
        else:
            args = ["--version", str(request.version)]
        data = self._run("restore", args)
        return RestoreResult.from_dict(data)

    # ------------------------------------------------------------------
    # Maintenance
    # ------------------------------------------------------------------

    def optimize(self, request: OptimizeRequest | None = None) -> OptimizeResult:
        """Run compaction and optional version pruning.

        Args:
            request: Optional :class:`~temporal_plane.models.OptimizeRequest`.
                Defaults to compact-only with no pruning.

        Returns:
            An :class:`~temporal_plane.models.OptimizeResult`.
        """
        if request is None:
            request = OptimizeRequest()
        args = ["--older-than-days", str(request.older_than_days)]
        if request.prune:
            args.append("--prune")
        data = self._run("optimize", args)
        return OptimizeResult.from_dict(data)

    def stats(self, *, scope: str | None = None) -> StoreStats:
        """Return store statistics.

        Args:
            scope: Optional scope filter.

        Returns:
            A :class:`~temporal_plane.models.StoreStats` snapshot.
        """
        args: list[str] = []
        if scope is not None:
            args += ["--scope", scope]
        data = self._run("stats", args)
        return StoreStats.from_dict(data["stats"])

    def export(self, destination: Path | str) -> None:
        """Export the store to a file.

        Args:
            destination: Path where the export archive will be written.
        """
        self._run("export", ["--destination", str(destination)])

    def import_store(self, source: Path | str) -> None:
        """Import a previously exported store archive.

        Args:
            source: Path to the export archive to import.
        """
        self._run("import", ["--source", str(source)])
