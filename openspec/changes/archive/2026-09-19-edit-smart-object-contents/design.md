## Context

Every smart-object lifecycle action except Edit Contents is shipped:
`convert_to_smart_object`, `place_smart_object`, `rasterize_smart_object`,
`replace_smart_object_contents`, `open_as_smart_object`, and the read-only
`smart_object_source_bytes` accessor all live in
`crates/pictura-render/src/document_ops/layer_ops/smart_object.rs`. Each app
command follows the same wiring: a `command_ids` constant, a
`registry.add(...)` entry in `command_tree.cpp`, a bridge method in
`impl_layers_smart_object.rs`, and a handler plus enabled provider in
`frame_menus.cpp`. The Smart Objects handlers resolve their target layer path
from `layersPanel_->currentPath()` and gate it on a read-only `layer_can_*`
predicate (`frame_menus.cpp:585-690`).

`Layer > Smart Objects > Edit Contents` is a disabled leaf
(`crates/pictura-app/cpp/command_tree.cpp:379`). The app is multi-document:
`PicturaMainWindow` owns `QList<DocEntry> docs_` and a `QTabWidget* tabs_`
(`frame.h:182-213`); each document is a cxx-qt `PictureView`. `frame.cpp`
already has `addDocument`, `openPath`, `saveActive`, `saveActiveAs`,
`closeDocument`, and `removeDocument`. The bridge already exposes exactly the
primitives Edit Contents needs: `export_smart_object_contents(layer_path, dest)`
writes the payload byte-for-byte, `open(path)` loads a PSD as an ordinary
document, `save(path)` serializes the current document, and
`replace_smart_object_contents(layer_path, file_path)` swaps the embedded source.

The CS6 contract (`docs/05-layers/smart-objects.md:40`) says Edit Contents opens
the source and updates the host when it is saved. Only a PSD/PSB payload can be
opened as a document in this app, so a non-PSD source (a placed JPEG) stays
disabled.

## Goals / Non-Goals

**Goals:**

- One eligibility predicate, one commit bridge, and one app command that opens
  an object's source in a new untitled editor tab and re-embeds it on Save.
- Saving the editor swaps the origin's embedded source and records exactly one
  `"Edit Contents"` undo state; discarding leaves the origin untouched.
- Small diff: reuse the tested bridge primitives, add no new byte-crossing FFI,
  add no new files, and leave `replace_smart_object_contents` and its tests
  exactly as they are.

**Non-Goals:**

- External editors, vector/PDF sources, linked (external) objects, the
  double-click-thumbnail trigger, New Smart Object via Copy, and Stack Mode.
- Changing `File > Save As` behavior for editors.

## Decisions

### D1. Reuse the existing bridge through a per-session temporary file

`crates/pictura-app/cpp/frame.h`

```
struct SmartObjectEditSession {
    PictureView* editor = nullptr;   // the new untitled tab
    PictureView* origin = nullptr;   // the document that owns the layer
    QString layerPath;               // path of the smart-object layer in origin
    QString filename;                // display name for the swapped source
    QTemporaryDir* temp = nullptr;   // owns the temp file for the session
};
QList<SmartObjectEditSession> editSessions_;
```

Edit Contents performs, in order:

1. `origin->export_smart_object_contents(layerPath, tempFile)` — the payload is
   written byte-for-byte by the existing, tested export path.
2. `auto* editor = new PictureView(this); editor->open(tempFile);` — the source
   loads as an ordinary document (its own layers and composite), **not** as a
   document wrapping the source as a smart object.
3. `addDocument(editor, QString())` — an untitled tab, exactly like
   `openAsSmartObjectPath` (`frame.cpp:327`).
4. Append a `SmartObjectEditSession` owning the `QTemporaryDir`.

Save on the editor (D4) writes the edited document back to the same temp file
with `editor->save(tempFile)` and hands it to `origin->commit_smart_object_edit`.

<!-- ponytail: reusing the export/open/save bridge through a session temp file
     avoids adding new byte-crossing FFI (no `open_editor_from_bytes` /
     `commit_edit_bytes` pair). Every byte already crosses an existing, tested
     API, so the whole feature is C++ orchestration plus a label. Ceiling: one
     disk round-trip per save; revisit only if a save ever measures hot. -->

