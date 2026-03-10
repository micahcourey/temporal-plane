"""Smoke tests for the AI DX Toolkit — Mnemix adapter.

These tests mock the ``mnemix.Mnemix`` client so the adapter
behavior can be verified without a running store or binary.
"""

from __future__ import annotations

import sys
from pathlib import Path
from unittest.mock import MagicMock, call, patch

import pytest

# Add the adapter directory to the path so we can import the module directly
# when running tests from outside the package.
sys.path.insert(0, str(Path(__file__).parent.parent))

from mnemix_adapter import MnemixAdapter
from mnemix.models import (
    CheckpointResult,
    MemoryDetail,
    RecallEntry,
    RecallResult,
    MemorySummary,
    StoreStats,
)


# ---------------------------------------------------------------------------
# Shared fixtures
# ---------------------------------------------------------------------------

_MEMORY_DETAIL = MemoryDetail(
    id="mem-001",
    scope_id="agent-x",
    kind="observation",
    title="Title",
    summary="Summary",
    detail="Detail",
    pinned=False,
    pin_reason=None,
    importance=50,
    confidence=100,
    created_at="2026-01-01T00:00:00Z",
    updated_at="2026-01-01T00:00:00Z",
    source_session_id=None,
    source_tool=None,
    source_ref=None,
    tags=[],
    entities=[],
    metadata={},
)

_SUMMARY = MemorySummary(
    id="mem-001",
    scope_id="agent-x",
    kind="observation",
    title="Title",
    summary="Summary",
    pinned=False,
    pin_reason=None,
    importance=50,
    confidence=100,
    created_at="2026-01-01T00:00:00Z",
    updated_at="2026-01-01T00:00:00Z",
    tags=[],
    entities=[],
)

_CHECKPOINT = CheckpointResult(
    name="sess-001",
    version=2,
    created_at="2026-01-01T00:00:00Z",
    description="session start",
)

_STATS = StoreStats(
    scope=None,
    total_memories=5,
    pinned_memories=1,
    version_count=3,
    latest_checkpoint="sess-001",
)


@pytest.fixture()
def mock_client() -> MagicMock:
    return MagicMock()


@pytest.fixture()
def adapter(mock_client: MagicMock) -> MnemixAdapter:
    with patch("mnemix_adapter.Mnemix", return_value=mock_client):
        a = MnemixAdapter(store=Path("/tmp/test-store"))
    a._client = mock_client
    return a


# ---------------------------------------------------------------------------
# ensure_store
# ---------------------------------------------------------------------------


class TestEnsureStore:
    def test_calls_init(self, adapter: MnemixAdapter, mock_client: MagicMock) -> None:
        adapter.ensure_store()
        mock_client.init.assert_called_once()


# ---------------------------------------------------------------------------
# record_observation
# ---------------------------------------------------------------------------


class TestRecordObservation:
    def test_delegates_to_remember(
        self, adapter: MnemixAdapter, mock_client: MagicMock
    ) -> None:
        mock_client.remember.return_value = _MEMORY_DETAIL
        result = adapter.record_observation(
            id="mem-001",
            scope="agent-x",
            title="Title",
            summary="Summary",
            detail="Detail",
        )
        assert result is _MEMORY_DETAIL
        mock_client.remember.assert_called_once()
        req = mock_client.remember.call_args[0][0]
        assert req.kind == "observation"
        assert req.id == "mem-001"
        assert req.scope == "agent-x"

    def test_passes_tags_and_source_tool(
        self, adapter: MnemixAdapter, mock_client: MagicMock
    ) -> None:
        mock_client.remember.return_value = _MEMORY_DETAIL
        adapter.record_observation(
            id="mem-002",
            scope="agent-x",
            title="T",
            summary="S",
            detail="D",
            tags=["tag1"],
            source_tool="my-tool",
        )
        req = mock_client.remember.call_args[0][0]
        assert req.tags == ["tag1"]
        assert req.source_tool == "my-tool"


# ---------------------------------------------------------------------------
# record_decision
# ---------------------------------------------------------------------------


