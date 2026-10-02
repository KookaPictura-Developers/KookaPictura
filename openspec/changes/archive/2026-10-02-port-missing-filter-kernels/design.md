# Design: port-missing-filter-kernels

## Context

`pictura_filters::Filter` is one enum in `crates/pictura-filters/src/filter/types.rs`,
dispatched by `apply` in `filter/apply.rs`. Each arm validates its parameters
first and calls a free function in a family module. The Noise, Stylize, and
Sharpen families already exist (`noise.rs`, `stylize/`, `sharpen.rs`); the app
maps filter-kind strings to `Filter` values in
`crates/pictura-app/src/cxxqt_object/helpers.rs::filter_from_kind`.

## Goals / Non-Goals

**Goals**

- Add the six documented kernels with the ranges and defaults in
  `docs/06-filters/{noise,stylize,sharpen}-filters.md`.
- Preserve the crate contract: planar 8-bit in place, channels 3 or 4, alpha
  untouched, validate before mutation, clamp only at the final store, never
  panic.
- Deterministic output for every variant, with no RNG or seed parameter.

**Non-Goals**

- Filter-menu entries, option dialogs, and combo wiring (separate follow-up).
- GPU backends; the CPU path is the oracle.
- 16/32-bit documents (the engine is 8-bit throughout).

## Decisions

**D1 — `Filter` variants and parameter enums.** Add
`DustAndScratches { radius: u32, threshold: u32 }`,
`Extrude { kind: ExtrudeType, size: u32, depth: f32, level_based: bool,
solid_front: bool, mask_incomplete: bool }`,
`Tiles { count: u32, offset: u32, fill: TileFill, foreground: [u8; 3],
background: [u8; 3] }`,
`TraceContour { level: u8, edge: ContourEdge }`,
`Wind { method: WindMethod, from_right: bool }`, and
`SmartSharpen { amount: f64, radius: f64, reduce_noise: f64,
remove: SharpenRemove, angle: f64 }`. New parameter enums
`ExtrudeType { Blocks, Pyramids }`,
`TileFill { BackgroundColor, ForegroundColor, InverseImage, UnalteredImage }`,
`ContourEdge { Lower, Upper }`, `WindMethod { Wind, Blast, Stagger }`, and
`SharpenRemove { GaussianBlur, LensBlur, MotionBlur }` live beside the other
parameter enums in `lib.rs` and are re-exported at the crate root.

**D2 — Placement.** Dust & Scratches goes in `noise.rs` next to `median`,
reusing the shared `window_median`. Extrude, Tiles, Trace Contour, and Wind
live under the `stylize/` submodule (`stylize/{extrude,tiles,trace_contour,wind}.rs`),
each a free function called by `stylize` re-exports. Smart Sharpen goes in
`sharpen.rs` beside `unsharp_mask`, reusing the shared Gaussian kernel
(`gaussian_blur_planes`/`sigma_from_radius`) and `blur::motion`; its Lens Blur
is a disc (circular) mean over a per-plane summed-area table. `blur::motion` is
corrected to Photoshop's counter-clockwise screen-angle convention (the vertical
component is negated for this y-down buffer), which also fixes the existing
Motion Blur filter's diagonal direction; a 45° unit test pins it.

**D3 — Dispatch.** Each variant gets one arm in `apply`, matching the existing
shape: call the family function, which calls `validate` first and returns
`FilterError::InvalidParams` for out-of-range values before touching pixels.

**D4 — Alpha and clamping.** Every kernel writes channels `0..min(channels, 3)`
only and copies channel 3 unchanged for RGBA. Intermediate math is widened
(`f64`/`i32`) and clamped to `0..=255` only at the final 8-bit store; no wrap.

**D5 — Determinism.** Extrude's non-level depths, Tiles offsets, and Wind streak
selection come from an integer coordinate hash (`hash(x, y)` over the cell or
pixel position). There is no RNG and no seed parameter in this change, so the
same input and parameters always produce bit-identical output.

**D6 — App mappings.** `filter_from_kind` gains six arms with the defaults in
the `filter-app-ui` delta, and the mapping guard test extends to cover them.

**D7 — Smart Sharpen More Accurate and tonal fade.** `more_accurate` selects a
second, higher-fidelity blur estimate per `remove` (a wider Gaussian support,
finer disc row sampling, and sub-pixel motion taps) instead of the default
path; the default path is untouched, so
the Gaussian/Unsharp-Mask equivalence still holds only with
`more_accurate: false`. `shadow` and `highlight` are
`TonalFade { amount: u8, width: u8, radius: u32 }`
(`default { amount: 0, width: 50, radius: 1 }`) that scale the sharpening
contribution inside the shadow (or highlight) tonal band — a per-pixel gate over
the band, in the spirit of Blend If. Both the More Accurate estimate and the
tonal gate are *(inferred)*/approximate: Adobe's blur estimate and gate formula
are closed, so they are behavioral-parity choices pinned by property tests, not a
pixel oracle. `amount`/`width` validate `0..=100` and `radius` validates
`1..=100`, rejected before mutation.

## Risks / Trade-offs

- [Level-based Extrude height depends on the luminance helper] -> the shared
  `luma` path is reused; a test pins the bright-cell-over-dark-neighbour
  ordering rather than absolute heights.
- [Lens Blur and Motion Blur PSFs are closed] -> implement the disc and
  directional blurs directly and mark the divergence with a `ponytail:` marker.
- [Tiles Inverse/Unaltered fills must read pre-filter pixels] -> snapshot the
  source plane before writing, since both fills need it.
- [Dust & Scratches Threshold 0 smooths the whole image] -> documented
  behavior; a test pins the gate and the uniform no-op.

## Open Questions

None blocking. Defaults not stated in the CS6 Help are carried from the
documented Photoshop dialog defaults and pinned in the `filter-app-ui`
requirement.
