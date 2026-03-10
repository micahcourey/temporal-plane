"""Tests for mnemix.models — serialization and model construction."""

from __future__ import annotations

import pytest

from mnemix.models import (
    CheckpointRequest,
    CheckpointResult,
    MemoryDetail,
    MemorySummary,
    OptimizeRequest,
    OptimizeResult,
    RecallEntry,
    RecallRequest,
    RecallResult,
    RememberRequest,
    RestoreRequest,
    RestoreResult,
    SearchRequest,
    StoreStats,
    VersionRecord,
)


# ---------------------------------------------------------------------------
# RememberRequest
# ---------------------------------------------------------------------------


class TestRememberRequest:
    def test_defaults(self) -> None:
        req = RememberRequest(
            id="mem-001",
            scope="test-scope",
            kind="observation",
            title="T",
            summary="S",
            detail="D",
        )
        assert req.importance == 50
        assert req.confidence == 100
        assert req.tags == []
        assert req.entities == []
        assert req.pin_reason is None
        assert req.metadata == {}
        assert req.source_session_id is None
        assert req.source_tool is None
        assert req.source_ref is None

    def test_frozen(self) -> None:
        req = RememberRequest(
            id="mem-001",
            scope="s",
            kind="fact",
            title="T",
            summary="S",
            detail="D",
        )
        with pytest.raises(Exception):
            req.id = "mutated"  # type: ignore[misc]


# ---------------------------------------------------------------------------
# RestoreRequest
# ---------------------------------------------------------------------------


class TestRestoreRequest:
    def test_checkpoint_only(self) -> None:
        req = RestoreRequest(checkpoint="my-cp")
        assert req.checkpoint == "my-cp"
        assert req.version is None

    def test_version_only(self) -> None:
        req = RestoreRequest(version=42)
        assert req.version == 42

    def test_both_raises(self) -> None:
        with pytest.raises(ValueError, match="exactly one"):
            RestoreRequest(checkpoint="cp", version=1)

    def test_neither_raises(self) -> None:
        with pytest.raises(ValueError, match="exactly one"):
            RestoreRequest()


# ---------------------------------------------------------------------------
# SearchRequest
# ---------------------------------------------------------------------------


class TestSearchRequest:
    def test_defaults(self) -> None:
        req = SearchRequest(text="query")
        assert req.scope is None
        assert req.limit == 10


# ---------------------------------------------------------------------------
# RecallRequest
# ---------------------------------------------------------------------------


class TestRecallRequest:
    def test_defaults(self) -> None:
        req = RecallRequest()
        assert req.text is None
        assert req.scope is None
        assert req.disclosure_depth == "summary_then_pinned"
        assert req.limit == 10


# ---------------------------------------------------------------------------
# CheckpointRequest
# ---------------------------------------------------------------------------


class TestCheckpointRequest:
    def test_with_description(self) -> None:
        req = CheckpointRequest(name="v1", description="First stable release")
        assert req.name == "v1"
        assert req.description == "First stable release"

    def test_without_description(self) -> None:
        req = CheckpointRequest(name="v1")
        assert req.description is None


# ---------------------------------------------------------------------------
# OptimizeRequest
# ---------------------------------------------------------------------------


class TestOptimizeRequest:
    def test_defaults(self) -> None:
        req = OptimizeRequest()
        assert req.prune is False
        assert req.older_than_days == 30


# ---------------------------------------------------------------------------
# MemorySummary.from_dict
# ---------------------------------------------------------------------------

_SUMMARY_DICT = {
    "id": "mem-abc",
    "scope_id": "scope-x",
    "kind": "observation",
    "title": "Title",
    "summary": "Short summary",
    "pinned": False,
    "pin_reason": None,
    "importance": 60,
    "confidence": 90,
    "created_at": "2026-01-01T00:00:00Z",
    "updated_at": "2026-01-02T00:00:00Z",
    "tags": ["alpha"],
    "entities": ["entity-1"],
}


class TestMemorySummary:
    def test_from_dict(self) -> None:
        m = MemorySummary.from_dict(_SUMMARY_DICT)
        assert m.id == "mem-abc"
        assert m.scope_id == "scope-x"
        assert m.kind == "observation"
        assert m.pinned is False
        assert m.importance == 60
        assert m.tags == ["alpha"]
        assert m.entities == ["entity-1"]

    def test_missing_optional_fields(self) -> None:
        data = {**_SUMMARY_DICT}
        data.pop("tags", None)
        data.pop("entities", None)
        m = MemorySummary.from_dict(data)
        assert m.tags == []
        assert m.entities == []


# ---------------------------------------------------------------------------
# MemoryDetail.from_dict
# ---------------------------------------------------------------------------

_DETAIL_DICT = {
    **_SUMMARY_DICT,
    "detail": "Full detail text",
    "source_session_id": None,
    "source_tool": None,
    "source_ref": None,
    "metadata": {"key": "value"},
}


