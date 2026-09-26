# verification-harness Specification

## Purpose
TBD - created by archiving change m0-walking-skeleton. Update Purpose after archive.

## Requirements

### Requirement: Golden-image comparison
The system SHALL provide `compare(a, b, tolerance) -> Result<Diff, _>` that
compares two equal-length 8-bit buffers using an absolute per-sample tolerance
and reports the total sample count, the number of samples above tolerance, the
maximum delta, the summed delta, and the mean delta; it SHALL return an error
when the buffer lengths differ.

#### Scenario: Samples within tolerance
- **WHEN** every sample differs by no more than the tolerance
- **THEN** `differing` is zero and `Diff::is_empty()` is true

#### Scenario: Samples above tolerance
- **WHEN** a sample differs by more than the tolerance
- **THEN** it is counted in `differing` and `max_delta` records the largest delta

#### Scenario: Buffer length mismatch
- **WHEN** the two buffers have different lengths
- **THEN** `compare` returns an error and does not compare any samples

### Requirement: Stable content hash
The system SHALL provide `hash_bytes` that computes a deterministic content hash,
so identical bytes always hash equally and a changed byte changes the hash, for
use as a determinism gate.

#### Scenario: Identical bytes hash equally
- **WHEN** the same bytes are hashed twice, including in separate runs
- **THEN** both hashes are equal

#### Scenario: Changed bytes change the hash
- **WHEN** one input byte changes
- **THEN** the resulting hash differs

### Requirement: pictura-diff CLI
The system SHALL provide a `pictura-diff` binary that reads two raw 8-bit files,
accepts a `--tolerance` (or `-t`) value, prints the sample, differing, max-delta,
and mean-delta metrics, prints usage for `--help`, and exits non-zero on any
difference above tolerance, IO error, unknown option, or length mismatch.

#### Scenario: Identical files
- **WHEN** both input files are byte-identical
- **THEN** the metrics are printed and the process exits 0

#### Scenario: Files differ beyond tolerance
- **WHEN** any sample differs by more than the given tolerance
- **THEN** the process exits non-zero

#### Scenario: Wrong number of files
- **WHEN** fewer or more than two files are given
- **THEN** usage is printed and the process exits non-zero

### Requirement: CI runs format, lint, and tests
The system SHALL run, on every push to `main` and on every pull request,
`cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D
warnings`, and the workspace test suite on the pinned toolchain. Format and
clippy SHALL run in a `lint` job and the test suite SHALL run in a separate
`test` job, so the two run in parallel. The `test` job SHALL run the whole
workspace under `cargo nextest run --workspace` and SHALL run doctests
explicitly with `cargo test --workspace --doc`; because that suite includes the
Qt application, the job SHALL install Qt. `scripts/verify-fast.sh` SHALL run the
same nextest invocation plus the explicit doctest run, so the local fast gate and
CI agree on the runner. The nextest run SHALL emit a JUnit XML artifact through
the repository's `.config/nextest.toml`, and CI SHALL upload that artifact. CI
SHALL build the Qt application under a pinned Qt 6 in a headless job and SHALL
run its `--headless --self-test` mode there. CI SHALL run a separate job that
installs the external comparison tools (ImageMagick and `psd-tools`) and runs the
workspace suite excluding the Qt application (`cargo test --workspace --exclude
pictura_app`), so that job neither installs Qt nor rebuilds the application, and
SHALL cache Rust registry sources and compiled artifacts between runs. The
system SHALL run `cargo deny check` when a `deny.toml` is present, installing
cargo-deny from a prebuilt release rather than compiling it from source. Every
CI job SHALL set a `timeout-minutes` bound.

#### Scenario: Push or pull request
- **WHEN** a commit is pushed to `main`, or a pull request is opened or synchronized
- **THEN** the workflow runs the format, clippy, and test steps and fails the job on a non-zero exit

