## Context

A Photoshop layer's **object-based** layer effects live in the `lfx2`
additional-layer-info tagged block (`Tag.OBJECT_BASED_EFFECTS_LAYER_INFO =
b"lfx2"`). Each effect is a key on the top-level descriptor whose value is an
`Objc` whose class id is the effect kind; `masterFXSwitch` is the document-wide
effect switch. `pictura-codec` preserves `lfx2` verbatim in `Layer.extra_blocks`.

Six kinds already ship (`DrSh`, `OrGl`, `IrSh`, `IrGl`, `FrFX`, and the
`SoFi`/`GrFl`/`patternFill` overlays) with a shared matte/decode pipeline and two
composite passes (`composite_layer_effects` below the content,
`composite_layer_effects_above` after it). This change adds the seventh, **Satin**
(`ChFX`): a directional interior band derived from a blurred content matte. The
module is split by pure moves into `layer_effects/{mod,shadows,glows,strokes,overlays}.rs`;
satin becomes a fifth file.

The pieces that already exist and are reused unchanged:

- `crates/pictura-render/src/layer_effects/mod.rs` — the descriptor readers
  `num_or`, `finite_f32`, `num_clamped`, `bool_or`, `decode_color`; the entry
  points `composite_layer_effects` and `composite_layer_effects_above`;
  `clamp_finite`, `content_matte`, `clip_rect`, `rect_empty`, `pad_rect`;
  `blur_matte`; and the caps `MAX_OPACITY = 100.0`, `MAX_DISTANCE = 30_000.0`,
  `MAX_SIZE = 250.0`.
- `crates/pictura-render/src/composite.rs` — `blend_parts`, `mask_alpha`,
  `desc_item`, `Canvas`, used by every shipped effect.
- `crates/pictura-render/src/layer_effects/shadows.rs` — the structural template
  for an above-content, interior-only composite: `decode_inner_shadow` and
  `composite_inner_shadow` (the padded-region matte, blur and `M`-confined
  above-content loop).
- `crates/pictura-render/src/layer_effects/overlays.rs` — the `read_effect` /
  `effect_object` / `decode_blend_mode` descriptor helpers and the shared
  `common` decode; satin mirrors the equivalent inline reads from `shadows.rs`
  rather than reaching into the overlays' private helpers.
- `crates/pictura-render/src/gpu/mod.rs` — `check_supported`'s `walk` predicate
  (lines 273-320), which already rejects the six shipped kinds.

### Ground truth read

- The preceding changes under
  `openspec/changes/archive/2026-09-19-layer-effects-*/`,
  `2026-09-20-layer-effects-stroke/` and `2026-09-20-layer-effects-overlays/`,
  plus the shipped `crates/pictura-render/src/layer_effects/{mod,shadows,glows,strokes,overlays}.rs`.
- `docs/05-layers/layer-styles.md`:
  - The **Satin** row (L36): "Interior shading that produces a satin finish."
  - The algorithm (L279-283): "Build an interior distance field at `Angle`; use
    `Distance` and `Size` to shape a band, apply the contour and `Invert`, then
    composite inside the layer with the Satin blend mode. It is the bevel's
    distance field without the lighting."
  - The parameter list (L182-186): `Blend Mode` (Multiply `(inferred)`), `Color`
    (black `(inferred)`), `Opacity` (**50 %**), `Angle` (**19°** `(inferred)`),
    `Distance` (**11 px** `(inferred)`), `Size` (**14 px** `(inferred)`),
    `Contour`, `Anti-alias`, `Invert` (**off**). `Scale Effects` (L305) names
    Satin `Distance`/`Size` as pixel parameters.
  - The application order (L309-316): Drop Shadow beneath the layer; the surface
    effects (Bevel, Satin, Overlays, Stroke) on/within it.

- **The effect class id and top-level key are `ChFX`** (ChromeFX):
  - libpsd `src/effects.c:308` `case 'ChFX':` calls `psd_get_layer_satin2`;
    `src/satin.c` `psd_get_layer_satin2` parses the satin object. libpsd's
    defaults are `multiply`, black, opacity 128 (≈ 50 %), angle 19, distance 11,
    size 14, identity contour, and `invert = psd_true`.
  - psd-tools `psd_tools/api/effects.py:537-538` `@register(Klass.ChromeFX.value)`
    `class Satin`; `psd_tools/terminology.py:206` `Klass.ChromeFX = b"ChFX"`.
    `Effects.__init__` (`effects.py:61-72`) looks up each effect by `item.classID`.
  - Live round-trip probe (this session): a `DescriptorBlock2` authored with
    `masterFXSwitch` plus a `ChFX` `Objc` saves and reopens; psd-tools reports an
    `Effects(satin)` whose keys are exactly
    `[enab, present, showInDialog, Md  , Clr , Opct, uglg, lagl, Dstn, blur, Invr, AntA, MpgS]`,
    decoding to `Satin enabled=True present=True blend=mul color={Rd:10,Grn:20,Bl:30} opacity=50 inverted=False angle=19 distance=11 size=14`.
    Authoring the object under any other id fails psd-tools with
    `ValueError: Effect class not found`.

