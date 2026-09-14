# Toolbox and Options Bar

- **Spec ID:** `UI-004`
- **Status:** `Draft`
- **Parity tier:** `Core`
- **New in CS6:** `Changed` — the Line and Shape tools became fully vector-based (stroke/fill via the options bar); brush/erodible-tip and static-tip projection additions; maximum brush size raised to 5,000 px; the brush HUD can also change opacity; tool presets stay reorganized after restart; new document presets; the Tools panel is single/double column with flyout groups (unchanged in mechanism from CS5).
- **Depends on:** `UI-001` application-frame, `UI-002` menus, `UI-003` workspace-and-docks, `02-ui-ux/keyboard-shortcuts.md`, `03-tools/*`, `07-color-painting/brush-presets.md`

> All module and widget names below are **design proposals**. No code exists in
> this repository. Tool names, groups and shortcuts come from the fetched CS6
> Help reference; anything else is marked *(inferred)* or `(to verify)`.

## CS6 behavior

### The Tools panel (toolbox)

- Starts docked at the **left of the screen**. Some tools have options that
  appear in the context-sensitive **options bar**.
- Tools can be shown in **one column** or side by side in **two columns**; toggle
  by clicking the **double arrow at the top of the Tools panel** (also exposed as
  a Layout control historically on the Application bar).
- **Hidden tools / flyouts:** a small **triangle at a tool's lower-right corner**
  means hidden tools are available. Hold the mouse button on the tool to reveal
  them, then click one. Alternatively `Alt`/`Option`-click the tool to cycle the
  hidden tools (except Add Anchor Point, Delete Anchor Point, Convert Point).
- **Tool tips** show the tool name below the pointer; the tool's **keyboard
  shortcut** is shown in the tool tip. With `Interface > Show Tool Tips` off,
  tool tips are hidden.
- **Temporary tool switch:** pressing and holding a tool shortcut switches to that
  tool only while held; releasing returns to the previous tool.
- **Cycle hidden tools:** by default `Shift` + repeatedly pressing the shortcut
  cycles the group. Disable `General > Use Shift Key For Tool Switch` to cycle
  without `Shift`.
- **Tool pointer** is configurable per `Cursors` preference: Standard, Precise,
  Normal Brush Tip, Full Size Brush Tip, Show Crosshair In Brush Tip, Show Only
  Crosshair While Painting. `Caps Lock` toggles standard/precise for some tools.
  Painting-cursor size/hardness can be adjusted on-canvas with `Alt`+right-drag
  (`Ctrl`+`Option` on Mac).
- At the bottom of the Tools panel are the **foreground/background colour**
  controls, the **Edit in Quick Mask Mode** toggle, and (in CS6) the
  **Screen Mode** button.

### Tool groups and verified shortcuts

From the CS6 Help "Keys for selecting tools" table. In a row with multiple
tools, repeatedly press the shortcut (with `Shift` by default) to cycle.

| Group (flyout) | Tools | Shortcut | Notes |
|---|---|---|---|
| Move | Move | `V` | |
| Marquee | Rectangular, Elliptical | `M` | Help marks `†` (same key used in Liquify) |
| Lasso | Lasso, Polygonal, Magnetic | `L` | |
| Selection | Magic Wand, Quick Selection | `W` | |
| Crop / Slice | Crop, Slice, Slice Select | `C` | |
| Sampling / Measure | Eyedropper, Color Sampler, Ruler, Note, Count | `I` | `†`; Count is Extended-only |
| Retouch | Spot Healing Brush, Healing Brush, Patch, Red Eye | `J` | |
| Paint | Brush, Pencil, Color Replacement, Mixer Brush | `B` | |
| Clone | Clone Stamp, Pattern Stamp | `S` | |
| History | History Brush, Art History Brush | `Y` | |
| Erase | Eraser, Background Eraser, Magic Eraser | `E` | `†` |
| Fill | Gradient, Paint Bucket | `G` | |
| Toning | Dodge, Burn, Sponge | `O` | |
| Pen | Pen, Freeform Pen | `P` | + Add/Delete Anchor Point, Convert Point |
| Path select | Path Selection, Direct Selection | `A` | |
| Type | Horizontal Type, Vertical Type, Horizontal Type Mask, Vertical Type Mask | `T` | |
| Shape | Rectangle, Rounded Rectangle, Ellipse, Polygon, Line, Custom Shape | `U` | CS6: fully vector |
| 3D object | Object Rotate, Roll, Pan, Slide, Scale | `K` | Extended only |
| 3D camera | Camera Rotate, Roll, Pan, Walk, Zoom | `N` | Extended only |
| Navigate | Hand | `H` | `†` |
| Rotate | Rotate View | `R` | |
| Navigate | Zoom | `Z` | `†` |

