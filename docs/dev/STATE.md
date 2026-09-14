# STATE — project resume anchor

Snapshot for resuming after a context break. Update after each milestone.

## Where things are

- Repo: `github.com/Zawaro/kooka-pictura`, branch `main`. Docs-only corpus +
  a working Rust/Qt engine.
- Toolchain: Rust 1.98 (`rust-toolchain.toml`), system Qt **6.11.1**, cxx-qt
  **0.10.0**, wgpu **30.0.1**, lcms2 **6.2.0** (system Little CMS 2.19).
- Oracles installed for tests: `psd-tools` 1.19, ImageMagick 7.1.2, `magick`.
- Test suite: **270 tests, 0 ignored** (one pre-existing app `#[ignore]`).
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
| `pictura-filters` | blur/sharpen/noise + stylize/other filters (`Filter` + `apply`); seeded Add Noise |
| `pictura-select` | selection coverage mask, boolean/modify ops, wand, color range |
| `pictura-render` | CPU compositor (27 blend modes, groups, masks, adjustment layers) + GPU compositor + PSD adjustment encode/decode + `apply_filter` (layer filter gated by mask) |
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
- **M6** — `pictura-filters`: blur (Gaussian/Box/Motion/Radial/Average/
  Blur(+More)/Surface), sharpen (Sharpen(+More)/Edges/Unsharp Mask), noise
  (Add Noise seeded/Median/Despeckle). ImageMagick oracle: Gaussian/Box/Median
  exact, USM ±6; Motion classified no-equivalent (IM kernel is one-sided). 47
  filter tests. OpenSpec change `m6-filters`, tasks checked.
- **M6-C** — filter integration: `pictura-render::apply_filter(layer, filter,
  mask)` (destructive, selection/mask-confined, alpha + layer meta preserved) and
  the app `apply_filter(kind)` command + dock control. Headless self-test proves
  confinement (`changed_inside=12 changed_outside=0`). OpenSpec change
  `m6-filter-integration`, tasks checked.
- **M7** — Stylize + Other filters: `Maximum`, `Minimum`, `Offset`, `High Pass`,
  `Custom` (Other); `Emboss`, `Find Edges`, `Solarize` (Stylize). ImageMagick
  oracle exact (Δ0) for Maximum/Minimum/Offset-wrap/Custom/Solarize; Emboss,
  Find Edges, High Pass, Offset-fill classified no-equivalent. App filter kinds
  added. OpenSpec change `m7-stylize-other`, tasks checked.

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

## Next: M8 (propose via OpenSpec first)

M7 (Stylize + Other filters) is complete. Candidate next areas: the remaining
filter families (Distort, Pixelate, Render, Liquify, Blur Gallery, Camera Raw,
Lens Correction) or a different spec area (`docs/07-color-painting` brush/paint
engine, `docs/03-tools` painting tools, `docs/09-automation`).

Process: every new milestone is proposed through OpenSpec first
(`openspec/changes/<name>`, new capabilities), validated, then implemented.
The M6/M6-C/M7 changes are not archived yet — `openspec archive <name>` promotes
their deltas into `openspec/specs/`.

## Known risks / open items

- Core API is frozen only where noted; adding fields breaks struct literals.
- PSD descriptor coverage is partial (adjustment layers, layer styles not yet).
- GPU is non-authoritative; non-separable blend modes and Dissolve are CPU-only.
- `pictura-app` has one `#[ignore]`d interop test.
