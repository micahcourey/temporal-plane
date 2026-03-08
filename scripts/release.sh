#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <version-tag>" >&2
  exit 1
fi

version_tag="$1"

./scripts/check.sh

git tag "$version_tag"
echo "Created tag $version_tag. Push with: git push origin $version_tag"
