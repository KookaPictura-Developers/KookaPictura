## Why

M1 produced the layer-stack data model but nothing turned it into pixels, so layers, opacity, masks, groups, and the 27 blend modes could not be rendered, flattened, or validated against Photoshop. M2 establishes the CPU compositor as the reference that every later rendering feature is checked against; the GPU path is a separate, later accelerator that never replaces this authority.

## What Changes

- Add `crates/pictura-render` with `composite_rgba(doc) -> PixelBuffer`: bottom-to-top CPU compositing of the visible layer stack into a document-resolution 4-channel straight-alpha 8-bit buffer.
- Implement W3C source-over compositing in normalized `f32` with premultiplied intermediates; layer opacity (0..=255) and a per-layer raster mask scale the source alpha.
- Implement all 27 Photoshop layer blend modes (Normal … Luminosity) plus group `Pass Through`, per W3C *Compositing and Blending Level 1* and the community definitions for the Photoshop-only modes.
- Composite **isolated** groups onto transparency and blend the result as one layer; composite **pass-through** groups directly against the running backdrop.
- Clip layers whose `PsdRect` lies partly or wholly outside the canvas.
- Handle `Dissolve` as a deterministic per-pixel stochastic threshold.
- Add an ImageMagick `-compose` differential oracle (`scripts/im_compose.py` plus committed fixtures) for the 19 modes ImageMagick implements with the same formula, and hand-computed W3C unit tests for the other 8.

## Capabilities

### New Capabilities

- `layer-compositing`: bottom-to-top CPU compositing of the layer stack into a document-resolution 4-channel straight-alpha 8-bit buffer, covering the W3C source-over equation, layer opacity, raster masks, isolated and pass-through group semantics, and off-canvas clipping.
- `blend-modes`: the 27 Photoshop blend functions (Normal through Luminosity) plus Dissolve, implemented against W3C *Compositing and Blending Level 1* and verified against W3C known values and an ImageMagick `-compose` oracle where ImageMagick supports the mode.

### Modified Capabilities

None. This change introduces new capabilities only.

## Impact

- New crate `crates/pictura-render` (CPU path); `composite_rgba` is its public entry point. GPU acceleration lives in the separate later change `m2-gpu-compositing` and is never the reference.
- New script `scripts/im_compose.py`; new fixtures and tests under `crates/pictura-render/tests/`.
- Uses `pictura-core` (`Document`, `Layer`, `BlendMode`, `LayerMask`, `PixelBuffer`); `pictura-testkit::compare` is a dev-dependency supplying the oracle tolerance check.
- Behavioral parity is bounded by Adobe's unpublished integer rounding and Dissolve noise; non-separable out-of-gamut mapping is tolerant. See `design.md`.
