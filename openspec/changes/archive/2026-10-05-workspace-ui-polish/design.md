# Design

## Context

See proposal.md for motivation. The shell is one C++/Qt6 layer
(`crates/pictura-app/cpp`) driven by a single `Theme` unit
(`theme.cpp`/`theme.h`): `kRamps` + `paletteFor` feed both the palette and the
generated QSS tokens (`${window}`, `${base}`, `${button}`, `${border}`,
`${hover}`, `${pressed}`, `${panelHeader}`). Panel chrome is
`panels/panel_group.*` and `panels/panel_column*`; the toolbox is `toolbox.cpp`
+ `tool_catalog.cpp`; the layers delegate is `panels/layers_panel_internal.h`.
The current polish is applied per-site as literals, which is why surfaces drift
and separators disagree. The C++ self-test is a hand-rolled oracle whose checks
only shrink; new GUI checks belong in the Qt Test suites.

## Goals / Non-Goals

**Goals:**

- One place for the cross-cutting values (button states, separator thickness,
  surface shade steps, tool-slot metrics), consumed by both the palette and the
  stylesheet.
- DPI-correct tool slots without double scaling.
- Fix the four interaction bugs at their shared roots: self-anchor docking,
  first-open menu geometry, side flyouts, one-click tool switching.

**Non-Goals:**

- No Quick Mask engine or overlay (Paint Mask is a button stub).
- No new dependency, no data model, no persistence format change.

## Decisions

### Theme owns the new metrics and shades

Add to `Theme` (next to `kPanelBorderWidth` / `kGroupDividerWidth`):

- `kChromeBorderWidth = 2` (chrome lines) and keep `kPanelBorderWidth = 1` for
  in-body lines. The 2 px chrome set is the menu bar bottom, options bar bottom,
  status bar top, icon-group grip, `QMenu::separator`, and `QToolBar::separator`;
  the widget-column header/bottom rules are 3 px from `ui/panel-column`'s shared
  constant. Replace the literal chrome sites listed in proposal scope with these
  constants; the Layers filter bar edges, Info-grid cross, and eye-gutter stay at
  the 1 px in-body width.
- `kShadeStep` and a `shade(color, ±1)` helper. **One step** is quantified as
  `QColor::lighter(112)` / `darker(112)` on the base surface — a single subtle
  increment matching the existing `${hover}` (lighter 120) / `${pressed}`
  (darker 120) vocabulary, so panel/bar/widget/group become `shade(window, +1)`
  and the workspace/empty pane `shade(window, -1)`, resolved at the current
  brightness level and re-applied on every brightness change through
  `styleSheetFor`. No per-site literals.
- Tool-slot base metrics in one place: `kSlotBaseW = 36`, `kSlotBaseH = 28`,
  `kSlotIconMaxW = 24`, `kSlotIconMaxH = 20` (96 DPI). The idle body of a slot
  that draws no outline is derived from the footprint minus twice the 1 px
  `kPanelBorderWidth` (34×26 at 96 DPI), not declared as separate constants.
  Replace `toolbox.cpp`'s `kSlotButtonSize = 32`
  and its `setFixedSize`/`contentWidth` sites.

### DPI scaling and the Qt6 double-scale guard

Qt 6 enables high-DPI scaling by default: widget geometry is expressed in
device-independent pixels and Qt multiplies by the device pixel ratio when it
paints. So the correct mechanism is to compute slots in **logical pixels scaled
by DPI but NOT apply `devicePixelRatio` again**: scale the base metrics by
`screen()->logicalDotsPerInch() / 96.0` (clamped to a sane range) and pass the
result to `setFixedSize`, letting Qt do the device transform once. The guard is
to derive from `logicalDotsPerInch`, never from `physicalDotsPerInch`, and to
re-apply on `screenChanged` / when the Tools column is re-hosted. Icon size is
`min(scaled base, kSlotIconMaxW/H scaled)`. Alternative considered: hard-coding
`devicePixelRatioF()` multiplication — rejected because Qt6 already applies it,
producing double-scaled slots on a 2× display.

### Self-anchor docking

`frame_columns.cpp::columnEdgeAnchorAt` skips `column == exclude`, so a drag
never sees its own source column's edge band. The whole-column path
(`resolveColumnMoveTarget`) intentionally excludes the source and must stay
unchanged. Fix: in the panel/group drag path (`panel_column_drag.cpp:259`
`columnEdgeAnchorAt(globalPos, this, &anchorSide)`), allow the source column to
anchor — pass `nullptr`/a flag so `columnEdgeAnchorAt` does not skip the source
for panel/group drags, and keep the `exclude` skip for whole-column moves.
Add a self-anchor guard so the item is not dropped onto the exact side it
already occupies.

### Menu geometry via sizeHint

`info_panel.cpp:264` reads `menu->size()` in the `QEvent::Show` filter, before
the menu is laid out, so the first open uses a stale (small) size. Fix: use
`menu->sizeHint()` for the placement math. Same class of fix (already correct)
in `toolbox.cpp::showSlotMenu`, which already uses `sizeHint()`.

