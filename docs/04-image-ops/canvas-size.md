# Canvas Size

- **Spec ID:** `IMG-002`
- **Status:** `Draft`
- **Parity tier:** `Core`
- **New in CS6:** `No` — the Canvas Size dialog and its options are unchanged from CS5; the CS6 Help documents the same Width/Height, Relative, Anchor, and Canvas Extension Color controls.
- **Depends on:** `IMG-001` (Image Size), `01-architecture/document-model.md` (`ARCH-002`), `01-architecture/undo-history.md`, `01-architecture/file-formats.md`, `02-ui-ux/menus.md`.

## CS6 behavior

`Image > Canvas Size` changes the canvas — the full editable area — without
resampling the image. Increasing the canvas adds space around the existing
image; decreasing it crops into the image. Source: CS6 Help, "Change the canvas
size".

- **Width and Height** take the new canvas dimensions, with a unit menu next to
  each field (inches, cm, mm, points, picas, percent, pixels, columns). By
  default the current canvas dimensions are shown.
- **Relative** switches Width/Height from absolute values to deltas: a positive
  number adds to the current canvas, a negative number subtracts from it. With
  Relative off, the entered value is the new absolute canvas dimension.
- **Anchor** is a 3×3 grid of squares. Selecting a square positions the existing
  image on the new canvas; the default is the center square. Choosing the
  top-left anchor and adding width/height, for example, adds the new space to
  the right and bottom.
- **Canvas Extension Color** controls the color of the added canvas:
  **Foreground**, **Background**, **White**, **Black**, **Gray**, or **Other**
  (opens the Color Picker). The white square to the right of the menu also opens
  the Color Picker.
- If the image has a **transparent background**, the added canvas is transparent
  and the Canvas Extension Color menu is not available: the Help states the menu
  "isn't available if an image doesn't contain a background layer."
- Cropping via Canvas Size discards pixels outside the new canvas; adding space
  does not add image data.

### Transparency behavior

- Transparent-background documents (no Background layer) get transparent added
  canvas; there is no fill color choice.
- Documents with a Background layer (or a regular layer with no transparency?)
  use the Canvas Extension Color. The Help's rule is specifically "doesn't
  contain a background layer", so the presence of a Background layer gates the
  menu. Whether a non-Background opaque layer also enables it is not documented
  (see Open questions).

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| `Image > Canvas Size` | Menu / dialog | `Alt+Ctrl+C` (Win), `Option+Cmd+C` (Mac) | Shortcut per CS6 map; verify |
| Dialog | Width / Height fields + unit menus | — | absolute by default |
| Dialog | Relative checkbox | — | turns W/H into deltas |
| Dialog | Anchor 3×3 grid | — | center default |
| Dialog | Canvas Extension Color menu | — | Foreground, Background, White, Black, Gray, Other |
| Dialog | Color swatch | — | opens Color Picker |
| Dialog | Current Size readout | — | shows current W × H and dimensions |
| Dialog | OK / Cancel | — | |

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Width | Number + unit | current canvas W | > 0 (absolute); any signed value with Relative | units: px, %, in, cm, mm, pt, pica, columns |
| Height | Number + unit | current canvas H | > 0 (absolute); any signed value with Relative | |
| Relative | Bool | Off | — | positive adds, negative subtracts |
| Anchor | Enum (3×3) | Center | 9 positions | positions existing image on new canvas |
| Canvas Extension Color | Enum | Background (inferred) | Foreground, Background, White, Black, Gray, Other | unavailable without a Background layer |
| Custom color | Color | — | full picker | via Other or the swatch |

## Algorithms & pipeline

- **Grow**: allocate a new canvas of W×H; blit the existing composited raster at
  the offset dictated by the anchor; initialize the added region to the extension
  color, or to transparent (alpha 0) when there is no Background layer.
- **Shrink**: crop the composited raster to the anchor-aligned rectangle of size
  W×H; discard pixels outside. This is equivalent to a destructive crop of the
  whole document (all layers), not a per-layer crop change.
- Because Canvas Size acts on the whole document, it must union and re-clip every
  layer's raster/bounds; layers are not resampled.
- **Color**: extension color comes from the current foreground/background color
  state or a literal picker color. In CMYK/Lab documents the custom color is
  stored in the document's color space; "White/Black/Gray" map to the mode's
  white/black/gray (e.g. 0/0/0/0 or 0/0/0/100 for CMYK). This mapping is
  inferred and needs verification.
- No interpolation is involved. Photoshop's exact compositing/clamping on grow is
  behavioral parity only where the added color meets partial-alpha edges.

## Rust module mapping

Proposed (names provisional), following `ARCH-002`:

- `pictura-core::ops::canvas_size::CanvasSizeSpec` — `width`, `height`
  (doc units), `relative: bool`, `anchor: Anchor9`, `extension: ExtensionColor`.
- `pictura-core::ops::canvas_size::apply(doc, spec) -> Command` — resizes the
  document canvas, offsets/re-clips all layers, fills the added region.