`†` Hand and Zoom share `H`/`Z` with the Liquify dialog's own keys.

The Help's galleries also name the same tools by role (Move; Marquee; Lasso;
Quick Selection; Magic Wand; Crop; Slice; Spot Healing Brush; Healing Brush;
Patch; Red Eye; Clone Stamp; Pattern Stamp; Eraser; Background Eraser; Magic
Eraser; Blur; Sharpen; Smudge; Dodge; Burn; Sponge; Brush; Pencil; Color
Replacement; Mixer Brush; History Brush; Art History Brush; Gradient; Paint
Bucket; Path Selection; Type; Type mask; Pen; Shape; Line; Custom Shape; Hand;
Rotate View; Zoom; Note; Eyedropper; Color Sampler; Ruler; Count; 3D object and
camera tools).

Note: the CS6 Help gallery also lists **Blur, Sharpen, Smudge** in the
retouching group — they share the flyout with other retouch tools but the Help's
tool-shortcut table does not assign them a unique letter; their placement is
`(to verify)`.

### The options bar

- Appears **below the menu bar** at the top of the workspace. It is
  **context-sensitive**: it changes with the selected tool. Some settings
  (painting mode, opacity) are shared by several tools; some are tool-specific.
- Can be moved using its **gripper bar** and docked at the top or bottom of the
  screen. Show/hide with `Window > Options`.
- **Reset a tool:** right-click (`Ctrl`-click on Mac) the tool icon in the
  options bar, then choose `Reset Tool` or `Reset All Tools`.
- CS6-specific option changes sourced from the Help: vector shape tools expose
  **stroke/fill/dashed-line** controls plus `Align Edges` and `Constrain Path
  Dragging`; the Eyedropper Sample menu gained "current layer and below" and
  "ignore adjustment layers"; Marquee/Lasso/Mask **feather accepts decimal
  values** (CS6); `Free Transform` gained an **Interpolation** menu; the brush
  HUD can change **opacity** with `Ctrl+Alt`+drag.

### Tool presets

- A **tool preset** saves a tool's options for reuse. Access via the **Tool
  Preset picker** in the options bar (leftmost), the **Tool Presets panel**
  (`Window > Tool Presets`), or the Preset Manager.
- The picker's **Current Tool Only** checkbox filters to the active tool;
  deselect to show all presets.
- **Create:** set options → click the Tool Preset button / `Window > Tool
  Presets` → **Create New Tool Preset** → name → OK.
- **Manage:** panel menu offers Show All Tool Presets, Sort By Tool,
  Show Current Tool Presets, Text Only / Small List / Large List, and
  Rename/Delete Tool Preset, Reset/Load/Replace/Save Tool Presets. `Alt`/`Option`
  -click an item deletes it.
- CS6: reorganized tool presets **remain after restart**; the SDK can read the
  tool name associated with a preset.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| Tools panel | Dock (left) | tool letters | Single/double column |
