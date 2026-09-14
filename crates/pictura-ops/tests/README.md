# M10-B — ImageMagick image-ops oracle

These tests diff `pictura_ops` against **ImageMagick** for the M10 image
operations. This is a *sanity* oracle, not a parity oracle: Photoshop's exact
resample kernels and arbitrary-rotation sampler are closed, and ImageMagick
implements the same family of operations with its own edge and bounding-box
conventions.

- Oracle script: `scripts/ops_oracle.py`
- Tests: `crates/pictura-ops/tests/oracle.rs`
- Verified against: ImageMagick **7.1.2-29 Q16-HDRI** (2026-07-27).

All differential tests run by default and skip (with a message) when `magick` is
not on `PATH`. Nothing is `#[ignore]`d.

## Op → ImageMagick mapping

Test image: a deterministic planar RGB8 ramp/checker (`R = 16x`, `G = 16y`, a
4×4 block checker in `B`). Resize uses 16×16; the right-angle turns and flips use
a non-square 16×12 so a dimension swap cannot pass silently. All "max delta"
figures are absolute 8-bit sample deltas measured against this image.

| `pictura_ops` | ImageMagick | Tolerance | Measured | Notes |
|---|---|---|---|---|
| `resize` `Resample::Nearest` | `-filter point -resize WxH!` | 0 | max 0 | Point sample; exact on the 16→32 upscale. |
| `resize` `Resample::Bilinear` | `-filter triangle -resize WxH!` | 0 | max 0 | 2×2 tent; exact on 16→32. IM widens the kernel when downscaling: 16→8 measures **37** (no-equivalent at scale < 1). |
| `resize` `Resample::Bicubic` | `-filter catrom -resize WxH!` | 1 | max 1 | Keys/Catmull-Rom (`a = -0.5`). IM `-filter cubic` is a **B-spline**, not Catmull-Rom: it measures **48**, so `catrom` is the faithful operator. |
| `resize_canvas` | `-background 'rgba(r,g,b,a)' -gravity G -extent WxH` | 0 | max 0 | Exact for all nine anchors, grow 20×18 and shrink 12×10 (3-channel). |
| `rotate90_cw` | `-rotate 90` | 0 | max 0 | Exact integer remap on non-square 16×12 (dims swap). |
| `rotate90_ccw` | `-rotate 270` | 0 | max 0 | Exact integer remap. |
| `rotate180` | `-rotate 180` | 0 | max 0 | Exact integer remap. |
| `flip_horizontal` | `-flop` | 0 | max 0 | Mirror about the vertical axis. |
| `flip_vertical` | `-flip` | 0 | max 0 | Mirror about the horizontal axis. |
| `rotate_arbitrary` | `-filter triangle -background 'rgba(0,0,0,0)' -rotate A` | 8 | max 8, mean 0.6 | Bilinear. Compared on the **central 12×12** only; see the bbox divergence below. |

### Anchor → `-gravity`

| `Anchor` | `-gravity` | `Anchor` | `-gravity` | `Anchor` | `-gravity` |
|---|---|---|---|---|---|
| `TopLeft` | `northwest` | `TopCenter` | `north` | `TopRight` | `northeast` |
| `MiddleLeft` | `west` | `Center` | `center` | `MiddleRight` | `east` |
| `BottomLeft` | `southwest` | `BottomCenter` | `south` | `BottomRight` | `southeast` |

Grow and shrink are both exact for every anchor: the added region is the
`background` fill and the source is blitted at the anchor offset, which is
exactly `-extent` after `-gravity`.

### `resize`: filter choice

`Resample::Bicubic` is Catmull-Rom (`cubic()` in `resize.rs` uses `a = -0.5`),
which is ImageMagick's `catrom` filter, **not** its `cubic` filter (a B-spline).
The differential uses 2× upscaling because IM's `-resize` widens the filter
support when downscaling (anti-aliasing), while Pictura samples a fixed 2×2 /
4×4 footprint at the destination centre. Measured on 16→8: `triangle` 37,
`catrom` 44, `cubic` 76 — all no-equivalent at scale < 1.

