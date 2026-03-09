"""Tests for temporal_plane._runner — CLI subprocess helpers."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path
from unittest.mock import MagicMock, patch

import pytest

from temporal_plane._runner import run
from temporal_plane.errors import (
    TemporalPlaneBinaryNotFoundError,
    TemporalPlaneCommandError,
    TemporalPlaneDecodeError,
)

_STORE = Path("/tmp/test-store")


def _make_result(stdout: str, returncode: int = 0) -> MagicMock:
    mock = MagicMock(spec=subprocess.CompletedProcess)
    mock.stdout = stdout
    mock.stderr = ""
    mock.returncode = returncode
    return mock


class TestFindBinary:
    def test_env_var_used_when_set(self, monkeypatch: pytest.MonkeyPatch) -> None:
        monkeypatch.setenv("TP_BINARY", "/custom/temporal-plane")
        envelope = {"kind": "stats", "data": {"stats": {}}}
        with patch("subprocess.run", return_value=_make_result(json.dumps(envelope))) as mock_run:
            run(_STORE, "stats", [])
        call_args = mock_run.call_args[0][0]
        assert call_args[0] == "/custom/temporal-plane"

    def test_raises_when_binary_missing(self, monkeypatch: pytest.MonkeyPatch) -> None:
        monkeypatch.delenv("TP_BINARY", raising=False)
        with patch("shutil.which", return_value=None):
            with pytest.raises(TemporalPlaneBinaryNotFoundError):
                run(_STORE, "init", [])


class TestRunDecoding:
    def _envelope(self, kind: str, data: dict) -> str:
        return json.dumps({"kind": kind, "data": data})

    def test_returns_data_from_envelope(self) -> None:
        payload = {"command": "stats", "stats": {"total_memories": 5}}
        raw = self._envelope("stats", payload)
        with patch("shutil.which", return_value="/usr/bin/temporal-plane"), \
             patch("subprocess.run", return_value=_make_result(raw)):
            result = run(_STORE, "stats", [])
        assert result == payload

    def test_flat_envelope_returned_as_is(self) -> None:
        flat = {"kind": "init", "status": "ok", "message": "done", "path": "/tmp"}
        with patch("shutil.which", return_value="/usr/bin/temporal-plane"), \
             patch("subprocess.run", return_value=_make_result(json.dumps(flat))):
            result = run(_STORE, "init", [])
        # flat envelopes have no "data" key, so the whole dict is returned
        assert result["status"] == "ok"

    def test_error_envelope_raises_command_error(self) -> None:
        err = {"kind": "error", "message": "store not found", "code": "store_not_found"}
        with patch("shutil.which", return_value="/usr/bin/temporal-plane"), \
             patch("subprocess.run", return_value=_make_result(json.dumps(err), returncode=1)):
            with pytest.raises(TemporalPlaneCommandError) as exc_info:
                run(_STORE, "init", [])
        assert exc_info.value.code == "store_not_found"
        assert "store not found" in str(exc_info.value)

    def test_non_json_with_nonzero_exit_raises_command_error(self) -> None:
        with patch("shutil.which", return_value="/usr/bin/temporal-plane"), \
             patch("subprocess.run", return_value=_make_result("fatal error", returncode=1)):
            with pytest.raises(TemporalPlaneCommandError):
                run(_STORE, "init", [])

    def test_non_json_with_zero_exit_raises_decode_error(self) -> None:
        with patch("shutil.which", return_value="/usr/bin/temporal-plane"), \
             patch("subprocess.run", return_value=_make_result("not json", returncode=0)):
            with pytest.raises(TemporalPlaneDecodeError):
                run(_STORE, "stats", [])

    def test_file_not_found_raises_binary_error(self) -> None:
        with patch("shutil.which", return_value="/usr/bin/temporal-plane"), \
             patch("subprocess.run", side_effect=FileNotFoundError):
            with pytest.raises(TemporalPlaneBinaryNotFoundError):
                run(_STORE, "init", [])

    def test_correct_args_passed_to_subprocess(self) -> None:
        payload = {"command": "init", "status": "ok", "message": "ready", "path": "/tmp"}
        with patch("shutil.which", return_value="/usr/bin/temporal-plane"), \
             patch("subprocess.run", return_value=_make_result(json.dumps(payload))) as mock_run:
            run(_STORE, "init", [])
        call_args = mock_run.call_args[0][0]
        assert call_args == ["/usr/bin/temporal-plane", "--store", str(_STORE), "--json", "init"]

    def test_subcommand_args_appended(self) -> None:
        payload = {"stats": {}}
        envelope = json.dumps({"kind": "stats", "data": payload})
        with patch("shutil.which", return_value="/usr/bin/temporal-plane"), \
             patch("subprocess.run", return_value=_make_result(envelope)) as mock_run:
            run(_STORE, "stats", ["--scope", "my-scope"])
        call_args = mock_run.call_args[0][0]
        assert "--scope" in call_args
        assert "my-scope" in call_args
