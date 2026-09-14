# Bit Depth and Conversion

- **Spec ID:** `IMG-005`
- **Status:** `Draft`
- **Parity tier:** `Core`
- **New in CS6:** `Changed` — the depth set (8/16/32 bpc) is unchanged, but CS6 expands the HDR workflow (HDR Pro presets) and reads more bit depths in TIFF files; the CS6 what's-new lists HDR Pro presets and TIFF bit-depth reads.
- **Depends on:** `IMG-004` (image modes), `IMG-001` (Image Size), `01-architecture/document-model.md` (`ARCH-002`), `01-architecture/color-management.md`, `01-architecture/file-formats.md`, `01-architecture/undo-history.md`.

## CS6 behavior

Bit depth is the amount of color information per pixel and is measured **per
channel** (bits-per-channel, bpc). Source: CS6 Help, "Bit depth" and "Support
for 16-bit images", "High dynamic range images".

- **8 bpc** — each channel has 2⁸ = 256 values. A Grayscale 8-bit image has 256
  grays; an RGB 8-bit image has 256 values per channel, i.e. over 16 million
  colors. RGB 8-bpc is often called a 24-bit image (8 × 3 channels).
- **16 bpc** — each channel has 2¹⁶ = 65,536 values. RGB 16-bpc is called a
  48-bit image; RGB 32-bpc is called a 96-bit image.
- **32 bpc** — floating-point, also known as **high dynamic range (HDR)**. The
  Help states luminance values are stored in a floating-point representation 32
  bits long, directly related to the amount of light in a scene, unlike the
  non-floating-point 16- and 8-bpc images.
- **Bitmap mode** is 1 bpc (black/white only) — a special case, not a general
  depth.
- The depth is set or changed with `Image > Mode > 8 Bits/Channel`,
  `16 Bits/Channel`, or `32 Bits/Channel`.
- **8 ↔ 16**: choose `Image > Mode > 16 Bits/Channel` or `8 Bits/Channel`.
- **8 or 16 → 32**: choose `Image > Mode > 32 Bits/Channel`.
- **32 → 16 or 8**: choose `Image > Mode > 16 Bits/Channel` or `8 Bits/Channel`;
  a tone-mapping dialog (HDR Conversion) appears (see Algorithms).

### Where each depth is supported

- **8 bpc** — all modes: Bitmap (as 1-bit), Grayscale, Duotone, Indexed, RGB,
  CMYK, Lab, Multichannel.
- **16 bpc** — Grayscale, RGB, CMYK, Lab, and Multichannel (documented support
  list). Not Bitmap, Duotone, or Indexed (those are 8-bit or 1-bit by
  definition).
- **32 bpc** — RGB Color and Grayscale only; conversion to/from 8 or 16
  Bits/Channel is supported. 32-bit cannot be Indexed, Multichannel, Bitmap,
  Duotone, CMYK, or Lab.

### Feature availability by depth

**16-bpc support (documented):**

- Working in Grayscale, RGB, CMYK, Lab, and Multichannel modes.
- All toolbox tools **except the Art History Brush**.
- All color and tonal adjustment commands **except Variations**.
- Layers, including adjustment layers.
- Some filters, including Liquify.

**32-bpc HDR support (documented feature list):**

- Adjustments: Levels, Exposure, Hue/Saturation, Channel Mixer, Photo Filter.
- Blend modes: Normal, Dissolve, Darken, Multiply, Lighten, Darker Color,
  Linear Dodge (Add), Lighter Color, Difference, Subtract, Divide, Hue,
  Saturation, Color, Luminosity.
- New 32-bpc documents: 32 bit is an option in the New dialog bit-depth menu.
- Edit menu: all commands (Fill, Stroke, Free Transform, Transform).
- File formats: PSD, PSB, Radiance (HDR), Portable Bit Map (PBM), OpenEXR, TIFF
  (LogLuv TIFF readable but not writable).
- Filters: Average/Box/Gaussian/Motion/Radial/Shape/Surface Blur, Add Noise,
  Clouds, Difference Clouds, Lens Flare, Smart Sharpen, Unsharp Mask, Emboss,
  De-Interlace, NTSC Colors, High Pass, Maximum, Minimum, Offset.
