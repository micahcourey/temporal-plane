"""Mnemix Python binding — CLI subprocess runner.

This module is internal.  Use :class:`mnemix.client.Mnemix`
rather than invoking this module directly.

The runner locates the ``mnemix`` binary, builds the argument list,
executes the subprocess, and returns a decoded ``dict`` from the CLI JSON
output.  Error envelopes (``{"kind": "error", ...}``) are converted to
:class:`~mnemix.errors.MnemixCommandError` before being
re-raised to callers.
"""

from __future__ import annotations

from importlib import resources
import json
import os
import shutil
import subprocess
from pathlib import Path
import sys
from typing import Any

from .errors import (
    MnemixBinaryNotFoundError,
    MnemixCommandError,
    MnemixDecodeError,
)

# The environment variable that overrides the binary path for tests or custom
# installations.
_ENV_BINARY = "MNEMIX_BINARY"
_BINARY_NAME = "mnemix"
_BINARY_ALIAS_NAME = "mx"


def _platform_binary_name(binary_name: str = _BINARY_NAME) -> str:
    if sys.platform == "win32":
        return f"{binary_name}.exe"
    return binary_name


def _find_bundled_binary(binary_name: str = _BINARY_NAME) -> str | None:
    """Return the packaged CLI binary path when the wheel bundles one."""
    candidate = resources.files("mnemix").joinpath(
        "_bin", _platform_binary_name(binary_name)
    )
    if candidate.is_file():
        return os.fspath(candidate)
    return None


def _find_binary(binary_name: str = _BINARY_NAME) -> str:
    """Return the path to a CLI binary.

    Resolution order:
    1. ``MNEMIX_BINARY`` environment variable.
    2. Bundled wheel binary, if present.
    3. The requested binary on ``PATH`` via :func:`shutil.which`.

    Raises:
        MnemixBinaryNotFoundError: if the binary cannot be found.
    """
    from_env = os.environ.get(_ENV_BINARY)
    if from_env:
        return from_env

    bundled = _find_bundled_binary(binary_name)
    if bundled:
        return bundled

    found = shutil.which(_platform_binary_name(binary_name))
    if found:
        return found

    raise MnemixBinaryNotFoundError(
        f"Could not find the '{_platform_binary_name(binary_name)}' binary. "
        "Install a Mnemix wheel that bundles the CLI, install the "
        f"Mnemix CLI separately, or set the {_ENV_BINARY} environment "
        "variable to the absolute path of the binary."
    )


def _run_cli_entrypoint(binary_name: str) -> int:
    """Run the requested CLI binary for console-script entry points."""
    binary = _find_binary(binary_name)
    completed = subprocess.run([binary, *sys.argv[1:]], check=False)
    return completed.returncode


def run(
    store: Path,
    subcommand: str,
    args: list[str],
) -> dict[str, Any]:
    """Run a ``mnemix`` subcommand and return the decoded JSON output.

    Args:
        store: Path to the Mnemix store directory.
        subcommand: CLI subcommand name (e.g. ``"remember"``).
        args: Additional arguments for the subcommand.

    Returns:
        The ``"data"`` portion of the CLI JSON envelope, or the raw dict for
        ``"status"`` kind outputs.

    Raises:
        MnemixBinaryNotFoundError: ``mnemix`` binary not on PATH.
        MnemixCommandError: CLI returned a non-zero exit code or an
            error-kind JSON envelope.
        MnemixDecodeError: Output could not be parsed as JSON or the
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
        raise MnemixBinaryNotFoundError(
            f"Binary not found: {binary}"
        ) from exc

    raw = result.stdout.strip() or result.stderr.strip()

    # Attempt JSON decoding first regardless of exit code; the CLI always
    # emits a structured JSON error envelope on failure.
    try:
        envelope = json.loads(raw)
    except json.JSONDecodeError as exc:
        if result.returncode != 0:
            raise MnemixCommandError(
                f"CLI exited with code {result.returncode}: {raw}"
            ) from exc
        raise MnemixDecodeError(
            f"Could not decode CLI output as JSON: {raw!r}"
        ) from exc

    kind = envelope.get("kind")

    if kind == "error":
        raise MnemixCommandError(
            message=envelope.get("message", "unknown error"),
            code=envelope.get("code", "unknown"),
        )

    if "data" in envelope:
        return envelope["data"]  # type: ignore[no-any-return]

    # Flat outputs (e.g. init status) have no nested "data" key.
    return envelope  # type: ignore[return-value]


def main() -> int:
    """Entry point for the packaged ``mnemix`` console script."""
    return _run_cli_entrypoint(_BINARY_NAME)


def main_alias() -> int:
    """Entry point for the packaged ``mx`` console script."""
    return _run_cli_entrypoint(_BINARY_ALIAS_NAME)
