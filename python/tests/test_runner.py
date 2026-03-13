"""Tests for mnemix._runner — CLI subprocess helpers."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path
from unittest.mock import MagicMock, patch

import pytest

from mnemix._runner import main, main_alias, run
from mnemix.errors import (
    MnemixBinaryNotFoundError,
    MnemixCommandError,
    MnemixDecodeError,
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
        monkeypatch.setenv("MNEMIX_BINARY", "/custom/mnemix")
        envelope = {"kind": "stats", "data": {"stats": {}}}
        with patch("subprocess.run", return_value=_make_result(json.dumps(envelope))) as mock_run:
            run(_STORE, "stats", [])
        call_args = mock_run.call_args[0][0]
        assert call_args[0] == "/custom/mnemix"

    def test_raises_when_binary_missing(self, monkeypatch: pytest.MonkeyPatch) -> None:
        monkeypatch.delenv("MNEMIX_BINARY", raising=False)
        with patch("mnemix._runner._find_bundled_binary", return_value=None), \
             patch("shutil.which", return_value=None):
            with pytest.raises(MnemixBinaryNotFoundError):
                run(_STORE, "init", [])

    def test_bundled_binary_used_when_present(self, monkeypatch: pytest.MonkeyPatch) -> None:
        monkeypatch.delenv("MNEMIX_BINARY", raising=False)
        envelope = {"kind": "stats", "data": {"stats": {}}}
        with patch("mnemix._runner._find_bundled_binary", return_value="/wheel/mnemix"), \
             patch("shutil.which", return_value=None), \
             patch("subprocess.run", return_value=_make_result(json.dumps(envelope))) as mock_run:
            run(_STORE, "stats", [])
        call_args = mock_run.call_args[0][0]
        assert call_args[0] == "/wheel/mnemix"

    def test_env_var_overrides_bundled_binary(self, monkeypatch: pytest.MonkeyPatch) -> None:
        monkeypatch.setenv("MNEMIX_BINARY", "/custom/mnemix")
        envelope = {"kind": "stats", "data": {"stats": {}}}
        with patch("mnemix._runner._find_bundled_binary", return_value="/wheel/mnemix"), \
             patch("subprocess.run", return_value=_make_result(json.dumps(envelope))) as mock_run:
            run(_STORE, "stats", [])
        call_args = mock_run.call_args[0][0]
        assert call_args[0] == "/custom/mnemix"

    def test_platform_binary_name_uses_windows_suffix(self) -> None:
        with patch("sys.platform", "win32"):
            from mnemix._runner import _platform_binary_name

            assert _platform_binary_name() == "mnemix.exe"
            assert _platform_binary_name("mx") == "mx.exe"

    def test_binary_alias_uses_bundled_alias_when_present(self, monkeypatch: pytest.MonkeyPatch) -> None:
        monkeypatch.delenv("MNEMIX_BINARY", raising=False)
        assert_path = "/wheel/mx"
        with patch("mnemix._runner._find_bundled_binary", return_value=assert_path), \
             patch("shutil.which", return_value=None):
            from mnemix._runner import _find_binary

            assert _find_binary("mx") == assert_path


class TestConsoleScripts:
    def test_main_entrypoint_runs_mnemix_binary(self) -> None:
        with patch("mnemix._runner._find_binary", return_value="/wheel/mnemix") as mock_find, \
             patch("subprocess.run", return_value=_make_result("", returncode=0)) as mock_run, \
             patch("sys.argv", ["mnemix", "--help"]):
            exit_code = main()

        assert exit_code == 0
        mock_find.assert_called_once_with("mnemix")
        mock_run.assert_called_once_with(["/wheel/mnemix", "--help"], check=False)

    def test_main_alias_runs_mx_binary(self) -> None:
        with patch("mnemix._runner._find_binary", return_value="/wheel/mx") as mock_find, \
             patch("subprocess.run", return_value=_make_result("", returncode=0)) as mock_run, \
             patch("sys.argv", ["mx", "--help"]):
            exit_code = main_alias()

        assert exit_code == 0
        mock_find.assert_called_once_with("mx")
        mock_run.assert_called_once_with(["/wheel/mx", "--help"], check=False)


class TestRunDecoding:
    def _envelope(self, kind: str, data: dict) -> str:
        return json.dumps({"kind": kind, "data": data})

    def test_returns_data_from_envelope(self) -> None:
        payload = {"command": "stats", "stats": {"total_memories": 5}}
        raw = self._envelope("stats", payload)
        with patch("mnemix._runner._find_bundled_binary", return_value=None), \
             patch("shutil.which", return_value="/usr/bin/mnemix"), \
             patch("subprocess.run", return_value=_make_result(raw)):
            result = run(_STORE, "stats", [])
        assert result == payload

    def test_flat_envelope_returned_as_is(self) -> None:
        flat = {"kind": "init", "status": "ok", "message": "done", "path": "/tmp"}
        with patch("mnemix._runner._find_bundled_binary", return_value=None), \
             patch("shutil.which", return_value="/usr/bin/mnemix"), \
             patch("subprocess.run", return_value=_make_result(json.dumps(flat))):
            result = run(_STORE, "init", [])
        # flat envelopes have no "data" key, so the whole dict is returned
        assert result["status"] == "ok"

    def test_error_envelope_raises_command_error(self) -> None:
        err = {"kind": "error", "message": "store not found", "code": "store_not_found"}
        with patch("mnemix._runner._find_bundled_binary", return_value=None), \
             patch("shutil.which", return_value="/usr/bin/mnemix"), \
             patch("subprocess.run", return_value=_make_result(json.dumps(err), returncode=1)):
            with pytest.raises(MnemixCommandError) as exc_info:
                run(_STORE, "init", [])
        assert exc_info.value.code == "store_not_found"
        assert "store not found" in str(exc_info.value)

    def test_non_json_with_nonzero_exit_raises_command_error(self) -> None:
        with patch("mnemix._runner._find_bundled_binary", return_value=None), \
             patch("shutil.which", return_value="/usr/bin/mnemix"), \
             patch("subprocess.run", return_value=_make_result("fatal error", returncode=1)):
            with pytest.raises(MnemixCommandError):
                run(_STORE, "init", [])

    def test_non_json_with_zero_exit_raises_decode_error(self) -> None:
        with patch("mnemix._runner._find_bundled_binary", return_value=None), \
             patch("shutil.which", return_value="/usr/bin/mnemix"), \
             patch("subprocess.run", return_value=_make_result("not json", returncode=0)):
            with pytest.raises(MnemixDecodeError):
                run(_STORE, "stats", [])

    def test_file_not_found_raises_binary_error(self) -> None:
        with patch("mnemix._runner._find_bundled_binary", return_value=None), \
             patch("shutil.which", return_value="/usr/bin/mnemix"), \
             patch("subprocess.run", side_effect=FileNotFoundError):
            with pytest.raises(MnemixBinaryNotFoundError):
                run(_STORE, "init", [])

    def test_correct_args_passed_to_subprocess(self) -> None:
        payload = {"command": "init", "status": "ok", "message": "ready", "path": "/tmp"}
        with patch("mnemix._runner._find_bundled_binary", return_value=None), \
             patch("shutil.which", return_value="/usr/bin/mnemix"), \
             patch("subprocess.run", return_value=_make_result(json.dumps(payload))) as mock_run:
            run(_STORE, "init", [])
        call_args = mock_run.call_args[0][0]
        assert call_args == ["/usr/bin/mnemix", "--store", str(_STORE), "--json", "init"]

    def test_subcommand_args_appended(self) -> None:
        payload = {"stats": {}}
        envelope = json.dumps({"kind": "stats", "data": payload})
        with patch("mnemix._runner._find_bundled_binary", return_value=None), \
             patch("shutil.which", return_value="/usr/bin/mnemix"), \
             patch("subprocess.run", return_value=_make_result(envelope)) as mock_run:
            run(_STORE, "stats", ["--scope", "my-scope"])
        call_args = mock_run.call_args[0][0]
        assert "--scope" in call_args
        assert "my-scope" in call_args