#### Scenario: A feature-branch push is covered once
- **WHEN** a commit is pushed to a branch that has an open pull request
- **THEN** the build jobs run once, via the pull request `synchronize` event, rather than once for the push and once for the pull request

#### Scenario: Tests run under nextest
- **WHEN** the workflow runs the test step
- **THEN** the workspace suite runs under `cargo nextest run --workspace` and doctests run explicitly

#### Scenario: Lint and test run in parallel
- **WHEN** the workflow starts
- **THEN** format and clippy run in the `lint` job while the test suite runs in the `test` job, independent of each other

#### Scenario: The local fast gate uses nextest
- **WHEN** `scripts/verify-fast.sh` runs the test step
- **THEN** it runs `cargo nextest run --workspace --no-fail-fast` followed by `cargo test --workspace --doc`

#### Scenario: The JUnit artifact is uploaded
- **WHEN** the workflow test step completes
- **THEN** `target/nextest/default/junit.xml` has been written and is uploaded as an artifact

#### Scenario: The Qt app self-tests headlessly
- **WHEN** the workflow builds the Qt application with the pinned Qt 6 toolchain
- **THEN** it runs `--headless --self-test` with and without a document fixture and fails the job on a non-zero exit

#### Scenario: External oracles are exercised
- **WHEN** the oracle job runs with ImageMagick and `psd-tools` installed and `pictura_app` excluded
- **THEN** the tests that otherwise self-skip compare against the external tools instead of passing vacuously, without a Qt install

#### Scenario: Compiled artifacts are reused
- **WHEN** a later workflow run starts with a populated Rust registry cache and compiler cache
- **THEN** dependencies and previously compiled objects are restored rather than rebuilt from scratch

#### Scenario: A runaway job is bounded
- **WHEN** a CI job exceeds its configured `timeout-minutes`
- **THEN** the run is cancelled rather than consuming minutes indefinitely

### Requirement: Non-goal guard
The system SHALL provide `scripts/guard.sh` that fails when tracked source under
`crates/` references `.8bf` plugin binaries, when an `artboard*` file appears
under `crates/`, or when `docs/` is modified without a `TASK-ALLOWS-DOCS` marker,
and SHALL exit 0 when none of those conditions hold.

#### Scenario: .8bf reference under crates/
- **WHEN** a tracked `crates/` source contains an `.8bf` reference
- **THEN** the guard exits non-zero and prints the offending lines

#### Scenario: artboard artifact under crates/
- **WHEN** a file named `artboard*` exists under `crates/`
- **THEN** the guard exits non-zero and names the file

#### Scenario: docs modified without a marker
- **WHEN** `docs/` differs from the guard base and no commit carries `TASK-ALLOWS-DOCS`
- **THEN** the guard exits non-zero and lists the changed files

#### Scenario: Clean tree
- **WHEN** no non-goal artifact is present and docs are unchanged or allowed
- **THEN** the guard exits 0

### Requirement: A live control-server end-to-end check

The repository SHALL provide `scripts/verify-control.sh` that launches a separate
`pictura --headless --control` process on a temporary local socket, connects over
that socket, and drives a documented recipe over the newline-delimited JSON
protocol: read `status`, apply a rectangular selection that is a strict subset of
the active layer, apply a `filter`, assert the filter changed at least one pixel
and changed no pixel outside the selection, `undo` and assert every pixel is
restored, and take a `screenshot`. The script SHALL exit non-zero on any mismatch
or if `./build/pictura` is absent, and SHALL tear the process down and remove the
socket on exit. `scripts/verify-full.sh` SHALL run the script after it builds the
binary, the build-free `verify-fast.sh` SHALL NOT, and the CI headless job SHALL
run it too.

#### Scenario: The live recipe passes on a built tree

- **WHEN** `scripts/verify-control.sh` runs against a freshly built binary and the committed fixture
- **THEN** every step's assertion holds and the script exits 0

#### Scenario: A mismatch fails the gate

- **WHEN** an assertion does not hold (a pixel that should have changed did not, the undo did not restore it, or the screenshot is not a PNG)
- **THEN** the script prints the failing step and exits non-zero

