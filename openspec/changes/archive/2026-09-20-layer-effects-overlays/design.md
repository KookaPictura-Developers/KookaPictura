## Context

A Photoshop layer's **object-based** layer effects live in the `lfx2`
additional-layer-info tagged block (`Tag.OBJECT_BASED_EFFECTS_LAYER_INFO =
b"lfx2"`). Each effect is a key on the top-level descriptor whose value is an
`Objc` whose class id is the effect kind; `masterFXSwitch` is the document-wide
effect switch. `pictura-codec` preserves `lfx2` verbatim in `Layer.extra_blocks`.

Five kinds already ship (`DrSh`, `OrGl`, `IrSh`, `IrGl`, `FrFX`) with a shared
matte/decode pipeline and two composite passes (`composite_layer_effects` below
the content, `composite_layer_effects_above` after it). This change adds the
three **overlay** kinds, the first effects that fill the layer's own content
coverage with a source rather than building an offset/bled matte. The module was
split by pure moves into `layer_effects/{mod,shadows,glows,strokes}.rs`; the
overlays become a fourth file.

The pieces that already exist and are reused unchanged:

- `crates/pictura-render/src/layer_effects/mod.rs` — the descriptor readers
  `num_or` (:86), `finite_f32` (:95), `num_clamped` (:102), `bool_or` (:106),
  `decode_color` (:114); the entry points `composite_layer_effects` (:141) and
  `composite_layer_effects_above` (:162); `clamp_finite` (:196), `content_matte`
  (:207), `clip_rect` (:254), `rect_empty` (:263); `MAX_OPACITY = 100.0` (:68).
- `crates/pictura-render/src/composite.rs` — `blend_parts`, `mask_alpha`,
  `desc_item`, `Canvas`, used by every shipped effect.
- `crates/pictura-render/src/layer_effects/strokes.rs` — the structural template
  for an above-content fill: `decode_stroke` (:49) and `composite_stroke` (:119).
- `crates/pictura-render/src/fill.rs` — `decode_gradient_fill` (:24),
  `decode_pattern_fill` (:67), `decode_origin` (:107), `decode_kind` (:124),
  `decode_stops` (:145), `gradient_rgba` (:273), `pattern_tile_rgba` (:554),
  `fill_coverage_matte` (:582), the `Tile` sampler (:447) and
  `PATTERN_PLACEHOLDER` (:436). These are the overlay sources.
- `crates/pictura-render/src/gpu/mod.rs` — `check_supported`'s `walk` predicate
  (:273-305), which already rejects the five shipped kinds.

### Ground truth read

- The five preceding changes under
  `openspec/changes/archive/2026-09-19-layer-effects-*/` and
  `2026-09-20-layer-effects-stroke/`, plus the shipped
  `crates/pictura-render/src/layer_effects/{mod,shadows,glows,strokes}.rs`. The
  Stroke change is the template: a new `layer_effects/*.rs`, decode + composite,
  an above-content call, the GPU predicate extension, a psd-tools fixture and a
  self-skipping oracle.
- `docs/05-layers/layer-styles.md` **Color / Gradient / Pattern Overlay** table
  (~L188-194): Color Overlay — Blend Mode, Color, Opacity (100 % `(inferred)`);
  Gradient Overlay — Blend Mode, Opacity, Gradient, Reverse, Style
  (Linear/Radial/Angled/Reflected/Diamond), Align With Layer, Angle, Scale,
  Dither (CS6); Pattern Overlay — Blend Mode, Opacity, Pattern, Snap To Origin,
  Link With Layer, Scale (1–1000 % `(inferred)`). The algorithm (~L285-290):
  "**Overlays** replace the coverage-matte color: Color Overlay fills `M` with a
  flat color; Gradient Overlay evaluates a gradient (`Style`, `Align With Layer`,
  `Angle`, `Scale`, `Reverse`, CS6 `Dither`); Pattern Overlay tiles a pattern
  (`Scale`, `Link With Layer`, `Snap To Origin`)." The application order
  (~L309-316): Drop Shadow beneath the layer, the surface effects (Bevel, Satin,
  Overlays, Stroke) on/within it.

