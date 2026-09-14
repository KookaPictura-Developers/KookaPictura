## Why

Every masked edit in Photoshop — adjustment layers, fills, filters, layer
masks — is constrained by the active selection, but nothing in the crate graph
yet represents, combines, reshapes, or derives one. M5 introduces the selection
as a document-sized 8-bit coverage mask and the algebra that produces and
reshapes it, unlocking masked rendering for the layers shipped in M4.

## What Changes

- New crate `crates/pictura-select` implementing `Selection` as a
  document-sized 8-bit coverage mask (0 = outside, 255 = inside).
- Boolean algebra `combine(other, SelectOp)` with Replace / Add / Subtract /
  Intersect, plus `invert`, `none`, and `all`.
- Modify operations: `feather` (Gaussian blur), `expand`/`contract`
  (dilate/erode), `border`, and `smooth` (majority/median).
- Image-derived selection tools: `magic_wand` (flood by tolerance, contiguous
  or global), `grow`, `similar`, and `color_range` (colour + fuzziness).
- Save/load to and from an 8-bit alpha `Channel`.
- An ImageMagick differential oracle (`scripts/select_oracle.py`) for
  expand/contract/feather, with the square-versus-disk structuring-element
  divergence documented, plus property/known-value tests for the tools that
  have no faithful ImageMagick equivalent.

## Capabilities

### New Capabilities

- `selection-model`: the coverage-mask representation, the boolean combine
  algebra, invert/none/all, and the modify operations (feather, expand,
  contract, border, smooth), with the ImageMagick oracle and its documented
  divergence.
- `selection-tools`: image-derived selection (magic wand, grow, similar, colour
  range) and alpha-channel save/load.

### Modified Capabilities

- None.

## Impact

- New crate `crates/pictura-select`; depends on `pictura-core` (`Channel`,
  `PixelBuffer`) and the workspace `thiserror`. No other new dependencies.
- New oracle script `scripts/select_oracle.py`; tests under
  `crates/pictura-select/tests/`.
- Consumed by the Qt bridge in `crates/pictura-app`; the UI wiring itself is
  tracked separately in `m5-selection-integration`.
- No `docs/` changes.
