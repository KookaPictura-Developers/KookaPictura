## Context

`pictura-codec` preserves every adjustment-layer block verbatim but models none
of their payloads. `pictura-render`'s `decode_adjustment` translates a payload
into a `pictura_adjust::Adjustment` for the encodings it understands; anything
else returns `None`, and the composite leaves the backdrop unchanged. Today that
covers `nvrt`/`invr`, `post`, `thrs`, `brit`, `levl`, `hue2`/`hue `, and a
4-byte in-house `SoCo`. `pictura-adjust` already implements Exposure, Vibrance,
and BlackWhite, so the only missing work is the decode.

Two payload families appear:

- **Fixed structs** (`expA`): a version word followed by fixed-width fields.
  Grounded by the Adobe PSD/PSB File Format Specification and mirrored by
  psd-tools 1.19 (`Exposure.read` reads `H3f`).
- **Descriptor blocks** (`vibA`, `blwh`): a 4-byte descriptor version (= 16)
  followed by a descriptor body. Grounded by psd-tools' accessors, which name
  the exact keys Photoshop writes.

`curv`, `selc`, `clrL`, and `gdrm` are descriptor/struct payloads with no
confident schema or no matching `pictura-adjust` operation. `phfl` and `mixr`
are structurally documented but need a colour transform or have an inconsistent
field description. They stay deferred; this is structural decoding, not verified
render parity.

## Goals / Non-Goals

**Goals:**

- Decode `expA`, `vibA`, and `blwh` into `ExposureParams`, `VibranceParams`, and
  `BlackWhiteParams`, matching psd-tools' field interpretation.
- A malformed, truncated, or wrong-version payload returns `None`; no panic, no
  error.
- Parse the descriptor payloads with the codec's existing descriptor DOM,
  exposed as a public function, so `pictura-render` does not grow a second
  descriptor parser.
- Keep every existing decode and every deferred key exactly as it is.

**Non-Goals:**

- Photo Filter (`phfl`) and Channel Mixer (`mixr`) decoding. See Decisions.
- Curves, Selective Color, Color Lookup, Gradient Map, and a real Photoshop
  `SoCo`/`GdFl`/`PtFl` descriptor.
- Colour-space conversion of any kind (XYZ, CMYK, Lab).
- GPU shader support for the new adjustments. They keep falling back to the CPU
  path.
- Verified pixel parity against Photoshop. The committed decode is structural:
  it reads the documented fields and hands them to the existing op.

## Decisions

### D1. Commit only to `expA`, `vibA`, `blwh`

Each committed key has a schema grounded in two independent places (the Adobe
spec and psd-tools), a clear field mapping to an existing `pictura-adjust`
operation, and no colour transform:

| Key | Payload shape | Evidence | Target |
|---|---|---|---|
| `expA` | `u16` version (= 1), `f32` exposure, `f32` offset, `f32` gamma | Adobe spec "Exposure"; psd-tools `Exposure.read` = `read_fmt("H3f")` | `Adjustment::Exposure(ExposureParams)` |
| `vibA` | descriptor version = 16, then object with `vibrance` and `Strt` | Adobe spec "Vibrance"; psd-tools `Vibrance.vibrance`/`.saturation` | `Adjustment::Vibrance(VibranceParams)` |
| `blwh` | descriptor version = 16, then object with `Rd  `, `Yllw`, `Grn `, `Cyn `, `Bl  `, `Mgnt`, `useTint`, `tintColor` | Adobe spec "Black White"; psd-tools `BlackAndWhite` accessors | `Adjustment::BlackWhite(BlackWhiteParams)` |

`tintColor` is a nested `Clr ` object carrying `Rd  `/`Grn `/`Bl  ` doubles on a
0..1 scale (psd-tools returns them raw). Each maps to a byte as
`tint_color = round(component * 255).clamp(0, 255)`. An absent `tintColor`
decodes to black. Missing numeric keys default to psd-tools' documented defaults
(`Rd  ` 40, `Yllw` 60, `Grn ` 40, `Cyn ` 60, `Bl  ` 20, `Mgnt` 80).

### D2. Reject rather than clamp on a malformed numeric field

