## Context

A Photoshop layer's **object-based** layer effects live in the `lfx2`
additional-layer-info tagged block (`Tag.OBJECT_BASED_EFFECTS_LAYER_INFO =
b"lfx2"`). Each effect is a key on the top-level descriptor whose value is an
`Objc` whose class id is the effect kind; `masterFXSwitch` is the document-wide
effect switch. `pictura-codec` preserves `lfx2` verbatim in `Layer.extra_blocks`.

Eight kinds ship (`DrSh`, `OrGl`, `IrSh`, `IrGl`, `FrFX`, the
`SoFi`/`GrFl`/`patternFill` overlays, and `ChFX` satin) with a shared
matte/decode pipeline and two composite passes (`composite_layer_effects` below
the content, `composite_layer_effects_above` after it). This change adds the
ninth, **Bevel & Emboss** (`ebbl`), as the first slice: the **Inner, Smooth**
technique with a highlight and a shadow. The module becomes a sixth file,
`layer_effects/bevel.rs` (pure moves; no behaviour change to the shipped kinds).

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
  for an above-content, interior-only composite (`decode_inner_shadow` /
  `composite_inner_shadow`: the padded-region matte, blur and `M`-confined loop)
  and the blend-mode decode pattern.
- `crates/pictura-render/src/layer_effects/satin.rs` — the structural template
  for an above-content, `M`-confined surface effect and the same-file
  `composite_*_for_test` helper.
- `crates/pictura-render/src/gpu/mod.rs` — `check_supported`'s `walk` predicate
  (lines 273-342), which already rejects the shipped kinds.

### Ground truth read

- The preceding changes under
  `openspec/changes/archive/2026-09-19-layer-effects-*/` and
  `2026-09-20-layer-effects-{stroke,overlays,satin}/`, plus the shipped
  `crates/pictura-render/src/layer_effects/{mod,shadows,glows,strokes,overlays,satin}.rs`.
- `docs/05-layers/layer-styles.md`:
  - The **Bevel & Emboss** row (L35): "Adds combinations of highlight and
    shadow, with optional separate edge Contour and Texture; styles are
    Inner/Outer/Pillow/Stroke Bevel and Emboss."
  - The parameter table (L164-180): `Style` Inner Bevel, `Technique` Smooth,
    `Depth` 100 % `(inferred)` 0–1000 `(inferred)`, `Direction` Up, `Size` 5 px
    0–250, `Soften` 0 px 0–~250 `(inferred)`, `Angle` 120 `(inferred)`,
    `Altitude` 30 `(inferred)` 0–90, `Use Global Light` on, `Gloss Contour`
    Linear, `Anti-alias` on, `Highlight Mode/Color/Opacity` Screen/white/75 %
    `(inferred)`, `Shadow Mode/Color/Opacity` Multiply/black/75 % `(inferred)`,
    the Contour sub-panel and the Texture sub-panel.
  - The algorithm (L260-277): "Height field. Compute a signed distance / matte
    transition across the layer edge over width `Size`. `Technique` selects
    smoothing (Smooth) or distance measurement (Chisel Hard/Soft) … **Direction**
    `Up` raises the surface, `Down` inverts it. **Normal & light.** Derive a
    surface normal from the height gradient; build the light vector from
    `Angle`/`Altitude`; Lambertian dot product yields a signed shading value.
    **Highlight/Shadow.** Positive shading → highlight …; negative → shadow ….
    **Gloss Contour** remaps the shading (applied *after* shading); … `Soften`
    blurs the final shading. `Style` … selects which side of the matte the bevel
    occupies."
  - The application order (L309-316): Drop Shadow beneath the layer; the surface
    effects (Bevel, Satin, Overlays, Stroke) on/within it.
