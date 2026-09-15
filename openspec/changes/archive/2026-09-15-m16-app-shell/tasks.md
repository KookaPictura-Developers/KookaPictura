## 1. Brief and interface freeze (orchestrator)

- [x] 1.1 Write `docs/dev/m16-app-shell.md` milestone brief
- [x] 1.2 Freeze `theme.h`, `session.h`, `commands.h`, `frame.h`, `image_view.h` interfaces and the command-id list
- [x] 1.3 Record sub-agent workstreams (A: theme+session, B1: commands, B2: command_tree, C: frame+image_view) with disjoint file ownership; orchestrator keeps `main.cpp`, `CMakeLists.txt`, `cxxqt_object.rs`, self-test
- [x] 1.4 Commit brief + proposal with a `TASK-ALLOWS-DOCS` message

## 2. Rust bridge

- [x] 2.1 Add `has_document()` to `PictureView` in `crates/pictura-app/src/cxxqt_object.rs`
- [x] 2.2 Verify `has_document()` through the self-test enablement assertions (6.5); bridge QObjects are not unit-testable without Qt

## 3. Theme and session (workstream A)

- [x] 3.1 Implement `cpp/theme.{h,cpp}`: Fusion style plus a generated dark `QPalette` per brightness level
- [x] 3.2 Implement four brightness levels with an apply function and level count
- [x] 3.3 Implement `cpp/session.{h,cpp}`: load/save `{layout, brightness, schema_version}` to `$XDG_STATE_HOME/kooka-pictura/state.json`
- [x] 3.4 Make the session write atomic via `QSaveFile`; fall back to defaults on missing/corrupt input
- [x] 3.5 Mark the deferred typed `prefs.toml` crate with a `ponytail:` comment naming the upgrade path

## 4. Command registry and menu tree (workstream B)

- [x] 4.1 Implement `cpp/commands.{h,cpp}`: `CommandSpec` table (id, path, label, shortcut, implemented, checkable)
- [x] 4.2 Build the menu bar from the table with `QAction::setData(commandId)`
- [x] 4.3 Implement handler registration by id and dispatch of `triggered` by id; unregistered ids are inert
- [x] 4.4 Add per-command enablement predicates evaluated on `aboutToShow`; document-requiring commands query `has_document()`
- [x] 4.5 Add dynamic labels for the undo/redo history command
- [x] 4.6 Populate the full documented top-level tree (File, Edit, Image, Layer, Type, Select, Filter, View, Window, Help) with documented submenu ordering; unimplemented leaves disabled
- [x] 4.7 Register the `Window > Panels` entries and the Help > About command in the tree

## 5. Application frame (workstream C)

- [x] 5.1 Implement `cpp/frame.{h,cpp}`: `PicturaMainWindow` hosting menu bar, central `ImageView`, status bar, and dock area
- [x] 5.2 Register the existing debug panel as the `layers` dock with a unique `objectName`; reject duplicate names
- [x] 5.3 Implement the status bar (magnification, document size, tool-hint placeholder) and its view-options popup
- [x] 5.4 Register handlers for the implementable commands: Edit Undo/Redo/Step, Select All/Deselect, Image Rotation 90 CW/CCW/180 and flips, View Zoom In/Out/Fit/100%, Window Panels, Help About, File Open
- [x] 5.5 Implement screen modes (Standard / Full With Menu Bar / Full) with `F` and `Shift+F`
- [x] 5.6 Implement canvas-colour cycling with `Space+F`
- [x] 5.7 Implement `Tab` hide-all and `Shift+Tab` (behaves as `Tab` until Tools/options bar exist)
- [x] 5.8 Wire layout `saveState`/`restoreState` and brightness through the session store on startup and quit

## 6. Integration and self-test (orchestrator)

- [x] 6.1 Shrink `main.cpp` to argument parsing, `PicturaMainWindow` construction, and the self-test
- [x] 6.2 Add the new sources to `CMakeLists.txt`
- [x] 6.3 Extend `--self-test`: menu bar contains the ten menus in order
- [x] 6.4 Extend `--self-test`: a dispatched command fires its handler; an unregistered command is inert
- [x] 6.5 Extend `--self-test`: document-requiring commands are disabled with no document and enabled after open
- [x] 6.6 Extend `--self-test`: brightness level applies and steps
- [x] 6.7 Extend `--self-test`: screen mode cycles forward and backward
- [x] 6.8 Extend `--self-test`: layout save/restore round-trips in a temporary XDG state dir; duplicate `objectName` is rejected
- [x] 6.9 Extend `--self-test`: `Tab` hides and restores panels
- [x] 6.10 Assign self-test exit codes starting at 25 and keep existing engine checks passing

## 7. Verification and close-out

- [x] 7.1 `cmake -S . -B build && cmake --build build`
- [x] 7.2 `xvfb-run -a ./build/pictura --self-test` and the fixture self-test pass
- [x] 7.3 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`
- [x] 7.4 `openspec validate --all --strict` and `bash scripts/guard.sh`
- [x] 7.5 Update `docs/dev/STATE.md` (crate table, M16 entry, test count) with a `TASK-ALLOWS-DOCS` message
- [x] 7.6 Archive the change with `openspec archive m16-app-shell`