### `rotate_arbitrary`: bbox and sampling divergence

ImageMagick's `-rotate` grows the canvas to its own bounding box, which is
larger than Pictura's `ceil(W|cos θ| + H|sin θ|)`: at 30° IM emits **24×24** while
Pictura emits **22×22** (one extra row/column per side). IM also samples the
rotated edges differently from Pictura's inverse bilinear map, so the corners
disagree with the background fill.

The differential therefore compares the **central 12×12** region, where the
rotated source is intact in both. At 30° with `-filter triangle` the measured
max delta is **8** (mean 0.6). Other angles diverge more because IM's bounding
box parity shifts the centre by half a pixel: 45° measures max **132** and is
recorded as no-equivalent alignment.

## Exact ImageMagick flags

```bash
magick -size 16x16 -depth 8 rgb:IN.rgb \
    <OPERATOR ARGS> \
    -depth 8 -write rgb:OUT.rgb -format '%wx%h' info:
```

The script prints the resulting `WxH` on stdout (resize and arbitrary rotation
change the size). It reads/writes raw 8-bit samples: interleaved `rgb`/`rgba`
by default, or channel planes with `--planar` (matching `PixelBuffer::data`).

| Op | Flags |
|---|---|
| Nearest resize | `-filter point -resize WxH!` |
| Bilinear resize | `-filter triangle -resize WxH!` |
| Bicubic resize | `-filter catrom -resize WxH!` (`cubic` is a B-spline) |
| Canvas resize | `-background 'rgba(r,g,b,a)' -gravity {northwest…southeast} -extent WxH` |
| 90° CW / CCW / 180° | `-rotate 90` / `-rotate 270` / `-rotate 180` |
| Flip horizontal / vertical | `-flop` / `-flip` |
| Arbitrary rotate | `-filter triangle -background 'rgba(0,0,0,0)' -rotate ANGLE` |

`--background` takes `r,g,b,a` with an 8-bit alpha and is formatted as an
ImageMagick `rgba()` string.

## Running

```bash
cargo test -p pictura-ops
```

Manual inspection:

```bash
python3 scripts/ops_oracle.py version
python3 scripts/ops_oracle.py apply --size 16x16 --planar \
    --op resize --width 32 --height 32 --filter catrom IN.rgb OUT.rgb
python3 scripts/ops_oracle.py apply --size 16x16 --planar \
    --op resize_canvas --width 20 --height 18 --gravity center \
    --background 10,20,30,255 IN.rgb OUT.rgb
python3 scripts/ops_oracle.py apply --size 16x12 --planar \
    --op rotate90_cw IN.rgb OUT.rgb
python3 scripts/ops_oracle.py apply --size 16x16 --planar \
    --op rotate_arbitrary --angle 30 --filter triangle \
    --background 0,0,0,0 IN.rgb OUT.rgb
python3 scripts/ops_oracle.py apply --size 16x16 --planar \
    --im-args="-negate" IN.rgb OUT.rgb
```

`MAGICK=/path/to/magick` overrides the binary; `--im-args=...` must be attached
with `=` so argparse does not read the leading `-` as another option.

## Not expressed by this oracle

- Downscaling (`scale < 1`) for Bilinear/Bicubic: IM widens the filter support,
  so it is a different (anti-aliased) operator. Nearest is exact at any scale.
- Alpha behaviour in `resize_canvas`: the differential uses 3-channel buffers
  (`resize_canvas` over a 4-channel buffer measures 17–189 because IM's
  `-extent` composites/rounds the background alpha differently). The unit tests
  in `src/canvas.rs` cover the alpha fill contract directly.
- Arbitrary-rotation edge/corner pixels: IM's bbox padding and edge sampler
  diverge from Pictura's background fill; only the central region is compared.
- 16-bit / 32-bit and non-RGB colour spaces: M10 is 8-bit per-channel.
