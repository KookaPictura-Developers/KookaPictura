## Context

`read_psd` currently skips the color-mode-data and image-resource sections by
length (`read.rs`), writes both as zero (`write.rs`), drops every tagged
additional-layer-info key it does not model (`read_layer_record`), re-emits
blending ranges as empty, drops the mask block's trailing bytes, discards the
`-3` channel, and loses the original key of an unrecognized blend mode. The
model has nowhere to keep any of it. `Document`/`Layer`/`LayerMask` are plain
`PartialEq` structs built with struct literals throughout the workspace.

## Goals / Non-Goals

**Goals**

- An open→save round-trip preserves every block the engine does not interpret,
  byte-for-byte, for unmodified documents and unrelated edits.
- Zero behaviour change for engine-created documents: their preservation storage
  is empty, so `write_psd` output is identical (golden intact).
- A Document read from a file and re-read after writing compares equal.

**Non-Goals**

- Whole-file byte-identity: section lengths and channel order are recomputed, so
  the file is not bit-identical, only block-faithful.
- Rendering the preserved data (effects, smart objects, text) — roadmap P3.
- Preserving data through edits that delete the layer carrying it (expected).

## Decisions

### D1. Store opaque bytes, not parsed semantics

Preserved data is raw: `Document.color_mode_data`, `Document.image_resources`,
`Document.global_layer_mask`, `Document.layer_section_extra`; `Layer.blend_key`
(`Option<[u8;4]>`), `Layer.blending_ranges`, `Layer.extra_blocks`
(`Vec<LayerBlock>`), `Layer.raw_channels` (`Vec<RawChannel>`); `LayerMask.extra`.
Re-emitting the exact bytes cannot drift from the source, unlike a parse/re-emit
of an under-specified block.

### D2. Model fields, `Default`, and struct-literal churn

`Layer`, `LayerMask`, and `Document` gain the fields and a `Default` impl;
every struct literal in the workspace appends `..Default::default()`. This is
mechanical; the compiler enumerates the sites. The new fields default empty, so
constructed values are unchanged apart from the added fields.

### D3. A recognized blend key is not stored

`read_layer_record` sets `Layer.blend_key = Some(key)` only when
`BlendMode::from_psd_key(key)` is `None`; a recognized key leaves it `None`
because `BlendMode` already round-trips. `write_record` keeps a stored key while
it still agrees with the layer's mode — an unknown key agrees while the mode is
the `Normal` fallback (so it survives), a known key agrees only if it maps to the
mode — and otherwise writes `layer.blend.to_psd_key()`. So an unknown key
survives, a changed mode supersedes a stale key, and a constructed document
equals a read one.

### D4. Unmodeled channels keep their full stream

Channel ids outside `{0,1,2,-1,-2}` (notably `-3`, the real user mask) are
consumed as `declared_len` raw bytes — including the 2-byte compression header —
into `RawChannel { id, data }`. No dimensions are needed because the bytes are
never decoded; `write_layer_info` re-emits them with `declared_len = data.len()`
in the same channel-info order.

### D5. Capture points in the codec

- Header: `color_mode_data` = the color-mode section bytes; `image_resources` =
  the resource-section bytes.
- Layer section: `global_layer_mask` = the global-mask payload; the bytes after
  it to the section end = `layer_section_extra`.
- Layer record: `blending_ranges` = the ranges block bytes; mask `extra` = the
  mask block bytes after the 18-byte fixed part; unknown tagged keys ->
  `extra_blocks` in encounter order.
- `write_psd` re-emits each with a recomputed length; empty vectors reproduce
  today's zero-length sections exactly.

## Risks / Trade-offs

- **Large mechanical diff** across ~160 struct literals. Mitigated by
  `..Default::default()` and compiler-guided fixes; semantics are untouched.
- **Preservation is keyed to the layer object**, so deleting a layer loses its
  unknown blocks (expected).
- **Unknown tagged blocks are re-emitted after the modeled ones**, so key order
  may differ from the source; PSD readers key by tag, not position.