- **Exact keys and defaults** (libpsd `src/satin.c` parser + psd-tools
  `Satin`/`_ColorMixin`/`_Effect` accessors + the probe):

  | Struct field | Key | TypeID | Default (Photoshop UI / docs) | psd-tools accessor |
  |---|---|---|---|---|
  | `enabled` | `enab` | `bool` | false | `_Effect.enabled` |
  | `present` | `present` | `bool` | false | `_Effect.present` |
  | `blend_mode` | `Md  ` | `BlnM` enum | Multiply | `_ColorMixin.blend_mode` |
  | `color` | `Clr ` | `RGBC` obj | black | `_ColorMixin.color` |
  | `opacity` | `Opct` | `UntF` | 50 | `_Effect.opacity` (100 for absent) |
  | `angle_deg` | `lagl` | `UntF` `#Ang` | 19 | `Satin.angle` (0 for absent) |
  | `distance` | `Dstn` | `UntF` `#Pxl` | 11 | `Satin.distance` (120 for absent) |
  | `size` | `blur` | `UntF` `#Pxl` | 14 | `Satin.size` (120 for absent) |
  | `invert` | `Invr` | `bool` | false (docs) | `Satin.inverted` |
  | (ignored) | `uglg`, `showInDialog`, `AntA`, `MpgS` | | | |

  Two divergences are recorded rather than silently copied: the docs list
  `Invert` default **off** while libpsd defaults it **on**, and libpsd's contour
  key is `MpgS` (psd-tools `Satin.contour` uses `Key.MappingShape`), **not**
  `TrnS` (the task brief's hint). The decoder follows the docs' UI defaults and
  reads the psd-tools/libpsd-confirmed `MpgS`.

- **The render algorithm** (libpsd `src/satin.c`
  `psd_layer_effects_blend_satin` + `psd_satin_blend_offset`; `src/bitmap.c`
  `psd_inflate_bitmap`, `psd_bitmap_knock_out`): the effect region is the layer
  rect padded by `size`; the content alpha is blurred by `size`; the blurred
  field is differenced against a copy shifted by `distance` along `angle`; the
  absolute difference is the band; `invert` uses `255 − band`; the result is
  multiplied by the content alpha (libpsd does this twice) and composited above
  the content with the colour, opacity and blend mode.

## Goals / Non-Goals

**Goals:**

- Decode the `lfx2` `ChFX` object into a typed `Satin`, reusing the shipped
  descriptor readers, with Photoshop UI defaults and a no-op on absent or
  malformed input.
- Render satin on the CPU as a directional interior band above the content,
  confined to the masked content matte `M`.
- Keep the GPU path panic-free by rejecting an enabled and present satin before
  dispatch and falling back to the CPU composite.
- Prove the decode and render against a psd-tools-authored fixture and hand-built
  descriptors.

**Non-Goals:**

- Contour (`MpgS`), anti-alias (`AntA`), the global-light resource, `uglg` and
  `showInDialog`.
- libpsd's second (knockout) multiplication of the content coverage.
- Bevel & Emboss (`ebbl`), the legacy `lrFX` block, styles on groups, the
  isolated `Blend Interior Effects As Group` composite, `Scale Effects`, a GPU
  shader and any authoring UI.

## Decisions

### D1. One new file, no new abstractions

No new crate, module trait, or dependency. `mod.rs` already holds the shared
plumbing and gains only `mod satin;`, the re-exports, one call and a doc
paragraph. The new file:

```
crates/pictura-render/src/layer_effects/satin.rs
  Satin
  decode_satin
  composite_satin
```

`mod.rs` gains `mod satin;` and `pub use satin::{decode_satin, Satin};`; `lib.rs`'s
`pub use layer_effects::{...}` list gains the two items. The file is well under
the 1200 LOC cap.

### D2. Typed params

```rust
pub struct Satin {
    pub enabled: bool,
    pub present: bool,
    pub blend_mode: BlendMode, // Md  /BlnM, default Multiply
    pub color: [u8; 3],        // Clr /RGBC, default black
    pub opacity: f32,          // Opct, percent, 0..=100, default 50
    pub angle_deg: f32,        // lagl, default 19
    pub distance: f32,         // Dstn, px, 0..=30000, default 11
    pub size: f32,             // blur, px, 0..=250, default 14
    pub invert: bool,          // Invr, default false
}
```

No `use_global_angle` field: libpsd's satin parser has no `uglg` case and the
docs say the angle is local; an `uglg` item, if present, is ignored.

### D3. Descriptor keys and decode contract (grounded)

`decode_satin` takes `layer.extra_block(b"lfx2")`, requires at least `8` bytes,
hands `&data[4..]` to `pictura_codec::read_descriptor`, requires the top-level
object, and finds the `ChFX` `Objc` whose class id is `ChFX`. It reads:

- `enab`/`present` via `bool_or` (default false);
- `Md  ` (typeID `BlnM`): absent or unknown value → `Multiply`; a wrong typeID
  rejects;
- `Clr ` (`RGBC`, default black) via `decode_color`; a non-`RGBC` `Clr ` rejects;
- `Opct` via `num_clamped(.., 50.0, 0.0, MAX_OPACITY)`;
- `lagl` via `num_or(.., 19.0)` (no clamp, matching the shipped angle handling);
- `Dstn` via `num_clamped(.., 11.0, 0.0, MAX_DISTANCE)`;
- `blur` via `num_clamped(.., 14.0, 0.0, MAX_SIZE)`;
- `Invr` via `bool_or(.., false)`;
- `MpgS`, `AntA`, `uglg`, `showInDialog` are ignored.

Numerics are accepted as `UnitFloat` or `Double`; a value that is non-finite, or
finite as `f64` but overflows to infinity as `f32`, rejects. A missing `lfx2`, a
missing `ChFX`, an unknown data version, a wrong class id, a wrong-typed numeric,
a wrong `Md  ` typeID, a non-`RGBC` `Clr `, or a descriptor that fails to parse
is `None`; the decoder never panics. An effect whose `present` or `enab` is false
decodes but renders nothing.

This mirrors the `shadows.rs` descriptor reads; the overlay-only helpers
(`read_effect`, `effect_object`, `common`) stay private to `overlays.rs` rather
than being promoted, keeping the diff scoped.

### D4. The satin formula

Let `M` be the masked content coverage (pixel `-1` alpha, fill payload alpha, or
`1.0` inside the rect of a channel-less smart source, multiplied by
`mask_alpha/255`, exactly as the shipped effects), and let `B = blur(M, size)` be
a Gaussian blur of `M` by `size` pixels. With the clamped `distance`, the finite
`angle`, and screen coordinates (y down):

```
dx = -round(distance · cos(angle))
dy = +round(distance · sin(angle))
band(x, y) = |B(x - dx, y - dy) - B(x + dx, y + dy)|      (clamped to 0..=1)
field(x, y) = invert ? 1 - band : band
satin(x, y)  = M(x, y) · field(x, y) · opacity/100
```

composited with `blend_parts(canvas, x, y, color/255, satin, blend_mode)` **above**
the layer content. This is libpsd's `psd_satin_blend_offset`: the difference of
the blurred distance field against itself shifted along the angle is the
directional band, and multiplying by `M` confines it to the interior. It matches
the docs' "interior distance field at `Angle` … the bevel's distance field
without the lighting".

- The shift sign convention (`dx = −distance·cos`, `dy = +distance·sin`) matches
  the shipped drop/inner-shadow offset, so `Invert` flips the band's polarity.
- The build region is `padded = pad_rect(source, |dx|max + blur_support)`, where
  `source = clip_rect(layer)` and `blur_support = ceil(3·sigma_from_radius(size))`.
  `B` is sampled with bounds checks (out-of-region = 0). libpsd pads by `size`
  only; padding by `dist_reach + blur_support` avoids truncating the band when
  `distance` approaches or exceeds `size`. This is a stated, bounded deviation.
- Clamp `distance`/`size`/`opacity` again in the composite (a hand-built struct
  must not panic); `angle` non-finite is treated as 0.

ponytail: a canvas-filling layer with the maximum `size` still costs
O(canvas · size) because the blur is a naive separable kernel; the common
small-layer and crafted off-canvas cases are bounded.

### D5. Compositing above the content, order

Satin is a surface effect, so it composites **after** the layer's content.
`composite_layer_effects_above` gains the satin check after Inner Shadow and
Inner Glow and **before** the overlays and Stroke:

```
inner shadow -> inner glow -> satin -> color overlay -> gradient overlay -> pattern overlay -> stroke
```

matching the CS6 effects-list order (Bevel, Satin, Overlays, Stroke). The shared
group / destructive-adjustment skip is unchanged. ponytail: Photoshop's exact
inter-effect order among the above-content effects is not modelled.

### D6. Bounding and early-out

Satin is confined to `source = clip_rect(layer, w, h)`; the matte and blur are
built over `padded` (above). Early-out on an empty `source` or `padded`, an
all-zero `M`, or `opacity == 0`. Every pixel outside `source` is byte-identical
to the same document without the effect. A zero `distance` with `invert` false
yields a zero band (no-op); with `invert` true it yields `satin = M · opacity`,
a uniform interior tint (libpsd's behaviour).

### D7. GPU fallback

`check_supported`'s `walk` (`crates/pictura-render/src/gpu/mod.rs:273-320`)
gains the same enabled-and-present test for `decode_satin`, returning the
existing `GpuError::UnsupportedLayerEffect` before any dispatch, ahead of the
adjustment check and after the existing effect checks. `composite_active` /
`composite_gpu_or_cpu` already treat every `GpuError` as a CPU fallback, so no
call-site change is needed. A malformed satin returns `None` and does not reject.

### D8. Fixture and oracle

`scripts/generate-fixtures.py` gains `satin()`: a `Base` pixel layer plus a
`ChFX`-bearing pixel layer whose record carries a `DescriptorBlock2` with
`masterFXSwitch: Bool(True)` and `ChFX` under
`Tag.OBJECT_BASED_EFFECTS_LAYER_INFO`, exactly as the `stroke()` builder does.
The authored `ChFX` uses non-default values that exercise every key —
`Md  ` `BlnM`/`mul `, a non-black `Clr ` `RGBC`, `Opct` 50, `uglg` false, `lagl`
120, `Dstn` 8, `blur` 6, `Invr` true, `AntA` true, `MpgS`
(`Descriptor({Name: "Linear"}, classID=b"TrnS")`) — so the oracle proves the
invert and contour keys survive. The new `satin.psd` is registered in `FIXTURES`
and regenerated; existing fixtures must stay byte-identical.

The codec oracle mirrors the predecessor ones: the `lfx2` block is present in
`extra_blocks` with version 1 / data version 16, the whole `Document` round-trips
`write_psd`/`read_psd` with `lfx2` preserved, and a self-skipping psd-tools check
reads the effect as a `Satin` asserting `enabled`, `present`, `opacity`,
`blend_mode`, `inverted`, `angle`, `distance`, `size` and the colour. `decode_satin`
then decodes the same bytes and the composite differs from the no-effect composite.

### D9. Deferred fidelity, marked as ceilings

`ponytail:` ceilings record: contour (`MpgS`), anti-alias (`AntA`), `uglg` and
`showInDialog` are ignored; the effective angle is the stored `lagl`, not the
global-light resource 1037; the band is confined to `M` once rather than libpsd's
extra knockout multiplication (which squares `M`); the build region pads by
`dist_reach + blur_support` rather than libpsd's `size`; `Scale Effects` is not
applied; groups, adjustment layers and smart filters carry no effect; the exact
inter-effect order is not modelled.

### D10. No app change

The app composites through `pictura_render::composite_rgba` / `composite_active`;
a satin renders there with no command, panel, or `CMakeLists.txt` change. No
authoring UI and no C++ self-test check are added.

## Risks / Trade-offs

- **The brief named `TrnS` as the satin contour key.** libpsd and psd-tools both
  use `MpgS` (`MappingShape`); keying on `TrnS` would miss a real contour. The
  decoder ignores the contour entirely, so this is documentation-level only, and
  D3 records the two-oracle anchor.
- **The docs' `Invert` default (off) diverges from libpsd's (on).** The decoder
  follows the docs; the render formula is polarity-symmetric, so either default
  is a one-bit difference and the fixture pins `Invr`.
- **The descriptor is untrusted input.** Missing keys default, wrong types and
  non-finite numbers return `None`, and the descriptor reader is depth-capped;
  the decoder never panics.
- **The pad-region deviation from libpsd.** Padding by `dist_reach + blur_support`
  is at least as correct as libpsd's `size` pad and stays canvas-clamped; it is
  recorded as a ceiling.

## Open Questions

- Photoshop's exact satin contour LUT and anti-alias behaviour, and the precise
  interaction of `distance` with `size`, need a CS6 pixel baseline; the docs give
  a behavioural description but no reference render.
- Whether PS writes a satin object under a legacy spelling; a real CS6 file would
  settle it, and the decoder already ignores unknown ids.
