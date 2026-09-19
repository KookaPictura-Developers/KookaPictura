## 1. Adjustment op (`pictura-adjust`)

- [x] 1.1 Add `GradientStop { location: u16, color: [u8; 3] }` and `GradientMapParams { stops: Vec<GradientStop>, reverse: bool }` to `crates/pictura-adjust/src/types.rs`, and add the `Adjustment::GradientMap(GradientMapParams)` variant (before `SolidFill`).
- [x] 1.2 Add `gradient_map(p, buf, n)` to `crates/pictura-adjust/src/tonal.rs`: validate at least two stops, `location <= 4096`, strictly increasing; build a 256-entry `[[u8; 3]; 256]` luminance LUT (Rec.601 `common::luma`, reversed when `reverse`), sample linearly between the bracketing stops with endpoint clamping, and write R/G/B leaving alpha untouched. Mark the linear/midpoint/dither ceilings with `ponytail:` comments.
- [x] 1.3 Dispatch `Adjustment::GradientMap(p) => gradient_map(p, buf, n)` in `crates/pictura-adjust/src/apply.rs`, and export `GradientMapParams`/`GradientStop` from `crates/pictura-adjust/src/lib.rs`.
- [x] 1.4 Add `GradientMap` to the `adjustments` array in `crates/pictura-adjust/src/tests.rs::alpha_is_never_modified` so the 16-destructive-variant alpha contract is exercised.
- [x] 1.5 Add kernel unit tests in `crates/pictura-adjust/src/tests.rs`: black→white identity on a neutral grey ramp, reverse flips, a three-stop gradient honors the interior stop, endpoint clamping, and invalid stops return `AdjustError::InvalidParams`.
- [x] 1.6 Add a `Mapping` row `GradientMap` (no ImageMagick equivalent, tolerance 0) to `crates/pictura-adjust/tests/oracle.rs` and to `NO_EQUIVALENT`, and set `MAPPING.len()` to 16 and the count to 13. Add a property test for the identity/reverse/clamp contract.

## 2. Renderer: decode Gradient Map

- [x] 2.1 Add `decode_gradient_map(d: &[u8]) -> Option<Adjustment>` in `crates/pictura-render/src/composite.rs`: require `u16` version 1 or 3 (skip the `4`-byte method when 3), read the `u8` reverse and `u8` dither flags, skip the unicode name (`u32` char count + UTF-16 big-endian), read the `u16` colour-stop count and each `u32 location`/`u32 midpoint`/`u16 mode`/four `u16` components, reduce the first three components with `v >> 8`, and return `Adjustment::GradientMap(GradientMapParams { stops, reverse })`. Reject truncated input, an unsupported version, fewer than two stops, non-increasing locations, and any location above 4096.
- [x] 2.2 Wire `b"grdm" => decode_gradient_map(&data.data)` into the `decode_adjustment` match and move `grdm` from the deferred to the committed list in the doc comment.
- [x] 2.3 Confirm the existing `nvrt`/`invr`, `post`, `thrs`, `brit`, `levl`, `hue2`/`hue `, `expA`, `phfl`, `vibA`, `blwh`, and 4-byte `SoCo` arms are unchanged.

## 3. Renderer: encode Gradient Map

- [x] 3.1 Add `pub fn encode_gradient_map(stops: &[GradientStop], reverse: bool) -> AdjustmentData` that writes version 1, the reverse byte, dither 0, an empty unicode name, the `u16` stop count, each stop (`u32` location, midpoint 50, mode 0, the RGB colour scaled to 16-bit, alpha 0, two pad bytes), zero transparency stops, the trailing fields with psd-tools' defaults (`expansion` 2, `length` 32, the rest zero), and pads to a 4-byte boundary.
- [x] 3.2 Re-export `encode_gradient_map` from `crates/pictura-render/src/lib.rs` alongside the other encoders.
- [x] 3.3 Add a unit test that `decode_adjustment(&encode_gradient_map(&stops, reverse))` equals `Adjustment::GradientMap` with those stops and `reverse`.

## 4. Renderer: decode tests

- [x] 4.1 Add a decoder unit test: a hand-built version-1 payload decodes to the expected stops (including a non-empty name and a `65535` colour reduced to `255`); a version-3 payload with a method decodes to the same stops; a truncated payload, an unsupported version, fewer than two stops, and non-increasing/out-of-range locations each return `None`.
- [x] 4.2 In `crates/pictura-render/src/tests/adjustment.rs::deferred_keys_still_none`, drop `grdm` from the loop.
- [x] 4.3 Add a composite test that a black→white Gradient Map adjustment layer over a non-uniform backdrop differs from the backdrop-only composite and maps a sampled pixel toward the gradient's grey for its luminance.

## 5. App wiring

- [x] 5.1 Add a `"gradient-map"` arm to `adjustment_layer` in `crates/pictura-app/src/cxxqt_object/helpers.rs` returning `("Gradient Map", encode_gradient_map(&[GradientStop { location: 0, color: [0, 0, 0] }, GradientStop { location: 4096, color: [255, 255, 255] }], false))`, and import the encoder and `GradientStop`.
- [x] 5.2 Add the `Gradient Map` / `adjustment:gradient-map` `imp(...)` row to the `adjustmentsPanel` table in `crates/pictura-app/cpp/panels/panel_group_menu.cpp` after Photo Filter.
- [x] 5.3 Add one C++ self-test check (next free code **285**) in `crates/pictura-app/cpp/selftest_layers_controls.cpp`: add a `gradient-map` adjustment layer, assert it is added and reported as an adjustment, and that the composite changes. Keep the file inside its `scripts/file-size-allowlist.txt` ceiling.

## 6. Fixture

- [x] 6.1 Add a `gradient_map()` builder to `scripts/generate-fixtures.py`: a `Base` pixel layer plus a `GradientMap(version=1, is_reversed=0, is_dithered=0, name="Black to White", color_stops=[ColorStop(0, 50, 0, (0, 0, 0, 0)), ColorStop(4096, 50, 0, (65535, 65535, 65535, 0))], transparency_stops=[TransparencyStop(0, 50, 255), TransparencyStop(4096, 50, 255)], expansion=2, interpolation=4096, length=32, ...)` layer, import `ColorStop`/`GradientMap`/`TransparencyStop`, and register `"gradient_map.psd": gradient_map` in `FIXTURES`. Leave `adjustment.psd` unchanged.
- [x] 6.2 Regenerate with `python3 scripts/generate-fixtures.py` and add `crates/pictura-codec/tests/fixtures/gradient_map.psd`.
- [x] 6.3 Add an oracle test in `crates/pictura-codec/tests/oracle.rs` asserting the fixture has the `Base` + `Gradient Map` layers, the `grdm` key and its payload length, and that the whole `Document` round-trips through `read_psd`/`write_psd`.
- [x] 6.4 Update `crates/pictura-codec/tests/fixtures/README.md`'s contents table and snippet to document the `gradient_map()` builder and the new fixture.

## 7. Gates

- [x] 7.1 `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`, `cargo test --workspace --doc`.
- [x] 7.2 `bash scripts/verify-full.sh` and a headless self-test; record counts.
- [x] 7.3 `openspec validate gradient-map-adjustment-decode --strict` and `openspec validate --all --strict`.
- [x] 7.4 Commit with the new golden fixture and state the fixture addition in the commit message. No `docs/` change is expected; if the roadmap's G8 entry is updated, commit it separately with `TASK-ALLOWS-DOCS`.
