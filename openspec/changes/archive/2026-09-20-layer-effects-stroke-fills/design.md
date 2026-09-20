## Context

A Photoshop layer's **object-based** layer effects live in the `lfx2`
additional-layer-info tagged block (`Tag.OBJECT_BASED_EFFECTS_LAYER_INFO =
b"lfx2"`). Each effect is a key on the top-level descriptor whose value is an
`Objc` whose class id is the effect kind; `masterFXSwitch` is the document-wide
effect switch. `pictura-codec` preserves `lfx2` verbatim in `Layer.extra_blocks`.

The Stroke (`FrFX`) slice shipped solid-colour only: `decode_stroke` reads
`PntT` (typeID `FrFl`) and returns `None` for any value other than `SClr`
(`crates/pictura-render/src/layer_effects/strokes.rs:86-95`), and
`composite_stroke` tints the content-edge band with one flat `color`
(`strokes.rs:195-215`). The `ponytail:` note at `strokes.rs:39-48` names the
deferral. The module was split by pure moves into
`layer_effects/{mod,shadows,glows,strokes,overlays,satin,bevel,legacy}.rs`.

The two non-solid overlays already decode and composite exactly the sources the
stroke needs:

- `crates/pictura-render/src/layer_effects/overlays.rs` —
  `decode_gradient_overlay` (:144), `decode_pattern_overlay` (:202),
  `with_gradient_defaults` (:168), and the above-content compositors
  `composite_gradient_overlay` (:303) / `composite_pattern_overlay` (:364) that
  reuse `crate::fill`.
- `crates/pictura-render/src/fill.rs` — `gradient_params_from_desc` (:33),
  `pattern_params_from_desc` (:92), `decode_origin` (:131), `gradient_rgba`
  (:297), the private `Tile` sampler (:471) / `tile_for` (:505), and
  `pattern_tile_rgba` (:578).
- `crates/pictura-render/src/layer_effects/mod.rs` — `content_matte`,
  `clip_rect`, `pad_rect`, `rect_empty`, `dilate_matte`, `erode_matte`,
  `clamp_finite`, `num_clamped`, `bool_or`, `decode_color`, `MAX_SIZE`,
  `MAX_OPACITY`, the re-exports (:116) and the two composite passes (:299,
  :321).
- `crates/pictura-render/src/composite.rs` — `blend_parts`, `mask_alpha`,
  `desc_item`, `Canvas`.

This change adds the gradient and pattern fill to the existing `FrFX` kind. No
new effect kind, no new module, and no new dependency: the stroke routes its
band into the overlay sources.

### Ground truth read

- ag-psd `src/descriptor.ts` `parseFxObject` / `serializeFxObject` (:1187-1215):
  `fillType: FrFl.decode(fx.PntT!)`; `if (fx.Grad) stroke.gradient =
  parseGradientContent(fx as any)` and `if (fx.Ptrn) stroke.pattern =
  parsePatternContent(fx as any)`. `parseGradientContent` (:1655) reads `Grad`,
  `Type` (`GrdT`), `Dthr`, `Rvrs`, `Angl`, `Scl `, `Algn`, `Ofst`;
  `parsePatternContent` (:1673) reads `Ptrn` (`Nm  `/`Idnt`), **`Lnkd`**, and
  `phase`. `serializePatternContent` writes `Lnkd`.
- ag-psd `src/psd.ts:133-147` `LayerEffectStroke` carries `fillType`
  (`'color' | 'gradient' | 'pattern'`), `color`, `gradient`, `pattern`.
- psd-tools `psd_tools/api/effects.py` — `class Stroke(_Effect, _ColorMixin,
  _PatternMixin, _GradientMixin)` (registered on `Klass.FrameFX.value = b"FrFX"`).
  `Stroke.fill_type` reads `Key.PaintType` (`b"PntT"`) defaulting to `b"SClr"`;
  `_GradientMixin` gives `.gradient`/`.type`/`.reversed`; `_PatternMixin` gives
  `.pattern` (`b"Ptrn"`) and `.linked` (**`b"Lnkd"`**, with a "Seems a bug."
  note) and `.phase`; `_ColorMixin` gives `.color`. `Stroke` has no
  `_AlignScaleMixin`, so it exposes no `.aligned`/`.scale` accessor; the
  gradient align key `Algn` is grounded by ag-psd's `parseGradientContent`.
