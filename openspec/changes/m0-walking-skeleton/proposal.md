## Why

M0 must prove the whole stack end to end — Rust ⇄ cxx-qt ⇄ Qt 6 ⇄ PSD codec ⇄
verification harness — before any CS6 feature spend. The milestone was built
before OpenSpec was adopted, so its contract has never been captured as
reviewable requirements.

## What Changes

- Retroactively record the already-built M0 walking skeleton as three new
  capabilities; no new code is introduced by this change.
- `application-shell`: a Rust-defined `QObject` exposed to C++ through cxx-qt,
  built by CMake via Corrosion, running a minimal Qt Widgets window headlessly.
- `psd-codec`: a hand-written PSD/PSB reader/writer for the single composite
  image only (8-bit RGB/Grayscale; raw and PackBits RLE), returning typed errors
  for malformed input instead of panicking.
- `verification-harness`: golden-image compare/hash, the `pictura-diff` CLI, the
  fmt/clippy/test CI workflow, and the `scripts/guard.sh` non-goal guard.
- Record M0's limits (GPU/wgpu deferred, no layers/masks/channels, no RLE write,
  PSD-only write) so later milestones can lift them explicitly.

## Capabilities

### New Capabilities

- `application-shell`: cxx-qt Rust↔Qt6 bridge plus a minimal Qt Widgets window
  shell built through CMake, runnable headless under Xvfb.
- `psd-codec`: minimal PSD/PSB container read/write for the single composite
  image (8-bit RGB/Grayscale; raw and PackBits RLE); malformed input returns an
  error, never panics.
- `verification-harness`: golden-image compare/hash plus the `pictura-diff` CLI,
  the CI workflow, and the `scripts/guard.sh` non-goal guard.

### Modified Capabilities

## Impact

- `crates/pictura-app/` — cxx-qt bridge object, `build.rs`, C++ shell; the only
  Qt-dependent crate.
- `crates/pictura-codec/` — composite PSD/PSB read/write subset.
- `crates/pictura-testkit/` — `compare`/`hash_bytes` plus the `pictura-diff` bin.
- `CMakeLists.txt` — top-level CMake/Corrosion build of the Qt executable.
- `scripts/guard.sh` and `.github/workflows/ci.yml` — non-goal guard and CI gates.
- No CS6 behavior is implemented in M0; no `docs/` file changes.
