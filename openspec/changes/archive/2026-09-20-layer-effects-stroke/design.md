## Context

A Photoshop layer's **object-based** layer effects live in the `lfx2`
additional-layer-info tagged block (`Tag.OBJECT_BASED_EFFECTS_LAYER_INFO =
b"lfx2"`). Each effect is a key on the top-level descriptor whose value is an
`Objc` whose class id is the effect kind; `masterFXSwitch` is the document-wide
effect switch. `pictura-codec` preserves `lfx2` verbatim in `Layer.extra_blocks`.

Four kinds already ship. The Drop Shadow slice added `Layer::extra_block`, the
CPU matte/dilate/erode/blur/bbox pipeline, the GPU `UnsupportedLayerEffect`
predicate, `blend_parts` and the `drop_shadow.psd` fixture. Outer Glow added
`OrGl`; Inner Shadow added the below/above composite split and `IrSh`; Inner Glow
added `IrGl`, `erode_matte` and the interior field. The module was split by pure
moves into `layer_effects/{mod,shadows,glows}.rs`. This change adds the fifth
kind, **Stroke** (`FrFX`), the first **surface** effect drawn as a band at the
content edge.

The pieces that already exist and are reused unchanged:
`crates/pictura-render/src/layer_effects/mod.rs` — `content_matte` (:191),
`clip_rect` (:238), `pad_rect` (:252), `rect_empty` (:247), `dilate_matte`
(:264), `erode_matte` (:301), `clamp_finite` (:180), the descriptor readers
`num_or`/`finite_f32`/`num_clamped`/`bool_or`/`decode_color` (:76-123), the
`MAX_SIZE = 250.0` constant (:62) and the two entry points
`composite_layer_effects` (:131) / `composite_layer_effects_above` (:151);
`crates/pictura-render/src/layer_effects/glows.rs` is the closest template for a
band built from `M`. The compositor (`composite.rs:75,108`) already calls the
below- and above-content passes. `pictura-filters` is not needed: a stroke has
no blur.

### Ground truth read

- The four preceding changes:
  `openspec/changes/archive/2026-09-19-layer-effects-{drop-shadow,outer-glow,inner-shadow,inner-glow}/`
  (proposal, design, specs, tasks) and the shipped
  `crates/pictura-render/src/layer_effects/{mod,shadows,glows}.rs`. The Inner
  Glow change is the structural template: a new `layer_effects/*.rs` file,
  decode + composite, an above-content call, the GPU predicate extension, a
  psd-tools fixture and a self-skipping oracle.
- `docs/05-layers/layer-styles.md` **Stroke** table (~L196-205): Size px, default
  **3 `(inferred)`**, range **1-250 (integer)**; Position enum default
  **Outside** (Outside, Inside, Center); Blend Mode default **Normal**; Opacity
  default **100**; Fill Type default **Color** (Color, Gradient, Pattern); Color
  default "foreground `(inferred)`". The algorithm (~L291-295): "**Stroke**
  computes a band at the coverage edge: dilate `M` outward (Outside), inward
  (Inside), or straddle it (Center) by `Size`, then fill with color/gradient/
  pattern. Layer-style strokes are built from the layer's *bitmap* matte, so they
  are always integer-width and have rounded corners, do not follow open paths,
  and do not feather." The application order (~L309-316): Drop Shadow beneath the
  layer, "the surface effects (Bevel, Satin, Overlays, Stroke) on/within it".
