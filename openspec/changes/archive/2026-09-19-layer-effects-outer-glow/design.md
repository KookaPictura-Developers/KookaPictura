## Context

A Photoshop layer's **object-based** layer effects live in the `lfx2`
additional-layer-info tagged block (`Tag.OBJECT_BASED_EFFECTS_LAYER_INFO =
b"lfx2"`). Each effect is a key on the top-level descriptor whose value is an
`Objc` whose class id is the effect kind; `masterFXSwitch` is the document-wide
effect switch. `pictura-codec` preserves `lfx2` verbatim in `Layer.extra_blocks`.
The Drop Shadow slice (`layer-effects-drop-shadow`) already decoded `DrSh` and
rendered it; it added `Layer::extra_block`, the CPU matte/dilate/blur/bbox
pipeline, the GPU `UnsupportedLayerEffect` predicate, `blend_parts` and the
`drop_shadow.psd` fixture. This change adds the next effect kind, **Outer Glow**
(`OrGl`), on the same machinery.

The pieces that already exist: `pictura-codec`'s descriptor DOM
(`read_descriptor`/`write_descriptor`, `DescValue::UnitFloat`/`Double`/`Enum`/
`Object`), `pictura-render`'s `layer_effects.rs` (`decode_drop_shadow`,
`content_matte`, `clip_rect`, `pad_rect`, `dilate_matte`, `blur_matte`,
`clamp_finite`, `finite_f32`) and its compositor (`Canvas`, `composite_layer`,
`blend_parts`, `channel`, `desc_item`, `mask_alpha`, `fill_coverage_matte`), and
`pictura-filters`' Gaussian blur. psd-tools / libpsd / ag-psd / psdkit are the
references for the parameter keys and meanings (verified below).

## Goals / Non-Goals

**Goals:**

- Decode the `lfx2` `OrGl` object into a typed `OuterGlow`, with Photoshop
  defaults and a no-op on absent or malformed input.
- Render the outer glow on the CPU behind the layer's own content as an exterior
  matte, reusing the drop-shadow bbox pipeline, matte, dilate and blur.
- Keep the GPU path panic-free by rejecting an enabled glow before dispatch and
  falling back to the CPU composite.
- Prove the decode and render against a psd-tools-authored `outer_glow.psd`
  fixture and hand-built descriptors.

**Non-Goals:**

- Any effect other than Outer Glow; Inner Glow, gradient-mode glows, and the
  precise distance-transform technique.
- Glow fidelity beyond the first slice: `Range`, contour, noise, jitter,
  anti-alias, `Grad`, layer opacity scaling the effect, `Layer Mask Hides
  Effects`.
- The legacy `lrFX` block, styles on groups, `Scale Effects`, presets, authoring
  UI, and a GPU shader.

## Decisions

### D1. `OrGl` is a second kind on the existing `layer_effects` module

No new crate, module, trait, or dependency. `layer_effects.rs` gains
`OuterGlow`, `GlowTechnique`, `decode_outer_glow`, and a glow composite helper
that reuses the existing matte/dilate/blur/bbox helpers. `composite_layer_effects`
(composited from `composite_layer` before the layer content) decodes both kinds
and composites each enabled one.

`OuterGlow`:

```rust
pub enum GlowTechnique { Softer, Precise }

pub struct OuterGlow {
    pub enabled: bool,
    pub present: bool,
    pub blend_mode: BlendMode,
    pub color: [u8; 3],
    pub opacity: f32,        // percent, 0..=100
    pub spread: f32,         // percent, 0..=100 (PSD `Ckmt`, treated as percent)
    pub size: f32,           // pixels, 0..=250, Gaussian radius
    pub technique: GlowTechnique,
}
```

There is no `angle_deg`, `distance`, `use_global_angle`, or `knocks_out`: a glow
has no offset, and it decodes neither `lagl`/`uglg` nor `layerConceals`.

### D2. Descriptor keys and defaults (grounded)

The `lfx2` payload is the same `DescriptorBlock2` as the shadow: a `u32` version,
a `u32` data version (`16`), then the version-16 body. `decode_outer_glow` takes
`&data[4..]`, calls `pictura_codec::read_descriptor`, requires the top-level
object, and finds the `OrGl` `Objc` (class id `OrGl`).

