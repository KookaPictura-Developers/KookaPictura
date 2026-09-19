#!/usr/bin/env bash
#
# check-file-size.sh - enforce the LOC hard cap on crate sources.
#
# Code cap 1200, test cap 1400 (target <800 for both, AGENTS.md rule 9). Tests
# are detected by path: Rust under tests/ or named tests.rs, C++ *_test.{cpp,h}.
# Files over the cap must be listed in scripts/file-size-allowlist.txt with
# their current line count. An allowlist entry is a ceiling, not a licence: it
# may only shrink as the file is split.

set -euo pipefail

repo_root=$(cd "$(dirname "$0")/.." && pwd)
cd "$repo_root"

CODE_BUDGET=1200
TEST_BUDGET=1400
allowlist="scripts/file-size-allowlist.txt"

# ponytail: path-based. Inline #[cfg(test)] in a src/ file counts as code, so a
# mostly-test file not named tests.rs gets the code budget.
budget_for() {
  case "$1" in
    */tests/* | */tests.rs | *_test.cpp | *_test.h | *_test.rs) echo "$TEST_BUDGET" ;;
    *) echo "$CODE_BUDGET" ;;
  esac
}

declare -A allowed=()
if [ -f "$allowlist" ]; then
  while read -r path loc _; do
    case "$path" in ''|\#*) continue ;; esac
    [ -n "${loc:-}" ] || continue
    allowed["$path"]="$loc"
  done < "$allowlist"
fi

violations=0
over=0
while IFS= read -r file; do
  budget=$(budget_for "$file")
  lines=$(wc -l < "$file")
  [ "$lines" -gt "$budget" ] || continue
  over=$((over + 1))
  if [ -z "${allowed[$file]:-}" ]; then
    echo "$file: $lines lines (budget $budget)"
    violations=$((violations + 1))
  elif [ "$lines" -gt "${allowed[$file]}" ]; then
    echo "$file: grew to $lines (allowed ${allowed[$file]})"
    violations=$((violations + 1))
  fi
done < <(find crates -type f \( -name '*.rs' -o -name '*.cpp' -o -name '*.h' \))

if [ "$violations" -gt 0 ]; then
  echo "check-file-size: FAILED ($violations violation(s), $over file(s) over budget)"
  exit 1
fi

echo "check-file-size: OK ($over files over budget, all allowlisted)"
