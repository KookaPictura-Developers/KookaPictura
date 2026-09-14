# Image Modes

- **Spec ID:** `IMG-004`
- **Status:** `Draft`
- **Parity tier:** `Core`
- **New in CS6:** `No` — the eight color modes and the `Image > Mode` submenu are the same set as CS5 (Bitmap, Grayscale, Duotone, Indexed Color, RGB, CMYK, Lab, Multichannel); CS6 does not add a color mode.
- **Depends on:** `IMG-005` (bit depth), `01-architecture/document-model.md` (`ARCH-002`), `01-architecture/color-management.md`, `01-architecture/file-formats.md`, `01-architecture/undo-history.md`, `02-ui-ux/menus.md`, `07-color-painting/color-models.md`.

## CS6 behavior

`Image > Mode` changes the color model of the document. The eight CS6 modes,
with their documented character (CS6 Help, "Color modes"):

| Mode | Channels | Bit depth | Character |
|---|---|---|---|
| **Bitmap** | 1 | 1 bpc | Two values only, black or white. Called a bitmapped 1-bit image. |
| **Grayscale** | 1 | 8/16/32 bpc | Shades of gray; 8-bit gives up to 256 values; uses the working-space range. |
| **Duotone** | 1 (8-bit grayscale data) | 8 bpc only | Monotone/duotone/tritone/quadtone using one to four custom inks; printable tonality. |
| **Indexed Color** | 1 | 8 bpc | Up to 256 colors via a color lookup table (CLUT); limited editing. |
| **RGB Color** | 3 (R,G,B) | 8/16/32 bpc | Default mode for new images; monitor model; 16.7 M colors at 8 bpc. |
| **CMYK Color** | 4 (C,M,Y,K) | 8/16 bpc | Process-ink percentages; for print color separation. |
| **Lab Color** | 3 (L,a,b) | 8/16 bpc | Device-independent; used as the color-management reference. L 0–100, a/b +127…−128. |
| **Multichannel** | 2–56 | 8/16 bpc | 256 gray levels per channel; specialized printing; no layers. |

Documented mode facts:

- The default channel count per mode: Bitmap/Grayscale/Duotone/Indexed = 1,
  RGB/Lab = 3, CMYK = 4. Alpha and spot channels can be added, but not to
  Bitmap mode.
- The Help lists Lab save formats (PS, PSB, PDF, Raw, TIFF, DCS 1.0/2.0) and
  index-color save formats (BMP, DICOM, GIF, EPS, PSB, PCX, PDF, Raw, PNG,
  Targa, TIFF, and more). See `01-architecture/file-formats.md`.
- **Duotone** is treated as a single-channel, 8-bit grayscale image; individual
  channels are manipulated through the Duotone Options curves, not directly.
- **Multichannel** guidelines (documented): layers are unsupported and flattened;
  original color channels become spot channels; CMYK → C,M,Y,K spot channels;
  RGB → C,M,Y spot channels; deleting a channel from RGB/CMYK/Lab automatically
  converts to Multichannel and flattens layers; export via DCS 2.0. **Indexed
  Color and 32-bit images cannot be converted to Multichannel.**
- Converting changes color values permanently. RGB → CMYK adjusts out-of-gamut
  values into the CMYK gamut; converting back does not restore them. Flattening
  before conversion is recommended because layer blend interactions change with
  mode.
- `Image > Mode > 8 Bits/Channel`, `16 Bits/Channel`, and `32 Bits/Channel`
  set the depth (see `IMG-005`).
- Modes not available for the active image appear **dimmed** in the submenu.

### What conversion loses

- **Color → Grayscale**: hue and saturation are discarded; adjacent colors can
  collapse to the same gray. (A Black & White adjustment layer preserves color
  information but is not a mode conversion.)
- **Color → CMYK**: out-of-gamut RGB colors are clipped/shifted; round-tripping
  back to RGB does not recover them.
- **Color → Indexed**: all but ≤256 colors are discarded; layers are flattened,
  hidden layers discarded.
- **Color → Bitmap**: reduced to two values (1 bpc); requires an 8-bit grayscale
  source.
