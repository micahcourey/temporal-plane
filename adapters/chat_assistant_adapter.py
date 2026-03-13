"""Mnemix adapter for chat-assistant workflows."""

from __future__ import annotations

from ._adapter_base import BaseAdapter, ContextBundle


class ChatAssistantAdapter(BaseAdapter):
    """Workflow helpers for conversational assistants."""

    def prepare_reply(
        self,
        *,
        scope: str,
        user_message: str,
        limit: int = 6,
    ) -> ContextBundle:
        return self._context_bundle(
            scope=scope,
            query=user_message,
            heading="Relevant user and conversation memory:",
            limit=limit,
            disclosure_depth="summary_then_pinned",
        )

    def store_preference(
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
            kind="preference",
            title=title,
            summary=summary,
            detail=detail,
            importance=85,
            pin_reason="Persistent user preference",
            tags=["chat", "preference"],
            source_tool="chat-assistant",
        )

    def store_fact(
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
            kind="fact",
            title=title,
            summary=summary,
            detail=detail,
            importance=70,
            tags=["chat", "fact"],
            source_tool="chat-assistant",
        )