- psd-tools, the independent oracle:
  - `psd_tools/api/effects.py:411-439` — `@register(Klass.FrameFX.value) class
    Stroke(_Effect, _ColorMixin, _PatternMixin, _GradientMixin)`; `.position`
    reads `Key.Style` and returns its `.enum` defaulting to `b"OutF"`;
    `.fill_type` reads `Key.PaintType` defaulting to `b"SClr"`; `.size` reads
    `Key.SizeKey` defaulting 0.0; `.overprint` reads `b"overprint"`. `_Effect`
    (:158-173) defaults `opacity` to 100 and `blend_mode` to `Enum.Normal`.
  - `psd_tools/composite/effects.py:82-107` — `draw_stroke_effect` switches on
    `desc.get(Key.PaintType).enum` over `Enum.SolidColor` / `Enum.Pattern` /
    `Enum.GradientFill`, and on `desc.get(Key.Style).enum` over
    `Enum.OutsetFrame` / `Enum.InsetFrame` / `Enum.CenteredFrame`, reading
    `desc.get(Key.SizeKey)`. It builds an image-based edge band (scharr edges,
    max filter, subtract the shape for Outset / multiply for Inset) — the same
    "band at the coverage edge" shape this change specifies, with independent
    confirmation that strokes are matte-based, not vector-based.
  - `psd_tools/psd/terminology.py` exact bytes (verified via `python3`):
    `Klass.FrameFX = Key.FrameFX = b"FrFX"`, `Key.Style = b"Styl"`,
    `Type.FrameStyle = b"FStl"`, `Enum.OutsetFrame = b"OutF"`,
    `Enum.InsetFrame = b"InsF"`, `Enum.CenteredFrame = b"CtrF"`,
    `Key.PaintType = b"PntT"`, `Type.FrameFill = b"FrFl"`,
    `Enum.SolidColor = b"SClr"`, `Enum.GradientFill = b"GrFl"`,
    `Enum.Pattern = b"Ptrn"`, `Key.SizeKey = b"Sz  "`, `Key.Mode = b"Md  "`,
    `Key.Opacity = b"Opct"`, `Key.Color = b"Clr "`, `Key.Enabled = b"enab"`.
  - Live round-trip probe: a `DescriptorBlock2` authored with
    `masterFXSwitch` + an `FrFX` object (`Styl` `FStl`/`OutF`, `PntT`
    `FrFl`/`SClr`, `Sz  ` 3 px, `Opct` 100, `Md  ` `Nrml`, black `Clr `) saved
    through `PSDImage.save` and reopened, is read by psd-tools as a `Stroke`
    with `enabled`/`present` true, `opacity` 100, `blend_mode` `Nrml`, black
    colour, `position` `OutF`, `fill_type` `SClr`, `size` 3.0. This is the exact
    fixture shape.

## Goals / Non-Goals

**Goals:**

- Decode the `lfx2` `FrFX` object into a typed `Stroke`, including the `Styl`
  Position enum and the `PntT` fill type, with Photoshop defaults and a no-op on
  absent or malformed input.
- Render the solid-colour stroke on the CPU as a content-edge band composited
  **above** the layer content, reusing the existing matte/geometry helpers.
- Keep the GPU path panic-free by rejecting an enabled stroke before dispatch
  and falling back to the CPU composite.
- Prove the decode and render against a psd-tools-authored `stroke.psd` fixture
  and hand-built descriptors.

**Non-Goals:**

- Gradient (`GrFl`) and pattern (`Ptrn`) stroke fills.
- Contour (`TrnS`), anti-alias (`AntA`), `overprint`, `Scale Effects`, and the
  exact Photoshop inter-effect order among the above-content effects.
- Every other effect kind (Bevel & Emboss, Satin, the overlays); the legacy
  `lrFX` block; styles on groups; the isolated `Blend Interior Effects As Group`
  composite; a GPU shader; and any authoring UI.

## Decisions

### D1. `FrFX` is a fifth kind in its own file

No new crate, module trait, or dependency. `mod.rs` already holds the shared
plumbing and stays untouched except for `mod strokes;`, the re-exports and one
call. The new file:

```
crates/pictura-render/src/layer_effects/strokes.rs
  Stroke, StrokePosition, decode_stroke, composite_stroke
```

`mod.rs` gains `mod strokes;` and
`pub use strokes::{decode_stroke, Stroke, StrokePosition};`; `lib.rs`'s
`pub use layer_effects::{...}` list gains `decode_stroke, Stroke,
StrokePosition`. Each file stays far under the 1200 LOC cap.

### D2. `Stroke` shape

```rust
/// The stroke position stored in `Styl` (typeID `FStl`).
pub enum StrokePosition {
    /// `OutF`: the band is built outside the content edge.
    Outside,
    /// `InsF`: the band is built inside the content edge.
    Inside,
    /// `CtrF`: the band straddles the content edge.
    Center,
}

pub struct Stroke {
    pub enabled: bool,
    pub present: bool,
    pub blend_mode: BlendMode,   // default Normal
    pub color: [u8; 3],          // default black (0, 0, 0)
    pub opacity: f32,            // percent, 0..=100
    pub size: u32,               // pixels, integer 1..=250
    pub position: StrokePosition,// `Styl`, default Outside
}
```

`size` is an integer because layer-style strokes are integer-width
(`docs/05-layers/layer-styles.md` ~L293, Bjango); the decoded numeric is rounded
to the nearest integer and clamped to `1..=250`.

### D3. Descriptor keys and defaults (grounded)

