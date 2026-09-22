## Why

A PSD's EXIF, IPTC-IIM, and XMP image resources are preserved byte-for-byte but
never decoded, so `File > File Info…` (already a disabled menu leaf) has nothing
to show. Photoshop surfaces this metadata as the File Info dialog; today the
bytes are opaque to both the engine and the user.

## What Changes

- Add a dependency-free `EXIF` decoder (`pictura-codec`): parse the TIFF/IFD
  structure of image resource 1058 (`Exif\0\0`-prefixed) and 1059 (bare TIFF),
  walking IFD0 and the Exif sub-IFD, into ordered `(tag, value)` entries.
- Add a dependency-free IPTC-IIM decoder: parse resource 1028's `0x1C`
  record/dataset stream into ordered `(record, dataset, bytes)` records.
- Expose `read_metadata(&Document) -> DocumentMetadata` gathering the decoded
  EXIF, IPTC, and the raw XMP packet (resource 1060) as UTF-8 text.
- Implement `File > File Info…` as a read-only dialog with three categories
  (Camera Data = EXIF, IPTC, Raw Data = XMP), fed by cxx-qt bridge getters.
- Add a byte-stable `metadata.psd` fixture carrying a real EXIF IFD, an IPTC-IIM
  block, and an XMP packet, proven against the independent `exiftool` decoder.

XMP is exposed as raw text only; field extraction, editing, templates, sidecars,
GPS, and the other File Info tabs are out of scope (marked as ceilings).

## Capabilities

### New Capabilities

- `psd-file-info`: decode a PSD's EXIF, IPTC-IIM, and XMP image resources into a
  read-only metadata model and surface them in the File Info dialog.

### Modified Capabilities

- (none)

## Impact

- `crates/pictura-codec`: new `exif`, `iptc`, and `metadata` modules and their
  `pub use` re-exports; no new dependency.
- `crates/pictura-core`: unchanged (`read_metadata` reads the existing
  `Document.image_resources`).
- `crates/pictura-app`: new `file_info_dialog.{h,cpp}` (registered in
  `CMakeLists.txt`), a promoted `command_ids::FileInfo`, a `frame_menus.cpp`
  handler, and `metadata_rows`/`xmp_packet` bridge getters in `impl_core.rs`.
- `scripts/generate-fixtures.py`: new `metadata.psd` builder; new
  `crates/pictura-codec/tests/fixtures/metadata.psd` and a metadata oracle that
  self-skips without `exiftool`.
