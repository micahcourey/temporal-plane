"""Temporal Plane Python binding — explicit exception hierarchy."""

from __future__ import annotations


class TemporalPlaneError(Exception):
    """Base exception for all Temporal Plane errors."""


class TemporalPlaneCommandError(TemporalPlaneError):
    """Raised when the Temporal Plane CLI exits with a non-zero status.

    Attributes:
        code: The error code string returned by the CLI in JSON mode.
        message: The human-readable error message from the CLI.
    """

    def __init__(self, message: str, code: str = "unknown") -> None:
        super().__init__(message)
        self.code = code
        self.message = message

    def __repr__(self) -> str:
        return f"TemporalPlaneCommandError(code={self.code!r}, message={self.message!r})"


class TemporalPlaneBinaryNotFoundError(TemporalPlaneError):
    """Raised when the ``tp`` binary cannot be located.

    Install the Temporal Plane CLI and ensure it is on ``PATH``, or set the
    ``TP_BINARY`` environment variable to the absolute path of the binary.
    """


class TemporalPlaneDecodeError(TemporalPlaneError):
    """Raised when the CLI JSON output cannot be decoded into the expected model."""
