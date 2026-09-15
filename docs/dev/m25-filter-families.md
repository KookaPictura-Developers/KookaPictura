# M25 — Brush Strokes, Sketch, Texture and Oil Paint filters

Goal: implement the four remaining CS6 filter families in `pictura-filters` —
**Brush Strokes (8), Sketch (14), Texture (6), Oil Paint (1)**, 29 filters total —
behind the shared `apply` contract. Adobe's kernels are closed, so these are
**behavioural-parity** models (as M6–M22): effect, parameter sensitivity,
determinism, alpha preservation, and no-panic edges, without an oracle.
OpenSpec change `m25-filter-families` (new capabilities `brush-stroke-filters`,
`sketch-filters`, `texture-filters`, `oil-paint-filter`).

## Scope

- Shared helpers reused from `crate::artistic::{reduce,noise,texture}`:
  posterize, edge magnitude, `clamp_u8`, value noise, surface height, emboss,
  `TextureOptions`.
- New public enums: `StrokeDirection`, `LightDirection`, `HalftoneType`,
  `GrainType`.
- 29 filters as `Filter` variants with typed parameters, validation, and seeds.
- Modules: `brush_strokes.rs`, `sketch/{mod.rs,relief.rs,paper.rs}`,
  `texture.rs`, `oil_paint.rs`.
- App `filter_from_kind` mappings and self-test coverage (exit codes 68–69).

## Out of scope (later milestones)

- Filter Gallery dialog, cumulative/reorder/hide/delete stack, thumbnails.
- Smart Filter entries and `Edit > Fade`.
- 16/32-bit and CMYK/Lab capability gating; `Load Texture` file I/O.
- GPU compute path for Oil Paint and the `GpuUnsupported` dialog; Oil Paint is a
  CPU behavioural model here (deliberate non-parity divergence, see `design.md`).

## Process

Orchestrator: brief, OpenSpec artifacts, dispatch, integration, verification,
archive, commit. Waves: (1) interface freeze in `lib.rs`; (2) family kernels in
parallel, each with tests — Brush Strokes, Sketch, Texture, Oil Paint; (3) app
mapping + self-test; (4) close-out.

## Verification

- `cargo test -p pictura-filters` and `cargo test --workspace`
- `cmake --build build`; fixture and no-argument self-tests exit 0 with new
  filter checks (exit codes 68–69)
- `cargo fmt/clippy`; `openspec validate --all --strict`; `guard.sh`
