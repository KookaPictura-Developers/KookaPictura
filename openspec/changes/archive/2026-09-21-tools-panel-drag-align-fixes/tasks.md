## 1. Top-align docked/pane content

- [x] 1.1 `Toolbox::updateContentMetrics` sets the body's vertical size policy to
  `Expanding` when the dock is not floating (keeps `Fixed` while floating), so the
  body fills the given height and the trailing stretch keeps the slots at the top.

## 2. Restore the outer-band drop preview

- [x] 2.1 `resolveToolboxDrop` shows the outermost visible column's shared edge
  indicator while the pointer is in the workspace outer band, as a visual-only
  anchor, and still returns `false` with a null anchor and `side == -1`.
- [x] 2.2 `commitToolboxDrop` clears that visual-only anchor before returning
  `false` so a declined commit leaves no line on screen.

## 3. Float on an empty-workspace release

- [x] 3.1 Remove the "nearest visible column anywhere in the central area"
  fallback from `resolveToolboxDrop`, so a release with no column under the
  pointer and outside the outer band declines and the release path floats the
  panel at the cursor.

## 4. Coverage

- [x] 4.1 `lss_tools_top_align` (417): a pane-hosted panel's body fills the dock
  height and starts at the top.
- [x] 4.2 `lss_tools_edge_preview` (418): the outer band shows the edge indicator
  and still declines; a declined commit clears it.
- [x] 4.3 `lss_tools_canvas_float` (419): a release over the empty document area
  declines the pane resolve and leaves the panel's state unchanged.

## 5. Verification

- [x] 5.1 `cmake --build build --parallel` and
  `./build/pictura --headless --self-test` — all passed.
- [x] 5.2 `bash scripts/verify-full.sh` — OK.
- [x] 5.3 `openspec validate --all --strict`.