| Double arrow (top of Tools) | Toggle | n/a | Switch 1- vs 2-column |
| Hidden-tool triangle | Flyout | hold click / `Alt`-click | Cycle with `Shift`+shortcut |
| Options bar | Tool bar | `Window > Options` | Context-sensitive; gripper; top/bottom dock |
| Tool icon in options bar | Context menu | right-click (`Ctrl`-click) | `Reset Tool`, `Reset All Tools` |
| Tool Preset picker | Popup | n/a | Current Tool Only checkbox |
| Tool Presets panel | Dock | `Window > Tool Presets` | Create/manage presets |
| Foreground/Background colour | Swatch | `X` swap, `D` defaults | Bottom of Tools panel |
| Quick Mask toggle | Button | `Q` | Bottom of Tools panel |
| Screen Mode button | Button | `F` / `Shift+F` | Bottom of Tools panel (CS6) |
| Tool tip | Overlay | n/a | Name + shortcut; `Show Tool Tips` |
| Painting cursor resize | On-canvas | `Alt`+right-drag | Size (H/V: hardness; `Ctrl+Alt` opacity) |

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Column layout | enum | 2-column `(to verify)` | 1 / 2 columns | Double-arrow toggle |
| Tool tooltip | bool | On | on / off | Interface preferences |
| Use Shift Key For Tool Switch | bool | On | on / off | General preferences |
| Show Tool Tips | bool | On | on / off | Interface preferences |
| Cursor style | enum | Standard | Standard / Precise | Cursors preferences |
| Painting cursor | enum | Normal Brush Tip | Normal / Full Size / Crosshair variants | Cursors preferences |
| Max brush size | px | 5,000 (CS6) | 1–5,000 | Increased in CS6 |
| Feather value | decimal | 0 | 0–? | CS6 decimal support |
| Shape stroke/fill | control set | — | colour / gradient / pattern; dashed | CS6 vector shape options |
| Tool preset list | enum | default library | Text Only / Small / Large | Per panel menu |
| Current Tool Only | bool | Off | on / off | Tool Preset picker |

## Algorithms & pipeline

Design proposal.

- **Tools panel = icon grid model.** Each tool is a `ToolId` with `{icon,
  shortcut, group, flyoutIndex, cursorDefaults, optionsFactory}`. Rendering is a
  vertical grid (1 or 2 columns) of `QToolButton`s with a flyout (`QMenu`) where
  `group` has >1 member and a corner triangle.
- **Single active tool.** A `ToolController` owns `active_tool: ToolId` and a
  `temporary_tool: Option<ToolId>` stack for spring-loaded switching (hold
  shortcut; release restores). Esc cancels a drag without changing the tool.
- **Cycle logic.** `Shift`+shortcut (when `Use Shift Key For Tool Switch`) or
  `Alt`-click advances the group; cycle wraps and is deterministic.
- **Options bar = per-tool widget factory.** Each `ToolId` supplies an
  `OptionsWidget` (Qt widget) built from a declarative option schema so
  common controls (mode, opacity, feather) can be reused. Rebuilding on tool
  change must preserve focus-safe state and be cheap (`QStackedWidget` of cached
  option widgets rather than reconstructing each time *(inferred optimisation)*).
- **Reset.** `Reset Tool` restores the factory options for the active tool;
  `Reset All Tools` restores every tool. Presets are not deleted by reset.
- **Tool presets.** A preset is `{ToolId, options snapshot, name}`. The picker
  reads from a `ToolPresetStore` with a `current_tool_only` filter; `Alt`-click
  deletes; load/replace/save use `.tpl`-style libraries managed by the common
  Preset Manager.
- **Screen-mode integration.** The Tools-panel Screen Mode button and the `F`
  key drive the same `ScreenModeController` as `UI-001`.

## Rust module mapping

Proposals.

- `pictura_tools::registry` — `ToolId`, `ToolGroup`, `ToolDescriptor`
  (`icon`, `shortcut`, `options_schema`, `cursor_defaults`), built-in table.
- `pictura_tools::controller` — `ToolController` (active/temporary tool,
  spring-load stack, cycle/reset).
- `pictura_tools::options` — option schemas and value structs per tool group
  (common: `mode`, `opacity`, `flow`, `feather`; shape: `fill`, `stroke`,
  `align_edges`, `constrain_path_dragging`).