- **The effect class id and top-level key are `ebbl`**:
  - libpsd `src/effects.c` `case 'ebbl':` calls `psd_get_layer_bevel_emboss2`;
    `src/bevel_emboss.c` `psd_get_layer_bevel_emboss2` parses the object and
    `psd_set_layer_bevel_emboss_default` sets its defaults. (Fetched from
    `https://github.com/TheNicker/libpsd` `src/bevel_emboss.c`, 692 lines.)
  - psd-tools `psd_tools/api/effects.py:439`
    `@register(Klass.BevelEmboss.value)` `class BevelEmboss`; `terminology.py`
    `Klass.BevelEmboss = b"ebbl"`. `Effects.__init__` looks up each effect by
    `item.classID`.
  - Live round-trip probe (this session): a `DescriptorBlock2` authored with
    `masterFXSwitch` plus an `ebbl` `Objc` saves and reopens; psd-tools reports
    a `BevelEmboss` whose keys are exactly `[enab, present, showInDialog, hglM,
    hglC, hglO, sdwM, sdwC, sdwO, bvlS, bvlT, bvlD, uglg, lagl, Lald, srgR,
    blur, Sftn, TrnS, MpgS, AntA, Inpr, useShape, useTexture, antialiasGloss]`,
    decoding to `enabled=True present=True highlight=Scrn/(250,240,230)/80
    shadow=Mltp/(10,20,30)/70 style=InrB technique=SfBL direction=In  angle=120
    altitude=30 depth=250 size=7 soften=3 use_global_light=False`.

- **Exact keys, typeIDs and defaults** (libpsd `src/bevel_emboss.c` parser +
  `psd_set_layer_bevel_emboss_default` + ag-psd's `bvlT`/`BESs` enum tables +
  psd-tools `BevelEmboss` accessors + the probe):

  | Struct field | Key | TypeID | Stored unit | Default (PS UI / docs) | psd-tools accessor |
  |---|---|---|---|---|---|
  | `enabled` | `enab` | `bool` | — | false | `_Effect.enabled` |
  | `present` | `present` | `bool` | — | false | `_Effect.present` |
  | highlight mode | `hglM` | enum `BlnM` | — | Screen | `BevelEmboss.highlight_mode` |
  | highlight color | `hglC` | `RGBC` obj | 0–255 | white | `BevelEmboss.highlight_color` |
  | highlight opacity | `hglO` | `UntF` | `#Prc` | 75 | `BevelEmboss.highlight_opacity` |
  | shadow mode | `sdwM` | enum `BlnM` | — | Multiply | `BevelEmboss.shadow_mode` |
  | shadow color | `sdwC` | `RGBC` obj | 0–255 | black | `BevelEmboss.shadow_color` |
  | shadow opacity | `sdwO` | `UntF` | `#Prc` | 75 | `BevelEmboss.shadow_opacity` |
  | `style` | `bvlS` | enum `BESl` | — | Inner Bevel | `BevelEmboss.bevel_style` |
  | `technique` | `bvlT` | enum `bvlT` | — | Smooth | `BevelEmboss.bevel_type` |
  | `direction` | `bvlD` | enum `BESs` | — | Up | `BevelEmboss.direction` |
  | `use_global_angle` | `uglg` | `bool` | — | true | `_AngleMixin.use_global_light` |
  | `angle_deg` | `lagl` | `UntF` | `#Ang` | 120 | `_AngleMixin.angle` |
  | `altitude_deg` | `Lald` | `UntF` | `#Ang` | 30 | `BevelEmboss.altitude` |
  | `depth` | `srgR` | `UntF` | `#Prc` | 100 | `BevelEmboss.depth` |
  | `size` | `blur` | `UntF` | `#Pxl` | 5 | `BevelEmboss.size` |
  | `soften` | `Sftn` | `UntF` | `#Pxl` | 0 | `BevelEmboss.soften` |
  | gloss contour (ignored) | `TrnS` | `Objc` | — | Linear | `BevelEmboss.contour` |
  | edge contour (ignored) | `MpgS` | `Objc` | — | Linear | — |
  | anti-alias (ignored) | `AntA`, `antialiasGloss` | `bool` | — | on | `BevelEmboss.anti_aliased` |
  | contour range (ignored) | `Inpr` | `UntF` | `#Prc` | 50 | — |
  | texture (ignored) | `useTexture`, `InvT`, `Algn`, `Scl `, `Ptrn` | bool/`UntF`/`Objc` | — | off | `BevelEmboss.use_texture` |
  | shape (ignored) | `useShape` | `bool` | — | false | `BevelEmboss.use_shape` |

