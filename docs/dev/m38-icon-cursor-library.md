# M38 — full CS6 toolbox icons and cursors, plus panel icons

- **Status:** proposed (`openspec/changes/m38-icon-cursor-library`); not implemented.
- **Type:** user-requested interruption to the layers-panel program. See the
  numbering note below.
- **Contract:** the frozen tool catalogue, the icon/cursor style guides, the
  hotspot table, and the panel-icon inventory in this file.
- **Consumers:** `crates/pictura-app/cpp/tools.{h,cpp}` (the catalogue),
  `toolbox.cpp` (single-column flyout grid), `icons.{h,cpp}` (asset lookup and
  cursor hotspot), `assets/pictura.qrc`, `assets/icons/`, `assets/cursors/`.

> **Numbering note (collision resolution).** The layers-panel program
> (`docs/dev/layers-panel-program.md`) had claimed **M38 — panel anatomy**. This
> user-requested milestone takes the **M38** number, so the layers-panel stages
> shift by one: panel anatomy → **M39**, filtering/search → **M40**, management
> operations → **M41**, styles/effects → **M42**, smart objects / vector masks /
> layer comps → **M43**. `layers-panel-program.md` and `docs/dev/STATE.md` carry
> the same note.

## 1. Scope

The app ships 10 tools and 8 tool cursors (M19). CS6 ships ~71 toolbox tools in
23 flyout groups, plus a panel icon per panel and per Layers-panel action. This
milestone freezes the **whole catalogue now** and draws the **whole asset set**,
but only the 10 existing tools are *implemented*; every other tool is shown
**disabled** with a "not implemented yet" tooltip. No tool engine lands here.

This is an asset + toolbox-chrome milestone. The Rust side, the document model,
and the compositor are untouched.

## 2. The frozen tool catalogue (71 tools)

**Implemented** is `true` only for the 10 tools that exist in `ToolId` today:
Move, Marquee (Rectangular), Lasso, Quick Selection, Crop, Eyedropper, Hand,
Zoom, Brush, Pencil. The asset id for those 10 is **unchanged** from M19
(`marquee` = Rectangular Marquee; `lasso` = Lasso).

Column key: `impl` = implemented; `Qt` = the `QCursor` fallback shape;
`hotspot` = cursor action point in the 24×24 cursor space, `(x, y)`, origin
top-left.

### 2.1 Flyout groups (23 slots)

The toolbox is a fixed list of **23 slots** in CS6 table order. A slot's visible
tool is the last-used member (Brush-led for Paint, Rectangular for Marquee,
etc.). `†` marks an Extended-only tool.

| # | Slot | Key | Members |
|---|---|---|---|
| 1 | Move | `V` | Move |
| 2 | Marquee | `M` | Rectangular Marquee, Elliptical Marquee |
| 3 | Lasso | `L` | Lasso, Polygonal Lasso, Magnetic Lasso |
| 4 | Selection | `W` | Magic Wand, Quick Selection |
| 5 | Crop / Slice | `C` | Crop, Perspective Crop, Slice, Slice Select |
| 6 | Sampling / Measure | `I` | Eyedropper, Color Sampler, Ruler, Note, Count† |
| 7 | Retouch | `J` | Spot Healing Brush, Healing Brush, Patch, Content-Aware Move, Red Eye |
| 8 | Paint | `B` | Brush, Pencil, Color Replacement, Mixer Brush |
| 9 | Clone | `S` | Clone Stamp, Pattern Stamp |
| 10 | History | `Y` | History Brush, Art History Brush |
| 11 | Erase | `E` | Eraser, Background Eraser, Magic Eraser |
| 12 | Fill | `G` | Gradient, Paint Bucket |
| 13 | Blur | — | Blur, Sharpen, Smudge |
| 14 | Toning | `O` | Dodge, Burn, Sponge |
| 15 | Pen | `P` | Pen, Freeform Pen, Add Anchor Point, Delete Anchor Point, Convert Point |
| 16 | Type | `T` | Horizontal Type, Vertical Type, Horizontal Type Mask, Vertical Type Mask |
| 17 | Path Select | `A` | Path Selection, Direct Selection |
| 18 | Shape | `U` | Rectangle, Rounded Rectangle, Ellipse, Polygon, Line, Custom Shape |
| 19 | 3D Object† | `K` | Object Rotate, Object Roll, Object Pan, Object Slide, Object Scale |
| 20 | 3D Camera† | `N` | Camera Rotate, Camera Roll, Camera Pan, Camera Walk, Camera Zoom |
| 21 | Hand | `H` | Hand |
| 22 | Rotate View | `R` | Rotate View |
| 23 | Zoom | `Z` | Zoom |

