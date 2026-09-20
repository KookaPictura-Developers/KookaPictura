## 1. Batch 1 — Startup/tab/brush keys + paint live-render regression (items 1, 2, 21)

- [x] 1.1 Item 1: in `crates/pictura-app/cpp/frame.cpp::refresh` show or hide the
  document tab pane from the empty/occupied state of `docs_` (one branch, no
  other visibility writer), so an empty workspace draws no ghost canvas; assert
  the pane's visibility after create/open/close/close-all through the same path.
- [x] 1.2 Item 2: in `crates/pictura-app/cpp/theme.cpp` add a
  `QTabBar#documentTabBar::tab { font-weight: 500; padding-right: …; }` rule
  scoped to the document tab bar; do not edit the unscoped `QTabBar::tab` rules
  or the line `selftest.cpp` string-matches.
- [x] 1.3 Item 21: in `crates/pictura-app/cpp/tools.cpp` route `[`/`]` through a
  new pure helper that maps `QKeyEvent` key + `nativeScanCode` (evdev 34/35 and
  the Shift variants) to a brush-size/hardness delta, guarded to Brush/Pencil.
- [x] 1.4 **Regression**: fix `ImageView::blitRegion`
  (`crates/pictura-app/cpp/image_view.cpp`) so the present-cache patch composes
  the painter as scale-then-translate (or translates by the scaled origin), so
  the region lands at `(x*zoom, y*zoom)`; keep `presentCache_.valid = false` on
  the non-patch branch; keep the full-resolution write and the one-state-on-
  release commit unchanged.
- [x] 1.5 Regression test: new C++ check (codes from **346**) that drives
  `begin_paint` + several `paint_dab`s at a zoom **below and above** 100 %,
  grabs the canvas **before release**, asserts the dab is present at the right
  position, then asserts `end_paint` adds exactly one history state and undo
  restores the pre-stroke image. This check must fail on the current order.
- [x] 1.6 Item 21 regression: Rust unit test for the key→delta helper (US key
  and evdev scan codes, Shift variants, non-paint guard).
- [x] 1.7 Batch 1 gate: `cargo fmt --all`, `cargo clippy --workspace
  --all-targets -- -D warnings`, `cargo nextest run --workspace`,
  `cargo test --workspace --doc`; `cmake --build build`;
  `./build/pictura --headless --self-test`.

## 2. Batch 2 — Layers panel polish (items 3, 4, 6, 7, 8, 9, 10, 11, 12)

- [x] 2.1 Item 3: correct `LayerRowDelegate::nameRect`
  (`cpp/panels/layers_panel_internal.h`) to mirror paint exactly (the `+4` gap,
  the right-edge badge/mask caps) and floor it to a non-zero width; treat a
  content-band double-click outside the eye/chevron/thumbnail/lock/fx/mask
  controls as a rename (style no-op for non-Background), routed through
  `layers_panel.cpp`'s hit-test.
- [x] 2.2 Item 4: in `LayersModel::flags` return `Qt::ItemIsDropEnabled` for the
  invalid parent index; set a closed-hand cursor around `startDrag` and restore
  it after, in `cpp/panels/layers_panel.cpp`.
- [x] 2.3 Item 6: tint only `eyeRect`'s background with the color label
  (`layers_panel_internal.h` paint) and remove the post-name `labelSwatch` paint;
  keep `labelSwatch` for the row menu.
- [x] 2.4 Item 7: clip the style's selected-row paint to the row minus the eye
  column and repaint the eye column with the base colour before drawing the
  eye/tint, so the active highlight never covers the eye.
- [x] 2.5 Item 8: draw regular-layer thumbnails over a cached two-tone
  checkerboard; keep the group folder glyph and no checkerboard.
- [x] 2.6 Item 9: draw a 1 px black thumbnail outline for every thumbnail and
  white 1 px corner brackets one pixel outside it for the singular active layer
  (no brackets for zero/multiple).
