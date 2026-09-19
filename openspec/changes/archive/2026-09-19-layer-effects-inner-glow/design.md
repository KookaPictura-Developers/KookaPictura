## Context

A Photoshop layer's **object-based** layer effects live in the `lfx2`
additional-layer-info tagged block (`Tag.OBJECT_BASED_EFFECTS_LAYER_INFO =
b"lfx2"`). Each effect is a key on the top-level descriptor whose value is an
`Objc` whose class id is the effect kind; `masterFXSwitch` is the document-wide
effect switch. `pictura-codec` preserves `lfx2` verbatim in `Layer.extra_blocks`.
The Drop Shadow slice (`layer-effects-drop-shadow`) added `Layer::extra_block`,
the CPU matte/dilate/blur/bbox pipeline, the GPU `UnsupportedLayerEffect`
predicate, `blend_parts` and the `drop_shadow.psd` fixture; Outer Glow
(`layer-effects-outer-glow`) added `OrGl`; Inner Shadow (`layer-effects-inner-shadow`)
added the below/above composite split, `erode_matte`, and `IrSh` as an interior
effect composited **above** the content. This change adds the fourth kind,
**Inner Glow** (`IrGl`), the interior counterpart of Outer Glow.

The pieces that already exist in `crates/pictura-render/src/layer_effects.rs`
(~803 LOC): `decode_outer_glow`, `decode_inner_shadow`, the shared `GlowTechnique`
(`GlowT`/`BETE`), `content_matte`, `fill_coverage_matte`, `dilate_matte`,
`erode_matte`, `blur_matte`, `clamp_finite`, `finite_f32`, `num_or`,
`num_clamped`, `bool_or`, `decode_color`, `clip_rect`, `pad_rect`, `rect_empty`,
and the compositor's `Canvas`, `blend_parts`, `channel`, `desc_item`,
`mask_alpha`. The compositor (`composite.rs`) already calls a below-content pass
(`composite_layer_effects`) and an above-content pass
(`composite_layer_effects_above`). `pictura-filters` provides the Gaussian blur.
psd-tools, libpsd and ag-psd are the references for the parameter keys and
meanings (verified below).

### Ground truth read

- `openspec/changes/archive/2026-09-19-layer-effects-inner-shadow/` (proposal,
  design, specs, tasks) is the template: `IrGl` is the same object-based shape,
  rendered above the content, reusing the padded matte/blur pipeline.
- `docs/05-layers/layer-styles.md` **Inner Glow** (~L157-160): "Same as Outer
  Glow, plus **Source** (`Edge` or `Center`); uses **`Choke`** instead of
  `Spread` (0-100 %). Default Source is Edge." Outer Glow (~L141-155): blend
  Screen, opacity 75, Technique Softer, Size 5. The glow algorithm (~L251-258):
  "Inner Glow = same on the interior matte, with `Source = Edge` (default) using
  the edge distance or `Source = Center` using distance from the content center."
- The shipped first-slice ceilings: `GlwT` `PrBL` (Precise) renders as `SfBL`
  (Softer); spread/choke map to a max/min filter of radius
  `round(value/100 · size)`; contour/noise/anti-alias ignored; the matte is a
  bounded padded `f32` buffer.

## Goals / Non-Goals

**Goals:**

- Decode the `lfx2` `IrGl` object into a typed `InnerGlow`, including the
  `glwS` **Source** enum, with Photoshop defaults and a no-op on absent or
  malformed input.
- Render the inner glow on the CPU as an interior matte confined to the layer's
  content, composited above the content, reusing the padded/padded-bbox
  pipeline, `erode_matte`, `blur_matte` and `blend_parts`.
- Keep the GPU path panic-free by rejecting an enabled inner glow before dispatch
  and falling back to the CPU composite.
- Split `layer_effects.rs` into `layer_effects/{mod,shadows,glows}.rs` by pure
  moves so the shared plumbing and the per-effect code each stay well under the
  1200 LOC code cap as further effect kinds land.
