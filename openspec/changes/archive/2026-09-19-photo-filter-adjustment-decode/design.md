## Context

`pictura-codec` preserves the `phfl` (Photo Filter) block verbatim, and
`pictura-adjust` already implements `Adjustment::PhotoFilter` (`PhotoFilterParams
{ color: [u8; 3], density: f64, preserve_luminosity: bool }`). `pictura-render`'s
`decode_adjustment` does not recognise the key, so a Photo Filter adjustment
layer is composited as a no-op. Two earlier decisions left it deferred: the
archived `2026-09-19-adjustment-payload-decode` change (D4) and the roadmap's G8.

The payload is a fixed struct, not a descriptor. psd-tools 1.19
(`psd_tools/psd/adjustments.py`, `PhotoFilter`) reads and writes it, so the
binary layout has an independent reference. What was missing was a confident
mapping from the stored fields to `PhotoFilterParams`'s sRGB `[u8; 3]`.

## Goals / Non-Goals

**Goals:**

- Decode a version-2 `phfl` payload into `Adjustment::PhotoFilter`, so the layer
  renders through the existing adjustment composite path.
- Provide `encode_photo_filter` so the app can create a Photo Filter adjustment
  layer in memory, round-tripping against the decoder.
- Wire the kind `photo-filter` through the app bridge and the Adjustments panel
  menu.
- Extend the psd-tools-authored `adjustment.psd` fixture with a Photo Filter
  layer, so the codec oracle proves the key and payload survive read and write.

**Non-Goals:**

- `phfl` version 3 (CIE XYZ). See D3.
- Curves (`curv`), Channel Mixer (`mixr`), and every other still-deferred key.
- Colour-space conversion of any kind. The decoder treats the components as
  sRGB bytes; it does not convert from the stored colour space.
- A Photo Filter GPU shader. Documents with one keep falling back to the CPU
  path, as today.
- Verified pixel parity against Photoshop. The decoding is structural and the
  render contract belongs to `image-adjustments`/`pictura-adjust`.

## Decisions

### D1. Version-2 byte layout

psd-tools `PhotoFilter.read` is the reference:

```
H   version          (2 or 3)
-- version 3: 3I  xyz
-- version 2: H   color_space, then 4H color_components
I   density          (u32)
B   luminosity       (u8)
    pad to a 4-byte boundary
```

The committed version-2 block is therefore:

| Offset | Size | Field |
|---|---|---|
| 0 | 2 | version (`u16`, must be 2) |
| 2 | 2 | colour space (`u16`, read but not interpreted) |
| 4 | 2 | colour component 0 |
| 6 | 2 | colour component 1 |
| 8 | 2 | colour component 2 |
| 10 | 2 | colour component 3 (ignored) |
| 12 | 4 | density (`u32`, percent) |
| 16 | 1 | luminosity (`u8`, 0/1) |
| 17 | 3 | padding (ignored) |

`decode_photo_filter` reads the fields at these offsets. Reading past the end is
a `None`, so a truncated block never panics.

### D2. The four components map to R, G, B, and are ignored in that order

**Assumption.** The four `u16` colour components are taken as R, G, B, and a
fourth value that is ignored; each of the first three must be `0..=255` or the
payload is rejected. This mapping is grounded only by the psd-tools-authored
fixture, not by a real Photoshop `phfl` file: no PSD in this repository contains
any adjustment key. The colour space (`u16`) and the fourth component are read
and discarded, so a non-RGB colour space is decoded as if it were RGB rather than
rejected. The assumption is deliberately narrow; a real CS6 Photo Filter fixture
would be the baseline that either confirms it or replaces it with a colour-space
transform.

Encoding clamps each colour component to `0..=255` and density to `0..=100`, so
the encoder can always produce a decodable block.

### D3. Version 2 only; version 3 returns `None`

Version 3 carries three `u32` CIE XYZ values. Converting those to the `[u8; 3]`
sRGB filter colour needs an XYZ-to-sRGB transform that is not confidently
groundable here. Returning `None` keeps the layer a no-op, which is exactly
today's behaviour and cannot regress a file. Version 3 is recorded as deferred in
the proposal.