- **The effect object class ids are `SoFi`, `GrFl`, `patternFill`.** The brief
  named `SoCo`/`PtFl`; those are the **fill-layer** descriptors
  (`composite.rs:355` consumes `SoCo`; `fill.rs`'s `decode_pattern_fill`
  consumes `PtFl`). Two independent implementations agree on the *effect*
  object ids:
  - psd-tools `psd_tools/api/effects.py:395-407`: `@register(Klass.SolidFill
    .value)` where `Klass.SolidFill = b"SoFi"` (`terminology.py:183`) registers
    `ColorOverlay`; `@register(b"GrFl")` registers `GradientOverlay`;
    `@register(b"patternFill")` registers `PatternOverlay`. `Effects.__init__`
    (:61-72) looks up each effect by `item.classID`.
  - ag-psd `src/descriptor.ts` `fieldToExtType` maps `SoFi -> SoFi`,
    `GrFl -> GrFl`, `patternFill -> patternFill`; `parseEffects` (:1274-1290)
    destructures `SoFi`, `patternFill`, `GrFl` (and the multi variants
    `solidFillMulti`/`gradientFillMulti`). `serializeEffects` writes them.
  - Live round-trip probe (this session): a `DescriptorBlock2` authored with
    `masterFXSwitch` plus `SoCo` / `GrFl` / `PtFl` objects fails psd-tools
    `PSDImage.save` with `ValueError: Effect class not found for b'SoCo'`;
    authoring `SoFi` / `GrFl` / `patternFill` saves and reopens with
    `ColorOverlay`, `GradientOverlay`, `PatternOverlay`, each `enabled`/`present`
    true. So the decoder keys on the psd-tools/ag-psd ids.

- **Exact keys and enum typeIDs** (ag-psd `parseEffectObject`/`serializeEffectObject`
  and psd-tools `_ColorMixin`/`_GradientMixin`/`_PatternMixin`/`_AlignScaleMixin`):

  | Overlay | Object class id | Keys |
  |---|---|---|
  | Color | `SoFi` | `enab`, `present`, `showInDialog`, `Md  ` (`BlnM`), `Clr ` (`RGBC` `Rd `/`Grn `/`Bl  `), `Opct` |
  | Gradient | `GrFl` | `enab`, `present`, `showInDialog`, `Md  ` (`BlnM`), `Opct`, `Grad` (`Grdn`, `GrdF`=`CstS`, `Clrs` stops), `Angl`, `Type` (`GrdT`), `Rvrs`, `Algn`, `Scl `, `Ofst` (`Pnt `, ignored), `Dthr` (ignored) |
  | Pattern | `patternFill` | `enab`, `present`, `showInDialog`, `Md  ` (`BlnM`), `Opct`, `Ptrn` (`Ptrn` object with `Nm  ` / `Idnt`), `Scl `, `Angl` (decoded, not applied), `Algn`, `phase` (`Pnt ` origin, ignored if absent) |

- **Defaults** (psd-tools accessors + Photoshop UI; the probe confirmed the
  authored values decode one-to-one):
  - Color Overlay: blend **Normal** (`_ColorMixin.blend_mode`, `effects.py:203`),
    opacity **100** (`_Effect.opacity`, `:175`), colour **red `#FF0000`**
    (Photoshop's default; `(inferred)` — real files always carry `Clr `, so the
    default is observable only for a hand-built minimal descriptor; this crate
    cannot resolve a document foreground).
  - Gradient Overlay: blend **Normal**, opacity **100**, gradient
    **black→white linear** (`Type` default `Lnr `, `_GradientMixin.type`, `:268`;
    `Angl` default 0, `:257`), `Rvrs` **false** (`:274`), `Scl ` **100**
    (percent; psd-tools' accessor defaults to `1.0` — a divergence, real files
    carry `Scl `), `Algn` **true** (Photoshop "Align With Layer" is on by
    default; psd-tools returns false for an absent key — a divergence).
  - Pattern Overlay: blend **Normal**, opacity **100**, `Scl ` **100**, `Algn`
    **true** (Photoshop "Link With Layer" is on by default), `Angl` 0.

## Goals / Non-Goals

**Goals:**

- Decode the `lfx2` `SoFi`/`GrFl`/`patternFill` objects into three typed params
  structs, reusing `GradientStop`/`GradientKind`/`GradientFillParams` and
  `PatternFillParams` and the existing descriptor readers, with Photoshop
  defaults and a no-op on absent or malformed input.
- Render each overlay on the CPU as the source colour composited above the
  content with alpha `M · source_alpha · opacity/100` and the overlay's blend
  mode, confined to the content coverage.
- Keep the GPU path panic-free by rejecting an enabled and present overlay
  before dispatch and falling back to the CPU composite.
- Prove the decode and render against psd-tools-authored fixtures and
  hand-built descriptors.

**Non-Goals:**

- Gradient noise/`Dither`/`Ofst`/stop midpoints, and pattern rotation (`Angl`).
- Bevel & Emboss (`ebbl`), Satin (`ChFX`), the legacy `lrFX` block, styles on
  groups, the isolated `Blend Interior Effects As Group` composite, `Scale
  Effects`, a GPU shader and any authoring UI.

