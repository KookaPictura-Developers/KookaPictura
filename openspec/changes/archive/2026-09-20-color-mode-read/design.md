## Context

`read_psd` parses the header and gates on `depth == 8` and
`mode ∈ {Grayscale, RGB}` before it does anything else
(`crates/pictura-codec/src/read.rs:43-50`). `pictura-core` already carries the
full `ColorMode` enum with the correct channel counts
(`crates/pictura-core/src/lib.rs:11-36`), `Document.mode`, and
`Document.depth`, so the mode is already a first-class model field; the renderer
and app are effectively RGB/Grayscale today (`store_composite` special-cases
`ColorMode::Rgb`, `helpers_composite.rs:50-74`). P2 preserves the
color-mode-data section verbatim (`Document.color_mode_data`) but nothing reads
it.

`ColorMode::color_channels()` already encodes the format's mapping:
Bitmap/Grayscale/Indexed/Duotone = 1, RGB/Lab = 3, CMYK = 4, Multichannel =
header-authoritative. That table is the grounding and needs no change.

## Goals / Non-Goals

**Goals:**

- Open Bitmap, Indexed, CMYK, and Lab 8-bit PSDs, producing a renderable
  RGB/Grayscale `Document`.
- Interpret the Indexed palette so an Indexed file shows the right colors.
- Keep the data honest: the file's original mode is recorded and reported, and
  the save-as-RGB is documented rather than silent.
- Prove the palette layout, the CMYK/Lab formulas, and the Bitmap bit packing
  against an independent implementation (psd-tools) and a committed fixture.

**Non-Goals:**

- Photoshop's embedded-ICC color management. CMYK and Lab use standard,
  profile-free formulas; no pixel parity is claimed.
- Re-encoding a normalized document back to its source mode. A save writes the
  working mode.
- Multichannel and Duotone (no RGB mapping / needs the spot-ink spec).
- 16- and 32-bit samples (G4; needs a `u16` sample model).
- Editing/authoring an Indexed palette ("Image > Mode > Color Table") in this
  slice.

## Decisions

### D1. Normalize to the working mode on read (chosen save semantics)

Of the three options in the brief — (a) preserve the source mode and re-encode
on save, (b) normalize to RGB and write RGB, (c) keep the original bytes
alongside a derived RGB view — we take **(b) normalize-on-load**.

Why:

- The renderer composites RGB/Grayscale layers and the app's whole surface
  (`new_document`, `store_composite`, `Document::from_rgba`) is RGB/Grayscale.
  Keeping `mode = Cmyk` while the composite holds RGB contradicts the
  established invariant that `composite.channels == mode.color_channels()`.
- Option (a) means re-encoding on save. That is *not* lossless for CMYK/Lab
  (an inverse conversion cannot recover the original inks) and needs
  quantization for Bitmap/Indexed, so it would not actually preserve anything
  except for a never-edited file — and the engine has no reliable "never
  edited" signal.
- Option (c) needs a second buffer plus a source-mode field threaded through
  render and app, a much larger change for no consumer in this slice. The
  palette-only version (keep `color_mode_data` on an RGB document) adds a
  confusing invariant for a view ("Color Table") that does not exist yet.

The cost of (b) is explicit: **opening a non-RGB PSD and saving changes the
document's color mode to RGB/Grayscale.** That is a real, documented loss, so
the reader records the source mode and the app warns (D7). This is the smallest
change that fits the model and does not lie.

### D2. What is accepted, converted, and rejected

| Header mode | Code | Action | Depth |
|---|---|---|---|
| Bitmap | 0 | expand 1-bit → RGB (black/white) | 1 (raw/RLE); 8 also accepted as gray |
| Grayscale | 1 | unchanged | 8 |
| Indexed | 2 | palette lookup → RGB | 8 |
| RGB | 3 | unchanged | 8 |
| CMYK | 4 | profile-free formula → RGB (approx.) | 8 |
| Multichannel | 7 | `Unsupported` | — |
| Duotone | 8 | `Unsupported` (spec preserved, not interpreted) | — |
| Lab | 9 | profile-free CIELAB → sRGB (approx.) | 8 |

Depth 1 is accepted **only** for Bitmap; every other mode requires depth 8.
Depth 16/32 stays `Unsupported` for all modes. `split_planes` already rejects a
header channel count below `mode.color_channels()`; extra channels remain
alpha/spot/saved-selection planes and are never converted.

### D3. Bitmap: MSB-first 1-bit rows

Photoshop Bitmap is 1 bit per pixel. Rows are packed MSB-first and padded to a
byte boundary: row byte count is `ceil(width / 8)`, pixel `x` is bit
`0x80 >> (x % 8)`, a set bit is black (0) and a clear bit is white (255).

Grounded on **ag-psd** `decodeBitmap` (`src/helpers.ts:207`:
`const v = b & 0x80 ? 0 : 255`) and verified against **psd-tools** reading a
hand-built depth-1 file (an 8-pixel row `0xAA` reads back as luminance
`[0,255,0,255,0,255,0,255]`).