### 2.2 Catalogue

| id (`tool.<id>`) | Label | Key | Grp | Slot | impl | Qt fallback | hotspot |
|---|---|---|---|---|---|---|---|
| `move` | Move | `V` | 1 | Move | ✓ | `Qt::SizeAllCursor` | (2,2) |
| `marquee` | Rectangular Marquee | `M` | 2 | Marquee | ✓ | `Qt::CrossCursor` | (12,12) |
| `ellipticalmarquee` | Elliptical Marquee | `M` | 2 | Marquee | | `Qt::CrossCursor` | (12,12) |
| `lasso` | Lasso | `L` | 3 | Lasso | ✓ | `Qt::CrossCursor` | (12,12) |
| `polygonallasso` | Polygonal Lasso | `L` | 3 | Lasso | | `Qt::CrossCursor` | (12,12) |
| `magneticlasso` | Magnetic Lasso | `L` | 3 | Lasso | | `Qt::CrossCursor` | (12,12) |
| `magicwand` | Magic Wand | `W` | 4 | Selection | | `Qt::CrossCursor` | (12,12) |
| `quickselection` | Quick Selection | `W` | 4 | Selection | ✓ | `Qt::CrossCursor` | (12,12) |
| `crop` | Crop | `C` | 5 | Crop / Slice | ✓ | `Qt::CrossCursor` | (12,12) |
| `perspectivecrop` | Perspective Crop | `C` | 5 | Crop / Slice | | `Qt::CrossCursor` | (12,12) |
| `slice` | Slice | `C` | 5 | Crop / Slice | | `Qt::CrossCursor` | (12,12) |
| `sliceselect` | Slice Select | `C` | 5 | Crop / Slice | | `Qt::CrossCursor` | (12,12) |
| `eyedropper` | Eyedropper | `I` | 6 | Sampling / Measure | ✓ | `Qt::CrossCursor` | (2,22) |
| `colorsampler` | Color Sampler | `I` | 6 | Sampling / Measure | | `Qt::CrossCursor` | (2,22) |
| `ruler` | Ruler | `I` | 6 | Sampling / Measure | | `Qt::CrossCursor` | (2,22) |
| `note` | Note | `I` | 6 | Sampling / Measure | | `Qt::CrossCursor` | (2,22) |
| `count` | Count (Extended) | `I` | 6 | Sampling / Measure | | `Qt::CrossCursor` | (12,12) |
| `spothealingbrush` | Spot Healing Brush | `J` | 7 | Retouch | | `Qt::CrossCursor` | (2,22) |
| `healingbrush` | Healing Brush | `J` | 7 | Retouch | | `Qt::CrossCursor` | (2,22) |
| `patch` | Patch | `J` | 7 | Retouch | | `Qt::CrossCursor` | (12,12) |
| `contentawaremove` | Content-Aware Move | `J` | 7 | Retouch | | `Qt::CrossCursor` | (12,12) |
| `redeye` | Red Eye | `J` | 7 | Retouch | | `Qt::CrossCursor` | (12,12) |
| `brush` | Brush | `B` | 8 | Paint | ✓ | `Qt::CrossCursor` | (2,22) |
| `pencil` | Pencil | `B` | 8 | Paint | ✓ | `Qt::CrossCursor` | (2,22) |
| `colorreplacement` | Color Replacement | `B` | 8 | Paint | | `Qt::CrossCursor` | (2,22) |
| `mixerbrush` | Mixer Brush | `B` | 8 | Paint | | `Qt::CrossCursor` | (2,22) |
| `clonestamp` | Clone Stamp | `S` | 9 | Clone | | `Qt::CrossCursor` | (2,22) |
| `patternstamp` | Pattern Stamp | `S` | 9 | Clone | | `Qt::CrossCursor` | (2,22) |
| `historybrush` | History Brush | `Y` | 10 | History | | `Qt::CrossCursor` | (2,22) |
| `arthistorybrush` | Art History Brush | `Y` | 10 | History | | `Qt::CrossCursor` | (2,22) |
| `eraser` | Eraser | `E` | 11 | Erase | | `Qt::CrossCursor` | (2,22) |
| `backgrounderaser` | Background Eraser | `E` | 11 | Erase | | `Qt::CrossCursor` | (2,22) |
| `magiceraser` | Magic Eraser | `E` | 11 | Erase | | `Qt::CrossCursor` | (2,22) |
| `gradient` | Gradient | `G` | 12 | Fill | | `Qt::CrossCursor` | (2,22) |
| `paintbucket` | Paint Bucket | `G` | 12 | Fill | | `Qt::CrossCursor` | (2,22) |
| `blur` | Blur | — | 13 | Blur | | `Qt::CrossCursor` | (2,22) |
| `sharpen` | Sharpen | — | 13 | Blur | | `Qt::CrossCursor` | (2,22) |
| `smudge` | Smudge | — | 13 | Blur | | `Qt::CrossCursor` | (2,22) |
| `dodge` | Dodge | `O` | 14 | Toning | | `Qt::CrossCursor` | (2,22) |
| `burn` | Burn | `O` | 14 | Toning | | `Qt::CrossCursor` | (2,22) |
| `sponge` | Sponge | `O` | 14 | Toning | | `Qt::CrossCursor` | (2,22) |
| `pen` | Pen | `P` | 15 | Pen | | `Qt::CrossCursor` | (2,2) |
| `freeformpen` | Freeform Pen | `P` | 15 | Pen | | `Qt::CrossCursor` | (2,2) |
| `addanchorpoint` | Add Anchor Point | — | 15 | Pen | | `Qt::CrossCursor` | (2,2) |
| `deleteanchorpoint` | Delete Anchor Point | — | 15 | Pen | | `Qt::CrossCursor` | (2,2) |
| `convertpoint` | Convert Point | — | 15 | Pen | | `Qt::CrossCursor` | (2,2) |
| `horizontaltype` | Horizontal Type | `T` | 16 | Type | | `Qt::IBeamCursor` | (12,12) |
| `verticaltype` | Vertical Type | `T` | 16 | Type | | `Qt::IBeamCursor` | (12,12) |
| `horizontaltypemask` | Horizontal Type Mask | `T` | 16 | Type | | `Qt::IBeamCursor` | (12,12) |
| `verticaltypemask` | Vertical Type Mask | `T` | 16 | Type | | `Qt::IBeamCursor` | (12,12) |
| `pathselection` | Path Selection | `A` | 17 | Path Select | | `Qt::CrossCursor` | (12,12) |
| `directselection` | Direct Selection | `A` | 17 | Path Select | | `Qt::CrossCursor` | (12,12) |
| `rectangle` | Rectangle | `U` | 18 | Shape | | `Qt::CrossCursor` | (12,12) |
| `roundedrectangle` | Rounded Rectangle | `U` | 18 | Shape | | `Qt::CrossCursor` | (12,12) |
| `ellipse` | Ellipse | `U` | 18 | Shape | | `Qt::CrossCursor` | (12,12) |
| `polygon` | Polygon | `U` | 18 | Shape | | `Qt::CrossCursor` | (12,12) |
| `line` | Line | `U` | 18 | Shape | | `Qt::CrossCursor` | (12,12) |
| `customshape` | Custom Shape | `U` | 18 | Shape | | `Qt::CrossCursor` | (12,12) |
| `objectrotate` | Object Rotate (Extended) | `K` | 19 | 3D Object | | `Qt::CrossCursor` | (12,12) |
| `objectroll` | Object Roll (Extended) | `K` | 19 | 3D Object | | `Qt::CrossCursor` | (12,12) |
| `objectpan` | Object Pan (Extended) | `K` | 19 | 3D Object | | `Qt::CrossCursor` | (12,12) |
| `objectslide` | Object Slide (Extended) | `K` | 19 | 3D Object | | `Qt::CrossCursor` | (12,12) |
| `objectscale` | Object Scale (Extended) | `K` | 19 | 3D Object | | `Qt::CrossCursor` | (12,12) |
| `camerarotate` | Camera Rotate (Extended) | `N` | 20 | 3D Camera | | `Qt::CrossCursor` | (12,12) |
| `cameraroll` | Camera Roll (Extended) | `N` | 20 | 3D Camera | | `Qt::CrossCursor` | (12,12) |
| `camerapan` | Camera Pan (Extended) | `N` | 20 | 3D Camera | | `Qt::CrossCursor` | (12,12) |
| `camerawalk` | Camera Walk (Extended) | `N` | 20 | 3D Camera | | `Qt::CrossCursor` | (12,12) |
| `camerazoom` | Camera Zoom (Extended) | `N` | 20 | 3D Camera | | `Qt::CrossCursor` | (12,12) |
| `hand` | Hand | `H` | 21 | Hand | ✓ | `Qt::OpenHandCursor` | (9,2) |
| `rotateview` | Rotate View | `R` | 22 | Rotate View | | `Qt::CrossCursor` | (12,12) |
| `zoom` | Zoom | `Z` | 23 | Zoom | ✓ | `Qt::CrossCursor` | (9,2) |

