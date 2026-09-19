## 1. Renderer: decode Photo Filter

- [x] 1.1 Add `decode_photo_filter(d: &[u8]) -> Option<Adjustment>` in `crates/pictura-render/src/composite.rs`: require `u16` version = 2, read the `u16` colour space (ignored), the four `u16` colour components, the `u32` density at offset 12, and the `u8` luminosity at offset 16; reject truncated input, versions other than 2, any of the first three components above 255, and density above 100. Return `Adjustment::PhotoFilter(PhotoFilterParams { color, density, preserve_luminosity })`.
- [x] 1.2 Wire `b"phfl" => decode_photo_filter(&data.data)` into the `decode_adjustment` match and update the doc-comment key list to name `phfl` as committed and version-3 `phfl` as deferred.
- [x] 1.3 Confirm the existing `nvrt`/`invr`, `post`, `thrs`, `brit`, `levl`, `hue2`/`hue `, `expA`, `vibA`, `blwh`, and 4-byte `SoCo` arms are unchanged.

## 2. Renderer: encode Photo Filter

- [x] 2.1 Add `pub fn encode_photo_filter(color: [u8; 3], density: f64, preserve_luminosity: bool) -> AdjustmentData` that clamps density to `0..=100` and each component to `0..=255`, emits version 2 with a zero colour space, the fourth component zero, the big-endian `u32` density, the `u8` luminosity flag, and 3 pad bytes (20 bytes total).
- [x] 2.2 Re-export `encode_photo_filter` from `crates/pictura-render/src/lib.rs` alongside the other encoders.
- [x] 2.3 Add a unit test that `decode_adjustment(&encode_photo_filter([255, 180, 80], 25.0, true))` equals `Adjustment::PhotoFilter` with those values; and that an out-of-range density is clamped and still decodes.

## 3. Renderer: decode tests

- [x] 3.1 Add a decoder unit test: a hand-built 20-byte version-2 payload decodes to the expected `PhotoFilterParams`; a truncated payload, a version-3 payload, a component above 255, and a density above 100 each return `None`.
- [x] 3.2 In `crates/pictura-render/src/tests/adjustment.rs::deferred_keys_still_none`, drop `phfl` from the loop and add a version-3 `phfl` payload that still returns `None`.
- [x] 3.3 Add a composite test that a Photo Filter adjustment layer over a non-uniform backdrop differs from the backdrop-only composite, warms it (red above blue), and keeps per-pixel luminance within tolerance.
- [x] 3.4 Assert `decode_adjustment_subset_and_unknown` and the encoder round-trip cases still pass.

## 4. App wiring

- [x] 4.1 Add a `"photo-filter"` arm to `adjustment_layer` in `crates/pictura-app/src/cxxqt_object/helpers.rs` returning `("Photo Filter", encode_photo_filter([255, 180, 80], 25.0, true))`, and import the encoder.
- [x] 4.2 Add the `Photo Filter` / `adjustment:photo-filter` `imp(...)` row to the `adjustmentsPanel` table in `crates/pictura-app/cpp/panels/panel_group_menu.cpp` after Hue-Saturation.
- [x] 4.3 Add one C++ self-test check (next free code **283**) in `crates/pictura-app/cpp/selftest*.cpp`: add a `photo-filter` adjustment layer, assert it is added and reported as an adjustment, and that the composite warms (red above blue at a sampled pixel) while staying within tolerance. Keep the file inside its `scripts/file-size-allowlist.txt` ceiling.

## 5. Fixture regeneration

- [x] 5.1 Add an `adjustment()` builder to `scripts/generate-fixtures.py` that reproduces the documented existing 5 adjustment layers (Invert, Posterize, Threshold, BrightnessContrast, Levels) plus a `PhotoFilter(version=2, color_space=0, color_components=(255, 180, 80, 0), density=25, luminosity=1)` layer, and register `"adjustment.psd": adjustment` in `FIXTURES`.
- [x] 5.2 Regenerate `crates/pictura-codec/tests/fixtures/adjustment.psd` with `python3 scripts/generate-fixtures.py`; verify the existing layers' bytes are unchanged apart from appended-layer offsets.
- [x] 5.3 Update `crates/pictura-codec/tests/oracle.rs::adjustment_layers_preserve_key_and_bytes` to expect the `PhotoFilter` name and assert the `phfl` key and its 20 payload bytes; the whole-document round-trip assertion continues to cover it.
- [x] 5.4 Update `crates/pictura-codec/tests/fixtures/README.md`'s contents table and snippet to document the `adjustment()` builder and the new layer.

## 6. Gates

- [x] 6.1 `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`, `cargo test --workspace --doc`.
- [x] 6.2 `bash scripts/verify-full.sh` and a headless self-test; record counts.
- [x] 6.3 `openspec validate photo-filter-adjustment-decode --strict` and `openspec validate --all --strict`.
- [x] 6.4 Commit with the regenerated golden fixture and state the fixture change in the commit message. No `docs/` change is expected; if the Curves deferral is recorded in `docs/dev/psd-support-roadmap.md`, commit it separately with `TASK-ALLOWS-DOCS`.
