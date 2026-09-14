# Image Size

- **Spec ID:** `IMG-001`
- **Status:** `Draft`
- **Parity tier:** `Core`
- **New in CS6:** `Changed` — the interpolation menu gains **Bicubic Automatic**, which auto-selects the best resample method for the direction of the resize; the CS6 Help "What's new" list names this under Image resizing.
- **Depends on:** `IMG-005` (bit depth), `01-architecture/document-model.md` (`ARCH-002`), `01-architecture/color-management.md`, `01-architecture/undo-history.md`, `02-ui-ux/menus.md`.

## CS6 behavior

`Image > Image Size` changes the pixel dimensions, the print (document) size, or
the resolution of the whole document. The dialog has three areas and a checkbox
row. Source: CS6 Help, "Image size and resolution" / "Change pixel dimensions of
an image" / "Change the print dimensions and resolution".

- **Pixel Dimensions** — width, height, and the file size. The new pixel
  dimensions and new file size appear at the top of the dialog, with the old
  file size in parentheses. Width/height accept pixels or percent (choose the
  unit next to the field).
- **Document Size** — physical width/height and **Resolution** (ppi). Document
  size is the base size at which the image is placed into another application.
  Units for width/height include percent, inches, cm, mm, points, picas, and
  **Columns** (columns use the width and gutter from the Units & Rulers
  preferences). Resolution units include pixels/inch and pixels/cm.
- **Auto** button (in the dialog) computes a recommended resolution from a
  screen frequency: enter the output device's screen frequency and choose
  Draft (1× screen frequency, not below 72 ppi), Good (1.5×), or Best (2×).
  The screen value is used only to calculate resolution, not to set printing.
- **Checkbox row** (bottom of the dialog): **Scale Styles**, **Constrain
  Proportions**, **Resample Image**, plus the interpolation method dropdown to
  the right of **Resample Image**. Source: CS6 Help, "View the print size
  on-screen" lists Scale Styles / Constrain Proportions / Resample Image as the
  three bottom options; the file-size and dialog layout are community-confirmed.
- **Constrain Proportions** (on by default) links width and height; editing one
  updates the other. **Scale Styles** (available only when Constrain Proportions
  is selected) scales layer-style effects with the resize.
- **Resample Image** (on by default) changes the amount of image data. With it
  unchecked, pixel dimensions are locked and read-only, and editing document
  size or resolution adjusts the other value to keep the pixel count constant.
  When resampling is off, **Constrain Proportions**, **Scale Styles**, and the
  interpolation dropdown are grayed out. Source: CS6 Help plus Photoshop
  Essentials.
- **Reset**: hold `Alt`/`Option` and click **Reset** to restore the values
  initially shown in the dialog.
- The `View > Print Size` command (or Print Size in the Hand/Zoom options bar)
  redraws the image at its approximate document size; it is documented for CS5
  and CS6 and reflects the Image Size dialog's Document Size.
- For the quick current size, hovering the document window's information box
  displays file size information.

### Resampling methods

Documented in the CS6 Help, "Resampling", and selectable in the Image Size
dialog dropdown and in `Edit > Preferences > General > Image Interpolation
Methods`:

| Method | Documented character |
|---|---|
| Nearest Neighbor | Fast, less precise; replicates pixels; for non-antialiased edges; can look jagged. |
| Bilinear | Averages surrounding pixels; medium quality. |
| Bicubic | Slower, more precise; smoother tonal gradations than Nearest/Bilinear. |
| Bicubic Smoother | Bicubic-based, intended for enlarging. |
| Bicubic Sharper | Bicubic-based with enhanced sharpening, intended for reducing; preserves detail. |
| Bicubic Automatic | **New in CS6**; auto-selects the best method based on the type of resize. |

The CS6 Help lists Bicubic Automatic as a CS6 image-resizing addition. The
image-resizing dialog's dropdown lists all six methods; community sources
describe Bicubic Automatic as the CS6 default (see Open questions). Adobe's
downsampling advice is to downsample and then apply Unsharp Mask.

