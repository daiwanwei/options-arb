#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "$0")/.." && pwd)
cd "$repo_root"

mkdir -p artifacts

export RUN_LIVE_SMOKE=1
export ASSERT_LIVE_SMOKE=1

echo "Running live smoke harness..."
cargo test -q -p common --test live_smoke -- --nocapture | tee artifacts/live-smoke-test-output.txt

cat > artifacts/live-smoke-summary.txt <<'TXT'
Live smoke executed via crates/common/tests/live_smoke.rs
- venues: deribit, derive, aevo, premia, stryke
- policy: timeout=8s, retry=3 attempts per venue
- command: RUN_LIVE_SMOKE=1 cargo test -q -p common --test live_smoke -- --nocapture
TXT

echo "Smoke summary written to artifacts/live-smoke-summary.txt"