class TestRecordDecision:
    def test_uses_decision_kind(
        self, adapter: MnemixAdapter, mock_client: MagicMock
    ) -> None:
        mock_client.remember.return_value = _MEMORY_DETAIL
        adapter.record_decision(
            id="dec-001",
            scope="agent-x",
            title="Use Rust",
            summary="Chose Rust for performance.",
            detail="Full rationale.",
        )
        req = mock_client.remember.call_args[0][0]
        assert req.kind == "decision"

    def test_pin_reason_forwarded(
        self, adapter: MnemixAdapter, mock_client: MagicMock
    ) -> None:
        mock_client.remember.return_value = _MEMORY_DETAIL
        adapter.record_decision(
            id="dec-001",
            scope="agent-x",
            title="Critical decision",
            summary="Summary",
            detail="Detail",
            pin_reason="Core architectural choice",
        )
        req = mock_client.remember.call_args[0][0]
        assert req.pin_reason == "Core architectural choice"


# ---------------------------------------------------------------------------
# fetch_context
# ---------------------------------------------------------------------------


class TestFetchContext:
    def test_returns_flat_list(
        self, adapter: MnemixAdapter, mock_client: MagicMock
    ) -> None:
        entry = RecallEntry(layer="pinned_context", reasons=["pinned"], memory=_SUMMARY)
        mock_client.recall.return_value = RecallResult(
            scope="agent-x",
            query_text=None,
            disclosure_depth="summary_then_pinned",
            count=1,
            pinned_context=[entry],
            summaries=[],
            archival=[],
        )
        result = adapter.fetch_context(scope="agent-x")
        assert len(result) == 1
        assert result[0].layer == "pinned_context"

    def test_uses_summary_then_pinned_depth(
        self, adapter: MnemixAdapter, mock_client: MagicMock
    ) -> None:
        mock_client.recall.return_value = RecallResult(
            scope="agent-x",
            query_text=None,
            disclosure_depth="summary_then_pinned",
            count=0,
            pinned_context=[],
            summaries=[],
            archival=[],
        )
        adapter.fetch_context(scope="agent-x", query="test")
        req = mock_client.recall.call_args[0][0]
        assert req.disclosure_depth == "summary_then_pinned"
        assert req.text == "test"

    def test_all_layers_combined(
        self, adapter: MnemixAdapter, mock_client: MagicMock
    ) -> None:
        pinned = RecallEntry(layer="pinned_context", reasons=["pinned"], memory=_SUMMARY)
        summary_entry = RecallEntry(layer="summary", reasons=["summary_kind"], memory=_SUMMARY)
        archival = RecallEntry(layer="archival", reasons=["archival_expansion"], memory=_SUMMARY)
        mock_client.recall.return_value = RecallResult(
            scope="s",
            query_text=None,
            disclosure_depth="summary_then_pinned",
            count=3,
            pinned_context=[pinned],
            summaries=[summary_entry],
            archival=[archival],
        )
        result = adapter.fetch_context(scope="s")
        assert len(result) == 3
        assert result[0].layer == "pinned_context"
        assert result[1].layer == "summary"
        assert result[2].layer == "archival"


# ---------------------------------------------------------------------------
# create_session_checkpoint
# ---------------------------------------------------------------------------


class TestCreateSessionCheckpoint:
    def test_checkpoint_created(
        self, adapter: MnemixAdapter, mock_client: MagicMock
    ) -> None:
        mock_client.checkpoint.return_value = _CHECKPOINT
        result = adapter.create_session_checkpoint("sess-001", description="session start")
        assert result.name == "sess-001"
        req = mock_client.checkpoint.call_args[0][0]
        assert req.name == "sess-001"
        assert req.description == "session start"


# ---------------------------------------------------------------------------
# get_stats
# ---------------------------------------------------------------------------


class TestGetStats:
    def test_returns_stats(
        self, adapter: MnemixAdapter, mock_client: MagicMock
    ) -> None:
        mock_client.stats.return_value = _STATS
        result = adapter.get_stats()
        assert result.total_memories == 5

    def test_passes_scope(
        self, adapter: MnemixAdapter, mock_client: MagicMock
    ) -> None:
        mock_client.stats.return_value = _STATS
        adapter.get_stats(scope="my-agent")
        mock_client.stats.assert_called_once_with(scope="my-agent")