Total: **71 tools, 23 slots, 10 implemented**.

## 3. Hotspot rules

The cursor art for a tool MUST place its action point at the hotspot below. The
renderer (`icons.cpp`) looks the hotspot up by tool id; any id with no entry
falls back to the centre `(12,12)` (non-tool cursors do not exist today).

| Rule | Purpose | Hotspot | Tools |
|---|---|---|---|
| Centre | crosshair-style selection, crop, path, shape, 3D and rotate tools | `(12,12)` | Marquee; Lasso; Magic Wand; Quick Selection; Crop; Perspective Crop; Slice; Slice Select; Count; Patch; Content-Aware Move; Red Eye; Type; Path Selection; Direct Selection; Shape; 3D Object; 3D Camera; Rotate View |
| Pointer tip | the Move tool's action point is the compound cursor's arrowhead tip | `(2,2)` | Move |
| Lower-left tip | brush-like tools whose nib/point is the action point | `(2,22)` | Eyedropper; Color Sampler; Ruler; Note; Spot Healing Brush; Healing Brush; Brush; Pencil; Color Replacement; Mixer Brush; Clone Stamp; Pattern Stamp; History Brush; Art History Brush; Eraser; Background Eraser; Magic Eraser; Gradient; Paint Bucket; Blur; Sharpen; Smudge; Dodge; Burn; Sponge |
| Upper-left nib | pen tools whose nib points at the anchor | `(2,2)` | Pen; Freeform Pen; Add Anchor Point; Delete Anchor Point; Convert Point |
| Pointing finger | the fingertip / glass action point | `(9,2)` | Hand; Zoom |

