## Context

`.github/workflows/ci.yml` runs four jobs on every push and pull request:
`dco` (pull requests only), `rust` (fmt, clippy, nextest, doctests, cargo-deny),
`qt-headless` (CMake build, self-test, control e2e), and `oracles` (installs
ImageMagick, `psd-tools`, and ag-psd, then runs `cargo test --workspace`). A run
from issue #89 cost about 16 job-minutes; `rust` was the critical path at 6m54s.
Superseded pushes run to completion, and CI runs none of the repository's
`guard.sh`, `openspec validate`, or `check-file-size.sh` gates.

Constraints that shape the design:

- The repository is **private**, so Actions minutes are billed.
- **No branch protection** is configured (the API returns 403 on the free plan),
  so a skipped job cannot strand a merge. This makes a docs-only skip safe.
- `target/` is 41 GB (debug 39 GB, release 2.3 GB), so caching it is
  impractical; `cache-targets: "false"` stays.
- The tool-gated oracle suites live in seven Qt-free engine crates (`codec`,
  `color`, `adjust`, `ops`, `select`, `render`, `filters`). Only `pictura-app`
  pulls `cxx-qt`.

## Goals / Non-Goals

**Goals:**

- Cut billed minutes on superseded and docs-only pushes.
- Make the `oracles` job stop re-running the whole workspace app build.
- Add the guard, OpenSpec, and file-size gates to CI.
- Make the local loop benefit from the same ideas where they translate.

**Non-Goals:**

- Merging `rust` and `qt-headless`. It saves a duplicate app compile but makes
  one long job and drops parallelism; kept open as a follow-up.
- Caching `target/`. 41 GB is impractical on Actions.
- Any change to PSD/engine behavior, dependencies, or app UI.

## Decisions

### D1. Slim `oracles`, do not merge it into `rust`

Run `cargo test --workspace --exclude pictura_app` in `oracles` and drop the Qt
install and app build from that job. All tool-gated suites are in Qt-free engine
crates, so the "run with ImageMagick and `psd-tools` installed" semantics are
preserved while the duplicate Qt/app work disappears.

- *Alternative:* enumerate the seven packages with `-p` flags. More brittle as
  crates move; `--exclude` is one flag and self-maintains.
- *Alternative:* install the tools in `rust` and delete `oracles`. Saves a whole
  compile/test pass but loses cross-job parallelism and grows the critical path;
  rejected unless billed minutes dominate.

### D2. `paths-ignore` at the workflow level, guards in their own workflow

GitHub only supports `paths-ignore` at the workflow `on:` level, not per job. If
`guards` lived in `ci.yml`, a docs-only change would skip it too. So the build
jobs stay in `ci.yml` behind workflow-level `paths-ignore`, and a new
`.github/workflows/guards.yml` (no path filter) always runs the guard, OpenSpec,
and file-size gates. No third-party path-filter action is needed.

`dco` moves to `guards.yml` so it also runs for docs-only pushes; today it is
skipped entirely on push, and commits go straight to `main`.

- *Alternative:* a `changes` detection job plus `needs.<job>.outputs.code`
  conditions. More moving parts than a second workflow file.

### D3. Scope the `oracles` apt and caches

Keep `lld` and `liblcms2-dev` (pictura-color links lcms2); drop Ninja, Vulkan
headers, and the Qt action from `oracles`. Keep sccache and the registry cache.

### D4. Prebuilt cargo-deny and timeouts

Replace `cargo install cargo-deny --locked` with
`taiki-e/install-action@cargo-deny`, gated on `deny.toml` as today (`deny.toml`
exists, so the step runs). Add a `timeout-minutes` to every job.

### D5. sccache is opt-in, not a config change

`.cargo/config.toml` cannot express "use sccache only if installed", and editing
it would change CI (which sets `RUSTC_WRAPPER` in the workflow `env`) and break
machines without sccache. Instead document `export RUSTC_WRAPPER=sccache` in
`DEVELOPING.md`, and have `verify-full.sh` add
`-DCMAKE_CXX_COMPILER_LAUNCHER=sccache` only when `command -v sccache` succeeds.

### D6. Docs-only fast path keyed on the real diff

