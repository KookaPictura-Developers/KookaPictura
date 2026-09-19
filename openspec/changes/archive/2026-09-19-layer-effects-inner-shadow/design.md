## Context

A Photoshop layer's **object-based** layer effects live in the `lfx2`
additional-layer-info tagged block (`Tag.OBJECT_BASED_EFFECTS_LAYER_INFO =
b"lfx2"`). Each effect is a key on the top-level descriptor whose value is an
`Objc` whose class id is the effect kind; `masterFXSwitch` is the document-wide
effect switch. `pictura-codec` preserves `lfx2` verbatim in `Layer.extra_blocks`.
The Drop Shadow slice (`layer-effects-drop-shadow`) added `Layer::extra_block`,
the CPU matte/dilate/blur/bbox pipeline, the GPU `UnsupportedLayerEffect`
predicate, `blend_parts` and the `drop_shadow.psd` fixture; the Outer Glow slice
(`layer-effects-outer-glow`) added `OrGl` as a second kind on the same machinery.
This change adds the next kind, **Inner Shadow** (`IrSh`), whose shadow lies
**inside** the layer's coverage and therefore composites **above** the content.

The pieces that already exist: `pictura-codec`'s descriptor DOM
(`read_descriptor`/`write_descriptor`, `DescValue::UnitFloat`/`Double`/`Enum`/
`Object`), `pictura-render`'s `layer_effects.rs` (`decode_drop_shadow`,
`decode_outer_glow`, `content_matte`, `blur_matte`, `dilate_matte`, `clip_rect`,
`pad_rect`, `clamp_finite`, `finite_f32`) and its compositor (`Canvas`,
`composite_layer`, `blend_parts`, `channel`, `desc_item`, `mask_alpha`,
`fill_coverage_matte`), and `pictura-filters`' Gaussian blur. psd-tools, libpsd
and ag-psd are the references for the parameter keys and meanings (verified
below).

## Goals / Non-Goals

**Goals:**

- Decode the `lfx2` `IrSh` object into a typed `InnerShadow`, with Photoshop
  defaults and a no-op on absent or malformed input.
- Render the inner shadow on the CPU as an interior matte confined to the layer's
  content, composited above the content, reusing the drop-shadow bbox pipeline,
  matte and blur and re-introducing a min-filter erode for `choke`.
- Keep the GPU path panic-free by rejecting an enabled inner shadow before
  dispatch and falling back to the CPU composite.
- Prove the decode and render against a psd-tools-authored `inner_shadow.psd`
  fixture and hand-built descriptors.

**Non-Goals:**

- Any effect other than Inner Shadow; Inner Glow, gradients and the other kinds.
- The `layerConceals` fireback behaviour, layer opacity scaling the effect,
  `Layer Mask Hides Effects`, the isolated `Blend Interior Effects As Group`
  composite, contour (`TrnS`), noise (`Nose`) and anti-alias (`AntA`).
- The document global-light resource (1037), the legacy `lrFX` block, styles on
  groups, `Scale Effects`, presets, authoring UI, and a GPU shader.

## Decisions

### D1. `IrSh` is a third kind on the existing `layer_effects` module

No new crate, module, trait, or dependency. `layer_effects.rs` gains
`InnerShadow`, `decode_inner_shadow`, and a composite helper that reuses the
existing matte/blur/bbox helpers and adds one erode helper. The module's
`composite_layer_effects` is split by composite order (D5): a below-content pass
(Drop Shadow, Outer Glow) and an above-content pass (Inner Shadow).

```rust
pub struct InnerShadow {
    pub enabled: bool,
    pub present: bool,
    pub blend_mode: BlendMode,   // default Multiply
    pub color: [u8; 3],          // default black
    pub opacity: f32,            // percent, 0..=100
    pub angle_deg: f32,          // stored local angle (`lagl`), degrees
    pub distance: f32,           // pixels, 0..=30000
    pub choke: f32,              // percent, 0..=100 (`Ckmt`), an erode
    pub size: f32,               // pixels, 0..=250, Gaussian radius
    pub use_global_angle: bool,  // decoded, effective angle uses `lagl`
    pub knocks_out: bool,        // decoded (`layerConceals`), no render effect
}
```

### D2. Descriptor keys and defaults (grounded)

The `lfx2` payload is the same `DescriptorBlock2` as the other kinds: a `u32`
version, a `u32` data version (`16`), then the version-16 body.
`decode_inner_shadow` takes `&data[4..]`, calls `pictura_codec::read_descriptor`,
requires the top-level object, and finds the `IrSh` `Objc` (class id `IrSh`).

