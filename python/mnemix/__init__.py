"""Mnemix Python binding.

This package provides a thin, typed Python client for the Mnemix
local-first memory layer. All product logic lives in the Rust ``mnemix`` CLI
binary; this package wraps its JSON output surface.

Quick start::

    from pathlib import Path
    from mnemix import Mnemix
    from mnemix.models import RememberRequest

    client = Mnemix(store=Path(".mnemix"))
    client.init()
    client.remember(RememberRequest(
        id="mem-001",
        scope="my-project",
        kind="observation",
        title="Hello Mnemix",
        summary="First memory via the Python binding.",
        detail="Confirming the Python client works end-to-end.",
    ))
"""

from .client import Mnemix
from .errors import (
    MnemixBinaryNotFoundError,
    MnemixCommandError,
    MnemixDecodeError,
    MnemixError,
)
from ._version import __version__
from .models import (
    CheckpointRequest,
    CheckpointResult,
    DisclosureDepth,
    MemoryDetail,
    MemoryKind,
    MemorySummary,
    OptimizeRequest,
    OptimizeResult,
    OptimizeRetentionResult,
    PolicyAction,
    PolicyCheckRequest,
    PolicyDecisionKind,
    PolicyDecisionResult,
    PolicyMode,
    PolicyRecordRequest,
    PolicyRuleEvaluation,
    PolicyTrigger,
    RecallEntry,
    RecallRequest,
    RecallResult,
    RememberRequest,
    RestoreRequest,
    RestoreResult,
    SearchRequest,
    ScopeStrategy,
    StatusResult,
    StoreStats,
    VersionRecord,
)

__all__ = [
    "Mnemix",
    # errors
    "MnemixError",
    "MnemixCommandError",
    "MnemixBinaryNotFoundError",
    "MnemixDecodeError",
    # request models
    "RememberRequest",
    "SearchRequest",
    "RecallRequest",
    "CheckpointRequest",
    "RestoreRequest",
    "OptimizeRequest",
    "PolicyCheckRequest",
    "PolicyRecordRequest",
    # response models
    "MemorySummary",
    "MemoryDetail",
    "CheckpointResult",
    "RecallEntry",
    "RecallResult",
    "VersionRecord",
    "RestoreResult",
    "OptimizeResult",
    "OptimizeRetentionResult",
    "PolicyDecisionResult",
    "PolicyRuleEvaluation",
    "StatusResult",
    "StoreStats",
    # value types
    "MemoryKind",
    "DisclosureDepth",
    "PolicyTrigger",
    "PolicyAction",
    "PolicyMode",
    "PolicyDecisionKind",
    "ScopeStrategy",
    # version
    "__version__",
]
