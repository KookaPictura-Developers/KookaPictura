#!/usr/bin/env bash
#
# guard.sh - non-goal artifact guard for Kooka Pictura (task M0-C).
#
# Fails when tracked code drifts into a declared non-goal:
#   1. crates/ source referencing Adobe .8bf plugin binaries (wrong ABI, non-goal).
#   2. an artboard* file appearing under crates/ (artboards are post-CS6 CC 2015).
#   3. docs/ modified without a TASK-ALLOWS-DOCS marker.
#
# Runnable locally with no setup:   bash scripts/guard.sh
#
# Overrides:
#   GUARD_BASE=<git-ref>     diff docs/ against this ref (CI/PR use). Without it,
#                            only uncommitted docs/ changes are checked, so the
#                            guard is green on a clean checkout of already-reviewed history.
#   TASK_ALLOWS_DOCS=1       allow docs/ changes without the commit-message marker

set -euo pipefail
cd "$(dirname "$0")/.."

fail=0
echo "guard: repo $(pwd)"

# --- 1. .8bf plugin-binary references in tracked crates/ sources ---------------
echo "check: crates/ references .8bf plugin binaries"
hits=$(git grep -n -i -e '8bf' -- 'crates/*' || true)
if [ -n "$hits" ]; then
    echo "FAIL: .8bf reference under crates/ (Adobe plugin ABI is a non-goal):"
    echo "$hits"
    fail=1
fi

# --- 2. artboard files under crates/ ------------------------------------------
echo "check: no artboard* files under crates/"
artboards=$(find crates \( -name target -o -name .git \) -prune -o -iname 'artboard*' -print 2>/dev/null)
if [ -n "$artboards" ]; then
    echo "FAIL: artboard artifact under crates/ (post-CS6 feature):"
    echo "$artboards"
    fail=1
fi

# --- 3. docs/ modified without a TASK-ALLOWS-DOCS marker ----------------------
echo "check: docs/ changes carry a TASK-ALLOWS-DOCS marker"
base="${GUARD_BASE:-}"

allow=0
if [ -n "${TASK_ALLOWS_DOCS:-}" ]; then
    allow=1
fi
if [ -n "$base" ]; then
    msgs=$(git log "$base"..HEAD --format='%B' 2>/dev/null || true)
    case "$msgs" in
        *TASK-ALLOWS-DOCS*) allow=1 ;;
    esac
fi

committed=""
if [ -n "$base" ]; then
    committed=$(git diff --name-only "$base"..HEAD -- docs/ || true)
fi
uncommitted=$(git status --porcelain -- docs/ || true)

if [ -n "$committed$uncommitted" ] && [ "$allow" -ne 1 ]; then
    echo "FAIL: docs/ modified without a TASK-ALLOWS-DOCS marker:"
    if [ -n "$committed" ]; then echo "$committed"; fi
    if [ -n "$uncommitted" ]; then echo "$uncommitted"; fi
    fail=1
fi
if [ -z "$base" ]; then
    echo "note: GUARD_BASE unset; checked uncommitted docs/ changes only"
fi

if [ "$fail" -ne 0 ]; then
    echo "guard: FAILED"
    exit 1
fi
echo "guard: OK"
