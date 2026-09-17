## 1. Headless flag

- [x] 1.1 In `crates/pictura-app/cpp/main.cpp`, detect `--headless` in the pre-`QApplication` `argv` scan and, when set with `QT_QPA_PLATFORM` unset, `qputenv("QT_QPA_PLATFORM", "offscreen")`
- [x] 1.2 Parse `--headless` in the main argument loop (add a `bool headless`), and after parsing set `selfTest = true` when `--headless` is given with no document path and no explicit `--self-test`
- [x] 1.3 In the self-test block, when `--headless` was passed, assert `QApplication::platformName() == "offscreen"` and `return 152` on mismatch with a `pictura self-test: FAIL: …` line
- [x] 1.4 Confirm `--headless` wins over the existing `--self-test` xcb override and does not affect `--interop-probe`

## 2. Build speed

- [x] 2.1 Add `.cargo/config.toml` with `[target.x86_64-unknown-linux-gnu] rustflags = ["-C", "link-arg=-fuse-ld=lld"]`
- [x] 2.2 Split `[profile.test]` in the root `Cargo.toml`: `opt-level = 0` for dependencies via `[profile.test.package."*"]`, with an explicit `opt-level = 2` override for every workspace member
- [x] 2.3 Remove the `RUSTFLAGS=…mold` export from `scripts/verify-fast.sh` (now covered by `.cargo/config.toml`)
- [x] 2.4 In `scripts/verify-full.sh`, configure CMake with `-G Ninja -DCMAKE_EXE_LINKER_FLAGS=-fuse-ld=lld` (fresh build) and build with `--parallel`
- [x] 2.5 In `scripts/verify-full.sh`, run `./build/pictura --headless --self-test` (with and without the `two_layers.psd` fixture) instead of `xvfb-run`

## 3. CI

- [x] 3.1 Add `lld`, `ninja-build`, sccache, and pinned Qt 6.11.1 setup (`jurplel/install-qt-action`) shared by the jobs, plus `Swatinem/rust-cache` (registry only) and `CARGO_INCREMENTAL=0`
- [x] 3.2 Rebuild the `rust` job: fmt, clippy, `cargo nextest run --workspace` (installed via `taiki-e/install-action`), and `cargo test --workspace --doc`
- [x] 3.3 Add the `qt-headless` job: CMake/Ninja build, then `./build/pictura --headless --self-test` with and without the fixture
- [x] 3.4 Add the `oracles` job: install ImageMagick and `psd-tools`, run `cargo test --workspace` for full coverage
- [x] 3.5 Configure `sccache` for both Rust (`RUSTC_WRAPPER`) and C++ (`CMAKE_CXX_COMPILER_LAUNCHER`); keep the `cargo deny` steps gated on `deny.toml`

## 4. Documentation

- [x] 4.1 Update `AGENTS.md`: document the test suite (nextest/cargo test, oracle self-skip vs installed), the headless mode (`--headless`, offscreen), and the new cache/linker/parallel build commands
- [x] 4.2 Update `docs/dev/STATE.md` command list and milestone anchor (M43 archived; this change's status)
- [x] 4.3 Update `docs/01-architecture/build-and-packaging.md` Qt QPA options with `offscreen`, and `docs/11-cross-cutting/testing-strategy.md` with the actual `--headless` mechanism and CI tiers

## 5. Verification

- [x] 5.1 `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` clean
- [x] 5.2 `cargo nextest run --workspace` = 588 passed / 7 ignored (counts unchanged) and `cargo test --workspace --doc` green
- [x] 5.3 `cmake -S . -B build -G Ninja` + `cmake --build build --parallel` succeeds with lld
- [x] 5.4 `QT_QPA_PLATFORM= ./build/pictura --headless --self-test` and the fixture run exit 0, with the offscreen assertion passing
- [x] 5.5 Record before/after `cargo build --tests --timings` and CMake wall-clock deltas; revert the `[profile.test]` split if oracle runtime regresses materially
- [x] 5.6 `openspec validate --all --strict` green

## 6. Archive

- [x] 6.1 `openspec archive ci-headless-and-speedup` and re-run `openspec validate --all --strict`
- [x] 6.2 Commit scoped to this change with `TASK-ALLOWS-DOCS` (docs were updated)
