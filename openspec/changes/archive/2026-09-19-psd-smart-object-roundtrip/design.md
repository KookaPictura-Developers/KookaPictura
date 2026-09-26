## Context

`psd-opaque-preservation` captures the bytes of a smart object: the layer-level
config descriptors (`SoLd`/`SoLE`, legacy `plLd`) land in `Layer.extra_blocks`,
and the document-level linked-layer list (`lnkD`/`lnk2`/`lnk3`/`lnkE`) lands in
`Document.layer_section_extra`. Nothing parses or consumes them.

A smart-object layer links to its source by a `uuid`: the `SoLd` descriptor
carries `Idnt`, and the document-level linked-layer list holds records keyed by
the same `uuid`. A record's `kind` is DATA for embedded content, external for a
path, or alias. For embedded content the record's data field is the source file
bytes.

Camera Raw settings have two storage models in PSD. A raw opened as a Smart
Object stores `crs:` XMP in the embedded payload. A Camera Raw Filter applied as
a smart filter stores its options in the layer's `SoLd` descriptor at
`filterFX.filterFXList[].Fltr`, with `filterID` 2683, plus the document
`FEid`/`FMsk` blocks. The fixtures in `docs/dev/camera-raw-cc-notes.md` confirm
the container and the CC filter model.

## Goals / Non-Goals

**Goals**

- Resolve a smart-object layer to its embedded source payload, filename, and
  filetype.
- Round-trip every smart-object and smart-filter block, and the `uuid` link,
  byte-for-byte, re-emitting the input descriptor's own form.
- Read and write the earliest-CC Camera Raw Filter settings (`filterID` 2683 /
  ACR 8 / PV2012) carried by `filterFX`; preserve later-CC keys.
- Author a valid config descriptor and linked source record so Photoshop reopens
  a PSD we write with the object intact. Authoring emits `SoLd` (outer version 4,
  descriptor block version 16), which CS6 can reopen.
- Prove the round-trip against the supplied Photoshop CC fixtures.

**Non-Goals**

- Decoding or rendering the embedded raw, and the raw pipeline
  (`FILT-100`/`WF-012`). Authoring consumes a supplied composite.
- Applying or editing the CS6 raw-as-Smart-Object `crs:` path. It is exposed and
  re-emitted byte-exact but is preserved-only, not modeled or edited.
- Validated parity at the CS6 end or the earliest-CC end. The only fixture is a
  reference smart-object file, so tolerant parse plus byte-preserving write is the
  guarantee; behavior unproven for a version is stated as such.
- 16-bit depth (roadmap G4). A 16-bit fixture is produced later.
- External (linked) source loading. Embedded content is the target; external
  records are preserved, not read from disk.
- Editing smart-object or smart-filter contents in the UI.

## Decisions

### D1. The preserved bytes stay the source of truth

`SmartObject` and the smart-filter view are typed views derived on read.
Re-emission writes the preserved descriptor and record bytes verbatim when they
exist; the typed fields are used only when authoring a new object. This keeps an
unmodified round-trip from drifting inside an under-specified descriptor.

### D2. Resolve by `uuid`, expose the payload

`read_psd` parses the document-level linked-layer list into records and matches
the layer's `SoLd.Idnt`. The matched record's data is the embedded payload. A
record whose kind is not DATA is reported as external and is not read from disk.

### D3. Authoring emits `SoLd`; round-trip re-emits the input verbatim

When preserved bytes exist, the writer re-emits the descriptor and record
verbatim, including the input's own descriptor form: a `SoLE` object stays
`SoLE` byte-faithfully. When authoring a new object, the writer emits a `SoLd`
descriptor with outer version 4 and descriptor block version 16 (so CS6 can
reopen it) and an embedded `lnk2` record with a deterministic, RFC-4122
v4-shaped uuid derived from the filename and payload, so identical embedded
sources resolve to one shared record; record version 7, `filetype` `8BPB`, and
`creator` `8BIM`, matching `test_with_smart_object01.psd`.
The parser accepts versions 1 through 8; a descriptor block version it does not
recognize (or an `SoLE` variant) is preserved without error rather than failing
the file.

### D4. Camera Raw settings: `Fltr` at earliest CC; `crs:` is preserve-only

A `filterFXList` entry names the filter with `filterID` and holds options in
`Fltr`. For Camera Raw, `filterID` is 2683 and the settings model targets the
earliest CC Camera Raw Filter (Photoshop CC v14, ACR 8, process version
PV2012); the keys are read with the mapping in `docs/dev/camera-raw-cc-notes.md`.
Keys introduced by later CC releases (`Dhze`, `Upri`, `GuUr`, `Rtch`, `REye`,
`LCs `) are preserved, not modeled. The CS6 raw-as-Smart-Object `crs:` XMP is
exposed and re-emitted byte-exact but is not modeled or edited.

### D5. An unresolved object degrades, it does not fail

A layer with a smart-object block but no matching record is kept as an opaque
preserved block and flagged unresolved; `write_psd` re-emits it unchanged. A
malformed descriptor or record returns a typed error, never a panic.

### D6. Smart-filter blocks are modeled separately from the object

`filterFX` is parsed into a list of `{ filterID, name, enabled, options }`.
`FEid`/`FXid` and `FMsk` are preserved and associated with their layer. The
Camera Raw Filter is the only filter whose options are decoded in this change;
others keep their raw descriptor bytes.

## Risks / Trade-offs

- **Descriptor internals are under-specified.** Adobe interop is fixture-gated:
  round-tripping our own output is not evidence Photoshop accepts it. The two
  supplied fixtures are the evidence base; a 16-bit fixture is pending.
- **Version coverage is not fixture-validated.** The only reference fixture is a
  reference smart-object file, so CS6 and earliest-CC behavior cannot be checked
  against a file. The guarantee is tolerant parse plus byte-preserving write;
  where a behavior is unproven for a version, the artifacts say so explicitly.
- **Authoring without a decoder cannot produce a raw's merged raster.** The
  requirement takes the composite as an input, so the raw path is a later
  consumer; a raster or embedded-PSB smart object can be authored and tested now.
- **Unknown-key order.** Re-emission may reorder descriptor keys; PSD readers
  key by name, not position, so this is acceptable.
