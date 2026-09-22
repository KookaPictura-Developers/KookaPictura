## Why

The File Info dialog decodes EXIF/IPTC/XMP but is read-only: there is no way to
change a caption, byline, or copyright and keep it. Photoshop's File Info edits
metadata and saves it into the IPTC-IIM (1028) resource. The codec already
re-emits the resource section verbatim on save, and the document snapshot used
for undo already captures that section, so the write path is missing only a
framer, an IIM encoder, and an editable surface.

## What Changes

- `pictura-codec`: add `frame_image_resource(id, name, data)`, `Iptc::set` /
  `Iptc::remove`, `encode_iptc`, and `set_iptc_fields(&mut Document, fields)`
  that decodes resource 1028, applies the edits, re-frames the block, and
  reassigns `Document.image_resources`.
- `File > File Info…`: the IPTC page becomes an editable form for the six core
  fields (Object Name, By-line, Copyright Notice, Caption/Abstract, Credit,
  Source); OK writes them into the document as one undo state and marks it dirty.
- A save persists the edited resource (the writer already emits the section
  verbatim); a round-trip oracle proves the edit survives via `exiftool`.
- Ceilings: XMP is not edited (raw-preserved), non-core IPTC records are
  preserved but not editable, the legacy 1034/1035 mapping is not written, and
  IIM values are written as UTF-8 (non-ASCII Portable-ASCII is not enforced).

## Capabilities

### New Capabilities

- `psd-iptc-write`: encode IPTC-IIM records, frame an image-resource block, and
  apply IPTC field edits to a document so a save persists them.

### Modified Capabilities

- `psd-image-resources`: adds framing a block from typed fields (the inverse of
  the existing parser/encoder).
- `psd-file-info`: the File Info dialog becomes editable for IPTC core fields
  and applies the edits as one undo state.

## Impact

- `crates/pictura-codec`: `image_resources.rs` (framer), `iptc.rs` (set/remove/
  encode), `metadata.rs` (`set_iptc_fields`), `lib.rs` re-exports; no new
  dependency.
- `crates/pictura-app`: editable `file_info_dialog.{h,cpp}`, `showFileInfo`,
  bridge `iptc_edit_fields` / `apply_iptc_edits` in `impl_core.rs` + the bridge
  declaration; new self-test 457.
- No write-format change: the saved PSD's byte layout for every other section is
  unchanged.
