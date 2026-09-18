#!/usr/bin/env bash
#
# verify-fast.sh - fast verification loop for Kooka Pictura.
#
# Format, lint, one test run, the non-goal guard, and spec validation. No Qt
# build and no app self-tests; use verify-full.sh for those.
#
# The test step routes through scripts/test-report.sh (nextest + doctests +
# the app self-tests when ./build/pictura exists), which prints the unified
# report. One nextest run per gate.

set -euo pipefail
cd "$(dirname "$0")/.."

echo "== fmt =="
cargo fmt --all --check
echo "fmt=ok"

echo "== clippy =="
clippy_errors=$(cargo clippy --workspace --all-targets -- -D warnings 2>&1 | grep -cE '^error' || true)
echo "clippy_errors=$clippy_errors"
[ "$clippy_errors" -eq 0 ] || { echo "clippy: FAILED"; exit 1; }

echo "== test =="
bash scripts/test-report.sh never

echo "== file-size =="
bash scripts/check-file-size.sh

echo "== guard =="
bash scripts/guard.sh

echo "== openspec =="
openspec validate --all --strict 2>&1 | tail -1

echo "verify-fast: OK"