The `IrSh` object's keys (human name → PSD key → value type), from psd-tools'
`api/effects.py` accessors, `psd/terminology.py`, and libpsd's
`psd_layer_effects_inner_shadow` struct:

| Parameter | PSD key | Type | Default | DrSh drift |
|---|---|---|---|---|
| enabled | `enab` | `bool` | false | same |
| present | `present` | `bool` | false | same |
| shown in dialog | `showInDialog` | `bool` | false | same |
| blend mode | `Md  ` | `enum` typeID `BlnM` | **Multiply** (`mul `) | DrSh defaults Normal |
| colour | `Clr ` | `RGBC` object, `Rd `/`Grn `/`Bl  ` `doub` 0..=255 | black | same |
| opacity | `Opct` | unit float percent (or `doub`) | 75 | same |
| use global light | `uglg` | `bool` | true | same |
| local angle | `lagl` | unit float degrees | 120 | same |
| distance | `Dstn` | unit float pixels | 5 | same |
| choke | `Ckmt` | unit float percent | 0 | **DrSh reads the same key as Spread (dilate); IrSh reads it as Choke (erode)** |
| size | `blur` | unit float pixels | 5 | same |
| contour | `TrnS` | descriptor | Linear | same (ignored) |
| anti-alias | `AntA` | `bool` | true | same (ignored) |
| noise | `Nose` | unit float percent | 0 | same (ignored) |
| knock out | `layerConceals` | `bool` | true | DrSh-only control; carried, no render effect |

The keys are identical to `DrSh` except for the meaning of `Ckmt` and the
absence of a real knock-out control. `Ckmt` is the choke (erode) for Inner
Shadow, exactly the erode direction the drop-shadow slice removed when it
corrected `Ckmt` to Spread: the docs' Inner Shadow entry says **Choke** (0 %,
0–100; shrinks the matte before blur), whereas Drop Shadow uses Spread (dilates).
`layerConceals` is a Drop Shadow dialog control: libpsd's
`psd_layer_effects_inner_shadow` has no knock-out field (its struct is
`effect_enable, blend_mode, color, native_color, opacity, angle, use_global_light,
distance, choke, size, contour_lookup_table, anti_aliased, noise`) and psd-tools'
`InnerShadow` exposes no `layer_knocks_out` (only `DropShadow` does). The decoder
therefore reads `layerConceals` if present for structural symmetry with
`decode_drop_shadow`, but the field has no observable render effect and the
interior confinement is unconditional; this is a stated parity ceiling, not a
claim.

Numeric values are read from either `UnitFloat` or `Double`. A missing `Md  `
decodes to Multiply, a missing colour to black, an unknown blend key falls back
to Multiply (mirroring the other kinds), and absent numeric/boolean keys take the
defaults above. A finite value outside its documented range is clamped (`opacity`
and `choke` `0..=100`, `distance` `0..=30000`, `size` `0..=250`); a non-finite
value, or a finite `f64` that overflows to infinity as an `f32`, rejects the
effect. A missing `lfx2`, a missing `IrSh`, an unknown data version, a
wrong-typed numeric, a wrong `Md  ` typeID, a non-`RGBC` `Clr `, or a descriptor
that fails to parse is `None`; the decoder never panics. A `IrSh` whose `present`
or `enab` is false decodes but renders nothing.

**Evidence.** psd-tools authoring/round-trip probe: a `DescriptorBlock2` with
`masterFXSwitch` and an `IrSh` object (`Md  ` `BlnM`/`mul `, `Clr ` RGBC
(10,20,30), `Opct` 75, `uglg` false, `lagl` 120, `Dstn` 5, `Ckmt` 0, `blur` 5,
`TrnS` Linear, `AntA` true, `Nose` 0) under `Tag.OBJECT_BASED_EFFECTS_LAYER_INFO`
wrote a header `lfx2 … 00000001 00000010` (version 1, data version 16) and
re-read as `psd_tools.api.effects.InnerShadow` with exactly those values:
`enabled` true, `present` true, `blend_mode` `mul `, colour 10/20/30, `opacity`
75, `choke` 0, `size` 5, `angle` 120, `distance` 5, `use_global_light` false, and
no `layer_knocks_out` attribute. The predecessor fixtures already proved the same
`DescriptorBlock2` round-trip through `pictura-codec`.

### D3. Angle: stored `lagl`, global light deferred

