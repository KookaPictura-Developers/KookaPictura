## Why

A prior change (`fix-app-ui-issues`, archived) fixed seventeen app-UI defects, but
a follow-up report shows sixteen remaining or incorrect behaviors, each with a
root cause confirmed by code recon. They are small and mostly local — a missing
model virtual, a wrong suffix, a global where a per-column value is needed — but
several contradict what the just-merged specs now say. The intervening specs are
therefore wrong in three places (tab extension, Background locks, scrollbar
hiding) and must be corrected before any code lands. This change encodes the
sixteen fixes as five independently committable, spec-backed batches so the
interactions can be implemented without re-litigating the investigation.

## What Changes

- **Persistence and import identity (items 1, 2, 3).** Each panel column stores
  its own `normal`/`iconic` rail mode (schema v7 → v8, global `panelRailMode`
  kept as the legacy seed). A tab shows the file name *with its extension*, so it
  reads `image01.jpg (RGB/8)`. An imported opaque image becomes a Background with
  `TRANSPARENCY | POSITION` (`0x05`) locks, not every lock.
- **Canvas view (items 4, 14).** Both scrollbars are always visible; the canvas
  is freely pannable and the bars are a projection of `offset_`/`zoom_`. Holding
  `Space` temporarily switches to the Hand tool with an open-hand cursor and pans
  on drag, restoring the previous tool, cursor, and pan state on release.
- **Layers panel (items 5, 6, 7, 8).** The layer row model advertises drag/drop
  (`mimeTypes`, `canDropMimeData`, `supportedDropActions`) and the view enters the
  dragging state so sibling reorders resolve at the drop. A Background converts
  to a normal layer on a double-click outside the name and when dropped on the
  New Layer button. `Ctrl`-clicking a row thumbnail selects that layer's pixels
  from its alpha channel. Nesting-locked groups refuse drag-and-drop into or out
  of them.
- **Active-layer gating (items 9, 10).** One resolver in `pictura-app` names the
  exactly-one active layer a tool edits; the panel pushes selection, an import
  auto-activates its layer, and zero or multiple selection refuses the edit. The
  transparency lock becomes per-pixel alpha-preserving rather than a blanket
  refusal.
- **Cursor, ring, and hints (items 11, 12, 15).** Brush/Pencil hide the mouse
  cursor and rely on the drawn ring, which now renders outside the canvas clip.
  The status bar hosts a per-tool keycap hint strip (Shift = Add, Alt = Subtract
  for selection tools) that highlights a pressed key.
- **Numeric fields (item 16).** Hardness/Opacity/Flow show `%`, Feather opens no
  slider popup, and the `px` suffix loses its leading space.
- **Paint performance (item 13).** Per-dab incremental dirty, a true region CPU
  compositor, and a present-cache region update keep a 4000² brush at the
  canvas-view budget.
- **BREAKING (item 2):** the `document-tabs` and `image-import` requirements that
  say an opened raster uses its *base name* are wrong; the tab MUST use the file
  name **and extension**.
- **BREAKING (item 3):** the `image-import` requirement that an opaque import is
  `LockFlags::all()` is wrong; the Background lock set is `TRANSPARENCY |
  POSITION` (`0x05`).
- **BREAKING (item 4):** the `canvas-scrollbars` requirement "Scrollbars hide
  when the document fits" is removed; both bars are always visible.

## Capabilities

### New Capabilities

- `tool-hint-bar`: a bottom status-bar strip of context keycaps for the active
  tool, with modifier hints (Shift = Add, Alt = Subtract for selection tools) and
  a pressed-key highlight, sourced from the command registry and tool shortcuts.

### Modified Capabilities

- `panel-column`: each column records and restores its own rail mode, with the
  global `panelRailMode` as the legacy seed.
- `workspace-persistence`: schema v8 adds a per-column `railMode` and restores it
  per column.
- `document-tabs`: the tab title uses the file name including its extension.
- `image-import`: the display name includes the extension; an opaque import is a
  Background locked `TRANSPARENCY | POSITION`.
- `canvas-scrollbars`: both bars are always visible instead of hiding when the
  document fits.
- `layers-panel`: the model advertises drag/drop and the view sets the dragging
  state; Background converts via double-click and New-Layer drop; a thumbnail
  `Ctrl`-click selects layer pixels.
- `layer-management`: Background conversion is reachable from the panel
  affordances.
- `selection-tools`: a doc-sized selection can be built from a layer's alpha
  channel.
- `layer-locks`: per-pixel transparency enforcement and nesting-lock reparenting
  refusal.
- `brush-tools`: paint targets the active layer, the ring draws outside the
  canvas clip, and Brush/Pencil hide the OS cursor.
- `filter-application`: filters target the active layer and honor the
  transparency lock per pixel.
- `free-transform`: Free Transform resolves the active layer.
- `tool-framework`: one active-layer resolver for tool edits; the status bar
  hosts the hint bar; brush tools use a blank cursor.
- `paint-engine`: incremental dirty regions and region compositing meet the
  canvas-view dab budget.
- `canvas-tools`: Space temporarily activates Hand panning.
- `numeric-fields`: options-bar suffix, popup, and spacing configuration.

## Impact

- **App C++ plus targeted Rust.** No new crate or dependency.
  `CMakeLists.txt` gains the new `selftest_*.cpp` files explicitly (no globbing).
- `crates/pictura-app/cpp/`: `session.{h,cpp}`, `frame_session.cpp`,
  `frame.cpp`, `image_view.{h,cpp}`, `tools.cpp`, `layers_panel.cpp`,
  `layers_panel_internal.h`, `layers_panel_actions.cpp`, `options_bar.cpp`,
  `canvas_scrollbars.cpp`, `frame_build.cpp`, and the new hint-bar widget home.
- `crates/pictura-app/src/cxxqt_object/`: `impl_core.rs`, `impl_paint.rs`,
  `impl_filters.rs`, `impl_transform.rs`, `impl_selection.rs`, `impl_layers_merge.rs`,
  and the shared active-layer resolver.
- `crates/pictura-render/`: region CPU compositor (`gpu/mod.rs`), per-pixel
  transparency-lock write path, and the layer-ops nesting guard.
- `crates/pictura-paint/`: per-dab incremental dirty in `stroke.rs`.
- `crates/pictura-select/`: alpha-to-selection construction is reused, not
  duplicated.
- `crates/pictura-app/cpp/selftest*.cpp`: one new file per batch starting at
  failure code **324** (highest in use is **323**); `selftest.cpp` (6729 LOC, at
  its 6730 allowlist ceiling) does not grow.
- `openspec/specs/`: one archived capability for `tool-hint-bar` and MODIFIED /
  ADDED / REMOVED deltas for the listed capabilities. No `docs/` change.
