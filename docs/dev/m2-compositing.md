# M2 — Layer-Stack Compositing

Goal: turn the M1 model into pixels. Composite the layer stack — opacity,
raster masks, groups, and all 27 blend modes — into a composite image, verified
against golden images and an independent oracle.

**CPU is the authority.** The golden/differential oracle runs on CPU; a GPU
implementation is an accelerator added later (M2.5), never the reference.

## Scope

In:
- Bottom-to-top compositing of pixel layers onto the canvas.
- Source-over alpha compositing with the 27 blend functions
  (`docs/05-layers/blend-modes.md`; W3C Compositing and Blending Level 1).
- Layer opacity (0..=255) and per-layer raster mask (`LayerMask.data`, `-2`)
  applied to alpha.
- Groups: recurse; **isolated** groups composite children onto transparency then
  act as one layer. Pass-through groups composite children directly against the
  running backdrop (PSD default) — implement if it stays correct, else document.
- Layers may sit outside the canvas (negative/offset `PsdRect`); clip on composite.
- New crate `crates/pictura-render` (CPU path).

Out (later):
- GPU/WGSL compositing (M2.5), clipping masks, layer styles, adjustment layers,
  smart objects, CMYK/Lab, 16/32-bit compositing, color management.

## Contract (M2-A authoritative)

```rust
// crates/pictura-render
/// Composite the layer stack into 4-channel (R,G,B,A) planar, **straight alpha**,
/// 8-bit, at document resolution. Unpremultiplied output.
pub fn composite_rgba(doc: &pictura_core::Document) -> pictura_core::PixelBuffer;
```

- Blend math in normalized `f32` (`0.0..=1.0`); convert to 8-bit at the end.
- The W3C source-over form:
  `Co = (1-αb)·αs·Cs + αs·αb·B(Cb,Cs) + (1-αs)·αb·Cb`, `αo = αs + αb·(1-αs)`
  (evaluated with straight alpha and premultiplied intermediates).
- A layer with no `-1` channel is opaque inside its bounds, transparent outside.
- Missing/empty mask = fully opaque.

## Task DAG

| ID | Task | Owner | Owns | Acceptance |
|---|---|---|---|---|
| M2-A | CPU compositor + per-mode tests | agent | `crates/pictura-render` | `composite_rgba` composes pixels; 27 blend unit tests vs W3C values |
| M2-B | ImageMagick oracle + goldens | agent | `scripts/**`, `crates/pictura-render/tests/**`, fixtures | Same layer scenes composited by `magick -compose` match within tolerance |
| M2-C | Integrate + review | orchestrator | — | workspace green; goldens + oracle pass |

## Oracle

ImageMagick (`magick`/`convert`) has `-compose {multiply,screen,overlay,darken,
lighten,color-burn,color-dodge,soft-light,hard-light,difference,exclusion,hue,
saturation,color,luminize,…}` and `-composite`. Generate a few small RGBA layer
stacks, composite them both ways, and compare with `pictura-testkit::compare`
(tolerance: exact for 8-bit integer paths where possible, ±1 otherwise).

## Exit gate

- `cargo test --workspace` green including per-mode and golden tests.
- ImageMagick differential tests pass within tolerance for the modes IM supports;
  unsupported-by-IM modes are covered by hand-computed W3C unit tests.
- `scripts/guard.sh` green.