class TestMemoryDetail:
    def test_from_dict(self) -> None:
        m = MemoryDetail.from_dict(_DETAIL_DICT)
        assert m.detail == "Full detail text"
        assert m.metadata == {"key": "value"}
        assert m.source_tool is None

    def test_with_source_fields(self) -> None:
        data = {
            **_DETAIL_DICT,
            "source_session_id": "sess-1",
            "source_tool": "test-tool",
            "source_ref": "repo#L10",
        }
        m = MemoryDetail.from_dict(data)
        assert m.source_session_id == "sess-1"
        assert m.source_tool == "test-tool"
        assert m.source_ref == "repo#L10"


# ---------------------------------------------------------------------------
# CheckpointResult.from_dict
# ---------------------------------------------------------------------------


class TestCheckpointResult:
    def test_from_dict(self) -> None:
        cp = CheckpointResult.from_dict(
            {"name": "cp-1", "version": 5, "created_at": "2026-01-01T00:00:00Z", "description": "First"}
        )
        assert cp.name == "cp-1"
        assert cp.version == 5
        assert cp.description == "First"

    def test_no_description(self) -> None:
        cp = CheckpointResult.from_dict(
            {"name": "cp-1", "version": 5, "created_at": "2026-01-01T00:00:00Z"}
        )
        assert cp.description is None


# ---------------------------------------------------------------------------
# RecallEntry and RecallResult
# ---------------------------------------------------------------------------

_ENTRY_DICT = {
    "layer": "pinned_context",
    "reasons": ["pinned", "scope_filter"],
    "memory": _SUMMARY_DICT,
}

_RECALL_DICT = {
    "scope": "scope-x",
    "query_text": "test query",
    "disclosure_depth": "summary_then_pinned",
    "count": 1,
    "pinned_context": [_ENTRY_DICT],
    "summaries": [],
    "archival": [],
}


class TestRecallResult:
    def test_from_dict(self) -> None:
        result = RecallResult.from_dict(_RECALL_DICT)
        assert result.count == 1
        assert result.disclosure_depth == "summary_then_pinned"
        assert len(result.pinned_context) == 1
        assert result.pinned_context[0].layer == "pinned_context"
        assert result.pinned_context[0].reasons == ["pinned", "scope_filter"]
        assert result.pinned_context[0].memory.id == "mem-abc"

    def test_empty_layers(self) -> None:
        data = {**_RECALL_DICT, "pinned_context": [], "count": 0}
        result = RecallResult.from_dict(data)
        assert result.pinned_context == []


# ---------------------------------------------------------------------------
# VersionRecord
# ---------------------------------------------------------------------------


class TestVersionRecord:
    def test_from_dict(self) -> None:
        v = VersionRecord.from_dict(
            {
                "version": 3,
                "recorded_at": "2026-01-03T00:00:00Z",
                "checkpoint_name": "v1",
                "checkpoint_version": 1,
                "summary": None,
            }
        )
        assert v.version == 3
        assert v.checkpoint_name == "v1"
        assert v.summary is None


# ---------------------------------------------------------------------------
# RestoreResult
# ---------------------------------------------------------------------------


class TestRestoreResult:
    def test_from_dict_with_checkpoint(self) -> None:
        cp = {"name": "pre-restore", "version": 4, "created_at": "2026-01-04T00:00:00Z"}
        data = {
            "command": "restore",
            "target": {"kind": "checkpoint", "name": "cp-1", "version": 2},
            "previous_version": 4,
            "restored_version": 2,
            "current_version": 5,
            "pre_restore_checkpoint": cp,
        }
        result = RestoreResult.from_dict(data)
        assert result.previous_version == 4
        assert result.restored_version == 2
        assert result.current_version == 5
        assert result.pre_restore_checkpoint is not None
        assert result.pre_restore_checkpoint.name == "pre-restore"

    def test_from_dict_no_checkpoint(self) -> None:
        data = {
            "previous_version": 2,
            "restored_version": 1,
            "current_version": 3,
            "pre_restore_checkpoint": None,
        }
        result = RestoreResult.from_dict(data)
        assert result.pre_restore_checkpoint is None


# ---------------------------------------------------------------------------
# OptimizeResult
# ---------------------------------------------------------------------------


class TestOptimizeResult:
    def test_from_dict(self) -> None:
        data = {
            "previous_version": 3,
            "current_version": 4,
            "compacted": True,
            "prune_old_versions": False,
            "pruned_versions": 0,
            "bytes_removed": 1024,
            "retention": {"minimum_age_days": 30, "delete_unverified": False, "error_if_tagged_old_versions": True},
            "pre_optimize_checkpoint": None,
        }
        result = OptimizeResult.from_dict(data)
        assert result.compacted is True
        assert result.bytes_removed == 1024
        assert result.pre_optimize_checkpoint is None
        assert result.retention.minimum_age_days == 30
        assert result.retention.delete_unverified is False
        assert result.retention.error_if_tagged_old_versions is True


# ---------------------------------------------------------------------------
# StoreStats
# ---------------------------------------------------------------------------


class TestStoreStats:
    def test_from_dict(self) -> None:
        s = StoreStats.from_dict(
            {
                "scope": None,
                "total_memories": 42,
                "pinned_memories": 5,
                "version_count": 10,
                "latest_checkpoint": "v2",
            }
        )
        assert s.total_memories == 42
        assert s.pinned_memories == 5
        assert s.latest_checkpoint == "v2"
