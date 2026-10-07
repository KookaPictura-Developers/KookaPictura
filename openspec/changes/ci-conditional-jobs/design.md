# Design

## Context

See proposal.md — Why. Current state: `ci.yml` starts on every code PR and runs
four parallel jobs with no discrimination; docs-only PRs never start the
workflow (`paths-ignore`, spec'd in "Docs-only changes run guards, not builds").
`build.rs` documents that hand-written `.cpp` files are compiled by CMake only,
so cargo jobs cannot observe a `.cpp`-only change. The repo has no branch
protection today. Research (GitHub docs on required status checks, dorny
users, coverage.py's CI) pinned the one hard rule: a workflow-level skip reports
nothing and can wedge a required check forever; a job-level `if:` skip reports
success.

## Goals / Non-Goals

Goals:
- Skip `lint`/`test`/`oracles` when — and only when — the change is provably
  C++-only; never skip `qt-headless` for code changes.
- Fail open: any doubt (unrecognized path, filter failure, mixed diff) runs
  every job.

Non-Goals:
- The optional fail-fast cascade (`needs:` gating behind `lint`) — independent
  trade-off, explicitly out of scope (proposal.md).
- Changing the docs-only workflow-level `paths-ignore`, `guards.yml`,
  `verify-fast.sh`, or adding branch protection.

## Decisions

**Job-level gating, not workflow-level filters.** The workflow keeps an
unfiltered `pull_request`/`push` trigger; a cheap `changes` job computes what
changed and heavy jobs gate on its output with `if:`. Alternative — more
`paths:` filters — was rejected because a workflow-level skip reports no status
at all, which becomes a merge deadlock the day a check is made required
(GitHub: "Troubleshooting required status checks"). Job-level skips report
success and cost nothing extra today.

**`dorny/paths-filter` (SHA-pinned) for diff acquisition, shell for bucketing.**
The action resolves the correct base for both `pull_request` (API diff) and
`push` (before-sha) without a checkout; hand-rolling that base logic is where
false skips come from. Bucketing (`cpp_only` = non-empty diff and every path
under `crates/pictura-app/cpp/`) runs as one shell step over the action's file
list, because the action's pattern matcher ORs its patterns and cannot express
"only this directory" directly. New third-party dependency justification (repo
rule 4): correct PR/push base resolution without reimplementation; SHA-pinned
like the other actions in the workflow.

**Gate on "skip proven", not "run proven".** Conditions read
`needs.changes.outputs.cpp_only != 'true'`, so unrecognized paths and empty
outputs evaluate to "run": run-on-doubt is the default state, not a list of
exceptions. (A *crashed* filter job is different: the implicit `success()` on a
job-level `if` skips the gated jobs — and the failed filter job marks the run
red, so nothing goes green with missing checks.)

**Filter contents.** The file list bucket treats `.github/workflows/**` and any
path outside `crates/pictura-app/cpp/` as disqualifying for `cpp_only`, so a
workflow edit or a root-config edit runs everything. The docs-only case never
reaches this logic (`paths-ignore` already skipped the workflow).

**Single-line `if:` expressions.** Multi-line `${{ }}` expressions silently
mis-evaluate in Actions (documented gotcha; `actionlint`/`zizmor` flag it), so
gating conditions stay on one line without `${{ }}`.

## Risks / Trade-offs

- [False-negative skip: a change that should run a job doesn't] → Gate is
  "cpp_only proven true"; anything else runs. Acceptance scenarios in
  tasks.md exercise each bucket (cpp-only, rust, mixed, workflow edit,
  unrecognized path).
- [New third-party action supply chain] → SHA-pinned commit, minimal
  `permissions` (`contents: read`, `pull-requests: read`), justification in the
  commit message.
- [Filter job fails → wrong gating] → A failed `changes` job skips the gated
  jobs (implicit `success()` on their `if:`) and marks the run red itself; an
  empty output from a *successful* filter evaluates `!= 'true'` and runs
  everything (fail open). Either way the run cannot go green with silently
  missing checks.
- [Push-event base edge (new branch, force push)] → The action's push handling
  covers `before = 000…0`; worst case the file list is empty ⇒ not
  `cpp_only` ⇒ everything runs.

## Migration Plan

One PR touching only `.github/workflows/ci.yml`. Rollback = revert the commit;
no data or artifact migration. If the gating misbehaves in review, the
acceptance scenarios are re-runnable as PRs against the branch.

## Open Questions

None. (Whether branch protection is ever added does not change this design —
job-level gating is correct either way.)
