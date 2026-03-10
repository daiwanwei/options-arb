#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "$0")/.." && pwd)
cd "$repo_root"

echo "[ci_local] running cargo fmt --all --check"
cargo fmt --all --check

echo "[ci_local] running cargo clippy --workspace --all-targets -- -D warnings"
cargo clippy --workspace --all-targets -- -D warnings

echo "[ci_local] running cargo test -q"
cargo test -q

if [[ "${1:-}" == "--with-act" ]]; then
  if ! command -v act >/dev/null 2>&1; then
    echo "[ci_local] error: 'act' not found in PATH"
    echo "Install: https://nektosact.com/installation/"
    exit 1
  fi

  echo "[ci_local] running act for .github/workflows/ci.yml"
  act pull_request -W .github/workflows/ci.yml
fi

echo "[ci_local] done"
