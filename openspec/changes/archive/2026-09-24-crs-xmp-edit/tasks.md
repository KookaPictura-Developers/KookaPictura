# Tasks: crs-xmp-edit

## 1. Model

- [x] 1.1 Add `pictura_core::CrsSettings` with the eleven `Option<f64>` fields (Exposure2012 … Tint).
- [x] 1.2 Add `SmartObject.crs: Option<CrsSettings>` default `None`.

## 2. Parse

- [x] 2.1 `crates/pictura-codec/src/crs_xmp.rs` (or beside `smart_object.rs`): bounded scan of a packet for the fixed `crs:` names (attribute + element form); non-finite/unparsable → field `None`.
- [x] 2.2 Fill `so.crs` in `build_smart_object` when `crs_xmp` is `Some`.

## 3. Edit + splice

- [x] 3.1 Span-patch helper: replace one `crs:Name` value; optional insert on a safe `rdf:Description`; return `None` when unsafe.
- [x] 3.2 `set_crs_property(doc, uuid, name, value)`: locate layer, validate name ∈ fixed set, patch packet, update `crs_xmp` + `payload`, rewrite that uuid’s `liFD` payload in `doc.layer_section_extra`, refresh `so.crs`. Validate-then-commit (no partial mutation on error).
- [x] 3.3 Export from `pictura-codec` `lib.rs`.

## 4. Spec delta

- [x] 4.1 MODIFIED `psd-smart-objects` “Camera Raw crs settings are preserved”: unedited preserve stays; add typed expose + `set_crs_property` scenarios; remove “no edit API”.

## 5. Tests and gates

- [x] 5.1 Synthetic packet with `crs:Exposure2012="+0.50"` etc. → typed view parses.
- [x] 5.2 `set_crs_property` → write → re-read: new value in view and in raw `layer_section_extra`; bytes outside the packet unchanged.
- [x] 5.3 Unknown name / missing uuid / no packet → `Err`, document unchanged.
- [x] 5.4 Unmodified open→save of a payload with `crs:` stays byte-identical (existing fixture path).
- [x] 5.5 `cargo nextest run -p pictura-core -p pictura-codec`, `cargo fmt --all --check`, `cargo clippy -p pictura-core -p pictura-codec --all-targets -- -D warnings`, `openspec validate crs-xmp-edit --strict`.