- **Enum values** (libpsd `psd_get_layer_bevel_emboss2` + ag-psd's
  `bvlS`/`bvlT`/`BESs` tables + the probe):
  - `bvlS` (typeID `BESl`): `InrB` Inner, `OtrB` Outer, `Embs` Emboss, `PlEb`
    Pillow, classID `strokeEmboss` Stroke.
  - `bvlT` (typeID `bvlT`): `SfBL` Smooth, `PrBL` Chisel Hard, `Slmt` Chisel
    Soft.
  - `bvlD` (typeID `BESs`): `In  ` Up, `Out ` Down.

  Three divergences from the task brief are recorded rather than copied:
  highlight/shadow are **`hglM`/`hglC`/`hglO`** and **`sdwM`/`sdwC`/`sdwO`**
  (not `hglm`/`sglm`/`sglC`/`sglO`); Outer is **`OtrB`** (not `OutB`); the
  direction values are **`In  `/`Out `** (not `DrUp`/`DrDn`); the technique has
  **three** values including `Slmt` (Chisel Soft); and depth is `srgR` (not
  `Scl `, which is the texture scale; `glwT`/`srgC` are not bevel keys).

- **The render algorithm** (docs L260-277; libpsd `src/bevel_emboss.c`
  `psd_layer_effects_blend_bevel_emboss`): libpsd's shipped blend only handles
  the Outer style and offsets a blurred inflated matte by the light vector
  `distance_x = -(size·cos(angle)·cos(altitude))`,
  `distance_y = (size·sin(angle)·cos(altitude))` (the same sign convention as the
  shipped shadows); the Inner/Emboss/Pillow branches are empty, so libpsd is a
  reference for the light vector and the highlight/shadow blend, not for the
  inner height field. The docs' height-field → normal → Lambertian model is the
  behavioural contract; its closed internals (distance metric, normal smoothing,
  `Depth` scaling) are a behavioral-parity ceiling.

## Goals / Non-Goals

**Goals:**

- Decode the `lfx2` `ebbl` object into a typed `BevelEmboss`, reusing the shipped
  descriptor readers, with Photoshop UI defaults and a no-op on absent or
  malformed input.
- Render the Inner/Smooth slice on the CPU as a lit interior bevel above the
  content, confined to the masked content matte `M`.
- Keep the GPU path panic-free by rejecting an enabled and present bevel before
  dispatch and falling back to the CPU composite.
- Prove the decode and render against a psd-tools-authored fixture and hand-built
  descriptors.

**Non-Goals:**

- The chisel techniques, the Outer/Emboss/Pillow/Stroke styles, contour, gloss
  contour, texture, anti-alias, the global-light resource, `useShape` and
  `showInDialog`.
- libpsd's knock-out and mask passes, a GPU shader and any authoring UI.
- Exact Adobe numeric parity for the height profile, depth scaling or the
  highlight/shadow blend.

## Decisions

### D1. One new file, no new abstractions

No new crate, module trait, or dependency. `mod.rs` already holds the shared
plumbing and gains only `mod bevel;`, the re-exports, one call, two caps and a
doc paragraph. The new file:

```
crates/pictura-render/src/layer_effects/bevel.rs
  BevelStyle / BevelTechnique / BevelDirection / BevelHighlight / BevelShadow
  BevelEmboss
  decode_bevel_emboss
  composite_bevel_emboss
```

`mod.rs` gains `mod bevel;` and
`pub use bevel::{decode_bevel_emboss, BevelDirection, BevelEmboss, BevelStyle, BevelTechnique};`;
`lib.rs`'s `pub use layer_effects::{...}` list gains the same items. The file is
well under the 1200 LOC cap.

### D2. Typed params

```rust
pub enum BevelStyle { Inner, Outer, Emboss, Pillow, Stroke }
pub enum BevelTechnique { Smooth, ChiselHard, ChiselSoft }
pub enum BevelDirection { Up, Down }

pub struct BevelHighlight { pub mode: BlendMode, pub color: [u8; 3], pub opacity: f32 }
pub struct BevelShadow   { pub mode: BlendMode, pub color: [u8; 3], pub opacity: f32 }

pub struct BevelEmboss {
    pub enabled: bool,             // enab
    pub present: bool,             // present
    pub style: BevelStyle,         // bvlS, default Inner
    pub technique: BevelTechnique, // bvlT, default Smooth
    pub direction: BevelDirection, // bvlD, default Up
    pub depth: f32,                // srgR, percent, 0..=1000, default 100
    pub size: f32,                 // blur, px, 0..=250, default 5
    pub soften: f32,               // Sftn, px, 0..=250, default 0
    pub angle_deg: f32,            // lagl, default 120
    pub altitude_deg: f32,         // Lald, degrees, 0..=90, default 30
    pub use_global_angle: bool,    // uglg, default true (decoded, inert)
    pub highlight: BevelHighlight, // hglM/hglC/hglO, Screen/white/75
    pub shadow: BevelShadow,       // sdwM/sdwC/sdwO, Multiply/black/75
}
```