The per-row values in §2.2 are authoritative; the table above is the rule
summary. Every hotspot MUST lie inside `[0, 24) × [0, 24)`.

## 4. Icon style guide

Match `assets/icons/tool.move.svg`:

- `width`/`height` omitted; `viewBox="0 0 24 24"` on the root `<svg>`.
- `fill="none"` on the root.
- `stroke="#c8c8c8"`, `stroke-width="1.5"`, `stroke-linecap="round"`,
  `stroke-linejoin="round"`.
- **One accent colour is allowed at most:** `#3d6f99` (the accent already used by
  `tool.eyedropper.svg`, `window.panels.layers.svg`, and
  `window.panels.tools.svg`). At most one element per icon may carry it; the
  rest are `#c8c8c8`. An icon may use zero accents.
- No `<text>` elements, no embedded raster, no scripts. Glyph-like marks (e.g.
  the Count numeral) are drawn as paths so no font is required.
- Legible at 16 px and 20 px; the 24 px grid is a design aid, not a render size.
- The 10 existing tool icons MUST NOT be redrawn by this milestone (a later
  visual pass may; behaviour is unchanged either way).

## 5. Cursor style guide

Match `assets/cursors/tool.move.svg`:

- `viewBox="0 0 24 24"`; same path art as the matching icon unless the cursor
  needs a distinct shape (e.g. the Hand is a hand, not the move cross; the Move
  cursor compounds an arrowhead top-left with the move cross bottom-right).