- `pictura_tools::presets` — `ToolPreset { tool, name, values }`,
  `ToolPresetStore`, library load/save/replace, `current_tool_only` filter.
- `pictura_tools::cursors` — cursor style resolution and on-canvas size/hardness
  HUD math.

Crossing types: `ToolId`, `ToolPreset`, small option value structs, `f32`
brush size/hardness/opacity. The Tools panel itself is Qt and consumes this via
the bridge; no pixels cross.

## Qt6 component mapping

| Proposal | Base | Responsibility |
|---|---|---|
| `ToolsPanel` | `QWidget` dock | Icon grid (1/2 col), flyout triangles, colour swatches, Quick Mask + Screen Mode buttons |
| `ToolButton` | `QToolButton` | Icon + `QMenu` flyout; `InstantPopup`/`DelayedPopup` per group |
| `ToolController` | `QObject` | Active/temporary tool, shortcut and cycle handling, reset |
| `OptionsBar` | `QToolBar` | Gripper, top/bottom docking, `QStackedWidget` of cached `OptionsWidget`s |
| `OptionsWidget` (per tool) | `QWidget` | Declarative controls; shared sub-widgets for mode/opacity/feather |
| `ToolPresetPicker` | `QToolButton` + `QMenu` | Popup preset list, Current Tool Only |
| `ToolPresetsPanel` | `QWidget` dock | Full preset management |

Tool buttons need `setPopupMode(QToolButton::MenuButtonPopup)` to show the
corner triangle for groups. The options bar is a `QToolBar` so `QMainWindow`
already handles gripper and top/bottom docking; a `QStackedWidget` switches the
active tool's option widget.

## Data-model impact

- **Tool state is session state**, not document data: active tool, temporary
  tool, column layout, options-bar position, last-used options, tool presets.
  Persisted in the preference/session store; never in PSD/XMP.
- **Tool selection produces no history state.** Tool *actions* produce history
  via `ARCH-007`; merely choosing a tool does not.
- **Options feed commands.** Tool option values are inputs to the command layer;
  a preset is a saved options snapshot, not a document node.
- **Shortcut mapping** of tool letters feeds `02-ui-ux/keyboard-shortcuts.md` and the Keyboard Shortcuts
  dialog (`UI-002`).
- CS6 persists reorganized tool presets across restarts; our preset store must
  therefore be durable, with library load/save via the Preset Manager.

## Edge cases

- **Standard vs Extended.** 3D tools (`K`, `N`) are absent in Standard; the
  Tools panel and shortcut table must reflect the edition.
- **No flyout room.** In 1-column narrow layouts the flyout menu must still open
  on-screen; clamp to screen geometry.
- **Shortcut collision.** A tool letter that clashes with a menu or custom
  shortcut must resolve deterministically and warn in the Keyboard Shortcuts
  dialog.
- **Spring-loaded + focus loss.** Releasing a held shortcut after the window
  loses focus must not leave the temporary tool active.
- **Options rebuild cost.** Switching tools rapidly must not flicker or leak;
  cache per-tool option widgets.
- **GPU-unavailable cursor previews.** On-canvas brush-size previews "require
  OpenGL"; without a GPU the cursor must degrade to a non-preview form rather
  than fail.
- **Huge brushes.** 5,000 px brushes at high zoom must not stall the HUD or the
  cursor outline.
- **Preset library missing/corrupt.** Fall back to the default library and keep
  user presets.
- **Unicode/localised tool names.** Tool tips and the presets panel must use the
  translation catalogue; ids stay stable.
- **Accessibility.** Icon-only tool buttons need accessible names, focus order
  and keyboard reachability (`02-ui-ux/accessibility.md`).

## Parity acceptance criteria

- Given the Tools panel, clicking the top double arrow toggles between one and
  two columns and the choice survives restart.
- Given a tool with hidden tools, holding the mouse reveals the flyout; `Alt`
  -click cycles the group; `Shift`+shortcut cycles when `Use Shift Key For Tool
  Switch` is on and not when it is off.
- Given a tool shortcut is pressed and held, the temporary tool is active;
  releasing returns to the previous tool, including after an Esc-cancelled drag.
