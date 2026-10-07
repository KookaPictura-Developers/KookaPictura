# Proposal

## Why

Every code PR runs all four CI jobs (`lint`, `test`, `qt-headless`, `oracles`) even when the change cannot affect some of them — a `.cpp`-only change still pays for the full cargo workspace suite plus the expensive oracles setup (ImageMagick AppImage, pip, npm), and the existing spec wording ("run on every push and pull request") leaves no room to skip anything except the documented docs-only case. This change lets CI skip jobs that provably cannot be affected, while keeping the run-on-doubt default so no check is ever skipped by mistake.

## What Changes

- Add per-job path gating to `.github/workflows/ci.yml`: a cheap filter job computes the changed paths, and each heavy job is gated with a single-line `if:` (job-level skip reports Success, so a future required check can never deadlock the way a workflow-level skip would).
- Recognized buckets: `.rs` changes run everything (the Rust staticlib links into the Qt app); `.cpp`-only changes run `qt-headless` + `guards` and skip `lint`, `test`, `oracles`; `.github/workflows/**` changes run everything; any unrecognized path runs everything (run-on-doubt).
- The workflow file itself is part of the filter, so editing CI config re-runs everything.
- Keep the existing workflow-level `paths-ignore` for docs-only changes and the unconditional guards workflow — both are already spec'd and unchanged.
- Reconcile the spec: "CI runs format, lint, and tests" currently says jobs run on every push/PR; amend it to permit path-conditional per-job skips, and add a requirement covering the gating semantics (skip reporting, run-on-doubt, filter contents).

Out of scope: the optional fail-fast cascade (gating heavy jobs behind `lint` with `needs:`) — independent trade-off, decide separately.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `verification/verification-harness`: "CI runs format, lint, and tests" is amended so heavy jobs may be skipped per job when the changed paths cannot affect them; a new requirement defines the path-gating behavior (changed-path buckets, run-on-doubt, job-level skip reporting, workflow-file inclusion). The existing "Docs-only changes run guards, not builds" requirement is unchanged.

## Impact

- `.github/workflows/ci.yml` — new filter job, `if:` gates on `lint`/`test`/`qt-headless`/`oracles`, `permissions` for the filter.
- New dependency: `dorny/paths-filter` (pinned to a SHA) — first third-party non-GitHub action in the workflow set; justify in the commit.
- `openspec/specs/verification/verification-harness/spec.md` — requirement amendment after archive.
- No change to `guards.yml`, `verify-fast.sh`, or any Rust/C++ code.
- Risk: false-negative skips. Mitigated by run-on-doubt (unknown paths run everything) and including the workflow file in the filter; acceptance criteria in tasks.md check each bucket.
