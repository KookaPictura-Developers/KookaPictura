## 1. Color and model

- [x] 1.1 Add `Profile::description()` and `Profile::is_srgb()` to `crates/pictura-color/src/lib.rs` (lcms2 `InfoType::Description`, "srgb" match) with a unit test.
- [x] 1.2 Add `crates/pictura-color/examples/dump_adobe_rgb.rs` (writes `adobe_rgb().to_icc()` to stdout) and generate/commit `crates/pictura-codec/tests/fixtures/psd_icc_rgb.icc`.
- [x] 1.3 Add `Document.source_icc: Option<Vec<u8>>` to `crates/pictura-core/src/lib.rs`, default `None`.

## 2. Codec

- [x] 2.1 Add `pictura-color` to `crates/pictura-codec/Cargo.toml`.
- [x] 2.2 Add `raw: Vec<u8>` to `ImageResource`; store the exact block bytes in the parser; add `encode_image_resources(&[ImageResource]) -> Vec<u8>`; re-export; unit test `encode(decode(section)) == section`.
- [x] 2.3 Add an ICC pass in `crates/pictura-codec/src/read.rs` (new `icc.rs`): find resource 1039, `Profile::from_icc`, skip if `is_srgb()`, else convert the composite and every complete layer color-channel set (planar↔packed) with `pictura_color::convert(.., RelativeColorimetric, false)`, set `source_icc`, and rewrite `image_resources` without 1039. Run it after `normalize`.
- [x] 2.4 Unit tests: a synthetic non-sRGB profile converts; an sRGB/undecodable/absent profile leaves pixels and resources untouched; a partial-channel layer is skipped.

## 3. Fixture and oracle

- [x] 3.1 Add an `icc_profile()` builder to `scripts/generate-fixtures.py` embedding `psd_icc_rgb.icc` as resource 1039; register/commit `icc_profile.psd`.
- [x] 3.2 Oracle in `crates/pictura-codec/tests/`: `read_psd` sets `source_icc`, converts the composite within 1 LSB of an independent lcms2 conversion, and `write_psd` output drops resource 1039 while keeping other resources.
- [x] 3.3 Update `crates/pictura-codec/tests/fixtures/README.md`.

## 4. App

- [x] 4.1 Show a status-bar "Converted from embedded ICC profile (<description>)" notice when `source_icc` is set (`crates/pictura-app/src/cxxqt_object/impl_core.rs`), mirroring the depth/color-mode notices.

## 5. Gates and commit

- [x] 5.1 `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, targeted `cargo nextest`.
- [x] 5.2 `bash scripts/verify-full.sh` and `openspec validate --all --strict`.
- [x] 5.3 Confirm file-size and guard are green.
- [x] 5.4 Archive the change and commit code + spec + fixture.
- [x] 5.5 Update `docs/dev/psd-support-roadmap.md` and `STATE.md` in a separate docs commit.
