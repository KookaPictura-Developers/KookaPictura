# Spec Delta

## MODIFIED Requirements

### Requirement: CI runs format, lint, and tests
The system SHALL run, on every push to `main` and on every pull request,
`cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D
warnings`, and the workspace test suite on the pinned toolchain, except when
the "Jobs are skipped when the change cannot affect them" requirement skips a
job because the changed paths cannot affect it. Format and
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
- **THEN** the workflow runs the format, clippy, and test steps in every job the changed-path rules do not skip, and fails the job on a non-zero exit

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

## ADDED Requirements

### Requirement: Jobs are skipped when the change cannot affect them
For a change that is not docs-only (the docs-only case is governed by
"Docs-only changes run guards, not builds"), CI SHALL evaluate the changed
paths — the pull request diff, or the commits of a push — and SHALL skip each
build job (`lint`, `test`, `qt-headless`, `oracles`) whose inputs the change
cannot affect, while still starting the workflow. Skipping SHALL happen per job
inside a workflow that runs on the event, so a skipped job reports a successful
conclusion instead of reporting nothing at all; a skipped job MUST NOT depend on
a workflow-level path filter, so that a check later made required can never
leave a pull request waiting forever for a report. The evaluation SHALL apply
these rules:

- Any changed path that is not recognized by the rules below SHALL cause every
  job to run (run-on-doubt).
- Any change under `.github/workflows/**` SHALL cause every job to run.
- Any change to Rust sources or other inputs that feed the Rust build SHALL
  cause every job to run, because the Rust staticlib is linked into the Qt
  application and exercised by the test and oracle suites.
- A change limited to the hand-written C++ sources under
  `crates/pictura-app/cpp/` SHALL run `qt-headless` (and the guards workflow
  continues to run as before) and SHALL skip `lint`, `test`, and `oracles`,
  because the cargo build does not compile those translation units.
- The CI workflow file itself SHALL be treated as a path that causes every job
  to run, so editing the gating rules re-runs the full suite.

#### Scenario: C++-only change
- **WHEN** a pull request changes only files under `crates/pictura-app/cpp/`
- **THEN** `qt-headless` runs, `lint`, `test`, and `oracles` are skipped, and the guards workflow still runs

#### Scenario: Rust change runs everything
- **WHEN** a pull request changes any file under `crates/` outside `crates/pictura-app/cpp/`
- **THEN** `lint`, `test`, `qt-headless`, and `oracles` all run

#### Scenario: Mixed change runs everything
- **WHEN** a pull request changes both C++ sources and any other recognized input
- **THEN** every build job runs

#### Scenario: Unrecognized path runs everything
- **WHEN** a pull request changes a path not covered by the rules (for example a root-level config file)
- **THEN** every build job runs

#### Scenario: Workflow edit re-runs the suite
- **WHEN** a pull request changes `.github/workflows/ci.yml`
- **THEN** every build job runs even though the diff is otherwise empty of source

#### Scenario: A skipped job still reports
- **WHEN** the gating condition skips a build job on a pull request
- **THEN** that job's check run reports a successful conclusion rather than never reporting, so it cannot block a required check
