"""Host-specific Mnemix adapters."""

from .chat_assistant_adapter import ChatAssistantAdapter
from .ci_bot_adapter import CiBotAdapter
from .coding_agent_adapter import (
    CodingAgentAdapter,
    CodingOutcome,
    CodingTaskContext,
    OutcomeClassification,
    StoredOutcomeResult,
)
from .review_tool_adapter import ReviewToolAdapter

__all__ = [
    "ChatAssistantAdapter",
    "CiBotAdapter",
    "CodingAgentAdapter",
    "CodingOutcome",
    "CodingTaskContext",
    "OutcomeClassification",
    "ReviewToolAdapter",
    "StoredOutcomeResult",
]
