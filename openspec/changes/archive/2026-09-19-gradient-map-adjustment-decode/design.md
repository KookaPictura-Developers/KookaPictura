## Context

`pictura-codec` preserves the `grdm` (Gradient Map) block verbatim, and
`docs/04-image-ops/adjustments/gradient-map.md` (`ADJ-015`) defines the mapping:
map the composite luminance through a gradient. `pictura-adjust` has no
Gradient Map op, and `pictura-render`'s `decode_adjustment` does not recognise
the key, so a Gradient Map adjustment layer is composited as a no-op. Two
earlier decisions left it deferred: the archived
`2026-09-19-adjustment-payload-decode` change and the roadmap's G8.

The payload is the legacy `grdm` struct. psd-tools 1.19
(`psd_tools/psd/adjustments.py`, `GradientMap`/`ColorStop`/`TransparencyStop`)
reads and writes it, so the binary layout has an independent reference. The
real Photoshop files in psd-tools' test suite (`gradient-map.psd` and the
three `gradient-map-v3-*.psd`) confirm the field order, the version-3 method
bytes, and that colour components are 16-bit (`65535` = full).

## Goals / Non-Goals

**Goals:**

- Add `Adjustment::GradientMap` to `pictura-adjust`, so the layer renders
  through the existing adjustment composite path.
- Decode a version-1 or version-3 `grdm` payload into that variant.
- Provide `encode_gradient_map` so the app can create a Gradient Map adjustment
  layer in memory, round-tripping against the decoder.
- Wire the kind `gradient-map` through the app bridge and the Adjustments panel
  menu.
- Extend the psd-tools oracle fixtures with a Gradient Map layer, so the codec
  proves the key and payload survive read and write.

**Non-Goals:**

- Fidelity beyond plain linear interpolation between stops. See D4.
- Non-RGB gradient colour models (CMYK/Lab).
- The other still-deferred keys (`curv`, `mixr`, `clrL`, `selc`, real `SoCo`,
  `phfl` v3).
- A Gradient Map GPU shader. Documents with one keep falling back to the CPU
  path, as today.
- Verified pixel parity against Photoshop. The decoding is structural and the
  render contract belongs to `image-adjustments`/`pictura-adjust`.

## Decisions

### D1. The op and its parameters

`pictura-adjust` gains:

```rust
pub struct GradientStop {
    /// Photoshop gradient position, 0..=4096.
    pub location: u16,
    pub color: [u8; 3],
}
pub struct GradientMapParams {
    pub stops: Vec<GradientStop>,
    pub reverse: bool,
}
```

The kernel lives in `tonal.rs` next to Threshold (a luminance remap) and is
dispatched from `apply.rs`. For each pixel:

1. `l = common::luma(r, g, b) / 255.0` (Rec.601, the shared `luma` helper).
2. `l = 1.0 - l` when `reverse`.
3. `pos = l * 4096.0`; clamp below the first stop's location to the first colour
   and above the last stop's location to the last colour; otherwise find the
   bracketing stops and linearly interpolate each channel by
   `(pos - a.location) / (b.location - a.location)`.
4. Write the rounded sampled colour to R, G, B. Alpha is untouched.

Because the map depends only on luminance, a 256-entry `[[u8; 3]; 256]` LUT is
built once (rounded luminance index) and applied per pixel; this keeps the
kernel allocation-free per pixel and mirrors the `map_lut` style.

Validation (before any pixel is written): at least two stops, every
`location <= 4096`, and strictly increasing locations. `apply` returns
`AdjustError::InvalidParams` otherwise.

### D2. The `grdm` byte layout

psd-tools `GradientMap.read`/`ColorStop.read` is the reference:

```
H   version             (1 or 3)
B   is_reversed
B   is_dithered
-- version 3: 4s method (Gcls / Lnr  / Perc / Smth), ignored
I   name_length         (unicode chars)
2*N name (UTF-16 big-endian), no padding
H   color_stop_count
    per stop:
        I   location     (0..=4096)
        I   midpoint     (ignored)
        H   mode         (ignored)
        4H  color        (first three = R,G,B on the 16-bit scale; fourth ignored)
        2x  padding
H   transparency_stop_count
    per stop: 2I H (location, midpoint, opacity) — ignored
4H  expansion(=2), interpolation, length(=32), mode
I   random_seed
2H  show_transparency, use_vector_color
I   roughness
H   color_model
4H  minimum_color
4H  maximum_color
2x  padding, then pad to a 4-byte boundary
```

`decode_gradient_map` reads only far enough to reach the colour stops: version,
the two flags, the method when version 3, the unicode name, the count, and each
stop. Everything after the colour stops is ignored, so a future field change
does not require a parser change.

### D3. Colour stops are 16-bit and reduced to 8-bit

A real Photoshop `gradient-map.psd` from the psd-tools test suite stores
`(65535, 0, 0, 0)` and `(65535, 65535, 65535, 0)`. The decoder therefore treats
each of the first three components as a 16-bit channel and reduces it to 8-bit
with `v >> 8` (equivalently `v * 255 / 65535` rounded toward zero); full scale
(`65535`) maps exactly to `255`. The encoder scales 8-bit back to 16-bit with
`c * 257`, the exact inverse, so `decode(encode(x)) == x` for every 8-bit value.
The fourth component and the midpoint/mode fields are ignored.