The `lfx2` payload is the same `DescriptorBlock2` as the other kinds: a `u32`
version, a `u32` data version (`16`), then the version-16 body.
`decode_stroke` takes `&data[4..]`, calls `pictura_codec::read_descriptor`,
requires the top-level object, and finds the `FrFX` `Objc` (class id `FrFX`).

| Parameter | PSD key | Type | Default | Source |
|---|---|---|---|---|
| enabled | `enab` | `bool` | false | psd-tools `_Effect.enabled` |
| present | `present` | `bool` | false | psd-tools `_Effect.present` |
| shown in dialog | `showInDialog` | `bool` | false | psd-tools `_Effect.shown` (ignored) |
| blend mode | `Md  ` | `enum` typeID `BlnM` | **Normal** (`Nrml`) | `_Effect.blend_mode` default `Enum.Normal` |
| colour | `Clr ` | `RGBC` object, `Rd `/`Grn `/`Bl  ` `doub` 0..=255 | **black** | `_ColorMixin.color`; docs Stroke Color |
| opacity | `Opct` | unit float percent (or `doub`) | **100** | `_Effect.opacity` default 100; docs 100 |
| position | `Styl` | `enum` typeID **`FStl`** | **Outside** (`OutF`) | `Stroke.position` default `b"OutF"` |
| fill type | `PntT` | `enum` typeID **`FrFl`** | **solid** (`SClr`) | `Stroke.fill_type` default `b"SClr"` |
| size | `Sz  ` | unit float pixels (or `doub`) | **3** | docs default 3; `Stroke.size` reads `Key.SizeKey` |
| overprint | `overprint` | `bool` | false | `Stroke.overprint` (ignored) |
| anti-alias | `AntA` | `bool` | true | ignored |
| contour | `TrnS` | descriptor | Linear | ignored |
| gradient | `Grad`/`GrdT`, `Angl`, `Type`, `Rvrs`, `Algn`, `Scl ` | — | — | **deferred** |
| pattern | `Ptrn`, `Lnkd` | — | — | **deferred** |

**The position key is `Styl` with enum typeID `FStl`, and the fill-type key is
`PntT` with enum typeID `FrFl`.** `FrFX` is both the `Klass` of the effect
object and the `Key` of the `lfx2` descriptor entry; the object's class id is
`FrFX`. Numeric values are read from either `UnitFloat` or `Double`. A missing
`Md  ` decodes to Normal, a missing colour to black, an unknown blend key to
Normal, a missing `Styl` or an unknown position value to Outside, and a missing
`PntT` to solid. Absent numeric keys take opacity 100 and size 3. A finite value
outside its documented range is clamped (`opacity` `0..=100`; `size` rounded
then clamped `1..=250`); a non-finite value, or a finite `f64` that overflows to
infinity as an `f32`, rejects the effect.

A missing `lfx2`, a missing `FrFX`, an unknown data version, a wrong-typed
numeric, a wrong `Md  ` / `Styl` / `PntT` typeID, a non-`RGBC` `Clr `, or a
descriptor that fails to parse is `None`; the decoder never panics. An `FrFX`
whose `present` or `enab` is false decodes but renders nothing.

**Non-solid fill types decode to `None`.** `PntT` is decoded (defaulting to
`SClr`); value `SClr` is solid, and value `GrFl` or `Ptrn` returns `None` for
this slice. This is the stated ceiling: a gradient or pattern stroke is ignored
exactly as it is today rather than being rendered wrongly as a solid band. Any
unknown `PntT` value also returns `None`.

**Colour default divergence.** The docs Stroke table lists the colour default as
"foreground `(inferred)`"; the decoder cannot resolve a document foreground
(smart-filter/pixel documents carry no foreground resource in this crate), so it
defaults to black, matching the shadow kinds and the change's stated shape. Real
files always carry `Clr `, so the default is observable only for a hand-built
minimal descriptor; it is recorded as a divergence, not a parity claim.

**Size default divergence.** The docs Stroke table and this decoder default
`Sz  ` to 3; psd-tools' `Stroke.size` accessor defaults to 0.0. Real files always
carry `Sz  `, so the divergence is observable only for a hand-built minimal
descriptor. A decoded `Sz  ` of 0 is clamped up to the minimum size 1 (the
decoded range is `1..=250`), so the hand-built no-op of D6 (a `Stroke` with
`size` 0 that never went through decode) is distinct from a decoded zero size.

**Band width divergence.** psd-tools' `draw_stroke_effect` builds its edge mask
with a doubled radius/edge mask; this change's exact integer max/min
(`dilate`/`erode`) band is the documented approximation.