- **Color → Multichannel**: layers flattened; channels become spot channels.
- **32 → 8/16 bit**: HDR dynamic range is compressed through tone mapping (see
  `IMG-005`).
- Any conversion is one-way for data that the target mode cannot represent.

### Conversion options (dialogs)

- **Bitmap** (`Image > Mode > Bitmap`, from 8-bit grayscale): Output resolution
  (input and output default to the current resolution) plus a conversion `Use`
  method: **50% Threshold** (above mid-gray 128 → white, below → black),
  **Pattern Dither** (geometric black/white dot configurations), **Diffusion
  Dither** (Floyd–Steinberg-style error diffusion from the upper-left, grainy
  film-like result), **Halftone Screen** (Frequency in lpi/lcm, Angle −180°…+180°,
  and Shape), and **Custom Pattern** (a tiled grayscale pattern).
- **Grayscale from Bitmap** (`Image > Mode > Grayscale`): a **size ratio** 1–16
  scales down the image, averaging multiple 1-bit pixels into one gray pixel.
- **Indexed Color** (`Image > Mode > Indexed Color`, from 8-bit Grayscale or
  RGB): Palette Type — Exact, System (Mac/Windows), Web (216), Uniform
  (8/27/64/125/216), Local (Perceptual/Selective/Adaptive), Master
  (Perceptual/Selective/Adaptive), Custom, Previous; **Colors** up to 256;
  **Forced** color inclusion (Black and White, Primaries, Web, Custom);
  **Transparency** and **Matte**; **Dithering** (None, Diffusion with Preserve
  Exact Colors, Pattern, Noise) with an amount percentage.
- **Duotone** (`Image > Mode > Duotone`, from 8-bit grayscale): Type
  (Monotone/Duotone/Tritone/Quadtone), an ink color per plate chosen from a color
  library, a per-ink curve (up to 13 points; horizontal highlight→shadow, ink
  density vertical), overprint colors, and Save/Load of settings.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| `Image > Mode > Bitmap` | Menu item | — | from 8-bit grayscale; opens Bitmap dialog |
| `Image > Mode > Grayscale` | Menu item | — | RGB→Gray prompts Discard |
| `Image > Mode > Duotone` | Menu item | — | 8-bit grayscale only; opens Duotone Options |
| `Image > Mode > Indexed Color` | Menu item | — | 8-bit Grayscale/RGB; opens Indexed Color dialog |
| `Image > Mode > RGB Color` | Menu item | — | |
| `Image > Mode > CMYK Color` | Menu item | — | may prompt about profile |
| `Image > Mode > Lab Color` | Menu item | — | |
| `Image > Mode > Multichannel` | Menu item | — | flattens; not from Indexed/32-bit |
| `Image > Mode > 8/16/32 Bits/Channel` | Menu item | — | depth, see `IMG-005` |
| Mode submenu | Unavailable modes | — | shown dimmed |
| Indexed Color dialog | Preview | — | preview conversion |
| Duotone Options | Preview | — | preview conversion |

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Target mode | Enum | current | the 8 modes | unavailable targets dimmed |
| Bitmap `Output` resolution | Number + unit | current | > 0 | input resolution shown |
| Bitmap `Use` | Enum | 50% Threshold (inferred) | 50% Threshold, Pattern Dither, Diffusion Dither, Halftone Screen, Custom Pattern | |
| Halftone `Frequency` | Number | — | 1.000–999.999 lpi; 0.400–400.00 lpc | decimals allowed |
| Halftone `Angle` | Number | 45° (common) | −180 … +180 | |
| Halftone `Shape` | Enum | Round (inferred) | dot shapes | |
| Custom Pattern | Pattern | — | tiled grayscale pattern | must be image-sized to avoid tiling |
| Grayscale `size ratio` (from Bitmap) | Integer | 1 | 1–16 | averages pixels on scale-down |
| Indexed `Palette` | Enum | Local (Perceptual) (inferred) | Exact, System (Mac/Win), Web, Uniform, Local×3, Master×3, Custom, Previous | |
| Indexed `Colors` | Integer | 256 | 1–256 | for Uniform/Perceptual/Selective/Adaptive |
| Indexed `Forced` | Enum | None | None, Black and White, Primaries, Web, Custom | |
| Indexed `Transparency` | Bool | On (inferred) | — | adds a transparent index entry |
| Indexed `Matte` | Color / None | None | color picker, None | blends anti-aliased edges |
| Indexed `Dither` | Enum | Diffusion (inferred) | None, Diffusion, Pattern, Noise | |
| Indexed `Dither amount` | Percent | 75% (inferred) | 0–100 | |
| Duotone `Type` | Enum | Monotone | Monotone, Duotone, Tritone, Quadtone | |
| Duotone ink color | Color / library | Black (Monotone) | any color library | descending order for saturation |
| Duotone curve | Curve | diagonal | up to 13 points | per ink |