## Decisions

### D1. Three overlay kinds in one new file

No new crate, module trait, or dependency. `mod.rs` already holds the shared
plumbing and gains only `mod overlays;`, the re-exports and three calls. The new
file:

```
crates/pictura-render/src/layer_effects/overlays.rs
  ColorOverlay, GradientOverlay, PatternOverlay
  decode_color_overlay, decode_gradient_overlay, decode_pattern_overlay
  composite_color_overlay, composite_gradient_overlay, composite_pattern_overlay
```

`mod.rs` gains `mod overlays;` and
`pub use overlays::{...}`; `lib.rs`'s `pub use layer_effects::{...}` list gains
the three structs. The file is well under the 1200 LOC cap.

### D2. Typed params

```rust
pub struct ColorOverlay {
    pub enabled: bool,
    pub present: bool,
    pub blend_mode: BlendMode, // default Normal
    pub color: [u8; 3],        // default red (inferred)
    pub opacity: f32,          // percent, 0..=100
}

pub struct GradientOverlay {
    pub enabled: bool,
    pub present: bool,
    pub blend_mode: BlendMode,       // default Normal
    pub opacity: f32,                // percent, 0..=100
    pub stops: Vec<GradientStop>,    // at least two, strictly increasing
    pub reverse: bool,               // Rvrs, default false
    pub kind: GradientKind,          // Type/GrdT, default Linear
    pub angle_deg: f32,              // Angl, default 0
    pub scale: f32,                  // Scl, percent, default 100
    pub align_with_layer: bool,      // Algn, default true
}

pub struct PatternOverlay {
    pub enabled: bool,
    pub present: bool,
    pub blend_mode: BlendMode,  // default Normal
    pub opacity: f32,           // percent, 0..=100
    pub pattern_id: String,     // Ptrn/Idnt
    pub scale: f32,             // Scl, percent, default 100
    pub angle_deg: f32,         // Angl, decoded for symmetry; not applied
    pub align_with_layer: bool, // Algn, default true
    pub origin: (i32, i32),     // phase/Pnt , default (0, 0)
}
```

`GradientStop`/`GradientKind` are `pictura_adjust` types re-exported by
`pictura-render`. No new type is introduced beyond these three structs.

### D3. Descriptor keys and decode contract (grounded)

`decode_*_overlay` takes `layer.extra_block(b"lfx2")`, requires at least `8`
bytes, hands `&data[4..]` to `pictura_codec::read_descriptor`, requires the
top-level object, and finds the effect `Objc` whose class id is the canonical id
(`SoFi` / `GrFl` / `patternFill`). It then reads `enab`, `present`, `Md  `
(typeID `BlnM`, default/unknown → `Normal`), and `Opct` (default 100, clamped
`0..=100`). Numerics are accepted as `UnitFloat` or `Double`; a value that is
non-finite, or finite as `f64` but overflows to infinity as `f32`, rejects.

- Color: `Clr ` (`RGBC`, default red).
- Gradient: the `GrFl` object's gradient fields are decoded by the reused
  `fill::gradient_params_from_desc` (see D7), then stored in the struct fields.
  The strict helper requires `Angl` and `Type`, so the **overlay layer** injects
  Photoshop's defaults before the call (`Angl` absent → `0`, `Type` absent →
  `GrdT`/`Lnr ` Linear); a present but malformed key is left to reject.
- Pattern: the `patternFill` object's pattern fields are decoded by the reused
  `fill::pattern_params_from_desc` (see D7), which yields `pattern_id`, `scale`,
  `align_with_layer` and the `phase` origin; the origin is carried in
  `PatternOverlay` so the render path does not re-parse the block.

A missing `lfx2`, a missing effect object, an unknown data version, a wrong
class id, a wrong-typed numeric, a wrong `Md  ` / `Type` / `GrdF` typeID, a
non-`RGBC` `Clr `, a `Grd`/`Ptrn` that is not an object, a missing `Idnt`, or a
descriptor that fails to parse is `None`; the decoder never panics. An effect
whose `present` or `enab` is false decodes but renders nothing.

### D4. The overlay formula

Let `M` be the masked content coverage (drop-shadow design D4: pixel `-1` alpha,
`SoCo`/`GdFl`/`PtFl` fill alpha, or `1.0` inside the rect of a channel-less smart
source, multiplied by `mask_alpha/255`), and let `S(x, y)` be the overlay source
colour with alpha `S_a(x, y)`:

```
overlay(x, y) = S(x, y) with alpha  M(x, y) · S_a(x, y) · opacity/100
```