### D4. Band formula and pixel rounding

Let `M` be the masked content coverage (drop-shadow design D4: pixel `-1` alpha,
`SoCo`/`GdFl`/`PtFl` fill alpha, or `1.0` inside the rect of a channel-less smart
source, multiplied by `mask_alpha/255`). Let `n` be the decoded integer size
(clamped again in the composite helper so a hand-built `Stroke` cannot panic).
`dilate` is `dilate_matte` (a separable max filter of integer radius), `erode` is
`erode_matte` (a separable min filter of the same radius), both exact and
already shipped.

```
Outside:  band = dilate(M, n) - M
Inside:   band = M - erode(M, n)
Center:   out_r = (n + 1) / 2      // integer ceil
          in_r  = n / 2            // integer floor
          band  = dilate(M, out_r) - erode(M, in_r)
band      = band.clamp(0, 1)       // f32 partial-alpha safety
stroke(x, y) = band(x, y) · opacity/100
```

- The filters are integer-radius max/min filters; there is **no blur**. A size
  `n` outside band is exactly `n` pixels thick; an inside band is exactly `n`
  pixels thick; a center band is `out_r` pixels outside plus `in_r` inside, so
  `out_r + in_r = n` exactly. For odd `n` the extra pixel is biased **outward**
  (`out_r = ceil(n/2)`); Photoshop's exact odd-size bias is unverified and
  recorded as a stated approximation. The literal symmetric
  `dilate(M, round(n/2)) - erode(M, round(n/2))` is rejected because it renders
  an even-width band for odd `n`.
- `M` confines the Inside band to the interior; `dilate(M, n) − M` is zero
  wherever the content is opaque, so the Outside band never alters the interior;
  the Center band's `erode(M, in_r)` term is zero outside the content, so the
  outer half of the band does not alter the interior's `erode` core.
- For binary `M` the values are 0/1; for a partial-alpha matte they are graded,
  and the final `clamp(0, 1)` guards against the min/max difference leaving the
  unit interval.

### D5. Compositing above the content

A stroke is drawn on the layer surface, so it must composite **after** the
layer's content, like Inner Shadow and Inner Glow. `composite_layer_effects_above`
in `mod.rs` gains a third check after the two interior effects:

```
if let Some(stroke) = decode_stroke(layer) {
    if stroke.enabled && stroke.present {
        strokes::composite_stroke(canvas, layer, doc, &stroke);
    }
}
```

The shared group / destructive-adjustment skip is unchanged. `composite_stroke`
reuses `blend_parts` with the stroke's own `blend_mode`: for each pixel over the
padded region it computes `alpha = band · opacity/100` and, when positive, calls
`blend_parts(canvas, x, y, color, alpha, stroke.blend_mode)`. The layer's opacity,
fill and mask are not applied a second time; the mask is already folded into `M`
(and therefore into the band).

ponytail: the stroke is composited after the interior effects, so it sits on top
of them; Photoshop's exact inter-effect order (and the isolated
`Blend Interior Effects As Group` composite) is not modelled. The observable
contract (a band at the content edge, tinted by colour/opacity/blend, position
selects the side, exterior/interior pixels unchanged as the position allows)
holds.

### D6. Bounding and early-out

Content exists only inside `source = clip_rect(layer, w, h)`. An Outside or
Center band extends at most `out_r ≤ n` pixels beyond `source`; an Inside band
is inside `source`. The matte is built over `padded = pad_rect(source, n, w, h)`
for every position: the outer pad both bounds the outside half of the band and,
for Inside/Center-adjacent erosion, supplies the zero content beyond the source
edge that the min filter needs (edge-clamping a `source`-sized buffer would keep
a spurious `erode = 1` at the boundary). The composite runs over `padded`.
Early-out on an empty `source`, an empty `padded`, an all-zero `M`, `size == 0`
(a hand-built `Stroke`), or `opacity == 0`. Clamp the numerics again so a
hand-built `Stroke` cannot panic. The max/min filters are separable, so a
canvas-filling layer with `size = 250` costs `O(canvas · size)` — the existing
compositor ceiling, inherited unchanged.

### D7. Shared plumbing, not duplicated math

`content_matte`, `clip_rect`, `pad_rect`, `rect_empty`, `dilate_matte`,
`erode_matte`, `clamp_finite`, `num_or`, `num_clamped`, `bool_or`, `decode_color`
and `blend_parts`/`mask_alpha` are reused unchanged. `Stroke` and
`StrokePosition` are the only new types. No trait, factory or generic pipeline is
added.