- Prove the decode and render against a psd-tools-authored `inner_glow.psd`
  fixture and hand-built descriptors.

**Non-Goals:**

- Any effect other than Inner Glow; Bevel & Emboss, Satin, the overlays and
  Stroke.
- Gradient glows, contour (`TrnS`), noise (`Nose`), the `Range` (`Inpr`) remap,
  anti-alias (`AntA`), the `Precise` technique, layer opacity scaling the effect,
  `Layer Mask Hides Effects`, and the isolated `Blend Interior Effects As Group`
  composite.
- A true centroid-distance `Center` implementation (a stated approximation).
- The legacy `lrFX` block, styles on groups, `Scale Effects`, presets, authoring
  UI, and a GPU shader.

## Decisions

### D1. `IrGl` is a fourth kind; split `layer_effects` by concern

No new crate, module trait, or dependency. The file is ~803 LOC today and the
remaining effect kinds (Bevel & Emboss, Satin, overlays, Stroke) all land in the
same module, so before adding a fourth effect the file is split by **pure moves**
(no behavior change) into:

```
crates/pictura-render/src/layer_effects/
  mod.rs      module docs, MAX_* constants, descriptor readers (num_or,
              finite_f32, num_clamped, bool_or, decode_color), GlowTechnique,
              geometry (clip_rect/pad_rect/rect_empty), matte/blur helpers
              (content_matte, dilate_matte, erode_matte, blur_matte,
              clamp_finite), is_destructive_adjustment, and the below/above
              entry points composite_layer_effects / composite_layer_effects_above
  shadows.rs  DropShadow, InnerShadow, decode_drop_shadow, decode_inner_shadow,
              composite_drop_shadow, composite_inner_shadow
  glows.rs    OuterGlow, InnerGlow, GlowSource, decode_outer_glow,
              decode_inner_glow, composite_outer_glow, composite_inner_glow
```

`mod.rs` re-exports the public types so `lib.rs`'s `pub use
layer_effects::{...}` is unchanged. Each resulting file is far under the cap and
the shared file never has to grow for a new effect kind.

### D2. `InnerGlow` shape

```rust
/// The inner-glow source stored in `glwS` (typeID `IGSr`).
pub enum GlowSource {
    /// `SrcE`: the glow starts at the content edge and fades inward.
    Edge,
    /// `SrcC`: the glow is strongest away from the content edge.
    Center,
}

