## 1. Restore the three placements

- [x] 1.1 `resolveToolboxDrop`: the workspace outer band no longer resolves as a
  splitter pane (guarded before the column passes), so it can dock.
- [x] 1.2 `dockToolbox(side)` and `floatToolboxAt(pos)` added; the release
  handler decides pane / outside-float / edge-dock / float.
- [x] 1.3 Title-bar double-click toggles float and re-dock.

## 2. Coverage

- [x] 2.1 `toolboxOnColumnForTest` outer-band case updated to assert the resolver
  declines (the release path docks).
- [x] 2.2 Check 415 `lss_tools_dock_float`: pane, left-edge dock, right band,
  double-click float, double-click re-dock.

## 3. Verification

- [x] 3.1 `./build/pictura --headless --self-test` — 350 passed, 0 failed.
- [x] 3.2 `bash scripts/verify-full.sh` — TOTAL 1652 passed, 11 skipped, 0 failed.
- [x] 3.3 `openspec validate --all --strict`.

## 4. Not done

- [ ] 4.1 The outer band is reserved for docking, so a pane cannot be dropped in
  a column's outer 28 px at the workspace edge; pane placement still works over
  the column body.