The temp file lives in a `QTemporaryDir` whose destructor removes the directory,
so a session that is never saved still cleans up. `QTemporaryDir` is already
used in `main.cpp` and `selftest.cpp`; no new dependency.

### D2. Eligibility is an `Embedded` object with a non-empty payload that parses as PSD/PSB

Add a read-only bridge predicate
`layer_can_edit_smart_object_contents(&self, path: &QString) -> bool`, which
delegates to the engine
`pictura_render::can_edit_smart_object_contents(doc, path)`. The engine defines
it as `can_replace_smart_object_contents(doc, path)` (non-group,
non-adjustment, kind `Embedded`, non-empty payload) AND
`smart_object_source_bytes(doc, path)` parsing as a PSD/PSB via `read_psd`.
Sharing the replace predicate keeps eligibility consistent with the commit —
`replace_smart_object_contents` also requires kind `Embedded` — so a
hypothetical non-`Embedded` object carrying a payload can never be enabled and
then refused. The added `read_psd` check is what separates "can be exported"
from "can be edited in-app": a placed JPEG has a payload export can write, but
it is not a PSD/PSB document, so Edit Contents must refuse it rather than open a
tab that cannot load. `pictura-codec` is already a direct dependency of
`pictura-app`.

### D3. The commit bridge shares the replace engine path and only changes the label

`replace_smart_object_contents` and `commit_smart_object_edit` differ only in
the recorded history label. Factor the common body into one private helper in
`impl_layers_smart_object.rs`, e.g.
`apply_smart_object_source(self, path, file_path, label) -> bool`: read the file,
derive the display name from its base name, call
`pictura_render::replace_smart_object_contents`, and on success
`clear_link_sets()`, `recomposite()`, and `record(label)`. Then:

- `replace_smart_object_contents(path, file_path)` delegates with
  `"Replace Contents"` — same signature, behavior, and label as today, so its
  Rust and C++ tests are untouched.
- `commit_smart_object_edit(path, file_path)` delegates with `"Edit Contents"`.

No engine logic is duplicated and no new engine operation is introduced.

### D4. Save interception lives at the top of `saveActive()`

`PicturaMainWindow::saveActive()` currently resolves the active `DocEntry` path
(or opens Save As) and calls `saveActiveAs`. Insert a session check first: when
the active view is the `editor` of a session, do

```
editor->save(tempFile);                 // serialize the edited document
origin->commit_smart_object_edit(layerPath, tempFile);
refresh();
return true;
```

and return without touching `saveActiveAs`, without a `QFileDialog`, and without
writing any user-chosen file. The editor's internal path points at the temp
file, but the tab's `DocEntry.path` is empty (it was added untitled), so nothing
else can accidentally write a user file. `saveActiveAs` is intentionally
unchanged; editors use Save (Ctrl+S).

### D5. Entry point and command wiring mirror the sibling Smart Objects commands

- `frame.h` declares `bool editSmartObjectContents(const QString& layerPath);` in
  the public document-operations block, implemented in `frame.cpp` beside
  `openAsSmartObjectPath`. It takes the already-resolved layer path so the
  self-test can drive it directly; the menu handler resolves the path.
- `commands.h`:
  `inline constexpr char LayerSmartObjectEditContents[] = "layer.smartObject.editContents";`.
- `command_tree.cpp:379`: replace the disabled `{Layer, Smart Objects, Edit
  Contents}` leaf with `registry.add(command_ids::LayerSmartObjectEditContents,
  {"Layer", "Smart Objects", "Edit Contents"}, QStringLiteral("Edit Contents"),
  QKeySequence(), true);`.
- `frame_menus.cpp`: add `currentEditableSmartPath`, mirroring
  `currentReplaceableSmartPath`, that returns
  `layersPanel_->currentPath()` only when
  `view->layer_can_edit_smart_object_contents(path)`; set the handler to call
  `editSmartObjectContents(path)`, and the enabled provider to require a
  non-empty path. No file dialog.

### D6. Session lifetime rules

- **Discard (editor closed without saving):** `closeDocument`/`removeDocument`
  finds the session whose `editor` is the removed view, deletes the session and
  its `QTemporaryDir`, and does nothing to the origin. No history state.
