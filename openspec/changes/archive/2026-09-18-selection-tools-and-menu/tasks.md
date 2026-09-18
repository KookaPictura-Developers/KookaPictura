## 1. Elliptical Marquee + marquee options

- [ ] 1.1 Flip `EllipticalMarquee` to `implemented = true` in `tools.cpp` `kToolTable` and add it to `implementedToolIds()`; update its hint text from "not implemented yet".
- [ ] 1.2 Add `feather` (f64) to the `select_ellipse` bridge in `impl_selection.rs` and apply `Selection::feather(r)` to the rasterised ellipse before `combine_with`.
- [ ] 1.3 Add `EllipticalMarquee` cases to `ToolController::handlePressed/handleMoved/handleReleased` mirroring `Marquee`, calling `select_ellipse` on release and emitting `selectionCommitted()`.
- [ ] 1.4 Add selection option fields to `ToolController` (feather 0, style Normal, fixed ratio 1:1, fixed size 100x100) with accessors; apply Style constraining to the drag rectangle before the bridge call.
- [ ] 1.5 Replace the bare Marquee/EllipticalMarquee options page with `buildSelectionPage`: mode buttons, Feather (0-250, decimal), Style combo with conditional Fixed Ratio / Fixed Size fields, and an Anti-alias checkbox shown for the ellipse only, disabled with a reason.
- [ ] 1.6 Document the anti-alias and fixed-ratio/size defaults as inferred ceilings in the options-page tooltips and a `ponytail:` comment.

## 2. Polygonal Lasso + lasso options

- [ ] 2.1 Flip `PolygonalLasso` to `implemented = true` and add it to `implementedToolIds()`; update its hint text.
- [ ] 2.2 Add polygonal vertex state to `ToolController` (`QPolygonF polygonPoints_`, in-progress flag) and cases in pressed/moved/released: press appends a vertex (calling `begin_lasso`/`lasso_add_point`), move previews `points + cursor`, close and `end_lasso()` on first-vertex click, double-click, and Enter if the frame key path is a one-liner.
- [ ] 2.3 Handle `Esc` to clear the in-progress polygonal path without changing the document selection.
- [ ] 2.4 Give the Lasso and Polygonal Lasso options page mode buttons, Feather, and a disabled Anti-alias checkbox with a reason; wire Feather through the lasso commit path.
- [ ] 2.5 Keep `MagneticLasso` `implemented = false`; document the deferral reason (no edge map / fastening-point tracker) in its tool hint.

## 3. Magic Wand + wand options

- [ ] 3.1 Flip `MagicWand` to `implemented = true` and add it to `implementedToolIds()`; update its hint text.
- [ ] 3.2 Add `contiguous: bool` (and an anti-alias flag) to the `magic_wand` bridge; pass `contiguous` to `pictura_select::magic_wand` while keeping `current_buffer` as the sample source.
- [ ] 3.3 Add a `MagicWand` press case to `ToolController` that calls `magic_wand(x, y, tolerance, contiguous)` and emits `selectionCommitted()` on success.
- [ ] 3.4 Add wand option fields (`contiguous_ = true`, `antiAlias_ = true`, `sampleAllLayers_ = true`) and build the wand options page: mode, Tolerance, Contiguous checkbox, and disabled Anti-alias / Sample All Layers checkboxes with reasons.
- [ ] 3.5 Update the Quick Selection options page to show mode (New/Add/Subtract only), Tolerance, and disabled Sample All Layers / Auto-Enhance checkboxes with reasons; note the brush pop-up and Refine Edge are deferred.

## 4. Select menu operations + bridge

- [ ] 4.1 Add `select.reselect`, `select.inverse`, `select.modify.border|smooth|expand|contract|feather`, `select.grow`, `select.similar`, `select.save`, `select.load`, `select.allLayers`, `select.deselectLayers`, and `select.similarLayers` ids to `commands.h`.
- [ ] 4.2 Replace the matching `leaf(...)` rows in `command_tree.cpp` with `registry.add(id, path, label, shortcut, true)`; leave Color Range, Refine Edge, and Transform Selection as `leaf(...)`.
- [ ] 4.3 Add the `deselected_selection` field and `reselect`, `invert_selection`, `modify_selection`, `grow_selection`, `similar_selection`, `save_selection`, `load_selection`, and `select_all_layers` bridge methods in `impl_selection.rs`; populate `deselected_selection` in `deselect()` and New-mode replacement.
- [ ] 4.4 Register handlers in `frame_menus.cpp` for Reselect, Inverse, Modify*, Grow, Similar, Save, Load, All Layers, Deselect Layers, and Similar Layers, reusing `QInputDialog` for parameters and channel choice; implement layer commands via `LayersPanel::selectPaths` + `PictureView::select_similar`.
- [ ] 4.5 Register enable providers: document-required for the group, selection-required for Reselect/Inverse/Modify*/Grow/Similar/Save, channels-non-empty for Load, current-layer for Similar Layers, and refresh after each applied command.
- [ ] 4.6 Ensure one `record(...)` per applied command and none on refusal/cancellation; add a `ponytail:` note for the positional `Alpha N` channel-name mapping (Channel has no name field).

## 5. Verification

- [ ] 5.1 Add Rust tests for any new `pictura-select`-facing logic that can run without Qt (feather-before-combine geometry, modify dispatch bounds); run `cargo nextest run -p pictura-select -p pictura-app`.
- [ ] 5.2 Add app self-tests with the next free codes starting at 242: elliptical marquee commit, polygonal lasso commit/cancel, magic wand contiguous toggle, inverse + reselect, each Modify operation, grow/similar, save/load round trip, and layer selection commands.
- [ ] 5.3 Add self-tests for the enable/disable and refusal semantics (no document, no selection, wrong-length channel) asserting the history count does not change on refusal.
- [ ] 5.4 Reuse `scripts/select_oracle.py` for the Modify morphology and feather differential checks; note that no external oracle exists for polygon/ellipse rasterisation.
- [ ] 5.5 Run `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `bash scripts/verify-fast.sh`, and `./build/pictura --headless --self-test`; record results.