## Algorithms & pipeline

- **Mode = color model + channel set + depth + capability flags.** Conversion is
  a pipeline through a color-managed reference; where the target cannot represent
  source values, data is discarded or clipped. The documented rule is that
  conversions are permanent for unrepresentable data.
- **RGB ↔ CMYK / Lab**: ICC-profile transforms using the document working spaces
  (`01-architecture/color-management.md`); rendering intent/black-point
  compensation come from Color Settings. Use lcms2 in the proposal.
- **RGB → Grayscale**: documented as discarding hue/saturation; the exact
  luminance weights are not stated in the Help and are marked inferred
  (standard Rec.601/Rec.709-style weighted sum).
- **RGB/Grayscale → Indexed**: build a CLUT (median-cut or octree for
  Perceptual/Selective/Adaptive; fixed tables for System/Web/Uniform), map each
  pixel to the nearest entry, then dither. Error diffusion is the standard
  Floyd–Steinberg variant; Pattern/Noise are structured variants.
- **Grayscale → Bitmap**: threshold at 128, or dither/error-diffuse, or halftone
  screen (a rotated periodic dot screen per frequency/angle/shape), or a custom
  tiled pattern. The halftone screen becomes part of the image (can moiré when
  re-screened).
- **Grayscale → Duotone**: map gray levels through each ink's transfer curve to
  ink percentages; overprint colors model ink-on-ink appearance.
- **Multichannel**: split the source channels into independent spot channels.
- Where Adobe's exact algorithm is closed (luminance weights, dither kernels,
  halftone spot generation), the spec targets **behavioral parity only**.

### Conversion matrix

Rows = source mode, columns = target mode. Legend: **Y** documented allowed;
**(y)** inferred allowed; **✖** documented blocked; **V** allowed via an
intermediate mode (documented workflow); **?** unknown. The exact CS6 menu
enablement per cell is not published and must be verified (see Open questions).

| from \ to | Bitmap | Gray | Duotone | Indexed | RGB | CMYK | Lab | Multi |
|---|---|---|---|---|---|---|---|---|
| **Bitmap** | — | Y | (y) | (y) | V | V | V | (y) |
| **Grayscale** | Y | — | Y | Y | Y | Y | Y | (y) |
| **Duotone** | (y) | (y) | — | (y) | (y) | (y) | (y) | Y |
| **Indexed** | V | (y) | (y) | — | (y) | (y) | (y) | ✖ |
| **RGB** | V | Y | V | Y | — | Y | Y | Y |
| **CMYK** | V | Y | V | V | Y | — | Y | Y |
| **Lab** | V | Y | V | V | Y | Y | — | Y |
| **Multichannel** | (y) | (y) | (y) | (y) | (y) | (y) | (y) | — |

Documented constraints that drive the matrix: Bitmap requires an 8-bit grayscale
source; Duotone requires 8-bit grayscale; Indexed requires 8-bpc Grayscale or
RGB; Indexed and 32-bit cannot become Multichannel; converting to
Multichannel/Bitmap/Indexed flattens layers; deleting a channel from RGB/CMYK/Lab
converts to Multichannel.

## Rust module mapping

Proposed (names provisional), following `ARCH-002`:

