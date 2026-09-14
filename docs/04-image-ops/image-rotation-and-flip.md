# Image Rotation and Flip

- **Spec ID:** `IMG-003`
- **Status:** `Draft`
- **Parity tier:** `Core`
- **New in CS6:** `No` — the `Image > Image Rotation` submenu (180°, 90° CW, 90° CCW, Arbitrary, Flip Canvas Horizontal/Vertical) is unchanged from CS5; CS6's new non-destructive viewing rotation lives in the separate Rotate View tool.
- **Depends on:** `IMG-001` (Image Size), `IMG-002` (Canvas Size), `TOOL-011` (Crop tool straighten), `TOOL-017` (Ruler/Eyedropper), `01-architecture/document-model.md` (`ARCH-002`), `01-architecture/undo-history.md`, `02-ui-ux/menus.md`.

## CS6 behavior

`Image > Image Rotation` rotates or flips an entire image. The commands act on
the whole document, not on individual layers, parts of layers, paths, or
selection borders; transforming a selection or layer uses Transform/Free
Transform instead. Source: CS6 Help, "Rotate or flip an entire image".

Submenu commands:

- **180°** — half-turn.
- **90° CW** — quarter-turn clockwise.
- **90° CCW** — quarter-turn counterclockwise.
- **Arbitrary** — rotate by a user angle between **−359.99 and 359.99**. The
  dialog offers `°CW` / `°CCW` to choose direction; click OK to apply. As with
  90/180, Photoshop grows the canvas to the rotated image's bounding box.
- **Flip Canvas Horizontal** — flips the image along the vertical axis.
- **Flip Canvas Vertical** — flips the image along the horizontal axis.

Notes from the Help:

- **Image Rotation is destructive** and actually modifies the file information.
  For non-destructive viewing rotation, use the **Rotation (Rotate View) tool**
  (see `TOOL-013`).
- The 90°/180° turns and flips are lossless integer remaps; **Arbitrary rotation
  requires interpolation** and produces new pixels at the rotated edges. The
  Help does not state the interpolation method for Arbitrary; the Image Size /
  General-preferences interpolation method is the natural default (inferred).

### Straighten paths

Two documented ways to straighten a crooked scan or photo:

- **Crop tool Straighten**: draw a reference line along a feature that should be
  horizontal or vertical; the image is rotated to level the line and the canvas
  is resized to the rotated pixels. With the Crop tool's straighten control,
  Photoshop straightens and automatically crops; holding `Alt`/`Option` when
  clicking Straighten avoids the automatic crop (revealing the surrounding
  canvas). Source: CS6 Help, "Crop and straighten photos | CS6".
- **Ruler tool**: drag a measuring line along a feature that should be horizontal
  or vertical, then choose `Image > Image Rotation > Arbitrary`. The angle needed
  to straighten the image is filled into the Rotate Canvas dialog automatically.
  Source: CS6 Help, "Position with the Ruler tool".

Straighten is a convenience wrapper over Arbitrary rotation + canvas growth +
crop; it is specified here and in `TOOL-011`.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| `Image > Image Rotation > 180°` | Menu | — | half-turn, lossless |
| `Image > Image Rotation > 90° CW` | Menu | — | quarter-turn, lossless |
| `Image > Image Rotation > 90° CCW` | Menu | — | quarter-turn, lossless |
| `Image > Image Rotation > Arbitrary` | Menu / dialog | — | −359.99 … 359.99, `°CW`/`°CCW` |
| `Image > Image Rotation > Flip Canvas Horizontal` | Menu | — | flips about vertical axis |
| `Image > Image Rotation > Flip Canvas Vertical` | Menu | — | flips about horizontal axis |
| Crop options bar | Straighten toggle | — | draw reference line |
| Ruler options bar | Straighten button | — | then `Image > Image Rotation > Arbitrary` |
| `Edit > Transform` | Transform / Free Transform | `Ctrl/Cmd+T` | rotates a layer/selection instead |

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Angle (Arbitrary) | Number + direction | 0 | −359.99 … 359.99 | degrees, two decimals |
| Direction | Enum | CW (inferred) | `°CW`, `°CCW` | |
| 180 / 90 CW / 90 CCW | Action | — | — | fixed angles |
| Flip Canvas Horizontal/Vertical | Action | — | — | fixed axes |