### D8. GPU fallback

`check_supported`'s `walk` (`crates/pictura-render/src/gpu/mod.rs:273-301`)
currently rejects an enabled, present `DropShadow`, `OuterGlow`, `InnerShadow`
and `InnerGlow`. It gains the same test for `decode_stroke`, returning the
existing `GpuError::UnsupportedLayerEffect` before any dispatch, ahead of the
adjustment check. `composite_active` / `composite_gpu_or_cpu` already treat every
`GpuError` as a CPU fallback, so no call-site change is needed.

### D9. Fixture and oracle

`scripts/generate-fixtures.py` gains `stroke()`: a `Base` pixel layer plus a
`Stroked` pixel layer whose record carries a `DescriptorBlock2` with
`masterFXSwitch: Bool(True)` and an `FrFX` object (`Descriptor` with classID
`b"FrFX"`) under `Tag.OBJECT_BASED_EFFECTS_LAYER_INFO`. The authored keys mirror
the probe: `enab`, `present`, `showInDialog`, `Md  ` (`BlnM`/`Nrml`), `Clr `
`RGBC` (0, 0, 0), `Opct` 100, `Styl` (`FStl`/`OutF`), `PntT` (`FrFl`/`SClr`),
`Sz  ` 3 px, `overprint` false. The new `stroke.psd` is registered in `FIXTURES`
as `"stroke.psd": stroke` and regenerated; existing fixtures must stay
byte-identical.

The codec oracle mirrors the predecessor ones: the `lfx2` block is present in
`extra_blocks` with version 1 / data version 16, the whole `Document` round-trips
`write_psd`/`read_psd` with `lfx2` preserved, and a self-skipping psd-tools check
reads the layer's effect as `Stroke` asserting `enabled`, `present`, `opacity`,
`blend_mode`, `position`, `fill_type`, `size` and colour. `decode_stroke` then
decodes the same bytes and the composite differs from the no-effect composite.

### D10. Deferred fidelity, marked as ceilings

`ponytail:` ceilings record: only the solid-colour fill is decoded — a `GrFl` or
`Ptrn` `PntT` decodes to `None`; contour (`TrnS`), anti-alias (`AntA`) and
`overprint` are ignored; the colour default diverges from the docs' "foreground
`(inferred)`"; the Center odd-size bias and Photoshop's exact inter-effect order
among the above-content effects are not modelled; `Scale Effects` is not applied;
groups, adjustment layers and smart filters carry no effect; the stroke math runs
over a bounded padded `f32` buffer (the existing compositor ceiling).

### D11. No app change

The app composites through `pictura_render::composite_rgba` /
`composite_active`; a decoded stroke renders there with no command, panel, or
`CMakeLists.txt` change. No authoring UI and no C++ self-test check are added.
`docs/dev/STATE.md` is the only doc that would name the change, and it is updated
separately under `TASK-ALLOWS-DOCS`.

## Risks / Trade-offs

- **The fill-type and position typeIDs are easy to get wrong.** The change
  prompt suggested `FStT`/`FrFl` for the fill type; the references agree the
  position typeID is `FStl` and the fill type is `PntT` typeID `FrFl`. Mitigation:
  D3 records the exact psd-tools `Key`/`Type`/`Enum` bytes, the
  `composite/effects.py` switch, and a live round-trip probe; decode tests assert
  a wrong `Styl`/`PntT` typeID is `None`.
- **The colour default diverges from the docs.** Mitigation: real files carry
  `Clr `, the divergence is named, and the fixture authors an explicit colour.
- **A non-solid `FrFX` is dropped.** Mitigation: this matches the status quo (no
  stroke rendered), is named as a ceiling, and the fill type is decoded so a
  later slice only has to add the gradient/pattern fill.
- **The descriptor is untrusted input.** Missing keys default, wrong types and
  non-finite numbers return `None`, and the descriptor reader is depth-capped;
  the decoder never panics.

## Open Questions

- Whether Photoshop CS6 writes a `PntT` or `Styl` value outside the sets above;
  the decoder returns `None` / falls back and a CS6 capture would settle it.
- Photoshop's exact odd-size Center bias and the inter-effect order among Stroke,
  Inner Shadow and Inner Glow need a CS6 pixel baseline; the docs' behavioral
  proposal gives direction tests but no reference render.
- The exact gradient/pattern `FrFX` keys (`Grad`/`GrdT`, `Angl`, `Type`, `Rvrs`,
  `Algn`, `Scl `, `Ptrn`, `Lnkd`) for a later fill slice.