- `pictura-core::color::ColorMode` — enum `Bitmap | Grayscale | Duotone |
  Indexed | Rgb | Cmyk | Lab | Multichannel`, plus a `capabilities()` method
  returning max depth, whether layers are allowed, and whether extra channels
  are allowed.
- `pictura-core::ops::convert::convert(doc, target: ColorMode, opts) -> Command`
  — dispatch per source/target pair; produces the new pixel buffers.
- `pictura-core::ops::convert::BitmapOptions { output_ppi, method: BitmapMethod }`;
  `IndexedOptions { palette, colors, forced, transparency, matte, dither, amount }`;
  `DuotoneOptions { kind, inks: Vec<Ink { color, curve }>, overprint }`.
- `pictura-core::color::palette::{median_cut, octree, uniform, web, system}` and
  `pictura-core::color::dither::{floyd_steinberg, pattern, noise}`.
- `pictura-core::color::icc` — lcms2-backed transforms for RGB↔CMYK/Lab.
- Boundary types: `ColorMode` as an FFI-safe enum; option structs serialized as
  plain fields for the Qt dialog.

## Qt6 component mapping

- `ImageModeMenu` — dynamic `QMenu` for `Image > Mode`; items are checked/enabled
  from `ColorMode::capabilities()`; emits `modeChangeRequested`.
- `BitmapConversionDialog`, `IndexedColorDialog`, `DuotoneOptionsDialog` — modal
  `QDialog`s with the option sets above; `IndexedColorDialog` embeds a color-table
  preview and a `PalettePreview` widget.
- `DuotoneCurveEditor` (`QWidget`) — 13-point curve per ink; reusable by Curves
  (`IMG-0xx`).
- `ColorModeBadge` / status-bar readout — shows the active mode.
- Widgets over QML: modal dialogs with dense validation and a color table favor
  Widgets; the palette grid could be a `QAbstractTableModel` + `QTableView`.

## Data-model impact

- Document field `color_mode` and a per-channel buffer model; changing mode
  rebuilds the channel set. Layer model must reject layers in modes that forbid
  them (Bitmap, Indexed, Multichannel) — flatten on conversion and surface the
  alert.
- Pixel storage varies: 1-bit packed for Bitmap, u8/u16/f32 per channel otherwise;
  Indexed stores u8 indices plus a CLUT (`Vec<Rgba8>` or mode-specific).
- Serialization: PSD stores mode (and indexed palette) in the header; PSB for
  large documents. Duotone is serialized as grayscale plus the duotone resource;
  exact key names TBD (`01-architecture/file-formats.md`).
- Undo: one history state per conversion. Conversion is destructive, so undo must
  retain the full prior channel data (large); snapshots or scratch-disk staging
  are the memory strategy (`01-architecture/undo-history.md`).
- Color-management state (assigned profile, working space) is touched by
  RGB↔CMYK/Lab conversion.

## Edge cases

- **Bitmap**: 1 bpc; cannot hold layers, filters, or adjustments; cannot add
  channels; conversion requires 8-bit grayscale upstream.
- **Duotone**: 8-bit grayscale only; 16/32-bit must be reduced first; curves are
  per ink; converting to Multichannel turns curves into spot channels.
- **Indexed**: 8-bpc Grayscale/RGB only; flattens and discards hidden layers;
  conversion to Multichannel is blocked.
- **Multichannel**: no layers; deleting a channel from RGB/CMYK/Lab auto-converts
  and flattens; Indexed and 32-bit are blocked.
- **32-bit**: only RGB and Grayscale; cannot become Indexed, Multichannel,
  Bitmap, Duotone, CMYK, or Lab without first going to 16/8-bit.
- **16-bit**: Grayscale/RGB/CMYK/Lab/Multichannel; conversion to Bitmap/Indexed/
  Duotone requires 8-bit.
- **CMYK gamut**: RGB→CMYK clips out-of-gamut colors; round-trip is lossy
  (documented).
- **Empty/1-px documents**: conversion must handle 0-color (Indexed with 1 entry),
  empty bitmaps, and zero-size channel sets without panic.
