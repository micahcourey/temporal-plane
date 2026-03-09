"""Temporal Plane Python binding.

This package provides a thin, typed Python client for the Temporal Plane
local-first memory layer.  All product logic lives in the Rust ``tp`` CLI
binary; this package wraps its JSON output surface.

Quick start::

    from pathlib import Path
    from temporal_plane import TemporalPlane
    from temporal_plane.models import RememberRequest

    tp = TemporalPlane(store=Path(".temporal-plane"))
    tp.init()
    tp.remember(RememberRequest(
        id="mem-001",
        scope="my-project",
        kind="observation",
        title="Hello Temporal Plane",
        summary="First memory via the Python binding.",
        detail="Confirming the Python client works end-to-end.",
    ))
"""

from .client import TemporalPlane
from .errors import (
    TemporalPlaneBinaryNotFoundError,
    TemporalPlaneCommandError,
    TemporalPlaneDecodeError,
    TemporalPlaneError,
)
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

__version__ = "0.1.0"

__all__ = [
    "TemporalPlane",
    # errors
    "TemporalPlaneError",
    "TemporalPlaneCommandError",
    "TemporalPlaneBinaryNotFoundError",
    "TemporalPlaneDecodeError",
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
