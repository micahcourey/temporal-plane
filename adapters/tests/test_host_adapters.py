"""Smoke tests for the host-specific Mnemix adapters."""

from __future__ import annotations

import sys
from pathlib import Path
from unittest.mock import MagicMock, patch

import pytest

sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from adapters import (  # noqa: E402
    ChatAssistantAdapter,
    CiBotAdapter,
    CodingAgentAdapter,
    ReviewToolAdapter,
)
from mnemix.models import (  # noqa: E402
    CheckpointResult,
    MemoryDetail,
    MemorySummary,
    OptimizeResult,
    OptimizeRetentionResult,
    RecallEntry,
    RecallResult,
    RestoreResult,
    StoreStats,
    VersionRecord,
)

_MEMORY_DETAIL = MemoryDetail(
    id="mem-001",
    scope_id="scope:demo",
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
    scope_id="scope:demo",
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
    name="ci-123",
    version=2,
    created_at="2026-01-01T00:00:00Z",
    description="checkpoint",
)

_RESTORE = RestoreResult(
    previous_version=2,
    restored_version=1,
    current_version=3,
    pre_restore_checkpoint=_CHECKPOINT,
)

_OPTIMIZE = OptimizeResult(
    previous_version=3,
    current_version=4,
    compacted=True,
    prune_old_versions=False,
    pruned_versions=0,
    bytes_removed=1024,
    retention=OptimizeRetentionResult(
        minimum_age_days=30,
        delete_unverified=False,
        error_if_tagged_old_versions=True,
    ),
    pre_optimize_checkpoint=_CHECKPOINT,
)

_VERSION = VersionRecord(
    version=3,
    recorded_at="2026-01-01T00:00:00Z",
    checkpoint_name="ci-123",
    checkpoint_version=2,
    summary="summary",
)

_STATS = StoreStats(
    scope=None,
    total_memories=5,
    pinned_memories=1,
    version_count=3,
    latest_checkpoint="ci-123",
)


@pytest.fixture()
def mock_client() -> MagicMock:
    client = MagicMock()
    entry = RecallEntry(layer="pinned_context", reasons=["pinned"], memory=_SUMMARY)
    client.recall.return_value = RecallResult(
        scope="scope:demo",
        query_text=None,
        disclosure_depth="summary_then_pinned",
        count=1,
        pinned_context=[entry],
        summaries=[],
        archival=[],
    )
    client.remember.return_value = _MEMORY_DETAIL
    client.checkpoint.return_value = _CHECKPOINT
    client.stats.return_value = _STATS
    client.pins.return_value = [_SUMMARY]
    client.history.return_value = [_SUMMARY]
    client.search.return_value = [_SUMMARY]
    client.show.return_value = _MEMORY_DETAIL
    client.versions.return_value = [_VERSION]
    client.restore.return_value = _RESTORE
    client.optimize.return_value = _OPTIMIZE
    return client


def _build(adapter_cls: type, mock_client: MagicMock):
    with patch("adapters._adapter_base.Mnemix", return_value=mock_client):
        adapter = adapter_cls(store=Path("/tmp/test-store"))
    adapter._client = mock_client
    return adapter


def test_coding_agent_start_task_uses_task_title_for_recall(
    mock_client: MagicMock,
) -> None:
    adapter = _build(CodingAgentAdapter, mock_client)

    bundle = adapter.start_task(
        scope="repo:mnemix",
        task_title="Fix memory adapter",
        mode="deep",
    )

    req = mock_client.recall.call_args[0][0]
    assert req.scope == "repo:mnemix"
    assert req.text == "Fix memory adapter"
    assert req.disclosure_depth == "full"
    assert "coding task" in bundle.prompt_preamble
    mock_client.pins.assert_called_once_with(scope="repo:mnemix", limit=10)
    mock_client.history.assert_called_once_with(scope="repo:mnemix", limit=5)


