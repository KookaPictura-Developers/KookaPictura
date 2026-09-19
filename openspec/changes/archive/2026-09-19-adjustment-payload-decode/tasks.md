## 1. Codec: expose the descriptor DOM

- [x] 1.1 In `crates/pictura-codec/src/lib.rs`, add `pub fn read_descriptor(bytes: &[u8]) -> Result<DescValue, PsdError>` that builds a `common::Reader` and delegates to the existing `descriptor::read_descriptor` (the `camera_raw_options` path and the crate-private reader are unchanged).
- [x] 1.2 Add a unit test in `crates/pictura-codec` proving `read_descriptor` parses a hand-built version-16 descriptor object and returns `PsdError` on truncated input.

## 2. Renderer: decode the committed keys

- [x] 2.1 Add `decode_exposure` for `expA`: require `u16` version = 1, read `f32` exposure/offset/gamma; reject truncated input, non-finite values, and gamma ≤ 0; return `Adjustment::Exposure(ExposureParams)`.
- [x] 2.2 Add `decode_vibrance` for `vibA`: call `read_descriptor`, require a top-level object, read the `vibrance` and `Strt` integer keys (default 0 when absent), reject wrong types and values outside −100…100; return `Adjustment::Vibrance`.
- [x] 2.3 Add `decode_black_white` for `blwh`: call `read_descriptor`, require a top-level object, read `Rd  `/`Yllw`/`Grn `/`Cyn `/`Bl  `/`Mgnt` (psd-tools defaults 40/60/40/60/20/80), `useTint` (bool), and the nested `tintColor` `Clr ` object's `Rd  `/`Grn `/`Bl  ` (0–255, default black); reject wrong types and percentages outside −200…300; return `Adjustment::BlackWhite`.
- [x] 2.4 Wire the three helpers into the `decode_adjustment` match on `expA`, `vibA`, and `blwh`; every helper returns `Option` so a malformed payload is `None`, never a panic or error. The deferred keys share the fall-through `_ => None` arm (the in-house `SoCo` arm keeps a `ponytail:` comment); their reasons are named in the Deferred keys section below.
- [x] 2.5 Confirm the existing `nvrt`/`invr`, `post`, `thrs`, `brit`, `levl`, `hue2`/`hue `, and 4-byte `SoCo` arms are byte-for-byte unchanged.

## 3. Tests

- [x] 3.1 Unit-test `expA`: a well-formed 14-byte payload decodes to the expected `ExposureParams`; truncated, version ≠ 1, non-finite, and gamma ≤ 0 payloads each return `None`.
- [x] 3.2 Unit-test `vibA`: a hand-built descriptor with `vibrance`/`Strt` decodes to the expected `VibranceParams`; a truncated descriptor and an out-of-range key each return `None`.
- [x] 3.3 Unit-test `blwh`: a hand-built descriptor with all channel keys plus `useTint`/`tintColor` decodes to the expected `BlackWhiteParams`; truncated and out-of-range payloads return `None`.
- [x] 3.4 Assert the existing `decode_adjustment_subset_and_unknown` / `encode_decode_round_trips` cases still pass, and add deferred keys (`curv`, `selc`, `clrL`, `gdrm`, `phfl`, `mixr`, descriptor `SoCo`) each returning `None`.
- [x] 3.5 Add an adjustment-layer composite test in `crates/pictura-render/src/tests/adjustment.rs`: one committed key (e.g. `blwh` that desaturates a coloured backdrop) over a coloured backdrop yields output that differs from the backdrop-only composite, proving the decode is a non-no-op end to end.
- [x] 3.6 Add a composite test that a document with a deferred payload (`clrL` or `curv`) equals the backdrop-only composite.

## 4. Gates

- [x] 4.1 `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`.
- [x] 4.2 `bash scripts/verify-full.sh` and a headless self-test; record counts.
- [x] 4.3 `openspec validate adjustment-payload-decode --strict` and `openspec validate --all --strict`.
- [x] 4.4 Update `docs/dev/STATE.md` and `docs/dev/psd-support-roadmap.md` (G8 moves from "descriptor payloads not decoded" to "exposure, vibrance, and black/white decoded"); commit with `TASK-ALLOWS-DOCS`.

## Deferred keys

These stay preserved on disk but `decode_adjustment` returns `None` for them. The
reasons are repeated here so a future change can pick them up without
re-litigating the schema:

- `curv` (Curves): psd-tools marks the payload "highly experimental and
  unstable"; the version-1 map format is undocumented. No confident schema.
- `selc` (Selective Color): no `pictura-adjust` operation to decode into, and
  the plate layout is not grounded here.
- `clrL` (Color Lookup): descriptor-based; no `pictura-adjust` operation.
- `gdrm` (Gradient Map): complex stop/transparency structure; no
  `pictura-adjust` operation.
- `phfl` (Photo Filter): the fixed struct is grounded (v3 = 3×`u32` XYZ, v2 =
  colour space + 4×`u16` components, then density + luminosity), but the colour
  conversion to an sRGB `[u8; 3]` is not confidently groundable. Deferred until
  a conversion baseline exists. (A `PhotoFilterParams` op already exists.)
- `mixr` (Channel Mixer): the Adobe spec states 20 bytes but describes ten
  ("4 * 2 bytes of color with 2 bytes of constant") and psd-tools reads only
  `5h` plus opaque remainder; the full 12-value `ChannelMixerParams` layout is
  not groundable. (A `ChannelMixerParams` op already exists.)
- real `SoCo` descriptor: Photoshop's `Clr ` solid-color sheet descriptor is not
  decoded; only the in-house 4-byte payload is. Deferred.
- Color Balance (`blnc`): not in the codec's `ADJUSTMENT_KEYS`, so no payload
  reaches the decoder; out of scope.
