#!/usr/bin/env bash
set -euo pipefail

store_path="${MNEMIX_STORE:-.mnemix}"
workflow_key="commit-$(git rev-parse --short HEAD)"
changed_files="$(git diff --cached --name-only)"

if [[ -z "${changed_files}" ]]; then
  exit 0
fi

args=()
while IFS= read -r path; do
  args+=(--path "$path")
done <<< "${changed_files}"

result="$(
  mnemix --store "${store_path}" --json policy check \
    --trigger on_git_commit \
    --workflow-key "${workflow_key}" \
    --host coding-agent \
    "${args[@]}"
)"

decision="$(
  python3 -c 'import json,sys; print(json.load(sys.stdin)["data"]["decision"])' <<< "${result}"
)"

if [[ "${decision}" == "block" || "${decision}" == "require_action" ]]; then
  echo "policy runner blocked commit workflow ${workflow_key}" >&2
  echo "${result}" >&2
  exit 1
fi