- Two stacked copies of the art:
  1. **Halo**: `<g fill="#ffffff" stroke="#ffffff" stroke-width="4">` — the
     white halo that keeps the cursor legible on a dark canvas.
  2. **Body**: `<g fill="#ffffff" stroke="#202020" stroke-width="1.5">` — white
     fill with a dark outline for light canvases.
- Cursors are **monochrome** (`#ffffff` / `#202020`); no accent colour, so the
  cursor never reads as image content.
- `stroke-linecap="round"`, `stroke-linejoin="round"` on the root.
- The art MUST be drawn so the tool's action point (§2.2/§3) is at the documented
  hotspot; the user does not re-calibrate.
- Legible on both the light grid and the dark canvas, and at 2× DPR (the
  renderer already renders at `devicePixelRatio`).

## 6. Toolbox UI design

CS6 single-column with flyout groups; the current two-column grid is replaced.

- **Single column, one button per slot.** 23 slot buttons top-to-bottom in §2.1
  order. Each is a checkable, auto-raising `QToolButton`, icon 20×20 on a 30×30
  button, with the slot's visible tool's icon.
- **Triangle for hidden tools.** A slot with more than one catalogue member
  shows a small triangle at its lower-right corner
  (`QToolButton::MenuButtonPopup`). Single-member slots show none.
- **Flyout.** Hold the mouse on the slot (or click the triangle) to open a
  `QMenu` listing every catalogue member, implemented or not; click one to pick
  it, which also makes it the slot's visible tool for the session.
- **`Alt`-click cycles** the slot's **enabled** members (unimplemented members
  are disabled and are skipped), wrapping deterministically. `Shift`+shortcut
  cycles the same set when `General > Use Shift Key For Tool Switch` is on (the
  default); with only one enabled member nothing cycles.
- **Visible tool.** The slot shows the last-used member (session state, default
  first implemented member: Paint shows Brush, Marquee shows Rectangular). The
  currently active tool's slot is checked.
- **Implemented tools** behave exactly as today: clicking selects, the shortcut
  activates, the checked state tracks `ToolController::activeToolChanged`, and
  the tool's icon/cursor/options bar are unchanged.
- **Unimplemented tools** are disabled, with the exact tooltip
  `<label> — not implemented yet` (the em dash plus one space each side). A slot
  whose members are all unimplemented is itself disabled with its default
  member's tooltip.
- **Shortcuts.** Each catalogue key is registered on the slot's enabled member
  only; pressing a disabled tool's key does nothing. The `B` slot keeps the
  Brush↔Pencil cycle behaviour.
- **Below the grid**, unchanged: the foreground/background swatch widget, then
  the Screen-Mode button (objectName `screenModeButton`).
- **Column toggle deferred.** The double-arrow 1-column/2-column toggle is not in
  this milestone; the toolbox stays single-column. (M23's two-column layout is
  superseded; see the `tool-framework` delta.)
- **No new tool engines.** Selecting an unimplemented tool is impossible, so the
  canvas pointer routing and options bar are untouched.