## Algorithms & pipeline

- **90°/180° rotations and flips** are exact index remaps of the raster:
  `(x, y) -> (H-1-y, x)` for 90° CW (dimensions swap), etc. No resampling, no
  color change, reversible bit-for-bit by the inverse command.
- **Arbitrary rotation**: rotate the image about the canvas center by ±θ, grow
  the canvas to the axis-aligned bounding box of the rotated rectangle
  (`W' = W|cos θ| + H|sin θ|`, `H' = W|sin θ| + H|cos θ|`), and resample into the
  new grid. The corners outside the original image are filled with the background
  color or left transparent for a transparent-background document (inferred from
  Canvas Size behavior). Interpolation method is inferred as the General
  preferences method; **behavioral parity only, kernel TBD**.
- **Canvas growth**: dimension formulas above are the standard bounding-box
  result and are inferred; the Help only says the canvas accommodates the
  rotated image.
- **Straighten**: measure the angle of the reference line from the nearest axis;
  rotate by its negative; grow the canvas; optionally auto-crop to the largest
  axis-aligned rectangle (Crop tool). Alt/Option skips the crop.
- Rotating a document with layers rotates all layer rasters/masks and their
  geometry; the Help states the command is document-wide.

## Rust module mapping

Proposed (names provisional), following `ARCH-002`:

- `pictura-core::ops::rotate::RotateSpec` — `QuarterTurn(Cw|Ccw)` | `Half` |
  `Arbitrary { degrees: f64, clockwise: bool }` |
  `Flip { axis: Horizontal|Vertical }`.
- `pictura-core::ops::rotate::apply(doc, spec) -> Command` — exact remap for
  quarter/half turns and flips; resample path for Arbitrary.
- `pictura-core::geometry::transform::rotate_bounds(w, h, angle) -> (w2, h2)` —
  bounding-box growth; reused by Crop straighten (`TOOL-011`) and Perspective
  Crop (`TOOL-012`).
- `pictura-core::ops::straighten::StraightenSpec` — reference points / measured
  angle, `auto_crop: bool`; delegates to `rotate::apply` + crop.
- Boundary types: `AngleDeg(f64)`, `Axis`, `QuarterTurn`.

## Qt6 component mapping

- `ImageRotationMenu` — dynamic `QMenu` mirroring the `Image > Image Rotation`
  submenu; commands emit `rotateRequested(RotateSpec)`.
- `ArbitraryRotateDialog` (`QDialog`) — angle spin box (−359.99 … 359.99) and a
  CW/CCW toggle; reuses `AngleSpinBox` from Transform (`TOOL-014`).
- `StraightenController` (`QObject`) — converts a reference line in the canvas
  scene into an angle and calls the straighten op; used by Crop and Ruler.
- Widgets over QML: menu integration and a small modal dialog favor Widgets; the
  canvas overlay for the reference line is a `QGraphicsScene` item.

## Data-model impact

- Document pixel dimensions swap (90°/270°) or grow (Arbitrary); all layer
  rasters, masks, and vector/type geometry rotate with the document.
- Undo: one history state per command. Record shape: rotation/flip enum (+angle)
  and before/after pixel dimensions; the inverse command restores exactly for
  quarter/half/flip, while Arbitrary redo reverses resampling only to the
  preserved prior raster.
- Serialization: no dedicated PSD key; the stored image data and resolution
  already reflect the rotation. XMP orientation metadata may be written for
  exports — see `10-workflow-io/file-info-and-metadata.md`.