pub struct InnerGlow {
    pub enabled: bool,
    pub present: bool,
    pub blend_mode: BlendMode,   // default Screen
    pub color: [u8; 3],          // default white (255, 255, 255)
    pub opacity: f32,            // percent, 0..=100
    pub choke: f32,              // percent, 0..=100 (`Ckmt`), an erode
    pub size: f32,               // pixels, 0..=250, Gaussian radius
    pub source: GlowSource,      // `glwS`, default Edge
    pub technique: GlowTechnique,// `GlwT`, default Softer; Precise renders Softer
}
```

`GlowTechnique` is reused unchanged from the Outer Glow slice.

### D3. Descriptor keys and defaults (grounded)

The `lfx2` payload is the same `DescriptorBlock2` as the other kinds: a `u32`
version, a `u32` data version (`16`), then the version-16 body.
`decode_inner_glow` takes `&data[4..]`, calls `pictura_codec::read_descriptor`,
requires the top-level object, and finds the `IrGl` `Objc` (class id `IrGl`).

| Parameter | PSD key | Type | Default | DrSh / OrGl drift |
|---|---|---|---|---|
| enabled | `enab` | `bool` | false | same |
| present | `present` | `bool` | false | same |
| shown in dialog | `showInDialog` | `bool` | false | same (ignored) |
| blend mode | `Md  ` | `enum` typeID `BlnM` | **Screen** (`scrn`) | same as OrGl |
| colour | `Clr ` | `RGBC` object, `Rd `/`Grn `/`Bl  ` `doub` 0..=255 | white | OrGl defaults pale yellow |
| opacity | `Opct` | unit float percent (or `doub`) | 75 | same |
| technique | `GlwT` | `enum` typeID `BETE` | Softer (`SfBL`) | same |
| choke | `Ckmt` | unit float percent (or `doub`) | 0 | OrGl reads the same key as Spread (dilate); IrGl reads it as Choke (erode) |
| size | `blur` | unit float pixels (or `doub`) | 5 | same |
| source | `glwS` | `enum` typeID **`IGSr`** | Edge (`SrcE`) | **OrGl has no source key** |
| contour | `TrnS` | descriptor | Linear | ignored |
| anti-alias | `AntA` | `bool` | true | ignored |
| noise | `Nose` | unit float percent | 0 | ignored |
| jitter | `ShdN` | unit float percent | 0 | gradient only, ignored |
| range | `Inpr` | unit float percent | 50 | ignored |

**The Source key is `glwS` with enum typeID `IGSr`, not `BETE`.** `BETE` is the
*technique* (`GlwT`) typeID. The canonical typeID is `IGSr` (capital S); the
legacy/mis-authored `IGsr` spelling is also accepted leniently, and any other
typeID is `None`. This is the one place the Outer Glow table does not carry over.

Numeric values are read from either `UnitFloat` or `Double`. A missing `Md  `
decodes to Screen, a missing colour to white, an unknown blend key falls back to
Screen, a missing `GlwT` decodes to Softer, an unknown technique value to Softer,
a missing `glwS` decodes to Edge, and an unknown source value to Edge (mirroring
the other kinds' unknown-enum fallbacks). Absent numeric keys take the defaults
above. A finite value outside its documented range is clamped (`opacity` and
`choke` `0..=100`, `size` `0..=250`); a non-finite value, or a finite `f64` that
overflows to infinity as an `f32`, rejects the effect. A missing `lfx2`, a
missing `IrGl`, an unknown data version, a wrong-typed numeric, a wrong `Md  `
typeID, a wrong `GlwT` typeID or a `glwS` typeID that is neither `IGSr` nor the
legacy `IGsr`, a non-`RGBC` `Clr `, or a descriptor that
fails to parse is `None`; the decoder never panics. An `IrGl` whose `present` or
`enab` is false decodes but renders nothing.

**Evidence.**

- psd-tools `psd/terminology.py`: `Key.InnerGlowSource = b"glwS"` (L1490) and
  `Type.InnerGlowSource = b"IGSr"` (L2005), `Enum.EdgeGlow = b"SrcE"` (L389),
  `Enum.CenterGlow = b"SrcC"` (L336), `Key.GlowTechnique = b"GlwT"` (L1423),
  `Enum.MatteTechnique = b"BETE"` (L2020), `Key.ChokeMatte = b"Ckmt"` (L1203).
- psd-tools `api/effects.py::InnerGlow.glow_source` (L386-392) reads
  `self.descriptor.get(Key.InnerGlowSource)` and returns `source.enum` with a
  `SrcE` default; `_GlowEffect.glow_type` returns `SfBL` by default;
  `_ChokeNoiseMixin.choke` reads `Ckmt`.
- ag-psd (the vendored bundle read for reference) defines
  `IGSr = createEnum('IGSr', 'edge', { edge: 'SrcE', center: 'SrcC' })` and
  serializes `'glwS' : e.IGSr.encode(s)` / deserializes
  `r.source = e.IGSr.decode(a)`; `GlwT` uses `BETE` with `SfBL`/`PrBL`.
- libpsd `src/inner_glow.c::psd_get_layer_inner_glow2` case `'glwS'` asserts the
  enum typeID is `'IGSr'` and maps `'SrcC'` -> center, `'SrcE'` -> edge; its
  default is `psd_glow_edge`, blend `screen`, opacity `191` (=75 %), colour
  `0xFFFFFFBE`, choke 0, size 5, technique `softer`.
- Live psd-tools round-trip probe (authored `DescriptorBlock2` with an `IrGl`
  object, saved and reopened): psd-tools reads it as
  `psd_tools.api.effects.InnerGlow` with `enabled`/`present` true,
  `blend_mode` `scrn`, `opacity` 75, `choke` 0, `size` 5, `glow_type` `SfBL`,
  colour (255,255,255); with `glwS` `IGSr`/`SrcE` it reports `glow_source`
  `SrcE`, with `SrcC` it reports `SrcC`, and with `glwS` absent it defaults to
  `SrcE`.

The white default follows this change's stated Inner Glow default. The docs
defer to Outer Glow ("same as Outer Glow"), whose table says pale yellow
`#ffffbe`; libpsd likewise defaults to `0xFFFFFFBE`. Real files always carry
`Clr `, so the default is observable only for a hand-built minimal descriptor;
it is recorded here as a divergence, not a parity claim.

