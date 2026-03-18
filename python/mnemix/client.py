"""Mnemix Python binding — high-level client.

The :class:`Mnemix` client is the primary entry point for Python
consumers. It wraps the Rust ``mnemix`` CLI JSON surface; all product logic
remains in the Rust binary.

Example usage::

    from pathlib import Path
    from mnemix import Mnemix
    from mnemix.models import RememberRequest

    client = Mnemix(store=Path(".mnemix"))
    client.init()

    client.remember(RememberRequest(
        id="mem-001",
        scope="project-alpha",
        kind="observation",
        title="Initial scaffolding complete",
        summary="Rust project scaffold created with workspace layout.",
        detail="Added Cargo.toml workspace, core, lancedb, cli, types crates.",
    ))

    results = client.search("scaffolding", scope="project-alpha")
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
    PolicyCheckRequest,
    PolicyDecisionResult,
    PolicyRecordRequest,
    RecallRequest,
    RecallResult,
    RememberRequest,
    RestoreRequest,
    RestoreResult,
    SearchRequest,
    StatusResult,
    StoreStats,
    VersionRecord,
)

_DEFAULT_STORE = Path(".mnemix")


class Mnemix:
    """High-level Python client for a Mnemix store.

    All operations delegate to the ``mnemix`` CLI binary via its ``--json`` output
    mode.  No product logic is duplicated here.

    Args:
        store: Path to the store directory.  Defaults to ``.mnemix``
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
        """Initialise a new Mnemix store at :attr:`store`.

        Idempotent — safe to call on an existing store.
        """
        self._run("init", [])

    # ------------------------------------------------------------------
    # Memory operations
    # ------------------------------------------------------------------

    def remember(self, request: RememberRequest) -> MemoryDetail:
        """Store a memory record and return the persisted detail view.

        Args:
            request: A :class:`~mnemix.models.RememberRequest`
                describing the memory to persist.

        Returns:
            The fully hydrated :class:`~mnemix.models.MemoryDetail`.
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
            The :class:`~mnemix.models.MemoryDetail` for that record.
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
            A list of :class:`~mnemix.models.MemorySummary` objects.
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
            request: Optional :class:`~mnemix.models.RecallRequest`.
                Defaults to a summary-then-pinned recall with no text filter.

        Returns:
            A :class:`~mnemix.models.RecallResult` with layered
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
            A list of pinned :class:`~mnemix.models.MemorySummary` objects.
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
            A list of :class:`~mnemix.models.MemorySummary` objects.
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
            request: A :class:`~mnemix.models.CheckpointRequest`.

        Returns:
            The created :class:`~mnemix.models.CheckpointResult`.
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
            A list of :class:`~mnemix.models.VersionRecord` objects.
        """
        data = self._run("versions", ["--limit", str(limit)])
        return [VersionRecord.from_dict(v) for v in data.get("versions", [])]

    def restore(self, request: RestoreRequest) -> RestoreResult:
        """Restore the store to a previous checkpoint or version.

        A pre-restore checkpoint is automatically created by the CLI unless
        the store is already at the requested state.

        Args:
            request: A :class:`~mnemix.models.RestoreRequest` with
                either ``checkpoint`` or ``version`` set.

        Returns:
            A :class:`~mnemix.models.RestoreResult`.
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
            request: Optional :class:`~mnemix.models.OptimizeRequest`.
                Defaults to compact-only with no pruning.

        Returns:
            An :class:`~mnemix.models.OptimizeResult`.
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
            A :class:`~mnemix.models.StoreStats` snapshot.
        """
        args: list[str] = []
        if scope is not None:
            args += ["--scope", scope]
        data = self._run("stats", args)
        return StoreStats.from_dict(data["stats"])

    # ------------------------------------------------------------------
    # Policy runner
    # ------------------------------------------------------------------

    def policy_check(self, request: PolicyCheckRequest) -> PolicyDecisionResult:
        """Evaluate policy requirements for a workflow trigger."""
        data = self._run("policy", self._policy_check_args("check", request))
        return PolicyDecisionResult.from_dict(data)

    def policy_explain(self, request: PolicyCheckRequest) -> PolicyDecisionResult:
        """Explain the policy decision for a workflow trigger."""
        data = self._run("policy", self._policy_check_args("explain", request))
        return PolicyDecisionResult.from_dict(data)

    def policy_record(self, request: PolicyRecordRequest) -> StatusResult:
        """Record policy evidence for a workflow key."""
        args = [
            "record",
            "--workflow-key", request.workflow_key,
            "--action", request.action,
        ]
        if request.reason is not None:
            args += ["--reason", request.reason]
        data = self._run("policy", args)
        return StatusResult.from_dict(data)

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

    def _policy_check_args(
        self,
        action: str,
        request: PolicyCheckRequest,
    ) -> list[str]:
        args = [
            action,
            "--trigger", request.trigger,
        ]
        if request.workflow_key is not None:
            args += ["--workflow-key", request.workflow_key]
        if request.host is not None:
            args += ["--host", request.host]
        if request.task_kind is not None:
            args += ["--task-kind", request.task_kind]
        if request.scope is not None:
            args += ["--scope", request.scope]
        for path in request.paths:
            args += ["--path", path]
        return args
