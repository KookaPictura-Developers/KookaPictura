## Context

`xmp.rs` parses resource 1060 into `XmpProperties` and patches an existing
packet in place, preserving unknown bytes. There is no serializer: the only
"new packet" path is a private `minimal_xmp_packet()` in `metadata.rs` used when
a document has no XMP, and it carries no properties. `set_xmp_fields` patches
the document's own packet; it cannot install an arbitrary one.

`WF-010` specifies templates as plain XMP files with three apply modes
(Append / Replace / keep-original-but-replace-matching) and an acceptance
criterion that the matching mode preserves image-specific camera data.
`pictura-codec` depends only on `thiserror`/`flate2`; templates must not add a
dependency.

## Goals / Non-Goals

**Goals:**
- Serialize the managed property set to a standalone, parseable packet.
- Apply a template with the three modes over the fixed property set, patching in
  place so unknown namespaces, EXIF, and other resources survive.
- Keep the six IPTC-Core fields synced to IIM.
- Export/Apply from File Info, each one undo state.

**Non-Goals:**
- Sidecars, a Metadata Templates folder / MRU, batch apply, arbitrary RDF or
  IPTC Extension, and template carriage for non-PSD formats.
- Adding `.xmp` as a codec read format: the app reads/writes the file and the
  codec parses/serializes the packet.
- Enforced exiftool oracle in CI (it self-skips; engine tests carry correctness).

## Decisions

- **Serializer, not a second writer.** `to_xmp_packet` builds a fresh minimal
  packet and writes each property with the correct RDF form (`rdf:Alt` for
  title/description/rights, `rdf:Seq` for creator, `rdf:Bag` for subject, simple
  element for the `photoshop:` fields and `xmpRights:Marked`). It is the only
  new packet-construction code; `apply_template` still uses `patch_xmp`.
- **Merge: three distinct modes.** Append only fills empty/absent fields; Replace
  overwrites every managed field and clears those the template omits;
  KeepOriginalReplaceMatching overwrites only fields the template defines and
  clears nothing. Clearing a field removes its property span and its IIM record.
- **Apply is a thin selection layer.** It computes the per-field updates for the
  mode, writes the nine XMP fields with `set_xmp_fields`, and writes the shared
  six to IIM with `set_iptc_fields`, so `pictura-codec` gains no new writer. The
  XMP and IIM writers already preserve the unparsed tail.
- **Export is the document's parsed properties**, not its raw packet: a template
  carries the nine managed fields, so applying it cannot drag in the exporting
  file's unique camera data. The raw packet stays visible under Raw Data and is
  never used as a template.
- **One undo state per accept/apply**, reusing the book history; Export does not
  mutate the document and records nothing.

## Risks / Trade-offs

- **List fields merge as full lists.** `dc:creator` and `dc:subject` are written
  with one `rdf:li` per template item, so export → apply preserves every item; a
  single-item update still compares by the property's scalar value.
- **Replace can clear fields.** A user choosing Replace on a document whose
  template omits a field loses that field (by design). KeepOriginal is the
  non-destructive default offered.
- **Standalone `.xmp` files are written by the app**, so a malformed template is
  rejected at parse (empty properties) rather than crashing; an unparseable
  template applies nothing.
- **File Info grows a control**, touching the `psd-file-info` dialog; the change
  keeps it additive and does not modify that capability's requirements.

## Migration Plan

Additive: new functions and a new dialog control; existing XMP behaviour is
untouched. No data migration.

## Open Questions

- `WF-010` leaves the CS6 merge-mode wording unverified; this change fixes the
  semantics above and documents them as the contract.