`size`, `soften`, `depth`, `opacity` and `altitude_deg` are clamped; `angle_deg`
is left unclamped, matching the shipped angle handling.

### D3. Descriptor keys and decode contract (grounded)

`decode_bevel_emboss` takes `layer.extra_block(b"lfx2")`, requires at least `8`
bytes, hands `&data[4..]` to `pictura_codec::read_descriptor`, requires the
top-level object, and finds the `ebbl` `Objc` whose class id is `ebbl`. It reads:

- `enab`/`present` via `bool_or` (default false);
- `bvlS` (`BESl`): absent or unknown value → `Inner`; `InrB`/`OtrB`/`Embs`/
  `PlEb` map to their variants and `strokeEmboss` → `Stroke`; a wrong typeID
  rejects;
- `bvlT` (`bvlT`): absent or unknown value → `Smooth`; `SfBL`/`PrBL`/`Slmt` map
  to `Smooth`/`ChiselHard`/`ChiselSoft`; a wrong typeID rejects;
- `bvlD` (`BESs`): absent or unknown value → `Up`; `In  `/`Out ` map to
  `Up`/`Down`; a wrong typeID rejects;
- `srgR` via `num_clamped(.., 100.0, 0.0, MAX_DEPTH)`;
- `blur` via `num_clamped(.., 5.0, 0.0, MAX_SIZE)`; `Sftn` via
  `num_clamped(.., 0.0, 0.0, MAX_SIZE)`;
- `lagl` via `num_or(.., 120.0)`; `Lald` via
  `num_clamped(.., 30.0, 0.0, MAX_ALTITUDE)`;
- `uglg` via `bool_or(.., true)`;
- `hglM`/`sdwM` (`BlnM`, default Screen/Multiply, unknown value → default, wrong
  typeID → reject), `hglC`/`sdwC` (`RGBC`, default white/black, non-`RGBC` →
  reject) via `decode_color`, and `hglO`/`sdwO` via
  `num_clamped(.., 75.0, 0.0, MAX_OPACITY)`;
- `TrnS`, `MpgS`, `AntA`, `antialiasGloss`, `Inpr`, `useShape`, `useTexture`
  and `showInDialog` are ignored.

The new caps `MAX_DEPTH = 1000.0` and `MAX_ALTITUDE = 90.0` join the existing
`MAX_*` constants in `mod.rs`. Numerics are accepted as `UnitFloat` or `Double`;
a value that is non-finite, or finite as `f64` but overflows to infinity as
`f32`, rejects. A missing `lfx2`, a missing `ebbl`, an unknown data version, a
wrong class id, a wrong-typed numeric, a wrong enum typeID, a non-`RGBC` colour,
or a descriptor that fails to parse is `None`; the decoder never panics. An
effect whose `present` or `enab` is false decodes but renders nothing.

### D4. The bevel formula

Let `M` be the masked content coverage over the padded region (pixel `-1` alpha,
fill payload alpha, or `1.0` inside the rect of a channel-less smart source,
multiplied by `mask_alpha/255`, exactly as the shipped effects). With the
clamped `size`, `soften`, `depth`, the finite `angle` and screen coordinates
(y down):

```
H(x, y)   = blur(M, size)[x, y]                         // height field, 0 outside, 1 inside
gx        = (H(x+1, y) - H(x-1, y)) / 2
gy        = (H(x, y+1) - H(x, y-1)) / 2
scale     = max(size, 1) · depth/100
N         = normalize( -gx·scale, -gy·scale, 1 )
L         = ( cos(alt)·cos(angle), -cos(alt)·sin(angle), sin(alt) )
shading   = dot(N, L) - sin(alt)                        // 0 on the flat interior
shading  *= (direction == Down ? -1 : 1)
if soften > 0: shading = blur( (shading+1)/2 mapped to 0..=1, soften ) mapped back
hi(x, y)  = M(x, y) · max( shading, 0) · highlight.opacity/100
sh(x, y)  = M(x, y) · max(-shading, 0) · shadow.opacity/100
```