- Image commands: Image Size, Canvas Size, Image Rotation, Crop, Trim, Duplicate,
  Apply Image, Calculations, Variables.
- Layers: new/duplicate layers, adjustment layers (Levels, Vibrance,
  Hue/Saturation, Channel Mixer, Photo Filter, Exposure), fill layers, masks,
  styles, supported blend modes, Smart Objects.
- Modes: RGB Color, Grayscale; conversion to 8 or 16 Bits/Channel.
- Tools excluded: Magnetic Lasso, Magic Wand, Spot Healing Brush, Healing Brush,
  Red Eye, Color Replacement, Art History Brush, Magic Eraser, Background Eraser,
  Paint Bucket, Dodge, Burn, Sponge.
- Selections: Invert, Modify Border, Transform Selection, Save/Load Selection.
- The HDR Color Picker and 32-bit readouts (Info panel eyedropper mode
  "32-Bit"; 32-bit preview options).

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| `Image > Mode > 8 Bits/Channel` | Menu item | — | |
| `Image > Mode > 16 Bits/Channel` | Menu item | — | |
| `Image > Mode > 32 Bits/Channel` | Menu item | — | creates HDR; available from 8/16 |
| HDR Conversion dialog | Dialog | — | on 32→16/8; tone mapping methods |
| `View > 32-Bit Preview Options` | Menu / dialog | — | Exposure and Gamma / Highlight Compression |
| Document status bar | `32-Bit Exposure` | — | per-view white point |
| Info panel | Eyedropper readout | — | 8-bit / 16-bit / 32-bit modes |
| New dialog | Bit depth menu | — | 1/8/16/32 (mode-dependent) |
| HDR Color Picker | Color picker | — | Intensity slider, 32-bit float values |

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Bit depth | Enum | 8 (inferred) | 1 (Bitmap), 8, 16, 32 | valid set depends on mode |
| 16-bit channel values | Integer | — | 0 … 65,535 | per channel |
| 8-bit channel values | Integer | — | 0 … 255 | per channel |
| 32-bit channel values | Float | — | full float range | HDR; may exceed [0,1] |
| 32-bit preview `Method` | Enum | Exposure and Gamma (inferred) | Exposure and Gamma, Highlight Compression | View > 32-Bit Preview Options |
| Preview `Exposure` | Number | 0 | EV / gain | |
| Preview `Gamma` | Number | 1.0 | contrast | |
| 32→16/8 tone mapping | Enum | Local Adaptation (inferred) | Local Adaptation, Equalize Histogram, Exposure and Gamma, Highlight Compression | HDR Conversion dialog |
| Local Adaptation params | — | — | Edge Glow Radius, Strength, Gamma, Exposure, Detail, Shadow, Highlight, Vibrance, Saturation, Toning Curve | documented |
| Exposure (Adjustments) | Slider | 0 | — | designed for 32-bpc |

## Algorithms & pipeline

- **Depth conversion is per-channel quantization.** 16→8 maps each 16-bit value
  to 8 bits, discarding tonal resolution (destructive for the finer gradations);
  8→16 up-converts by scaling (fills the new range) but does not recover data.
  Marked inferred as the obvious arithmetic; the Help documents only the commands
  and that converting is how you take advantage of features.
- **8/16 → 32**: values are promoted to floating point. The Help notes 32-bpc is
  the only format that can store full HDR range; the exact scaling for the up
  promotion is inferred.
- **32 → 16 or 8**: an HDR Conversion tone-mapping dialog appears with these
  documented methods:
  - **Local Adaptation** — adjusts local brightness regions; Edge Glow Radius,
    Strength, Tone and Detail (Gamma, Exposure in f-stops, Detail, Shadow,
    Highlight), Color (Vibrance, Saturation), and a Toning Curve over the
    luminance histogram.
  - **Equalize Histogram** — compresses dynamic range while preserving contrast;
    automatic.
  - **Exposure and Gamma** — manual gain (Exposure) and contrast (Gamma).
  - **Highlight Compression** — compresses highlights into the 8/16 range;
    automatic.
- **32-bpc preview** (`View > 32-Bit Preview Options`, stored in PSD/PSB/TIFF)
  uses Exposure and Gamma or Highlight Compression; `32-Bit Exposure` in the
  status bar sets a per-view white point. Preview adjustments do not edit the HDR
  data.
