## Why

XMP (resource 1060) is carried byte-for-byte but surfaced only as a raw string,
so the Description fields Photoshop users expect (`dc:title`, `dc:creator`,
`dc:description`, `dc:rights`, `photoshop:Credit`/`Source`) are unreadable in
File Info and uneditable. `WF-010` requires XMP field extraction and XMP
write-back, with IPTC Core synchronised between the XMP packet and the legacy
IIM record (resource 1028) and unknown namespaces preserved verbatim.

## What Changes

- `pictura-codec` parses the XMP packet into a fixed set of typed properties
  (title, creator, description, subject, rights, credit, source, headline,
  marked) handling both the attribute form and the `rdf:Alt`/`rdf:Seq`/`rdf:Bag`
  element forms, without resolving XML entities.
- A hand-rolled **span-patching** writer edits those properties in place: it
  replaces the bytes of a managed property and leaves every other byte of the
  packet — unknown namespaces, unknown properties, comments, the packet wrapper
  — verbatim. A packet it cannot recognise is left untouched (no-op), so a
  malformed or exotic packet is never corrupted. A document with no XMP resource
  gets a minimal well-formed packet when a managed property is first set.
- `set_file_info_fields` updates the shared six IPTC-Core fields in **both** the
  XMP packet and resource 1028 in one call, so the two never hold conflicting
  values; the value written is identical to both channels.
- File Info gains a **Description** category showing the parsed XMP properties
  (alongside the still-available raw packet under Raw Data); editing the six
  core fields now syncs XMP and IIM and remains one undo state.
- No new dependency: the packet is patched by hand (std only). Ceilings: only
  the fixed property set is extracted/edited (IPTC Extension and arbitrary RDF
  are preserved but not modelled), the raw XMP packet is still not editable
  directly, and a packet that uses structures outside the recognised shapes is
  read as far as possible but is not patched. Marked `ponytail:` in code.

## Capabilities

### New Capabilities
- `psd-xmp-metadata`: parse the XMP packet into typed properties, patch a fixed
  property set in place while preserving unknown content byte-for-byte, and
  synchronise the shared IPTC-Core fields between XMP (1060) and IIM (1028).

### Modified Capabilities
- `psd-file-info`: the XMP requirement no longer stops at raw text — the packet
  is now also exposed as parsed typed properties — and the File Info dialog adds
  a Description category while its core-field edits sync XMP and IIM.

## Impact

- `crates/pictura-codec`: new `xmp.rs` (parse, patch, properties) plus
  `set_xmp_fields`/`set_file_info_fields` in `metadata.rs`; exports in `lib.rs`.
  No new dependencies (rule 4 satisfied).
- `crates/pictura-app`: `file_info_dialog` gains a Description category and the
  bridge exposes parsed XMP rows and syncs on apply; a C++ self-test check.
- Oracles: extend the metadata oracle (self-skipping) to compare the parsed XMP
  properties and the save round-trip against `exiftool`. Note: CI does not
  install `exiftool`, so the engine unit tests must prove the parse/patch
  correctness independently of the oracle.
