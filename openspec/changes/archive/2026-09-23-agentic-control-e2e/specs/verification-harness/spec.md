## ADDED Requirements

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