## 7. Panel icon inventory

Panel icon ids reuse the frozen `window.panels.<name>` command-id namespace so
the dock tab, the rail button, and the `Window > Panels` menu action all resolve
the **same** asset through `icon(id)`. `window.panels.layers` and
`window.panels.tools` already exist and are reused.

### 7.1 Panel icons

| Asset id | Panel | Hosted? | Consumed by |
|---|---|---|---|
| `window.panels.tools` | Tools (toolbox) | yes | Tools dock tab, `Window > Tools` action |
| `window.panels.layers` | Layers | yes | Layers dock tab, `Window > Layers` action |
| `window.panels.history` | History | yes | History dock tab, rail button, `Window > History` |
| `window.panels.actions` | Actions | yes | Actions dock tab, rail button, `Window > Actions` |
| `window.panels.info` | Info | yes | Info dock tab, rail button, `Window > Info` |
| `window.panels.navigator` | Navigator | yes | Navigator dock tab, rail button, `Window > Navigator` |
| `window.panels.histogram` | Histogram | yes | Histogram dock tab, rail button, `Window > Histogram` |
| `window.panels.color` | Color | yes | Color dock tab, `Window > Color` |
| `window.panels.swatches` | Swatches | yes | Swatches dock tab, `Window > Swatches` |
| `window.panels.gradients` | Gradients | yes | Gradients dock tab, `Window > Gradients` |
| `window.panels.patterns` | Patterns | yes | Patterns dock tab, `Window > Patterns` |
| `window.panels.properties` | Properties | yes | Properties dock tab, `Window > Properties` |
| `window.panels.adjustments` | Adjustments | yes | Adjustments dock tab, `Window > Adjustments` |
| `window.panels.libraries` | Libraries | yes | Libraries dock tab, `Window > Libraries` |
| `window.panels.channels` | Channels | yes | Channels dock tab, `Window > Channels` |
| `window.panels.paths` | Paths | yes | Paths dock tab, `Window > Paths` |
| `window.panels.brushes` | Brushes | no | reserved (dock tab when hosted) |
| `window.panels.toolPresets` | Tool Presets | no | reserved |
| `window.panels.cloneSource` | Clone Source | no | reserved |
| `window.panels.measurementLog` | Measurement Log | no | reserved |
| `window.panels.notes` | Notes | no | reserved |
| `window.panels.character` | Character | no | reserved |
| `window.panels.paragraph` | Paragraph | no | reserved |
| `window.panels.typeStyles` | Type Styles | no | reserved |
| `window.panels.smartObjects` | Smart Objects | no | reserved |
| `window.panels.masks` | Masks | no | reserved |
| `window.panels.3d` | 3D | no | reserved |
| `window.panels.timeline` | Timeline | no | reserved |

The rail hosts five buttons today — History, Actions, Info, Navigator, Histogram
— each of which MUST render its panel icon instead of the current text glyph.

### 7.2 Layers-panel action strip

The CS6 bottom strip is seven buttons, left to right. Icons are added now; the
buttons themselves land with the layers-panel stages.

| Asset id | Button | Consumed by | Status |
|---|---|---|---|
| `layers.link` | Link Layers | Layers strip | icon now; button deferred (M41 link sets) |
| `layers.fx` | Layer Style (fx) | Layers strip | icon now; button deferred (M42 styles) |
| `layers.mask` | Add Layer Mask | Layers strip | icon now; button deferred (M39/M43 masks) |
| `layers.fillAdjustment` | New Fill / Adjustment Layer | Layers strip (Add Adjustment button) | icon now; button exists |
| `layers.group` | New Group | Layers strip (New Group button) | icon now; button exists |
| `layers.newLayer` | New Layer | Layers strip (New Layer button) | icon now; button exists |
| `layers.delete` | Delete | Layers strip (Delete Layer button) | icon now; button exists |

The app's extra Move Up / Move Down buttons keep their text labels; they are not
part of the CS6 seven and get no new asset.

