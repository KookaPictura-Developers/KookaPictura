# M15 — Render Filters (Clouds, Difference Clouds, Fibers, Lens Flare)

## Why

`Filter > Render` is the last filter family in `docs/06-filters/` with zero
implementation, while blur/sharpen/noise (M6), stylize/other (M7), pixelate
(M8), and distort (M9/M11) are all shipped. `FILT-060` scopes five CS6
commands; four of them — Clouds, Difference Clouds, Fibers, Lens Flare — are
plain destructive filters that fit the established `pictura-filters` pipeline
exactly. (Lighting Effects is a GPU workspace and Scripted Patterns is a Fill
dialog feature; both are separate future milestones, not deferred scope here.)

## What Changes

- New `render` module in `pictura-filters` with four filters implementing the
  existing `Filter` enum + `apply` dispatch:
  - `Clouds` — seeded fractal value-noise field interpolated between two
    colors, replacing the layer's RGB (`starker` mirrors Photoshop's
    Alt/Option variant).
  - `DifferenceClouds` — the same noise blended with existing RGB via the
    Difference formula; repeated runs accumulate marble patterning.
  - `Fibers` — seeded directional noise elongated along one axis;
    `variance` controls color variation/streak length, `strength` the weave.
  - `LensFlare` — deterministic additive light pass (bright core, ghost
    reflections per lens type, starburst rays); `brightness` 10–300%, unit
    `center` coordinates, four `LensType` values from the CS6 AppleScript
    reference.
- All four follow crate conventions: planar 8-bit in place, alpha untouched,
  `FilterError::InvalidParams` on bad input, seeds via `ChaCha8Rng`.
- App integration: `filter_from_kind` mappings (`clouds`,
  `difference-clouds`, `fibers`, `lens-flare`) with fixed default parameters,
  dock combo entries, and self-test coverage.
- Parity classification: all four are **no-equivalent** (Adobe's noise and
  flare models are closed; no ImageMagick operator generates fg/bg-colored
  fractal clouds or lens flares). Verified by property tests (spec'd
  behaviors, seeded determinism, alpha preservation), not by delta fitting —
  recorded in `docs/dev/m15-render-filters.md`.

## Capabilities

### New Capabilities

- `render-filters`: the four Render filters (Clouds, Difference Clouds,
  Fibers, Lens Flare) — parameter contracts, seeded determinism, replacement
  and additive semantics, alpha preservation.

### Modified Capabilities

## Impact

- `crates/pictura-filters/src/render.rs` — new module; `Filter` enum and
  `apply` dispatch extended; no new dependencies (`rand_chacha` already
  present).
- `crates/pictura-app/src/cxxqt_object.rs` — `filter_from_kind` mappings.
- `crates/pictura-app/cpp/main.cpp` — combo entries + self-test block.
- `pictura-render::apply_filter` needs no change (generic layer-filter path).
- Spec consumed: `docs/06-filters/render-filters.md` (`FILT-060`); Lighting
  Effects and Scripted Patterns explicitly out of scope.
