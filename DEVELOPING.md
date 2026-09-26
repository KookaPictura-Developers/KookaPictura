# Developing Kooka Pictura

Build, test, and contribution mechanics. Before your first change, read
[`CONTRIBUTING.md`](CONTRIBUTING.md): the provenance and licensing rules there
are not optional.

## Prerequisites

- Rust 1.98, pinned by `rust-toolchain.toml`.
- Qt 6 with the Core, Gui, Widgets, Svg, and Network modules, plus the matching
  `qmake6` (`qt6-base-dev` on Debian/Ubuntu).
- CMake >= 3.24, Ninja, and the `lld` linker.
- Little CMS 2 development headers (`liblcms2-dev`). The C++ binary links
  `-llcms2` because a Rust `staticlib` does not propagate its native link
  dependencies.
- Optional, for the oracle tests: ImageMagick 7 (`magick`) and Python
  `psd-tools>=1.19`. Suites self-skip when these are absent.
- Optional, for the `--interop-probe` diagnostic: Vulkan development headers
  (`libvulkan-dev`). The app and its wgpu GPU path build and run without them;
  the probe compiles to a stub when `vulkan.h` is missing.

## Build and run

```bash
cmake -S . -B build -G Ninja -DCMAKE_EXE_LINKER_FLAGS=-fuse-ld=lld
cmake --build build --parallel
./build/pictura
```

The app is a Rust `staticlib` (`pictura_app`, built by Corrosion from Cargo)
linked into the C++ `pictura` binary. cxx-qt codegen is driven by
`crates/pictura-app/build.rs` and `src/cxxqt_object.rs`. The engine crates stay
Qt-free; only `pictura-app` touches Qt.

`CMakeLists.txt` lists every `.cpp`/`.h` explicitly, so a new source file must be
added there. The CMake configure also links `compile_commands.json` into the
repo root for clangd/Serena C++ navigation; if C++ cross-file references look
fuzzy, reconfigure.

## Headless

```bash
./build/pictura --headless --self-test [file.psd]
```

`--headless` selects the offscreen QPA plugin before `QApplication` starts and
implies `--self-test` when no document is given, so it never blocks. The first
check asserts the platform is `offscreen`.

The C++ self-test is a hand-rolled sequential oracle in
`crates/pictura-app/cpp/selftest*.cpp`. It emits one
`pictura self-test: PASS|SKIP|FAIL <suite> <name>` token to stderr per check,
closes with `SUMMARY passed=<n> failed=<n> skipped=<n>`, and uses the exit code
as the failure identity: `ST_FAIL(code)` returns its code, and codes are
append-only. Names carry no milestone.

`--interop-probe` requires a real platform Vulkan instance and is not
offscreen-compatible.

## Test and verify

Rust tests are std `#[test]` only (no framework); nextest runs them.

```bash
cargo fmt --all                                  # CI runs --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace                    # one crate/test: -p <crate> [name]
cargo test --workspace --doc                     # doctests; nextest skips them

bash scripts/verify-fast.sh                      # fmt, clippy, test-report, guards
bash scripts/verify-full.sh                      # CMake build first, then verify-fast
openspec validate --all --strict
```

Oracle suites self-skip when `magick` / `psd-tools` are absent. Profiling and
GPU tests are `#[ignore]`d with a reason string and run with
`--ignored --nocapture`. See
[`docs/dev/testing-conventions.md`](docs/dev/testing-conventions.md) for how
each layer is built and reports.

## Layout

```
crates/        10 engine crates + the app
  core|codec|color|adjust|filters|ops|paint|select   spec math, no Qt
  pictura-render/   CPU compositing + GPU (wgpu/Vulkan) compute
  pictura-testkit/  golden-image compare / hashing
  pictura-app/      the only Qt crate: Rust cxx-qt bridge + C++/Qt shell
docs/          spec corpus + docs/dev/ working notes
openspec/      change proposals (changes/) + capability specs (specs/)
scripts/       verification gates + Python oracle tools
```

| Crate | Responsibility |
|---|---|
| `pictura-core` | Document/Layer/Channel/Mask/BlendMode/AdjustmentData; no dependencies |
| `pictura-codec` | PSD/PSB read/write: composite and layer channels, masks, adjustment keys, document channels, image resources and metadata |
| `pictura-color` | ICC profiles (sRGB/AdobeRGB/ProPhoto), convert/assign, intents, black-point compensation |
| `pictura-adjust` | Destructive adjustments and the native-depth kernels |
| `pictura-filters` | Blur, sharpen, noise, stylize, pixelate, distort, render, and the artistic families |
| `pictura-select` | Selection coverage, boolean/modify ops, wand, color range |
| `pictura-ops` | Image resize, canvas size, rotate/flip, arbitrary rotation |
| `pictura-render` | CPU compositor, GPU compositor/filter path, document and layer ops |
| `pictura-testkit` | Golden-image compare/hash and the `pictura-diff` CLI |
| `pictura-paint` | Brush/pencil dab-splatting stroke engine |
| `pictura-app` | cxx-qt `PictureView`, Qt C++ shell, panels, tools, bridge |

## Conventions

- **Definition of done.** Non-trivial logic (a branch, loop, parser, or
  money/security path) ships with one runnable check: a unit test, a `demo()`
  self-check, or an integration test. Trivial one-liners need none.
- **File size.** Target under 800 LOC; hard cap 1200 for code and 1400 for
  tests. A file over its cap must be listed in
  `scripts/file-size-allowlist.txt`, a ceiling that only shrinks.
- **No unrequested abstractions, no new dependencies** without a stated reason.
- **Milestones (`mNN`) live only in comments, docs, and specs** — never in
  identifiers, string literals, or test names (`scripts/check-milestone-names.py`).
- **Docs are gated.** A change under `docs/` fails `scripts/guard.sh` unless the
  commit message carries `TASK-ALLOWS-DOCS` (or you run with
  `TASK_ALLOWS_DOCS=1`).
- **Spec workflow.** Per-change requirements live in `openspec/changes/<name>/`
  and are validated with `openspec validate --all --strict`; archiving merges
  them into `openspec/specs/`.
- **Never weaken a test to make CI pass.** A changed golden baseline needs an
  explicit note in the task result.

## Where to go next

- [`CONTRIBUTING.md`](CONTRIBUTING.md) — DCO sign-off, provenance, asset rules.
- [`docs/README.md`](docs/README.md) — how to read the spec corpus.
- [`ROADMAP.md`](ROADMAP.md) — what ships, what is planned, what is not.
- [`docs/dev/STATE.md`](docs/dev/STATE.md) — resume anchor: where things are.
- [`docs/dev/psd-support-roadmap.md`](docs/dev/psd-support-roadmap.md) — the
  detailed PSD/PSB gap ledger.
