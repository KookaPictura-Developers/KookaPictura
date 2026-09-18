## 1. nextest configuration and fast gate

- [x] 1.1 Add `.config/nextest.toml` with `[profile.default]` status levels, `fail-fast = false`, and `[profile.default.junit] path = "junit.xml"`.
- [x] 1.2 Confirm a nextest run writes `target/nextest/default/junit.xml` and that `target/` already ignores it.
- [x] 1.3 Switch `scripts/verify-fast.sh` to `cargo nextest run --workspace --no-fail-fast` plus `cargo test --workspace --doc`, dropping the `test result: ok` text grep.

## 2. C++ self-test harness

- [x] 2.1 Add `crates/pictura-app/cpp/selftest_report.h` and `.cpp` with `SelfTestReport` emitting `SUITE`/`PASS`/`SKIP`/`FAIL`/`SUMMARY` tokens and deriving the suite from the check-name milestone prefix (bare keys → `core`/`misc`).
- [x] 2.2 Register the new `selftest_report` sources in `CMakeLists.txt`.
- [x] 2.3 Instrument one milestone at a time, replacing each existing two-line `fprintf`+`fflush` progress site with one pass call and each `FAIL` `fprintf`+`return N` pair with one fail call; run the full verify after each chunk.
- [x] 2.4 Keep `selftest.cpp` at or below its 7212-line allowlist ceiling; if a chunk would exceed it, split that milestone into `selftest_<milestone>.cpp` instead of raising the ceiling.
- [x] 2.5 Lower or remove the `selftest.cpp` entry in `scripts/file-size-allowlist.txt` to the new size and confirm `scripts/check-file-size.sh` passes.
- [x] 2.6 Confirm the process exit codes are unchanged and the `SUMMARY` counts equal the emitted token counts.

## 3. Reporter

- [x] 3.1 Write `scripts/report_tests.py` (python3 stdlib only) parsing the JUnit XML, the doctest summary, and the self-test token stream.
- [x] 3.2 Print per-suite lines, per-layer subtotals, a `TOTAL` line, and a `FAILURES` section; exit non-zero on any failure; support `--color auto|always|never`.
- [x] 3.3 Add a runnable check for the parser against a small fixture (a `demo()` self-check or one `test_*.py`).
- [x] 3.4 Write `scripts/test-report.sh` to run nextest, doctests, both self-test invocations, then the reporter.

## 4. Wiring verify-full and CI

- [x] 4.1 Make `scripts/verify-full.sh` reuse the same capture-and-report flow.
- [x] 4.2 Add the JUnit artifact upload to `.github/workflows/ci.yml`, keep nextest as the primary test step, and keep doctests explicit.
- [x] 4.3 Confirm nextest's live per-test output still streams while the reporter runs only at the end.

## 5. Documentation

- [x] 5.1 Document the token protocol and the report command in `docs/dev/testing-conventions.md`.
- [x] 5.2 Update the testing section of `AGENTS.md` (nextest gate, report command, artifact path).
- [x] 5.3 Ensure the commit touching `docs/` carries a `TASK-ALLOWS-DOCS` marker.

> The single `docs/` commit for 5.1–5.3 carries the `TASK-ALLOWS-DOCS` marker
> (added by the orchestrator when it commits).

## 6. Verification

- [x] 6.1 Run `openspec validate --all --strict` and fix findings.
- [x] 6.2 Run `scripts/verify-fast.sh` and confirm it is green under nextest.
- [x] 6.3 Run `scripts/test-report.sh` and confirm the unified report prints and exits 0.
- [x] 6.4 Deliberate-failure check: briefly introduce one failing Rust test and one failing self-test check, confirm the reporter lists both under `FAILURES` and exits non-zero, then revert both.
- [x] 6.5 Build via CMake and run `./build/pictura --headless --self-test` both with and without a document fixture.
