## Context

M0 is the walking-skeleton milestone: it proves each layer of the stack can talk
to the next before any CS6 behavior is implemented (`docs/dev/m0-walking-skeleton.md`).
The milestone was built before OpenSpec was adopted, so this document records the
decisions that were actually taken rather than proposing new ones.

Constraints in force at the time:

- System Qt 6.11.1 (`qmake6 -query QT_VERSION`), Rust 1.98 pinned by
  `rust-toolchain.toml`, `cxx-qt` 0.10.
- Linux desktop only; CI runs on `ubuntu-latest`; no GPU guarantee, so nothing in
  M0 may depend on one.
- `docs/` is the long-form contract and is not modified by this retrospective.
- No CS6 behavior exists in M0; the only document the codec understands is a
  single composite 8-bit image.

## Goals / Non-Goals

**Goals:**

- Prove Rust ⇄ cxx-qt ⇄ Qt 6 works and that the app builds through CMake and runs
  headless.
- Prove PSD/PSB composite read/write round-trips and that malformed input fails
  with a typed error instead of a panic.
- Prove the verification path: golden-image compare/hash, a `pictura-diff` gate,
  CI format/clippy/test, and a non-goal guard.
- Leave a runnable check behind for every non-trivial piece (codec tests, testkit
  tests, `--self-test`).

**Non-Goals:**

- Layers, masks, channels, adjustments, filters, color management, selections.
- GPU compositing: the wgpu/QRhi path is deferred to M0.5. M0 must run on CPU.
- Real UI panels, menus, tools, or workspace persistence — only a window with a
  central image view.
- Any file format other than PSD/PSB, and any PSD feature beyond the composite.
- PackBits RLE *write* and PSB *write* (M0 reads both; it writes raw PSD only).

## Decisions

### Bridge: CXX-Qt 0.10, not qmetaobject-rs or manual FFI

`docs/01-architecture/rust-qt-interop.md` chooses CXX-Qt for a Widgets
application: qmetaobject-rs targets QML only and is passively maintained, and
manual FFI would require hand-writing object lifetime and signal plumbing. M0
adopts that choice and pins `cxx-qt` 0.10. The bridge lives only in
`crates/pictura-app` so it can be swapped without touching the Rust core.

### Build: CMake-first via Corrosion

`docs/01-architecture/build-and-packaging.md` proposes CMake as the top-level
build with Cargo producing a `staticlib`. M0 implements exactly that:
`cxx_qt_import_crate` imports `pictura_app`, and the C++ shell is linked into the
`pictura` executable. Because Debian ships a Qt 5 `qmake` alongside `qmake6`,
the build prefers `qmake6` and verifies the reported major version.

### Shell: Qt Widgets `QMainWindow`

`docs/01-architecture/qt6-ui-design.md` selects Widgets over Qt Quick for the
frame and panels. M0 uses a `QMainWindow` with a custom central image widget and
a dark-grey background, which is the smallest thing that satisfies "opens a
window headless" while matching the chosen technology.

### Codec: hand-written parser, error-not-panic, raw-write/RLE-read

`docs/01-architecture/file-formats.md` notes there is no mature Rust PSD
*writer*; M0 therefore hand-writes the composite reader/writer. Every length is
validated before use and all offset math is checked, so untrusted input yields
`PsdError::{BadSignature, Unsupported, Truncated, Invalid}` — never a panic. This
mirrors the `ARCH-002` unsafe/validation policy. M0 writes raw image data only
(simplest valid PSD); it reads raw and PackBits RLE, and recognizes the 8-byte
length fields and 4-byte scanline counts of PSB.

### Determinism: dependency-free FNV-1a hash

`docs/11-cross-cutting/testing-strategy.md` asks for a determinism gate before
any tolerance is applied. M0 provides `hash_bytes` (FNV-1a) with no new
dependency and hashes fixed-seed input twice in a unit test. Golden comparison is
a byte-level absolute-tolerance `compare` — enough for M0's 8-bit composite and
extendable to the metric suite (PSNR/DSSIM/ΔE2000) when real render cases land.

### Guard: shell script for non-goals

`scripts/guard.sh` encodes the M0 non-goals as a runnable gate: no `.8bf` plugin
references, no `artboard*` files under `crates/`, and no `docs/` edits without a
`TASK-ALLOWS-DOCS` marker. It is a plain Bash script so it runs locally with no
setup and in CI unchanged.

## Risks / Trade-offs

- **CXX-Qt API churn.** `cxx-qt` is early-development; pinning 0.10 and confining
  the bridge to `pictura-app` keeps an upgrade from rippling into the core.
- **Qt version skew.** The build assumes a Qt 6 `qmake` is discoverable; if the
  bridge had failed against Qt 6.11 the fallback was Qt 6.8 LTS via aqtinstall or
  `qmetaobject-rs`. It did not fail, so no fallback is implemented.
- **GPU unproven in M0.** wgpu/QRhi is deferred to M0.5; M0 is CPU-only, so later
  GPU work carries the integration risk.
- **Write is the narrow end.** M0 writes PSD version 1 with raw compression and no
  layer section; RLE write and PSB write are deferred, so files are larger and
  PSB-sized documents cannot be saved yet. `ponytail:` the writer stays minimal
  until file size or PSB output matters.
- **`compare` is byte-level.** The class metrics (PSNR, DSSIM, ΔE2000) in the
  testing strategy are not implemented; M0 only needs 8-bit absolute tolerance,
  and adding metrics later does not change the `Diff` contract.
- **Guard docs check is diff-scoped.** Without `GUARD_BASE`, the guard only sees
  uncommitted `docs/` changes, so it is a PR-time gate rather than a hard repo
  invariant.
