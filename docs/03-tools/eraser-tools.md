# Eraser Tools

- **Spec ID:** `TOOL-023`
- **Status:** `Draft`
- **Parity tier:** `Core`
- **New in CS6:** `No` — the Eraser, Background Eraser, and Magic Eraser predate CS6 and are carried forward. The Pencil tool's Auto Erase (documented in the same CS6 Help topic) is carried too. No CS6-specific eraser change is listed in the Help.
- **Depends on:** `01-architecture/rust-core-design.md`, `01-architecture/undo-history.md`, `03-tools/brush-and-pencil.md`, `03-tools/color-replacement.md`, `05-layers/layer-masks.md`, `04-image-ops/image-modes.md`.

## CS6 behavior

The CS6 Help groups four topics under "Erasing parts of an image": the Eraser
tool, the Magic Eraser, the Background Eraser, and Pencil Auto Erase.

### Eraser tool (`E`)

- Changes pixels to **either the background color or transparency**:
  - On a **background** layer or a layer with **Lock Transparency** on, pixels
    become the **background color**.
  - Otherwise pixels are erased **to transparency**.
- The Eraser can also return the affected area to a state selected in the History
  panel (**Erase To History**).
- Procedure: select the Eraser; set the background color for background/locked
  cases; choose a **Mode**:
  - `Brush` and `Pencil` make the eraser behave like those tools, with a brush
    preset and **Opacity**/**Flow**.
  - `Block` is a hard-edged, fixed-size square with no opacity or flow control.
- Opacity 100% erases completely; lower opacity erases partially.
- **Erase To History:** select a history state/snapshot, then enable Erase To
  History; holding `Alt`/`Option` while dragging temporarily uses that mode.

### Magic Eraser

- Clicking with the Magic Eraser changes **all similar pixels to transparent**.
- On a layer with **Lock Transparency**, pixels change to the **background
  color**. Clicking the background **converts it to a layer** and erases similar
  pixels to transparency.
- Options:
  - **Tolerance** defines the range of colors erased.
  - **Anti-aliased** smooths the erased edges.
  - **Contiguous** erases only pixels contiguous to the clicked one; deselect to
    erase all similar pixels in the image.
  - **Sample All Layers** samples the erased color from combined visible layers.
  - **Opacity** sets the strength of the erasure (100% = complete).

### Background Eraser

- Erases pixels **to transparency** as the user drags, maintaining the edges of a
  foreground object.
- Samples the color at the center of the brush, the **hotspot**, and deletes that
  color wherever it appears inside the brush. It also performs color extraction at
  the edges of foreground objects so color halos are not left behind when the
  foreground is pasted elsewhere.
- **Overrides a layer's Lock Transparency setting.**
- Brush options: Diameter, Hardness, Spacing, Angle, Roundness; plus **Size** and
  **Tolerance** menus that can be varied by `Pen Pressure` or `Stylus Wheel` (or
  `Off`).
- Options bar:
  - **Limits**: `Discontiguous` (sampled color wherever it occurs under the
    brush), `Contiguous` (connected areas), `Find Edges` (connected areas while
    better preserving shape-edge sharpness).
  - **Tolerance**: low limits to colors very similar to the sampled color; high
    covers a broader range.
  - **Protect Foreground Color**: prevents erasing areas matching the current
    **foreground** color.
  - **Sampling**: `Continuous` (sample as you drag), `Once` (only the color first
    clicked), `Background Swatch` (only the current background color).
- The pointer is a brush shape with a cross hair at the hotspot.

### Pencil Auto Erase

Cross-referenced from `03-tools/brush-and-pencil.md`: with the **Pencil** tool and
**Auto Erase** on, if the drag begins with the cursor center over the foreground
color, the area is erased to the background color; otherwise it is painted with
the foreground color.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| Tools panel (Eraser slot) | Tool | `E` | Eraser; hidden tools: Background Eraser, Magic Eraser. |
| Options bar — Eraser | Tool bar | n/a | Mode (Brush/Pencil/Block), brush preset, Opacity, Flow, Erase To History. |
| Options bar — Magic Eraser | Tool bar | n/a | Tolerance, Anti-aliased, Contiguous, Sample All Layers, Opacity. |
| Options bar — Background Eraser | Tool bar | n/a | Limits, Tolerance, Protect Foreground Color, Sampling; brush pop-up with Size/Tolerance control menus. |
| Brush panel | Dock | `Window > Brush` | Tip shape/dynamics for Eraser (Brush/Pencil) and Background Eraser. |
| History panel | Dock | `Window > History` | Source state for Erase To History. |
| Foreground/Background swatch | Toolbox | `D`, `X` | Background color used when erasing on background/locked layers; foreground = Protected color. |
| Pencil options bar | Tool bar | `B` (Pencil) | Auto Erase toggle. |

## Parameters & ranges

| Control | Tool | Type | Default | Range / options | Notes |
|---|---|---|---|---|---|
| Mode | Eraser | enum | Brush | Brush / Pencil / Block | Block is fixed square, no opacity/flow. |
| Opacity | Eraser | percent | 100 | 0 – 100 | 100 = complete erase; lower = partial. |
| Flow | Eraser | percent | 100 | 0 – 100 | Brush/Pencil modes only. |
| Erase To History | Eraser | bool | off | on / off | Uses selected History state/snapshot. |
| Tolerance | Magic Eraser | int | not stated | not stated | Color range erased. |
| Anti-aliased | Magic Eraser | bool | on (inferred) | on / off | Smooths erased edges. |
| Contiguous | Magic Eraser | bool | on (inferred) | on / off | Off = erase all similar pixels. |
| Sample All Layers | Magic Eraser | bool | off | on / off | Sample combined visible layers. |
| Opacity | Magic Eraser | percent | 100 | 0 – 100 | Erasure strength. |
| Diameter / Hardness / Spacing / Angle / Roundness | Background Eraser | int / percent / percent / deg / percent | per tip | not stated | Brush pop-up. |
| Size control | Background Eraser | enum | Off | Off / Pen Pressure / Stylus Wheel | Varies size over stroke. |
| Tolerance control | Background Eraser | enum | Off | Off / Pen Pressure / Stylus Wheel | Varies tolerance over stroke. |
| Limits | Background Eraser | enum | Contiguous (inferred) | Discontiguous / Contiguous / Find Edges | Erasure region rule. |
| Tolerance | Background Eraser | int | not stated | not stated | Color range erased. |
| Protect Foreground Color | Background Eraser | bool | off | on / off | Don't erase pixels matching foreground. |
| Sampling | Background Eraser | enum | Continuous (inferred) | Continuous / Once / Background Swatch | Target sampling rule. |
| Auto Erase | Pencil | bool | off | on / off | See `TOOL-020`. |

## Algorithms & pipeline

Behavioral-parity proposals; Adobe's implementations are closed.

1. **Eraser (Brush/Pencil modes).** Compute the tip footprint exactly as for the
   Brush/Pencil (`TOOL-020`). For each covered pixel:
   - if the target is a background layer or Lock Transparency is on → composite
     the **background color** with coverage = `Opacity × Flow`;
   - otherwise → reduce the destination alpha by `coverage` (erase to
     transparency), and at full coverage clear RGB toward 0 to avoid color fringing.
   - Block mode uses a fixed hard square mask at full strength.
2. **Erase To History.** Instead of alpha subtraction, restore the pixel from the
   chosen History state/snapshot (tile-diffusion lookup,
   `01-architecture/undo-history.md`). `Alt`/`Option` temporarily selects this
   mode.
3. **Magic Eraser.** A global (non-contiguous) or flood-fill (contiguous) region
   grow from the clicked pixel using a color-distance test against `Tolerance`
   (`pictura-paint::color_distance`, shared with Color Replacement and Magic
   Wand). Selected pixels have their alpha reduced by `Opacity`; if
   `Anti-aliased`, boundary is partial. On a Lock-Transparency layer the result
   is applied as background-color fill instead. Clicking a background layer first
   converts it to a normal layer.
4. **Background Eraser.** For each dab, sample the hotspot color `H`. Grow a
   connected/discontiguous region within the brush by the `Tolerance` distance to
   `H` (per `Limits`). Set alpha to 0 across the matched region. For `Find Edges`,
   run the region grow but stop the erasure at strong local gradients so object
   edges survive. Also perform edge color extraction: at the matched/unmatched
   boundary, remove the sampled background contribution so no halo remains.
   `Protect Foreground Color` excludes pixels near the foreground color.
   `Sampling` chooses whether `H` is live, frozen at first click, or the current
   background swatch.
5. **Rasterization / commit.** Strokes accumulate in a sparse tile scratch buffer
   and commit tiles on pointer release, like other brush-like tools. Magic Eraser
   is a single-click commit.

## Rust module mapping

Design proposal.

- `pictura-paint::eraser` — `EraserEngine` (Brush/Pencil/Block modes),
  `EraseMode`, `erase_to_history` path.
- `pictura-paint::magic_eraser` — `MagicEraserEngine` (flood fill / global
  threshold, `tolerance`, `contiguous`, `anti_aliased`, `sample_all_layers`,
  `opacity`).
- `pictura-paint::background_eraser` — `BackgroundEraserEngine`
  (`Limits { Discontiguous, Contiguous, FindEdges }`, `Sampling { Continuous,
  Once, BackgroundSwatch }`, `protect_foreground`, pressure-varying
  size/tolerance), hotspot sampling and edge extraction.
- `pictura-paint::color_distance` — shared tolerance kernel.
- `pictura-paint::flood_fill` — scanline region grow used by Magic Eraser,
  Magic Wand, and Paint Bucket.
- `pictura-core::command` — `EraseStrokeCommand`, `MagicEraseCommand`,
  `BackgroundEraseCommand`; one history record each.

Crossing types: `LayerId`, `Rect`, `ColorSpace`, `PixelBuffer`, `TileDelta`.

## Qt6 component mapping

- `EraserOptionsBar` (`QWidget`) — Mode, tip, Opacity, Flow, Erase To History.
- `MagicEraserOptionsBar` / `BackgroundEraserOptionsBar` (`QWidget`).
- `BrushPanel` — reused for tip shape/dynamics.
- `HistoryPanel` — source-state selection for Erase To History.
- `CanvasView` (`QGraphicsView`) — hotspot cross-hair cursor for the Background
  Eraser; pixels stay in the GPU pipeline.

## Data-model impact

- **No PSD fields.** Only pixel tiles and the layer's existing lock/mask fields
  are involved.
- **Layer transparency.** Erasing to transparency mutates the layer's
  transparency (alpha) channel; on a background layer the same operation instead
  writes the background color. The model must expose whether a layer is a
  `Background` and whether Lock Transparency is set.
- **Background Eraser overrides Lock Transparency**, unlike the Eraser and Magic
  Eraser. The command layer must encode that the lock is intentionally bypassed.
- **Erase To History** is a non-destructive restore relative to History, so its
  undo record is a tile delta against the current state.
- **Magic Eraser on a Background layer** first converts the background to a
  normal layer — a layer-kind/structural change plus a pixel edit, recorded as
  one compound undo operation.

## Edge cases

- **Document color mode.**
  - RGB / CMYK / Lab: alpha erasing is available; the working-space conversion
    wraps the pixel edits.
  - Grayscale: same alpha behavior.
  - Bitmap: 1-bit; there is no alpha to subtract, so the Eraser paints the
    background color. Confirm the Magic/Background Eraser enableness in Bitmap.
  - Indexed: a single transparency index; erasing to transparency is limited.
    Cross-ref `04-image-ops/image-modes.md`.
  - Multichannel: transparency handling differs (no single alpha layer);
    confirm behavior.
- **Background layer.** Eraser and Magic Eraser write the background color;
  Magic Eraser converts the background to a layer first.
- **Lock Transparency.** Eraser and Magic Eraser write background color;
  Background Eraser overrides the lock and erases.
- **Empty / 1-px layers.** Flood fill and footprint math must no-op safely.
- **Huge documents (PSB).** Magic/Background Eraser region grows must operate on
  tiles, not a whole-canvas bitmap.
- **Anti-aliased Magic Eraser** produces partial alpha at edges; the boundary
  rule must be deterministic.
- **Undo/redo.** A drag stroke is one state; a Magic Eraser click is one state;
  Background Eraser is one state per stroke (matching `01-architecture/undo-history.md`).
- **Undo mid-stroke.** Atomic on release.
- **GPU unavailable.** All erasers must work on the CPU fallback.

## Parity acceptance criteria

1. Given a **background** layer, the Eraser writes the current background color;
   given a normal layer with Lock Transparency off, it writes transparency
   (alpha → 0).
2. Given a normal layer with Lock Transparency on, the Eraser writes the
   background color; the Background Eraser still erases to transparency.
3. Given Erase To History with a chosen state, dragging restores that state's
   pixels; `Alt`/`Option`-drag does the same temporarily.
4. Given a Magic Eraser click in a flat region with contiguous on, only the
   connected same-color region becomes transparent; with contiguous off, all
   similar pixels do. On a Background layer, the background is converted to a
   layer first.
5. Given Background Eraser `Sampling = Once`, changing canvas colors mid-drag does
   not change the target; `Continuous` adapts; `Background Swatch` uses the
   background color.
6. Given Background Eraser `Limits = Find Edges`, object edges are preserved
   better than under `Contiguous` in a side-by-side diff, and no background halo
   remains at the edge.
7. Given `Protect Foreground Color`, pixels matching the foreground color are
   not erased.
8. Given a Magic Eraser click, exactly one history state is created; given a drag
   eraser, exactly one per stroke.

## Sources

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` —
  official Photoshop CS6 Help reference, "Erasing parts of an image"
  (pp. 423–425): "Erase with the Eraser tool" (background vs transparency,
  Brush/Pencil/Block modes, opacity, Erase To History and `Alt`/`Option`);
  "Change similar pixels with the Magic Eraser tool" (tolerance, anti-aliased,
  contiguous, sample-all-layers, opacity, background→layer); "Change pixels to
  transparent with the Background Eraser tool" (hotspot sampling, size/tolerance
  pressure control, Discontiguous/Contiguous/Find Edges, tolerance, protect
  foreground, Continuous/Once/Background Swatch, overrides Lock Transparency);
  "Auto Erase with the Pencil tool".
- `https://www.w3.org/TR/compositing-1/` — standard blend/compositing formulas
  referenced for coverage compositing.

## Open questions

- **Numeric Tolerance range** for the Magic and Background Erasers (0–255 vs
  percent) and the default values are not stated in the Help. *Resolves with:* a
  CS6 options-bar capture or a scripted probe.
- **Defaults** for Contiguous, Anti-aliased, Limits, and Sampling are inferred.
  *Resolves with:* a CS6 screenshot on tool selection.
- **Bitmap / Indexed / Multichannel enableness** for the Magic and Background
  Erasers is not documented. *Resolves with:* CS6 experiments per color mode.
- **Exact Background Eraser edge-extraction algorithm** (how the halo is
  removed) is unspecified. *Resolves with:* pixel comparison against CS6 and an
  accepted tolerance.
- **Background Eraser brush shape/hotspot geometry** (how the hotspot and brush
  bounds interact) is not fully specified. *Resolves with:* a CS6 UI reference.
- **Eraser behavior on group / smart-object / adjustment layers** is not stated.
  *Resolves with:* CS6 experiments.
