## Context

A smart object stores its embedded source as `SmartObject.payload`
(`crates/pictura-core`) and the engine has one operation per lifecycle action
in `crates/pictura-render/src/document_ops/layer_ops/smart_object.rs`:
`convert_to_smart_object`, `place_smart_object`, `replace_smart_object_contents`,
`rasterize_smart_object`, and `open_as_smart_object`. Each follows the same
shape: resolve the path with `resolve_path`, gate on a read-only `can_*`
predicate, and refuse without mutating.

`Layer > Smart Objects > Export Contents…` is a disabled leaf
(`crates/pictura-app/cpp/command_tree.cpp:379`). The payload is already read
elsewhere through the typed object (`layer_smart_object_state`), but there is no
way to write it to disk, and no engine function exposes the bytes. The app
commands follow the pattern: a `command_ids` constant, a `registry.add` entry in
`command_tree.cpp`, a bridge method in `impl_layers_smart_object.rs`, and a
handler plus enabled provider in `frame_menus.cpp`.

The CS6 contract (`docs/05-layers/smart-objects.md`) says Export Contents writes
the source in its original placed format, and a layer-created object exports as
PSB. This change writes the stored payload byte-for-byte and does not transcode:
a converted-from-layers object exports the PSD bytes that `write_psd` authored.
That format rule is deferred.

## Goals / Non-Goals

**Goals:**

- One pure engine accessor that returns the embedded payload for an eligible
  layer as an owned `Vec<u8>`, or `None`.
- One bridge that writes those bytes to a caller-chosen destination and reports
  whether the write succeeded.
- One app command with a stable id, correct availability, a `*.psd` save
  dialog, and no undo state because export only reads.

**Non-Goals:**

- Edit Contents; the interactive transform session; non-PSD/PSB transcoding on
  export; linked (external) objects; New Smart Object via Copy.

## Decisions

### D1. The accessor lives beside the other smart-object ops and is a pure read

`crates/pictura-render/src/document_ops/layer_ops/smart_object.rs` gains
`pub fn smart_object_source_bytes(doc: &Document, path: &str) -> Option<Vec<u8>>`,
registered in `layer_ops/mod.rs` and re-exported from `document_ops/mod.rs` and
`lib.rs`, matching the other ops. Eligibility is exactly: `resolve_path`
succeeds, the layer is not a group, `adjustment` is `None`, and
`smart_object.as_ref()` has a payload that is `Some` and non-empty. On success
it returns `payload.clone()`. There is no `kind` check because the contract is
"the layer carries a payload"; a non-embedded object with a payload is still
exportable, and one without a payload returns `None`.

It takes `&Document` rather than `&mut Document` and performs no mutation, so it
can back a read-only availability predicate and the self-test can call it
without changing history.

### D2. The bridge writes byte-for-byte with `std::fs::write`

`PictureView::export_smart_object_contents(path: &QString, dest: &QString) ->
bool` in `impl_layers_smart_object.rs`: resolve the payload through the engine
accessor on the current document; when it is `Some`, call `std::fs::write(dest,
bytes)` and return whether that succeeded; when it is `None`, return `false`.
`std::fs::write` truncates and overwrites, matching the save-dialog semantics.
No encoding step runs: the file holds the payload exactly as stored, so the
source's original format is preserved without the app knowing it.

A read-only predicate `layer_can_export_smart_object_contents(path: &QString) ->
bool` mirrors `layer_can_replace_smart_object_contents` and consults the
accessor, so the enabled provider calls the same eligibility logic as the
export itself.

### D3. The command is read-only and records no history state

- `command_ids::LayerSmartObjectExportContents = "layer.smartObject.exportContents"`
  in `commands.h`.
- `command_tree.cpp`: replace the disabled `{Layer, Smart Objects, Export
  Contents…}` leaf with `registry.add(...)` and `implemented = true`; the
  remaining Smart Objects placeholders (`New Smart Object via Copy`, `Edit
  Contents`, `Rasterize`, `Stack Mode`) stay disabled.
- `frame_menus.cpp`: an enabled provider consulting
  `layer_can_export_smart_object_contents` on the current layer, and a handler
  that opens `QFileDialog::getSaveFileName` filtered to `Photoshop files
  (*.psd)` (default `*.psd`), calls `export_smart_object_contents`, and reports
  the result.
- The handler MUST NOT call `clear_link_sets()`, `recomposite()`, or `record()`.
  Export mutates nothing, so there is no state to undo and the history count is
  unchanged. This is the key difference from Convert, Place, Replace, and
  Rasterize.

### D4. Verification

- **Rust engine test** in `crates/pictura-render/src/tests/smart_object.rs`:
  build a document, `place_smart_object` a written solid-colour PSD, and assert
  `smart_object_source_bytes` returns the exact placed bytes. Then assert
  `None` for a non-smart layer, a group, an adjustment layer, a smart object
  with an empty payload, and an unresolved path. Call it once more after an
  export to prove the accessor did not mutate the document (or compare against
  a pre-call clone).
- **C++ self-test** in `selftest_layers_smart_object.cpp`: take the next free
  exit code **282** (277-281 are used). Place or convert a smart object, export
  its contents to a temp path, and assert the file exists and its bytes parse
  with the codec (or match a recorded hash), a non-smart layer refuses, and the
  document's history count is unchanged by the export; remove the temp file.
  Add the `runLayersExportSmartObjectChecks` entry point to the header and call
  it from `selftest_layers_controls.cpp` after `runLayersOpenSmartObjectChecks`.
  Keep `selftest_layers_smart_object.cpp` under its file-size cap.

## Risks / Trade-offs

- **Arbitrary bytes land at a user path.** The dialog supplies the path and
  handles overwrite confirmation; the bridge returns `false` on any `std::fs`
  error rather than reporting success. Not a new trust boundary: the user
  already chooses save paths for Save As.
- **Format fidelity vs. the CS6 PSB rule.** Byte-for-byte export preserves the
  original payload but does not convert layer-created objects to PSB. Deferred;
  documented in the proposal's Non-Goals. A future change can transcode on
  export without touching the accessor.
- **No-op export looks like success.** When the layer has no payload the command
  is disabled, so the bridge's `false` is only a defensive path. The self-test
  asserts a non-smart layer refuses.
- **Exit-code discipline.** Take 282 (after the used 281), keep the check inside
  the existing `selftest_layers_smart_object` suite, and stay within the
  file-size allowance.
