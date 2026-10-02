# Proposal: port-missing-filter-kernels

## Why

Six CS6 filter kernels are documented in `docs/06-filters/` but absent from
`pictura-filters`: Dust & Scratches (`FILT-030`), Extrude, Tiles, Trace Contour,
and Wind (`FILT-050`), and Smart Sharpen (`FILT-020`). photorust implements all
six; the port closes the remaining gaps in the Noise, Stylize, and Sharpen
families with no new dependency. Issue #82, tracking issue #1.

## What Changes

- `pictura-filters` reaches the documented kernels:
  - **Dust & Scratches** (`noise`): threshold-gated per-plane windowed median
    over the `(2r+1)²` neighborhood; Radius 1–16, Threshold 0–255; a sample is
    replaced only when it differs from the local median by more than Threshold.
  - **Extrude** (`stylize`): Blocks/Pyramids height-field extrusion; Size 2–255,
    Depth 1.0–255.0, per-cell level-based option, Solid Front Faces, Mask
    Incomplete Blocks; protrusion comes from the cell luma or a coordinate hash.
  - **Tiles** (`stylize`): tiled offset with Background Color/Foreground
    Color/Inverse Image/Unaltered Image gap fill; Count 1–99, Offset 1–99 %;
    offsets come from a coordinate hash.
  - **Trace Contour** (`stylize`): per-channel Lower/Upper level crossing at
    Level 0–255.
  - **Wind** (`stylize`): Wind/Blast/Stagger horizontal streaks, From the
    Right/From the Left.
  - **Smart Sharpen** (`sharpen`): sharpening against the blur the softness came
    from, with Amount 1–500 %, Radius 0.1–64, Remove = Gaussian Blur/Lens
    Blur/Motion Blur, Angle for Motion Blur (Photoshop's counter-clockwise
    screen angle), a Reduce Noise amount 0–100 (a CC-era control that CS6 lacks,
    carried from photorust), CS6's **More Accurate** higher-fidelity blur path,
    and the Advanced **Shadow/Highlight** tonal fades
    (`TonalFade { amount, width, radius }`).
- The shared `blur::motion` samples along Photoshop's counter-clockwise
  screen-angle convention (the vertical component is negated for this y-down
  buffer), so the Motion Blur filter and Smart Sharpen's Motion Blur removal both
  smear in the documented direction.
- All six extend the existing `Filter` enum and `apply` dispatch, leave alpha
  bit-identical, validate before mutating (`FilterError::InvalidParams`), clamp
  only at the final 8-bit store, and sample clamp-to-edge.
- App: six `filter_from_kind` mappings with fixed defaults, so the existing
  command path applies them.
- Verification is property tests: none of the six has a faithful ImageMagick
  operator, so behavior is pinned by coordinate-hash determinism, alpha
  preservation, parameter validation, and the documented geometry properties,
  except Smart Sharpen's Gaussian path, which reduces to Unsharp Mask.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `imaging/noise-filters`: ADD Dust & Scratches.
- `imaging/stylize-filters`: ADD Extrude, Tiles, Trace Contour, Wind.
- `imaging/sharpen-filters`: ADD Smart Sharpen, including More Accurate and
  Shadow/Highlight.
- `imaging/filter-app-ui`: MODIFIED "Filter-kind mapping and defaults" gains
  the six new kinds.

## Impact

- `crates/pictura-filters/src/filter/types.rs` (variants + parameter enums),
  `filter/apply.rs` (dispatch), `noise.rs`, `stylize.rs` (+ `stylize/` for the
  extrusion, tiling, contour, and wind renderers), `sharpen.rs`, and `blur.rs`
  (the Motion Blur angle convention).
- `crates/pictura-app/src/cxxqt_object/helpers.rs` (`filter_from_kind`).
- No new dependency.

## Out of Scope

- `Diffuse` and `Glowing Edges` (Stylize) and `Lighting Effects`, `Flame`, and
  `Picture Frame` (Render) stay unported: they are gallery/GPU/pattern effects
  outside the `FILT-020`/`FILT-030`/`FILT-050` parity tiers, or have no
  faithful kernel.
- The Filter-menu entries, per-filter option dialogs, and combo wiring remain a
  separate follow-up; this change lands the engine kernels (including Smart
  Sharpen's More Accurate and Shadow/Highlight controls) and the
  `filter_from_kind` mappings only.

## Provenance

All six are Adobe-closed algorithms: behavioral parity only, with no verified
algorithmic match. Ported from photorust
(<https://github.com/perfecto25/photorust>, GPL-3.0-or-later; DCO certified per
issue #1) with a `Source:` trailer on each ported kernel. No new dependency.
