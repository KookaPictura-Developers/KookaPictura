## Context

`pictura-select` already implements the whole selection algebra the CS6 Select
menu and marquee/lasso/wand tools need: `Selection::{rect,ellipse,polygon,
invert,feather,expand,contract,border,smooth}`, `combine`/`combine_with`,
`magic_wand`/`grow`/`similar`/`color_range`, and `to_channel`/`from_channel`
(`crates/pictura-select/src/lib.rs`). The app bridge exposes only select-all,
deselect, magic wand (contiguous, tolerance), rect/ellipse, lasso, quick-select,
crop, and bounds
(`crates/pictura-app/src/cxxqt_object/impl_selection.rs`). The CS6 toolbox marks
Elliptical Marquee, Polygonal Lasso, and Magic Wand `implemented = false`
(`tools.cpp:22-33`); their options-bar pages are a bare label; and every Select
menu row except All/Deselect is an unimplemented `leaf(...)`
(`command_tree.cpp:502-531`). The work is wiring, not new math.

Constraints: no `crates/` code in this change (proposal only); `docs/` is the
behavioral contract and is not edited; milestones must not appear in
identifiers or test names; files stay under the 800-LOC target; new `.cpp`/`.h`
files would need a `CMakeLists.txt` entry, so the design avoids new files where
the standard Qt dialogs suffice.

## Goals / Non-Goals

**Goals:**

- Enable Elliptical Marquee, Polygonal Lasso, and Magic Wand with CS6 options.
- Give the marquee/lasso/Quick-Selection options bars their real controls.
- Wire the Select menu (Reselect, Inverse, Modify, Grow, Similar, Save/Load,
  All/Deselect/Similar Layers) through the bridge with CS6 enable rules and one
  undo state per applied command.
- Keep every un-modelled option visible and honest (anti-alias, per-layer
  sampling, Auto-Enhance) or visible-but-disabled with a documented reason
  (Magnetic Lasso, Color Range, Refine Edge, Transform Selection).
- Leave a runnable check: Rust tests for any new engine-facing logic, app
  self-tests per tool and per menu command, oracle checks where one exists.

**Non-Goals:**

- Magnetic Lasso edge following, Color Range UI, Refine Edge UI, Transform
  Selection, Free Transform, and channel *operations* beyond New (Add/Subtract/
  Intersect on save) — deferred with reasons.
- Anti-aliased rasterisation of ellipse/polygon/wand boundaries; Photoshop's
  exact AA filter is undocumented and the engine is binary (inferred ceiling).
- Active-layer-only sampling for Magic Wand / Quick Selection / Grow / Similar;
  the engine's `current_buffer` is the visible composite.
- PSD round-tripping of saved-selection *names*; `Channel` carries only `id`.
- New selection algorithms or changes to `crates/pictura-select`.

## Decisions

### D1. Bridge surface in `impl_selection.rs`

Add to `PictureView`:

| Method | Engine call | Notes |
|---|---|---|
| `invert_selection() -> bool` | `Selection::invert` | One `record("Inverse Selection")`. |
| `reselect() -> bool` | restores `deselected_selection` | `record("Reselect")`. |
| `modify_selection(op: &QString, amount: f64) -> bool` | `border`/`smooth`/`expand`/`contract`/`feather` | `op` is `border`/`smooth`/`expand`/`contract`/`feather`; unknown op returns false. One record. |
| `grow_selection(tolerance: i32) -> bool` | `pictura_select::grow(sel, buffer, tol)` | One record. |
| `similar_selection(tolerance: i32) -> bool` | `pictura_select::similar(sel, buffer, tol)` | One record. |
| `save_selection(name: &QString) -> bool` | `Selection::to_channel(id)` appended to `doc.channels` | `record("Save Selection")`. |
| `load_selection(name: &QString) -> bool` | `Selection::from_channel(ch, w, h)` | `record("Load Selection")`. |
| `select_all_layers() -> QStringList` | `pictura_render::flatten_rows(doc)` paths | Read-only; no history. |

`select_ellipse` gains a `feather: f64` argument applied to the rasterised shape
before `combine_with`. `magic_wand` and `quick_select` gain a `contiguous: bool`
argument. `apply_selection` is unchanged.

`deselected_selection: Option<Selection>` is a new field alongside `selection`.
`deselect()` moves the current selection into it; a New-mode `apply_selection`
or `select_all()` replacing a non-empty selection also moves it there; a
committed tool selection clears it. This is CS6's "last deselected" memory.

`deselect_layers` and `select_similar_layers` are **not** Rust bridge methods:
the involved state (which rows are selected) lives in `LayersPanel`
(`selectedPaths`/`selectPaths`), not the document. "All Layers" needs the
document side, so `select_all_layers()` supplies the paths and the frame calls
`layersPanel_->selectPaths(paths, paths.first())`. "Deselect Layers" clears via
`selectPaths({}, {})`. "Similar Layers" reuses the existing
`PictureView::select_similar(path)` and then `selectPaths`.

### D2. Tool activation