The `OrGl` object's keys (human name → PSD key → value type), from psd-tools
`api/effects.py` + `terminology.py`, libpsd `outer_glow.c`, ag-psd
`descriptor.ts`, and psdkit `effects.dart`:

| Parameter | PSD key | Type | Default |
|---|---|---|---|
| enabled | `enab` | `bool` | false |
| present | `present` | `bool` | false |
| shown in dialog | `showInDialog` | `bool` | false |
| blend mode | `Md  ` | `enum` typeID `BlnM` | Screen (`scrn`) |
| colour | `Clr ` | `RGBC` object, `Rd `/`Grn `/`Bl  ` `doub` 0..=255 | `#FFFFBE` (255,255,190) |
| opacity | `Opct` | unit float percent (or `doub`) | 75 |
| technique | `GlwT` | `enum` typeID `BETE`, `SfBL`/`PrBL` | Softer (`SfBL`) |
| spread | `Ckmt` | unit float (Photoshop tags it `#Pxl`) | 0 |
| size | `blur` | unit float pixels | 5 |
| noise | `Nose` | unit float percent | 0 (ignored) |
| jitter | `ShdN` | unit float percent | 0 (ignored) |
| range | `Inpr` | unit float percent | 50 (ignored) |
| anti-alias | `AntA` | `bool` | (ignored) |
| contour | `TrnS` | descriptor | Linear (ignored) |
| gradient | `Grad` | descriptor | absent (gradient glows deferred) |

Four key names are settled by the evidence, correcting the brief's guesses:

- **Technique is `GlwT` (capital G), not `glwT`.** All four references read
  `GlwT`: libpsd `case 'GlwT'`, ag-psd `case 'GlwT': result.technique =
  BETE.decode(val)`, psdkit writes `key: 'GlwT'`, psd-tools `Key.GlowTechnique =
  b"GlwT"`. Its enum typeID is `BETE` (`psd-tools` `MatteTechnique = b"BETE"`,
  ag-psd `export const BETE = createEnum('BETE', ...)`, psdkit
  `typeId: 'BETE'`), with values `SfBL` (Softer) and `PrBL` (Precise).
- **Spread is `Ckmt`, not `Scl `.** `Scl ` is the effects-set scale (top-level
  `Effects.scale`) and the gradient-overlay scale, not a glow spread. libpsd
  reads `Ckmt` into `outer_glow->spread` (default 0, tagged `#Pxl`); ag-psd maps
  `'Ckmt': result.choke` with the comment `choke?: UnitsValue; // spread` on
  `LayerEffectsOuterGlow`; psdkit's `spread` getter reads `Ckmt` and writes
  `key: 'Ckmt'`. This is the same key the shipped drop-shadow decoder reads as
  `spread`, so the two effects share the field meaning.
- **`ShdN` is jitter, not spread.** psd-tools' `OuterGlow.spread` property reads
  `Key.ShadingNoise` (`ShdN`) and so does its `_GlowEffect.quality_jitter`, which
  is the internal contradiction that marks it a psd-tools bug. ag-psd maps
  `'ShdN': result.jitter` and libpsd maps `case 'ShdN'` to `jitter`; psdkit
  writes `ShdN` as a jitter constant. This slice therefore reads spread from
  `Ckmt` and ignores `ShdN`.
- **The colour default is `#FFFFBE`, not black.** Photoshop's Outer Glow default
  colour is the pale yellow `#FFFFBE`: libpsd's
  `psd_set_layer_outer_glow_default` sets `color = native_color = 0xFFFFFFBE`, and
  the docs table marks it `yellow (#ffffbe)`. A missing `Clr ` decodes to
  (255, 255, 190).

Defaults blend Screen (`scrn`), opacity 75, spread 0, size 5, technique Softer
are corroborated by libpsd's defaults (`psd_blend_mode_screen`, `opacity = 191`
= 75 %, `technique = psd_technique_softer`, `spread = 0`, `size = 5`) and the
docs Outer Glow table. The `present` and `enab` defaults (false) and the
absent-key semantics mirror the drop shadow.

