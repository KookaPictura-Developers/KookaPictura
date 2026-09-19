## Why

Camera Raw edits only survive a PSD round-trip inside a smart object. The
in-flight `psd-opaque-preservation` change makes the bytes survive an open→save,
but nothing resolves or owns them, so a Photoshop-authored smart object cannot
be read, its Camera Raw settings cannot be surfaced or edited, and a PSD we
write cannot be relied on to reopen in Photoshop with the object intact.

Scope decision (2026-09-19): smart objects are **Photoshop CS6 → current CC**,
covered as tolerant read plus byte-preserving round-trip. The only available
reference fixture is a Photoshop **a reference build** file, so we make **no
validated-parity claim** at the CS6 end or the earliest-CC end; behavior that is
unproven for a version is stated as such. Two Camera Raw storage models exist. A
raw opened as a Smart Object stores `crs:` XMP in the embedded payload (CS6 path,
no reference file: preserve-only, not modeled or edited). A Camera Raw Filter
applied as a smart filter stores its settings in the smart object's `SoLd`
descriptor under `filterFX.filterFXList[].Fltr` with `filterID` 2683; the
settings model targets the **earliest CC** Camera Raw Filter (Photoshop CC v14,
ACR 8, process version PV2012), and later-CC keys are preserved, not modeled.
Round-trip re-emits the input descriptor's own form (`SoLE` stays `SoLE`);
authoring a new object emits `SoLd` (outer version 4, descriptor block version
16) so CS6 can reopen it.

Roadmap P2.5, gaps G13/G14/G15/G16/G17. Fixture findings are recorded in
`docs/dev/camera-raw-cc-notes.md`.

## What Changes

- The document model gains an embedded smart-object view on a layer: the config
  descriptor (`SoLd`/`SoLE`, legacy `plLd`), the linked source record
  (`lnkD`/`lnk2`/`lnk3`/`lnkE`), the embedded payload, and the `uuid` that links
  layer to source.
- `read_psd` resolves a smart-object layer to its embedded source payload,
  filename, and filetype instead of leaving the blocks anonymous.
- `write_psd` re-emits a resolved smart object byte-faithfully, preserving the
  input descriptor's own form (`SoLE` stays `SoLE`), and authors a `SoLd`
  descriptor (outer version 4, descriptor block version 16, CS6-readable) plus a
  linked source record with a matching `uuid` when given an embedded source.
- The model gains a smart-filter view: the layer's `filterFX` list and its
  `Fltr` options, plus the document `FEid`/`FXid` and filter mask `FMsk` blocks.
  A Camera Raw Filter (`filterID` 2683) exposes its earliest-CC (ACR 8 / PV2012)
  settings; later-CC keys are preserved.
- The Camera Raw Filter settings round-trip: read, edit, and write the
  earliest-CC `Fltr` descriptor keys without dropping the ones we do not model.
  `crs:` XMP is preserved-only and is never modeled or edited.
- A fixture-driven round-trip oracle is defined over both supplied Photoshop
  PSDs.
- **BREAKING**: none at the API-consumer level; `Layer` gains an optional field
  defaulting to `None`.

## Capabilities

### New Capabilities

- `psd-smart-objects`: resolve, round-trip, and author embedded smart objects in
  PSD files.
- `psd-smart-filters`: round-trip smart-filter blocks and read/write the Camera
  Raw Filter settings they carry.

### Modified Capabilities

<!-- None. The preservation guarantees landed in psd-opaque-preservation; this
     change consumes them, authors new smart objects, and models smart filters. -->

## Impact

- `crates/pictura-core/src/lib.rs`: `Layer.smart_object: Option<SmartObject>`,
  the `SmartObject` type, and the smart-filter representation; struct literals
  gain `..Default::default()`.
- `crates/pictura-codec/src/{read,write,common}.rs`: resolve the `uuid` link,
  parse the linked records and `filterFX`, and author a descriptor plus record.
- `crates/pictura-codec/src/tests.rs`, `crates/pictura-codec/tests/oracle.rs`:
  round-trip and `psd-tools` oracle tests, plus the reference fixtures.
- `assets/test_with_smart_object0{1,2}.psd` move under
  `crates/pictura-codec/tests/fixtures/` with recorded provenance.
- Depends on `psd-opaque-preservation` for the preserved bytes.
- No new runtime dependency; `psd-tools` remains test-only.
- Out of scope: decoding or rendering the embedded raw, the raw pipeline
  (`FILT-100`/`WF-012`), and 16-bit depth (roadmap G4). Authoring a raw smart
  object needs the decoder for the layer raster, so this change takes a payload
  and composite as inputs. A 16-bit fixture is produced later.
