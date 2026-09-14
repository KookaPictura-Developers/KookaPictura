## ADDED Requirements

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
The system SHALL run, on every push and pull request, `cargo fmt --all --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, and
`cargo test --workspace` on the pinned toolchain, and SHALL run `cargo deny
check` when a `deny.toml` is present.

#### Scenario: Push or pull request
- **WHEN** a commit is pushed or a pull request is opened
- **THEN** the workflow runs the format, clippy, and test steps and fails the job on a non-zero exit

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