Numeric values are read from either `UnitFloat` or `Double`. The spread unit is
ignored (Photoshop tags glow spread `#Pxl`, the docs define it as a percent), so
the numeric value is clamped to `0..=100` like the shadow. A non-finite value, or
a finite `f64` that overflows to infinity as an `f32`, rejects the effect. A
missing `OrGl`, an unknown data version, a wrong-typed numeric, a wrong `Md  `
typeID, a wrong `GlwT` typeID, or a non-`RGBC` `Clr ` is `None`; the decoder
never panics. An unknown `GlwT` enum value falls back to Softer (mirroring how an
unknown blend key falls back to `Screen`).

**Evidence.** A psd-tools authoring/round-trip probe wrote an `OrGl` with
`Md  `=`scrn`, `Clr ` RGBC (40,80,120), `Opct` 60, `GlwT` `BETE`/`PrBL`, `Ckmt`
20, `blur` 10 and re-read it as `psd_tools.api.effects.OuterGlow` with exactly
those values (`glow_type` `PrBL`, `choke` 20, `size` 10, `blend_mode` `scrn`,
colour 40/80/120, opacity 60). The `drop_shadow.psd` predecessor already proved
the same `DescriptorBlock2` round-trip through `pictura-codec`.

### D3. Exterior matte and render pipeline

A glow has no offset, so the renderer is the drop-shadow pipeline without the
shift and with an exterior knock-out. For an enabled, present `OuterGlow` on a
visible non-group, non-destructive-adjustment layer, with `source =
layer.rect ∩ canvas`:

1. `dilate_radius = round(spread / 100 · size)` and `blur_support =
   3 · sigma_from_radius(size)` (the existing helpers and caps).
2. `padded = pad_rect(source, dilate_radius + blur_support)`; build the content
   matte `M` over `padded` exactly as the shadow does (`content_matte`:
   pixel `-1` alpha, `SoCo`/`GdFl`/`PtFl` fill alpha, or `1.0` for a channel-less
   smart source) and multiply by `mask_alpha/255`. Early-out if `source` is empty
   or `M` is all zero.
3. `halo = blur_matte(dilate_matte(M, dilate_radius), size)`.
4. `glow = halo · (1 − M)`: the exterior mask zeroes the glow where the content
   is opaque. This is the docs' "exterior effects use the outside region `1 − M`"
   and libpsd's `psd_bitmap_knock_out(&dst_bmp, &knock_bmp)` (knock the original
   content alpha out of the blurred alpha). For a pixel layer `M` is `0` outside
   `layer.rect`, so the visible halo outside the rect is unattenuated; the
   knock-out only removes the glow hidden under the content regardless of
   composite order, so a semi-transparent interior shows through proportionally
   to `1 − M`.
5. Tint each pixel by `color` and `opacity/100` and `blend_parts` it into the
   canvas over `padded ∩ canvas`, using the effect's own `blend_mode`, before the
   layer's content (the existing `composite_layer_effects` call site).

`Technique::Softer` and `Technique::Precise` both render as step 3; `Precise` is
decoded and carried only. The docs' "Precise = distance measure" and the
`Range`/contour/noise/jitter controls are deferred and marked as ceilings.

### D4. Shared plumbing, not duplicated math

`blend_parts`, `channel`, `desc_item`, `mask_alpha`, `content_matte`, `clip_rect`,
`pad_rect`, `dilate_matte`, `blur_matte`, `clamp_finite`, `finite_f32`, `num_or`,
`num_clamped`, `bool_or`, `decode_color` are reused unchanged. The glow helper is
the shadow helper minus the angle/distance/offset and plus the exterior knock-out,
so the two stay side by side in one module. The numeric/boolean/colour readers
already ignore the keys a glow does not use, so no reader change is needed. If a
cleaner factoring appears while implementing, factor only size/dilate/blur/matte;
do not add a trait or a generic pipeline with one shape.

### D5. GPU fallback

`check_supported`'s `walk` currently rejects an enabled, present `DropShadow`.
It gains the same test for `decode_outer_glow`, returning the existing
`GpuError::UnsupportedLayerEffect` before any dispatch. The check stays ahead of
the adjustment check so a fill layer with a glow reports the effect error, not
`UnsupportedAdjustment`. `composite_active` / `composite_gpu_or_cpu` already treat
every `GpuError` as a CPU fallback, so no call-site change is needed.