- [x] 2.7 Item 10: add `LayerRowLinkedRole` (path in the frame's `link_sets`)
  and `LayerRowPlacedRole` (`Layer.smart_object` External/Alias) to the row
  projection; italicise `Background`, keep other names normal, underline
  linked/placed rows in the delegate.
- [x] 2.8 Item 11: add one named row-height constant with a floor of ~28 px used
  by both `sizeHint` and the delegate's centring math.
- [x] 2.9 Item 12: hide `lockNesting_` in the Layers panel
  (`cpp/panels/layers_panel.cpp`, `layers_panel_actions.cpp`); leave the engine
  `NESTING` rules and PSD flag untouched.
- [x] 2.10 Item 5: convert the Background to a normal layer through a
  name-and-color dialog, defaulting to the next free `Layer N` and no label
  (`cpp/panels/layers_panel.cpp` double-click and New-Layer drop, `cpp/panels/layer_new_dialog.{h,cpp}`
  name+color-only factory, bridge `set_layer_name_path`/`set_layers_color` or a
  combined op); cancelling leaves the Background unchanged and records nothing;
  each conversion is exactly one undo state.
- [x] 2.11 Batch 2 regression: new `crates/pictura-app/cpp/selftest_layers_round3.cpp`
  (register in `CMakeLists.txt`, invoked from `runLayersControlsChecks`, codes
  from **346+**) covering the rename band, top-level drop indicator + closed-hand
  cursor, Background conversion dialog (accept/cancel, `Layer N` default, label
  applied), eye-only tint with no name swatch, highlight clip, thumbnail
  checkerboard/outline/brackets, row roles/fonts, row-height floor, and the
  hidden nesting button.
- [x] 2.12 Batch 2 gate: fmt/clippy/nextest/doctests, `scripts/verify-full.sh`,
  headless self-test.

## 3. Batch 3 — Visibility perf + invisible layers (items 13, 17)

- [x] 3.1 Item 13: route the `set_layers_visible` path through the region fast
  path (`layer_visibility_region` → `refresh_region`) instead of always
  `batch_changed` + full `recomposite` + full-doc snapshot; fall back to
  `recomposite` only when a region is `None`; keep one history state.
- [x] 3.2 Item 13 regression: Rust unit test that a visibility toggle takes the
  region path (region blit fired, no full `changed`) and equals a full
  recomposite byte-for-byte; a `#[ignore]`d 4000² print-only profile; a C++
  check that the eye toggle on a large document takes the region path and meets
  the sub-1s target on the reference run (record the measured value).
- [x] 3.3 Item 17: add the shared `active_layer_visible` helper in
  `crates/pictura-app/src/cxxqt_object/`; refuse paint/filter edits on an
  invisible active layer with the Block/Forbidden cursor, while selection and
  copy stay available.
- [x] 3.4 Item 17: fix `compute_move_preview`
  (`crates/pictura-app/src/cxxqt_object/impl_transform.rs`) to save and restore
  the active layer's `visible` flag instead of forcing `visible = true`.
- [x] 3.5 Item 17 regression: Rust tests for the helper and the
  preview save/restore; C++ checks that paint on an invisible layer is refused
  with the Block cursor with no history state, and that a Move on an invisible
  layer translates and records one state while leaving it invisible.
- [x] 3.6 Batch 3 gate: fmt/clippy/nextest/doctests, `scripts/verify-full.sh`,
  headless self-test.

## 4. Batch 4 — Selection modifiers (items 14, 15, 16)

- [ ] 4.1 Item 14: in `cpp/tools.cpp::refreshCursor` gate the move-selection
  cursor on no Shift/Alt so the combine cursor wins while hovering an existing
  selection.
- [ ] 4.2 Item 15: capture the drag modifiers at press (`dragMods_`) in
  `cpp/tools_marquee.cpp` / the press handler and use them for the marquee
  geometry, the release raster, and the drag cursor, so releasing Shift/Alt
  mid-drag keeps the mode and geometry.
- [ ] 4.3 Item 16: keep Alt geometry-only (from-centre) driven by `dragMods_`
  and ensure it never pre-toggles a combine mode before a drag starts; note any
  spec conflict with the existing combine-mode wording in the change notes.
- [ ] 4.4 Item 14/15/16 regression: extend
  `crates/pictura-app/cpp/selftest_tools_selection.cpp` (or a round-3 TU) with
  the combine-cursor-under-modifier, released-modifier-retention, and
  Alt-no-pretoggle checks.
- [ ] 4.5 Batch 4 gate: fmt/clippy/nextest/doctests, `scripts/verify-full.sh`,
  headless self-test.

## 5. Batch 5 — Move nudge + Alt clone (items 18, 19)

- [ ] 5.1 Item 18: add plain-arrow 1 px and `Shift+Arrow` 10 px nudge for the
  Move tool through the existing `translate_layer`, one history state per nudge,
  refused for position-locked/Background/group/adjustment/no-document.
- [ ] 5.2 Item 19: add a Rust `begin_move_duplicate` (duplicate the active layer
  or the selected pixels, recomposite, `compute_move_preview`, no `record`);
  `commit_move` records one state; make the duplicate the active layer on commit
  and preview the clone during the drag.
- [ ] 5.3 Item 18/19 regression: Rust tests for `begin_move_duplicate` (clone
  inserted, no record at begin, one state at commit, clone active); C++ checks
  for the 1 px/10 px nudge and the Alt-drag clone preview/commit.
- [ ] 5.4 Batch 5 gate: fmt/clippy/nextest/doctests, `scripts/verify-full.sh`,
  headless self-test.

## 6. Batch 6 — Workspace zoom/scroll (items 20, 22, 23, 24)

- [ ] 6.1 Item 20: implement the transient Alt eyedropper in `cpp/tools.cpp`
  (sample + eyedropper cursor) with release restoring the previous tool and no
  history; add the tool-framework cursor branch.
- [ ] 6.2 Item 22: Shift doubles the wheel zoom step in
  `ImageView::wheelEvent`/`zoomAt`.
- [ ] 6.3 Item 23: apply the shared wheel modifier precedence — side-wheel pans
  horizontally, `Ctrl+Alt` pans vertically, `Alt` pans horizontally, otherwise
  zoom at the cursor.
- [ ] 6.4 Item 24: anchor the Zoom-tool click (`cpp/tools.cpp`) and the Navigator
  slider at the click/cursor point through `setZoom(zoom, anchor)`.
- [ ] 6.5 Item 20/22/23/24 regression: C++ checks for the transient eyedropper
  sample/cursor/restore, the Shift step, the three wheel pan cases, and
  cursor-anchored Zoom-click/Navigator zoom.
- [ ] 6.6 Batch 6 gate: fmt/clippy/nextest/doctests, `scripts/verify-full.sh`,
  headless self-test.

## 7. Verification and gates

- [ ] 7.1 `cargo fmt --all` (CI runs `--check`), `cargo clippy --workspace
  --all-targets -- -D warnings`.
- [ ] 7.2 `cargo nextest run --workspace` and `cargo test --workspace --doc`.
- [ ] 7.3 `bash scripts/verify-full.sh`; every `scripts/file-size-allowlist.txt`
  ceiling holds; each new `selftest_*.cpp` stays under its cap and
  `crates/pictura-app/cpp/selftest.cpp` stays at **6729 LOC**.
- [ ] 7.4 `./build/pictura --headless --self-test` and record the
  passed/failed/skipped counts; confirm new codes start at **346** and no
  existing code was reused.
- [ ] 7.5 `openspec validate polish-app-ui --strict` and
  `openspec validate --all --strict`; both must report valid.
- [ ] 7.6 No `docs/` change in this change; a docs edit would be a separate
  `TASK-ALLOWS-DOCS` commit.

## 8. Explicitly not done / ceilings

- [ ] 8.1 No resident GPU layer sources, tiling, display-time LoD, or
  GPU-resident zero-copy present.
- [ ] 8.2 No compositor math, ±1 LSB parity, or PSD/PSB format change; the
  region fast path must remain byte-identical to a full recomposite.
- [ ] 8.3 No Layer Style dialog: a non-Background content-band double-click
  stays a documented no-op.
- [ ] 8.4 The Layers panel hides the nesting-lock button only; the engine
  `NESTING` refusal and PSD flag are unchanged and stay tested.
- [ ] 8.5 Wheel `Shift`/`Alt`/`Ctrl+Alt` combinations beyond the specified cases
  (for example Ctrl-only zoom acceleration) are not added.
- [ ] 8.6 No new crate, dependency, or `docs/` change; `selftest.cpp` does not
  grow.