### 7.3 History panel

| Asset id | Button | Consumed by |
|---|---|---|
| `history.snapshot` | Create New Snapshot | History panel's `Create Snapshot` button |

## 8. Verification approach

One C++ self-test step (`m38_icons`) is the runnable check; it fails the build's
self-test when any asset is missing or malformed.

1. **Catalogue completeness.** Iterate the frozen catalogue (the table in
   `tools.cpp`). For every entry assert `icon("tool." + id)` is non-null,
   `cursor("tool." + id)` is non-null, and its hotspot is defined and inside
   `[0,24)²`. Assert the entry count is 71 and the implemented count is 10.
2. **Every SVG parses.** Enumerate `:/icons/` and `:/cursors/` with
   `QDir::entryList`, load each with `QSvgRenderer`, and assert `isValid()`.
   (A file in the qrc that is not valid SVG fails here.)
3. **Toolbox shows the catalogue.** Build the `Toolbox`; assert it has one slot
   per group (23), each slot's flyout menu lists exactly its catalogue members,
   and the total menu entries equal 71.
4. **Implemented enabled, unimplemented disabled.** For each catalogue entry
   assert the corresponding button/menu action is enabled iff implemented; for
   each unimplemented one assert the tooltip is exactly
   `<label> — not implemented yet`.
5. **Panel icons resolve.** For every id in §7.1 assert `icon(id)` is non-null;
   for every id in §7.2 and §7.3 assert `icon(id)` is non-null; assert the rail
   has five buttons and each carries a non-null icon.
6. **Report.** Print
   `pictura self-test: m38_icons tools=71 icons=71 cursors=71 hotspots=71 svg=1 groups=23 implemented=10 disabled=61 panels=N strip=7 snapshot=1`
   and exit non-zero with fresh codes on the first failure. Run under
   `xvfb-run -a ./build/pictura --self-test`.

The existing M19 checks (all bundled icons/cursors resolve; unknown ids are
null) stay.

## 9. qrc generation approach

`assets/pictura.qrc` stays checked in (CMake is not touched). The asset section
of `tasks.md` regenerates it with a **`scripts/gen_qrc.sh`** that globs
`assets/icons/*.svg` and `assets/cursors/*.svg`, sorts the list, and rewrites
the `<qresource>` block (relative paths, no other changes). The script is a few
lines of POSIX `sh`; no new dependency, no build-system change. The `m38_icons`
self-test is the guard: a catalogue id with no qrc entry fails step 1–2, so a
stale qrc cannot ship. Hand-editing the qrc is the fallback but is discouraged at
~180 entries.

## 10. Source conflicts and open questions

- **Blur / Sharpen / Smudge** are a single unlettered slot. The CS6 tool table
  gives them no key (reachable only by flyout or `Alt`-click), matching
  `toolbox-and-options-bar.md`; one secondary source implies `R` may reach them,
  which is rejected. The catalogue uses no key.
- **Dodge / Burn / Sponge** are `O`. `O` is also Overlay's blend-mode `Shift+Alt`
  key, which is not a tool-shortcut collision.
- **Brush / Pencil** both keep `B` (M21 behaviour); with the whole Paint slot on
  `B`, `Shift+B` cycles only the two implemented members.
- **Count** is Extended-only and shares `I`; in Standard it should be treated as
  absent, not merely disabled. The catalogue keeps it with the Extended marker
  and lets the Standard/Extended edition gate its slot membership.
- **3D tools** (groups 19–20) are Extended-only; Standard hides the two slots.
- **Zoom hotspot.** CS6's Zoom cursor is a magnifier whose action point is often
  described at the lens centre; this contract follows the task's rule and puts
  Zoom at the pointing finger `(9,2)`. If the art reads better with the lens
  centre, the per-row value is the one to revisit.
- **Existing 10 icons.** `assets/icons/tool.move.svg` etc. are not redrawn; the
  new style guide applies to the 61 new tool icons and the panel/strip icons.
