## Why

The Qt application is still the M0 walking skeleton: one `QMainWindow`, one
image view, one debug dock, and two shortcuts. It exposes a real image engine
(M0–M15) through a shell that looks nothing like Photoshop CS6 and has no place
to hang a dialog, a tool, or a menu command. Every future feature needs a frame,
a command dispatch layer, and enablement state; building them one feature at a
time would fork the shell. M16 lays that foundation once.

## What Changes

- Add a **command registry**: one declarative table of commands (id, menu path,
  label, shortcut, enablement) that drives the menu bar, context menus, and
  future keyboard/menu customization from a single source.
- Replace the M0 window with a **CS6-shaped application frame**: menu bar,
  central canvas, status bar, and dock areas.
- Add the **full documented menu tree** (File, Edit, Image, Layer, Type, Select,
  Filter, View, Window, Help) in documented order. Commands with a working
  handler are enabled; the rest are present but disabled, so menu shape and
  enablement are real from day one.
- Add a **dark interface theme** with four brightness levels (Fusion palette;
  `Shift+F1`/`Shift+F2`), and **screen modes** (Standard / Full With Menu Bar /
  Full) cycled with `F` / `Shift+F`, plus canvas-colour cycling with `Space+F`.
- Add a **status bar** (magnification, document size, tool-hint placeholder) with
  a view-options popup.
- **Register panels as docks** with stable `objectName`s, layout persistence via
  `saveState`/`restoreState`, a `Window > Panels` toggle, and `Tab`/`Shift+Tab`
  hide-all.
- Persist **session state** (dock layout + brightness) to the XDG state
  directory so the frame survives restart.
- Extend the Rust bridge with `has_document()` for command enablement and status.

Deferred to later milestones (not this change): multi-document tabs and the file
lifecycle (M17), options bar and tools (M18), real panels and the Preferences
dialog (M19), localized/customizable menus and a custom `QStyle`.

## Capabilities

### New Capabilities

- `command-registry`: the declarative command table, menu-tree construction,
  command dispatch, and per-command enablement, driving the menu bar and future
  context/custom menus.
- `workspace-persistence`: panel registration as docks, layout
  `saveState`/`restoreState`, the `Window > Panels` toggle and hide-all, and the
  XDG session store for layout and interface brightness.

### Modified Capabilities

- `application-shell`: the headless window requirement is upgraded from a bare
  image view to a CS6-shaped frame (menu bar, status bar, theme, screen modes,
  dock areas) while keeping the cxx-qt bridge, CMake build, and self-test
  guarantees.

## Impact

- Affected crate: `pictura-app` (C++ shell gains `commands`, `theme`, `frame`,
  and `session` units; `main.cpp` shrinks to startup and self-test; Rust bridge
  gains `has_document()`).
- Build: `CMakeLists.txt` gains the new sources. No new dependencies; session
  persistence uses Qt `QSaveFile` rather than a Rust `prefs.toml` crate
  (deferred, marked with a `ponytail:` note).
- Docs/specs: `openspec/specs/application-shell` is modified; `command-registry`
  and `workspace-persistence` are new.
- Verification: `--self-test` grows assertions for menu order, command dispatch,
  brightness, screen-mode cycling, layout save/restore in a temporary XDG dir,
  and hide-all. No document-format or engine behavior changes.