- `psd_tools/api/effects.py` `PatternOverlay(_OverlayEffect, _AlignScaleMixin,
  _PatternMixin)` is why the shipped **overlay** decoder uses `Algn`: the overlay
  has both mixins. The stroke does not, so its pattern link is `Lnkd`.
- **Live probe (this session, psd-tools 1.19.0).** A `DescriptorBlock2`
  authored with `masterFXSwitch` plus an `FrFX` object carrying
  `Key.PaintType = Enumerated(Type.FrameFill, Enum.GradientFill)` and a `Grad`
  `Grdn` object saves and reopens as a `Stroke` with `fill_type == b"GrFl"`,
  `gradient is not None`, `angle 45.0`, `type b"Lnr "`; the same with
  `Enum.Pattern` plus a `Ptrn` object reopens with `fill_type == b"Ptrn"` and
  `pattern is not None`. So the psd-tools authoring path (`stroke()`'s builder
  plus the `Grad`/`Ptrn` keys) works for the fixtures.
- The live `openspec/specs/layer-effects/spec.md` Stroke requirements (:481,
  :549) and `gpu-compositing` requirement (:318) that this change modifies.

## Goals / Non-Goals

**Goals:**

- Decode `PntT` `GrFl` into the `Grad` gradient content and `PntT` `Ptrn` into
  the `Ptrn` pattern content, reusing `fill::gradient_params_from_desc` /
  `pattern_params_from_desc` and the promoted `with_gradient_defaults`, with
  Photoshop defaults and a no-op on absent or malformed input.
- Replace `Stroke.color` with a `StrokeFill` enum (`Solid` / `Gradient` /
  `Pattern`); keep the solid path byte-identical.
- Composite the band from the gradient (`fill::gradient_rgba`) or pattern
  (`fill::Tile`) source over the same padded band region, above the content.
- Keep the GPU path panic-free: a gradient/pattern stroke now decodes to `Some`,
  so the existing rejection predicate forces the CPU fallback.
- Prove the decode and render against two psd-tools-authored fixtures and
  hand-built descriptors, with a stated approximation for the fill extent.

**Non-Goals:**

- Gradient noise, `Dither`, `Ofst`, stop midpoints, non-linear interpolation
  (inherited from `fill.rs`); pattern rotation (`Angl`).
- Contour (`TrnS`), anti-alias (`AntA`), `overprint`, `Scale Effects`, the exact
  inter-effect order, groups/adjustment layers/smart filters, a GPU stroke
  shader, and any authoring UI.
- Photoshop-exact stroke gradient/pattern extent and pattern phase anchoring
  (ungrounded without a CS6 pixel baseline).

## Decisions

### D1. Two fill kinds on the existing `FrFX`, not a new effect

No new module, class, trait, factory or dependency. The fill is a property of
the stroke, so `strokes.rs` grows the two decode branches and the compositor
gains the source sampling; `mod.rs` gains only the promoted helper, the
`StrokeFill` re-export, and no new call site (the single `composite_stroke`
dispatch at `mod.rs:361-365` is unchanged). `with_gradient_defaults` is the only
cross-file move.

### D2. `StrokeFill` enum, `Stroke` becomes `Clone`

```rust
/// The fill source of a stroke, from `PntT` (typeID `FrFl`).
#[derive(Debug, Clone, PartialEq)]
pub enum StrokeFill {
    /// `SClr`: one flat colour on the `0..=255` scale (default black).
    Solid([u8; 3]),
    /// `GrFl`: the gradient content under `Grad`.
    Gradient {
        params: GradientFillParams, // stops/reverse/kind/angle_deg/scale
        align_with_layer: bool,     // `Algn`, default true
    },
    /// `Ptrn`: the pattern content under `Ptrn`.
    Pattern {
        params: PatternFillParams, // pattern_id/scale/link_with_layer/origin
        angle_deg: f32,            // `Angl`, decoded for symmetry; not applied
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stroke {
    pub enabled: bool,
    pub present: bool,
    pub blend_mode: BlendMode,
    pub fill: StrokeFill,
    pub opacity: f32,          // percent, 0..=100
    pub size: u32,             // pixels, integer 1..=250
    pub position: StrokePosition,
}
```

