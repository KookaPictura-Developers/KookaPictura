## 1. Brief and frozen interfaces (orchestrator)

- [x] 1.1 Write `docs/dev/m39-panel-anatomy.md`: the CS6 anatomy cited from
  `LAY-002`/`PAN-001`; the current-state inventory; the frozen path grammar;
  the bridge read/write/batch/create API; the multi-selection refusal table;
  the solo snapshot semantics; the Panel Options defaults and thumbnail-contents
  rule; the model/delegate/menu/tooltip design; the drag-reorder deferral; the
  staged tasks and non-goals
- [x] 1.2 Write `proposal.md`, `design.md`, this `tasks.md`, and the
  `specs/layers-panel/spec.md` delta (four MODIFIED + eight ADDED requirements)
- [x] 1.3 Freeze in `design.md`: the path grammar and projection; the
  top-level-wrapper decision; the bridge signatures; the per-node scan/refuse
  rule; the solo snapshot ownership; the one-visible-column model; the delegate
  badges and fallback; the Panel Options defaults and session schema v3; the
  minimal wired menus; the drag-reorder deferral
- [x] 1.4 Update `docs/dev/STATE.md`: M39 proposed, the layers-panel program
  line, and the next milestone
- [x] 1.5 Validate: `openspec validate m39-panel-anatomy --strict` and
  `openspec validate --all --strict`

## 2. pictura-render: path/tree core

- [x] 2.1 Add path helpers to
  `crates/pictura-render/src/document_ops/layer_ops.rs`:
  `resolve_path(&Document, &str) -> Option<&Layer>`,
  `resolve_path_mut(&mut Document, &str) -> Option<&mut Layer>`, a
  `flatten_rows(&Document) -> Vec<(String, u32)>` (depth-first, topmost-first
  `(path, depth)` pairs) and `parent_path`, plus `is_background` as the single
  source of truth (the app's `is_background_layer` delegates); malformed/
  out-of-range paths return `None` and never panic
- [x] 2.2 Add the batch ops over `&mut Document`: `set_visible_paths`,
  `apply_visibility`, `set_blend_paths`, `set_opacity_paths`,
  `set_fill_paths`, `set_lock_paths`, `set_color_paths`, `delete_paths`,
  `duplicate_paths`, `group_paths`, `ungroup_paths`, `rename_path`,
  `move_path`, each returning the number changed (or the new path) and applying
  the per-node refusal table; `add_layer_in`/`add_group_in` for tree-aware
  insertion
- [x] 2.3 No top-level compatibility wrappers were added: the bridge keeps
  every legacy top-level `layer_*(i)`/`set_layer_*(i)`/M37 op **byte-identical**
  (the frozen legacy surface) and the new path surface is the tree
  implementation (the M37 ops stay; no Rust top-level wrappers needed)
- [x] 2.4 Re-export the new items from `document_ops/mod.rs` and the crate root
- [x] 2.5 Unit tests: `path_resolve_nested`, `path_resolve_rejects` (empty,
  leading zero, sign, trailing slash, out-of-range), flatten order/depth for a
  group with children, each batch op's refusal skip (Background, fully locked,
  group-fill), `group_paths` whole-op refusal, `apply_visibility` exactly-these
  semantics, and `add_layer_in` inside a group
- [x] 2.6 `cargo test -p pictura-render`; `cargo fmt`/`clippy` clean

## 3. Bridge tree API

- [x] 3.1 Add the `#[qinvokable]` declarations and implementations for
  `layer_row_count` and the `layer_row_*` getters (path, depth, name, kind,
  visible, blend, opacity, fill, lock, color, clipping, has_mask,
  has_adjustment, expandable, child_count, thumbnail, mask_thumbnail)
- [x] 3.2 Add the path single mutators and the `QStringList` batch mutators
  with `QStringList` imported from `cxx_qt_lib`; each successful call
  `recomposite()` then `record(<label>)`, a zero-change call records nothing
  and emits no `changed`
- [x] 3.3 Leave every existing top-level `layer_*(i)` method
  **byte-identical** rather than re-routing it to path `"<i>"`: its current
  signature and refusal result are preserved (including
  `set_layer_visible`/`remove_layer` void returns), nothing calls the legacy
  surface after the panel migration, and the deliberate asymmetry is recorded —
  legacy `move_layer` still swaps unconditionally while `move_layer_path`
  refuses the Background and fully-locked layers (LAY-002 ruling)
- [x] 3.4 Add `add_layer_in`/`add_group_in` for tree-aware insertion; leave the
  M37 `add_layer(above)`/`add_group(above)` byte-identical (frozen legacy)
- [x] 3.5 `cargo check -p pictura_app` and `cargo test --workspace` (existing
  bridge tests unchanged)

## 4. Tree model, delegate, and badges

