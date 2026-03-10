"""Mnemix Python binding.

This package provides a thin, typed Python client for the Mnemix
local-first memory layer.  All product logic lives in the Rust ``tp`` CLI
binary; this package wraps its JSON output surface.

Quick start::

    from pathlib import Path
    from mnemix import Mnemix
    from mnemix.models import RememberRequest

    tp = Mnemix(store=Path(".mnemix"))
    tp.init()
    tp.remember(RememberRequest(
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
    "StoreStats",
    # value types
    "MemoryKind",
    "DisclosureDepth",
    # version
    "__version__",
]
