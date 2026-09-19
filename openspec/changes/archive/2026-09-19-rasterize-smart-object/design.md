## Context

`convert-to-smart-object` gave a layer a typed `SmartObject` while keeping its
raster proxy. The renderer draws a proxy-backed layer from the proxy and only
decodes the embedded payload for a channel-less layer
(`crates/pictura-render/src/composite.rs::composite_smart_source`). The codec
resolves a layer's config descriptor (`SoLd`/`SoLE`/`plLd`/`PlLd`) to a
document-level `lnk*` record by uuid (`crates/pictura-codec/src/smart_object.rs`)
and re-emits both preserved byte ranges on write:
`Layer.extra_blocks` inside the layer info, and `Document.layer_section_extra`
as the trailing tagged blocks (`write.rs`).

`Layer > Rasterize > Smart Object` is a disabled leaf
(`crates/pictura-app/cpp/command_tree.cpp`). Clearing `Layer.smart_object`
alone is not enough: a layer read from a file still carries its preserved config
block (which `smart_writer::should_author` uses as a signal to *not* author) and
the document still carries the linked record in `layer_section_extra`, so the
save would contain a smart object with no typed layer pointing at it.

## Goals / Non-Goals

**Goals:**

- One engine operation, `rasterize_smart_object(doc, path) -> bool`, that turns
  one smart-object layer back into a plain pixel layer.
- The layer's pixel channels hold the rendered content after rasterizing: an
  existing proxy is kept as-is, a source-only layer is materialized from the
  decoded payload.
- The preserved config block and the matching document-level linked record are
  dropped, so `write_psd` emits neither a smart object nor an orphan `lnk*`
  record.
- One app command with a stable id, correct availability, and exactly one undo
  state on success (none on refusal).

**Non-Goals:**

- Edit Contents; Replace Contents; Export Contents…; Place / Open As Smart
  Object from a file; linked (external) objects; New Smart Object via Copy.
- Reconstructing editable contents: rasterizing is destructive by definition,
  the object is consumed, not copied.

## Decisions

### D1. The engine op lives beside `convert_to_smart_object`

`crates/pictura-render/src/document_ops/layer_ops/smart_object.rs` gains
`pub fn rasterize_smart_object(doc: &mut Document, path: &str) -> bool`,
registered in `layer_ops/mod.rs` and re-exported from `document_ops/mod.rs` and
`lib.rs`, matching `convert_to_smart_object`. It is a pure function over
`&mut Document` and follows the existing refusal contract: resolve the path,
return `false` without mutating when the target is ineligible or when the
content cannot be produced.

Eligibility is exactly: `resolve_path` succeeds; not a group; `adjustment` is
`None`; and `smart_object.is_some()`. The proxy/content rule from D2 decides
whether decode is attempted; a required-but-failed decode is also a refusal.

### D2. The existing proxy wins over decoding (matching the render path)

The render path (`composite_smart_source`) draws from the proxy whenever the
layer has a color channel, and only decodes the embedded payload for a
channel-less layer. Rasterizing mirrors that order exactly:

- **Has a color channel (`id == 0`)**: the existing channels *are* the rendered
  content. Leave every pixel channel byte-for-byte unchanged; do not decode.
- **No color channel**: decode the embedded payload through the same
  embedded-source path (prefer the preserved merged composite when usable, else
  `composite_rgba`), scale the decoded source into the layer's `rect` with the
  renderer's integer ratio, and write channels `0..mode.color_channels()` plus a
  `-1` alpha channel. An empty/undecodable payload refuses.

Rationale: this guarantees that rasterizing a proxy-backed object cannot change
a single pixel — the composite is identical before and after, exactly as
conversion was. A `@`-less layer is the only case where rasterizing is
observable in pixels, and it is the only case that may refuse for a bad payload.

### D3. Persistence comes from dropping the preserved bytes

The typed `smart_object` is cleared on success, but persistence is carried by
removing the *preserved* bytes that the writer re-emits:

- `layer.extra_blocks.retain(|b| !matches!(&b.key, b"SoLd" | b"SoLE" | b"plLd"
  | b"PlLd"))` — otherwise the layer info re-emits a config descriptor for an
  object that no longer exists, and `should_author` would keep suppressing an
  authored record.
