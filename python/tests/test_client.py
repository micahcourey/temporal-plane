"""Tests for mnemix.client — subprocess boundary and client methods.

These tests use ``pytest-mock`` to patch ``mnemix._runner.run``
so no binary is required at test time.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any
from unittest.mock import MagicMock, patch

import pytest

from mnemix import Mnemix
from mnemix.errors import MnemixCommandError
from mnemix.models import (
    CheckpointRequest,
    OptimizeRequest,
    RecallRequest,
    RememberRequest,
    RestoreRequest,
)

# ---------------------------------------------------------------------------
# Shared fixtures
# ---------------------------------------------------------------------------

_MEMORY_SUMMARY = {
    "id": "mem-001",
    "scope_id": "scope-a",
    "kind": "observation",
    "title": "Test memory",
    "summary": "A test memory",
    "pinned": False,
    "pin_reason": None,
    "importance": 50,
    "confidence": 100,
    "created_at": "2026-01-01T00:00:00Z",
    "updated_at": "2026-01-01T00:00:00Z",
    "tags": [],
    "entities": [],
}

_MEMORY_DETAIL = {
    **_MEMORY_SUMMARY,
    "detail": "Full detail",
    "source_session_id": None,
    "source_tool": None,
    "source_ref": None,
    "metadata": {},
}

_CHECKPOINT = {
    "name": "cp-1",
    "version": 2,
    "created_at": "2026-01-01T00:00:00Z",
    "description": None,
}


@pytest.fixture()
def tp() -> Mnemix:
    return Mnemix(store=Path("/tmp/test-store"))


# ---------------------------------------------------------------------------
# init
# ---------------------------------------------------------------------------


class TestInit:
    def test_calls_init_subcommand(self, tp: Mnemix) -> None:
        with patch("mnemix._runner.run") as mock_run:
            mock_run.return_value = {"command": "init", "status": "ok", "message": "ready"}
            tp.init()
        mock_run.assert_called_once_with(Path("/tmp/test-store"), "init", [])


# ---------------------------------------------------------------------------
# remember
# ---------------------------------------------------------------------------


class TestRemember:
    def _request(self, **kwargs: Any) -> RememberRequest:
        return RememberRequest(
            id="mem-001",
            scope="scope-a",
            kind="observation",
            title="Test memory",
            summary="A test memory",
            detail="Full detail",
            **kwargs,
        )

    def test_returns_memory_detail(self, tp: Mnemix) -> None:
        with patch("mnemix._runner.run") as mock_run:
            mock_run.return_value = {"command": "remember", "action": "upsert", "memory": _MEMORY_DETAIL}
            result = tp.remember(self._request())
        assert result.id == "mem-001"
        assert result.detail == "Full detail"

    def test_includes_tags_and_entities(self, tp: Mnemix) -> None:
        req = self._request(tags=["alpha", "beta"], entities=["project-x"])
        with patch("mnemix._runner.run") as mock_run:
            mock_run.return_value = {"command": "remember", "action": "upsert", "memory": _MEMORY_DETAIL}
            tp.remember(req)
        args = mock_run.call_args[0][2]
        assert "--tag" in args
        assert "alpha" in args
        assert "--entity" in args
        assert "project-x" in args

    def test_includes_pin_reason(self, tp: Mnemix) -> None:
        req = self._request(pin_reason="Important decision")
        with patch("mnemix._runner.run") as mock_run:
            mock_run.return_value = {"command": "remember", "action": "upsert", "memory": _MEMORY_DETAIL}
            tp.remember(req)
        args = mock_run.call_args[0][2]
        assert "--pin-reason" in args
        assert "Important decision" in args

    def test_includes_metadata(self, tp: Mnemix) -> None:
        req = self._request(metadata={"key": "val"})
        with patch("mnemix._runner.run") as mock_run:
            mock_run.return_value = {"command": "remember", "action": "upsert", "memory": _MEMORY_DETAIL}
            tp.remember(req)
        args = mock_run.call_args[0][2]
        assert "--metadata" in args
        assert "key=val" in args


# ---------------------------------------------------------------------------
# show
# ---------------------------------------------------------------------------


class TestShow:
    def test_returns_memory_detail(self, tp: Mnemix) -> None:
        with patch("mnemix._runner.run") as mock_run:
            mock_run.return_value = {"command": "show", "memory": _MEMORY_DETAIL}
            result = tp.show("mem-001")
        assert result.id == "mem-001"
        mock_run.assert_called_once_with(Path("/tmp/test-store"), "show", ["--id", "mem-001"])


# ---------------------------------------------------------------------------
# search
# ---------------------------------------------------------------------------


class TestSearch:
    def test_returns_list(self, tp: Mnemix) -> None:
        with patch("mnemix._runner.run") as mock_run:
            mock_run.return_value = {
                "command": "search",
                "count": 1,
                "scope": None,
                "query_text": "test query",
                "memories": [_MEMORY_SUMMARY],
            }
            results = tp.search("test query")
        assert len(results) == 1
        assert results[0].id == "mem-001"

    def test_passes_scope_and_limit(self, tp: Mnemix) -> None:
        with patch("mnemix._runner.run") as mock_run:
            mock_run.return_value = {"memories": []}
            tp.search("q", scope="my-scope", limit=5)
        args = mock_run.call_args[0][2]
        assert "--scope" in args
        assert "my-scope" in args
        assert "--limit" in args
        assert "5" in args

    def test_empty_results(self, tp: Mnemix) -> None:
        with patch("mnemix._runner.run") as mock_run:
            mock_run.return_value = {"memories": []}
            results = tp.search("nothing")
        assert results == []


# ---------------------------------------------------------------------------
# recall
# ---------------------------------------------------------------------------


class TestRecall:
    def test_default_request(self, tp: Mnemix) -> None:
        with patch("mnemix._runner.run") as mock_run:
            mock_run.return_value = {
                "scope": None,
                "query_text": None,
                "disclosure_depth": "summary_then_pinned",
                "count": 0,
                "pinned_context": [],
                "summaries": [],
                "archival": [],
            }
            result = tp.recall()
        assert result.count == 0
        args = mock_run.call_args[0][2]
        assert "--disclosure-depth" in args
        assert "summary_then_pinned" in args

    def test_custom_request(self, tp: Mnemix) -> None:
        req = RecallRequest(text="search text", scope="s", disclosure_depth="full", limit=5)
        with patch("mnemix._runner.run") as mock_run:
            mock_run.return_value = {
                "scope": "s",
                "query_text": "search text",
                "disclosure_depth": "full",
                "count": 0,
                "pinned_context": [],
                "summaries": [],
                "archival": [],
            }
            tp.recall(req)
        args = mock_run.call_args[0][2]
        assert "--text" in args
        assert "search text" in args
        assert "--scope" in args
        assert "--disclosure-depth" in args
        assert "full" in args


# ---------------------------------------------------------------------------
# pins / history
# ---------------------------------------------------------------------------


class TestPins:
    def test_returns_list(self, tp: Mnemix) -> None:
        with patch("mnemix._runner.run") as mock_run:
            mock_run.return_value = {"command": "pins", "count": 1, "memories": [_MEMORY_SUMMARY]}
            result = tp.pins()
        assert len(result) == 1

    def test_passes_scope(self, tp: Mnemix) -> None:
        with patch("mnemix._runner.run") as mock_run:
            mock_run.return_value = {"memories": []}
            tp.pins(scope="my-scope")
        args = mock_run.call_args[0][2]
        assert "--scope" in args


class TestHistory:
    def test_returns_list(self, tp: Mnemix) -> None:
        with patch("mnemix._runner.run") as mock_run:
            mock_run.return_value = {"memories": [_MEMORY_SUMMARY]}
            result = tp.history()
        assert len(result) == 1


# ---------------------------------------------------------------------------
# checkpoint
# ---------------------------------------------------------------------------


class TestCheckpoint:
    def test_creates_checkpoint(self, tp: Mnemix) -> None:
        with patch("mnemix._runner.run") as mock_run:
            mock_run.return_value = {"command": "checkpoint", "action": "create", "checkpoint": _CHECKPOINT}
            result = tp.checkpoint(CheckpointRequest(name="cp-1", description="desc"))
        assert result.name == "cp-1"
        args = mock_run.call_args[0][2]
        assert "--name" in args
        assert "--description" in args


# ---------------------------------------------------------------------------
# versions
# ---------------------------------------------------------------------------


class TestVersions:
    def test_returns_list(self, tp: Mnemix) -> None:
        version_data = {
            "version": 1,
            "recorded_at": "2026-01-01T00:00:00Z",
            "checkpoint_name": None,
            "checkpoint_version": None,
            "summary": None,
        }
        with patch("mnemix._runner.run") as mock_run:
            mock_run.return_value = {"command": "versions", "count": 1, "scope": None, "versions": [version_data]}
            result = tp.versions()
        assert len(result) == 1
        assert result[0].version == 1


# ---------------------------------------------------------------------------
# restore
# ---------------------------------------------------------------------------


class TestRestore:
    def test_restore_by_checkpoint(self, tp: Mnemix) -> None:
        data = {
            "command": "restore",
            "target": {"kind": "checkpoint", "name": "cp-1", "version": 2},
            "previous_version": 3,
            "restored_version": 2,
            "current_version": 4,
            "pre_restore_checkpoint": _CHECKPOINT,
        }
        with patch("mnemix._runner.run") as mock_run:
            mock_run.return_value = data
            result = tp.restore(RestoreRequest(checkpoint="cp-1"))
        assert result.restored_version == 2
        args = mock_run.call_args[0][2]
        assert "--checkpoint" in args

    def test_restore_by_version(self, tp: Mnemix) -> None:
        data = {
            "previous_version": 3,
            "restored_version": 1,
            "current_version": 4,
            "pre_restore_checkpoint": None,
        }
        with patch("mnemix._runner.run") as mock_run:
            mock_run.return_value = data
            result = tp.restore(RestoreRequest(version=1))
        args = mock_run.call_args[0][2]
        assert "--version" in args
        assert "1" in args


# ---------------------------------------------------------------------------
# optimize
# ---------------------------------------------------------------------------


class TestOptimize:
    def test_default_no_prune(self, tp: Mnemix) -> None:
        data = {
            "command": "optimize",
            "previous_version": 2,
            "current_version": 3,
            "compacted": True,
            "prune_old_versions": False,
            "pruned_versions": 0,
            "bytes_removed": 0,
            "retention": {"minimum_age_days": 30, "delete_unverified": False, "error_if_tagged_old_versions": True},
            "pre_optimize_checkpoint": None,
        }
        with patch("mnemix._runner.run") as mock_run:
            mock_run.return_value = data
            result = tp.optimize()
        args = mock_run.call_args[0][2]
        assert "--prune" not in args
        assert result.compacted is True

    def test_prune_flag_included(self, tp: Mnemix) -> None:
        data = {
            "previous_version": 2,
            "current_version": 3,
            "compacted": True,
            "prune_old_versions": True,
            "pruned_versions": 2,
            "bytes_removed": 4096,
            "retention": {"minimum_age_days": 10, "delete_unverified": False, "error_if_tagged_old_versions": True},
            "pre_optimize_checkpoint": None,
        }
        with patch("mnemix._runner.run") as mock_run:
            mock_run.return_value = data
            tp.optimize(OptimizeRequest(prune=True, older_than_days=10))
        args = mock_run.call_args[0][2]
        assert "--prune" in args
        assert "--older-than-days" in args
        assert "10" in args


# ---------------------------------------------------------------------------
# stats
# ---------------------------------------------------------------------------


class TestStats:
    def test_returns_stats(self, tp: Mnemix) -> None:
        with patch("mnemix._runner.run") as mock_run:
            mock_run.return_value = {
                "command": "stats",
                "stats": {
                    "scope": None,
                    "total_memories": 10,
                    "pinned_memories": 2,
                    "version_count": 5,
                    "latest_checkpoint": "cp-1",
                },
            }
            result = tp.stats()
        assert result.total_memories == 10
        assert result.latest_checkpoint == "cp-1"


# ---------------------------------------------------------------------------
# Error propagation
# ---------------------------------------------------------------------------


class TestErrorPropagation:
    def test_command_error_propagated(self, tp: Mnemix) -> None:
        with patch("mnemix._runner.run") as mock_run:
            mock_run.side_effect = MnemixCommandError("store not found", "store_not_found")
            with pytest.raises(MnemixCommandError, match="store not found"):
                tp.stats()
