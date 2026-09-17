## Context

The application-shell contract already demands the app run under a headless
display (`openspec/specs/application-shell/spec.md`, "Headless window shell"),
but headlessness is achieved only by wrapping the executable in `xvfb-run`. CI
does not exercise the Qt app at all: `.github/workflows/ci.yml` has one `rust`
job whose `cargo test --workspace` includes `pictura_app`, so it cannot compile
on a stock runner (`crates/pictura-app/build.rs` requires `qmake6`), and no Qt is
installed. The local `target/` is ~24 GB (`deps` 16 GB, `incremental` 4.2 GB)
because `[profile.test] opt-level = 2` in `Cargo.toml` applies to all 123
dependencies, and the CMake build is serial (Unix Makefiles, no `--parallel`)
with `ld.bfd`.

## Goals / Non-Goals

**Goals:**
- An explicit `--headless` mode that needs no X/Wayland and is asserted in the
  app self-test.
- CI that actually builds the Qt app and runs its headless self-test, plus a
  full external-oracle tier.
- Materially faster local and CI builds/links, and parallel test execution,
  without changing test semantics or coverage.

**Non-Goals:**
- Any OCR/GPU/window behavior change, new Rust API, or compositor work.
- Multi-platform CI matrix, Flatpak/packaging, or a `deny.toml`.
- Rewriting the 5000-line C++ self-test into QTest/CTest.
- Fine-tuning `[profile.test]` beyond the dependency/workspace split.

## Decisions

### 1. `--headless` selects the offscreen QPA before `QApplication`
Detected in the existing pre-`QApplication` `argv` scan in `main.cpp`; when set
and `QT_QPA_PLATFORM` is unset, `qputenv("QT_QPA_PLATFORM", "offscreen")`. The
existing `--self-test` xcb override already guards on the variable being unset,
so `--headless` wins. `frame.show()` is kept: the offscreen plugin supports it
and the self-test asserts widget visibility/layout.
- *Alternatives*: skip `show()` in headless — breaks layout/visibility
  assertions; rely solely on `QT_QPA_PLATFORM` externally — leaves no in-app
  affordance and no assertion.
- `--headless` with no PSD and no `--self-test` implies `--self-test`, so it
  never blocks in `app.exec()`. `--interop-probe` is not forced to offscreen
  (the offscreen plugin cannot create a Vulkan platform instance).

### 2. `lld` via a repo `.cargo/config.toml`
`[target.x86_64-unknown-linux-gnu] rustflags = ["-C", "link-arg=-fuse-ld=lld"]`
applies to `cargo test`/`clippy` and to Corrosion's cargo invocation from CMake.
The `verify-fast.sh` `RUSTFLAGS=…mold` hack is removed.
- *Alternatives*: mold — equivalent, but `lld` is the more portable apt package
  and is already present locally; per-script RUSTFLAGS — does not reach CMake.

### 3. `[profile.test]` dependencies drop to `opt-level = 0`
`[profile.test.package."*"] opt-level = 0` plus an explicit `opt-level = 2`
override for all 11 workspace members keeps oracle runtime identical while the
dependency build shrinks. Measured before/after during implementation; revert if
an oracle test regresses materially.
- *Alternatives*: `opt-level = 1` everywhere — slows every pixel test; leaving
  `2` — keeps the 16 GB dependency tree.

### 4. Ninja + `--parallel` for CMake
`verify-full.sh` configures `-G Ninja -DCMAKE_EXE_LINKER_FLAGS=-fuse-ld=lld` and
builds `--parallel`; CI does the same. The existing `build/` (Unix Makefiles)
must be deleted once.

### 5. CI: three jobs, registry cache + `sccache` artifacts
- `rust` (fast): fmt, clippy, `cargo nextest run --workspace`, `cargo test
  --workspace --doc`; Qt installed because `pictura_app` must compile.
- `qt-headless`: Qt 6.11.1 via `jurplel/install-qt-action`, `lld`/`ninja-build`,
  CMake/Ninja build, `./build/pictura --headless --self-test` (with and without
  the `two_layers.psd` fixture).
- `oracles`: installs ImageMagick + `psd-tools`, then `cargo test --workspace`
  for full coverage (this is why the fast job's self-skipping tests stay
  meaningful).
`Swatinem/rust-cache@v2` caches registry/git only (`cache-targets: false`);
`mozilla-actions/sccache-action` caches compiled objects for both Rust
(`RUSTC_WRAPPER`) and C++ (`CMAKE_CXX_COMPILER_LAUNCHER`). `CARGO_INCREMENTAL=0`
in CI. Qt 6.11.1 is pinned for parity with the dev box (apt's 6.4.2 is too old).
- *Alternatives*: rust-cache on the whole 23 GB `target/` (exceeds the ~10 GB
  repo cache cap); apt Qt (version mismatch); one slow job (loses fast feedback).

### 6. Non-zero exit code `152`
`main.cpp`'s highest exit code is `151` (the M43 session check), so the headless
platform assertion uses `152`.

## Risks / Trade-offs

- **Offscreen screen geometry.** Self-tests were proven under Xvfb (1280×1024);
  the offscreen plugin defaults smaller and the frame is `resize(1100,700)`.
  → First implementation step runs `--headless --self-test`; if layout
  assertions fail, CI keeps `xvfb-run` and `--headless` stays a dev affordance.
- **`sccache` and cxx-qt generated C++.** → Verify `sccache --show-stats` hit
  rate on a second CI run; the CMake launcher must wrap the generated TU.
- **`profile.test` split changes fingerprints.** → One full rebuild; confirm
  oracle runtime with a timed run before/after and revert the split on
  regression.
- **`install-qt-action` 6.11.1 availability.** → Fall back to the newest 6.11.x
  and record the divergence.
- **Cache eviction.** Registry-only + sccache is intended to fit the ~10 GB cap;
  check the second run reports a cache hit.
- **Ninja generator switch.** → Documented one-time `rm -rf build`.
- **Spec drift.** `verification-harness`'s CI requirement is MODIFIED, so the
  nextest/Qt/oracle split must be worded as the contract, and local
  `cargo test --workspace` must remain valid.

## Migration Plan

Land `.cargo/config.toml` and the profile split first (one rebuild), then
`--headless`, then CI, then docs. Rollback is per-file: delete
`.cargo/config.toml`, restore `[profile.test]`, revert the CI job split; no data
or API migration.