- **Color management**: depth conversion respects the document working space;
  RGB↔CMYK/Lab profile transforms are separate from depth. See
  `01-architecture/color-management.md`.
- Where Adobe's exact quantization/tone-mapping curves are closed, target
  **behavioral parity only**.

### Depth × mode support matrix

Legend: **Y** supported; **—** not applicable (mode is fixed at another depth);
**✖** unavailable.

| Mode \ depth | 1 bpc | 8 bpc | 16 bpc | 32 bpc |
|---|---|---|---|---|
| Bitmap | Y | — | — | — |
| Grayscale | — | Y | Y | Y |
| Duotone | — | Y | — | — |
| Indexed | — | Y | — | — |
| RGB | — | Y | Y | Y |
| CMYK | — | Y | Y | ✖ |
| Lab | — | Y | Y | ✖ |
| Multichannel | — | Y | Y | ✖ |

## Rust module mapping

Proposed (names provisional), following `ARCH-002`:

- `pictura-core::color::BitDepth` — enum `U1 | U8 | U16 | F32`; methods
  `max_value()`, `is_float()`, `byte_size()`.
- `pictura-core::pixels::ChannelBuffer` — depth-generic buffer (packed 1-bit,
  `Vec<u8>`, `Vec<u16>`, `Vec<f32>`) with a deinterleaved per-channel layout.
- `pictura-core::ops::bitdepth::{to_8, to_16, to_32}` — quantization/promotion
  kernels with a documented rounding policy.
- `pictura-core::ops::hdr::HdrConversionSpec` — tone-mapping method + parameters
  (`LocalAdaptation`, `EqualizeHistogram`, `ExposureGamma`, `HighlightCompression`).
- `pictura-core::ops::hdr::tonemap(hdr: &ChannelBuffer<f32>, spec) -> ChannelBuffer<u16|u8>`.
- `pictura-core::color::hdr_preview` — preview transform (exposure/gamma,
  highlight compression) kept separate from stored data.
- Boundary types: depth enum, tone-map parameters as plain floats for the Qt
  dialog.

## Qt6 component mapping

- `BitDepthMenu` — checked items in `Image > Mode`, enabled from
  `ColorMode` × `BitDepth` capabilities.
- `HdrConversionDialog` (`QDialog`) — method combo plus a `QStackedWidget` of
  parameter pages (Local Adaptation curve/toning, Exposure/Gamma sliders); live
  preview.
- `HdrPreviewOptionsDialog` and `DocumentStatusBar` 32-bit exposure slider.
- `InfoReadoutMode` — selects 8/16/32-bit values in the Info panel
  (`02-ui-ux/panels/info-panel.md`).
- `HdrColorPicker` — Intensity slider and float readout, an extension of the
  standard color picker.
- Widgets over QML: the tone-mapping dialog needs a live preview hook and
  native sliders; Widgets with a GPU preview surface.

## Data-model impact

- `ChannelBuffer`'s element type and `BitDepth` are document/state fields; a
  depth change reallocates every layer's buffers and may double/quadruple memory.
- Feature-capability gating: filters/tools/adjustments query `BitDepth` to
  enable/disable (16-bpc and 32-bpc lists above).
- Modes constrain depth: Bitmap fixed at U1, Duotone/Indexed at U8, 32-bit only
  RGB/Grayscale.
- Serialization: PSD/PSB depth field in the header; TIFF/PSD/PSB/OpenEXR/HDR
  support 32-bpc per the file list. Exact per-format depth handling in
  `01-architecture/file-formats.md`.
- Undo: a depth change is destructive and large; undo must retain the prior-depth
  buffers (snapshot/scratch-disk).

## Edge cases

- **32→8/16 loses HDR range** by design; the tone-mapping dialog is mandatory.
- **16→8 is destructive** for tonal data; no dialog (inferred from documented
  command behavior).
- **Up-conversion does not recover** clipped/discarded values.
- **Bitmap (1 bpc)**: cannot add channels; converting to it requires 8-bit
  grayscale; converting from it to Grayscale offers a size ratio.
- **Duotone/Indexed are 8-bit only**: 16/32 must be reduced to 8-bit first.
- **CMYK/Lab at 32-bit**: unsupported; convert mode or depth.
- **32-bpc preview settings** are stored only in PSD/PSB/TIFF per the Help; other
  formats drop them.