- **Origin closed:** drop every session whose `origin` is the removed view. The
  orphaned editor tab stays open as a plain untitled document; because its
  `DocEntry.path` is empty, a later Save routes to Save As and never writes the
  deleted temp file.
- **Frame destroyed:** `editSessions_` is destroyed with the window and each
  `QTemporaryDir` removes its directory.
- Multiple editors for the same layer are allowed (each with its own temp);
  last save wins, matching Photoshop's independent edit windows. This is a
  deliberate simplification, not a guarantee of serialized edits.

Cleanup runs in `removeDocument` (which `closeDocument` calls after the unsaved
prompt), so the temp is dropped exactly when the tab disappears.

### D7. Verification

- **Rust eligibility test** in `crates/pictura-app/src/cxxqt_object/tests.rs`
  (alongside `save_after_edit_serializes_the_current_composite`): `PictureView`
  is a C++-constructed QObject (cxx-qt 0.10 exposes no Rust constructor; see the
  note in `cxxqt_object/tests_impl.rs`), so the `commit_smart_object_edit`
  bridge cannot be driven from a Rust test. The added test covers the pure
  eligibility predicate `can_edit_smart_object_contents` only: an `Embedded`
  PSD/PSB payload is editable; a non-parsing payload, an empty/absent payload, a
  group, an adjustment layer, a non-`Embedded` kind, and an unresolved path are
  refused; and the predicate mutates nothing.
- **Commit bridge coverage** is end-to-end in C++ self-test 288 (open, edit,
  save, exactly one `"Edit Contents"` state, origin recomposite, discard) plus
  the existing engine tests for the shared `replace_smart_object_contents`
  operation (`replace_swaps_source_and_drops_preserved_blocks`,
  `replace_clears_proxy_and_renders_new_source`,
  `replace_refuses_ineligible_and_malformed_targets` in
  `crates/pictura-render/src/tests/smart_object.rs`), which the bridge calls
  unchanged.
- **C++ self-test** `selftest_layers_smart_object.cpp`: extend the existing
  suite with exit code **288** (287 is the current maximum; codes are
  append-only). Create an origin with a converted smart object, call
  `frame.editSmartObjectContents(layerPath)`, assert one new untitled editor
  tab whose document is the decoded source and whose history is untouched on the
  origin, then modify the editor through an existing bridge call, set it active,
  call `frame.saveActive()`, and assert the origin shows exactly one
  `"Edit Contents"` history state with the new payload; then open a second
  editor, modify it, close it without saving, and assert the origin is
  unchanged. Assert a placed non-PSD / ineligible layer is refused and adds no
  tab. Clean up temp files. A second check **289** covers the session lifetime:
  closing an editor deletes the session temp file; closing the origin leaves the
  orphaned editor open as an untitled tab and drops the session temp.
- No new `.cpp`/`.h`, so `CMakeLists.txt` is unchanged.

## Risks / Trade-offs

- **Disk round-trip on every save.** → Accepted; the save is user-initiated and
  the file is local. Named in D1 as the deliberate ceiling of the lazy bridge.
- **The editor's internal path points at the temp file.** If the origin closes
  first and the temp dir is removed, an editor Save must not target the deleted
  path. → The tab's `DocEntry.path` is empty, so `saveActive` takes the Save As
  branch and never writes the temp path; the interception is only active while a
  live session exists.
- **Save recursion.** The editor Save path must not fall through to
  `saveActiveAs`/`view->save` on a user path. → The session check is the first
  branch in `saveActive` and returns, so `saveActiveAs` is unreachable for an
  editor.
- **Two editors for one layer.** Independent sessions can commit out of order;
  the later save wins. → Documented as a deliberate simplification; Photoshop
  behaves the same way and the only cost is a stale first editor.
- **Mutating a shared helper could change Replace behavior.** → The helper is
  extracted as a pure move parameterized only by its label; the Replace
  requirement and its existing tests pin the old signature, behavior, and
  `"Replace Contents"` label.
- **Exit-code discipline.** Take 288 (after the used 287), keep the check inside
  the existing `selftest_layers_smart_object` suite, and stay within the
  file-size allowance.
