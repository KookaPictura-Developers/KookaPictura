## 1. Tools panel placement (repeat fix)

- [x] 1.1 Consume the title-bar press/move/release in `Toolbox::eventFilter` so
  Qt's `QDockWidget` title-bar drag can never start; follow the cursor when
  floating; record the grab offset.
- [x] 1.2 `toolbarDragFinished` commits the resolved splitter boundary and, when
  nothing resolves, floats the dock under the cursor instead of an outer edge.
- [x] 1.3 Self-test `lss_tools_title_drag` (407): real title-bar gesture lands a
  pane at the expected index, proves the dock did not float, and keeps a
  no-target release floating.

## 2. Column header drag, edge placement, menu

- [x] 2.1 A bare workspace-edge drop resolves a null anchor so the column lands
  at the splitter extreme, including left of a Tools pane.
- [x] 2.2 Header left-click (no drag) opens the header menu: `Collapse to Icons`,
  `Auto-Collapse Iconic Panels`, `Auto-show Hidden Panels`, `Interface Options…`.
- [x] 2.3 Self-test `lss_header_menu` (408): the indicator shows, a far-left drop
  lands at index 0 left of Tools, and each menu action takes effect.

## 3. Floating group parity

- [x] 3.1 Toggle immediately left of close.
- [x] 3.2 Collapsed height snaps to the icon row.
- [x] 3.3 Collapsed row renders as the docked strip's `panelIconGroup` box with
  a grip divider and 34/24 icons.
- [x] 3.4 `QSizeGrip` + minimum size.
- [x] 3.5 Single-panel float tab drag redirects to a whole-overlay group drag.
- [x] 3.6 Overlay dims over a valid drop target.
- [x] 3.7 Self-tests 409-414.

## 4. Group context menu

- [x] 4.1 Corner container, reserved grip, and `▾` button share the header
  background; the button is vertically centred.
- [x] 4.2 Live per-widget menu gains `Close` (active tab) and `Close Group`,
  wired through the same close path.
- [x] 4.3 Self-test `lss_panel_menu_close` (413).

## 5. Verification

- [x] 5.1 `cmake --build build --parallel`; `./build/pictura --headless
  --self-test` — 349 passed, 0 failed.
- [x] 5.2 `bash scripts/verify-full.sh` — TOTAL 1652 passed, 11 skipped, 0 failed.
- [x] 5.3 `openspec validate --all --strict` — 82 passed, 0 failed.

## 6. Not done / ceilings

- [ ] 6.1 A bare-edge column drop draws the indicator on the outermost column
  rather than on the Tools pane itself; placement is correct, the line is
  approximate when the Tools pane is the extreme pane.
- [ ] 6.2 Real-compositor QDockWidget drag ordering is no longer relied on
  (the gesture consumes it), but the self-test still drives the gesture
  synthetically rather than through an OS input device.
