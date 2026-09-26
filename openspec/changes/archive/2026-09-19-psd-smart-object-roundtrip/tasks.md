## 1. Model

- [x] 1.1 Add `SmartObject` to `pictura-core`: `uuid: String`, `filename: String`, `filetype: [u8;4]`, `creator: [u8;4]`, `kind: SmartObjectKind` (`Embedded`/`External`/`Alias`/`Unresolved`), `config_descriptor: Vec<u8>`, `payload: Option<Vec<u8>>`.
- [x] 1.2 Add `Layer.smart_object: Option<SmartObject>` and `..Default::default()` where needed.
- [x] 1.3 Add a smart-filter view: `SmartFilter { filter_id: i32, name: String, enabled: bool, options: Vec<u8> }`, reached from the layer's `filterFX`; keep `FEid`/`FXid`/`FMsk` bytes on the document or layer as preserved.
- [x] 1.4 Keep the opaque storage as the byte source of truth; the typed views derive on read and are ignored for re-emission when preserved bytes exist.

## 2. Read (resolve)

- [x] 2.1 Parse a layer's `SoLd`/`SoLE`/`plLd` block to extract `Idnt` (tolerant of unknown keys; keep the raw bytes).
- [x] 2.2 Parse the document-level `lnkD`/`lnk2`/`lnk3`/`lnkE` list into records: kind, version, uuid, filename, filetype, creator, datasize, payload.
- [x] 2.3 Match by `uuid`; populate `Layer.smart_object`; mark external/alias as non-embedded, unresolved when no record matches.
- [x] 2.4 Parse `filterFX` into `SmartFilter` entries; expose the Camera Raw Filter (`filterID` 2683) `Fltr` descriptor.
- [x] 2.5 Parse the `Fltr` short keys with the mapping in `docs/dev/camera-raw-cc-notes.md`.

## 3. Write (re-emit and author)

- [x] 3.1 Re-emit a resolved smart object's preserved descriptor and record bytes verbatim, in the same document-level list.
- [x] 3.2 Author: build a `SoLd` descriptor and an embedded `lnk2` record with a deterministic, RFC-4122 v4-shaped uuid derived from the filename and payload, so identical embedded sources resolve to one shared record; record version 7, filetype `8BPB`, creator `8BIM`, and the payload as the record data.
- [x] 3.3 Write `filterFX` entries; for the Camera Raw Filter, write `Fltr` short keys and preserve unmodeled keys.
- [x] 3.4 Emit a typed error on a malformed descriptor or record; never panic.

## 4. Tests

- [x] 4.1 Round-trip `test_with_smart_object01.psd`; assert descriptor, record, payload, and uuid are unchanged.
- [x] 4.2 Author a document with an embedded payload, write, re-read; assert payload hash and filename.
- [x] 4.3 `psd-tools` oracle: parse our authored file and assert it reports a smart object with the same embedded data.
- [x] 4.4 Round-trip `test_with_smart_object02.psd`; assert `filterFX`, `Fltr`, `FEid`, and `FMsk` are unchanged.
- [x] 4.5 A Camera Raw Filter option edit round-trips without dropping unmodeled keys.
- [x] 4.6 External record preserved, never read from disk.
- [x] 4.7 Malformed/truncated descriptor and record error without panic.

## 5. Adobe interop fixtures

> Fixture limitation: only the synthetic smart-object fixtures 01/02 exist. CS6 and
> earliest-CC behavior cannot be fixture-validated because there is no such
> reference file; that validation is deferred. Tolerant parse plus byte-preserving
> write is the guarantee, and unproven behavior is stated as such.

- [ ] 5.1 Move `assets/test_with_smart_object0{1,2}.psd` under `crates/pictura-codec/tests/fixtures/`; record provenance (self-produced, no Adobe assets).
- [x] 5.2 Fixture round-trip oracle with `psd-tools` over both files.
- [ ] 5.3 Manual Photoshop CC reopen: deferred follow-up, no Photoshop available in this environment; acceptance is the automated `psd-tools` oracle (5.2).
- [ ] 5.4 Produce a 16-bit variant later; add it to the oracle once roadmap G4 depth support lands.

## 6. Gates

- [ ] 6.1 `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`.
- [ ] 6.2 `bash scripts/verify-full.sh` and a headless self-test; record counts.
- [ ] 6.3 `openspec validate --all --strict`.
- [ ] 6.4 Update `docs/dev/STATE.md` and `docs/dev/psd-support-roadmap.md`; commit with `TASK-ALLOWS-DOCS`.