`sh` is composited above the content with `shadow.color`/`shadow.mode`, then
`hi` with `highlight.color`/`highlight.mode` (shadow first, highlight last).

- The light vector uses the shipped sign convention
  (`dx = -distance·cos`, `dy = +distance·sin`), so `Angle` 120 with
  `Altitude` 30 puts the highlight on the upper-left interior edge and the shadow
  on the lower-right, matching the shipped drop/inner shadow offset and the
  Photoshop global-light default. `Direction = Down` negates the shading and
  swaps highlight and shadow.
- `scale` multiplies the gradient by `size` so the shading contrast is
  approximately independent of the bevel width; `depth` then scales it. This is a
  stated model choice, not a grounded Adobe constant.
- The build region is `padded = pad_rect(source, blur_support(size) +
  blur_support(soften) + 1)`, where `source = clip_rect(layer)` and
  `blur_support = ceil(3·sigma_from_radius(r))`; samples outside the built region
  are the exterior value `0`. libpsd pads by `size * 2` for the effect data.
- Clamp `size`/`soften`/`depth`/`opacity`/`altitude` again in the composite (a
  hand-built struct must not panic); a non-finite `angle` is treated as 0.

ponytail: the blur is the crate's naive separable Gaussian, so a canvas-filling
layer at maximum `size` costs O(canvas · size); the common small-layer and
crafted off-canvas cases are bounded.

### D5. Slice scope: only Inner + Smooth renders

The docs' `Style` chooses which side of the matte the bevel occupies and the
`Technique` selects the distance profile; both are selection semantics, not
tuning. This slice composites only `BevelStyle::Inner` with
`BevelTechnique::Smooth` (the Photoshop defaults). Any other `style` or
`technique` decodes into the typed struct but **renders nothing**, recorded as a
`ponytail:` ceiling; a later slice adds the outer/pillow profiles and the chisel
distance transforms without changing the decoder. `Direction`, `Depth`, `Size`,
`Soften`, `Angle`, `Altitude` and the highlight/shadow shading are all honoured
within that slice.

### D6. Compositing above the content, order

Bevel is a surface effect, so it composites **after** the layer's content.
`composite_layer_effects_above` gains the bevel check after Inner Glow and
**before** Satin:

```
inner shadow -> inner glow -> bevel & emboss -> satin -> color overlay -> gradient overlay -> pattern overlay -> stroke
```

matching the CS6 effects-list order (Bevel, Satin, Overlays, Stroke). The shared
group / destructive-adjustment skip is unchanged. ponytail: Photoshop's exact
inter-effect order among the above-content effects and the relative order of the
bevel highlight and shadow are not modelled.

### D7. Bounding and early-out

Bevel is confined to `source = clip_rect(layer, w, h)`; the matte, height field
and shading are built over `padded` (above). Early-out on an empty `source`/`padded`,
an all-zero `M`, or both `highlight.opacity` and `shadow.opacity` zero. Every
pixel outside `source` is byte-identical to the same document without the effect.
A non-`Inner` style or non-`Smooth` technique is a no-op. `Depth` 0 yields zero
shading and is a no-op.

### D8. GPU fallback

`check_supported`'s `walk` (`crates/pictura-render/src/gpu/mod.rs:273-342`) gains
the same enabled-and-present test for `decode_bevel_emboss`, returning the
existing `GpuError::UnsupportedLayerEffect` before any dispatch, alongside the
existing effect checks and ahead of the adjustment check. `composite_active` /
`composite_gpu_or_cpu` already treat every `GpuError` as a CPU fallback, so no
call-site change is needed. A malformed bevel returns `None` and does not reject.

### D9. Fixture and oracle

`scripts/generate-fixtures.py` gains `bevel()`: a `Base` pixel layer plus an
`ebbl`-bearing pixel layer whose record carries a `DescriptorBlock2` with
`masterFXSwitch: Bool(True)` and `ebbl` under
`Tag.OBJECT_BASED_EFFECTS_LAYER_INFO`, exactly as the `satin()` builder does. The
authored `ebbl` uses non-default values that exercise every decoded key —
`hglM` `BlnM`/`Scrn`, a non-white `hglC` `RGBC`, `hglO` 80, `sdwM`
`BlnM`/`Mltp`, a non-black `sdwC` `RGBC`, `sdwO` 70, `bvlS` `BESl`/`InrB`,
`bvlT` `bvlT`/`SfBL`, `bvlD` `BESs`/`In  `, `uglg` false, `lagl` 120, `Lald` 30,
`srgR` 250, `blur` 7, `Sftn` 3, plus `TrnS`/`MpgS` contour objects, `AntA`,
`Inpr`, `useShape`, `useTexture` and `antialiasGloss` — so the oracle proves the
highlight/shadow and enum keys survive. The new `bevel.psd` is registered in
`FIXTURES` and regenerated; existing fixtures must stay byte-identical.