As with Drop Shadow, psd-tools' `_AngleMixin.angle` returns the document global
angle resource (default 30) when `uglg` is set and `lagl` otherwise.
Kooka Pictura does not decode image resources, so this slice reads `lagl`
(default 120) and renders with it; `uglg` is decoded for a later slice. This is a
stated parity ceiling (D9), not a claim of parity. The fixture authors `uglg`
false so the stored `lagl` is the effective angle.

### D4. Interior matte: invert, offset, erode, blur, confine

The inner shadow is the drop shadow of the **inverted** content matte, clipped to
the content. Let `M` be the masked content coverage (D4 of the drop-shadow
design: pixel `-1` alpha, `SoCo`/`GdFl`/`PtFl` fill alpha, or `1.0` inside the
rect of a channel-less smart source, multiplied by `mask_alpha/255`). For a
shadow at angle `θ` and distance `d`, the screen-space offset is the same as the
drop shadow, `dx = -d·cos θ` and `dy = +d·sin θ` (y down). The interior matte at
canvas `(x, y)` is

```
shadow(x, y) = M(x, y) · B(x − dx, y − dy)
```

where `B = blur(erode(1 − M, choke_radius), size)`. The `M` factor is the
**interior confinement**: every pixel where the content is transparent (`M = 0`)
is unchanged. The offset on the inverted matte with the same `dx`/`dy` as an
outer shadow puts the shadow on the interior edge opposite the outer-shadow
direction (default 120° → the upper-left interior), mirroring Photoshop's
convention that the inner shadow falls toward the light. A sample of `B` outside
the built region is treated as `1` (the exterior value), so a canvas-edge or
small-layer sample stays correct. `blur` commutes with the integer offset, so
`B` is built unshifted and sampled shifted at composite time, exactly like the
drop-shadow pipeline.

`choke` erodes the inverted matte before the blur (a min filter of radius
`round(choke / 100 · size)`), narrowing the shadow; `size` is the Gaussian
radius. Both are clamped again in the composite helper so a hand-built
`InnerShadow` cannot panic the filter. The blurred matte is multiplied by
`M`, by `opacity/100` and by the colour, giving the effect's source alpha and
colour.

### D5. Compositing above the content

The drop-shadow and outer-glow slices composite from `composite_layer_effects`
**before** the layer's content. An inner shadow must composite **after** it,
otherwise opaque content hides it. `composite_layer` (in `composite.rs`) is
therefore:

1. the below-content pass (Drop Shadow, Outer Glow) — the existing
   `composite_layer_effects`, unchanged;
2. the content (group / adjustment / pixels / smart source);
3. an above-content pass for the Inner Shadow on a visible non-group,
   non-destructive-adjustment layer.

Splitting the effect call into a below and an above pass keeps the shared
group/destructive-adjustment skip and adds no trait or generic pipeline. The
above pass reuses `blend_parts` with the effect's own `blend_mode` and its matte
alpha weighted by `opacity/100`; the layer's opacity, fill and mask are not
applied a second time. The shadow is composited over the region
`source = layer.rect ∩ canvas` only, so pixels outside the content rect are
byte-identical to the no-effect composite (a stronger bbox bound than the outer
effects, which need padding).

ponytail: compositing the interior effect after the layer's content means it
blends against the layer-over-backdrop result rather than an isolated content
buffer. `Blend Interior Effects As Group` (which would isolate interior effects
before the layer blends) and the exact Photoshop "shade the layer content" order
are deferred; the observable contract (interior-only, exterior byte-identical,
tinted by colour/opacity/blend) holds.

### D6. Shared plumbing, not duplicated math

`blend_parts`, `channel`, `desc_item`, `mask_alpha`, `content_matte`, `clip_rect`,
`pad_rect`, `dilate_matte`, `blur_matte`, `clamp_finite`, `finite_f32`, `num_or`,
`num_clamped`, `bool_or` and `decode_color` are reused unchanged. One helper is
re-introduced:

```rust
fn erode_matte(src: &[f32], w: usize, h: usize, r: i64) -> Vec<f32>
```

mirroring `dilate_matte` with an `axis_min` pass (`f32::INFINITY` seed, min
window) instead of `axis_max`. The numeric/boolean/colour readers already ignore
keys an effect does not use, so no reader change is needed. If a cleaner factoring
appears while implementing, factor only the shared invert/offset/erode/blur
steps; do not add a trait or a generic pipeline with one shape.

### D7. GPU fallback

`check_supported`'s `walk` currently rejects an enabled, present `DropShadow` and
`OuterGlow`. It gains the same test for `decode_inner_shadow`, returning the
existing `GpuError::UnsupportedLayerEffect` before any dispatch, ahead of the
adjustment check. `composite_active` / `composite_gpu_or_cpu` already treat every
`GpuError` as a CPU fallback, so no call-site change is needed.