#### Scenario: The process and socket are cleaned up

- **WHEN** the script finishes, successfully or not
- **THEN** the launched `pictura` process is terminated and the temporary socket directory is removed

#### Scenario: The full gate runs the live check

- **WHEN** `scripts/verify-full.sh` runs
- **THEN** it runs `scripts/verify-control.sh` after the build and fails if the script fails

#### Scenario: CI runs the live check

- **WHEN** the CI headless job runs after building the app
- **THEN** it runs `scripts/verify-control.sh` and the job fails if the script fails

### Requirement: Superseded CI runs are cancelled
The CI workflow SHALL set a `concurrency` group keyed on the workflow and ref
with `cancel-in-progress: true`, so a newer push to the same ref cancels the
older in-progress run.

#### Scenario: Rapid pushes on one ref
- **WHEN** a second push to the same ref starts while the first run is in progress
- **THEN** the first run is cancelled and the second proceeds

#### Scenario: Different refs run independently
- **WHEN** pushes arrive on two different refs
- **THEN** both runs proceed without cancelling each other

### Requirement: Docs-only changes run guards, not builds
The build jobs (format/lint/test, Qt headless, and oracles) SHALL be skipped when
every changed path matches `docs/**`, `openspec/**`, `**/*.md`, or `.serena/**`,
using workflow-level `paths-ignore`. A separate workflow with no path filter
SHALL run `scripts/guard.sh`, `openspec validate --all --strict`, and
`scripts/check-file-size.sh` on every push and pull request, so a docs-only
change is still gated. A mixed change SHALL run the build jobs, because
`paths-ignore` skips a workflow only when all changed paths match.

#### Scenario: Docs-only change
- **WHEN** a push or pull request changes only `docs/**`, `openspec/**`, `*.md`, or `.serena/**`
- **THEN** the build jobs do not run and the guards workflow runs the guard, OpenSpec validation, and file-size checks

#### Scenario: Code change
- **WHEN** a change touches any path outside the ignored set
- **THEN** the build jobs run as before

#### Scenario: Guards fail
- **WHEN** `scripts/guard.sh`, `openspec validate --all --strict`, or `scripts/check-file-size.sh` exits non-zero
- **THEN** the guards workflow fails

### Requirement: Local verification fast path and compiler caching
`scripts/verify-fast.sh` SHALL detect the changed paths of the working tree
(committed diff against the merge base with `main`, plus untracked files). When
every changed path matches `docs/**`, `openspec/**`, `*.md`, or `.serena/**`, it
SHALL run only `scripts/guard.sh` and `openspec validate --all --strict` and skip
format, lint, and tests. When the changed-path set is empty or contains any other
path, it SHALL run the full gate. `scripts/verify-full.sh` SHALL pass
`-DCMAKE_CXX_COMPILER_LAUNCHER=sccache` to the CMake configure only when
`sccache` is available, and `DEVELOPING.md` SHALL document opting into
`RUSTC_WRAPPER=sccache` and cleaning `target/`.

#### Scenario: Docs-only working diff
- **WHEN** `scripts/verify-fast.sh` runs and every changed path is `docs/**`, `openspec/**`, `*.md`, or `.serena/**`
- **THEN** it runs the guard and OpenSpec validation and does not run fmt, clippy, or tests

#### Scenario: Clean tree or code change
- **WHEN** the working diff is empty or contains a path outside the fast-path set
- **THEN** `scripts/verify-fast.sh` runs the full gate

#### Scenario: sccache present
- **WHEN** `scripts/verify-full.sh` runs on a machine where `sccache` is on `PATH`
- **THEN** the CMake configure receives `-DCMAKE_CXX_COMPILER_LAUNCHER=sccache`

#### Scenario: sccache absent
- **WHEN** `sccache` is not on `PATH`
- **THEN** `scripts/verify-full.sh` configures CMake without a compiler launcher and still builds
