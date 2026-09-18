## Why

The repo has three test layers — Rust unit/integration/oracle tests, Rust
doctests, and the C++ app self-test — but no single view of them. The C++
self-test prints ~170 unstructured `key=value` progress lines and ~216
`FAIL` lines, so a run gives no pass/skip/fail counts per milestone; the local
fast gate still parses `cargo test` text while CI already uses nextest. A
pytest/vitest-style unified report makes the whole suite legible in one place
and switches the fast gate to the tool CI already relies on.

## What Changes

- **Structured C++ self-test.** A new `SelfTestReport` harness emits machine
  tokens (`SUITE` / `PASS` / `SKIP` / `FAIL` / `SUMMARY`), instrumenting every
  self-test check with an explicit suite and name. Suite is the existing
  milestone prefix (`m47_float_close` → `m47`; bare keys → `core`/`misc`).
  Exit codes stay unchanged. `selftest.cpp` must not exceed its 7212-line
  allowlist ceiling, so existing output lines are replaced in place (the file
  shrinks) and the allowlist entry is lowered.
- **nextest is the primary runner.** `scripts/verify-fast.sh` runs
  `cargo nextest run --workspace --no-fail-fast` plus explicit doctests instead
  of `cargo test`.
- **JUnit artifact.** A new `.config/nextest.toml` writes
  `target/nextest/default/junit.xml`; CI uploads it as an artifact.
- **Unified reporter.** New `scripts/report_tests.py` (python3 stdlib only)
  parses the JUnit XML, the doctest summary, and the self-test tokens, then
  prints per-suite counts, subtotals, a total, and a failures section; exits
  non-zero on any failure. New `scripts/test-report.sh` orchestrates nextest,
  doctests, and both self-test invocations, then runs the reporter.
  `verify-full.sh` uses the same capture + reporter. No new dependencies.

## Capabilities

### New Capabilities

- `test-reporting`: the unified end-of-run test report, the structured C++
  self-test token protocol, and the nextest configuration that feeds both.

### Modified Capabilities

- `verification-harness`: nextest becomes the primary workspace runner (rather
  than "when available"), `scripts/verify-fast.sh` runs nextest plus explicit
  doctests, and CI emits and uploads the JUnit artifact.

## Impact

- New: `.config/nextest.toml`, `scripts/test-report.sh`, `scripts/report_tests.py`,
  `crates/pictura-app/cpp/selftest_report.h`,
  `crates/pictura-app/cpp/selftest_report.cpp`.
- Edit: `scripts/verify-fast.sh`, `scripts/verify-full.sh`,
  `.github/workflows/ci.yml`, `CMakeLists.txt` (add `selftest_report` sources),
  `crates/pictura-app/cpp/selftest.cpp`, `scripts/file-size-allowlist.txt`,
  `docs/dev/testing-conventions.md`, `AGENTS.md`.
- No new dependencies. Touching `docs/` requires a `TASK-ALLOWS-DOCS` marker on
  the eventual commit.