The codec oracle mirrors the predecessor ones: the `lfx2` block is present in
`extra_blocks` with version 1 / data version 16, the whole `Document` round-trips
`write_psd`/`read_psd` with `lfx2` preserved, and a self-skipping psd-tools check
reads the effect as a `BevelEmboss` asserting `enabled`, `present`,
`highlight_mode`/`highlight_opacity`/`highlight_color`,
`shadow_mode`/`shadow_opacity`/`shadow_color`, `bevel_style`, `bevel_type`,
`direction`, `altitude`, `depth`, `size` and `soften`. `decode_bevel_emboss`
then decodes the same bytes and the composite differs from the no-effect
composite.

### D10. Deferred fidelity, marked as ceilings

`ponytail:` ceilings record: only Inner + Smooth renders; the chisel techniques,
the Outer/Emboss/Pillow/Stroke styles, contour (`MpgS`), gloss contour (`TrnS`),
contour `Range` (`Inpr`), anti-alias (`AntA`/`antialiasGloss`), texture
(`useTexture`, `InvT`, `Algn`, `Scl `, `Ptrn`), `useShape` and `showInDialog` are
decoded/ignored; the effective angle/altitude is the stored `lagl`/`Lald`, not the
global-light resource 1037; the height profile is a Gaussian blur of `M` rather
than Adobe's distance transform; `scale = size · depth/100` and the
`dot(N,L) - sin(alt)` flat-offset are ungrounded model choices; the build region
pads by the blur supports only; `Scale Effects` is not applied; groups,
adjustment layers and smart filters carry no effect; the exact inter-effect order
and the highlight/shadow order are not modelled.

### D11. No app change

The app composites through `pictura_render::composite_rgba` / `composite_active`;
a bevel renders there with no command, panel, or `CMakeLists.txt` change. No
authoring UI and no C++ self-test check are added.

## Risks / Trade-offs

- **The brief named wrong key letters and enum values.** Real PSD uses
  `hglM`/`hglC`/`hglO` and `sdwM`/`sdwC`/`sdwO` (not `hglm`/`sglm`), `OtrB` (not
  `OutB`) and `In  `/`Out ` (not `DrUp`/`DrDn`); D-context and the probe record
  the two-oracle anchor. Wrong keys would silently drop the highlight and shadow.
- **The height/lighting model is a proposal, not Adobe's closed algorithm.** The
  docs state the model but no numeric baseline exists; it is marked a
  behavioral-parity ceiling and pinned by sign/ordering tests (highlight
  upper-left, shadow lower-right, direction flips, depth/size/soften monotonic)
  rather than by golden CS6 pixels.
- **Only Inner + Smooth renders.** A real file with another style/technique
  still renders without the bevel. This is the stated slice; the decoder already
  carries the information needed to extend it.
- **The descriptor is untrusted input.** Missing keys default, wrong types and
  non-finite numbers return `None`, and the descriptor reader is depth-capped;
  the decoder never panics.
- **The blur cost at maximum `size`.** The naive separable Gaussian is bounded by
  the padded content region, not the full canvas; recorded as a ceiling.

## Open Questions

- Photoshop's exact inner-bevel height profile (distance metric, normal
  smoothing, the precise `Depth` scaling) and the highlight/shadow blend order
  need a CS6 pixel baseline; the docs give a behavioural description but no
  reference render.
- Whether PS writes the stroke-emboss style as the classID `strokeEmboss` (as
  libpsd and ag-psd suggest) or another tag; the decoder maps `strokeEmboss`
  explicitly and treats other unknown style values as the default `Inner`.
- Whether `hglO`/`sdwO` are always percent (`#Prc`) or can be stored as a raw
  `doub`; the decoder accepts both `UnitFloat` and `Double`, matching the shipped
  opacity handling.
