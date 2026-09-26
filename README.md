# Kooka Pictura

Kooka Pictura is an independent, from-scratch layered image editor for Linux,
targeting compatibility with **Adobe Photoshop CS6 (v13, 2012)** workflows.
The engine is Rust; the UI is Qt 6.

This is a working application, not just a spec. It opens and saves PSD/PSB,
composites layers on the CPU with an optional GPU path, and ships a large slice
of the CS6 tool, layer, adjustment, and filter surface.

## Layout

```
crates/
  core codec color adjust filters ops paint select   engine: spec math, no Qt
  pictura-render/   CPU compositing + GPU (wgpu/Vulkan) compute
  pictura-testkit/  golden-image compare / hashing
  pictura-app/      the only Qt crate: Rust cxx-qt bridge + C++/Qt shell
docs/               long-form behavioral contract (spec corpus)
openspec/           change proposals (changes/) + capability specs (specs/)
scripts/            verification gates and Python oracle tools
```

The app is a Rust `staticlib` (`pictura_app`, built by Corrosion from Cargo)
linked into the C++ `pictura` binary. cxx-qt codegen is driven by
`crates/pictura-app/build.rs` and `src/cxxqt_object.rs`. New `.cpp`/`.h` files
must be listed in `CMakeLists.txt`. The engine crates stay Qt-free; only
`pictura-app` touches Qt.

## Build

Requirements:

- Rust 1.98 (pinned by `rust-toolchain.toml`)
- Qt 6 with Core, Gui, Widgets, Svg, and Network (`qt6-base-dev`, and the
  matching `qmake6`)
- CMake >= 3.24, Ninja, and `lld`

```bash
cmake -S . -B build -G Ninja -DCMAKE_EXE_LINKER_FLAGS=-fuse-ld=lld
cmake --build build --parallel
./build/pictura
```

## Test and verify

Rust tests use std `#[test]` only (no framework); nextest runs them.

```bash
cargo fmt --all                                  # CI runs --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace                    # one crate/test: -p <crate> [name]
cargo test --workspace --doc                     # doctests; nextest skips them

bash scripts/verify-fast.sh                      # fmt, clippy, test-report, guards
bash scripts/verify-full.sh                      # CMake build first, then verify-fast
openspec validate --all --strict
```

Some oracle suites self-skip when `magick` / `psd-tools` are absent; profiling
and GPU tests are `#[ignore]`d and run with `--ignored --nocapture`.

## Headless

```bash
./build/pictura --headless --self-test [file.psd]
```

`--headless` selects the offscreen QPA plugin and implies `--self-test` when no
document is given. The C++ self-test is a hand-rolled sequential oracle in
`crates/pictura-app/cpp/selftest*.cpp`; it emits one
`pictura self-test: PASS|SKIP|FAIL <suite> <name>` token per check to stderr and
uses the exit code as the failure identity.

## Docs

- [`docs/dev/STATE.md`](docs/dev/STATE.md) — resume anchor: where things are.
- [`docs/dev/testing-conventions.md`](docs/dev/testing-conventions.md) — how
  every test layer is built and reports.
- [`docs/README.md`](docs/README.md) — how to read the spec corpus.
- [`openspec/`](openspec/) — per-change requirements and task lists.

## Legal

Kooka Pictura is an independent project. It is not affiliated with,
endorsed, sponsored, or approved by Adobe Inc. **Adobe**, **Photoshop**,
**Camera Raw**, **Adobe Camera Raw**, and **Lightroom** are trademarks or
registered trademarks of Adobe Inc. Adobe marks are used here only nominatively
to identify the Photoshop CS6 behavior being reimplemented and the documented
on-disk PSD/PSB identifiers required for compatibility. The project ships no
Adobe source, binaries, fonts, profiles, or creative assets. See
[`NOTICE.md`](NOTICE.md) and
[`docs/00-overview/licensing-and-provenance.md`](docs/00-overview/licensing-and-provenance.md).

## License

Kooka Pictura is free software under the **GNU GPL v3.0 or later** (see
[`LICENSE`](LICENSE)). Third-party component licenses are listed in
[`THIRD-PARTY-LICENSES`](THIRD-PARTY-LICENSES) with texts under
[`LICENSES/`](LICENSES/).