`GradientFillParams` and `PatternFillParams` are the existing `pictura-adjust`
types (`crates/pictura-adjust/src/types.rs:122,134`, `Debug + Clone + PartialEq`)
already used by the overlays; `PatternFillParams` already carries
`link_with_layer` and the `phase` `origin`, so only the pattern `angle_deg`
symmetry field is added. Reusing them means the composite passes
`params` straight to `gradient_rgba` / the tile sampler with no rebuild.

`Stroke` drops `Copy` because of the `Vec`/`String` payloads; it keeps `Clone`
and `PartialEq`. The solid branch is a pure shape swap (`color: [u8; 3]` →
`fill: StrokeFill::Solid([u8; 3])`) and produces the same `blend_parts` call and
therefore the same pixels, so no golden changes. Consumer updates are mechanical:
`lib.rs` (:53-56) and `mod.rs` (:116) re-export `StrokeFill`; the shipped tests
asserting `stroke.color` (`tests/layer_effects/stroke.rs:175,189,841`) assert
`stroke.fill` instead. `gpu/mod.rs:313-316`, `mod.rs:256` and `legacy.rs`'s
`stroke: None` do not change.

### D3. Descriptor keys and decode contract (grounded)

`decode_stroke` keeps its current framing: `layer.extra_block(b"lfx2")`, at least
8 bytes, `&data[4..]` into `pictura_codec::read_descriptor`, require the
top-level object, find the `FrFX` `Objc` (class id `FrFX`). It then reads:

| Parameter | PSD key | Type | Default | Source |
|---|---|---|---|---|
| enabled / present | `enab` / `present` | bool | false | shipped |
| blend mode | `Md  ` | enum `BlnM` | `Normal` | shipped |
| opacity | `Opct` | unit float % | 100, clamp `0..=100` | shipped |
| position | `Styl` | enum `FStl` | `Outside` | shipped |
| size | `Sz  ` | unit float px | 3, round then clamp `1..=250` | shipped |
| **fill type** | `PntT` | enum `FrFl` | `SClr` (solid) | ag-psd `FrFl.decode(fx.PntT!)`; shipped defers only the value |
| solid colour | `Clr ` | `RGBC` | black | shipped; read only for `SClr` |
| gradient content | `Grad` | `Grdn` | required for `GrFl` | ag-psd `if (fx.Grad)` |
| gradient angle | `Angl` | unit float/`doub` deg | 0 | `with_gradient_defaults` |
| gradient type | `Type` | enum `GrdT` | `Lnr ` Linear | `with_gradient_defaults` |
| gradient reverse | `Rvrs` | bool | false | `gradient_params_from_desc` |
| gradient scale | `Scl ` | unit float/`doub` % | 100 | `gradient_params_from_desc` |
| gradient align | `Algn` | bool | true | ag-psd `parseGradientContent`; overlay default |
| pattern content | `Ptrn` | `Ptrn` object `Nm  `/`Idnt` | required for `Ptrn` | `pattern_params_from_desc` |
| pattern scale | `Scl ` | unit float/`doub` % | 100 | `pattern_params_from_desc` |
| pattern link | `Lnkd` | bool | true | psd-tools `_PatternMixin.linked`, ag-psd `parsePatternContent` |
| pattern origin | `phase` | `Pnt ` `Hrzn`/`Vrtc` | (0, 0) | `pattern_params_from_desc` |
| pattern angle | `Angl` | unit float/`doub` deg | 0 | decoded for symmetry; not applied |
| gradient `Ofst`, `Dthr` | `Ofst`, `Dthr` | `Pnt ` / bool | ignored | overlay ceiling |

