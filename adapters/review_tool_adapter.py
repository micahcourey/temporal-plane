"""Mnemix adapter for review-tool workflows."""

from __future__ import annotations

from ._adapter_base import BaseAdapter, ContextBundle


class ReviewToolAdapter(BaseAdapter):
    """Workflow helpers for code review and policy review tools."""

    def prepare_review(
        self,
        *,
        scope: str,
        review_topic: str,
        limit: int = 8,
    ) -> ContextBundle:
        return self._context_bundle(
            scope=scope,
            query=review_topic,
            heading="Relevant review memory and project conventions:",
            limit=limit,
        )

    def record_review_rule(
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
            kind="procedure",
            title=title,
            summary=summary,
            detail=detail,
            importance=85,
            pin_reason=pin_reason,
            tags=["review", "rule"],
            source_tool="review-tool",
        )

    def record_recurring_issue(
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
            importance=80,
            tags=["review", "recurring-issue"],
            source_tool="review-tool",
        )
