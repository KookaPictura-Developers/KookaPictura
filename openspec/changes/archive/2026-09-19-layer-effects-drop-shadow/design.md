## Context

A Photoshop layer's **object-based** layer effects live in the `lfx2`
additional-layer-info tagged block (`Tag.OBJECT_BASED_EFFECTS_LAYER_INFO =
b"lfx2"`; the legacy `lrFX` block is a different, obsolete structure). `lfx2`
is a `DescriptorBlock2`: a `u32` version and a `u32` data version followed by a
version-16 descriptor body. Each effect is a key on the top-level descriptor
whose value is an `Objc` whose class id is the effect kind (`DrSh` for Drop
Shadow); a `masterFXSwitch` bool and an optional `Scl ` carry the document-wide
effect switch and scale. `pictura-codec` preserves `lfx2` verbatim in
`Layer.extra_blocks` (`read.rs` routes unknown keys there; `write.rs` re-emits
them), but nothing decodes or renders it. There is no `pictura_style` crate yet.

The pieces that already exist: `pictura-codec`'s descriptor DOM
(`read_descriptor`/`write_descriptor`), `pictura-render`'s compositor
(`Canvas`, `composite_layer`, `blend_into`) and generative fills (`fill.rs`),
and `pictura-filters`' Gaussian blur (`blur::gaussian` /
`kernel::gaussian_blur_planes`). psd-tools models the effect set in
`api/effects.py` and its key names in `terminology.Key`; its `DropShadow` class
is the reference for the parameter meanings, and psd-tools can both author and
re-read an `lfx2` block (verified below).

## Goals / Non-Goals

**Goals:**

- Decode the `lfx2` `DrSh` object into a typed `DropShadow`, with Photoshop
  defaults and a no-op on absent or malformed input.
- Render the drop shadow on the CPU behind the layer's own content, reusing the
  existing compositor and blur.
- Keep the GPU path panic-free by rejecting an effect layer before dispatch and
  falling back to the CPU composite.
- Prove the decode and render against a psd-tools-authored `drop_shadow.psd`
  fixture and hand-built descriptors.

**Non-Goals:**

- Any effect other than Drop Shadow, the legacy `lrFX` block, and a GPU shader.
- The document global-light resource (1037), `Scale Effects`, styles on groups,
  `Create Layers` / `Rasterize Layer Style`, presets, and authoring UI.
- Drop-shadow fidelity beyond the first slice: knock-out semantics, layer
  opacity scaling the effect, `Layer Mask Hides Effects`, contour, noise,
  anti-alias.

## Decisions

### D1. `lfx2` is an extra block, not an adjustment key

Effects are not adjustments: a layer can carry pixels and an effect at once, the
effect decorates the layer's own content, and a layer has exactly one
`AdjustmentData` slot. Routing `lfx2` through `ADJUSTMENT_KEYS` / `decode_adjustment`
was rejected. Instead `pictura-core` gains a small getter over the preserved
blocks:

```rust
impl Layer {
    pub fn extra_block(&self, key: &[u8; 4]) -> Option<&LayerBlock> {
        self.extra_blocks.iter().find(|b| &b.key == key)
    }
}
```

and `pictura-render` gains `layer_effects.rs` with the typed `DropShadow` and
`decode_drop_shadow(&Layer) -> Option<DropShadow>`. This keeps descriptor parsing
in the crate that already decodes `SoCo`/`GdFl`/`PtFl` and leaves the preserved
bytes as the source of truth for re-save.

`DropShadow`:

```rust
pub struct DropShadow {
    pub enabled: bool,
    pub present: bool,
    pub blend_mode: BlendMode,
    pub color: [u8; 3],
    pub opacity: f32,        // percent, 0..=100
    pub angle_deg: f32,      // local angle
    pub distance: f32,       // pixels, 0..=30000
    pub spread: f32,         // percent, 0..=100 (PSD `Ckmt`)
    pub size: f32,           // pixels, 0..=250, Gaussian radius
    pub use_global_angle: bool,
    pub knocks_out: bool,
}
```

### D2. Descriptor keys and layout (grounded)

The `lfx2` payload starts with `DescriptorBlock2`'s two `u32`s. A probe with
psd-tools wrote a `DrSh` block whose header bytes are `00000001 00000010`
(version 1, data version 16) and whose effect object is `44 72 53 68` (`DrSh`)
followed by an `Objc` body. psd-tools re-read it and resolved every accessor.
Because `data[4..8]` is the data version `16`, the decoder skips only the
leading `u32` version and hands `&data[4..]` to
`pictura_codec::read_descriptor`, whose version-16 check consumes the data
version and then reads the body.

