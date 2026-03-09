"""Temporal Plane Python binding — CLI subprocess runner.

This module is internal.  Use :class:`temporal_plane.client.TemporalPlane`
rather than invoking this module directly.

The runner locates the ``temporal-plane`` binary, builds the argument list,
executes the subprocess, and returns a decoded ``dict`` from the CLI JSON
output.  Error envelopes (``{"kind": "error", ...}``) are converted to
:class:`~temporal_plane.errors.TemporalPlaneCommandError` before being
re-raised to callers.
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
from pathlib import Path
from typing import Any

from .errors import (
    TemporalPlaneBinaryNotFoundError,
    TemporalPlaneCommandError,
    TemporalPlaneDecodeError,
)

# The environment variable that overrides the binary path for tests or custom
# installations.
_ENV_BINARY = "TP_BINARY"
_BINARY_NAME = "temporal-plane"


def _find_binary() -> str:
    """Return the path to the ``temporal-plane`` binary.

    Resolution order:
    1. ``TP_BINARY`` environment variable.
    2. ``temporal-plane`` on ``PATH`` via :func:`shutil.which`.

    Raises:
        TemporalPlaneBinaryNotFoundError: if the binary cannot be found.
    """
    from_env = os.environ.get(_ENV_BINARY)
    if from_env:
        return from_env

    found = shutil.which(_BINARY_NAME)
    if found:
        return found

    raise TemporalPlaneBinaryNotFoundError(
        f"Could not find the '{_BINARY_NAME}' binary. "
        f"Install the Temporal Plane CLI or set the {_ENV_BINARY} environment "
        "variable to the absolute path of the binary."
    )


def run(
    store: Path,
    subcommand: str,
    args: list[str],
) -> dict[str, Any]:
    """Run a ``temporal-plane`` subcommand and return the decoded JSON output.

    Args:
        store: Path to the Temporal Plane store directory.
        subcommand: CLI subcommand name (e.g. ``"remember"``).
        args: Additional arguments for the subcommand.

    Returns:
        The ``"data"`` portion of the CLI JSON envelope, or the raw dict for
        ``"status"`` kind outputs.

    Raises:
        TemporalPlaneBinaryNotFoundError: ``temporal-plane`` binary not on PATH.
        TemporalPlaneCommandError: CLI returned a non-zero exit code or an
            error-kind JSON envelope.
        TemporalPlaneDecodeError: Output could not be parsed as JSON or the
            envelope structure was unexpected.
    """
    binary = _find_binary()
    cmd = [binary, "--store", str(store), "--json", subcommand, *args]

    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            check=False,
        )
    except FileNotFoundError as exc:
        raise TemporalPlaneBinaryNotFoundError(
            f"Binary not found: {binary}"
        ) from exc

    raw = result.stdout.strip() or result.stderr.strip()

    # Attempt JSON decoding first regardless of exit code; the CLI always
    # emits a structured JSON error envelope on failure.
    try:
        envelope = json.loads(raw)
    except json.JSONDecodeError as exc:
        if result.returncode != 0:
            raise TemporalPlaneCommandError(
                f"CLI exited with code {result.returncode}: {raw}"
            ) from exc
        raise TemporalPlaneDecodeError(
            f"Could not decode CLI output as JSON: {raw!r}"
        ) from exc

    kind = envelope.get("kind")

    if kind == "error":
        raise TemporalPlaneCommandError(
            message=envelope.get("message", "unknown error"),
            code=envelope.get("code", "unknown"),
        )

    if "data" in envelope:
        return envelope["data"]  # type: ignore[no-any-return]

    # Flat outputs (e.g. init status) have no nested "data" key.
    return envelope  # type: ignore[return-value]
