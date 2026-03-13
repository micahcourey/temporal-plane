#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF' >&2
usage: ./scripts/publish-release.sh <version> [--dry-run]

Examples:
  ./scripts/publish-release.sh 0.2.2
  ./scripts/publish-release.sh 0.2.2 --dry-run

This script publishes a release after the corresponding release-prep PR has
already merged to main.
EOF
  exit 1
}

if [[ $# -lt 1 || $# -gt 2 ]]; then
  usage
fi

version="$1"
dry_run="false"

if [[ $# -eq 2 ]]; then
  if [[ "$2" != "--dry-run" ]]; then
    usage
  fi
  dry_run="true"
fi

if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "version must match <major>.<minor>.<patch>: $version" >&2
  exit 1
fi

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

require_command() {
  local command_name="$1"
  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "required command not found: $command_name" >&2
    exit 1
  fi
}

should_skip_linux_preflight() {
  local host_arch docker_arch

  host_arch="$(uname -m)"
  docker_arch="$(docker info --format '{{.Architecture}}' 2>/dev/null || true)"

  [[ "$host_arch" == "arm64" && ( "$docker_arch" == "aarch64" || "$docker_arch" == "arm64" ) ]]
}

run() {
  if [[ "$dry_run" == "true" ]]; then
    printf '[dry-run] %q' "$@"
    printf '\n'
    return 0
  fi

  "$@"
}

read_workspace_version() {
  python3 - <<'PY'
from pathlib import Path
import re

text = Path("Cargo.toml").read_text()
match = re.search(r'(?m)^version = "([^"]+)"$', text)
if not match:
    raise SystemExit("workspace version not found in Cargo.toml")
print(match.group(1))
PY
}

read_python_version() {
  python3 - <<'PY'
from pathlib import Path
import re

text = Path("python/mnemix/_version.py").read_text()
match = re.search(r'(?m)^__version__ = "([^"]+)"$', text)
if not match:
    raise SystemExit("python package version not found")
print(match.group(1))
PY
}

require_command git
require_command gh
require_command python3
require_command docker

current_branch="$(git branch --show-current)"
if [[ "$current_branch" != "main" ]]; then
  echo "publish script must run from the main branch; current branch is $current_branch" >&2
  exit 1
fi

if [[ -n "$(git status --porcelain)" ]]; then
  echo "working tree must be clean before running the publish script" >&2
  exit 1
fi

tag="v$version"

if git rev-parse "$tag" >/dev/null 2>&1; then
  echo "tag already exists locally: $tag" >&2
  exit 1
fi

if git ls-remote --tags origin "refs/tags/$tag" | grep -q .; then
  echo "tag already exists on origin: $tag" >&2
  exit 1
fi

if gh release view "$tag" >/dev/null 2>&1; then
  echo "GitHub release already exists: $tag" >&2
  exit 1
fi

run git fetch origin main --tags
run git pull --ff-only origin main

workspace_version="$(read_workspace_version)"
python_version="$(read_python_version)"

if [[ "$workspace_version" != "$version" || "$python_version" != "$version" ]]; then
  echo "version alignment failed: Cargo.toml=$workspace_version python=$python_version expected=$version" >&2
  exit 1
fi

if should_skip_linux_preflight; then
  echo "warning: skipping local Docker Linux preflight on Apple Silicon ARM Docker; rely on merged Linux CI for the release commit" >&2
else
  run ./scripts/check-linux-release-build.sh
fi
run git tag -a "$tag" -m "$tag"
run git push origin "$tag"
run gh release create "$tag" --title "$tag" --generate-notes

cat <<EOF
Release published for $tag.

Checklist follow-up:
- Wait for .github/workflows/publish-python.yml to finish successfully.
- Verify the new version on PyPI and in a clean install.
EOF
