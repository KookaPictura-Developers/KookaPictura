## Context

`psd-smart-object-roundtrip` gave `Layer` a typed `SmartObject` and taught
`write_psd` to author a `SoLd` descriptor plus an embedded `lnk2` record when a
layer carries an embedded payload and no preserved config block
(`crates/pictura-codec/src/smart_writer.rs::should_author`). The renderer
already draws a layer from its raster proxy and only falls back to decoding an
embedded payload when the layer has no color channel
(`crates/pictura-render/src/composite.rs::composite_smart_source`).

The app exposes `Layer > Smart Objects > Convert to Smart Object` as a disabled
placeholder (`command_tree.cpp`). Nothing creates the typed object, so the
round-trip and render paths cannot be reached from the UI.

## Goals / Non-Goals

**Goals:**

- One engine operation, `convert_to_smart_object(doc, path) -> bool`, that turns
  a single raster pixel layer into a layer carrying an embedded smart object.
- The authored object is a valid PSD that `read_psd` resolves as an embedded
  smart object with a non-empty payload, and that survives save→load.
- Rendering is byte-for-byte unchanged: the layer keeps its pixel channels, so
  the composite is identical before and after conversion.
- One app command with a stable id, correct availability, and exactly one undo
  state on success (none on refusal).

**Non-Goals:**

- Rasterize Smart Object; Edit Contents; Replace Contents; Export Contents…;
  Place / Open As Smart Object from a file; linked (external) objects; New Smart
  Object via Copy; the Stack Mode submenu; editing embedded contents in the UI.
- Authoring a *layered* embedded source that would let a later Edit Contents
  reopen the layer structure. This change serializes the source document as
  written by `write_psd`; see D2.

## Decisions

### D1. The engine op lives in `layer_ops`, re-exported at the crate root

`crates/pictura-render/src/document_ops/layer_ops/smart_object.rs` holds
`convert_to_smart_object`, registered in `layer_ops/mod.rs` and re-exported from
`document_ops/mod.rs`, matching `rasterize_fill_content`. It is a pure function
over `&mut Document` and follows the existing refusal contract: resolve the path,
return `false` without mutating when the target is ineligible.

Eligibility is exactly: `resolve_path` succeeds; not a group; `adjustment` is
`None`; not the Background (`layer.background == false`); `smart_object` is
`None`; and `rect.width() > 0 && rect.height() > 0`. Resolving the path also
covers "the layer exists". No size cap beyond zero (the embedded source is a
normal PSD; `write_psd` already rejects unsupported modes/depths).

### D2. Embedded source = same-mode/depth document, layer copy translated to `(0,0)`

The source document is `Document::new(rect.width(), rect.height(), doc.mode,
doc.depth)`. It contains a copy of the target layer with its channels unchanged
but its `rect` translated so its top-left is `(0, 0)`, and a merged composite
equal to the layer's raster over that rect. `write_psd` serializes it to the
payload. This yields a valid, self-contained PSD.

**Deliberate simplification (alpha/proxy).** The conversion KEEPS the target
layer's existing pixel channels. This is the whole point: a layer with a proxy
renders from the proxy, so adding the smart-object link changes nothing visually
and the change stays a pure metadata authoring step with zero render risk. The
codec already prefers the proxy and only decodes the payload for a channel-less
layer, so authoring the object cannot alter the composite.

**Ceiling.** The source document is written by `write_psd`, whose composite is
the document's RGB layers composited to `mode`/`depth`; the embedded `lnk2`
payload therefore carries the merged image, not a rich layered source. A future
**Edit Contents** that wants to reopen the layer structure will need a layered
source document, so this change records that follow-up rather than pretending
the payload is editable. The payload still resolves and round-trips as an
embedded smart object, which is the contract this change specifies.

### D3. SmartObject fields and filename

The attached view is `SmartObject { kind: SmartObjectKind::Embedded, payload:
Some(bytes), filename: format!("{}.psd", layer.name), filetype: *b"8BPB",
creator: *b"8BIM", ..Default::default() }`. The empty/duplicate-name case is not
special-cased: Photoshop derives the embedded filename from the layer name and
the codec's deterministic uuid already dedupes identical filename+payload pairs,
so two same-named layers sharing a raster intent share one record. The current
pixel channels stay on the layer.

### D4. The app command mirrors the Rasterize bridge shape

- `command_ids::LayerSmartObjectConvertTo = "layer.smartObject.convertTo"` in
  `commands.h`.
- `command_tree.cpp` registers the existing Smart Objects leaf with this id and
  `implemented = true`; the other Smart Objects leaves stay disabled placeholders.
- A bridge method on `PictureView`, implemented next to the rasterize commands in
  a `impl_layers_smart_object.rs`, calling the engine op through the pinned
  `doc`. On success only: `clear_link_sets()`, `recomposite()`,
  `record("Convert to Smart Object")`. On refusal: return false, record nothing.
- `frame_menus.cpp` sets the handler (refresh on success) and an enabled provider
  that consults a read-only `can_convert_to_smart_object(path)` bridge predicate,
  so the menu item is enabled only for an eligible current layer.

Rationale: this is the exact pattern `rasterize_fill_content` /
`background_from_layer` already use, including the "one successful command = one
undo state, refusal = none" rule, so no new convention is introduced.

### D5. Verification

- Rust: an engine test in `crates/pictura-render/src/tests/smart_object.rs`
  covering the op directly — convert a raster layer, assert the view is
  `Embedded` with a non-empty payload, assert the proxy channels are untouched,
  assert `read_psd(payload)` resolves the object, assert the refusal cases (group,
  adjustment, background, already-smart, zero-size, bad path) leave the document
  equal.
- C++: a new `selftest_*` suite (next free exit code after 276) that creates a
  document with a raster layer, converts it, asserts the layer reports a smart
  object, asserts the composite is unchanged, asserts save→load preserves the
  object as embedded with a non-empty payload, and asserts refusal for a group
  and for the Background. Any new `.cpp`/`.h` is added to `CMakeLists.txt`
  explicitly.

## Risks / Trade-offs

- **The embedded source is a merged composite, not a layered document.** A later
  Edit Contents cannot reconstruct the layer tree from it. → Deferred and named
  here; this change only promises resolve + round-trip + unchanged rendering.
- **Filename collisions / empty names.** Two layers named alike produce the same
  deterministic uuid for identical rasters and share one linked record, which is
  correct Photoshop behaviour. No de-dup work is done. → Covered by the codec's
  `should_author`/`author_uuid`; no new code.
- **Menu enablement must not mutate.** The enabled provider uses a read-only
  predicate, mirroring `can_merge_*`, so hovering the menu cannot change the
  document or history.
- **Exit-code discipline.** The self-test uses the next free code and an
  append-only suite; a mis-assigned code breaks the failure identity. → Take 277
  (after `runToolsSelectionChecks`' 276) and add the new file to
  `CMakeLists.txt`.
