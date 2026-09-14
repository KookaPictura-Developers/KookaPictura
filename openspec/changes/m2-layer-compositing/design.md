## Context

M1 landed the document model (`Document`, `Layer`, `LayerMask`, `BlendMode`, `PixelBuffer`) but no pixels. M2 turns the layer stack into a composited image. Adobe's compositing algorithms are closed, so the strategy is a deterministic, independently verifiable CPU reference first: the golden and differential oracle run on the CPU, and any GPU path is a later accelerator (`m2-gpu-compositing`) that is checked against this reference, never the reverse.

The behavior contract is `docs/05-layers/blend-modes.md` (W3C *Compositing and Blending Level 1* plus community formulas for the Photoshop-only modes), with `layers-overview.md`, `layer-groups.md`, and `layer-masks.md` covering the stack, group, and mask semantics. `docs/dev/m2-compositing.md` scopes M2-A (compositor + mode tests) and M2-B (ImageMagick oracle + goldens).

## Goals / Non-Goals

**Goals:**

- Deterministic CPU compositor into a document-resolution 4-channel straight-alpha 8-bit buffer (`composite_rgba`).
- W3C source-over in normalized `f32` with premultiplied intermediates, plus all 27 Photoshop blend functions and Dissolve.
- Layer opacity and raster masks applied to source alpha.
- Isolated and pass-through group semantics, including off-canvas clipping.
- An independent ImageMagick `-compose` oracle for the modes ImageMagick implements with the same formula.

**Non-Goals:**

- GPU/WGSL compositing (separate change `m2-gpu-compositing`).
- Clipping masks, vector masks, layer styles, knockouts, Blend If, and fill opacity.
- CMYK/Lab, 16/32-bit compositing, and color management.
- Broad adjustment-layer support (owned by M4; a decode subset is co-located in the crate but is not part of these capabilities).
- Bit-exact Adobe parity, which is impossible against unpublished rounding and Dissolve noise.

## Decisions

1. **Straight alpha with premultiplied intermediates in `f32`.** Matches the W3C equation directly and allows one rounding at 8-bit write-back. Alternatives considered: `u8` fixed-point (no headroom, per-operation rounding drift) and premultiplied storage (complicates mask/opacity reasoning and output format).
2. **Single full-document `Vec<Px>` accumulator.** The simplest correct structure for M2. It is marked with a `ponytail:` ceiling comment; tiled/streaming compositing is the upgrade path when PSB-size documents no longer fit.
3. **Bottom-to-top walk in array order, recursing into groups.** Directly mirrors `layers-overview.md`; group handling is a recursion over the same routine.
4. **Group isolation boundary.** A `PassThrough` group is a true pass-through only when opacity is 255 and it has no mask; otherwise it composites as isolated with `PassThrough` treated as `Normal`. Rationale: real Photoshop applies group opacity/mask to the pass-through result while still letting children see the parent backdrop, but that mixed model is undocumented, and no CS6 baseline exists to verify it. The approximation is recorded in the crate docs and covered by a dedicated test.
5. **Dissolve as a deterministic splitmix hash.** Adobe's noise tile is unpublished, so a fixed per-pixel hash provides the observable contract (binary output, opacity-proportional coverage, stable across re-renders) without claiming bit parity. Skipping Dissolve would violate the 27-mode contract.
6. **ImageMagick as the differential oracle, within its limits.** 19 modes map to a like-for-like operator and are diffed with tolerance 0 (1 for Vivid Light rounding). The remaining eight are excluded because ImageMagick uses HSL-space blending or luminance-based comparisons, and are pinned by hand-computed W3C unit tests instead.
7. **Mask semantics: multiply alpha, default opaque.** An enabled mask sample multiplies source alpha; a missing, disabled, or out-of-rect mask leaves alpha unchanged unless the mask's `default_color` says otherwise. This matches `layer-masks.md` (black hides, white reveals, gray partial).
8. **Adjustment layers stay out of scope for these capabilities.** Their decode subset lives in the same crate for M4-B, but `layer-compositing` and `blend-modes` cover only pixel/group compositing; unknown adjustment keys are a no-op, never an error.

## Risks / Trade-offs

- **Adobe's integer rounding is closed** → the oracle is tolerance-based (±1 LSB) and W3C unit tests assert the formula, not Adobe's exact bytes.
- **Pass-through with opacity/mask is approximate** → documented in crate docs and `pass_through_group_with_opacity_falls_back_to_isolated`; upgrade only with a CS6 reference composite.
- **Dissolve is not Adobe's noise tile** → parity is behavioral only; the hash is deterministic per pixel and stable.
- **Full-document `f32` memory** → potential pressure on PSB/huge documents; the `ponytail:` comment names tiled streaming as the ceiling to raise.
- **Non-separable out-of-gamut mapping** → W3C `ClipColor` is the reference and is tolerant; Adobe's gamut mapping is unpublished.
- **Oracle is ImageMagick-version-specific** → fixtures are pinned to ImageMagick 7.1.2-29 Q16-HDRI and the mapping documents each excluded operator and why.
