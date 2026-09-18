## Context

Tests live in three layers. Rust unit, integration, and oracle tests use std
`#[test]` (600 tests, 8 `#[ignore]`); doctests run separately because nextest
does not execute them; the C++ app self-test is one flat 7212-line
`runSelfTest()` in `crates/pictura-app/cpp/selftest.cpp` that prints `key=value`
progress and `FAIL` lines and returns an exit code in ~2..198. CI already runs
`cargo nextest run --workspace` and `cargo test --workspace --doc`, but
`scripts/verify-fast.sh` still runs `cargo test --workspace` and greps
`test result: ok. N passed`. The Python `scripts/*.py` are oracle CLIs, not
tests. `pictura-testkit` is the only shared test utility.

Constraints: the repo has no `.config/` directory today; `docs/` edits need a
`TASK-ALLOWS-DOCS` marker; no new dependencies; `selftest.cpp` is allowlisted at
exactly 7212 lines and the instrumentation must not push it past that.

## Goals / Non-Goals

**Goals:** one end-of-run report covering every test layer; structured,
parseable C++ self-test output; nextest as the single workspace runner in both
the local fast gate and CI; a JUnit artifact CI can upload; the same reporter
for `verify-fast.sh` and `verify-full.sh`; no new dependencies.

**Non-Goals:** replacing the C++ self-test with Qt Test or CTest; rewriting the
~170 existing checks' logic; converting Python oracle CLIs into pytest tests;
byte-exact pytest/vitest output; parallelizing the self-test.

## Decisions

### D1 — Scope is all layers
The report covers Rust tests + doctests + the C++ self-test. Oracle tests are
Rust tests, so they are covered by nextest without separate handling. Python
scripts stay CLIs. Chosen over "Rust only" because the self-test is the layer
with no counts today and the main motivation.

### D2 — Full C++ instrumentation via a `SelfTestReport` harness
New `crates/pictura-app/cpp/selftest_report.h` / `.cpp` define `SelfTestReport`
with methods emitting machine tokens to standard error:

```
pictura self-test: SUITE <suite> <name>
pictura self-test: PASS <suite> <name> [detail]
pictura self-test: SKIP <suite> <name> <reason>
pictura self-test: FAIL <suite> <name> <code> <message>
pictura self-test: SUMMARY passed=<n> failed=<n> skipped=<n>
```

Suite is derived from the existing check-name prefix (`m47_float_close` →
`m47`); keys with no recognised milestone prefix map to `core`/`misc`. Each of
the ~170 checks gains an explicit suite+name, and each `FAIL` gains its existing
exit code. Exit codes are unchanged, so any external caller keying on them keeps
working. Chosen over leaving the self-test structureless: the token stream is
what lets one Python parser report all layers uniformly.

**File-size ceiling.** `selftest.cpp` may not exceed 7212 lines. Instrumentation
therefore REPLACES existing output in place: each existing two-line
`fprintf`+`fflush` progress site becomes one `ST_PASS(...)` call, and each
existing `FAIL fprintf` + `return N` pair becomes one `ST_FAIL(N, ...)` call.
The file shrinks by roughly 400 lines. The allowlist entry is lowered to the new
size. Work proceeds one milestone at a time, running the full verify after each
chunk; if a chunk would exceed the ceiling, that milestone splits into
`selftest_<milestone>.cpp` rather than raising the ceiling.

### D3 — nextest is the primary runner locally and in CI
`scripts/verify-fast.sh` runs `cargo nextest run --workspace --no-fail-fast`
plus `cargo test --workspace --doc`. `--no-fail-fast` so the reporter sees every
failure rather than the first. CI keeps nextest as primary (the existing
"when available" fallback wording is tightened) and adds an artifact upload.
Chosen over continuing to parse `cargo test` text: nextest is already the CI
tool, so the local gate stops diverging.

### D4 — JUnit XML in a new `.config/nextest.toml`
Add `[profile.default.junit] path = "junit.xml"` so nextest writes
`target/nextest/default/junit.xml` (already gitignored under `target/`). CI
uploads it. Chosen over a custom nextest format or a crate: one config line, no
dependency, standard format the reporter can parse with stdlib `xml.etree`.

### D5 — Reporter: `scripts/report_tests.py`
python3 stdlib only (`xml.etree`, `argparse`). Inputs are the JUnit XML, the
doctest summary text, and captured self-test stdout/stderr. Output is
pytest/vitest-like "close enough", not 1:1:

```
pictura test report
────────────────────────────────────────────────
Rust suites (nextest)
  pictura-core                12 passed
  pictura-codec               20 passed
  pictura-render             118 passed, 2 skipped
  subtotal                   600 passed · 8 skipped
App self-test (offscreen Qt)
  m47  panel-interaction       9 passed
  subtotal                   170 passed
Doctests                       3 passed
────────────────────────────────────────────────
TOTAL  773 passed · 8 skipped · 0 failed

FAILURES
  1) pictura-codec::oracle::psd_tools_sees_resized_document
        assertion `left == right` failed: ...
  2) m47::compact_shade (exit 195)
        M47 compact shade
```

It exits non-zero on any failure and supports `--color auto|always|never`.
Chosen over a shell-only parser: JUnit XML and token parsing in POSIX shell is
more code and more fragile.

### D6 — `scripts/test-report.sh` orchestrates
Runs nextest (yielding the JUnit XML), doctests, and both self-test invocations
(bare `--self-test` and with a `.psd` fixture), capturing each output, then
invokes the reporter. `verify-full.sh` reuses the same capture + reporter flow.
nextest's own human output still streams live; the reporter is the end-of-run
summary.

## Risks / Trade-offs

- [Instrumenting `selftest.cpp` in place is a large, mechanical edit] →
  milestone-sized chunks, full verify after each, and the split fallback if the
  ceiling would be crossed.
- [The token format is a new interface other tooling could depend on] → documented
  in `docs/dev/testing-conventions.md` and covered by a reporter fixture test.
- [Doctest output parsing can drift across Rust versions] → the parser matches
  the stable `test result: ok. N passed` summary and a missing summary is
  reported as an error rather than silently counted zero.
- [A deliberately failing test could be missed by the reporter] → tasks include
  a deliberate-failure check proving a non-zero exit and a FAILURES entry.
- [`.config/nextest.toml` is the first `.config/` entry] → contained, and nextest's
  documented lookup path.