### D6. Fixture and oracle

`scripts/generate-fixtures.py` gains `outer_glow()`: a `Base` pixel layer plus a
`Glowing` layer whose record carries a `DescriptorBlock2` with
`masterFXSwitch: Bool(True)` and an `OrGl` object (`Descriptor` with
`classID=b"OrGl"`) under `Tag.OBJECT_BASED_EFFECTS_LAYER_INFO`. The authored keys
are `enab`, `present`, `showInDialog`, `Md  ` (`BlnM`/`scrn`), `Clr ` RGBC, `Opct`,
`GlwT` (`BETE`/`SfBL` or `PrBL`), `Ckmt`, `blur`, `Nose`, `ShdN`, `AntA`, `TrnS`,
`Inpr`. The new `outer_glow.psd` is registered in `FIXTURES` and regenerated;
existing fixtures must stay byte-identical.

The codec oracle mirrors the drop-shadow one: the `lfx2` block is present in
`extra_blocks` with `version 1` / `data version 16`, the whole `Document`
round-trips `write_psd`/`read_psd` with `lfx2` preserved, and a self-skipping
psd-tools check reads the layer's effect as `OuterGlow` printing
`glow_type.decode()` and `choke` (not `spread`, which reads the buggy `ShdN`) so
the authored technique and spread are asserted. `decode_outer_glow` then decodes
the same bytes and the composite differs from the no-effect composite.

### D7. Deferred fidelity, marked as ceilings

`ponytail:` ceilings record: `Precise` renders as `Softer`; spread maps to a
max-filter dilate of radius `round(spread/100 · size)` (percent of size, not
Photoshop's spread-then-blur split); the `Ckmt` unit (`#Pxl`) is read as a
percent per the docs; `Range`, contour, noise, jitter, anti-alias and gradient
mode are ignored; a gradient-mode glow would render as the decoded (or default)
solid colour; the glow is built over a full-`padded` f32 matte (the existing
compositor ceiling); the knock-out uses `1 − M` (multiplicative) rather than
Photoshop's exact equation.

### D8. No app change

The app composites through `pictura_render::composite_rgba` /
`composite_active`; a decoded glow renders there with no command, panel, or
`CMakeLists.txt` change. No authoring UI and no C++ self-test check are added.
`docs/dev/STATE.md` is updated separately under `TASK-ALLOWS-DOCS`.

## Risks / Trade-offs

- **psd-tools' `OuterGlow.spread` reads the wrong key.** A fixture-oracle that
  asserted `effect.spread` would assert `0` and hide the real spread. Mitigation:
  the oracle asserts `effect.choke` (`Ckmt`) and `effect.glow_type`; the design
  records the psd-tools bug and the four-source evidence.
- **Spread mapping is inferred.** Mitigation: the default is 0 (no-op), the
  fixture pins a non-zero spread, and the mapping is a named ceiling.
- **`Precise` is approximated by `Softer`.** Mitigation: decoded and carried, the
  fixture can pin `SfBL`, and the ceiling is stated.
- **The knock-out equation is approximate.** Mitigation: the exterior zero inside
  opaque content is the only observable contract asserted; partial-alpha parity
  is not claimed.
- **Gradient-mode glows are not detected.** Mitigation: a `Grad` glow still
  renders a solid glow from `Clr ` (or the default yellow), which is visually
  wrong but bounded; gradient support is deferred.
- **The descriptor is untrusted input.** Missing keys default, wrong types and
  non-finite numbers return `None`, and the descriptor reader is depth-capped;
  the decoder never panics.
- **`layer_effects.rs` and its test file grow.** Mitigation: both stay under the
  file-size caps; split the glow tests into their own module if the test file
  approaches 1400 LOC.

## Open Questions

- Whether Photoshop always writes `Ckmt` for a glow spread and `ShdN` for
  jitter; the four references agree, but a CS6 capture would settle it.
- The exact exterior knock-out (multiplicative `1 − M` versus libpsd's
  `knock_out`) and the spread-to-radius mapping need a CS6 pixel baseline.
- Whether `OrGl` ever carries a `Scl ` and whether a first slice should fold the
  top-level effects-set `Scl ` (Scale Effects) into the glow parameters.
