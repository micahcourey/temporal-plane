#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../python"

PYTHON="${PYTHON:-}"
if [[ -z "$PYTHON" ]]; then
	if command -v python3 >/dev/null 2>&1; then
		PYTHON="$(command -v python3)"
	elif command -v python >/dev/null 2>&1; then
		PYTHON="$(command -v python)"
	else
		echo "python3 or python is required" >&2
		exit 1
	fi
fi

VENV_DIR=".release-venv"
INSTALL_VENV_DIR=".release-install-venv"

cleanup() {
	rm -rf "$VENV_DIR" "$INSTALL_VENV_DIR"
}

trap cleanup EXIT

"$PYTHON" -m venv --clear "$VENV_DIR"
source "$VENV_DIR/bin/activate"

python -m pip install --upgrade pip
python -m pip install -e ".[dev,release]"
python -m pytest
rm -rf dist build
python -m build
python -m twine check --strict dist/*

deactivate

"$PYTHON" -m venv --clear "$INSTALL_VENV_DIR"
source "$INSTALL_VENV_DIR/bin/activate"

python -m pip install --upgrade pip
python -m pip install dist/*.whl
python -c 'import temporal_plane; print(temporal_plane.__version__)'
