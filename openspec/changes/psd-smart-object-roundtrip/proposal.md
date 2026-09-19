## Why

Camera Raw edits only survive a PSD round-trip inside a smart object. The
in-flight `psd-opaque-preservation` change makes the bytes survive an open→save,
but nothing resolves or owns them, so a Photoshop-authored smart object cannot
be read, its Camera Raw settings cannot be surfaced or edited, and a PSD we
write cannot be relied on to reopen in Photoshop with the object intact.

Scope decision (2026-09-19): target the **a reference build** model, matching the
Photoshop files we can produce. Two Camera Raw storage models exist. A raw opened
as a Smart Object stores `crs:` XMP in the embedded payload (CS6 path, no
reference file). A Camera Raw Filter applied as a smart filter stores its
settings in the smart object's `SoLd` descriptor under
`filterFX.filterFXList[].Fltr` with `filterID` 2683 (CC path, confirmed by
`assets/test_with_smart_object02.psd`). The CC filter path is the acceptance
target.

Roadmap P2.5, gaps G13/G14/G15/G16/G17. Fixture findings are recorded in
`docs/dev/camera-raw-cc-notes.md`.

## What Changes

- The document model gains an embedded smart-object view on a layer: the config
  descriptor (`SoLd`/`SoLE`, legacy `plLd`), the linked source record
  (`lnkD`/`lnk2`/`lnk3`/`lnkE`), the embedded payload, and the `uuid` that links
  layer to source.
- `read_psd` resolves a smart-object layer to its embedded source payload,
  filename, and filetype instead of leaving the blocks anonymous.
- `write_psd` re-emits a resolved smart object byte-faithfully, and authors a
  valid config descriptor plus linked source record with a matching `uuid` when
  given an embedded source.
- The model gains a smart-filter view: the layer's `filterFX` list and its
  `Fltr` options, plus the document `FEid`/`FXid` and filter mask `FMsk` blocks.
  A Camera Raw Filter (`filterID` 2683) exposes its settings.
- The Camera Raw Filter settings round-trip: read, edit, and write the `Fltr`
  descriptor keys without dropping the ones we do not model.
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
