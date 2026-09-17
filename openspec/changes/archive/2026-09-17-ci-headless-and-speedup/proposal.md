## Why

The `application-shell` spec already requires the app to run under the offscreen
QPA plugin, but the executable has no headless flag — it always parses
`--self-test`/`--interop-probe` and constructs a full `QApplication`, so headless
runs depend on an external `xvfb-run`. CI never verifies any of it: the single
`rust` job runs only `cargo test --workspace`, has no Qt, and cannot even compile
`pictura-app` on a stock runner. The same job is slow for avoidable reasons:
serial CMake via Unix Makefiles, `ld.bfd` linking, `opt-level = 2` applied to all
123 dependencies, and no artifact caching.

## What Changes

- **`--headless` flag.** The app gains an explicit headless mode that selects the
  offscreen QPA plugin before `QApplication` is constructed. With no document and
  no `--self-test`, `--headless` implies `--self-test` so it never blocks.
- **Offscreen self-test assertion.** A `--headless` self-test run verifies
  `QApplication::platformName() == "offscreen"` and exits non-zero otherwise.
- **Fast linkers.** A repo `.cargo/config.toml` links with `lld` for every cargo
  invocation (including CMake/Corrosion). The `verify-fast.sh` mold-only hack is
  removed in favour of the shared config.
- **Cheaper test profile.** `[profile.test]` drops dependencies to `opt-level = 0`
  while keeping every workspace crate at `2`, so oracle runtime is unchanged but
  the dependency build shrinks.
- **Parallel CMake builds.** `verify-full.sh` configures with Ninja and builds
  with `--parallel`; the self-tests run headless via `--headless` instead of
  `xvfb-run`.
- **CI is rebuilt into three jobs:** a fast `rust` job (fmt, clippy, nextest,
  doctests), a `qt-headless` job (Qt 6.11.1 + CMake build + `--headless
  --self-test`), and a full `oracles` job that installs ImageMagick and
  `psd-tools` and runs the whole suite. CI gains registry caching, `sccache`
  artifact caching, `lld`, Ninja, and `CARGO_INCREMENTAL=0`.
- **Docs.** `AGENTS.md` documents the test suite and headless mode; the testing
  strategy and STATE anchors are refreshed.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `application-shell`: add a requirement for an explicit `--headless` flag (the
  existing "Headless window shell" requirement keeps the Xvfb/offscreen contract).
- `verification-harness`: modify "CI runs format, lint, and tests" to allow the
  nextest runner, the Qt headless job, caching, and the external-oracle tier.

## Impact

- `crates/pictura-app/cpp/main.cpp` — flag parsing, offscreen selection,
  self-test platform assertion (exit code 152).
- `Cargo.toml` — `[profile.test]` dependency override.
- `.cargo/config.toml` — new file (lld).
- `scripts/verify-full.sh`, `scripts/verify-fast.sh` — linker, Ninja, headless.
- `.github/workflows/ci.yml` — three jobs, Qt, caching, nextest, oracles.
- `AGENTS.md`, `docs/11-cross-cutting/testing-strategy.md`,
  `docs/01-architecture/build-and-packaging.md`, `docs/dev/STATE.md`.
- No Rust API, document, codec, compositor, or new-dependency change.