- Smart Objects: rotating the canvas does not rotate the Smart Object's embedded
  source pixels (community-inferred; same pattern as crop); re-editing the Smart
  Object reveals unrotated source. Track under Open questions.

## Edge cases

- **Lossless vs lossy**: 90°/180°/flips must be pixel-exact and reversible;
  Arbitrary introduces interpolation error and is not exactly reversible.
- **Transparent background**: Arbitrary-rotation corners are transparent rather
  than filled (inferred).
- **Bitmap mode**: the Help documents that rotation is not allowed for Bitmap
  mode images (CS5 statement assumed to carry into CS6 — inferred).
- **16/32-bit**: rotation works for all depths; 32-bpc rotation must run in
  float. The CS6 32-bpc feature list includes Image Rotation.
- **CMYK/Lab/Indexed/Duotone**: mode-independent geometry; resampling operates
  per channel in the working space.
- **1-px / huge documents**: bounding-box math in 64-bit; PSB 300,000 px limit.
- **Odd/even dimensions**: 90° remaps must handle non-square documents; no
  off-by-one.
- **Undo/redo**: quarter/half/flip undo restores exact raster; Arbitrary undo
  restores the pre-rotation canvas and discards interpolated pixels.
- **GPU-unavailable**: CPU rotation/resample path.

## Parity acceptance criteria

1. Given a non-square 8-bit RGB document, `90° CW` swaps W/H and maps each source
   pixel to the exact target index; `90° CCW` then restores the original exactly.
2. Given `180°` twice, the document is bit-identical to the original.
3. Given `Flip Canvas Horizontal` then `Flip Canvas Vertical`, the result is
   equivalent to `180°`.
4. Given `Arbitrary` at +θ then −θ, the result differs only by interpolation
   error (not exactly reversible) and canvas dimensions match the bounding box.
5. Given Arbitrary on a transparent-background document, corner pixels outside
   the original rectangle are fully transparent.
6. Given a Ruler measuring line at angle θ from horizontal, `Arbitrary` is
   pre-filled with the correction angle within 0.01°.
7. Given Crop-tool Straighten with a featured line, committing levels the line
   and the final canvas is cropped; `Alt`/`Option`-clicking Straighten leaves the
   surrounding canvas visible.
8. Given any command, exactly one history state is added; undo restores the prior
   document.

## Sources

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` —
  official CS6 Help reference: "Rotate or flip an entire image" (submenu
  commands, −359.99 … 359.99 Arbitrary, `°CW`/`°CCW`, Flip Canvas, destructive
  note and Rotate View pointer), "Crop and straighten photos | CS6" (Crop
  Straighten, automatic crop, Alt/Option to avoid cropping), "Position with the
  Ruler tool" (measuring line auto-fills the Arbitrary angle), the 32-bpc feature
  list (Image Rotation), and the CS6 note that the Rotation tool provides
  non-destructive viewing rotation.

## Open questions

- **Interpolation method used by Arbitrary Image Rotation** (General preference
  vs a fixed Bicubic). *Resolves with:* a CS6 rotation output comparison and an
  inspection of the General preferences interaction.
- **Canvas growth fill** for opaque vs transparent backgrounds (background color
  vs transparent). *Resolves with:* CS6 test on both document types.
- **Bitmap-mode rotation restriction** in CS6 (documented for CS5). *Resolves
  with:* CS6 behavior test.
- **Smart Object semantics**: whether the canvas rotation leaves embedded source
  pixels unchanged. *Resolves with:* CS6 test/re-editing a Smart Object after a
  canvas rotation.
- **Exact shortcut for Rotate View and whether Arbitrary has one.** *Resolves
  with:* the CS6 keyboard shortcuts reference.
- **Whether straighten angle is measured in image pixels or screen space** at
  non-100% zoom. *Resolves with:* CS6 observation at zoom.