### D8. Fixture and oracle

`scripts/generate-fixtures.py` gains `inner_shadow()`: a `Base` pixel layer plus
an `Inner` layer whose record carries a `DescriptorBlock2` with
`masterFXSwitch: Bool(True)` and an `IrSh` object (`Descriptor` with
`classID=b"IrSh"`) under `Tag.OBJECT_BASED_EFFECTS_LAYER_INFO`. The authored keys
are `enab`, `present`, `showInDialog`, `Md  ` (`BlnM`/`mul `), `Clr ` RGBC
(10,20,30), `Opct` 75, `uglg` `Bool(False)`, `lagl` 120 `Angle`, `Dstn` 5
`Pixels`, `Ckmt` 0 `Percent`, `blur` 5 `Pixels`, `TrnS` Linear, `AntA` true,
`Nose` 0 `Percent`. The new `inner_shadow.psd` is registered in `FIXTURES` and
regenerated; existing fixtures must stay byte-identical.

The codec oracle mirrors the predecessor ones: the `lfx2` block is present in
`extra_blocks` with version 1 / data version 16, the whole `Document` round-trips
`write_psd`/`read_psd` with `lfx2` preserved, and a self-skipping psd-tools check
reads the layer's effect as `InnerShadow` asserting `enabled`, `present`,
`opacity`, `blend_mode`, `choke`, `size`, `angle`, `distance` and colour (there
is no `layer_knocks_out` property to assert). `decode_inner_shadow` then decodes
the same bytes and the composite differs from the no-effect composite.

### D9. Deferred fidelity, marked as ceilings

`ponytail:` ceilings record: the effective angle is `lagl`, not the global-light
resource; `choke` maps to a min-filter erode of radius `round(choke/100 · size)`;
the offset uses the same convention as the drop shadow and is sampled at integer
pixels; `knocks_out` is decoded but has no effect (the confinement is
unconditional); the interior effect composites against the layer-over-backdrop
result rather than an isolated content buffer; layer opacity does not scale the
effect; the layer mask is folded into the matte rather than gated by `Layer Mask
Hides Effects`; contour, noise and anti-alias are ignored; groups, adjustment
layers and smart filters carry no effect; the matte is built over a bounded
padded `f32` buffer (the existing compositor ceiling).

### D10. No app change

The app composites through `pictura_render::composite_rgba` /
`composite_active`; a decoded effect renders there with no command, panel, or
`CMakeLists.txt` change. No authoring UI and no C++ self-test check are added.
`docs/dev/STATE.md` is the only doc that would name the change, and it is updated
separately under `TASK-ALLOWS-DOCS`.

## Risks / Trade-offs

- **`knocks_out` is decoded but inert.** Photoshop's Inner Shadow dialog and both
  references have no knock-out control, but the required typed shape carries one
  for symmetry with `DropShadow`. Mitigation: the fixture omits `layerConceals`,
  the default is documented, and the field is stated to have no observable effect.
- **The interior effect composites after the layer, not against an isolated
  content buffer.** `Blend Interior Effects As Group` is deferred. Mitigation:
  the observable contract tested is interior-only, exterior byte-identical, and
  tinted by colour/opacity/blend; partial-alpha exactness is not claimed.
- **Choke mapping is inferred.** Mitigation: the default is 0 (no-op), a
  non-zero `choke` is exercised in tests for a narrower shadow, and the mapping is
  a named ceiling.
- **The effective angle ignores global light.** Mitigation: `uglg` is carried for
  a follow-up; the fixture pins `lagl`; the ceiling is stated.
- **The descriptor is untrusted input.** Missing keys default, wrong types and
  non-finite numbers return `None`, and the descriptor reader is depth-capped;
  the decoder never panics.
- **`layer_effects.rs` and its test file grow.** Mitigation: `layer_effects.rs`
  stays under the 1200 cap; the inner-shadow tests go in a new
  `tests/layer_effects/inner_shadow.rs` submodule so no test file approaches 1400.

## Open Questions

- Whether Photoshop CS6 writes `layerConceals` for an `IrSh`; libpsd and
  psd-tools model none, but a CS6 capture would settle it.
- The exact choke-to-erode mapping and the offset sign convention need a CS6
  pixel baseline; the docs' behavioral-parity proposal gives a direction test but
  no reference render.
- Whether a first slice should read image resource 1037 (global light) instead of
  `lagl` when `uglg` is on.
- Whether `present` false effects should decode at all or be dropped like
  psd-tools does.