- `doc.layer_section_extra = pictura_codec::remove_linked_source(
  &doc.layer_section_extra, &uuid).unwrap_or_else(|| ...)` — clones and
  replaces only when a record matched, so an unrelated section (or the
  converted-in-memory case where the uuid is empty and nothing is preserved) is
  left untouched.

This is the whole point: convert authors the object on write; rasterize must
make the write stop authoring it.

### D4. `remove_linked_source` rebuilds one block, copies the rest byte-for-byte

New public codec function
`pictura_codec::remove_linked_source(layer_section_extra: &[u8], uuid: &str) ->
Option<Vec<u8>>`. It walks the top-level `8BIM` tagged blocks with the same
length/even-padding rules as `collect_linked_records`. For a
`lnkD`/`lnk2`/`lnk3`/`lnkE` block it parses the `u64`-length-prefixed record
list, keeps every record whose Pascal uuid differs, and re-emits the block with
`write_tag` (same key, recomputed length, even padding). Every non-linked block
and every surrounding byte is copied unchanged. Returns `Some` iff at least one
record was removed; `None` when nothing matched. A malformed block is copied
verbatim and never panics. Placed in `smart_object.rs` (it already owns the
linked-record parser) unless that file's ceiling forces a split to a new
`linked.rs`; either way `lib.rs` re-exports the name.

### D5. The app command mirrors the Rasterize wiring

- `command_ids::LayerRasterizeSmartObject = "layer.rasterize.smartObject"` in
  `commands.h` (next to `LayerRasterizeFillContent` / `LayerRasterizeLayer`).
- `command_tree.cpp`: replace the disabled `{Layer, Rasterize, Smart Object}`
  leaf with `registry.add(...)` and `implemented = true`; the other Rasterize
  placeholders stay disabled.
- `commands.h`/bridge: `layer_can_rasterize_smart_object(&self, path) -> bool`
  (read-only) and `rasterize_smart_object(self: Pin<&mut Self>, path) -> bool`
  implemented in `impl_layers_smart_object.rs`. On success only:
  `clear_link_sets()`, `recomposite()`, `record("Rasterize Smart Object")`; a
  refusal returns false and records nothing.
- `frame_menus.cpp`: handler (refresh on success) and enabled provider using the
  read-only predicate, exactly like `currentSmartPath` for Convert.

### D6. Verification

- **Rust engine test** in `crates/pictura-render/src/tests/smart_object.rs`:
  proxy layer rasterizes with channels unchanged, `smart_object` cleared, and
  `SoLd` gone from `extra_blocks`; source-only layer materializes from the
  payload into channels; refusals (non-smart, group, adjustment, undecodable
  payload) leave the document equal; save→load after rasterizing resolves no
  smart object and carries no orphan `lnk*` record.
- **Codec unit test** in `smart_object.rs` (or `linked.rs`): a section with a
  `lnk2` holding two records plus an unrelated block; remove one uuid and assert
  the survivor and the unrelated block are byte-preserved, the removed uuid is
  gone, and a non-matching uuid returns `None`.
- **C++ self-test** extending `selftest_layers_controls.cpp` (no new file, so
  `CMakeLists.txt` is unchanged) with the next free exit code after 277, i.e.
  278: convert→rasterize, assert the layer no longer reports a smart object, the
  composite is unchanged, save→load leaves no smart object, and a non-smart
  layer refuses with no history state.

## Risks / Trade-offs

- **Rasterize is destructive.** The object's contents are consumed, not copied.
  → This is Photoshop's contract for Rasterize; Edit Contents/Replace Contents
  stay deferred so the only loss is the pointer, and one undo state restores it.
- **A source-only layer's materialized pixels may differ slightly from the
  on-screen decode.** The op must reuse the renderer's decode+scale path, not a
  second implementation. → Implement materialization on top of the existing
  embedded-source decode (share or mirror `composite_smart_source`'s ratio) and
  pin it with the Rust test.
- **Block rebuild must preserve bytes exactly.** A rebuilt `lnk2` whose padding
  or length differs corrupts the section. → Reuse `write_tag`'s padding rule and
  the reader's `u64`+4-byte-pad list format; the unit test asserts byte
  preservation of every survivor/other block.
- **Exit-code discipline.** The self-test uses an append-only code and a
  mis-assigned code breaks the failure identity. → Take 278 (after 277), keep
  the check inside the existing suite and within its file-size ceiling.
