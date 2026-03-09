"""Temporal Plane Python binding — typed request and response models.

All response models are plain ``dataclass`` instances built from the CLI JSON
contract.  The field names and types mirror the Rust ``output/mod.rs`` view
structs exactly so that future contract changes are obvious.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Literal


# ---------------------------------------------------------------------------
# Shared value types
# ---------------------------------------------------------------------------

MemoryKind = Literal[
    "observation",
    "decision",
    "preference",
    "summary",
    "fact",
    "procedure",
    "warning",
]

RecallLayer = Literal["pinned_context", "summary", "archival"]

RecallReason = Literal[
    "pinned",
    "scope_filter",
    "text_match",
    "summary_kind",
    "importance_boost",
    "recency_boost",
    "archival_expansion",
]

DisclosureDepth = Literal["summary_only", "summary_then_pinned", "full"]


# ---------------------------------------------------------------------------
# Request helpers
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class RememberRequest:
    """Parameters for the ``remember`` command.

    ``id`` and ``scope`` must be valid Temporal Plane identifiers (lowercase
    alphanumeric plus hyphens, non-empty).  All other fields default to
    sensible values.
    """

    id: str
    scope: str
    kind: MemoryKind
    title: str
    summary: str
    detail: str
    importance: int = 50
    confidence: int = 100
    tags: list[str] = field(default_factory=list)
    entities: list[str] = field(default_factory=list)
    pin_reason: str | None = None
    metadata: dict[str, str] = field(default_factory=dict)
    source_session_id: str | None = None
    source_tool: str | None = None
    source_ref: str | None = None


@dataclass(frozen=True)
class SearchRequest:
    """Parameters for the ``search`` command."""

    text: str
    scope: str | None = None
    limit: int = 10


@dataclass(frozen=True)
class RecallRequest:
    """Parameters for the ``recall`` command."""

    text: str | None = None
    scope: str | None = None
    disclosure_depth: DisclosureDepth = "summary_then_pinned"
    limit: int = 10


@dataclass(frozen=True)
class CheckpointRequest:
    """Parameters for the ``checkpoint`` command."""

    name: str
    description: str | None = None


@dataclass(frozen=True)
class RestoreRequest:
    """Parameters for the ``restore`` command.

    Exactly one of ``checkpoint`` or ``version`` must be provided.
    """

    checkpoint: str | None = None
    version: int | None = None

    def __post_init__(self) -> None:
        if (self.checkpoint is None) == (self.version is None):
            raise ValueError("Provide exactly one of 'checkpoint' or 'version'.")


@dataclass(frozen=True)
class OptimizeRequest:
    """Parameters for the ``optimize`` command."""

    prune: bool = False
    older_than_days: int = 30


# ---------------------------------------------------------------------------
# Response models
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class MemorySummary:
    """Lightweight memory record returned in list and search results."""

    id: str
    scope_id: str
    kind: str
    title: str
    summary: str
    pinned: bool
    pin_reason: str | None
    importance: int
    confidence: int
    created_at: str
    updated_at: str
    tags: list[str]
    entities: list[str]

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "MemorySummary":
        return cls(
            id=data["id"],
            scope_id=data["scope_id"],
            kind=data["kind"],
            title=data["title"],
            summary=data["summary"],
            pinned=data["pinned"],
            pin_reason=data.get("pin_reason"),
            importance=data["importance"],
            confidence=data["confidence"],
            created_at=data["created_at"],
            updated_at=data["updated_at"],
            tags=list(data.get("tags", [])),
            entities=list(data.get("entities", [])),
        )


@dataclass(frozen=True)
class MemoryDetail:
    """Full memory record including ``detail`` text and source metadata."""

    id: str
    scope_id: str
    kind: str
    title: str
    summary: str
    detail: str
    pinned: bool
    pin_reason: str | None
    importance: int
    confidence: int
    created_at: str
    updated_at: str
    source_session_id: str | None
    source_tool: str | None
    source_ref: str | None
    tags: list[str]
    entities: list[str]
    metadata: dict[str, str]

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "MemoryDetail":
        return cls(
            id=data["id"],
            scope_id=data["scope_id"],
            kind=data["kind"],
            title=data["title"],
            summary=data["summary"],
            detail=data["detail"],
            pinned=data["pinned"],
            pin_reason=data.get("pin_reason"),
            importance=data["importance"],
            confidence=data["confidence"],
            created_at=data["created_at"],
            updated_at=data["updated_at"],
            source_session_id=data.get("source_session_id"),
            source_tool=data.get("source_tool"),
            source_ref=data.get("source_ref"),
            tags=list(data.get("tags", [])),
            entities=list(data.get("entities", [])),
            metadata=dict(data.get("metadata", {})),
        )


@dataclass(frozen=True)
class CheckpointResult:
    """A checkpoint record returned by ``checkpoint`` and related commands."""

    name: str
    version: int
    created_at: str
    description: str | None

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "CheckpointResult":
        return cls(
            name=data["name"],
            version=data["version"],
            created_at=data["created_at"],
            description=data.get("description"),
        )


@dataclass(frozen=True)
class RecallEntry:
    """A single entry in a recall result with layer and explanation metadata."""

    layer: str
    reasons: list[str]
    memory: MemorySummary

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "RecallEntry":
        return cls(
            layer=data["layer"],
            reasons=list(data.get("reasons", [])),
            memory=MemorySummary.from_dict(data["memory"]),
        )


@dataclass(frozen=True)
class RecallResult:
    """Full recall result including layered pinned, summary, and archival entries."""

    scope: str | None
    query_text: str | None
    disclosure_depth: str
    count: int
    pinned_context: list[RecallEntry]
    summaries: list[RecallEntry]
    archival: list[RecallEntry]

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "RecallResult":
        return cls(
            scope=data.get("scope"),
            query_text=data.get("query_text"),
            disclosure_depth=data["disclosure_depth"],
            count=data["count"],
            pinned_context=[RecallEntry.from_dict(e) for e in data.get("pinned_context", [])],
            summaries=[RecallEntry.from_dict(e) for e in data.get("summaries", [])],
            archival=[RecallEntry.from_dict(e) for e in data.get("archival", [])],
        )


@dataclass(frozen=True)
class VersionRecord:
    """A single version entry from the ``versions`` command."""

    version: int
    recorded_at: str
    checkpoint_name: str | None
    checkpoint_version: int | None
    summary: str | None

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "VersionRecord":
        return cls(
            version=data["version"],
            recorded_at=data["recorded_at"],
            checkpoint_name=data.get("checkpoint_name"),
            checkpoint_version=data.get("checkpoint_version"),
            summary=data.get("summary"),
        )


@dataclass(frozen=True)
class RestoreResult:
    """Result of a ``restore`` operation."""

    previous_version: int
    restored_version: int
    current_version: int
    pre_restore_checkpoint: CheckpointResult | None

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "RestoreResult":
        pre = data.get("pre_restore_checkpoint")
        return cls(
            previous_version=data["previous_version"],
            restored_version=data["restored_version"],
            current_version=data["current_version"],
            pre_restore_checkpoint=CheckpointResult.from_dict(pre) if pre else None,
        )


@dataclass(frozen=True)
class OptimizeRetentionResult:
    """Retention settings that were active during an ``optimize`` operation.

    Mirrors ``OptimizeRetentionView`` in the CLI output layer.
    """

    minimum_age_days: int
    delete_unverified: bool
    error_if_tagged_old_versions: bool

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "OptimizeRetentionResult":
        return cls(
            minimum_age_days=data["minimum_age_days"],
            delete_unverified=data["delete_unverified"],
            error_if_tagged_old_versions=data["error_if_tagged_old_versions"],
        )


@dataclass(frozen=True)
class OptimizeResult:
    """Result of an ``optimize`` operation."""

    previous_version: int
    current_version: int
    compacted: bool
    prune_old_versions: bool
    pruned_versions: int
    bytes_removed: int
    retention: OptimizeRetentionResult
    pre_optimize_checkpoint: CheckpointResult | None

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "OptimizeResult":
        pre = data.get("pre_optimize_checkpoint")
        return cls(
            previous_version=data["previous_version"],
            current_version=data["current_version"],
            compacted=data["compacted"],
            prune_old_versions=data["prune_old_versions"],
            pruned_versions=data["pruned_versions"],
            bytes_removed=data["bytes_removed"],
            retention=OptimizeRetentionResult.from_dict(data["retention"]),
            pre_optimize_checkpoint=CheckpointResult.from_dict(pre) if pre else None,
        )


@dataclass(frozen=True)
class StoreStats:
    """Statistics snapshot returned by the ``stats`` command."""

    scope: str | None
    total_memories: int
    pinned_memories: int
    version_count: int
    latest_checkpoint: str | None

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "StoreStats":
        return cls(
            scope=data.get("scope"),
            total_memories=data["total_memories"],
            pinned_memories=data["pinned_memories"],
            version_count=data["version_count"],
            latest_checkpoint=data.get("latest_checkpoint"),
        )
