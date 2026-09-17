#!/usr/bin/env bash
#
# verify-fast.sh - fast verification loop for Kooka Pictura.
#
# Format, lint, one test run, the non-goal guard, and spec validation. No Qt
# build and no app self-tests; use verify-full.sh for those.
#
# The single `cargo test` run is tee'd so the pass/fail counts are parsed from
# one execution instead of running the whole suite twice.

set -euo pipefail
cd "$(dirname "$0")/.."

# mold is installed on the dev box; scoped to this script so CI (no mold) is
# unaffected. First run after changing this recompiles once (fingerprint change).
export RUSTFLAGS="${RUSTFLAGS:-} -C link-arg=-fuse-ld=mold"

echo "== fmt =="
cargo fmt --all --check
echo "fmt=ok"

echo "== clippy =="
clippy_errors=$(cargo clippy --workspace --all-targets -- -D warnings 2>&1 | grep -cE '^error' || true)
echo "clippy_errors=$clippy_errors"
[ "$clippy_errors" -eq 0 ] || { echo "clippy: FAILED"; exit 1; }

echo "== test =="
log=$(mktemp)
trap 'rm -f "$log"' EXIT
cargo test --workspace 2>&1 | tee "$log"
total_passed=$(grep -oP 'test result: ok\. \K[0-9]+(?= passed)' "$log" | awk '{s+=$1} END {print s+0}')
failed=$(grep -cE 'FAILED' "$log" || true)
echo "total_passed=$total_passed failed=$failed"
[ "$failed" -eq 0 ] || { echo "test: FAILED"; exit 1; }

echo "== guard =="
bash scripts/guard.sh

echo "== openspec =="
openspec validate --all --strict 2>&1 | tail -1

echo "verify-fast: OK"
