#!/usr/bin/env bash
#
# check-selftest-budget.sh - pin the total ST_BEGIN count across the C++ shell.
#
# New GUI checks belong in the Qt Test suites (AGENTS.md rule 11); runSelfTest()
# only shrinks. This fails when a new ST_BEGIN site appears, so the budget can
# only be lowered (checks retired) and never raised (checks added). The budget
# is recorded in scripts/selftest-budget.txt.

set -euo pipefail

repo_root=$(cd "$(dirname "$0")/.." && pwd)
cd "$repo_root"

budget_file="scripts/selftest-budget.txt"
count=$(grep -rho 'ST_BEGIN' crates/pictura-app/cpp --include='*.cpp' | wc -l)
budget=$(grep -E '^[0-9]+$' "$budget_file" 2>/dev/null | head -n1 || true)

if [ -z "$budget" ]; then
  echo "check-selftest-budget: FAILED (no budget number in scripts/selftest-budget.txt)"
  exit 1
fi

if [ "$count" -gt "$budget" ]; then
  echo "check-selftest-budget: FAILED ($count ST_BEGIN > budget $budget)"
  exit 1
fi

echo "check-selftest-budget: OK ($count/$budget)"
