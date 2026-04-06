#!/usr/bin/env bash
set -euo pipefail

store_path="${MNEMIX_STORE:-.mnemix}"
workflow_key="${1:-pr-demo}"
trigger="${2:-on_pr_open}"

result="$(
  mnemix --store "${store_path}" --json policy explain \
    --trigger "${trigger}" \
    --workflow-key "${workflow_key}" \
    --host ci-bot
)"

decision="$(
  python3 -c 'import json,sys; print(json.load(sys.stdin)["data"]["decision"])' <<< "${result}"
)"

echo "${result}"

if [[ "${decision}" == "block" || "${decision}" == "require_action" ]]; then
  exit 1
fi