- `pictura-core::geometry::Anchor9` — nine-position enum with `(f32, f32)`
  fractions; reused by `TOOL-014` (Move & Transform align) and New Document.
- `pictura-core::document::CanvasRect` — `{ x, y, w, h: i64 }` document-space
  bounds, separate from per-layer bounds. 64-bit-safe for PSB.
- `ExtensionColor` — `Foreground | Background | White | Black | Gray | Custom(Color)`.

## Qt6 component mapping

- `CanvasSizeDialog` (`QDialog`) — Width/Height `QuantitySpinBox`es (shared with
  `IMG-001`), Relative checkbox, a custom `AnchorGrid` widget, extension-color
  `QComboBox` plus `ColorSwatchButton`.
- `AnchorGrid` (`QWidget`) — 3×3 clickable cells with a highlight for the active
  anchor; accessible via keyboard arrows and space.
- `ColorSwatchButton` (`QToolButton`) — opens `QColorDialog`; shared with other
  color controls.
- Widgets over QML: modal, native validation and the compact anchor grid favor
  Widgets.

## Data-model impact

- Document canvas rect (origin + W/H) changes; total pixel dims of the composite
  change correspondingly.
- On grow, each layer gains transparent/fill pixels in the added region; on
  shrink, layer content outside the new canvas is clipped (retained or discarded
  per the destructive semantics — Canvas Size is destructive).
- Undo: one history state per commit. Record shape: before/after canvas rect and
  the new pixels added (or the cropped-away region if undo-costly). Grow is cheap
  to reverse (discard added region); shrink may need the discarded region stored
  for redo.
- Serialization: canvas size is implicit in the PSD document image data and
  resolution; no separate PSD key is documented.
- If the document is a Smart Object source or has artboards, canvas bounds
  interact with those containers — see `05-layers/artboards.md` / smart objects.

## Edge cases

- **Transparent background**: added canvas transparent; extension-color menu
  disabled. No Background layer means no fill choice.
- **Relative negative larger than the canvas**: result would be zero/negative
  size — must be rejected (minimum 1×1).
- **Shrink through the anchor**: cropping can remove all content for a given
  anchor; empty result still renders as a valid empty canvas.
- **Anchor + Relative combined**: deltas apply around the chosen anchor; the
  original image shifts opposite to the anchor when growing.
- **16/32-bit**: the operation is color-model/depth independent; fill color must
  be represented at the document's depth (float fill for 32-bpc).
- **CMYK/Lab**: White/Black/Gray extension values map to the mode's channel
  encoding; Foreground/Background use current colors converted to the document
  mode.
- **Indexed/Bitmap/Duotone**: extension color limited to available palette
  entries; exact behavior inferred.
- **1-px / huge (PSB)**: 64-bit canvas coordinates; 300,000 px limit applies.
- **Undo/redo**: growing then undoing must restore exact prior bounds and discard
  fill; shrinking then undoing restores cropped pixels.
- **GPU-unavailable**: CPU blit/crop path.

## Parity acceptance criteria

1. Given a 1000×1000 document with a Background, Canvas Size 1200×1200, center
   anchor, Background extension color -> the original image is centered and a
   100 px border of the background color surrounds it.
2. Given the same document and the top-left anchor with Relative off and
   1200×1200 -> the new space appears on the right and bottom only.
3. Given Relative on and `+100` width / `-50` height -> the canvas becomes
   (W+100) × (H−50).
4. Given a transparent-background document, added canvas is fully transparent
   and the Canvas Extension Color menu is disabled.
5. Given extension color Foreground with a red foreground -> all added pixels are
   the foreground color.
6. Given Canvas Size smaller than the document, pixels outside the new bounds are
   absent after commit and undo restores them exactly.
7. Given OK, exactly one history state is added; a second identical commit with
   Relative is reversible to the original canvas.

## Sources

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` —
  official CS6 Help reference: "Change the canvas size" (Width/Height + units,
  Relative, Anchor, Canvas Extension Color options and the Color Picker swatch,
  transparency behavior, and the Background-layer gate on the extension menu);
  "Make a frame" (increasing canvas size to build a border); 32-bpc feature list
  includes `Canvas Size` under Image commands; "Specify columns for an image"
  (columns as a unit).

## Open questions

- **Default Canvas Extension Color** (Background vs White). *Resolves with:* a
  clean CS6 dialog screenshot.
- **Extension menu gating**: is it the presence of a literal Background layer,
  or simply the absence of transparency? *Resolves with:* CS6 test with an opaque
  regular layer above a transparent base.
- **White/Black/Gray channel values for CMYK/Lab/Indexed** documents. *Resolves
  with:* CS6 conversion tests in each mode.
- **Interaction with artboards and Smart Objects** (does Canvas Size resize the
  artboard container or change placement?). *Resolves with:* CS6 observation.
- **Exact shortcut** (`Alt+Ctrl+C` assumed). *Resolves with:* the CS6 keyboard
  shortcuts reference.
- **Whether shrink is recorded as a crop or as an irreversible pixel discard**
  for undo memory. *Resolves with:* CS6 history panel observation on a large file.
