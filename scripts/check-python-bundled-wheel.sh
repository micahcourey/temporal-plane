#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
python_root="$repo_root/python"
python_bin="${PYTHON_BIN:-$python_root/.venv/bin/python}"
verify_venv="$python_root/.bundled-wheel-venv"

resolve_python() {
    local candidate="$1"
    if [[ -x "$candidate" ]]; then
        echo "$candidate"
        return 0
    fi
    if command -v "$candidate" >/dev/null 2>&1; then
        command -v "$candidate"
        return 0
    fi
    return 1
}

venv_python() {
    local venv_dir="$1"
    if [[ -x "$venv_dir/bin/python" ]]; then
        echo "$venv_dir/bin/python"
    else
        echo "$venv_dir/Scripts/python.exe"
    fi
}

python_bin="$(resolve_python "$python_bin")" || {
	echo "python interpreter not found: $python_bin" >&2
	exit 1
}

cleanup() {
	rm -rf "$verify_venv"
}

trap cleanup EXIT

"$repo_root/scripts/build-python-wheel-with-cli.sh"

rm -rf "$verify_venv"
"$python_bin" -m venv "$verify_venv"
verify_python="$(venv_python "$verify_venv")"
"$verify_python" -m pip install --upgrade pip
"$verify_python" -m pip install "$python_root"/dist/*.whl

cd "$repo_root"
"$verify_python" <<'PY'
import os
import tempfile
from pathlib import Path

from mnemix import Mnemix
from mnemix._runner import _find_binary

os.environ.pop("MNEMIX_BINARY", None)
os.environ.pop("MNEMIX_BINARY", None)

binary = _find_binary()
print(binary)
assert "mnemix" in binary
assert Path(binary).exists()

with tempfile.TemporaryDirectory() as tmpdir:
    store = Path(tmpdir) / ".mnemix"
    client = Mnemix(store=store)
    client.init()
    assert store.exists()
PY
