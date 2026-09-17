## 1. Brief and frozen interfaces (orchestrator)

- [x] 1.1 Write `docs/dev/m40-tools-panel.md`: the cited CS6 behavior, the
  current-state inventory with file refs, the frozen interfaces (flyout
  indicator/opening/keys, column toggle, standalone dock, generic `Shift` cycling
  + preference, session v4), the task split, and the non-goals
- [x] 1.2 Write `proposal.md`, `design.md`, this `tasks.md`, and the
  `specs/tool-framework/spec.md` + `specs/application-shell/spec.md` deltas
  (MODIFIED "Tool registry and active tool" and "Tools panel"; ADDED the column
  layout, flyout indicator/opening, flyout keys, shift-key preference, and the
  standalone dock)
- [x] 1.3 Freeze in `design.md`: the custom-paint triangle; the 300 ms
  hold/right-click open below the button; the scoped
  `setShortcut` + `setShortcutVisibleInContextMenu` key display; the title-bar
  grid reflow and target-layout icon; the per-dock tabify event filter and
  `dockLocationChanged` fallback; the generic per-group letter handler; session
  schema v4; the test hooks
- [x] 1.4 Update `docs/dev/STATE.md`: M40 proposed, the M40/M41 pair, and the
  Layers-panel program shifted by two (filtering → M42, management → M43, styles
  → M44, smart objects → M45); canvas-perf tracks by name
- [x] 1.5 Validate: `openspec validate m40-tools-panel --strict` and
  `openspec validate --all --strict`

## 2. Assets: the two column icons

- [x] 2.1 Draw `assets/icons/panel.columnsOne.svg` (left double-chevron) and
  `assets/icons/panel.columnsTwo.svg` (right double-chevron) in the M19 style:
  24×24 `viewBox`, `fill="none"`, `stroke="#c8c8c8"`, `stroke-width="1.5"`, round
  caps/joins, no `<text>`, at most one `#3d6f99` accent
- [x] 2.2 Add both files to `assets/pictura.qrc` and confirm `icon("panel.columnsOne")`
  and `icon("panel.columnsTwo")` resolve non-null
- [x] 2.3 Confirm a missing icon is a guarded fallback (the toggle draws a text
  arrow), matching the M38 null-asset policy

## 3. Flyout triangle, long-press/right-click, and keys

- [x] 3.1 Remove `setPopupMode(QToolButton::MenuButtonPopup)` and paint the
  lower-right filled triangle in `ToolSlotButton::paintEvent` iff the slot's
  group has two or more members (implemented or not), keeping the icon centred
- [x] 3.2 Set the hold timer to 300 ms; on timeout pop the menu with
  `menu->popup(button->mapToGlobal(QPoint(0, button->height())))` clamped to the
  screen's available geometry; right-click opens immediately; a release before
  the timeout activates the slot's current tool and opens nothing
- [x] 3.3 Give every flyout `QAction` `setShortcut(slotKey)`,
  `setShortcutVisibleInContextMenu(true)`, and
  `Qt::WidgetWithChildrenShortcut`; keep disabled items with the
  `<label> — not implemented yet` tooltip; remove `setShortcut()` from
  `refreshSlot()`
- [x] 3.4 Add the deterministic test hooks: `openSlotFlyoutForTest`,
  `slotMenuForTest`, `slotMenuActionsForTest`, `hasFlyoutTriangleForTest`

## 4. Title bar and one/two-column toggle

- [x] 4.1 Build the custom title bar (`setTitleBarWidget`) with a `Tools` label
  and a flat double-arrow `QToolButton` showing the target layout's icon
  (`panel.columnsTwo` in one column, `panel.columnsOne` in two)
- [x] 4.2 Reflow the 23 slots in the `QGridLayout`: one column `(i, 0)`, two
  columns row-major `(i / 2, i % 2)`; keep the foreground/background widget and
  the Screen Mode button below the grid in both layouts