### Side-opening flyouts and one-click switching

`toolbox.cpp::showSlotMenu` currently pops below the button. Change the anchor
to the button's right edge, flipping to the left when `x + hint.width()` exceeds
the available geometry, and clamp vertically only. The one-click rule lives in
`ToolSlotButton::mousePressEvent/ReleaseEvent`: when another slot's menu is
open, a press on a different slot must cancel/close it and activate (or open
that slot's menu) in the same gesture rather than being swallowed by the popup.

### Screen-mode button menu

The toolbox screen-mode button already emits `screenModeRequested`
(`frame_build.cpp:267`). Replace the bare click with an InstantPopup menu of the
three existing `ScreenMode` values plus their checked state, wired to the
existing `ViewScreenMode*` command handlers, so `F`/`Shift+F` cycling and the
menu share one path.

### Catalogue retention and hidden 3D/Camera slots

Keep the Object (3D) and Camera entries in `tool_catalog.cpp` — the enum and
table stay stable, so no catalogue count changes. The Tools panel skips their
slots when it builds its buttons, so the panel presents 21 visible slots while
the hidden entries remain resolvable. The hide is a presentation filter in the
toolbox build, not a catalogue edit.

### Layers two-column row

`LayerRowDelegate::paint` and the hit-testing split on `kEyeColumn` (26): the
gutter column returns no selectable region, and the content column paints at
`contentLeft + 2`. Selection painting uses a lighter-grey fill instead of
`QPalette::Highlight`; the model stays single-column (`columnCount == 1`) — the
two columns are a paint/hit-test concern, not new model columns. The row fill
becomes `shade(listBase, +1)`, and the 1px eye-gutter separator is unchanged.

### Numeric field and hint bar

`numeric_field.cpp:61` changes `AlignRight|AlignVCenter` to
`AlignLeft|AlignVCenter`. The Move-tool hint (`tool_catalog.cpp:329-332`,
`tool_hint_bar.cpp`) replaces the single `Arrows` key with four chevron glyph
keycaps.

**Note for apply:** the tool-framework scenario whose body now describes
side-opening keeps its original title `Holding opens the flyout below the button
[m40_flyout]` so the archive can match it by name; the normative text and THEN
say beside/right/left. Do not rename it without a dedicated rename.

## Risks / Trade-offs

- [DPI rounding on fractional scales produces a 1px seam] → derive both slot and
  icon metrics from the same scaled float and round once; visual, not golden.
- [Global `QToolButton` QSS change leaks to menus/dialogs] → scope the idle-less
  rule to the panel-family selectors (already the styling convention) and keep
  the generic rule for non-panel buttons.
- [Hiding the 3D/Camera slots breaks a pinned visible-slot self-test] → update
  the presented-slot count assertion in `selftest_shell_round4.cpp` to 21; the
  catalogue count is unchanged and the budget stays lower-only.
- [Renaming `kSlotButtonSize` ripples through toolbox tests] → keep a
  compatibility accessor for the test hooks rather than editing every test.

## Open Questions

None blocking. The exact "one step" factor (112 vs 115) is a visual call made in
`Theme` and can be nudged without changing any spec.

## Palette overhaul and workspace layout (refinement pass)

The explicit palette supersedes the `shade(window, ±1)` token derivation. The
`Ramp`/`kRamps` table becomes the default-level palette literal; the token list
grows a `${separator}` (`#404040`), `${tableBg}` (`#404040`), `${inputBg}`
(`#3b3b3b`), `${inputBorder}` (`#595959`), `${buttonBorder}` (`#595959`), and
`${buttonPressed}` (`#303030`), and `${base}` stops doubling as menu/header/table
input. `${window}` becomes `#363636` (file bar, inactive tabs, dock titles,
scrollbar), `${panel}`/`${activeTab}` `#4d4d4d`, `${panelHeader}` `#363636`,
`${border}` `#2e2e2e`, `${workspace}` `#1f1f1f`. Brightness levels keep the
exact default palette and shift each colour by one `Theme::shade` step per level
away from the default, re-applied through `styleSheetFor`. Tests that compared a
surface to `shade(window, ±1)` are updated to the literal values.

The central-band frame (2 px panel + 1 px border per side, 1 px inside) is
reserved as a 3 px margin on the central band rather than an overlay, so it never
clips column/options-bar content; the 2 px is the window surface showing through
and the 1 px is drawn in the theme border colour. The Tools column leaves
`centerSplitter_` for a fixed left slot in the central widget, so there is no
handle between it and the workspace; the splitter then holds only the document
tabs and the widget columns, and the index logic that assumed the tools pane at
index 0 (`panelColumns`, `sideOf`, `movePanelColumn`, `newColumnSideAt`,
`refreshToolsWidth`, the anchor `eventFilter`) is rebased on the document-tabs
index. The iconic strip drops `panelIconDivider` and the grip's bottom border and
gains a 1 px `panelIconGroup` bottom border; iconic columns resize from the
workspace-facing edge on either side, which the existing splitter already allows
once the missed handle target and the persist path are covered by a test.
