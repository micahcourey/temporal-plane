#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
python_root="$repo_root/python"
python_bin="${PYTHON_BIN:-$python_root/.venv/bin/python}"
cli_binary="${MNEMIX_CLI_BINARY:-${TP_CLI_BINARY:-$repo_root/target/debug/mnemix}}"
staging_dir="$python_root/mnemix/_bin"
staged_binary="$staging_dir/mnemix"
build_venv="$python_root/.build-wheel-venv"

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

python_bin="$(resolve_python "$python_bin")" || {
	echo "python interpreter not found: $python_bin" >&2
	exit 1
}

venv_python() {
	local venv_dir="$1"
	if [[ -x "$venv_dir/bin/python" ]]; then
		echo "$venv_dir/bin/python"
	else
		echo "$venv_dir/Scripts/python.exe"
	fi
}

if [[ ! -f "$cli_binary" ]]; then
	echo "CLI binary not found: $cli_binary" >&2
	exit 1
fi

if [[ "$cli_binary" == *.exe ]]; then
	staged_binary="$staging_dir/mnemix.exe"
fi

cleanup() {
	rm -f "$python_root/mnemix/_bin/mnemix"
	rm -f "$python_root/mnemix/_bin/mnemix.exe"
	rm -f "$python_root/mnemix/_bin/mnemix"
	rm -f "$python_root/mnemix/_bin/mnemix.exe"
	rm -rf "$build_venv"
	rmdir "$staging_dir" 2>/dev/null || true
}

trap cleanup EXIT

mkdir -p "$staging_dir"
cp "$cli_binary" "$staged_binary"
chmod +x "$staged_binary" 2>/dev/null || true

rm -rf "$build_venv"
"$python_bin" -m venv "$build_venv"
build_python="$(venv_python "$build_venv")"

cd "$python_root"
rm -rf dist build
"$build_python" -m pip install --quiet --upgrade pip build
"$build_python" -m build --wheel

wheel_path="$(ls dist/*.whl)"
case "$wheel_path" in
	*-any.whl)
		echo "expected a platform-specific wheel, got: $wheel_path" >&2
		exit 1
		;;
esac
