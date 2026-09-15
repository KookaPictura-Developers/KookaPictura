# M16 — App shell foundation (menu registry, frame, theme, docks)

Goal: replace the M0 walking-skeleton window with a CS6-shaped application frame
so every later feature (dialogs, tools, panels, file lifecycle) has a frame,
a command dispatch layer, and enablement state to hang on. OpenSpec change:
`m16-app-shell` (MODIFIED `application-shell`; new `command-registry` and
`workspace-persistence`).

## Scope

- `cpp/commands.{h,cpp}` — declarative command table (stable id, menu path,
  label, shortcut, dynamic label, enablement) and a `CommandRegistry` that builds
  the menu bar with `QAction::setData(id)`, dispatches `triggered` by id, and
  re-evaluates enablement on `aboutToShow`.
- `cpp/frame.{h,cpp}` — `PicturaMainWindow`: menu bar, central canvas, status
  bar, dock area; screen modes (`F`/`Shift+F`), canvas colour (`Space+F`),
  `Tab`/`Shift+Tab` hide-all, dock layout save/restore.
- `cpp/theme.{h,cpp}` — Fusion style plus a generated dark `QPalette` per
  brightness level (four levels, `Shift+F1`/`Shift+F2`).
- `cpp/session.{h,cpp}` — atomic `QSaveFile` store of `{layout, brightness,
  schema_version}` at `$XDG_STATE_HOME/kooka-pictura/state.json`.
- `cpp/image_view.{h,cpp}` — the existing `ImageView` extracted from `main.cpp`
  with zoom presets and a canvas-colour property.
- Rust bridge — `has_document()` for enablement/status; no engine change.
- Full documented top-level menu tree (File, Edit, Image, Layer, Type, Select,
  Filter, View, Window, Help); commands without a handler are present and
  disabled.

## Out of scope (later milestones)

- M17: multi-document tabs, New/Open dialogs beyond a basic Open, Save/Save As,
  dirty state, recent files.
- M18: toolbox, tool state, options bar contents.
- M19: panels beyond the existing debug dock, Preferences dialog.
- Custom menu/shortcut sets, custom `QStyle`, localization, accessibility.

## Process

Per the project convention: brief + OpenSpec proposal first
(`TASK-ALLOWS-DOCS`), then freeze the four headers, then dispatch write-capable
sub-agents on disjoint files (A: theme+session, B: commands, C: frame+image_view),
then the orchestrator integrates `main.cpp`/`CMakeLists.txt`/the bridge, extends
`--self-test` (new exit codes from 25), verifies independently, and archives.

## Verification

- `cmake -S . -B build && cmake --build build`
- `xvfb-run -a ./build/pictura --self-test crates/pictura-codec/tests/fixtures/two_layers.psd`
  with new assertions: the ten menus in order; dispatch of a registered command
  and inertness of an unregistered one; document-required commands disabled with
  no document and enabled after open; brightness applies and steps; screen mode
  cycles both ways; layout save/restore round-trips under a temporary
  `XDG_STATE_HOME` and a duplicate `objectName` is rejected; `Tab` hides and
  restores panels.
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace`
- `openspec validate --all --strict`, `bash scripts/guard.sh`