- **Large documents**: Bitmap/Indexed can shrink file size dramatically; PSB
  limits apply.
- **GPU-unavailable**: conversion runs on CPU; ICC transforms may be CPU-bound.
- **Undo/redo**: destructive — full before-image retention; converting twice must
  not accumulate mode errors.

## Parity acceptance criteria

1. Given an 8-bit RGB image, `Image > Mode > Grayscale` discards color and the
   undo restores the exact original RGB data.
2. Given an 8-bit RGB image and target Grayscale, the submenu shows unavailable
   targets dimmed (e.g. Indexed shown but Duotone dimmed).
3. Given an 8-bit grayscale image, 50% Threshold maps values >128 to white and
   ≤128 to black exactly.
4. Given an 8-bit RGB image and Indexed with Web palette and Diffusion dither,
   the result contains no color outside the 216-color web palette.
5. Given an 8-bit grayscale image, `Image > Mode > Duotone` with a monotone ink
   and a diagonal curve leaves pixel ink percentages unchanged.
6. Given a layered RGB document, conversion to Indexed or Multichannel flattens
   the layers and only one history state is added.
7. Given an Indexed or 32-bit document, Multichannel is dimmed/unavailable.
8. Given RGB→CMYK then CMYK→RGB of an in-gamut color, the color returns within
   the tolerance of the chosen ICC rendering intent; an out-of-gamut color does
   not round-trip.
9. Given a 16-bit grayscale document, Duotone is unavailable until converted to
   8-bit Grayscale.

## Sources

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` —
  official CS6 Help reference: "Color modes" (all eight modes, channel counts,
  Lab ranges and save formats, grayscale/bitmap/duotone/indexed/multichannel
  descriptions, Multichannel conversion guidelines), "Converting between color
  modes" (permanent data change, gamut clipping, flatten recommendation, dimmed
  unavailable modes, flattening on Multichannel/Bitmap/Indexed), "Convert an
  image to Bitmap mode" (output resolution, all five `Use` methods, Halftone
  Screen ranges), "Convert a color photo to Grayscale mode" (Discard, Black &
  White note), "Convert a Bitmap mode image to Grayscale mode" (size ratio 1–16),
  "Convert a grayscale or RGB image to indexed color" and "Conversion options for
  indexed-color images" (palette types, Colors, Forced, Transparency/Matte,
  Dithering), "About duotones", "Convert an image to duotone", "Modify the
  duotone curve", "Specifying overprint colors", "Saving and loading duotone
  settings"; "Color channels" (default channel counts).
- `https://ulearn.tech/photoshop-colour-modes` — community summary confirming
  layer flattening on Multichannel/Bitmap, the "Don't Flatten" alert option for
  other conversions, and 8/16/32-bpc per-channel semantics.

## Open questions

- **Exact per-cell enablement of the CS6 `Image > Mode` submenu** (particularly
  Bitmap↔Duotone, Indexed↔Duotone/Bitmap, Multichannel→Duotone). *Resolves with:*
  CS6 UI observation of the menu in each mode.
- **RGB→Grayscale luminance weights** used by Photoshop (Rec.601 vs Rec.709 vs
  working-space Y). *Resolves with:* CS6 output comparison on colored patches.
- **Exact perceptual/selective/adaptive palette algorithms** (median cut vs
  octree vs proprietary). *Resolves with:* publicly documenting CS6 indexed
  output, or the Adobe SDK.
- **Default selected option in each conversion dialog** (Bitmap method, Indexed
  palette/dither amounts, Duotone type). *Resolves with:* clean CS6 screenshots.
- **Whether the RGB→CMYK profile prompt appears and its default** (working-space
  vs embedded profile). *Resolves with:* CS6 Color Settings observation.
- **PSD serialization of duotone inks/curves and the indexed palette key names.**
  *Resolves with:* the Adobe PSD file-format specification plus a CS6-saved
  sample.
- **Behavior of conversions on Smart Objects and artboards** (flatten vs
  rasterize). *Resolves with:* CS6 observation.
- **Halftone screen shape catalog and default angle.** *Resolves with:* CS6 UI
  and output sample.