composited with `blend_parts(canvas, x, y, S_rgb, alpha, blend_mode)` **above**
the layer content. This is the docs' "replace the coverage-matte color":
`M` multiplies the source, so the overlay is absent where the content is absent
and fully replaces the content colour where `M = 1`, `Normal`, `opacity = 100`.

- Solid: `S = color/255`, `S_a = 1`.
- Gradient: `S` is the pixel of `fill::gradient_rgba` (which is always opaque,
  `S_a = 1`), sampled as in D5.
- Pattern: `S`/`S_a` are the pattern tile pixel (nearest-neighbour tiling, the
  placeholder when the id is absent), sampled as in D6.

The layer's opacity, fill and mask are not applied a second time; the mask is
already folded into `M`. Early-out on an empty content rect, an all-zero `M`,
`opacity == 0`, or an empty source; clamp the numerics again so a hand-built
struct cannot panic. There is no blur, so no padding and no `O(canvas·size)`
cost: the composite is `O(content rect)` plus the one gradient/pattern tile
generation.

### D5. Gradient source and `align_with_layer`

Reuse `fill::gradient_rgba(rect_w, rect_h, &params)`.

- `align_with_layer = true`: the gradient rect is the **layer rect**
  (`layer.rect.width() × height()`); pixel `(x, y)` samples local
  `(x - rect.left, y - rect.top)`.
- `align_with_layer = false`: the gradient rect is the **canvas**
  (`canvas.w × canvas.h`); pixel `(x, y)` samples `(x, y)`.

`angle_deg`, `scale`, `reverse` and `kind` flow through `GradientFillParams` into
the existing geometry (`gradient_rgba` :273), so the overlay shares the fill
layer's gradient math exactly. ponytail: `Ofst` (offset) is not modelled and the
canvas-sized unaligned buffer is the existing compositor ceiling.

### D6. Pattern source, `align_with_layer` and the origin

Reuse `pictura_codec::decode_patterns(doc)` and
`fill::pattern_tile_rgba(patterns, &params, rect_left, rect_top, w, h)` over the
layer rect, where `params` is the `PatternFillParams` from D3. `pattern_tile_rgba`
already resolves the pattern by id (the `PATTERN_PLACEHOLDER` grey when the id is
absent — the documented fallback) and applies `scale` (nearest-neighbour, the
existing ceiling). `link_with_layer` is set from `align_with_layer`, so `true`
anchors the tile to the layer rect and `false` to the document origin, exactly as
the fill-layer path. `phase`'s `Hrzn`/`Vrtc` supplies the origin when present.
ponytail: pattern `Angl` rotation is decoded but not applied.

### D7. Reuse the fill decoders by extracting their descriptor bodies

`fill::decode_gradient_fill` and `fill::decode_pattern_fill` currently do
`read_descriptor` and then the field work inline. Extract the field work into
`pub(crate) fn gradient_params_from_desc(&DescValue) -> Option<GradientFillParams>`
and `pub(crate) fn pattern_params_from_desc(&DescValue) -> Option<PatternFillParams>`,
and make the public decoders thin wrappers. The overlay decoders call the same
helpers on the `GrFl`/`patternFill` objects, so no gradient-stop or
pattern-payload decoding is duplicated. The gradient helper's angle/type keys
(`Angl`/`Type`) are the overlay's too, with the overlay-layer defaults of D3.