The `DrSh` object's keys (human name → PSD key → value type), from psd-tools
`api/effects.py`'s accessors and `terminology.Key`:

| Parameter | PSD key | Type | Default |
|---|---|---|---|
| enabled | `enab` | `bool` | false |
| present | `present` | `bool` | false |
| shown in dialog | `showInDialog` | `bool` | false |
| blend mode | `Md  ` | `enum` typeID `BlnM` | Normal |
| colour | `Clr ` | `RGBC` object, `Rd `/`Grn `/`Bl  ` `doub` 0..=255 | black |
| opacity | `Opct` | unit float percent (or `doub`) | 75 |
| use global light | `uglg` | `bool` | true |
| local angle | `lagl` | unit float degrees | 120 |
| distance | `Dstn` | unit float pixels | 5 |
| spread | `Ckmt` | unit float percent | 0 |
| size | `blur` | unit float pixels | 5 |
| contour | `TrnS` | descriptor | Linear |
| anti-alias | `AntA` | `bool` | true |
| noise | `Nose` | unit float percent | 0 |
| knocks out | `layerConceals` | `bool` | true |

Two brief key names are corrected by the oracle: the spread key is `Ckmt`
(psd-tools `Key.ChokeMatte`, Photoshop reusing the legacy choke key for Drop
Shadow's Spread), not `chokeMatte`, and the knock-out flag is
`layerConceals`, not `knocksOut`. The blend enum is a `BlnM` typeID whose value
is a normal Photoshop blend-mode key (`mul ` for Multiply), decoded through
`pictura_core::BlendMode::from_psd_key`; an unknown or missing key is `Normal`.
The colour object class is `RGBC` and its components are on the `0..=255` scale,
like `SoCo`. Numeric values are read from either `UnitFloat` or `Double` so a
writer that drops the unit still decodes; a non-finite value, or a finite `f64`
that overflows to infinity as an `f32`, is rejected. A finite value outside its
documented range is clamped (`opacity`/`spread` `0..=100`, `distance`
`0..=30000`, `size` `0..=250`) so a crafted `1e30` cannot drive an unbounded
dilate/blur.

`present` and `enab` are separate: psd-tools keeps only objects whose `present`
is set, and a `DrSh` with `enab` false is decoded but must render nothing. The
decoder returns `None` for a missing `lfx2`, a missing `DrSh`, an unknown data
version, or a parse error, and never panics.

### D3. Angle: stored `lagl`, global light deferred

psd-tools' `_AngleMixin.angle` returns the document's global angle resource
when `uglg` is set and `lagl` otherwise; the probe showed a `DrSh` with
`uglg` true resolving to the resource default `30` even though `lagl` held
`120`. Kooka Pictura does not decode image resources (`psd-support-roadmap.md`
G5), and the global-light resource is out of scope, so this slice reads `lagl`
(default 120) and renders with it; `uglg` is decoded for a later slice. This is
a stated parity ceiling (D9) and an Open Question, not a claim of parity.

### D4. Coverage matte

The shadow is the layer's own content alpha, so the matte reuses the content
path rather than a second interpretation. A helper returns the content alpha at
a canvas pixel:

- a pixel layer: the `-1` alpha channel sample;
- a `SoCo` fill: the payload's alpha;
- a `GdFl` fill: 1.0 inside the layer rect;
- a `PtFl` fill: the resolved tile's alpha (the pattern's own alpha, or the
  placeholder's 255);
- an embedded smart-object source: 1.0 inside the layer rect.

The matte is that alpha multiplied by `mask_alpha(layer, x, y) / 255`. The layer
opacity and fill are **not** folded in (a fill layer's own contribution still
applies them through the normal content composite). Groups and adjustment layers
are skipped: effects on a group and on an adjustment layer are deferred.

### D5. Offset, spread, blur, tint

For a shadow at angle `θ` (degrees) and distance `d`, the screen-space offset
(y down) is `dx = -d·cos θ`, `dy = +d·sin θ`, so the default 120° casts down and
right and 0° casts left. This satisfies the docs' "the vertical offset is
opposite the angle" and the parity criterion that the centroid is `d` px from
the content.

Spread dilates the coverage before blur. The docs give the mapping as inferred,
so this slice uses a max-filter dilate of radius `round(spread / 100 · size)` on
the matte and marks it a ceiling (D9): `spread` 0 is a no-op and 100 with a
non-zero `size` gives a hard edge, matching the docs' "expands the matte before
blur; 100 = hard edge". `size` is the Gaussian radius, blurred with
`pictura_filters::blur::gaussian` over a three-channel `PixelBuffer` whose three
planes all hold the matte (the blur leaves alpha untouched and is separable and
clamp-to-edge). The blurred
matte is multiplied by `opacity / 100` and the colour, giving the shadow's
source alpha and colour. `distance`, `spread`, `size` and `opacity` are clamped
to their documented ranges before use, so a hand-built or crafted value cannot
panic the renderer.

### D6. Compositing behind the content

`blend_into` bakes in the layer's blend mode, opacity, fill and mask. The shadow
carries its own blend mode and opacity and its matte already includes the mask,
so the source-over equation is factored into a
`blend_parts(canvas, x, y, cs, alpha, mode)` helper that `blend_into` calls with
the layer's alpha already weighted by opacity, fill and mask; the shadow calls it
with the effect's `blend_mode` and its matte alpha weighted by `opacity/100`.
`composite_layer` renders the shadow first, then the
layer's content, so the shadow sits over the lower layers and under the content.

### D7. GPU fallback

`check_supported` already walks the stack and rejects unsupported adjustments.
It gains an effect check: a visible layer whose `decode_drop_shadow` yields an
enabled effect returns the new `GpuError::UnsupportedLayerEffect`. The existing
`composite_active` / `composite_gpu_or_cpu` callers already treat any `GpuError`
as a CPU fallback, so no call-site change is needed. This mirrors the
`PatternFill` case, which decodes but has no `adjustment_params` arm.

### D8. Fixture and oracle

`scripts/generate-fixtures.py` gains `drop_shadow()`: a base pixel layer plus a
signed layer, with a `DescriptorBlock2` `DrSh` attached under
`Tag.OBJECT_BASED_EFFECTS_LAYER_INFO` on the layer record. The probe confirmed
psd-tools both writes and re-reads the effect, and a Rust decode of the same
bytes is the oracle: the `lfx2` block is present in `extra_blocks`,
`decode_drop_shadow` returns the expected parameters, and the document
round-trips through `write_psd`/`read_psd`. A self-skipping psd-tools check
asserts the fixture's effect is readable by psd-tools as DropShadow.

### D9. Deferred fidelity, marked as ceilings

`ponytail:` ceilings record: the effective angle is `lagl`, not the global-light
resource; spread maps to a max-filter dilate of radius `spread/100 · size`; the
knock-out flag (`layerConceals`) is decoded but the shadow always composites
behind the content (no knock-out of partial alpha); layer opacity does not scale
the shadow; the layer mask is folded into the matte rather than gated by `Layer
Mask Hides Effects`; contour, noise and anti-alias are ignored; groups,
adjustment layers and smart filters carry no effect; the shadow is built over a
full-canvas f32 matte (the existing compositor ceiling).

### D10. No app change

The app composites through `pictura_render::composite_rgba` /
`composite_active`; a decoded effect renders there with no command, panel, or
`CMakeLists.txt` change. No authoring UI is added, and no C++ self-test check is
needed (the coverage is Rust tests). `docs/dev/STATE.md` is the only doc that
would name the change, and it is updated separately under `TASK-ALLOWS-DOCS`.

## Risks / Trade-offs

- **The effective angle ignores global light.** A file that changed its global
  angle renders at the stored `lagl`. Mitigation: decoded `uglg` is carried for a
  follow-up; the fixture pins `lagl`; the ceiling is stated.
- **Spread mapping is inferred.** Mitigation: the default is 0 (no-op), the
  fixture uses 0, and the mapping is named as a ceiling.
- **Knock-out is not implemented.** A semi-transparent layer shows the shadow
  through it (the `knocks_out = false` look). Mitigation: the flag is decoded and
  the behaviour is a stated Non-Goal.
- **A full-canvas f32 matte per effect.** Matches the existing compositor's
  ceiling; acceptable until a profile asks for tiling.
- **The descriptor is untrusted input.** Missing keys default, wrong types and
  non-finite numbers return `None`, and the descriptor reader is depth-capped;
  the decoder never panics.
- **A new GPU error variant is public API.** Adding `UnsupportedLayerEffect` is
  additive; the existing `Display` arm and callers are updated.

## Open Questions

- The CS6 Drop Shadow dialog default angle: the docs table says `30 (global)`
  (matching the global-light default), while the brief and the classic dialog
  say 120. This slice defaults absent `lagl` to 120; a CS6 capture would settle
  which value Photoshop writes into a new effect.
- Whether Photoshop always writes `lagl` even when `uglg` is on, and whether a
  first slice should read resource 1037 instead.
- The exact spread-to-radius mapping and the offset sign convention need a CS6
  pixel baseline; the docs' parity criteria give a direction test but no
  reference render.
- Whether `present` false effects should decode at all or be dropped like
  psd-tools does.