Decode rules:

- `PntT` absent → `Solid` with the shipped `Clr ` default (black). This is the
  current behaviour and the shipped spec; ag-psd's `FrFl.decode(undefined)` has
  no defined fill, and a real `FrFX` always carries `PntT`.
- `PntT` `SClr` → `StrokeFill::Solid(clr)` where `Clr ` is `RGBC` (absent →
  black, non-`RGBC` → `None`), exactly as today.
- `PntT` `GrFl` → `Grad` must be an object; decode with
  `fill::gradient_params_from_desc(&with_gradient_defaults(frfx))` (which
  requires `Grad`/`GrdF` `CstS`/`Clrs`), then read `Algn` with
  `bool_or(frfx, b"Algn", true)`. `Clr ` is ignored.
- `PntT` `Ptrn` → `Ptrn` must be a `Ptrn` object; decode with
  `fill::pattern_params_from_desc(frfx)`, then override `link_with_layer`:
  `Lnkd` when present, else the `Algn` value the helper read (default true).
  Read `Angl` with `num_or(frfx, b"Angl", 0.0)`. `Clr ` is ignored.
- Unknown `PntT` value → `None`.
- Numerics accept `UnitFloat` or `Double`; a non-finite value, or a finite `f64`
  that overflows `f32`, is `None`. A wrongly-typed key is `None`. Never panics.

**Why `Lnkd` and not only `Algn` for the stroke pattern.** The stroke's
psd-tools mixin is `_PatternMixin` (reads `Lnkd`) and ag-psd's
`parsePatternContent`/`serializePatternContent` use `Lnkd`; the overlay uses
`Algn` because `PatternOverlay` also mixes in `_AlignScaleMixin`. psd-tools flags
`Lnkd` with "Seems a bug.", so the decoder prefers `Lnkd` but falls back to
`Algn` (the value `pattern_params_from_desc` already read, default true). This is
a tolerant read of an ambiguous key, recorded in Open Questions.

