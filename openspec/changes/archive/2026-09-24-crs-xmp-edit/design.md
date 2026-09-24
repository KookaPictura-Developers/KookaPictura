# Design: crs-xmp-edit

## Context

`extract_crs` finds `<x:xmpmeta>…</x:xmpmeta>` inside an embedded `liFD` payload when the packet contains `crs:` and stores the packet on `SmartObject.crs_xmp`. The full payload is also cloned to `SmartObject.payload`. On write, **preserved** linked records are re-emitted from `Document.layer_section_extra` (opaque `reframe_document_extra`); only **authored** objects (no config block) call `author_lnk2_bytes` from `SmartObject.payload`. Therefore editing `crs_xmp`/`payload` alone does not change a Photoshop-sourced file.

Parallel: `set_camera_raw_option` edits the `Fltr` descriptor in `SoLd` in place and updates the typed view. Document-level `patch_xmp` already does byte-span replacement for managed XMP fields (entity-free, no XXE).

## Goals / Non-Goals

**Goals:**

- Read a fixed set of `crs:` numeric attributes into a typed view.
- Edit one attribute in place so open→save re-emits the new packet.
- Unmodified documents stay byte-identical.

**Non-Goals:**

- Full ACR UI / every `crs:` key / structured XMP containers.
- Sidecar `.xmp` files, Camera Raw Filter `Fltr` (already shipped), rendering ACR math.
- Rebuilding the whole `lnk*` list from memory (patch the preserved span only).

## Decisions

### D1. Typed view is a nested struct, not loose fields

`SmartObject.crs: Option<CrsSettings>` with `exposure: Option<f64>`, `contrast: Option<f64>`, `highlights`, `shadows`, `whites`, `blacks`, `clarity`, `vibrance`, `saturation`, `temperature`, `tint`. Derived on read from `crs_xmp` when present; `None` when there is no packet. Unknown keys are not lifted.

Parsing: scan the packet for `crs:Name="…"` (and `crs:Name>…</crs:Name>` if that form appears) inside the already-bounded packet; decode optional leading `+`; `f64::from_str`; non-finite / unparsable → field `None`. No XML DOM — same bounded scanner style as `xmp.rs` (caps, no entity expansion beyond the five predefined if any decoding is needed for attribute values).

### D2. Edit API takes Document + uuid

```rust
pub fn set_crs_property(
    doc: &mut Document,
    uuid: &str,
    name: &str,           // local name without the `crs:` prefix, e.g. "Exposure2012"
    value: f64,
) -> Result<(), PsdError>
```

1. Find the layer whose `smart_object.uuid` matches (recursive children).
2. Require `kind == Embedded`, `crs_xmp` present, and the name is one of the fixed set (unknown name → `Unsupported`).
3. Span-patch the packet (replace only the matched attribute value; if the attribute is absent, insert it on the `rdf:Description` start tag that already carries other `crs:` attrs — if no safe insertion point, `Invalid`).
4. Update `so.crs_xmp`, re-splice the packet into `so.payload`, and rewrite the `liFD` payload for that uuid inside `doc.layer_section_extra` (same “find tag, replace inner payload, keep framing” idea as `remove_linked_source` but replace-in-place rather than drop).
5. Refresh `so.crs` from the new packet.

Failure at any step leaves the document unchanged (clone-then-commit or validate-before-mutate).

### D3. Write path unchanged

No new branch in `write_psd`. Unedited files re-emit `layer_section_extra` as today. Edited files already carry the new bytes in that field.

### D4. Name set is closed

The eleven keys above. `set_crs_property` rejects anything else. Ceiling (`ponytail:`): widen the list or switch to a generic attribute patcher when a fixture needs more.

### D5. No fixture with real ACR XMP

Synthetic packet in unit tests (same style as `embedded_payload_with_crs_xmp_is_exposed`). Document that a Photoshop-authored raw-as-smart-object fixture remains deferred (roadmap already notes “no fixture”).

## Risks / Trade-offs

- [Attribute vs element form] → Support both `crs:K="v"` and `crs:K>v</crs:K>`; if neither matches safely, fail without mutation.
- [Payload size / splice bugs] → Prove with round-trip test: edit → write → re-read → same uuid, new value, rest of payload byte-equal outside the packet span.
- [uuid not found / external SO] → `Invalid` / `Unsupported`; no partial write.
