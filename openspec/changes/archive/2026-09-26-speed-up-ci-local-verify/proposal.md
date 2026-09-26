# Proposal: speed-up-ci-local-verify

## Why

Every push runs four CI jobs (`.github/workflows/ci.yml`) for roughly 16
job-minutes, and the repository is private, so those minutes are billed. A
superseded push still runs the previous build to completion, a docs-only change
pays for the full Rust and Qt builds, and the `oracles` job re-runs the entire
workspace suite the `rust` job already ran. The local loop has the same misses:
`sccache` is installed but never wired, `verify-fast.sh` pays for the full gate
on a docs-only edit, and `target/` has grown to 41 GB.

## What Changes

- **Cancel superseded runs.** Add `concurrency` with `cancel-in-progress: true`
  keyed on the ref, so a rapid second push stops the first.
- **Run build jobs once per change.** Restrict the `push` trigger to `main` and
  let the pull request `synchronize` event cover branch pushes, so a push to a
  branch with an open PR no longer starts two full runs.
- **Parallelize lint and test.** Split format/clippy into a `lint` job and the
  test suite into a separate `test` job, so the two run at the same time.
- **Bump the Node-20 actions** (`checkout`, `upload-artifact`, `sccache-action`)
  to their Node-24 releases.
- **Skip builds for docs-only changes.** Add `paths-ignore` for `docs/**`,
  `openspec/**`, `**/*.md`, and `.serena/**` to the build jobs, and add a cheap
  `guards` job that runs on every change.
- **A `guards` job.** Run `scripts/guard.sh`, `openspec validate --all --strict`,
  and `scripts/check-file-size.sh`. CI runs none of these today; a docs-only
  change then gets guards instead of a build.
- **Scope the `oracles` job.** Replace `cargo test --workspace` with a run that
  excludes the Qt app, so the job no longer installs Qt or rebuilds the app; the
  tool-gated oracle suites live in Qt-free engine crates.
- **Prebuilt cargo-deny.** Install it with `taiki-e/install-action@cargo-deny`
  instead of `cargo install --locked` (a source build).
- **Bound runaways.** Add `timeout-minutes` to every job.
- **DCO on push.** Run the `dco` job on `push` as well as `pull_request`; commits
  go straight to `main`, so sign-off is unchecked there today.
- **Wire sccache locally, opt-in.** Document `RUSTC_WRAPPER=sccache` and pass
  `-DCMAKE_CXX_COMPILER_LAUNCHER=sccache` from `verify-full.sh` only when
  `sccache` is present, leaving `.cargo/config.toml` and CI unchanged.
- **Docs-only fast path in `verify-fast.sh`.** When the working diff touches only
  `docs/**`, `openspec/**`, `*.md`, or `.serena/**`, run `guard.sh` and
  `openspec validate` and skip fmt, clippy, and tests.
- **Document `target/` hygiene.** Add a `cargo sweep`/`cargo clean` note to
  `DEVELOPING.md`; no new dependency.
- **BREAKING**: none.

## Capabilities

### New Capabilities

<!-- None. -->

### Modified Capabilities

- `verification-harness`: the "CI runs format, lint, and tests" requirement is
  rewritten to cover concurrency, push builds scoped to `main`, docs-only skip,
  the guards job, the scoped oracle job, prebuilt cargo-deny, timeouts, and DCO
  on push. A new requirement covers the docs-only fast path in `verify-fast.sh`
  and the opt-in sccache wiring.

## Impact

- `.github/workflows/ci.yml`: concurrency, `paths-ignore`, push scoped to `main`,
  the scoped `oracles` job, prebuilt cargo-deny, timeouts.
- `.github/workflows/guards.yml` (new): always-on guard, file-size, OpenSpec
  validation, and DCO on push and pull request.
- `scripts/verify-fast.sh`: docs-only fast path.
- `scripts/verify-full.sh`: conditional sccache compiler launcher.
- `DEVELOPING.md`: sccache and `target/` hygiene notes.
- No new runtime dependency, no PSD byte-layout change, no app UI. `cargo-sweep`
  stays optional and uninstalled. Ceiling: the `rust` + `qt-headless` merge is
  left open (it trades billed minutes for wall-clock).
