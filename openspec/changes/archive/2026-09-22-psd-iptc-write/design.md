## Context

`decode_image_resources` / `encode_image_resources` (archived
`psd-image-resources`) give a typed view over the preserved section, but
`encode_image_resources` only concatenates each record's existing `raw` bytes,
so changing `ImageResource.data` has no effect on save. The only resource
mutation today is the ICC pass stripping 1039 (`icc.rs:56-60`). There is no IIM
encoder and no way to frame a block from typed fields.

The PSD writer emits `document.image_resources` verbatim
(`write.rs:611-612`), and the undo `Snapshot` clones the whole `Document`
(including `image_resources`), so an edit to `doc.image_resources` is persisted
and undoable with no writer or history change.

## Goals / Non-Goals

**Goals:**

- Set, remove, and encode IPTC-IIM records, and frame an image-resource block
  from `(id, name, data)`.
- Apply the six core IPTC fields to a document and persist them on save via a
  normal open→save round-trip.
- Make the File Info IPTC page editable and record one undo state per accept.

**Non-Goals:**

- Editing EXIF or XMP; XMP stays raw-preserved (no XML parser, no XXE surface).
- Editing IPTC records outside the six core fields (they are preserved
  byte-for-byte on save), or writing the legacy 1034/1035 resources.
- IIM↔XMP synchronization (documented as a Photoshop behavior; the ceiling is
  that XMP is not updated).

## Decisions

**Re-frame through the existing encoder.** Add
`frame_image_resource(id, name, data) -> ImageResource` (the exact inverse of the
parser, promoted from the test helper at `image_resources.rs:107-121`), then
`set_iptc_fields` decodes the section, edits resource 1028, and calls the
existing `encode_image_resources`. Other records keep their original `raw`
(including alternate `MeSa`/`8B64` signatures), so the section stays lossless.

**IIM edits are `set`/`remove` + `encode_iptc`.** `Iptc::set` replaces in place
or appends; `remove` drops a record; `encode_iptc` is the exact inverse of
`parse_iptc`. An empty edited value removes the record, so clearing a field
works.

**Six editable fields, everything else preserved.** Object Name (2:5), By-line
(2:80), Copyright Notice (2:116), Caption/Abstract (2:120), Credit (2:110), and
Source (2:115). The dialog shows the editable core fields as line edits plus a
read-only table of the remaining records.

**Metadata-only undo.** The bridge method mutates `doc`, then
`record("File Info")` + `changed()` with no `recomposite` (the `select_all`
pattern), because metadata does not affect the raster composite.

**UTF-8 IIM ceiling.** Values are written as UTF-8 bytes; IIM is byte-oriented
and non-ASCII is not Portable-ASCII-safe. Marked as a ceiling; a real editor
would transcode to the IIM charset.

## Risks / Trade-offs

- **XMP/IIM divergence** when a file has XMP core fields but no IIM → the edit
  creates IIM while XMP keeps the old value. Accepted ceiling; the fixture's XMP
  carries only `dc:format`.
- **Clearing a field removes its record**, which changes the IIM stream more than
  a zero-length value would; matches "empty means absent" and is covered by a
  test.
- **No recomposite** → the undo snapshot's raster is unchanged, as required
  (`file-info-and-metadata.md`: metadata is excluded from the composite).

## Migration Plan

None: additive codec functions and an editable dialog. Existing documents with
no IPTC gain a resource 1028 only when a field is edited.

## Open Questions

- Whether to transcode IIM to a legacy charset and whether to sync XMP — both
  deferred until an XMP field model exists.
