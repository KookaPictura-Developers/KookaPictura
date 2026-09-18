#!/usr/bin/env bash
#
# test-report.sh - run every test layer and print the unified report.
#
# Runs nextest (which writes target/nextest/default/junit.xml), the doctests,
# and both app self-test invocations, then hands the captured output to
# report_tests.py. Exits non-zero when the reporter reports a failure, so a
# failing layer fails this script.
#
# Usage: scripts/test-report.sh [auto|always|never]   (colour mode, default auto)

set -euo pipefail
cd "$(dirname "$0")/.."

color="${1:-auto}"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

junit="target/nextest/default/junit.xml"

echo "== nextest =="
set +e
cargo nextest run --workspace --no-fail-fast 2>&1 | tee "$tmp/nextest.log"
nextest_rc=${PIPESTATUS[0]}
set -e

echo "== doctests =="
set +e
cargo test --workspace --doc 2>&1 | tee "$tmp/doctests.log"
doctest_rc=${PIPESTATUS[0]}
set -e

selftest_args=()
if [ -x ./build/pictura ]; then
  echo "== app self-test =="
  bare="$tmp/selftest-bare.log"
  psd="$tmp/selftest-psd.log"
  set +e
  ./build/pictura --headless --self-test >"$bare" 2>&1
  ./build/pictura --headless --self-test \
    crates/pictura-codec/tests/fixtures/two_layers.psd >"$psd" 2>&1
  set -e
  selftest_args=(--selftest "$bare" --selftest "$psd")
else
  echo "== app self-test == (skipped: ./build/pictura not built)"
fi

echo "== report =="
set +e
python3 scripts/report_tests.py \
  --junit "$junit" \
  --doctests "$tmp/doctests.log" \
  "${selftest_args[@]}" \
  --color "$color"
report_rc=$?
set -e

if [ "$nextest_rc" -ne 0 ] || [ "$doctest_rc" -ne 0 ]; then
  exit 1
fi
exit "$report_rc"