`tools.cpp` flips `implemented` to true for `EllipticalMarquee`,
`PolygonalLasso`, and `MagicWand` in `kToolTable` and adds them to
`implementedToolIds()`. `ToolController::handlePressed/handleMoved/handleReleased`
gain cases:

- `EllipticalMarquee`: same drag state machine as `Marquee`, released through
  `select_ellipse` with the current feather. Style constraining (Fixed Ratio /
  Fixed Size) is applied to the drag rectangle in `ToolController` before the
  bridge call; Normal passes the drag rectangle through.
- `PolygonalLasso`: press appends a vertex to `polygonPoints_` and starts a
  `begin_lasso`; move repaints the preview overlay as `points + cursor`; a press
  within a small radius of the first vertex, or a second press within the
  double-click interval and distance, commits via `end_lasso()`. `Esc` clears
  the in-progress path without touching the selection. `Enter` is wired through
  the frame's key path only if the existing key handling makes that a one-liner;
  otherwise click-to-close and double-click close are the shipped path and
  Enter is documented as a gap. No new class: the state is fields on
  `ToolController` (one switch, not one class per tool).
- `MagicWand`: press calls `magic_wand(x, y, tolerance, contiguous, aa)`; no
  drag. `EllipticalMarquee`/`PolygonalLasso`/`MagicWand` emit
  `selectionCommitted()` on success.

`ToolController` gains option fields with defaults from CS6: `antiAlias_`
(true), `feather_` (0), `style_` (Normal), `fixedRatio_` (1:1 inferred),
`fixedSize_` (100×100 inferred), `contiguous_` (true), `sampleAllLayers_`
(true, not switchable), and Quick-Selection `autoEnhance_` (true, not
switchable).

### D3. Options bar

`options_bar.{h,cpp}` gains dedicated pages. Reused controls (mode buttons,
Feather spin, Tolerance spin) come from the existing `buildCombinePage`; the new
parameterisation is a helper `buildSelectionPage(ToolId)`:

- Marquee pages: mode, Feather (0-250, decimal), Anti-alias checkbox
  (disabled with reason for Rectangular; disabled/inferred for Elliptical),
  Style combo + conditional Fixed Ratio fields (`QDoubleSpinBox`×2) or Fixed
  Size fields (`QSpinBox`×2, px only — the in/cm unit combo is deferred and
  documented).
- Lasso pages: mode, Feather, Anti-alias (disabled, inferred).
- Polygonal Lasso: same as Lasso.
- Magic Wand: mode, Tolerance, contiguous checkbox, Anti-alias (disabled,
  inferred), Sample All Layers (checked, disabled: engine samples the visible
  composite).
- Quick Selection: mode (New/Add/Subtract — no Intersect), Tolerance, Sample
  All Layers (checked, disabled), Auto-Enhance (checked, disabled), Refine Edge
  (deferred, not shown).

Anti-alias and Sample All Layers controls stay visible and carry a tooltip that
names the ceiling, per the "keep the option honest" rule. Disabled controls are
not silently enabled no-ops.

### D4. Command ids and menu wiring

`commands.h` gains stable ids:

```
select.reselect              select.inverse
select.modify.border         select.modify.smooth
select.modify.expand         select.modify.contract
select.modify.feather        select.grow
select.similar               select.save
select.load                  select.allLayers
select.deselectLayers        select.similarLayers
```

In `command_tree.cpp`, the corresponding `leaf(...)` rows become
`registry.add(commandId, path, label, shortcut, true)`; shortcuts stay as
documented (`Shift+Ctrl+D`, `Shift+Ctrl+I`, `Ctrl+Alt+A`, `Shift+F6`).
Color Range…, Refine Edge…, and Transform Selection stay `leaf(...)`
(unimplemented) and untouched, so they remain visible-but-disabled. Magnetic
Lasso stays `implemented = false` in the tool table.

`frame_menus.cpp` gains handlers and enable providers:

- Document-required: Reselect, Inverse, Modify*, Grow, Similar, Save, Load,
  All/Deselect/Similar Layers.
- Selection-required: Reselect (stored deselected selection non-empty),
  Inverse, Modify*, Grow, Similar, Save.
- Load: document + `doc.channels` non-empty.
- Similar Layers: document + a current layer (`layersPanel_->selectedPaths()`
  non-empty).
- Each handler records exactly one history state on success; a refusal (missing
  document, missing selection, engine error) returns without recording.

Parameter dialogs reuse `QInputDialog` (`getInt`/`getDouble`) rather than new
dialog classes, so no `CMakeLists.txt` change: Border 1-200, Smooth 1-100,
Expand/Contract 1-100, Feather 0-250 (double). Save Selection uses
`QInputDialog::getText` for the name and a New channel; Load uses
`QInputDialog::getItem` over `doc.channels` labels (`Alpha 1`…`Alpha N`). The
label→channel mapping is positional: `Alpha N` is `doc.channels[N-1]`; the
channel's `id` is whatever `to_channel` assigned. Save assigns the next free
`i16` id. Names are not persisted because `Channel` has no name field (ceiling).

### D5. Engine reuse map

