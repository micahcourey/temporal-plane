#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"

require_command() {
  local command_name="$1"
  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "required command not found: $command_name" >&2
    exit 1
  fi
}

require_command docker

docker run --rm \
  -v "$repo_root:/work:ro" \
  ubuntu:22.04 \
  bash -lc '
    set -euo pipefail
    export DEBIAN_FRONTEND=noninteractive
    apt-get update
    apt-get install -y --no-install-recommends \
      build-essential \
      ca-certificates \
      clang \
      cmake \
      curl \
      libssl-dev \
      pkg-config \
      protobuf-compiler
    curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain 1.94.0 --profile minimal
    . "$HOME/.cargo/env"
    cp -a /work /tmp/mnemix-linux-release-src
    export CARGO_HOME=/tmp/cargo-home
    export CARGO_TARGET_DIR=/tmp/mnemix-linux-release-target
    cargo build --manifest-path /tmp/mnemix-linux-release-src/Cargo.toml --release -p mnemix-cli
  '