### D4. Reject in the decoder, clamp in the encoder

The decoder returns `None` for: a version other than 2, a truncated payload, a
colour component greater than 255, or a density greater than 100. It does not
clamp, so a corrupt file cannot silently render a different filter. This matches
the existing decoders' range checks and the archived change's D2. The encoder is
the only clamping site, because it is built from typed app input rather than
untrusted bytes.

`preserve_luminosity` is the luminosity byte interpreted as a boolean
(`!= 0`).

### D5. Fixture regeneration

`scripts/generate-fixtures.py` currently does not emit `adjustment.psd`; the
README documents a hand-run snippet instead. The change adds an `adjustment()`
builder that reproduces the documented existing 5 adjustment layers (Invert,
Posterize, Threshold, BrightnessContrast, Levels) plus a new Photo Filter layer,
using `psd_tools.psd.adjustments.PhotoFilter`:

```python
PhotoFilter(version=2, color_space=0,
            color_components=(255, 180, 80, 0), density=25, luminosity=1)
```

The regenerated fixture is a golden file. Regeneration must be verified to leave
the existing layers' bytes unchanged apart from the offsets that appending a
layer shifts. The committed golden change must be stated in the commit message.
The codec oracle's name and payload assertions are updated to include
`PhotoFilter` and to assert the `phfl` bytes.

### D6. The app authoring path reuses the encoder

`helpers.rs::adjustment_layer` gains a `"photo-filter"` arm returning
("Photo Filter", `encode_photo_filter([255, 180, 80], 25, true)`), matching the
defaults the other kinds use (a freshly added layer visibly changes the
composite). `panel_group_menu.cpp` adds
`imp(QStringLiteral("Photo Filter"), QStringLiteral("adjustment:photo-filter"))`
after the Hue-Saturation row. No new bridge method is needed: `add_adjustment`
already dispatches on the kind string.

### D7. Curves stays deferred

`Adjustment::Curves` is a single composite curve; Photoshop stores per-channel
curves, and the legacy `curv` channel-bitmap order is ungrounded. No real
Photoshop fixture in this repository contains any adjustment key, so there is no
baseline to check a mapping against. Decoding now would bind the wrong curve.
This is recorded in the proposal so the docs can capture it; it is not part of
this change.

## Risks / Trade-offs

- **The R,G,B-from-4-components mapping is unverified against Photoshop.** If a
  real `phfl` file stores CMYK or an otherwise different component order, the
  colour is wrong. The layer still renders (a wrong tint instead of a no-op),
  which is a behaviour change from today. The narrow 0..=255 check and the
  assumption note in D2 bound the damage; a real CS6 fixture is the resolution.
- **The committed golden fixture changes.** Appending a layer rewrites
  `adjustment.psd`. The mitigation is D5: verify the existing 5 layers' bytes are
  unchanged apart from offsets, and state the fixture change in the commit.
- **Colour space is ignored.** A version-2 block that is not RGB is decoded as
  RGB. Documented in D2 and the non-goals; a colour-space transform is future
  work.
- **GPU documents now decode further before being rejected.** A file with a
  `phfl` layer previously returned `None` from `decode_adjustment` and was
  rejected by `gpu/mod.rs`; now it decodes and is still rejected because
  `adjustment_params` has no Photo Filter shader. The fallback outcome is
  identical.
- **No pixel parity claim.** The decoded parameters are handed to the existing
  `pictura-adjust` op unchanged; how faithfully that op reproduces Photoshop's
  mathematics is owned by `pictura-adjust` and its tests.

## Migration Plan

None for documents: an existing file is read as before, and a `phfl` layer that
previously no-op'd now renders. The fixture regeneration is the only committed
golden change.

## Open Questions

- The exact component order and scale in a real Photoshop version-2 `phfl`
  block, and whether the fourth component is meaningful. Resolves with a real
  CS6 Photo Filter fixture.
- The colour-space enum values Photoshop writes for preset versus custom
  filters. Resolves with the same fixture.