### Bit-depth behavior

The CS6 Help 32-bpc feature list explicitly includes `Image Size` under Image
commands, so the dialog operates on 8-, 16-, and 32-bpc documents. Resampling
of 32-bpc documents must be done in floating point to avoid clipping; exact
kernel behavior is not published.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| `Image > Image Size` | Menu / dialog | `Alt+Ctrl+I` (Win), `Option+Cmd+I` (Mac) | Shortcut per CS6 shortcut map; verify |
| Dialog | Width / Height fields (Pixel Dimensions) | — | px or percent |
| Dialog | File size readout | — | new size, old in parentheses |
| Dialog | Document Width / Height | — | includes Columns unit |
| Dialog | Resolution | — | ppi or px/cm |
| Dialog | Auto button | — | Screen frequency + Quality |
| Dialog | Scale Styles | — | enabled only with Constrain Proportions |
| Dialog | Constrain Proportions | — | on by default |
| Dialog | Resample Image | — | on by default |
| Dialog | Interpolation dropdown | — | six methods |
| Dialog | OK / Cancel / Reset | — | `Alt`/`Option` + Reset restores |
| Preferences | `Edit > Preferences > General > Image Interpolation Methods` | — | sets dialog default |
| `View > Print Size` | Menu / options bar | — | Uses Document Size |

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Width (Pixel Dimensions) | Number + unit | current | 1 … 300,000 px | PSB limit; PSD 30,000 |
| Height (Pixel Dimensions) | Number + unit | current | 1 … 300,000 px | |
| Pixel unit | Enum | pixels | pixels, percent | |
| Width / Height (Document Size) | Number + unit | current | > 0 | percent, in, cm, mm, pt, pica, columns |
| Resolution | Number + unit | current | > 0 | pixels/inch, pixels/cm |
| Scale Styles | Bool | On (inferred) | — | only with Constrain Proportions |
| Constrain Proportions | Bool | On | — | |
| Resample Image | Bool | On | — | off locks pixel dimensions |
| Interpolation | Enum | Bicubic Automatic (CS6, community) | Nearest Neighbor, Bilinear, Bicubic, Bicubic Smoother, Bicubic Sharper, Bicubic Automatic | |
| Auto `Screen` | Number + unit | — | screen frequency | used only to compute resolution |
| Auto `Quality` | Enum | — | Draft, Good, Best | 1× (min 72 ppi), 1.5×, 2× |
| Reset | Action | — | — | `Alt`/`Option` + Reset |

## Algorithms & pipeline

- **Relationship** (documented): pixel dimensions = document (output) size ×
  resolution. With resampling off, width, height, and resolution are coupled
  (change one, the other two adjust) so the pixel count is constant.
- **Resample** computes the target raster by sampling the source with the chosen
  kernel. Standard reference kernels (inferred, standard image processing):
  Nearest = point sample; Bilinear = 2×2 tent filter; Bicubic = 4×4 cubic
  convolution (e.g. Catmull-Rom / Keys cubic). Photoshop's exact cubic
  coefficients and edge/clamping behavior are not published — **behavioral
  parity only, kernel TBD**.
- **Bicubic Smoother / Sharper** are documented as Bicubic-based with a smoothing
  or sharpening character; the exact kernels are closed. Marked inferred.
- **Bicubic Automatic** selects a method from the resize direction (community:
  Sharper when downsampling, Smoother when upsampling). The CS6 Help only states
  it "auto-selects the best resample method based on the type of resize";
  the mapping is inferred. **Behavioral parity only.**
- **Scale Styles**: layer-style parameters (blur radius, offset, stroke width,
  pattern scale) are multiplied by the same W/H scale factor so effects retain
  their visual proportion. Exact scaling rules for every style parameter are not
  published (inferred).