### D4. Interior field: erode the content matte, blur, then confine

Let `M` be the masked content coverage (D4 of the drop-shadow design: pixel `-1`
alpha, `SoCo`/`GdFl`/`PtFl` fill alpha, or `1.0` inside the rect of a
channel-less smart source, multiplied by `mask_alpha/255`). Let `ch` be the choke
radius, `ch = round(choke/100 · size)` (clamped again in the composite helper so
a hand-built `InnerGlow` cannot panic). Build

```
anchor = erode(M, ch)            // min filter: choke shrinks the content matte
B      = blur(anchor, size)      // Gaussian of radius `size`
field  = match source {
    Edge   => 1 - B,             // high outside/near the edge, 0 deep inside
    Center => B,                 // high deep inside, ~0.5 at the edge
}
glow(x, y) = M(x, y) · field(x, y) · opacity/100
```

- **Edge** (`1 - B`): with `choke = 0`, `anchor = M`, `B = blur(M)`, so
  `field = 1 - blur(M)` is ~0 deep inside and ~0.5 at the edge; `M · field`
  leaves a soft band just inside the edge that fades inward. A larger `choke`
  erodes `M` first, moving the falloff inward, so the band becomes a solid
  plateau hugging the edge followed by the `size` falloff — matching the
  observable control ("Choke: how much of the glow area is rendered at full
  opacity versus faded"; at `choke = size` it is a hard interior stroke).
- **Center** (`B`): `M · blur(erode(M, ch))` is high in the interior and falls to
  ~0.5 at the edge, so the glow is strongest at the center and recedes from the
  edges. `choke` pulls the falloff off the edge.
- `M` confines both to the content interior; the pixels the content does not
  cover are untouched. There is **no offset** (unlike Inner Shadow): the effect
  is symmetric and the field itself provides the edge/center difference.
- With `size = 0` and `choke = 0`, `B = M`, so Edge is `M · (1 - M) = 0` (a
  no-op) and Center is `M · M = M` (a flat interior). `M · (1 - M)` is zero only
  for a binary matte, so the composite helper **short-circuits and returns**
  when `size == 0 && choke == 0 && source == Edge`; a partial-alpha matte then
  stays byte-identical to no effect rather than picking up an unintended tint.
  Center keeps the flat-interior behavior. Both are consistent with "no blur, no
  choke" and are exercised by the render tests.

**Grounding for the formula.** libpsd's
`psd_layer_effects_blend_inner_glow` starts from the layer alpha `M`, reverses it
to `1 - M`, applies the layer mask, copies the reversed mask as the knock-out,
applies the choke (a Gaussian + edge find over `choke_size = (choke·size+50)/100`,
leaving `blur_size = size - choke_size` for the second Gaussian), adjusts the
range, and — **only for `psd_glow_center` — reverses the alpha again** before the
contour, noise and `psd_bitmap_knock_out` (which multiplies by the interior
`M`). So Edge keeps the processed `1 - M` and Center keeps its complement, then
both are confined to the interior. That is exactly `field = 1 - B` for Edge and
`field = B` for Center with `B` the processed `anchor`, and `blur(1 - anchor)`
equals `1 - blur(anchor)` for a normalized blur. The `anchor = erode(M, ch)`
min-filter is this slice's stand-in for libpsd's blur-then-edge-find choke (a
stated ceiling, as in the Inner Shadow slice), and the GIMP inner-glow study
confirms the standard "Softer" technique is blur-based and that choke is
piecewise (fully solid above 50 %); this slice's linear erode is the marked
approximation.

### D5. Compositing above the content

Outer Glow composites from `composite_layer_effects` **before** the content; an
inner glow must composite **after** it, like Inner Shadow, or opaque content
hides it. The existing above-content pass in `composite.rs`:

```
crate::layer_effects::composite_layer_effects_above(canvas, layer, doc);
```

gains a second check in `mod.rs`: decode `IrGl`, and when `enabled && present`
call `glows::composite_inner_glow`. The shared group / destructive-adjustment
skip is unchanged; the composite helper reuses `blend_parts` with the effect's
own `blend_mode` and its matte alpha weighted by `opacity/100`; the layer's
opacity, fill and mask are not applied a second time. The glow is composited
over `source = layer.rect ∩ canvas` only, so pixels outside the content rect are
byte-identical to the no-effect composite.

ponytail: compositing the interior effect after the layer's content means it
blends against the layer-over-backdrop result rather than an isolated content
buffer. `Blend Interior Effects As Group` and the exact Photoshop shade order are
deferred; the observable contract (interior-only, exterior byte-identical,
tinted by colour/opacity/blend, Source changes the field) holds. When both
interior effects are present, Inner Shadow is composited first, then Inner Glow;
Photoshop's exact inter-effect order is not modelled.

### D6. Bounding and early-out

Content exists only inside `source`, so `anchor` is non-zero only within `ch` of
`source` and `B` extends at most `blur_support = ceil(3·sigma_from_radius(size))`
further. The matte is built over `padded = pad_rect(source, ch + blur_support, w,
h)`, exactly the Outer Glow padding, and the composite runs over `source`. The
`anchor` buffer is zero outside `padded` (no content), so the crate blur's
zero-padding is exact. Early-out on an empty `source`, an empty `padded`, or an
all-zero `M`; clamp the numerics again so a hand-built `InnerGlow` cannot panic.
The existing `O(canvas · size)` naive-blur ceiling is inherited unchanged.

### D7. Shared plumbing, not duplicated math

`blend_parts`, `channel`, `desc_item`, `mask_alpha`, `content_matte`, `clip_rect`,
`pad_rect`, `rect_empty`, `erode_matte`, `blur_matte`, `clamp_finite`,
`finite_f32`, `num_or`, `num_clamped`, `bool_or` and `decode_color` are reused
unchanged (moved, not rewritten, by D1). `GlowSource` is the only new type beyond
`InnerGlow`; `GlowTechnique` is shared with Outer Glow. No trait, factory or
generic pipeline is added.

### D8. GPU fallback

`check_supported`'s `walk` currently rejects an enabled, present `DropShadow`,
`OuterGlow` and `InnerShadow`. It gains the same test for `decode_inner_glow`,
returning the existing `GpuError::UnsupportedLayerEffect` before any dispatch,
ahead of the adjustment check. `composite_active` / `composite_gpu_or_cpu`
already treat every `GpuError` as a CPU fallback, so no call-site change is
needed.

### D9. Fixture and oracle

`scripts/generate-fixtures.py` gains `inner_glow()`: a `Base` pixel layer plus a
`Glow` pixel layer whose record carries a `DescriptorBlock2` with
`masterFXSwitch: Bool(True)` and an `IrGl` object (`Descriptor` with
`classID=b"IrGl"`) under `Tag.OBJECT_BASED_EFFECTS_LAYER_INFO`. The authored keys
are `enab`, `present`, `showInDialog`, `Md  ` (`BlnM`/`scrn`), `Clr ` RGBC
(255,255,255), `Opct` 75, `GlwT` (`BETE`/`SfBL`), `Ckmt` 0, `blur` 5,
`glwS` (`IGSr`/`SrcE`), `Nose` 0, `ShdN` 0, `Inpr` 50, `AntA` true, `TrnS`
Linear. The new `inner_glow.psd` is registered in `FIXTURES` and regenerated;
existing fixtures must stay byte-identical.

The codec oracle mirrors the predecessor ones: the `lfx2` block is present in
`extra_blocks` with version 1 / data version 16, the whole `Document` round-trips
`write_psd`/`read_psd` with `lfx2` preserved, and a self-skipping psd-tools check
reads the layer's effect as `InnerGlow` asserting `enabled`, `present`,
`opacity`, `blend_mode`, `choke`, `size`, `glow_type` and `glow_source`.
`decode_inner_glow` then decodes the same bytes and the composite differs from
the no-effect composite.

### D10. Deferred fidelity, marked as ceilings

`ponytail:` ceilings record: `Source = Center` lights the edge-distant interior
rather than a bounded centroid-radius blob; `Precise` renders as `Softer`;
`choke` maps to a min-filter erode of radius `round(choke/100 · size)` rather
than libpsd's blur-then-edge-find (and the documented 50 % piecewise behavior is
not modelled); the white colour default diverges from the docs' pale yellow;
gradient glows, contour, noise, range, jitter and anti-alias are ignored; the
interior effect composites against the layer-over-backdrop result rather than an
isolated content buffer; layer opacity does not scale the effect; the layer mask
is folded into the matte rather than gated by `Layer Mask Hides Effects`; the
inter-effect order (Inner Shadow then Inner Glow) is not modelled; groups,
adjustment layers and smart filters carry no effect; the matte is built over a
bounded padded `f32` buffer (the existing compositor ceiling).