- [x] 4.3 Size the dock per layout (one-column minimum 66 px; two-column minimum
  104 px); expose `columns()`/`setColumns(int)` with a 1-or-2 clamp and the
  title-bar test hook

## 5. Standalone dock and tabify refusal

- [x] 5.1 Set `setAllowedAreas(Qt::LeftDockWidgetArea | Qt::RightDockWidgetArea)`
  and `setFeatures(Movable | Floatable | Closable)`; keep the dock's
  `objectName` `toolsPanel` and the `Window > Panels > Tools` wiring
- [x] 5.2 Install an event filter that rejects a drag/drop targeting a dock tab
  bar, and connect `dockLocationChanged`: if `tabifiedDockWidgets(toolbox)` is
  non-empty, `setFloating(true)` then re-add the dock to its previous side
  (the reactive tabify-veto fallback)
- [x] 5.3 Keep the custom title bar draggable so moving and floating still work;
  record the missing float/close glyphs and the fallback limitation in the brief

## 6. Generic Shift+key cycling, preference, and session v4

- [x] 6.1 Add the letter/shift → group helper over `allToolIds()`/`toolInfo`
  (`tools.{h,cpp}`); each letter maps to exactly one group
- [x] 6.2 Delete the `B`/`Shift+B` `cyclePaintTool` lambda in `frame.cpp` and
  register one plain and one `Shift` shortcut per distinct letter: with the
  preference on, the plain letter selects the slot's current member and
  `Shift`+letter cycles implemented members (wrapping); with it off, the plain
  letter cycles; an all-unimplemented group does nothing
- [x] 6.3 Extend `session.{h,cpp}` to schema v4 with `toolsColumns` (default 1)
  and `useShiftKeyForToolSwitch` (default true); load the defaults for a
  missing/older value and keep the M39 load-before-write save path
- [x] 6.4 Pass the loaded preference and column count into `buildTools()` and
  persist `tools->columns()` in `saveSession()`; state that there is **no
  Preferences dialog** in M40 and M41 gives the key a UI

## 7. Tests and self-test

- [x] 7.1 C++ `m40_columns` (exit 113/114): default one column, toggle to two and
  back, row-major reflow, the 23 slots, the bottom controls pinned, both icons
  resolve
- [x] 7.2 C++ `m40_flyout` (115): triangle on a multi-member slot, none on a
  single-member slot, the member split, and the test hook opening the flyout at
  the button's bottom edge. The right-click-immediate and quick-release-select
  timing paths exist but are **not self-tested** — only the triangle, member
  split, and popup placement are asserted
- [x] 7.3 C++ `m40_keys` (116): every item shows the group's key; an unimplemented
  item is disabled with its tooltip; the disabled item's shared key does not
  change the active tool and the plain letter still reaches the frame handler
- [x] 7.4 C++ `m40_shift` (117): preference on — plain letter selects the current
  member and `Shift`+letter cycles; preference off — the plain letter cycles; the
  all-unimplemented Retouch slot does nothing
- [x] 7.5 C++ `m40_dock` (118): allowed areas left/right only, features
  movable/floatable/closable, a simulated tabify leaves no tabified dock (the
  fallback re-docks), and the title bar is draggable
- [x] 7.6 C++ `m40_session` (119): schema v4 defaults (1, true), a change
  round-trips, and a schema-3 store loads the defaults without disturbing the
  other fields
- [x] 7.7 `xvfb-run -a ./build/pictura --self-test` and the `two_layers.psd`
  variant, both exit 0; confirm every earlier check is unchanged

## 8. Verification and close-out

- [x] 8.1 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets --
  -D warnings`; `cargo test --workspace` (expected unchanged — no Rust change)
- [x] 8.2 `cmake -S . -B build && cmake --build build`
- [x] 8.3 `openspec validate m40-tools-panel --strict`;
  `openspec validate --all --strict`
- [x] 8.4 Record the M40 result in `docs/dev/STATE.md`
- [ ] 8.5 Archive the change (`openspec archive m40-tools-panel`) and commit —
  deferred; this task does not commit
