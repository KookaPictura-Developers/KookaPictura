## 1. Tools panel placement

- [x] 1.1 Extend `resolveToolboxDrop` with a workspace outer-band pass and a
  nearest-visible-column fallback, so any point in the central area resolves to a
  column edge.
- [x] 1.2 `commitToolboxDrop` accepts a null anchor and hosts the pane at the
  splitter head/tail for a bare workspace edge.
- [x] 1.3 Add `toolboxOnColumnForTest` and self-test `lss_toolbox_column_drop`
  (406): either side of a non-edge column plus both outer bands.

## 2. Column header drag

- [x] 2.1 Add `resolveColumnMoveTarget` / `movePanelColumn` to the frame and
  reject an unresolved side so a no-target release leaves the column in place.
- [x] 2.2 Install a header event filter in `PanelColumn` (toggle child excluded),
  drive the existing edge indicator, and commit on release.
- [x] 2.3 Add hooks + self-test `lss_col_header_move` (401).

## 3. Panel group tabs

- [x] 3.1 `setUsesScrollButtons(false)` so overflow compresses/elides; clamp
  `tabInsertionX` inside the bar so an end-of-bar drop still lands on the tab bar.
- [x] 3.2 Self-test `lss_tab_compress` (402).

## 4. Reserved group drag grip

- [x] 4.1 Add the always-present blank grip at the right of the tab bar and route
  its press-drag through the group-drag signals; keep the corner visible.
- [x] 4.2 Self-test `lss_grip_drag` (403).

## 5. Floating group top bar and icon mode

- [x] 5.1 Add the float top bar with an icon/normal toggle and the moved close
  control; the bar drags the overlay.
- [x] 5.2 `collapsedToIconsChanged` -> `PanelFloat::syncToContent` with a minimum
  height; style the icon row as a panel surface; `createFloat` sizes for icons.
- [x] 5.3 Self-tests `lss_float_header` (404) and `lss_float_icons` (405).

## 6. Verification

- [x] 6.1 `cmake --build build --parallel`; `./build/pictura --headless
  --self-test` — 341 passed, 0 failed.
- [x] 6.2 `bash scripts/verify-full.sh` — TOTAL 1644 passed, 11 skipped, 0 failed.
- [x] 6.3 `openspec validate --all --strict` — 82 passed, 0 failed.

## 7. Not done

- [ ] 7.1 No persistence of a Tools-panel splitter-pane index beyond the existing
  `saveState` layout blob.
- [ ] 7.2 Float icon mode reuses the group's icon row (single group), not a
  per-float rebuild of the multi-group column strip.