- Given each verified tool, pressing its shortcut (row above) selects it; the
  tool tip shows the name and shortcut.
- Given a tool is selected, the options bar shows that tool's controls; moving it
  by the gripper and docking top/bottom works; `Window > Options` hides/shows it.
- Given right-click on the options-bar tool icon, `Reset Tool` restores the
  active tool's defaults and `Reset All Tools` restores every tool.
- Given a created tool preset with Current Tool Only on, only that tool's presets
  appear; with it off, all presets appear; `Alt`-click deletes a preset.
- Given reorganized presets, restart the app and the order persists (CS6
  behavior).
- Given Standard edition, 3D tools are absent; given Extended, `K`/`N` groups
  appear.
- Given the Eyedropper Sample menu, CS6 options "current layer and below" and
  "ignore adjustment layers" are present.
- Given a vector Shape tool, the options bar exposes fill/stroke/dashed controls
  and `Align Edges`/`Constrain Path Dragging`.
- Given `devicePixelRatio = 2`, tool icons render crisp and the flyout triangles
  are correctly placed.

## Sources

Fetched for this document:

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` —
  official Photoshop CS6 Help reference (downloaded and text-extracted).
  Establishes: Tools panel location and single/double-column toggle via the
  double arrow; hidden tools and the corner triangle; tool tips and shortcuts;
  temporary tool switching; `Use Shift Key For Tool Switch`; cursor settings
  (Standard/Precise/Normal Brush Tip/Full Size/Crosshairs/Caps Lock, on-canvas
  size & hardness); the complete tool gallery (selection, crop/slice,
  retouching, painting, drawing/type, navigation/notes/measure, 3D); the tool
  shortcut table (`V M L W C I J B S Y E G O P A T U K N H R Z`); the options bar
  description (context-sensitive, gripper, top/bottom docking,
  `Window > Options`) and `Reset Tool`/`Reset All Tools`; tool presets (picker,
  Current Tool Only, create/manage, Text/Small/Large, Alt-delete); CS6 changes
  (vector shapes with stroke/fill/`Align Edges`/`Constrain Path Dragging`,
  decimal feather, Free Transform interpolation, Eyedropper sample options,
  brush HUD opacity, 5,000 px max brush, reorganized presets persisting).
- `https://itwiki.wpunj.edu/images/e/ee/Photoshop_CS6_Extended_-_Dacier.pdf` —
  secondary student tutorial; corroborates the Tools-palette location, flyout
  access by holding the mouse, and the foreground/background colour controls.
  Secondary source.
- SearXNG meta-search queries used to locate secondary sources. No facts taken
  from snippets alone.

Not parsed: `help.adobe.com` (HTTP 403 for `helpx.adobe.com`; the archive PDF was
downloaded instead).

## Open questions

- **Blur/Sharpen/Smudge flyout placement and shortcuts.** The Help galleries
  list them but the tool-shortcut table does not give them letters. Resolve with
  a CS6 Tools-panel capture.
- **Default column layout.** Whether CS6 defaults to one or two columns is
  inferred. Resolve with a first-run capture.
- **Exact flyout composition and order per group.** The set of tools per group is
  largely sourced; the order within each flyout is `(to verify)`.
- **Full options-bar control set per tool.** This spec covers the documented
  shared/CS6-changing controls; a per-tool exhaustive control table belongs in
  `03-tools/*` and is not fully sourced here.
- **Brush HUD modifier mapping.** CS6 documents `Ctrl+Alt`+drag for opacity and
  `Alt`+right-drag for size/hardness; the exact axis/behaviour matrix needs a
  capture.
- **Tool preset file format.** Whether CS6 `.tpl` libraries are interoperable is
  unverified; we propose our own format.
- **Accessibility names/roles for tool buttons.** Unspecified; resolve in
  `02-ui-ux/accessibility.md`.
- **`Design` vs. `Typography` workspace** affects which panels a tool preset may
  be expected to accompany. See `UI-003` Open questions.
