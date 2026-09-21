# Tasks

## 1. Drop mark beside the docked Tools column

- [x] 1.1 `PanelColumn::resolveDrop`/`showColumnEdgeIndicator`: draw the
  new-column mark for a target resolved beside the Tools column (an
  `anchorColumn` target), while a drag over the Tools body that resolves no
  beside-column target stays mark-free (the atomic rule).
- [x] 1.2 Keep checks 423 (`tools_column_atomic`) and 424
  (`tools_column_boundary_rejected`) and `newColumnDropForTest("left"/"tools")`
  green.

## 2. Floating icon mode

- [x] 2.1 `PanelFloat::syncToContent`/`setResizable`: collapsed-to-icons hides
  the resize grip and shrink-wraps both axes; expanding restores the grip and
  the shared normal minimum width.
- [x] 2.2 `PanelGroup::rebuildIconRow`: stack the floating icon row vertically
  (one icon per row).
- [x] 2.3 `PanelGroup::makeIconButton`/`eventFilter`: each floating icon starts a
  panel drag through the same grammar the docked strip uses, and a click below
  the threshold still opens the panel.
- [x] 2.4 A collapsed floating group can be dragged and merged into another
  floating panel.

## 3. Group-on-group body outline

- [x] 3.1 `resolveLocalDrop`/`updateDrag`: a whole-group drag over a different
  group's body resolves a tabify target, so the blue outline is drawn and the
  drop merges; the dragged group's own body stays an above/below reorder.

## 4. Layers control row

- [x] 4.1 `layers_panel.cpp`: blend mode and Opacity share the control row's
  slack so the blend input no longer dominates.

## 5. Shared minimum width

- [x] 5.1 `kPanelMinWidth`, `kMinNormalWidth`, `kDefaultNormalWidth`, and
  `PanelFloat::kFloatMinWidth` rise to 300; keep `kMaxNormalWidth` and the
  iconic strip minimum as bounds.

## 6. Coverage

- [x] 6.1 New self-test checks (next free code 444) for: the beside-Tools mark,
  the collapsed-float grip/size, the vertical icon row, the icon drag, and the
  group body outline.
- [x] 6.2 A check that a normal-mode column and a floating overlay each report a
  minimum width of at least 300.
- [x] 6.3 Existing checks 423, 424, 436, 130, 131, 132, and the whole self-test
  stay green.

## 7. Verification

- [x] 7.1 `openspec validate workspace-panel-icon-drag-width --strict` and
  `openspec validate --all --strict`.
- [x] 7.2 `cmake --build build --parallel`.
- [x] 7.3 `QT_QPA_PLATFORM=offscreen ./build/pictura --headless --self-test` —
  zero failures.
- [x] 7.4 `bash scripts/verify-full.sh` — OK.