- **Info panel / eyedropper** must switch to 32-bit readout to show HDR values;
  otherwise values may look clipped.
- **Huge (PSB) 32-bit documents**: memory is 4× the 8-bit size per channel;
  scratch-disk fallback required.
- **GPU-unavailable**: tone mapping on CPU (can be slow for Local Adaptation).
- **Undo/redo**: depth changes restore exact prior buffers; converting twice must
  not accumulate bias.

## Parity acceptance criteria

1. Given an 8-bit grayscale image, `Image > Mode > 16 Bits/Channel` doubles the
   buffer to u16 and leaves visible values unchanged (scaled); undo restores the
   exact u8 data.
2. Given a 16-bit image with values 0x0100 and 0x01FF, converting to 8-bit
   produces quantized values that do not distinguish the pair (rounding per the
   chosen policy).
3. Given an 8-bit or 16-bit document, `Image > Mode > 32 Bits/Channel` yields an
   HDR-capable float document (write OpenEXR/TIFF succeeds).
4. Given a 32-bit document, `Image > Mode > 16 Bits/Channel` opens the HDR
   Conversion dialog with all four documented methods.
5. Given 32→16 with Highlight Compression, no highlight value clips above the
   16-bit maximum.
6. Given a 32-bit document, the submenu dims CMYK, Lab, Indexed, Duotone, Bitmap,
   and Multichannel.
7. Given a 16-bit document, Variations is unavailable and the Art History Brush
   is unavailable; all other tools and adjustments remain enabled.
8. Given a 32-bit document, only the documented tools/adjustments/filters are
   enabled; excluded tools are disabled.
9. Given 32-Bit Preview Options changes, the preview changes without altering
   pixel data; reopening a PSD/PSB/TIFF restores the stored preview settings.

## Sources

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` —
  official CS6 Help reference: "Bit depth" (definitions, 24/48/96-bit, HDR),
  "Photoshop support for 16-bit images" (modes, tools except Art History Brush,
  adjustments except Variations, layers, filters), "Convert between bit depths"
  (8/16/32 commands), "High dynamic range images" (floating-point 32-bit,
  luminance), "Features that support 32-bpc HDR images" (full tools/
  adjustments/blend-modes/filters/image-commands/layers/modes/tools list),
  "Convert from 32 bits to 16 or 8 bpc", "Options for 16- or 8-bit images"
  (Local Adaptation, Equalize Histogram, Exposure and Gamma, Highlight
  Compression), "Options for 32-bit images", "Adjust displayed dynamic range for
  32-bit HDR images" (View > 32-Bit Preview Options, status-bar 32-Bit Exposure,
  Info 32-bit readout), "About the HDR Color Picker", the CS6 what's-new "File
  formats — Reads more bit depths in TIFF files" and "New HDR Pro presets".
- `https://ulearn.tech/photoshop-colour-modes` — community: 8-bit = 256 values
  per channel, 16-bit = 65,536 per channel, posterization rationale.

## Open questions

- **Exact 16→8 and 8→16 quantization policy** (round vs truncate, dithering on
  down-conversion, gamma consideration). *Resolves with:* CS6 output value
  comparison on gradient ramps.
- **Exact tone-mapping curves** for Local Adaptation, Equalize Histogram, and
  Highlight Compression. *Resolves with:* CS6 output comparison or Camera Raw/
  HDR Pro documentation.
- **Default tone-mapping method and parameter values** in the HDR Conversion
  dialog. *Resolves with:* clean CS6 screenshot.
- **Default bit depth for new documents per mode** (8 assumed). *Resolves with:*
  CS6 New dialog observation.
- **Whether 16→8 offers any dialog or applies silently** (assumed silent). *Resolves
  with:* CS6 behavior test.
- **Precise per-filter 16/32-bpc support** beyond the documented lists (e.g.
  which Smart Filters work at 32-bit). *Resolves with:* CS6 or CS6 help filter
  pages per filter.
- **32-bit handling in CMYK/Lab conversions** (must mode-change first; exact
  ordering and any warning). *Resolves with:* CS6 test.
- **Storage of 32-bit preview settings in formats other than PSD/PSB/TIFF.**
  *Resolves with:* the file-format specification plus CS6 save/open tests.
