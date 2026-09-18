#!/usr/bin/env bash
#
# check-file-size.sh - enforce the 1000-LOC hard cap on crate sources.
#
# Files over the cap must be listed in scripts/file-size-allowlist.txt with
# their current line count. An allowlist entry is a ceiling, not a licence: it
# may only shrink as the file is split.

set -euo pipefail

repo_root=$(cd "$(dirname "$0")/.." && pwd)
cd "$repo_root"

BUDGET=1000
allowlist="scripts/file-size-allowlist.txt"

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
  lines=$(wc -l < "$file")
  [ "$lines" -gt "$BUDGET" ] || continue
  over=$((over + 1))
  if [ -z "${allowed[$file]:-}" ]; then
    echo "$file: $lines lines (budget $BUDGET)"
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
