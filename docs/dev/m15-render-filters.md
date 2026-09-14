# M15 — Render filters (Clouds, Difference Clouds, Fibers, Lens Flare)

Goal: the last unimplemented simple filter family from `docs/06-filters/`
(`FILT-060`). Lighting Effects (GPU workspace) and Scripted Patterns (Fill
dialog) are separate future milestones, not deferred scope. OpenSpec change:
`m15-render-filters` (new capability `render-filters`).

## What landed

- `crates/pictura-filters/src/render.rs` — four filters over the standard
  planar buffer (alpha untouched, `InvalidParams` before mutation):
  - Shared seeded value-noise helper: ChaCha8Rng lattice table, smoothstep
    bilinear, 5 octaves, lacunarity 2, gain 0.5, normalized [0,1].
    `ponytail:` markers name the closed-Adobe-noise divergence and the
    gradient-noise upgrade path.
  - `clouds` — replaces RGB with the field mapped between `color_a`/`color_b`;
    `starker` = smoothstep contrast curve (the Alt/Option variant).
  - `difference_clouds` — same field, per-channel `|existing − field|`
    (Difference formula); repeated runs accumulate marble patterning.
  - `fibers` — x-elongated field; y-frequency scales with `strength`, x-scale
    + per-row jitter with `variance` (validated 0..=100 / 1..=100; jitter
    alone would cancel out of horizontal-difference probes, so x-frequency
    participates too — deviation from the first brief, noted in tests).
  - `lens_flare` — deterministic additive pass (core gaussian falloff, ghost
    chain per `LensType { Zoom, Prime35, Prime105, MoviePrime }`, starburst
    spokes), unit center clamped, brightness validated 10..=300, clamp 255.
    `ponytail:` marker: artistic amplitudes, bounding-box upgrade path.
- `Filter` enum + `apply` dispatch extended; `LensType` re-exported at crate
  root. No new dependencies.
- App: `filter_from_kind` mappings (`clouds`, `difference-clouds`, `fibers`,
  `lens-flare`) with fixed defaults (black/white, starker false, seed 1,
  variance 16, strength 4, brightness 100, center (0.5, 0.5), Zoom), combo
  entries, and the per-mapping guard test extended.
- Self-test (exit code 24, runs on the reopened fixture after the M14
  checks): `apply_filter("clouds")` must change pixels only inside the blue
  quadrant rect (4,4)-(8,8) — 16 pixels — leaving the red quadrant and the
  transparent quadrants bit-identical; a second seeded application is
  bit-identical; `lens-flare` applies and stays confined.

## Parity classification

All four are **no-equivalent**: Adobe's noise functions and flare model are
closed, and no ImageMagick operator generates fg/bg-colored fractal clouds or
lens flares (`plasma:fractal` is a different generator, not a semantic
match). Per the repo convention (M8/M9), verification is property tests
pinning the spec'd behaviors — seeded determinism, endpoint color ranges,
alpha preservation, Difference formula, parameter validation, brightness
monotonicity, per-lens geometry — not delta fitting against a
non-corresponding operator. 13 new tests in `render.rs`.

## Verification

- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets
  -- -D warnings` clean; `cargo test --workspace` 401 passed / 0 failed
  (13 engine tests + 1 extended mapping guard).
- `cmake --build build` green (one integration fix: the M15 self-test counters
  collided with the M6-C block's `insideChanged`/`outsideChanged` in the same
  scope — renamed to `cloudsInside`/`cloudsOutside`);
  `xvfb-run -a ./build/pictura --self-test
  crates/pictura-codec/tests/fixtures/two_layers.psd` exits 0 with
  `clouds=1 confined=1 inside=16 reapply_ident=1 flare=1` logged.
- `openspec validate --all --strict` green; `scripts/guard.sh` OK.

## Deferred

- Lighting Effects (GPU workspace, 17 presets, bump maps) and Scripted
  Patterns (fill path) — their own milestones per `FILT-060`.
- 16/32-bit gates (the engine is 8-bit throughout today).