| Command | Engine function |
|---|---|
| Elliptical Marquee | `Selection::ellipse` → `.feather(r)` → `combine_with` |
| Polygonal Lasso | `Selection::polygon` → `.feather(r)` → `combine_with` (via begin/add/end) |
| Magic Wand | `magic_wand(buffer, x, y, tolerance, contiguous)` |
| Reselect | stored `deselected_selection` |
| Inverse | `Selection::invert` |
| Modify: Border / Smooth / Expand / Contract / Feather | `Selection::border` / `smooth` / `expand` / `contract` / `feather` |
| Grow / Similar | `pictura_select::grow` / `similar` |
| Save / Load Selection | `Selection::to_channel` / `from_channel` |
| All Layers | `pictura_render::flatten_rows` |
| Similar Layers | `pictura_render::select_similar` (existing) |

### D6. Honest ceilings

- **Anti-alias** (Marquee/Lasso/Wand): the engine rasterisers are binary
  (pixel-centre tests), so the control is present and disabled with a reason.
  A fractional-coverage rasteriser would be new engine work.
- **Feather** (tool time): implemented, applied to the primitive before combine;
  `sigma = radius / 2` is the existing inferred profile (`selection-model` spec
  already documents it).
- **Sample All Layers**: present, checked, disabled — `current_buffer` is the
  visible composite; active-layer-only sampling is not wired.
- **Auto-Enhance**: present, checked, disabled — Quick Selection is currently
  `magic_wand(contiguous)` and has no edge-flow refinement.
- **Style units / defaults**: px only; 1:1 ratio and 100×100 size are inferred
  because the CS6 Help does not state shipped defaults.
- **Saved-selection names**: not persisted (`Channel.id` only).
- **Magnetic Lasso**: deferred — edge-map build, Width/Contrast/Frequency cost
  function, and fastening-point state machine are not in the engine.
- **Color Range / Refine Edge / Transform Selection**: deferred; their engine
  primitives exist or overlap with other milestones but their dialogs and
  transforms are separate work.

## Risks / Trade-offs

- [Enabled anti-alias/feather controls that do nothing] → anti-alias is
  disabled with a visible reason; feather is actually applied, so the enabled
  control is truthful.
- [Polygon close detection (first-vertex vs double-click) can mis-close] →
  radius and interval are constants; the preview shows the closing segment
  before commit; `Esc` clears.
- [Reselect memory can restore a stale selection after unrelated edits] →
  cleared on any newly committed tool selection; only Deselect and New-mode
  replacement populate it, matching the CS6 "last deselected" wording.
- [Save/Load name mapping is positional and brittle if channels are reordered]
  → documented; the dialog is built from the current `doc.channels` order at
  open time.
- [One undo state per command may be too coarse if a modify op is a no-op] →
  `record` is only reached after a successful engine call; no-op modify is
  avoided by refusing zero amounts.
- [Wiring many menu rows touches a large command table] → all ids are stable
  constants and every row keeps its documented path and shortcut, so future
  localization and custom menu sets are unaffected.

## Migration Plan

N/A — additive UI/bridge wiring in one crate, no schema or on-disk format
change. Rollback is reverting the change; disabled tools return to disabled.

## Open Questions

- Enter-to-close for the Polygonal Lasso: does the frame key path expose a
  one-line hook, or is double-click/click-first-point the shipped interaction?
- Does the engine ever gain a fractional-coverage rasteriser (retires the
  anti-alias ceiling)?
- Should `Channel` gain a name field so Save/Load Selection parity is exact, or
  is positional `Alpha N` acceptable for the milestone?
- Are active-layer-only wand sampling and Quick-Selection Auto-Enhance in a
  later milestone, and if so do they belong to `selection-tools` or a new
  capability?

## Residual inferred behavior and documented ceilings

Independent verification confirmed the requirements. These deliberate
deviations are recorded so the archived spec does not overclaim.

- **Deferred tools/commands.** Magnetic Lasso (needs an edge map and a
  fastening-point tracker), Color Range…, Refine Edge…, and Transform Selection
  remain visible-but-disabled. The toolbox hint names the reason for Magnetic
  Lasso; the menu rows stay `leaf(...)`.
- **Option honesty.** Anti-alias (the rasteriser is binary), Sample All Layers
  (the wand already samples the visible composite), and Quick-Selection
  Auto-Enhance are shown checked/defaulted but disabled with tooltips. The
  Quick-Selection brush pop-up is deferred.
- **Shift 45° snap** on the Polygonal Lasso is deferred (`ponytail:` comment).
- **Channel naming.** `pictura_core::Channel` has no name field, so Save/Load
  Selection use positional `Alpha N` labels; a name is not persisted.
- **Reselect memory.** `deselected_selection` is session-only (not in a
  `History::Snapshot`); it is populated on Deselect, a New-mode replacement, and
  Select All replacing a non-empty selection, and cleared by other commits.
- **Similar Layers** reuses `select_similar`, which excludes the reference
  layer, so the handler prepends the current path to match CS6.
- **Empty `selectPaths`.** The panel now treats an empty path list as "clear the
  selection"; the Select Linked handler guards against applying an empty result
  so an unlinked layer keeps its current row selection.