The image-data row stride changes from `width` to `ceil(width/8)` for depth 1,
in the raw path, the RLE scanline table, and `planar_len`. Compression 0 (raw)
and 1 (PackBits RLE) are supported at depth 1; ZIP/ZIP-with-prediction at depth
1 stays `Unsupported` (Photoshop writes raw/RLE for Bitmap; prediction on
1-bit rows is not defined here).

psd-tools' public writer refuses depth 1 (`PSDImage.new` allows only 8/16/32),
so the fixture is authored by psd-tools with a header mutation
(`header.depth = 1` then `ImageData.set_data`), which saves and re-reads
correctly; a hand-built RLE variant exercises the RLE row stride.

A depth-1 **layer** channel is bit-packed with the same `ceil(width / 8)` stride
(psd-tools' `_create_image` decodes every layer channel at the document depth).
`read_channel_data` therefore expands a depth-1 layer channel to an 8-bit
`width * height` plane through `bitmap_rows_to_rgb` before the layer conversion,
so the layer path agrees with the composite instead of decoding the packed bytes
as a wider 8-bit row.

### D4. Indexed: 768-byte non-interleaved palette

The color-mode-data section is exactly 768 bytes: 256 red bytes, then 256
green, then 256 blue. A pixel whose stored index is `i` maps to
`(byte[i], byte[256 + i], byte[512 + i])`. A section length other than 768 is
`PsdError::Invalid` (malformed), never a panic.

Grounded on **ag-psd** (`src/psdReader.ts:245-252`: 256 reads for `r`, then
256 for `g`, then 256 for `b`, rejecting a length != 768) and **psd-tools**
`ColorModeData.interleave` (`psd/color_mode_data.py`: `value[i], value[i+256],
value[i+512]`), and verified by authoring an Indexed fixture and reading its
RGB back.

### D5. CMYK: the profile-free integer formula

Photoshop's stored CMYK channel bytes are **inverted** relative to PIL's CMYK
convention: 0 = full ink, 255 = no ink. With stored planes `C, M, Y, K` in
`0..=255`, the working RGB is

```
r = (C * K) / 255     // integer floor
g = (M * K) / 255
b = (Y * K) / 255
```

Grounded twice:
- **ag-psd** `cmykToRgb` (`src/psdReader.ts:1031`): `dst = ((((c*k)|0)/255)|0)`,
  i.e. `floor(c*k/255)`.
- **psd-tools** inverts the stored planes (`pil_io.post_process`:
  `ImageChops.invert`) and hands them to PIL's `cmyk2rgb`, whose formula is
  `(255-C')*(255-K')/255` on the inverted `C'`; substituting `C' = 255-C`
  gives `C*K/255`. Pillow **rounds** it (a `round`-based LUT), while the engine
  floors, so a stored pixel can differ by at most 1.

Verified end to end through psd-tools: stored `(C,M,Y,K) = (128,64,32,200)`
composites to RGB `(100,50,25)`, matching `floor(C*K/255)`. The original fixture
held every plane constant, so the floor/round difference cancelled and an exact
comparison passed; the fixture now varies the planes and the oracle compares
within 1 LSB, which makes the rounding visible.

`ponytail:` this is a **profile-free approximation**. Photoshop converts
through the embedded CMYK ICC profile, so saturated colors will differ; no
parity is claimed and the tests assert only the formula and the psd-tools
agreement on the fixture values within the 1-LSB rounding ceiling.

### D6. Lab: standard CIELAB(D50) → sRGB

The three planes are `L, a, b` with the 8-bit Lab convention used by
psd-tools/Pillow: `L* = L * 100 / 255`, `a* = a - 128`, `b* = b - 128`. Convert
`Lab → XYZ(D50) → linear sRGB` with a Bradford D50→D65 adaptation, then apply
the sRGB transfer function and clamp to `0..=255`. The combined matrix is

```
[  3.1338561, -1.6168667, -0.4906146 ]
[ -0.9787684,  1.9161415,  0.0334540 ]
[  0.0719453, -0.2289914,  1.4052427 ]
```

with the usual CIELAB inverse (`t^3` above the `216/24389` break, the linear
segment below) and D50 white `(0.96422, 1.0, 0.82521)`.

Grounded on **lcms2**: Pillow 12 converts `"LAB"` through `ImageCms`
(`PIL/Image.py:1210-1223`, an lcms2 Lab identity profile → sRGB). Pillow's
`.convert("RGB")` builds that transform with **optimization enabled**, so lcms2
precomputes a color LUT and interpolates it; for the in-gamut
`(L,a,b)8 = (225,82,114)` the optimized path returns `(33,246,246)` while the
exact transform returns `(60,246,246)`. The engine implements the exact
transform and matches lcms2 with `cmsFLAGS_NOOPTIMIZE` within 1 LSB for in-gamut
colors (verified over a 60 000-sample sweep: max 1, 0 % above 1).

The Lab oracle therefore compares against the exact lcms2 transform (Pillow
`ImageCms` with `Flags.NOOPTIMIZE`), not psd-tools' optimized composite, within a
**1-LSB** tolerance. `ponytail:` the formula is an approximation of the fully
color-managed transform; the recorded ceiling is (a) a renderer that precomputes
an optimized LUT can differ by up to ~20 LSB on in-gamut colors, and (b)
out-of-gamut colors clip rather than gamut-map the way lcms2's rendering does.

### D7. Normalization, provenance, and the app notice

After conversion the document is:

- `mode` = `Rgb` for Bitmap/Indexed/CMYK/Lab; `Grayscale` stays `Grayscale`.
- `depth` = `Eight`.
- `source_mode` = `Some(header_mode)` when the header mode was normalized
  (Bitmap/Indexed/CMYK/Lab), `None` for Grayscale/RGB and for constructed
  documents. This is the only model change.
- `color_mode_data` = empty for a normalized Indexed document (the palette was
  consumed); retained verbatim for Grayscale/RGB as P2 does.
- Every layer's color planes are converted by the same function (D3–D6), so the
  layer tree the app re-composites is RGB. Non-color channels (`-1`
  transparency, `-2` mask, unmodeled) are untouched. Converting each CMYK/Lab
  layer before RGB compositing is itself an approximation (Photoshop composites
  in the source mode); it is part of the same `ponytail:` ceiling.

`PictureView` exposes `mode_notice() -> QString` ("Converted from CMYK"), and
`frame.cpp` shows it in the status bar after a successful open. `new_document`
and `decode_image` are unchanged: creation still offers only RGB/Grayscale, and
Qt raster import already builds an RGB document.

### D8. Fixtures and oracles

| Fixture | Author | Proves |
|---|---|---|
| `indexed.psd` | psd-tools (`INDEXED`, hand-set palette + index plane) | palette order and index → RGB |
| `cmyk.psd` | psd-tools (`CMYK`, varied inverted planes, one pixel layer) | the CMYK formula (within the 1-LSB rounding) and layer conversion |
| `lab.psd` | psd-tools (`LAB`, eight non-neutral in-gamut colors; layered Lab is rejected by psd-tools itself) | the Lab formula against the exact lcms2 transform within 1 |
| `bitmap.psd` | psd-tools with `header.depth = 1` mutation | 1-bit packing and depth-1 read |

The oracle re-reads the Indexed, CMYK, and Bitmap fixtures with psd-tools (its
composite/indices) and the Lab fixture with psd-tools plus lcms2's exact
transform, and compares to `read_psd` (CMYK within 1, Lab within 1 against the
exact transform — not psd-tools' optimized `.convert("RGB")`). A Rust hand-built
RLE Bitmap exercises the depth-1 RLE row stride and a hand-built layered Bitmap
exercises the depth-1 layer channel expansion (the P1 ZIP hand-built precedent).
A pure-Rust round-trip test reads
each fixture, asserts `mode`/`source_mode`, writes it, re-reads, and asserts the
normalized document is stable and the output opens in psd-tools as RGB.

## Risks / Trade-offs

- **Save is lossy in mode.** Mitigated by `source_mode` + the app notice; a
  future re-encode slice can consume `source_mode` and the (currently dropped)
  palette.
- **CMYK/Lab are approximations.** Marked `ponytail:`; no parity claim; the
  oracle checks the formula and reference agreement, not Photoshop pixels. The
  reference itself has a ceiling: psd-tools/Pillow rounds CMYK (`≤1` vs the
  engine's floor) and its default Lab `.convert("RGB")` uses an optimized lcms2
  LUT that can differ by up to ~20 LSB in-gamut, so the Lab oracle uses the
  exact lcms2 transform.
- **Per-layer conversion is not Photoshop's pipeline.** The merged composite
  from a CMYK file and the RGB re-composite from its converted layers can
  differ slightly. Bounded by the approximation and documented.
- **`source_mode` struct-literal churn.** One `Option` field, defaulted by the
  existing `Default` impls; the compiler enumerates any literal that lists all
  fields.
- **Bitmap depth 1 / palette length.** A malformed palette or a short depth-1
  channel returns a typed error, never a panic.
- **psd-tools cannot author layered Lab.** The Lab oracle uses a flat fixture;
  layer conversion is additionally covered by a synthetic-buffer unit test.

## Migration Plan

None for documents: a previously refused file now opens; an RGB/Grayscale file
is read exactly as before. No existing golden fixture changes (the new fixtures
are additive). Rolling back is reverting the commit; the `Document` gains a
field whose default is `None`, so old constructed documents are unchanged.

## Open Questions

- Whether the CMYK formula should prefer the embedded ICC profile when present
  (that is the `color-management` / P6 work, not this slice).
- Whether Photoshop ever emits depth-1 Bitmap with ZIP compression (assumed
  not; it stays `Unsupported`).
- Whether the app should show the notice as a transient status message or a
  document property; the design assumes the status bar.
