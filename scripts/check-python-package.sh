#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
python_root="$repo_root/python"
python_bin="${PYTHON:-python3}"
release_venv="$python_root/.release-venv"

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
	echo "python3 or python is required" >&2
	exit 1
}

cleanup() {
	rm -rf "$release_venv"
}

trap cleanup EXIT

rm -rf "$release_venv"
"$python_bin" -m venv "$release_venv"
release_python="$(venv_python "$release_venv")"

cd "$python_root"
"$release_python" -m pip install --upgrade pip
"$release_python" -m pip install -e ".[dev,release]"
"$release_python" -m pytest

rm -rf dist build
"$release_python" -m build --sdist
"$release_python" -m twine check --strict dist/*

cd "$repo_root"
PYTHON_BIN="$release_python" ./scripts/check-python-bundled-wheel.sh
