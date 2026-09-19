## Context

`smart-writer` authors a smart object on write only for a layer that is
`Embedded` with a payload **and** has no preserved config block
(`smart_writer::should_author`). A layer read from a PSD, or one already
authored and re-read, carries a preserved `SoLd`/`SoLE`/`plLd`/`PlLd` block in
`Layer.extra_blocks` and a matching document-level record in
`Document.layer_section_extra`; both are re-emitted verbatim. `read_psd` resolves
the typed `SmartObject` from those preserved bytes
(`crates/pictura-codec/src/smart_object.rs`).

`Layer > Smart Objects > Replace Contents…` is a disabled leaf
(`crates/pictura-app/cpp/command_tree.cpp:380`). The compositor draws a
proxy-backed layer from its color channels and only decodes the embedded payload
for a channel-less layer (`composite_smart_source`). Adobe's action keeps the
layer's transform and swaps only the source.

## Goals / Non-Goals

**Goals:**

- One engine operation that swaps a replaceable layer's embedded source and
  makes it render at the layer's existing rect.
- The layer's transform/geometry and every other property are untouched; only
  the smart-object source and pixel channels change.
- A save after the operation carries the NEW payload, not the old one.
- One app command with a stable id, correct availability, and exactly one undo
  state on success (none on refusal).

**Non-Goals:**

- File > Open As Smart Object…; Edit Contents; Export Contents…; the interactive
  transform session; non-PSD/PSB formats; linked (external) objects.

## Decisions

### D1. The engine op lives beside the other smart-object ops

`crates/pictura-render/src/document_ops/layer_ops/smart_object.rs` gains
`pub fn replace_smart_object_contents(doc: &mut Document, path: &str,
filename: &str, bytes: &[u8]) -> bool`, registered in `layer_ops/mod.rs` and
re-exported from `document_ops/mod.rs` and `lib.rs`, matching
`convert_to_smart_object`. It is a pure function over `&mut Document` and
follows the existing refusal contract: resolve the path, return `false` without
mutating when the target is ineligible or the bytes do not parse.

Eligibility is exactly: `resolve_path` succeeds; not a group; `adjustment` is
`None`; and `smart_object` is `Some` with `kind == Embedded` and
`payload.is_some()`. Decoding is `pictura_codec::read_psd(bytes).ok()`; a
non-PSD/PSB source is a refusal, so no other format is reachable.

### D2. Replace the typed source and clear the proxy

On success set `smart_object.payload = Some(bytes.to_vec())`,
`smart_object.filename = filename`, `filetype = *b"8BPB"`, `creator =
*b"8BIM"`, and clear `smart_object.uuid`. Clear the layer's pixel channels.
Clearing the channels is what makes the swap observable: a proxy color channel
wins over the embedded source in the compositor, so a left-in-place proxy would
keep rendering the old pixels. A channel-less layer renders the new payload
through the existing embedded-source path, scaled into the layer's existing
`rect`, which is exactly the transform-preserving behavior we want.

Preservation is the other half of the contract: `rect`, name, mask, blend,
opacity, fill, color, and lock flags MUST be untouched. The only mutations are
the typed source fields and `channels`.

### D3. Drop the preserved blocks and record so the save re-authors the new source

This is the key correctness point. `should_author` refuses to author when the
layer still carries a preserved config block, and the writer re-emits
`layer_section_extra` verbatim. If either were left behind, `write_psd` would
re-emit the OLD `SoLd` and the OLD `lnk2` record, so a save→load would resolve
the old source and silently undo the replace. Therefore on success:

- remove `SoLd`/`SoLE`/`plLd`/`PlLd` from `layer.extra_blocks`, so
  `should_author` returns the typed object and the writer builds a fresh `SoLd`;
- when the old uuid was non-empty, call
  `pictura_codec::remove_linked_source(&doc.layer_section_extra, &old_uuid)` and
  replace the section only when it returns `Some`, so the old `lnk2` record is
  gone and the writer authors a fresh one for the new payload.

To author a fresh link the writer derives its uuid deterministically from
filename + payload (`smart_writer::author_uuid`), so clearing `so.uuid` keeps
the typed state from matching the removed record and the new link is
self-consistent.

### D4. The app command mirrors the Place wiring

- `command_ids::LayerSmartObjectReplaceContents = "layer.smartObject.replaceContents"`
  in `commands.h`.
- `command_tree.cpp`: replace the disabled `{Layer, Smart Objects, Replace
  Contents…}` leaf with `registry.add(...)`, `implemented = true`; the other
  Smart Objects placeholders stay disabled.
- Bridge `replace_smart_object_contents(self: Pin<&mut Self>, path: &QString,
  file_path: &QString) -> bool` in `impl_layers_smart_object.rs`: read the file,
  derive the display name from its base name, invoke the engine op on the
  current document. On success only `clear_link_sets()`, `recomposite()`,
  `record("Replace Contents")`; a refusal returns false and records nothing.
- `frame_menus.cpp`: a `QFileDialog::getOpenFileName` filtered to Photoshop
  files (`*.psd *.psb`) and an enabled provider using a read-only
  `layer_can_replace_smart_object_contents(path)` predicate, mirroring Convert.

### D5. Verification

- **Rust engine test** in `crates/pictura-render/src/tests/smart_object.rs`:
  read `assets/test_with_smart_object01.psd` (self-skip when absent) so the layer
  carries a preserved `SoLd` plus a document-level `lnk2` with a real uuid, then
  replace with a written solid-colour PSD and assert the payload and filename
  changed, `uuid` is empty, `extra_blocks` has no `SoLd`-family block, the old
  uuid is gone from `layer_section_extra`, `rect`/name/blend/opacity/mask are
  unchanged, and `write_psd`→`read_psd` resolves the NEW payload. Refusals
  (non-smart layer, group, adjustment, malformed bytes, bad path) compare the
  document to a pre-call clone.
- **C++ self-test** extending `selftest_layers_controls.cpp` (no new file, so
  `CMakeLists.txt` is unchanged) with the next free exit code after 279, i.e.
  **280**: place/convert a smart object, replace its contents with a written
  solid-colour PSD, assert the composite changes, exactly one history state, and
  a malformed file refuses with no state.

## Risks / Trade-offs

- **Replace is destructive to the old source.** The old payload is dropped and
  one undo state restores it. This matches Photoshop's Replace Contents.
- **A rebuilt `lnk2` whose padding or length differs corrupts the section.**
  `remove_linked_source` already reuses `write_tag`'s padding and the reader's
  list format and is pinned by its own unit test; replace reuses it rather than
  re-parsing.
- **A stale proxy would hide the swap.** Clearing the channels is mandatory;
  the Rust test asserts the composite/payload actually changed, so a
  left-in-place proxy fails the test.
- **Exit-code discipline.** Take 280 (after the used 279), keep the check inside
  the existing suite, and stay within the file-size allowance.