- [x] 4.1 Promote `LayersModel` to `QAbstractItemModel`: build the tree from
  the flat `layer_row_*` projection (parent = path minus the last segment),
  expose the frozen role enum, `DisplayRole`/`EditRole`/`ToolTipRole`/
  `CheckStateRole`, and `setData` for the eye and the name (routed to the
  path/batch bridge methods)
- [x] 4.2 Add `LayerRowDelegate`: paint the eye, thumbnail or group folder
  glyph, name, color swatch, clip indent + base underline, mask thumbnail, and
  `fx` badge (when `HasAdjustmentRole`); handle the eye hit-test; `sizeHint`
  follows the thumbnail size; omit a badge whose asset is null
- [x] 4.3 Configure the `QTreeView`: root decorated, items expandable,
  `ExtendedSelection`, `SelectRows`, `setUniformRowHeights(true)`,
  `setDragEnabled(false)`; keep the M38 strip objectNames and drop the two Move
  text buttons
- [x] 4.4 Panel-side expansion state (`QSet<QString>` of expanded group paths,
  default collapsed, a new group expanded) and re-selection by path after
  `refresh()`
- [x] 4.5 Header controls read the current row and enable from the selection;
  applying them calls the matching batch method

## 5. Multi-selection, solo, rename, tooltips

- [x] 5.1 Wire the header controls (visibility, lock, blend, opacity, fill,
  color) and Delete/Group/Ungroup/Duplicate to the batch methods over the
  `QItemSelectionModel` selection, one undo step each
- [x] 5.2 Implement the per-node refusal table and verify a mixed selection
  skips ineligible nodes without failing the batch
- [x] 5.3 Implement solo: `Alt`-click snapshot (`QHash<QString,bool>`) +
  `apply_visibility(paths, "Solo Visibility")`; a second `Alt`-click restores
  with `"Restore Visibility"`; clear the snapshot on document switch and before
  structural ops
- [x] 5.4 Implement inline rename with `Tab`/`Shift+Tab` commit-and-move over
  the visible rows without wrapping
- [x] 5.5 Set every row's `ToolTipRole` to `"<name> (<kind>)"`

## 6. Panel Options and menus

- [x] 6.1 Add the `Panel Options…` dialog (thumbnail size, thumbnail contents,
  Expand New Effects) with defaults Medium / Entire Document / on
- [x] 6.2 Extend `session.{h,cpp}` to schema v3 with `layersThumbSize`,
  `layersThumbContents`, `layersExpandNewEffects`; older/missing values load the
  defaults; wire the loaded values into the delegate and thumbnails
- [x] 6.3 Implement the panel menu (`layersPanelMenu`) and the row context menu
  with the wired commands only; move the M36 color-label menu into a `Color
  Label` submenu; add the eye right-click show-only/show-all; no unimplemented
  command shown
- [x] 6.4 Move reordering to `move_layer_path` from the menus and confirm the
  strip is exactly the CS6 seven

## 7. Drag-reorder — explicitly deferred

- [x] 7.1 Done as deferred: no drag-reorder shipped in M39 —
  `setDragEnabled(false)`, no `dropMimeData`; reordering is menu-only. The M41
  drop rules are recorded (no locked/Background displacement, groups accept
  children, no drop into a descendant, illegal targets not highlighted) in the
  brief and design

## 8. Tests and self-test

- [x] 8.1 C++ `m39_tree` (exit codes 102–103): projection order/depth/paths,
  add-inside-group, nested rename by path
- [x] 8.2 C++ `m39_multi` (104–105): a two-path blend batch is one undo step and
  skips the Background; a group fill is skipped; multi-selection group/ungroup
  is one step
- [x] 8.3 C++ `m39_solo` (106): solo hides all others, a second `Alt` restores
  exactly, and undo restores
- [x] 8.4 C++ `m39_rename` (107): `Tab` commits and moves down, `Shift+Tab` up,
  no wrap
- [x] 8.5 C++ `m39_options` (108): defaults Medium / Entire Document / on and a
  session round-trip
- [x] 8.6 C++ `m39_badges`, `m39_menus`, `m39_tooltip`, `m39_strip`
  (109–112): mask/fx/clip badges, panel/row menu contents and color label,
  tooltips, and the seven-button strip with no Move buttons
- [x] 8.7 `xvfb-run -a ./build/pictura --self-test` and the `two_layers.psd`
  variant, both exit 0; confirm every earlier check (m20–m38) is unchanged

## 9. Verification and close-out

- [x] 9.1 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets --
  -D warnings`; `cargo test --workspace`
- [x] 9.2 `cmake -S . -B build && cmake --build build`
- [x] 9.3 `openspec validate m39-panel-anatomy --strict`;
  `openspec validate --all --strict`
- [x] 9.4 Record the M39 result in `docs/dev/STATE.md`
- [ ] 9.5 Archive the change (`openspec archive m39-panel-anatomy`) and commit
  — deferred; this task does not commit
