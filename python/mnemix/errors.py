"""Mnemix Python binding — explicit exception hierarchy."""

from __future__ import annotations


class MnemixError(Exception):
    """Base exception for all Mnemix errors."""


class MnemixCommandError(MnemixError):
    """Raised when the Mnemix CLI exits with a non-zero status.

    Attributes:
        code: The error code string returned by the CLI in JSON mode.
        message: The human-readable error message from the CLI.
    """

    def __init__(self, message: str, code: str = "unknown") -> None:
        super().__init__(message)
        self.code = code
        self.message = message

    def __repr__(self) -> str:
        return f"MnemixCommandError(code={self.code!r}, message={self.message!r})"


class MnemixBinaryNotFoundError(MnemixError):
    """Raised when the ``tp`` binary cannot be located.

    Install the Mnemix CLI and ensure it is on ``PATH``, or set the
    ``MNEMIX_BINARY`` environment variable to the absolute path of the binary.
    """


class MnemixDecodeError(MnemixError):
    """Raised when the CLI JSON output cannot be decoded into the expected model."""
