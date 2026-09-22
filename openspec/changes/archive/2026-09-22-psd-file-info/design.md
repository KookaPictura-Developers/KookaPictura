## Context

`pictura-codec` already frames the image-resource section into typed
`ImageResource { id, name, data, raw }` records (`decode_image_resources`), and
the PSD ids for EXIF (1058/1059), IPTC-IIM (1028), and XMP (1060) are exported
constants. The bytes themselves are never interpreted, so `File > File Info…`
(already a disabled command leaf in `command_tree.cpp`) cannot be enabled.

There is no XML/TIFF/IPTC parser in the workspace and no such dependency. The
only TIFF reading is `probe.rs::parse_tiff`, a private, minimal IFD walker that
returns width/height/bit-depth for import probing; it is not reusable as an
EXIF decoder.

The long-form contract is `docs/10-workflow-io/file-info-and-metadata.md`
(`WF-010`); it defers the XMP library choice and mandates that unknown metadata
round-trips byte-for-byte.

## Goals / Non-Goals

**Goals:**

- Decode the common EXIF and IPTC-IIM fields of a PSD into typed, ordered
  records without a new dependency, never panicking on malformed input.
- Surface them, plus the raw XMP packet, in a read-only File Info dialog.
- Prove the decode against the independent `exiftool` decoder.

**Non-Goals:**

- Parsing XMP into fields, or editing/serializing any metadata (raw passthrough
  already covers round-trip fidelity).
- Metadata templates, sidecars, GPS, DICOM, video/audio, or the other File Info
  tabs — ceilings for a later change.
- Touching `Document` or the write path.

## Decisions

**Lazy decode, not `Document` fields.** `read_metadata(&Document) ->
DocumentMetadata` reads the already-preserved `document.image_resources`, the
same shape as `decode_image_resources`. This avoids adding fields to
`pictura-core::Document` (struct, `Document::new`, and both `read.rs` literals)
and keeps metadata out of the save path, which already re-emits the raw section.

**Dependency-free hand-rolled decoders.** TIFF/IFD and IPTC-IIM are small,
well-specified binary formats; the existing bounds-checked `Reader` already
covers the reads. Alternatives considered: a new `kamadak-exif` / `quick-xml`
dependency (rejected: AGENTS rule 4, and XMP needs no parser here); extending
`probe.rs::parse_tiff` (rejected: it serves a different caller and drops values).

**EXIF acceptance.** The TIFF stream in resource 1058/1059 may be prefixed with
the 6-byte `Exif\0\0` Application-1 header or be a bare TIFF; `parse_exif`
accepts both. (The committed fixture and exiftool both use the bare form under
1058.) It reads the II/MM byte order, walks IFD0 then the Exif sub-IFD (`0x8769`),
and decodes ASCII (2), SHORT (3), LONG (4), RATIONAL (5), and UNDEFINED (7)
values outside the 4-byte inline slot when needed; a SHORT/LONG/RATIONAL with
`count > 1` is exposed as raw `Undefined` bytes. It caps the entry count per IFD
and the total value bytes cloned, so a malformed blob cannot loop, panic, or
amplify memory.

**IPTC-IIM acceptance.** A record is `0x1C`, record number, dataset number, a
big-endian `u16` length, then that many bytes. Parsing stops at the first
malformed record and returns what it read; a record cap bounds a hostile file.

**XMP is raw text.** No XML parser is added. XXE/billion-laughs are avoided by
not parsing; the packet is exposed as UTF-8-lossy text for the Raw Data tab.

**Bridge shape.** Three `#[qinvokable]` getters (`exif_rows`, `iptc_rows`,
`xmp_packet`) returning `QStringList` / `QString`, matching the existing notice
getters and the no-custom-struct bridge convention. Rows are `"Label\tValue"`.
The dialog takes the rows as constructor arguments, so it is decoupled from the
bridge and testable without a document.

## Risks / Trade-offs

- **`exiftool` missing in a dev environment** → the oracle self-skips with a
  message (the `psd-tools`/PIL oracle pattern); CI's `oracles` job installs the
  tools.
- **Tag coverage is a subset** → only the common IFD0/Exif fields render; the
  rest are dropped. Documented as a ceiling; the raw resource still round-trips.
- **`psd-tools` may reinterpret a resource on save** (it did for ICC) → verified
  that EXIF/IPTC/XMP are written back as opaque bytes; the fixture regenerates
  deterministically.

## Migration Plan

None: additive modules, a new dialog, and a new fixture. No format, schema, or
save-path change.

## Open Questions

- Whether to promote EXIF/IPTC into typed `Document` fields later (needed for
  editing/templates) — deferred until an edit path exists.
