# STATE — project resume anchor

Snapshot for resuming after a context break. Update after each milestone.

## Where things are

- Repo: `github.com/Zawaro/kooka-pictura`, branch `main`. Docs-only corpus +
  a working Rust/Qt engine.
- Toolchain: Rust 1.98 (`rust-toolchain.toml`), system Qt **6.11.1**, cxx-qt
  **0.10.0**, wgpu **30.0.1**, lcms2 **6.2.0** (system Little CMS 2.19).
- Oracles installed for tests: `psd-tools` 1.19, ImageMagick 7.1.2, `magick`.
- Test suite: **~177 tests, 0 ignored** (one pre-existing app `#[ignore]`).
- OpenSpec **1.3.1** (`/usr/bin/openspec`). M0–M5 documented retroactively as
  ten changes in `openspec/changes/` (17 capability specs, all `validate --strict` green).

## Commands

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cmake -S . -B build && cmake --build build
xvfb-run -a ./build/pictura --self-test
xvfb-run -a ./build/pictura --self-test crates/pictura-codec/tests/fixtures/two_layers.psd
bash scripts/guard.sh
openspec validate --all --strict
```

## Crates

| Crate | Responsibility |
|---|---|
| `pictura-core` | Document/Layer/Channel/Mask/BlendMode(27+pass)/AdjustmentData; no deps |
| `pictura-codec` | PSD/PSB read/write: composite, layers, masks, adjustment keys, document channels |
| `pictura-color` | ICC profiles (sRGB/AdobeRGB/ProPhoto), convert/assign, intents, BPC |
| `pictura-adjust` | 15 destructive adjustments (`apply`) |
| `pictura-select` | selection coverage mask, boolean/modify ops, wand, color range |
| `pictura-render` | CPU compositor (27 blend modes, groups, masks, adjustment layers) + GPU compositor + PSD adjustment encode/decode |
| `pictura-testkit` | golden compare/hash + `pictura-diff` CLI |
| `pictura-app` | cxx-qt `PictureView` QObject + Qt C++ shell (layer/adjustment dock, zoom/pan, GPU demo) |

## Milestones done

- **M0/M0.5** — cxx-qt↔Qt6 walking skeleton, PSD codec, harness, CI, GPU spike.
  GPU: offscreen wgpu→readback→QImage works; QRhi imports the wgpu VkDevice/image;
  on-screen present via `QRhiWidget` is blocked (`crates/pictura-app/GPU-INTEROP-NOTES.md`).
- **M1** — layer/channel/mask model + layered PSD round-trip (+psd-tools oracle).
- **M2/M2.5** — CPU compositor (27 modes) + ImageMagick oracle; GPU compositor
  (22 modes, Δ0); app renders the layer stack.
- **M3** — ICC color management (+ ImageMagick oracle).
- **M4 (A/B/C)** — 15 adjustments; adjustment layers (decoded: `nvrt`,`post`,
  `thrs`,`brit`,`levl`,`hue2`; others preserved-only); app adjustment UI.
- **M5 (A/B/C)** — selection math (+ IM morphology oracle); selection↔PSD
  channels; selection-masked adjustments in the app.

## Spec workflow (OpenSpec)

OpenSpec is the per-change requirements layer over `docs/`. See `AGENTS.md`
"Spec workflow (OpenSpec)". Retrospective M0–M5 changes live in
`openspec/changes/`; each has `proposal.md`, `design.md`, `tasks.md`, and
`specs/<capability>/spec.md` deltas. They are **unarchived**: `openspec archive
<name>` promotes deltas into `openspec/specs/<capability>/spec.md`. New work
(e.g. M6) starts as a new change, not as code.

## Conventions (keep doing)

- Task briefs live in `docs/dev/m*-*.md`; docs changes need a commit message
  containing `TASK-ALLOWS-DOCS` or `TASK_ALLOWS_DOCS=1` for `guard.sh`.
- Each milestone: freeze interfaces → dispatch 2–3 `general` sub-agents on
  **disjoint crates** → orchestrator integrates, un-ignores oracle tests,
  verifies, commits. Never let an implementer verify its own work without an
  independent oracle (psd-tools / ImageMagick).
- Oracles: don't fake tolerances. Where ImageMagick/Photoshop semantics diverge,
  reclassify as "no faithful equivalent" and use property/known-value tests.

## Next: M6 — filters

Plan: blur / sharpen / noise families as pure functions over `PixelBuffer`
(planar 8-bit), then compositor/app integration. Oracle: ImageMagick
(`-blur`/`-gaussian-blur`/`-sharpen`/`-median`/`-noise`) where semantics match;
W3C/standard kernels for the rest. Spec: `docs/06-filters/*.md`.

Process: propose M6 through OpenSpec first (`openspec/changes/m6-*`, new
capabilities such as `blur-filters`/`sharpen-filters`/`noise-filters`), validate,
then implement. Do not skip the proposal like M0–M5 did.

## Known risks / open items

- Core API is frozen only where noted; adding fields breaks struct literals.
- PSD descriptor coverage is partial (adjustment layers, layer styles not yet).
- GPU is non-authoritative; non-separable blend modes and Dissolve are CPU-only.
- `pictura-app` has one `#[ignore]`d interop test.