### D4. Deliberate ceilings (approximations)

`gradient-map.md` marks these internals closed or inferred. They are stated
here and each gets a `ponytail:` comment in the decoder/kernel:

- **Midpoint bias ignored** — plain linear interpolation between adjacent stops;
  Photoshop's midpoint slider and smoothness are not modelled.
- **Dither ignored** — the dither flag is read but not applied; output is
  deterministic.
- **Transparency/opacity stops ignored** — the mapped output is opaque and the
  compositor treats it as such.
- **Interpolation modes ignored** — the version-3 method and the trailing
  interpolation field are read past.
- **Non-RGB colour models ignored** — `color_model`, `minimum_color`, and
  `maximum_color` are parsed past; stops are always treated as RGB.

### D5. Version 3 is accepted, not rejected

Unlike `phfl` version 3 (which carries CIE XYZ needing a transform this repo
cannot ground), `grdm` version 3 differs from version 1 only by the 4-byte
method after the two flags. Accepting versions 1 and 3 matches psd-tools'
validator and the real v3 fixtures, and costs four bytes of skip. Any other
version returns `None`.

### D6. Reject in the decoder, never panic

The decoder returns `None` for: a version other than 1 or 3, a truncated
payload, fewer than two colour stops, a non-increasing location, or a location
above 4096. This matches the existing decoders' range checks and means a
decoded `GradientMap` always passes `apply`'s validation, so a malformed file
cannot silently render a different adjustment and the layer stays a no-op.

### D7. The encoder writes a full version-1 block

`encode_gradient_map(stops: &[GradientStop], reverse: bool)` emits version 1
with an empty unicode name, the supplied stops (colour scaled back to 16-bit),
zero transparency stops, and the trailing fields with psd-tools' defaults
(`expansion` 2, `length` 32, everything else zero), padded to a 4-byte
boundary. The app only needs a block this repo's decoder reads, but matching
the full psd-tools layout keeps a re-saved document well-formed for other PSD
readers. The round-trip is `decode_adjustment(&encode_gradient_map(&stops,
reverse))` equal to `Adjustment::GradientMap` with the same stops and reverse.

### D8. The app authoring path reuses the encoder

`helpers.rs::adjustment_layer` gains a `"gradient-map"` arm returning
`("Gradient Map", encode_gradient_map(&black_to_white(), false))`, matching the
defaults the other kinds use (a freshly added layer visibly changes the
composite). The black→white stops are location 0/colour `[0, 0, 0]` and
location 4096/colour `[255, 255, 255]`. `panel_group_menu.cpp` adds
`imp(QStringLiteral("Gradient Map"),
QStringLiteral("adjustment:gradient-map"))` after the Photo Filter row. No new
bridge method is needed: `add_adjustment` already dispatches on the kind string.
One C++ self-test check (code 285) adds the kind, asserts it is reported as an
adjustment, and asserts the composite changes.

### D9. Fixture

A new `gradient_map()` builder authors `crates/pictura-codec/tests/fixtures/
gradient_map.psd` (RGB 8×8): a `Base` pixel layer plus a `Gradient Map`
adjustment layer built with psd-tools `GradientMap` and `ColorStop`, version 1,
`is_reversed`/`is_dithered` 0, two colour stops (location 0 black, 4096 white
on the 16-bit scale) and two transparency stops (opacity 255), mirroring the
real `gradient-map.psd`. It is registered in `FIXTURES` and regenerated with
`python3 scripts/generate-fixtures.py`. The existing `adjustment.psd` and its
oracle test are untouched. A new oracle test asserts the `grdm` key, the
payload length, and whole-`Document` round-trip; the fixtures README gains the
builder and the new layer.

## Risks / Trade-offs

- **The linear-only interpolation is unverified against Photoshop.** A gradient
  with midpoints or non-linear interpolation will map differently. The layer
  still renders (a plausible gradient instead of a no-op), which is a behaviour
  change from today. D4 records the ceiling; a real CS6 extraction is the
  resolution.
- **The render contract is per-document, not per-oracle.** No Photoshop pixel
  baseline exists for Gradient Map, so `pictura-adjust` tests are property-based
  (identity ramp, reversal, endpoint clamping) rather than differential.
- **Adding a variant touches the adjustment oracle table.** `MAPPING` grows to
  16 rows with `GradientMap` as a no-equivalent row, and the ImageMagick spec
  requirement is updated to match; the test's literal `15` becomes `16`.
- **GPU documents now decode further before being rejected.** A file with a
  `grdm` layer previously returned `None` from `decode_adjustment` and was
  rejected by `gpu/mod.rs`; now it decodes and is still rejected because
  `adjustment_params` has no Gradient Map shader. The fallback outcome is
  identical.

## Migration Plan

None for documents: an existing file is read as before, and a `grdm` layer that
previously no-op'd now renders. The only committed golden artifact is the new
`gradient_map.psd`; no existing fixture changes.

## Open Questions

- Photoshop's exact luminance space and the interpolation curve/midpoint
  behaviour. Resolves with a CS6 ramp-sample extraction (`ADJ-015` open
  questions).
- Whether a real CS6 `grdm` uses the 16-bit colour scale consistently for
  non-greyscale stops. The psd-tools test files confirm the scale for black and
  white; a richer CS6 fixture would confirm intermediate values.