- **Auto resolution**: resolution = screen × factor, factor 1.0/1.5/2.0 for
  Draft/Good/Best, with Draft floored at 72 ppi. This is documented.
- Pipeline: apply to the flattened composited raster plus per-layer rasters;
  resample each layer and mask; scale styles; recompose. Document boundaries and
  layer bounds scale together.

## Rust module mapping

Proposed (names provisional), following `ARCH-002`:

- `pictura-core::ops::image_size::ImageSizeSpec` — target pixel W/H, document
  W/H, resolution, unit choices, `constrain: bool`, `scale_styles: bool`,
  `resample: ResampleMethod`.
- `pictura-core::resample::ResampleMethod` — enum:
  `Nearest | Bilinear | Bicubic | BicubicSmoother | BicubicSharper | BicubicAutomatic`.
- `pictura-core::resample::resample(src: &Raster, dst_dims, method) -> Raster` —
  kernel dispatch; automatic resolves to a concrete kernel from the direction.
- `pictura-core::ops::image_size::apply(doc, spec) -> Command` — resamples all
  layer rasters/masks, optionally scales styles, updates document pixel dims,
  resolution, and physical size.
- `pictura-core::document::Resolution` — ppi f64 plus unit; `DocumentSize` in
  physical units. Boundary types cross to Qt as plain scalars (u32 pixels, f64
  physical, f64 ppi, `u32` enum discriminant).

## Qt6 component mapping

- `ImageSizeDialog` (`QDialog`) — two group boxes (Pixel Dimensions, Document
  Size), the Auto button, checkbox row, and interpolation `QComboBox`.
- `QuantitySpinBox` (`QWidget`) — number + unit combo, reused by Canvas Size
  (`IMG-002`), New Document, and Crop options.
- `AspectLinkController` (`QObject`) — enforces Constrain Proportions and the
  resample-off coupling (three-way width/height/resolution linkage).
- `AutoResolutionDialog` — Screen frequency + Quality, matches `QFormLayout`.
- Widgets over QML: the dialog is modal, keyboard-driven, and needs native
  spin-box/validation behavior.

## Data-model impact

- Document fields: pixel width/height (u32), physical width/height (f64 +
  unit), resolution (f64 + unit), optional scale factor for styles.
- Layer rasters and masks are resampled; vector/path/type geometry scales; layer
  style parameters scale when Scale Styles is on.
- Undo granularity: one history state per OK. Record shape: before/after pixel
  dims, resolution, and (for redo-free memory economy) either tile deltas or a
  marker that layer rasters were resampled. Resampling is large — a full
  before-raster copy is a memory tradeoff to document in `undo-history.md`.
- Serialization: PSD stores pixel dims and resolution; physical size derives from
  pixels ÷ resolution. XMP may mirror resolution/print dimensions.

## Edge cases

- **Resample off**: pixel dimensions read-only; Constrain Proportions, Scale
  Styles, interpolation grayed.
- **Constrain Proportions off**: independent W/H -> non-uniform scale (distortion).
- **Enlarging beyond 300,000 px**: PSB only; PSD caps at 30,000 px per dimension
  (see GLOSSARY). Over-limit input must be rejected or promoted to PSB.
- **1×1 / tiny documents**: minimum 1 px; kernels must clamp at edges.
- **16-bit and 32-bit**: 16-bpc supports the full pipeline; 32-bpc is documented
  as supported by the Image command but the resample kernel must run in float and
  not clip.
- **CMYK/Lab/multichannel**: resample operates per channel in the document's
  working space; no mode change.
- **Indexed/Bitmap/Duotone**: resampling color modes is limited; in practice
  Photoshop converts through a supported mode. Behavior for these modes is
  inferred, not documented — see Open questions.
- **Styles**: Scale Styles only affects layer styles, not smart filters or smart
  object contents.
