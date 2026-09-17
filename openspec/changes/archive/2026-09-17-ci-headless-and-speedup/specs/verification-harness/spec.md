## MODIFIED Requirements

### Requirement: CI runs format, lint, and tests
The system SHALL run, on every push and pull request, `cargo fmt --all --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, and the workspace test
suite on the pinned toolchain. The test suite SHALL be runnable with `cargo
nextest run --workspace` when nextest is available and with `cargo test
--workspace` otherwise, and doctests SHALL be run explicitly. CI SHALL build the
Qt application under a pinned Qt 6 and SHALL run its `--headless --self-test`
mode. CI SHALL run a separate job that installs the external comparison tools
(ImageMagick and `psd-tools`) and runs the tests that depend on them, and SHALL
cache Rust registry sources and compiled artifacts between runs. The system SHALL
run `cargo deny check` when a `deny.toml` is present.

#### Scenario: Push or pull request
- **WHEN** a commit is pushed or a pull request is opened
- **THEN** the workflow runs the format, clippy, and test steps and fails the job on a non-zero exit

#### Scenario: Tests run under nextest
- **WHEN** the workflow runs the test step and nextest is available
- **THEN** the workspace suite runs under `cargo nextest run --workspace` and doctests run explicitly

#### Scenario: The Qt app self-tests headlessly
- **WHEN** the workflow builds the Qt application with the pinned Qt 6 toolchain
- **THEN** it runs `--headless --self-test` with and without a document fixture and fails the job on a non-zero exit

#### Scenario: External oracles are exercised
- **WHEN** the oracle job runs with ImageMagick and `psd-tools` installed
- **THEN** the tests that otherwise self-skip compare against the external tools instead of passing vacuously

#### Scenario: Compiled artifacts are reused
- **WHEN** a later workflow run starts with a populated Rust registry cache and compiler cache
- **THEN** dependencies and previously compiled objects are restored rather than rebuilt from scratch
