## Why

Smart objects can be created (Convert, Place), swapped (Replace), extracted
(Export), opened as a new document, and dissolved (Rasterize), but the flagship
action is still missing: editing the embedded source and re-embedding it.
`Layer > Smart Objects > Edit Contents` is a disabled leaf
(`crates/pictura-app/cpp/command_tree.cpp:379`), and Adobe's action opens the
object's source in an editor and updates the host when that editor is saved.
Without it a user cannot round-trip an object's contents in the app at all.

## What Changes

- Add a read-only eligibility probe `layer_can_edit_smart_object_contents(path)
  -> bool`: `true` exactly when `smart_object_source_bytes(doc, path)` yields a
  non-empty payload AND `pictura_codec::read_psd(payload)` parses it as a
  PSD/PSB. A non-parsing source (a placed JPEG) is not editable in-app, so the
  command is disabled and records nothing.
- Add the bridge `PictureView::commit_smart_object_edit(layer_path, file_path) ->
  bool`. It runs the same engine swap as `replace_smart_object_contents` (shared
  helper, no duplicated engine logic) and on success clears the link sets,
  recomposites, and records exactly one `"Edit Contents"` history state. The
  existing `replace_smart_object_contents` keeps its signature, behavior, and
  `"Replace Contents"` label, and its tests are unchanged.
- Add the command `Layer > Smart Objects > Edit Contents` (stable id
  `LayerSmartObjectEditContents` = `"layer.smartObject.editContents"`), enabled
  only for an editable smart object, mirroring the path resolution of the other
  Smart Objects commands.
- Edit Contents opens the payload as a NEW untitled editor tab. It reuses only
  existing bridge calls through a per-session temporary file: export the payload
  byte-for-byte with `export_smart_object_contents`, construct a new
  `PictureView`, load the file as an ordinary document with `open(path)`, then
  `addDocument(editor, QString())`. The editor holds the decoded source as a
  normal document, not a document wrapping the source as a smart object. Opening
  records no history state on the origin and does not change it.
- Intercept Save for a session editor in `PicturaMainWindow::saveActive()`:
  `editor->save(temp_file)` then `origin->commit_smart_object_edit(layer_path,
  temp_file)`, refresh, and return success with no Save dialog and no user file
  written. `saveActiveAs` is left unchanged. The editor tab stays open and is
  marked clean.
- Track a small `SmartObjectEditSession` list in `PicturaMainWindow` (editor,
  origin, layer path, filename, temp dir). Closing an editor drops its session
  and temp dir; closing an origin drops every session targeting it and leaves
  the orphaned editor open as a plain untitled document. Discarding an editor
  leaves the origin unchanged and records nothing.
- Extend the C++ self-test suite `selftest_layers_smart_object.cpp` with exit
  code **288** and add engine/bridge coverage. No new `.cpp`/`.h`, so
  `CMakeLists.txt` is unchanged; no new dependencies.

## Capabilities

### New Capabilities

<!-- None: the action extends the existing smart-object-layer-actions capability. -->

### Modified Capabilities

- `smart-object-layer-actions`: adds requirements for the Edit Contents
  eligibility predicate, opening the source as a new untitled editor document,
  committing the edited source on Save with exactly one `"Edit Contents"` undo
  state, and the discard/close session lifetime. The existing convert, place,
  rasterize, replace, open-as, and export requirements are unchanged.

## Impact

- `crates/pictura-app/src/cxxqt_object/impl_layers_smart_object.rs` bridge
  (shared source-apply helper, the eligibility probe, and
  `commit_smart_object_edit`) and the `PictureView` declarations in
  `crates/pictura-app/src/cxxqt_object.rs`.
- `crates/pictura-app/cpp/frame.{h,cpp}` for `SmartObjectEditSession`, the
  session entry point, the Save interception, and cleanup.
- `crates/pictura-app/cpp/commands.h`, `command_tree.cpp`, and `frame_menus.cpp`
  for the command id, registration, handler, and enabled provider.
- `crates/pictura-app/cpp/selftest_layers_smart_object.{cpp,h}` (existing
  suite, no new file).
- `pictura-codec` is already a direct dependency of `pictura-app`, so the
  probe needs no manifest change. No new dependencies.

## Non-Goals

Deferred deliberately; each is a separate follow-up:

- External editors (Illustrator for vector PDF, Camera Raw for raw).
- Editing vector/PDF sources: only a PSD/PSB payload can be opened in-app, so a
  non-parsing source stays disabled.
- Linked (external) objects.
- The double-click-thumbnail trigger: this change ships the menu command only.
- New Smart Object via Copy and Stack Mode, which stay disabled placeholders.
- Changing `File > Save As` behavior for editors; editors use Save.
