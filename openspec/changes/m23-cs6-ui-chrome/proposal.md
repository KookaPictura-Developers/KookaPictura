## Why

The M16–M21 shell is functionally complete but visually generic: it uses the
stock Fusion dark palette with no stylesheet, a single-column tool strip, and
seven equal dock widgets. CS6's identity is its dark chrome — flat panel tabs and
title bars, a compact two-column tool grid with the foreground/background
control, tabbed panel groups, and a dark canvas. A reference screenshot of the
target is stored at `docs/02-ui-ux/reference/cs6-workspace.png`.

## What Changes

- Apply a CS6-style stylesheet from `Theme::apply` at all four brightness
  levels: menu bar, options bar, Tools panel, panel docks and their tabs/title
  bars, status bar, tool buttons, and scrollbars.
- Rebuild the Tools panel as a compact two-column tool grid with the
  foreground/background colour control, a reset-swatches affordance, and a
  screen-mode control at the bottom, matching the CS6 toolbox.
- Group the existing panels into CS6-style tabbed docks (Color/Swatches,
  Layers/History, Navigator/Info/Histogram) in the default layout.
- Style the document tab strip and keep the CS6 dark canvas colour.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `application-shell`: adds CS6-style chrome styling as a requirement, and a
  default dock-grouping / canvas-colour requirement.
- `tool-framework`: the Tools panel requirement changes from one button per tool
  in a strip to a compact two-column grid with the foreground/background
  control.

## Impact

- `crates/pictura-app/cpp/theme.cpp` gains a stylesheet (QSS) built per
  brightness level; `theme.h` may expose the stylesheet string for tests.
- `crates/pictura-app/cpp/toolbox.{h,cpp}` is rebuilt as a grid with the
  colour control; `frame.cpp` wires the colour control to `ColorState` and
  `Tab`/`Shift+Tab` behaviour.
- `crates/pictura-app/cpp/frame.cpp` tabifies the default docks and keeps the
  session layout restorable.
- No Rust or document-format changes; no new dependencies.
- Visual-only changes must not regress the M16–M22 self-tests (panel
  objectNames, tool switching, hide-all).