**psd-tools divergence.** `_PatternMixin.linked` reads `Lnkd` and defaults it to
**False** when the key is absent, while this decoder defaults the link to **true**
(the overlay's `Algn` default, via `pattern_params_from_desc`). The decoder
therefore defaults `true` for a stroke pattern with neither `Lnkd` nor `Algn`;
only a hand-built descriptor that omits both keys diverges from psd-tools. A real
Photoshop/psd-tools-authored pattern stroke carries `Lnkd`, so real files and the
fixture decode identically. `Lnkd` is read authoritatively — a valid `Lnkd` wins
even when a present-but-wrongly-typed `Algn` would otherwise reject, and `Algn`
is consulted only when `Lnkd` is absent.

**Absent `Grad`/`Ptrn` is `None`**, matching the overlay decoders (a gradient or
pattern fill whose payload is missing is malformed, not a silent solid). This is
the stated ceiling for a hand-built minimal descriptor; a real file always
carries the content object.

**`Clr ` default stays black**, not the overlay's red. The stroke's documented
default is "foreground `(inferred)`" and the shipped decoder uses black; this
change does not alter the solid default. Real files carry `Clr `, so the
divergence is observable only for a hand-built minimal descriptor.

### D4. **Fill extent and sampling geometry** (the implementer's contract)

The band is built over `padded = pad_rect(source, size, w, h)` — the same region
as today — because the band (and therefore the fill) extends up to `size` px
outside `source = clip_rect(layer, w, h)`. The fill source is sampled **over
`padded`**, per canvas pixel `(x, y)`:

- **Gradient, `align_with_layer = true`**: generate
  `gradient_rgba(layer.rect.width(), layer.rect.height(), &params)` over the
  **layer rect** (identical to `composite_gradient_overlay`), and sample local
  `(sx, sy) = (x - layer.rect.left, y - layer.rect.top)`, each **clamped** into
  `[0, gw-1]` / `[0, gh-1]`. Out-of-rect samples clamp to the nearest edge, which
  is Photoshop's gradient endpoint clamp: the stop sampler already clamps `z`
  outside the stop range, so the edge row/column is the endpoint colour.
- **Gradient, `align_with_layer = false`**: generate
  `gradient_rgba(canvas.w, canvas.h, &params)` over the **canvas** (the existing
  ceiling) and sample `(sx, sy) = (x, y)`. `padded` is canvas-clamped by
  `pad_rect`, so every sample is in range; the same clamp code is a bounded
  no-op.
- **Pattern**: sample the shipped `Tile` over `padded` at canvas `(x, y)` with
  anchor `(layer.rect.left, layer.rect.top)` when `link_with_layer` (so the
  phase matches the pattern overlay), else anchor `(0, 0)` (the sampler uses
  absolute canvas coordinates when unlinked). `Tile::sample` already wraps both
  axes with `rem_euclid`, so the pattern **tiles/repeats** across the band and
  beyond the content rect with no extra clamp.

Rationale: a stroke and an overlay with the same gradient/pattern params then
agree over their shared region (`source`), because both generate over the layer
rect and anchor the pattern to the layer rect. The stroke simply extends the
sample region to `padded`.

**Marked approximation.** Photoshop's exact stroke fill extent and pattern
anchor are ungrounded without a CS6 pixel baseline. Alternative considered and
rejected: generating the gradient over `padded` so it "extends naturally" —
that stretches the gradient's `s`/endpoints across the padded rect, diverging
from the overlay and from a layer-anchored gradient. `ponytail:` ceiling: the
aligned gradient clamps at the layer-rect edge and the pattern is anchored to
the layer rect (not to the band's outer edge); both are approximations, not
parity claims. If a future CS6 fixture shows Photoshop widens the gradient over
the padded bound, only D4's generation rect changes.

### D5. Gradient sampling implementation

```rust
let (gw, gh, left, top) = if align_with_layer {
    (layer.rect.width(), layer.rect.height(), layer.rect.left, layer.rect.top)
} else {
    (canvas.w as i32, canvas.h as i32, 0, 0)
};
if gw <= 0 || gh <= 0 { return; }
let grad = crate::fill::gradient_rgba(gw, gh, params);
// per padded pixel (x, y):
let gx = (x - left).clamp(0, gw - 1) as usize;
let gy = (y - top).clamp(0, gh - 1) as usize;
let c = grad[gy * gw as usize + gx];
```

The clamp mirrors the overlay's defensive `get` guard and makes a craft struct's
non-finite `scale` a bounded no-op. `gradient_rgba` is opaque (alpha 255), so the
band alpha is `band · opacity/100` (no extra source alpha).

### D6. Pattern sampling implementation

Add one `pub(crate)` function to `fill.rs`, extracted from `pattern_tile_rgba`
(a pure move; the two existing callers are unchanged):

```rust
pub(crate) fn pattern_tile_region(
    patterns: &[PatternPixels],
    params: &PatternFillParams,
    anchor: (i32, i32),
    region: (i32, i32, i32, i32),
) -> Vec<[u8; 4]> {
    let pattern = patterns.iter().find(|p| p.pattern_id == params.pattern_id);
    let tile = tile_for(pattern, params);
    let (x0, y0, x1, y1) = region;
    let mut out = Vec::with_capacity(((x1 - x0).max(0) * (y1 - y0).max(0)) as usize);
    for y in y0..y1 {
        for x in x0..x1 {
            out.push(tile.sample(x, y, anchor.0, anchor.1));
        }
    }
    out
}
```

`pattern_tile_rgba(patterns, params, rect_left, rect_top, w, h)` becomes a thin
wrapper with `anchor = (rect_left, rect_top)` and
`region = (rect_left, rect_top, rect_left + w, rect_top + h)`, so
`composite_pattern_fill` and `composite_pattern_overlay` are byte-identical.
The stroke calls it with `anchor = (layer.rect.left, layer.rect.top)` when
linked (else `(0, 0)`) and `region = padded`, and indexes
`(y - padded.top) * pw + (x - padded.left)`. `Tile`/`tile_for` stay private;
only the region bake is exposed. A missing pattern id still yields the shipped
grey `PATTERN_PLACEHOLDER` tile, not a no-op.

ponytail: pattern `Angl` rotation is decoded but not applied (inherited), and
the pattern library is decoded once per stroke layer (`decode_patterns(doc)`),
matching the overlay's existing per-effect decode.

### D7. Band composite formula

The band formula and early-outs are unchanged from the shipped stroke (D4 of the
stroke design): `Outside: dilate(M, n) − M`; `Inside: M − erode(M, n)`;
`Center: dilate(M, out_r) − erode(M, in_r)` with `out_r = ceil(n/2)`,
`in_r = floor(n/2)`; clamp to `0..=1`. Only the per-pixel source changes:

```
Solid:    rgb = color/255,                 src_a = 1
Gradient: rgb = sampled gradient colour,   src_a = 1
Pattern:  rgb = sampled tile colour,       src_a = tile_alpha/255
alpha = band(x, y) · src_a · opacity/100
blend_parts(canvas, x, y, rgb, alpha, stroke.blend_mode)
```

Same `blend_parts`, same `mask_alpha` folded into `M`, same early-outs (empty
source/padded, all-zero matte, `size` 0, `opacity` 0), same `clamp_finite`
re-clamp so a hand-built `Stroke` cannot panic, same `O(canvas · size)` max/min
ceiling plus one gradient/pattern buffer. The pattern tile alpha makes the band
multiplicative with the pattern's transparency, exactly as the pattern overlay.

### D8. Promote `with_gradient_defaults` (pure move)

Move `with_gradient_defaults` from `overlays.rs:168` to
`layer_effects/mod.rs` as `pub(crate)`, unchanged, and update the overlay call
site to `super::with_gradient_defaults`. It injects the absent `Angl` → `0` and
absent `Type` → `GrdT`/`Lnr ` defaults that the strict
`gradient_params_from_desc` requires; a present-but-malformed key is left for
the helper to reject. The stroke gradient decoder calls the same helper on its
`FrFX` object, so the two paths inject identical defaults with no duplication.
The module comment moves with it.

### D9. GPU fallback (spec text only)

`check_supported`'s `walk` (`crates/pictura-render/src/gpu/mod.rs:273-331`)
already rejects an enabled and present stroke via
`effects.stroke.as_ref().is_some_and(|e| on(e.enabled, e.present))`
(:313-316), without inspecting the fill type. Because `decode_stroke` now returns
`Some` for a gradient/pattern fill, such a document now returns
`GpuError::UnsupportedLayerEffect` and `composite_active` /
`composite_gpu_or_cpu` fall back to the CPU composite, which renders the fill.
No GPU code changes; only the `gpu-compositing` requirement text changes (the
clause "a stroke whose fill type is not solid SHALL NOT reject" is removed).
This also fixes a latent drop: today a gradient/pattern stroke decodes to
`None`, so the GPU path silently omits it.

### D10. Fixtures and oracles

`scripts/generate-fixtures.py` gains `stroke_gradient()` and
`stroke_pattern()`: a `Base` pixel layer plus a `Stroked` pixel layer whose
record carries a `DescriptorBlock2` with `masterFXSwitch: Bool(True)` and an
`FrFX` object under `Tag.OBJECT_BASED_EFFECTS_LAYER_INFO`, exactly as `stroke()`
does, but with `Key.PaintType` `GrFl`/`GradientFill` plus a `Grad` `Grdn` object
(`GrdF` `CstS`, a black→white `Clrs` list, `Angl`, `Type` `GrdT`/`Lnr `, `Rvrs`,
`Algn`, `Scl `), or `Ptrn`/`Pattern` plus a `Ptrn` object (`Nm  `/`Idnt`
`pictura-pattern`), `Scl `, `Algn`/`Lnkd`, `Angl`. The pattern builder also
writes the existing 2×2 `_fixture_pattern()` into the global `Patt` block
(mirroring `pattern_overlay()`), so the renderer resolves the real tile. Both
were verified authorable by the live probe above. Register
`"stroke_gradient.psd"` and `"stroke_pattern.psd"` in `FIXTURES`; existing
fixtures stay byte-identical.

The codec oracles mirror `oracle/stroke.rs`: the `lfx2` key is present in
`extra_blocks` with version 1 / data version 16, the whole `Document`
round-trips `write_psd`/`read_psd` with `lfx2` preserved, and a self-skipping
psd-tools check reads the layer's effect as a `Stroke` asserting `fill_type`
`GrFl`/`Ptrn` and the type-specific accessors (`gradient`/`type`/`angle`/
`reversed` for the gradient; `pattern` for the pattern). `decode_stroke` then
decodes the same bytes and the composite differs from the no-effect composite.

### D11. Deferred fidelity, marked as ceilings

`ponytail:` ceilings, updated on the stroke module comment: gradient noise,
`Dither`, `Ofst`, stop midpoints and non-linear interpolation are not modelled
(inherited from `fill.rs`); pattern `Angl` rotation is decoded but not applied;
the aligned gradient clamps at the layer-rect edge and the pattern is anchored to
the layer rect (D4); `Clr ` defaults to black; contour (`TrnS`), anti-alias
(`AntA`) and `overprint` are ignored; `Scale Effects` is not applied; groups,
adjustment layers and smart filters carry no effect; the exact inter-effect
order is not modelled; the stroke math runs over the bounded padded `f32`
buffer.

### D12. No app change

The app composites through `pictura_render::composite_rgba` /
`composite_active`; a gradient/pattern stroke renders there with no command,
panel, or `CMakeLists.txt` change. No authoring UI and no C++ self-test check are
added. `docs/dev/STATE.md` is the only doc that would name the change and is
updated separately under `TASK-ALLOWS-DOCS`.

## Risks / Trade-offs

- **The stroke pattern link key is ambiguous.** psd-tools marks `Lnkd` "Seems a
  bug." and the shipped overlay uses `Algn`; ag-psd and `_PatternMixin` use
  `Lnkd`. Mitigation: D3 prefers `Lnkd` (two oracles) and falls back to `Algn`
  (default true); a CS6 capture would settle it, recorded in Open Questions.
- **The fill extent is an approximation.** Photoshop's exact stroke
  gradient/pattern bound and phase anchor are ungrounded. Mitigation: D4 states
  the geometry, marks it a `ponytail:` ceiling, and keeps the stroke and overlay
  consistent over their shared region; the deferral is named, not a hidden
  parity claim.
- **`Stroke` loses `Copy`.** A `Vec`/`String` payload forces `Clone`. The crate
  is internal to the app; all consumers take `&Stroke` or move it, and the
  shipped tests are updated. No PSD byte changes.
- **The reused `pattern_params_from_desc` reads `Algn`.** A stroke pattern with
  a wrongly-typed `Algn` rejects even when `Lnkd` is well-formed. Mitigation:
  `Lnkd` is read first and passed to `pattern_params_from_desc_with_link`, which
  bypasses the helper's `Algn` requirement, so a valid `Lnkd` always wins and the
  `Algn` fallback is consulted only when `Lnkd` is absent.
- **The descriptor is untrusted input.** Missing keys default, wrong types and
  non-finite numbers return `None`, and the descriptor reader is depth-capped;
  the decoder never panics.

## Migration Plan

None: additive rendering behaviour on preserved data, no persisted-format
change, no app migration. Rollback is a revert; documents keep their `lfx2`
bytes either way.

## Open Questions

- Whether Photoshop CS6 writes `Lnkd` or `Algn` for a stroke pattern's link
  flag; the decoder reads both (preferring `Lnkd`) and a CS6 capture would let
  it key on one.
- Photoshop's exact stroke gradient/pattern extent and pattern phase anchor; D4
  approximates them and a CS6 pixel baseline would let the generation rect and
  anchor be pinned.
- Whether a stroke gradient carries `Scl ` (the stroke lacks psd-tools'
  `_AlignScaleMixin` accessor); the decoder takes the overlay default 100 and
  the fixture authors an explicit value.