`expA` requires at least the 14 bytes the fields occupy, version = 1, finite
exposure/offset, and gamma > 0 — the same preconditions `pictura-adjust`'s
exposure op enforces. `vibA`/`blwh` accept the descriptor only when it parses
as an `Object`; a present key of the wrong type is a malformed payload. Values
outside the slider ranges (vibrance/saturation outside −100…100, black-and-white
percentages outside −200…300) are rejected, matching the existing decoders'
range checks. Rejection is `None`, never a clamp, so a corrupt file cannot
silently render a different adjustment.

### D3. Descriptor parsing goes through `pictura-codec`

`pictura-render` already depends on `pictura-codec` (smart-object rendering).
The descriptor DOM lives in `crates/pictura-codec/src/descriptor.rs` with a
crate-private `read_descriptor(&mut Reader)`. The change adds a thin public
wrapper in `crates/pictura-codec/src/lib.rs`:

```rust
pub fn read_descriptor(bytes: &[u8]) -> Result<DescValue, PsdError>
```

`camera_raw_options` keeps calling the private reader through its own `Reader`
and is untouched. The renderer matches on the returned `DescValue::Object`'s
public `items` list to find keys; no new dependency and no second parser.
Using the DOM rather than hand-rolling offsets also inherits its depth cap and
bounds-checked reads, so a truncated descriptor is an error the renderer turns
into `None`.

### D4. Defer `phfl` and `mixr`

- **`phfl` (Photo Filter).** The struct is grounded (Adobe spec: version 2 or 3;
  v3 carries three `u32` XYZ values, v2 a colour space plus four `u16`
  components; then density `u32` and luminosity `u8`). But `PhotoFilterParams`
  wants an sRGB `[u8; 3]`, and converting either variant needs a colour-space
  transform that is not confidently groundable here: v3 is CIE XYZ
  fixed-point, v2 depends on an enumerated space with four components. Decoding
  the fields without the transform would store the wrong colour, which is worse
  than a no-op. Deferred until a conversion baseline exists.
- **`mixr` (Channel Mixer).** The Adobe spec states 20 bytes but describes them
  as "4 * 2 bytes of color with 2 bytes of constant" — ten bytes, not twenty —
  and psd-tools reads only `5h` before treating the remainder as opaque. The
  full twelve-value `ChannelMixerParams` (three output channels of three
  sources plus constants) cannot be confidently laid out from these sources.
  Deferred.

The remaining deferred keys (`curv`, `selc`, `clrL`, `gdrm`, real `SoCo`) are
preserved but not decoded: `curv` is explicitly "highly experimental and
unstable" in psd-tools (the version-1 map format is undocumented), and
`selc`/`clrL`/`gdrm` have no `pictura-adjust` operation to decode into. Note
that Color Balance (`blnc`) is not even in the codec's `ADJUSTMENT_KEYS` set, so
no payload reaches the decoder and it is out of scope.

### D5. Render through the existing adjustment path

`decode_adjustment` returns the `Adjustment`; `composite_adjustment` already
applies any `pictura_adjust::Adjustment` to the backdrop and blends the result
through the layer mask/opacity/blend. No new render code is needed. A composite
test proves one committed key is a non-no-op end to end.

## Risks / Trade-offs

- **Adobe internals are under-specified.** The committed field names come from
  psd-tools, not a normative descriptor dictionary. If Photoshop writes a variant
  that the reader cannot interpret, the decoder returns `None` and the layer is
  a no-op — the same behaviour as today, so the change cannot regress a file.
- **No pixel parity claim.** Black-and-white channel percentages and vibrance
  are handed to the existing ops unchanged; how faithfully those ops reproduce
  Photoshop's mathematics is owned by `pictura-adjust` and its tests, not by
  this decode.
- **GPU documents now decode further before being rejected.** A file with an
  `expA`/`vibA`/`blwh` layer previously returned `None` from `decode_adjustment`
  and was rejected by `gpu/mod.rs`; now it decodes and is still rejected because
  `adjustment_params` has no shader for it. The fallback outcome is identical.
- **Descriptor nesting.** `tintColor` is a nested object; the reader's depth cap
  keeps a crafted payload from exhausting the stack.
