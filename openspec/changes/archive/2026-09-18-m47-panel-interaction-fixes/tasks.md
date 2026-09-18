## 1. Panel lifecycle (empty + ghost)

- [x] 1.1 `PanelColumn::rehomeFloatsTo(PanelColumn*)` disconnects/rewires live floats to the target, moves `floats_`, copies visibility, and repoints `PanelFloat::onClose`.
- [x] 1.2 `removeColumnIfEmpty` scans emptiness before the float guard; re-homes floats for dynamic columns; primary hides even with a float. `clearDynamicColumns` also re-homes floats.
- [x] 1.3 `cleanupEmptyGroup` hides a group with all tabs hidden and calls `maybeRemoveSelf`; `commitDrop`/`cancelDrag` use `visibleTitles().isEmpty()`.

## 2. Compact strip interactions

- [x] 2.1 Removed the `buildIconStrip()` rebuild from `createFloat` so the compact drag keeps its mouse grab and follows the cursor.
- [x] 2.2 `resolveIconicDrop` returns `AboveGroup` for a hit on a group's grip.
- [x] 2.3 Compact group container uses the panel surface shade and grip dots are dark gray (theme.cpp).

## 3. Sizing

- [x] 3.1 Stable shared content floor applied to the column; horizontal scrollbar off; `minimumWidthFloorForTest` returns the floor.
- [x] 3.2 Iconic column fixed width (`setFixedWidth` + horizontal Fixed) and maximum cleared on exit.

## 4. Toolbar placement and transparency

- [x] 4.1 `Toolbox` arms the title-bar gesture regardless of float state (drag threshold gated); frame commits without the `isFloating()` re-check.
- [x] 4.2 `resolveToolboxDrop` anchors the Tools pane's neighbour and the column under the pointer; a pointer over no column keeps the panel's current state.
- [x] 4.3 `Toolbox` splitter-pane state keeps fixed width (not fixed height) while hosted in the splitter.
- [x] 4.4 `PanelColumn::floatBounds` unions the central rect with the tools dock/pane so widget overlays cross the toolbar.

## 5. Floating widget close control

- [x] 5.1 `PanelGroup` gets a rightmost close button shown only while floating.
- [x] 5.2 `PanelFloat::onClose` and `PanelColumn::closeFloat` re-home + `closeGroup` + destroy the shell.
- [x] 5.3 `insertGroupAt` resets the floating state so a re-docked group hides the close button.

## 6. Self-tests

- [x] 6.1 Empty column removed after its last group floats; re-homed float still closes/re-docks (`m47_empty_after_float`).
- [x] 6.2 Group with all tabs hidden is hidden and restorable (`m47_ghost_group`).
- [x] 6.3 Compact icon drag follows as a float and commits on release (`m47_compact_icon_float`).
- [x] 6.4 Compact grip drop creates a group above (`m47_compact_grip_group`).
- [x] 6.5 No horizontal scroll and content fits at the floor; iconic width fixed (`m47_no_hscroll`, `m47_iconic_fixed_width`).
- [x] 6.6 Tools pane hosted among columns; widget overlay crosses the toolbar (`m47_tools_pane`).
- [x] 6.7 Float close button hides panels and keeps the group restorable (`m47_float_close`).
- [x] 6.8 Compact group shade and dots (`m47_compact_shade`).

## 7. Verification

- [x] 7.1 `openspec validate --all --strict` (62 passed).
- [x] 7.2 `bash scripts/verify-fast.sh`: fmt ok, clippy 0 errors, 589 tests passed; guard only trips on the pre-existing untracked `docs/dev/mcp-agentic-control-plan.md`.
- [x] 7.3 CMake build and `./build/pictura --headless --self-test` (exit 0, all `m47_*` green).
- [x] 7.4 Independent verifier review; findings fixed (toolbar over-tab commit, drag threshold, `clearDynamicColumns` float orphan, `closeFloat` shell, re-homed close test).
