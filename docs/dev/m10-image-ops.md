# M10 — Image operations (resize, canvas, orientation)

Goal: add the destructive `Image > Image Size / Canvas Size / Image Rotation`
ops as pure functions in a new `pictura-ops` crate. Specs:
`docs/04-image-ops/image-size.md` (`IMG-001`), `canvas-size.md` (`IMG-002`),
`image-rotation-and-flip.md` (`IMG-003`).

## Scope

In:
- `resize` — Nearest / Bilinear / Bicubic resampling of every channel.
- `resize_canvas` — 3×3 anchor + background fill, grow or shrink.
- `rotate90_cw`, `rotate90_ccw`, `rotate180`, `flip_horizontal`,
  `flip_vertical` — exact integer index remaps, dimension swap on quarter turns.
- `rotate_arbitrary` — bilinear inverse map, expanded axis-aligned bounding box,
  background-filled corners.
- Pure functions returning a new `PixelBuffer`; the input is never mutated.

Out (later): Bicubic Smoother / Sharper / Automatic, document and layer-tree
integration, the app UI, DPI/resolution math, CMYK/Lab, 16/32-bit, Smart
Objects, straighten / Crop-tool integration.

Validation: `width`/`height` >= 1; `rotate_arbitrary` angle finite and in
`-359.99..=359.99`; otherwise `OpsError::InvalidParams`. 1×1 and 1-px inputs
must not panic. Channels 1..=4 and `data.len() == pixel_count * channels` are
required by `validate`.

## Contract

```rust
#[derive(Debug, thiserror::Error)]
pub enum OpsError {
    #[error("unsupported: {0}")]
    Unsupported(String),
    #[error("invalid parameters: {0}")]
    InvalidParams(String),
}

// src/resize.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resample { Nearest, Bilinear, Bicubic }
pub fn resize(buf: &PixelBuffer, width: u32, height: u32, resample: Resample)
    -> Result<PixelBuffer, OpsError>;

// src/canvas.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    TopLeft, Top, TopRight, Left, Center, Right, BottomLeft, Bottom, BottomRight,
}
pub fn resize_canvas(buf: &PixelBuffer, width: u32, height: u32, anchor: Anchor,
    background: [u8; 4]) -> Result<PixelBuffer, OpsError>;

// src/orient.rs
pub fn rotate90_cw(buf: &PixelBuffer) -> PixelBuffer;
pub fn rotate90_ccw(buf: &PixelBuffer) -> PixelBuffer;
pub fn rotate180(buf: &PixelBuffer) -> PixelBuffer;
pub fn flip_horizontal(buf: &PixelBuffer) -> PixelBuffer;
pub fn flip_vertical(buf: &PixelBuffer) -> PixelBuffer;
pub fn rotate_arbitrary(buf: &PixelBuffer, angle_deg: f64, background: [u8; 4])
    -> Result<PixelBuffer, OpsError>;
```

Semantics (planar 8-bit, per-channel, clamp-to-edge, no panics):
- **Nearest** — point sample; **Bilinear** — 2×2 tent filter; **Bicubic** — 4×4
  Keys/Catmull-Rom cubic. Photoshop's exact cubic coefficients and edge handling
  are unpublished (`IMG-001`), so parity is behavioral only.
- **resize_canvas** blits the source at the anchor offset and fills the new
  region with `background` (alpha 0 for transparent backgrounds); shrink crops.
- **Quarter/half turns and flips** are exact index remaps
  (`(x,y) -> (H-1-y, x)` for 90° CW); quarter turns swap W/H.
- **rotate_arbitrary** rotates about the center, grows the canvas to
  `W' = W|cos θ| + H|sin θ|`, `H' = W|sin θ| + H|cos θ|`, and bilinearly
  resamples; corners outside the source are `background`.

## Oracle

ImageMagick, on CPU, fixed inputs; measure and record which operator maps to
which variant:
- `-filter point -resize WxH` and `-filter triangle -resize WxH` and
  `-filter cubic -resize WxH` — classify against Nearest / Bilinear / Bicubic.
- `-background <c> -gravity <anchor> -extent WxH` — canvas grow/shrink.
- `-rotate 90` / `-rotate 180` / `-rotate 270` — exact; must be bit-identical.
- `-flop` / `-flip` — exact; must be bit-identical.
- `-rotate <angle>` — arbitrary; differential tolerance measured, kernel TBD.

No faithful operator for a case ⇒ record the observed delta as no-equivalent.

## Task DAG

| ID | Task | Owns |
|---|---|---|
| M10-A1 | `resize.rs` (Nearest/Bilinear/Bicubic) + tests | `crates/pictura-ops/src/resize.rs` |
| M10-A2 | `canvas.rs` (anchor + background, grow/shrink) + tests | `crates/pictura-ops/src/canvas.rs` |
| M10-A3 | `orient.rs` (5 exact remaps + arbitrary) + tests | `crates/pictura-ops/src/orient.rs` |
| M10-B | ImageMagick oracle + no-equivalent table | `crates/pictura-ops/tests/**` |
| M10-C | OpenSpec change + reconcile + verify | `openspec/**`, `docs/dev/**` |

## Exit gate

- `cargo fmt --all`, `cargo build -p pictura-ops`,
  `cargo test -p pictura-ops`, and
  `cargo clippy -p pictura-ops --all-targets -- -D warnings` all green.
- Oracle differentials within tolerance or documented no-equivalent.
- `openspec validate --all --strict` green with the new `m10-image-ops` change.