- **Undo/redo**: must restore prior pixel dims and layer rasters exactly;
  resample twice must not accumulate error beyond one double conversion.
- **GPU-unavailable**: resample on CPU; preview may be downsampled on GPU.

## Parity acceptance criteria

1. Given an 8-bit RGB 1000×1000 document, setting Width to 50% with resample on
   yields a 500×500 raster and the file-size readout halves.
2. Given Constrain Proportions on, changing Width to 2000 sets Height to 2000
   within one pixel; with it off, Height is unchanged.
3. Given Resample Image off, changing Resolution from 72 to 216 ppi changes
   document Width/Height by ÷3 and leaves pixel dimensions and pixel values
   identical.
4. Given Scale Styles on, a layer with a 10 px drop-shadow blur at 200% resize
   has a ~20 px blur within tolerance; with it off, the blur stays 10 px.
5. Given interpolation `Nearest Neighbor`, a 2× upscale of a two-color image
   contains only the original two colors.
6. Given interpolation `Bicubic Automatic` and a downscale, the result matches
   Bicubic Sharper; on an upscale, matches Bicubic Smoother (community mapping).
7. Given Auto with Screen=150 lpi and Quality=Best, the computed resolution is
   300 ppi within rounding.
8. Given OK, exactly one history state is added; undo restores the exact prior
   raster and dimensions.

## Sources

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` —
  official CS6 Help reference: "Image size and resolution" (pixel dimensions,
  resolution, document size, file size, 300,000 px limit), "Determine a
  suggested resolution" (Auto, Draft/Good/Best), "View the print size
  on-screen | CS5 and CS6", "Resampling" (Nearest/Bilinear/Bicubic/Smoother/
  Sharper, interpolation preference), "Change pixel dimensions of an image",
  "Change the print dimensions and resolution", "Specify columns for an image",
  and the CS6 what's-new "Image resizing — Bicubic Automatic option
  auto-selects the best resample method based on the type of resize".
- `https://www.photoshopessentials.com/essentials/resizing-vs-resampling/` —
  Steve Patterson: Pixel Dimensions vs Document Size, Scale Styles / Constrain
  Proportions / Resample Image checkbox row, Resample on by default, grayed
  options when Resample is off, and the practical Sharper=smaller /
  Smoother=larger guidance.
- `https://photographyuncapped.com/what-is-the-new-bicubic-automatic-feature-in-adobe-photoshop-cs6/adobe-cs6`
  — CS6 Bicubic Automatic in Preferences > General as the default method, with
  per-dialog override in Image Size.

## Open questions

- **Factory default interpolation method in CS6** (Bicubic Automatic vs Bicubic).
  The Help only says Bicubic Automatic is new; community sources say it is the
  default. *Resolves with:* a clean CS6 install screenshot or the CS6 preference
  defaults reference.
- **Exact Bicubic/Smoother/Sharper kernels and edge handling** (coefficients,
  clamp/wrap, gamma-correct resampling). *Resolves with:* a resample test against
  CS6 output, or Adobe SDK/Camera Raw documentation.
- **Bicubic Automatic direction mapping** (Sharper down / Smoother up) and its
  behavior at 1:1. *Resolves with:* CS6 output comparison at multiple scales.
- **Scale Styles arithmetic** for each style parameter and for smart filters.
  *Resolves with:* CS6 measurement of style parameters after resize.
- **Whether the dialog is enabled in Bitmap/Indexed/Duotone modes**, and which
  resample methods are offered there. *Resolves with:* CS6 UI observation per
  mode.
- **Exact Image Size shortcut** (`Alt+Ctrl+I` assumed). *Resolves with:* the CS6
  keyboard-shortcuts reference (`02-ui-ux/keyboard-shortcuts.md`).
- **Resampling a 32-bpc document**: whether Bicubic Automatic falls back to a
  specific kernel and whether highlight clipping is prevented. *Resolves with:*
  CS6 test on an HDR document.