`verify-fast.sh` inspects `git diff --name-only` against the merge base with
`main` plus untracked files. When every path matches `docs/**`, `openspec/**`,
`*.md`, or `.serena/**`, it runs `guard.sh` and `openspec validate` only. An
empty diff (clean tree) falls through to the full gate, so the fast path cannot
pass vacuously. Keying on paths, not a fixed prefix, covers root-level `*.md`
such as `AGENTS.md`.

### D7. `target/` hygiene is documentation only

Document `cargo sweep` (and plain `cargo clean`) in `DEVELOPING.md`. No new
dependency; `cargo-sweep` stays optional.

### D8. Build jobs run on pull requests, and on `main` pushes only

`on.push` is restricted to `main` in both workflows. Without it, a push to a
branch with an open pull request starts two full runs: one on
`refs/heads/<branch>` and one on `refs/pull/N/merge`. The concurrency group is
`workflow-ref`, so the refs differ and neither cancels the other. Restricting the
push trigger to `main` leaves the pull request `synchronize` event to cover
branch pushes (once, with rapid pushes cancelling each other), keeps post-merge
coverage on `main`, and keeps DCO running for direct commits.

- *Alternative:* a shared concurrency group via `github.head_ref || github.ref_name`
  would let the two runs cancel, but the survivor is nondeterministic and the
  loser shows as a cancelled check. Rejected as subtler for the same saving.
- *Trade-off:* a branch pushed with no open pull request gets no CI until one is
  opened; a draft pull request restores coverage.

### D9. Split `lint` from `test`; keep the app tests in the test job

sccache cannot cache `bin`/test/proc-macro crates, so the residual cost is
type-checking and linking the workspace's ~96 integration test binaries plus the
app. Clippy (108–120s) and the test run (170s) were serialized in one job and
share nothing (clippy uses `clippy-driver`), so they move to two jobs that run in
parallel: `lint` (fmt, clippy, cargo-deny) and `test` (nextest, doctests, JUnit).

Moving the app tests out of `test` and into the Qt headless job was tried and
reverted: it only saves the ~46s Qt install in `test`, while the ~131s app-test
compile lands behind the ~131s CMake build in `qt-headless`, making that job the
critical path at ~355s instead of the ~256s `test` job. The app tests therefore
stay in `test`, which installs Qt because the suite includes the app.

Per the `sccache` and `rust-cache` guidance, no caching setting changes: the hit
rate is already ~99% across jobs, `cache-targets` stays `false` (the 41 GB target
exceeds the 10 GB Actions cache), and the residual is unmatched by any cache key.

The Node-20 actions (`actions/checkout@v4`, `actions/upload-artifact@v4`,
`mozilla-actions/sccache-action@v0.0.9`) are bumped to their Node-24 releases
(`@v7`, `@v7`, `@v0.0.11`).

- *Alternative:* leave clippy in the test job. Rejected: it keeps ~108s on the
  critical path for no benefit.
- *Alternative:* cache `target/`. Rejected: 41 GB vs the 10 GB limit; rust-cache
  caches dependency artifacts only, and the work that remains is uncacheable
  test binaries.

## Risks / Trade-offs

- [Skipped build jobs leave a docs-only PR with only `guards` checks] → No branch
  protection exists, so nothing is stranded; re-evaluate if protection is added.
- [`--exclude pictura_app` silently loses a tool-gated app test in future] →
  There are none today; the guards/CI change must be revisited if a Qt-side
  oracle appears.
- [Opt-in sccache is easy to forget] → Document it prominently; the failure mode
  is only slower builds, not a broken gate.
- [concurrency cancels a run that was about to report a failure] → Standard
  trade-off; the follow-up commit re-runs the full gate.
- [DCO on push needs a commit range] → Use `github.event.before..github.event.after`
  for push and the PR base/head for pull requests.

## Migration Plan

Single PR against `ci/89-speed-up-ci-and-local-verify`. CI config changes take
effect on merge; no runtime migration and no rollback beyond reverting the
workflow. Verify by observing one code push to `main` (all jobs), one push to a
pull request branch (all jobs once, not twice), one docs-only push (guards only),
and two rapid pushes (first cancelled).

## Open Questions

- Whether to adopt the `rust` + `qt-headless` merge later, judged on measured
  job-minutes after this change lands.
