## 1. Bridge and Build (M0-A)

- [x] 1.1 Create `crates/pictura-app` with a `staticlib` + `rlib` crate type and `cxx-qt`/`cxx-qt-build` dependencies
- [x] 1.2 Define the `#[cxx_qt::bridge]` module and Rust `PictureView` QObject in `src/cxxqt_object.rs` and `src/lib.rs`
- [x] 1.3 Add `build.rs` that drives `CxxQtBuilder` for the `Gui` module and locates `qmake6` for standalone Cargo builds
- [x] 1.4 Add the top-level `CMakeLists.txt` with C++17, `find_package(Qt6 Core Gui Widgets)`, `cxx_qt_import_crate`, and the `pictura` executable
- [x] 1.5 Implement the C++ shell in `cpp/main.cpp`: `QMainWindow` with a custom central image widget
- [x] 1.6 Add `--self-test` so the app exercises the bridge, prints a summary, and exits non-zero on a missing image

## 2. Composite PSD/PSB Codec (M0-B)

- [x] 2.1 Define `PsdError` (bad signature, unsupported, truncated, invalid) with `thiserror`
- [x] 2.2 Implement a bounds-checked big-endian `Reader` for header and section fields
- [x] 2.3 Validate the header: signature, version, 1–56 channels, non-zero dimensions, 8-bit depth, RGB/Grayscale
- [x] 2.4 Read the composite image data for raw and PackBits RLE compression, with PSB 4-byte scanline counts
- [x] 2.5 Implement `write_psd` for the composite (raw, empty layer/mask section) and verify round-trip equality
- [x] 2.6 Add unit tests: raw parse, RLE parse (PSD and PSB), round-trip, LCG property, bad signature, truncation

## 3. Verification Harness (M0-C)

- [x] 3.1 Implement `compare`/`Diff` with absolute per-sample tolerance in `pictura-testkit`
- [x] 3.2 Implement the deterministic `hash_bytes` determinism gate
- [x] 3.3 Add the `pictura-diff` CLI with `--tolerance`, metrics output, and non-zero exit on difference
- [x] 3.4 Add unit tests for tolerance boundaries, length mismatch, hash stability, and fixed-seed determinism
- [x] 3.5 Add `.github/workflows/ci.yml` running fmt, clippy (`-D warnings`), and workspace tests
- [x] 3.6 Add `scripts/guard.sh` for `.8bf`, `artboard*`, and unmarked `docs/` changes

## 4. Integration and Determinism (M0-D/M0-E)

- [x] 4.1 Wire the Qt shell to load a codec-produced PSD through the bridge
- [x] 4.2 Verify the app builds via CMake and opens a window headless (Xvfb/offscreen)
- [x] 4.3 Confirm the workspace test suite is green and two fixed-seed runs hash equal
