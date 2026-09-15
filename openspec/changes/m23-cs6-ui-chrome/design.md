## Context

`Theme::apply` currently sets a Fusion `QPalette` from a four-level dark ramp
(`crates/pictura-app/cpp/theme.cpp`). That leaves the stock widget metrics and
many surfaces unstyled: `QDockWidget` title bars and tabs, the tool strip,
menu/tool/status bars, and scrollbars. The Tools panel is a single-column
`QToolBar` of one action per tool (`toolbox.cpp`), and `frame.cpp` registers
seven peer docks with no grouping. The reference target is
`docs/02-ui-ux/reference/cs6-workspace.png` and the corpus is
`docs/02-ui-ux/{application-frame,workspace-and-docks,toolbox-and-options-bar}.md`.

## Goals / Non-Goals

**Goals:**

- A CS6-look stylesheet applied at all four brightness levels, derived from the
  same ramp so `Shift+F1`/`Shift+F2` restyle everything.
- A compact two-column Tools panel with the foreground/background colour control
  and a screen-mode control.
- CS6-style tabbed panel groups in the default layout, still restorable through
  the session.
- A dark document canvas and a styled document tab strip.

**Non-Goals:**

- Faithful pixel metrics of CS6 (icon art, exact paddings, HUD/on-image
  displays, brush/HUD cursor). Those are later and are not sourced.
- New panels (Gradients, Patterns, Properties, Adjustments, Libraries, Channels,
  Paths, Brush). Panel grouping only rearranges what exists.
- Workspace presets, icon-collapse docks, floating-panel drop zones, and the
  workspace switcher.
- Any change to Rust, the document model, or the PSD format.

## Decisions

### A QSS stylesheet generated per brightness level, applied in `Theme::apply`

Build a stylesheet string from the level's ramp and call
`qApp->setStyleSheet(...)` after `setPalette`. Expose the string (e.g.
`Theme::styleSheet(level)`) so the self-test can assert it is non-empty and
level-dependent.

- *Why:* one code path for all four levels; QSS covers the surfaces the palette
  cannot (dock tabs, title bars, scrollbars, tool buttons).
- *Alternatives considered:* subclassing every widget (large, brittle); a
  separate `.qss` resource per level (four near-duplicate files).

### Tools panel: two-column grid with the foreground/background control

Replace the `QToolBar` with a `QWidget` grid of `QToolButton`s, two columns, in
CS6 slot order, plus a `ForegroundBackgroundWidget` (overlapping swatches,
click to activate, `X`/shuffle affordances) and a bottom screen-mode button.
The existing `ToolId` set is unchanged; the grid is presentation only.

- *Why:* the CS6 toolbox is a fixed narrow grid; a grid widget is the direct
  match and keeps the tool controller untouched.
- *Deferral:* tool-slot fly-outs (long-press groups) are not reproduced; each
  tool gets its own cell.

### Default dock grouping via `tabifyDockWidget`

Group Color/Swatches, Layers/History, and Navigator/Info/Histogram as tabbed
docks in `buildPanels`, and raise the first tab of each group. Keep stable
`objectName`s so `Tab`/`Shift+Tab` and the session layout are unaffected.

- *Why:* CS6 groups panels into tabbed docks; Qt's `tabifyDockWidget` gives this
  without a custom dock manager.
- *Alternative considered:* a custom dock manager — deferred.

### Canvas and tab strip styling

Set the document view background from the ramp (CS6 dark grey) and style
`QTabBar::tab` for the document tabs.

## Risks / Trade-offs

- **QSS can override palette-based states** → keep the palette as the source of
  truth and style only surfaces QSS needs; verify `Tab`/`Shift+Tab`, checked
  states, and disabled actions after styling.
- **Toolbox rewrite could break tool shortcuts** → preserve the existing action
  shortcuts and the `activeToolChanged` wiring; the self-test already covers tool
  switching.
- **Tabified docks change the session layout** → the layout is versioned and
  `restoreState` tolerates missing docks; a reset path exists via the Window
  menu. Verify the M16 session self-test still passes.
- **Visual-only changes are hard to assert** → the self-test checks structure
  (stylesheet non-empty and level-dependent, toolbox grid has the expected
  button count, fg/bg widget exists, default groups share a dock) rather than
  pixels.

## Migration Plan

Additive/visual. Rollback is reverting `theme.*`, `toolbox.*`, and the
`buildPanels` grouping. No data migration.

## Open Questions

- Exact CS6 brightness swatch values are unsourced; M23 keeps the existing four
  ramps and only changes what is styled.
- Whether the tools panel should show labels or icons only at the narrow width —
  M23 shows icon-only buttons with tooltips, like the reference.