def test_coding_agent_store_decision_uses_decision_kind(
    mock_client: MagicMock,
) -> None:
    adapter = _build(CodingAgentAdapter, mock_client)

    adapter.store_decision(
        memory_id="memory:decision-1",
        scope="repo:mnemix",
        title="Keep adapters host-specific",
        summary="Host workflows need different memory policy.",
        detail="Coding, chat, CI, and review hosts do not use memory the same way.",
        pin_reason="Cross-cutting policy",
    )

    req = mock_client.remember.call_args[0][0]
    assert req.kind == "decision"
    assert req.pin_reason == "Cross-cutting policy"
    assert req.source_tool == "coding-agent"


def test_coding_agent_supports_search_and_show(mock_client: MagicMock) -> None:
    adapter = _build(CodingAgentAdapter, mock_client)

    search_results = adapter.search_memory(text="memory adapter", scope="repo:mnemix")
    detail = adapter.load_memory("memory:decision-1")

    assert search_results == [_SUMMARY]
    assert detail is _MEMORY_DETAIL
    mock_client.search.assert_called_once_with(
        "memory adapter", scope="repo:mnemix", limit=10
    )
    mock_client.show.assert_called_once_with("memory:decision-1")


def test_coding_agent_exposes_checkpoint_restore_and_maintenance(
    mock_client: MagicMock,
) -> None:
    adapter = _build(CodingAgentAdapter, mock_client)

    checkpoint = adapter.checkpoint_before_risky_change(task_id="migration-1")
    versions = adapter.list_versions(limit=5)
    restored = adapter.restore_checkpoint("task-migration-1")
    optimized = adapter.optimize_store(prune=True, older_than_days=14)
    adapter.export_snapshot("/tmp/exported-store")
    adapter.stage_import("/tmp/source-store")

    assert checkpoint is _CHECKPOINT
    assert versions == [_VERSION]
    assert restored is _RESTORE
    assert optimized is _OPTIMIZE
    mock_client.versions.assert_called_once_with(limit=5)
    mock_client.export.assert_called_once_with("/tmp/exported-store")
    mock_client.import_store.assert_called_once_with("/tmp/source-store")


def test_chat_assistant_store_preference_pins_preference(
    mock_client: MagicMock,
) -> None:
    adapter = _build(ChatAssistantAdapter, mock_client)

    adapter.store_preference(
        memory_id="memory:pref-1",
        scope="user:demo",
        title="Concise answers",
        summary="The user prefers short direct answers.",
        detail="Prefer compact replies unless the user explicitly asks for depth.",
    )

    req = mock_client.remember.call_args[0][0]
    assert req.kind == "preference"
    assert req.pin_reason == "Persistent user preference"
    assert req.source_tool == "chat-assistant"


def test_ci_bot_prepare_run_creates_checkpoint_and_context(
    mock_client: MagicMock,
) -> None:
    adapter = _build(CiBotAdapter, mock_client)

    context = adapter.prepare_run(
        scope="repo:mnemix",
        run_id="123",
        pipeline="publish-python",
    )

    cp_req = mock_client.checkpoint.call_args[0][0]
    recall_req = mock_client.recall.call_args[0][0]
    assert cp_req.name == "ci-123"
    assert recall_req.text == "publish-python"
    assert context.checkpoint is _CHECKPOINT


def test_review_tool_records_recurring_issue_as_warning(
    mock_client: MagicMock,
) -> None:
    adapter = _build(ReviewToolAdapter, mock_client)

    adapter.record_recurring_issue(
        memory_id="memory:review-1",
        scope="repo:mnemix",
        title="Tests often skipped for adapter edits",
        summary="Adapter changes need lightweight verification.",
        detail="Changes in host adapters should at minimum run syntax checks or unit tests.",
    )

    req = mock_client.remember.call_args[0][0]
    assert req.kind == "warning"
    assert req.source_tool == "review-tool"
    assert req.tags == ["review", "recurring-issue"]


def test_all_adapters_share_stats_helper(mock_client: MagicMock) -> None:
    adapter = _build(ChatAssistantAdapter, mock_client)

    stats = adapter.get_stats(scope="user:demo")

    assert stats is _STATS
    mock_client.stats.assert_called_once_with(scope="user:demo")