This is **not** only-adds-acceptance: the extraction also **tightens** the
shared helper. It requires `Type`'s enum typeID to be `GrdT` and the nested
`GrdF`'s to be `GrdF` (`CstS`), where the old `decode_gradient_fill` matched on
the enum value alone; and it rejects a finite `f64` that overflows `f32` for
`Angl`/`Scl ` where the old code cast unchecked to `f32`. Both are the
robustness contract this crate states elsewhere ("wrong typeID / non-finite
rejects"): the typeID is part of the descriptor contract, and a cast to
infinity must not reach the gradient geometry. The public `GdFl` decoder is a
superset for valid files and rejects the same malformed inputs or more;
`gradient_fill_decode_rejects_malformed` still pins the `GdFl` contract (it
never asserted leniency for a wrong `Type`/`GrdF` typeID). `pattern_params_from_desc`
remains a pure move; its strict `PtFl` contract is unchanged.

### D8. Compositing above the content, order

An overlay is a surface effect, so it composites **after** the layer's content.
`composite_layer_effects_above` in `mod.rs` gains three checks after Inner Shadow
and Inner Glow and **before** Stroke:

```
color overlay -> gradient overlay -> pattern overlay -> stroke
```

so a stroke still outlines the top. The shared group / destructive-adjustment
skip is unchanged. ponytail: Photoshop's exact inter-effect order is not
modelled, matching the stroke ceiling.

### D9. Bounding and early-out

Overlays are confined to `source = clip_rect(layer, w, h)`, where the content
matte `M` is built; no padding is needed. Unaligned gradient generation uses the
canvas rect (one buffer, the existing ceiling). Early-out on an empty `source`,
an all-zero `M`, or `opacity == 0`. Every pixel outside `source` is byte-identical
to the same document without the effect.

### D10. GPU fallback

`check_supported`'s `walk` (`crates/pictura-render/src/gpu/mod.rs:273-305`) gains
the same enabled-and-present tests for the three overlay decoders, returning the
existing `GpuError::UnsupportedLayerEffect` before any dispatch, ahead of the
adjustment check. `composite_active` / `composite_gpu_or_cpu` already treat every
`GpuError` as a CPU fallback, so no call-site change is needed. A non-solid
pattern or gradient that fails to decode returns `None` from the decoder and does
not reject.

### D11. Fixtures and oracles

`scripts/generate-fixtures.py` gains `color_overlay()`, `gradient_overlay()` and
`pattern_overlay()`: a `Base` pixel layer plus an overlay pixel layer whose
record carries a `DescriptorBlock2` with `masterFXSwitch: Bool(True)` and the
effect object under `Tag.OBJECT_BASED_EFFECTS_LAYER_INFO`, exactly as the
`stroke()` builder does. The pattern builder also writes the existing 2×2
`_fixture_pattern()` into the global `Patt` block (mirroring `_pattern_fill`), so
psd-tools reads the `PatternOverlay` without a missing-pattern warning and the
Rust renderer resolves the real tile. The new `color_overlay.psd`,
`gradient_overlay.psd` and `pattern_overlay.psd` are registered in `FIXTURES` and
regenerated; existing fixtures must stay byte-identical.

The codec oracle mirrors the predecessor ones: the `lfx2` block is present in
`extra_blocks` with version 1 / data version 16, the whole `Document` round-trips
`write_psd`/`read_psd` with `lfx2` preserved, and a self-skipping psd-tools check
reads the effect as `ColorOverlay`/`GradientOverlay`/`PatternOverlay` asserting
`enabled`, `present`, `opacity`, `blend_mode` and the overlay-specific fields.
`decode_*` then decodes the same bytes and the composite differs from the
no-effect composite.

### D12. Deferred fidelity, marked as ceilings

`ponytail:` ceilings record: gradient `Ofst`, noise, CS6 `Dither`, stop midpoints
and non-linear interpolation are not modelled (inherited from `fill.rs`); pattern
`Angl` rotation is decoded but not applied; the colour default diverges from any
document foreground; `Scale Effects` is not applied; groups, adjustment layers
and smart filters carry no effect; the exact inter-effect order is not modelled;
the unaligned gradient buffer is canvas-sized.

### D13. No app change

The app composites through `pictura_render::composite_rgba` / `composite_active`;
an overlay renders there with no command, panel, or `CMakeLists.txt` change. No
authoring UI and no C++ self-test check are added.

## Risks / Trade-offs

- **The brief's class ids (`SoCo`/`PtFl`) are wrong for the effect objects.** If
  the decoder keyed on them, real PSDs and the psd-tools fixtures would not
  decode. Mitigation: D3 keys on the two-oracle-confirmed `SoFi`/`GrFl`/
  `patternFill`, and the design records the psd-tools/ag-psd anchors and the
  failing `SoCo` round-trip probe.
- **Gradient/pattern reuse couples the overlays to `fill.rs` semantics.** The
  extracted helpers keep those semantics, tightened as D7 records (typeID and
  overflow checks); the existing `gradient_fill_decode_rejects_malformed` test
  still pins the `GdFl` contract.
- **The descriptor is untrusted input.** Missing keys default, wrong types and
  non-finite numbers return `None`, and the descriptor reader is depth-capped;
  the decoder never panics.
- **Three fixtures and three test files add surface.** They are split by overlay
  so each file stays under the caps, and the fixtures reuse the shipped pattern.

## Open Questions

- Photoshop's exact inter-effect order among the overlays and Stroke, and the
  odd-size/offset details, need a CS6 pixel baseline; the docs' behavioral
  proposal gives direction tests but no reference render.
- Whether Photoshop writes an overlay object class id outside `SoFi`/`GrFl`/
  `patternFill` (e.g. a legacy spelling); a CS6 capture or a real file would
  settle it, and the decoder already ignores unknown ids.