### D11. No app change

The app composites through `pictura_render::composite_rgba` /
`composite_active`; a decoded effect renders there with no command, panel, or
`CMakeLists.txt` change. No authoring UI and no C++ self-test check are added.
`docs/dev/STATE.md` is the only doc that would name the change, and it is updated
separately under `TASK-ALLOWS-DOCS`.

## Risks / Trade-offs

- **`Center` is an approximation.** It is the libpsd complement-of-edge field,
  not a centroid distance transform. Mitigation: the docs say "distance from the
  content center"; this slice lights the interior far from any edge, the
  approximation is named, and the observable contract (Source changes the field;
  center- and edge-source composites differ) is tested.
- **The Source typeID is easy to get wrong.** The change prompt suggested `BETE`;
  the references agree it is `IGSr`. Mitigation: D3 records the exact psd-tools
  `Key`/`Enum` integers, the ag-psd enum, the libpsd assertion, and a live
  psd-tools probe; a decode test asserts a wrong `glwS` typeID is `None`.
- **The white default diverges from the docs.** Mitigation: real files carry
  `Clr `, the divergence is named, and the fixture authors an explicit colour.
- **`layer_effects.rs` is at the cap.** Mitigation: D1 splits it by pure moves
  into three files, each far under 1200 LOC.
- **The descriptor is untrusted input.** Missing keys default, wrong types and
  non-finite numbers return `None`, and the descriptor reader is depth-capped;
  the decoder never panics.

## Open Questions

- Whether Photoshop CS6 writes a `glwS` `IGSr` value outside `SrcE`/`SrcC`; the
  decoder falls back to Edge and a CS6 capture would settle it.
- The exact choke-to-field mapping and the Center falloff need a CS6 pixel
  baseline; the docs' behavioral-parity proposal gives direction tests but no
  reference render.
- Whether a later slice should implement the documented choke piecewise behavior
  and a true centroid distance transform for Center.
