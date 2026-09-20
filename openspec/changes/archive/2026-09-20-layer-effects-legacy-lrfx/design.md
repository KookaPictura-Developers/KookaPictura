## Context

Photoshop serializes layer effects twice. The **object-based** effects live in
the `lfx2` additional-layer-info block as a `DescriptorBlock2`; Kooka Pictura
decodes and renders all ten kinds
(`crates/pictura-render/src/layer_effects/{mod,shadows,glows,strokes,overlays,satin,bevel}.rs`).
The older **legacy** effects block, key `lrFX`
(`Tag.EFFECTS_LAYER = b"lrFX"`; the placeholder `docs/05-layers/layer-styles.md:371`
calls out its `cmnS`/`dsdw`/`isdw`/`oglw`/`iglw`/`bevl`/`sofi` set), is a
**fixed binary struct**, not a descriptor. Photoshop 5.0–6.0 wrote only `lrFX`,
and CS6 still writes it for compatibility, so a real file can carry one, the
other, or both.

`pictura-codec` already preserves `lrFX` verbatim: the reader routes any
untagged additional-layer-info key to `Layer.extra_blocks`
(`crates/pictura-codec/src/read.rs:583`) and the writer re-emits it in encounter
order (`crates/pictura-codec/src/write.rs:330`). `Layer::extra_block`
(`crates/pictura-core/src/lib.rs:601`) exposes it. Nothing decodes it, so a
legacy-only file renders with no effects.

The typed params are shared and already shipped:

- `DropShadow` / `InnerShadow` (`layer_effects/shadows.rs:14,35`)
- `OuterGlow` / `InnerGlow` / `GlowSource` / `GlowTechnique` (`glows.rs:13,30,37`)
- `BevelEmboss` / `BevelStyle` / `BevelTechnique` / `BevelDirection` (`bevel.rs:34,40,47`)
- `ColorOverlay` (`overlays.rs:32`)

The compositor entry points are `composite_layer_effects` (below the content)
and `composite_layer_effects_above` (`layer_effects/mod.rs:228,249`), and the GPU
rejection predicate is `check_supported` (`src/gpu/mod.rs:273`).

The reference for the binary layout is psd-tools
`psd_tools/psd/effects_layer.py` (the low-level `EffectsLayer`; its high-level
`api/effects.py` reads only `lfx2`) plus libpsd
`src/{effects,drop_shadow,inner_glow,outer_glow,bevel_emboss,color_overlay}.c`,
which the project already cites for effect semantics.

## Goals / Non-Goals

**Goals:**

- Decode the legacy `lrFX` block into the existing typed effect params.
- Render the mapped set by reusing the shipped `composite_*` functions unchanged.
- Resolve `lfx2`-over-`lrFX` precedence once, so the two are never double-applied.
- Reject a renderable legacy effect before GPU dispatch through the same check.
- Prove decode, precedence, render equivalence and GPU rejection with a
  psd-tools-authored `lrFX` fixture and hand-built bytes.

**Non-Goals:**

- Any `lrFX`-only effect: the block has no satin, stroke, gradient overlay or
  pattern overlay record — those stay `lfx2`-only.
- Contour, noise, anti-alias, and the document global-light resource (1037).
- Authoring legacy effects, a GPU shader, `Scale Effects`, styles on groups,
  and the isolated `Blend Interior Effects As Group` composite.
- Changing any existing `lfx2` requirement or decoder.

## Decisions

### D1. `lrFX` is a fixed struct, parsed separately

The legacy block is not a descriptor, so it does not go through
`read_descriptor`. `layer_effects/legacy.rs` gets a small, bounds-checked
big-endian reader over `&layer.extra_block(b"lrFX")?.data` and returns
`Option<LegacyEffects>`. Alternatives considered: routing it through the
descriptor DOM (impossible — different encoding) and folding the parser into the
existing decoders (rejected: it would make every `decode_*` depend on the
legacy layout).

### D2. Exact `lrFX` binary layout (grounded)

The tagged-block length wrapper is stripped by the codec, so `block.data` is the
`EffectsLayer` body. All integers are big-endian. There is **no alignment
padding** (`read_length_block(fp)` uses psd-tools' default `padding=1`, i.e.
none).

```
EffectsLayer:
  version : u16                 // 0
  count   : u16                 // 6 or 7
  count × {
    signature : 4s              // "8BIM"
    ostype    : 4s              // "cmnS" | "dsdw" | "isdw" | "oglw" | "iglw" | "bevl" | "sofi"
    length    : u32             // body size in bytes
    body      : [u8; length]
  }

Color (10 bytes, used by every effect):
  space   : u16                 // 0 = RGB
  channels: 4 × u16             // R, G, B, unused (16-bit per channel)

CommonStateInfo ("cmnS", body 7):
  version : u32                 // 0
  visible : u8                  // 0/1
  unused  : 2 bytes

ShadowInfo ("dsdw" drop, "isdw" inner; psd-tools body 51, libpsd 41 = v0 / 51 = v2):
  version         : u32         // 0 or 2
  blur            : u32         // pixels (Gaussian radius)
  intensity       : u32         // percent (Spread for drop, Choke for inner)
  angle           : i32         // degrees
  distance        : u32         // pixels
  color           : Color       // 10
  blend signature : 4s          // "8BIM"
  blend mode      : 4s          // layer vocabulary, e.g. "mul ", "scrn"
  enabled         : u8
  use_global_angle: u8
  opacity         : u8          // 0..=255
  native_color    : Color       // 10, psd-tools always writes it (v0 too); libpsd omits it at v0

GlowInfo ("oglw" outer, "iglw" inner; body 32 = v0, 42/43 = v2):
  version         : u32         // 0 or 2
  blur            : u32         // pixels (size)
  intensity       : u32         // percent (Spread outer, Choke inner)
  color           : Color       // 10
  blend signature : 4s          // "8BIM"
  blend mode      : 4s
  enabled         : u8
  opacity         : u8          // 0..=255
  // version >= 2:
  //   outer: native_color : Color           (10)
  //   inner: invert : u8, native_color : Color (1 + 10)

BevelInfo ("bevl"; body 58 = v0, 78 = v2):
  version            : u32      // 0 or 2
  angle              : i32      // degrees
  depth              : u32      // strength/depth
  blur               : u32      // pixels
  hl signature + mode: 4s + 4s  // "8BIM" + layer key
  sh signature + mode: 4s + 4s
  highlight_color    : Color    // 10
  shadow_color       : Color    // 10
  bevel_style        : u8       // 0 Outer, 1 Inner, 2 Emboss, 3 Pillow, 4 Stroke
  highlight_opacity  : u8       // 0..=255
  shadow_opacity     : u8       // 0..=255
  enabled            : u8
  use_global_angle   : u8
  direction          : u8       // 0 Up, 1 Down
  // version == 2: real_highlight_color : Color, real_shadow_color : Color (20)

SolidFillInfo ("sofi"; body 34):
  version         : u32         // must be 2
  blend signature : 4s          // "8BIM"
  blend mode      : 4s
  color           : Color       // 10
  opacity         : u8          // 0..=255
  enabled         : u8
  native_color    : Color       // 10
```

The body sizes are grounded in libpsd (`drop_shadow.c` reads
`41 or 51 (depending on version)`; `bevel_emboss.c` `58 for version 0, 78 for
version 2`; `color_overlay.c` `Size: 34`) and psd-tools' `effects_layer.py`.
psd-tools' `ShadowInfo` always serialises the trailing `native_color`, so its
version-0 body is 51 bytes, not libpsd's 41 (the fixture confirms `dsdw` length
51); the parser reads through the opacity byte and ignores the tail, so either
form decodes with no runtime impact. The `intensity` slot is libpsd's
`spread`/`choke` (`drop_shadow.c` default `spread = 0`; `inner_glow.c` reads
`choke` there), which settles its mapping (D4).

### D3. Typed `LegacyEffects` and the mapping

`legacy.rs` exposes:

```rust
pub(crate) struct LegacyEffects {
    pub drop_shadow: Option<DropShadow>,
    pub inner_shadow: Option<InnerShadow>,
    pub outer_glow: Option<OuterGlow>,
    pub inner_glow: Option<InnerGlow>,
    pub bevel: Option<BevelEmboss>,
    pub color_overlay: Option<ColorOverlay>,
}

pub(crate) fn decode_legacy_effects(layer: &Layer) -> Option<LegacyEffects>;
```

Mapping (`—` = the record does not carry the field; the value is the typed
effect default, i.e. exactly what the shipped `lfx2` decoder would produce for an
absent key):

| typed field | `dsdw`/`isdw` | `oglw` | `iglw` | `bevl` | `sofi` |
|---|---|---|---|---|---|
| `enabled` | `enabled` | `enabled` | `enabled` | `enabled` | `enabled` |
| `present` | `true` | `true` | `true` | `true` | `true` |
| `blend_mode` | `blend_mode` | `blend_mode` | `blend_mode` | — (per-half, below) | `blend_mode` |
| `color` | `color` | `color` | `color` | — (per-half) | `color` |
| `opacity` | `opacity` | `opacity` | `opacity` | — (per-half) | `opacity` |
| `angle_deg` | `angle` | — | — | `angle` | — |
| `distance` | `distance` | — | — | — | — |
| `spread` | `intensity` | `intensity` | — | — | — |
| `choke` | `intensity` (inner) | — | `intensity` | — | — |
| `size` | `blur` | `blur` | `blur` | `blur` | — |
| `use_global_angle` | `use_global_angle` | — | — | `use_global_angle` | — |
| `knocks_out` | — (false) | — | — | — | — |
| `technique` | — (Softer) | — (Softer) | — | — (Smooth) | — |
| `source` | — | — | `invert ? Center : Edge` | — | — |
| `style` | — | — | — | `bevel_style` | — |
| `direction` | — | — | — | `direction` | — |
| `depth` | — | — | — | `depth` | — |
| `soften` | — | — | — | `0` | — |
| `altitude_deg` | — | — | — | `30` | — |
| `highlight` | — | — | — | `{ mode: hl mode, color: hl color, opacity: hl opacity }` | — |
| `shadow` | — | — | — | `{ mode: sh mode, color: sh color, opacity: sh opacity }` | — |

Concrete field rules:

- **Blend mode** is the *layer* vocabulary (`norm`/`mul `/`scrn`), so it is
  decoded with `BlendMode::from_psd_key` — **not** `effect_blend_mode`, which is
  for `lfx2`'s capitalized `BlnM` vocabulary. `mod.rs:188` documents that split.
  An unknown key takes the typed effect default (drop Normal, inner Multiply,
  glows Screen, bevel halves as in `bevel.rs:168,174`, color overlay Normal).
- **Colour**: a legacy `Color` is 16-bit per channel; each channel maps to
  `u8` by taking the high byte (`value >> 8`). The fixture authors `v << 8` so
  the decoded byte is `v`.
- **Opacity**: the legacy byte is `0..=255` (`drop_shadow.c` default `191` ≈
  75 %; the value feeds libpsd's `fill_opacity`). Map to percent with
  `byte as f32 * 100.0 / 255.0`, then clamp `0..=100`.
- **Bevel style** byte: `0 Outer, 1 Inner, 2 Emboss, 3 Pillow, 4 Stroke` (the
  `psd_bevel_*` order libpsd uses). `technique` defaults to `Smooth`, so a
  legacy Inner bevel renders through the shipped `Inner + Smooth` renderer; any
  other style is a no-op (D6).
- **`cmnS`**: decode `visible` (default `true` when absent) and AND it into each
  record's `enabled`, matching libpsd (`effects.c` gates the whole set on
  `data->visible`).
- **`knocks_out`** has no legacy field; set `false` (libpsd's default). The
  shipped renderer already ignores `knocks_out`, so it cannot change pixels.

### D4. Single resolver with `lfx2` precedence

The two blocks must never both apply. `mod.rs` gains one shared set and one
resolver:

```rust
pub(crate) struct LayerEffects {
    pub drop_shadow: Option<DropShadow>,
    pub outer_glow: Option<OuterGlow>,
    pub inner_shadow: Option<InnerShadow>,
    pub inner_glow: Option<InnerGlow>,
    pub bevel: Option<BevelEmboss>,
    pub satin: Option<Satin>,
    pub stroke: Option<Stroke>,
    pub color_overlay: Option<ColorOverlay>,
    pub gradient_overlay: Option<GradientOverlay>,
    pub pattern_overlay: Option<PatternOverlay>,
}

pub(crate) fn decode_layer_effects(layer: &Layer) -> LayerEffects {
    if layer.extra_block(b"lfx2").is_some() {
        LayerEffects::from_lfx2(layer) // the shipped decode_* decoders
    } else {
        decode_legacy_effects(layer).map(LayerEffects::from_legacy).unwrap_or_default()
    }
}
```

The gate is **block-level**: an `lfx2` block present at all makes it
authoritative and the whole `lrFX` block is ignored, so no effect is mixed or
applied twice. This matches "when both are present, `lfx2` takes precedence" and
is the smallest correct rule — per-effect fallback could render a legacy effect
that the newer block intentionally omitted. `composite_layer_effects`,
`composite_layer_effects_above` and `check_supported` all call
`decode_layer_effects` once per layer.

### D5. Render by reusing the shipped compositors

No `composite_*` function changes. The below-content pass consumes
`effects.drop_shadow` / `effects.outer_glow`; the above-content pass consumes
`inner_shadow`, `inner_glow`, `bevel`, `satin`, `color_overlay`,
`gradient_overlay`, `pattern_overlay`, `stroke`, exactly as today. A legacy
record that maps to a `Some` typed param renders through the same code as the
equivalent `lfx2` record; because the renderer offsets with
`dx = -distance·cos(angle)`, `dy = +distance·sin(angle)`, which is libpsd's own
convention (`drop_shadow.c`), the geometry matches.

### D6. Unmapped legacy is a documented no-op

`lrFX` has no satin/stroke/gradient/pattern record and no gradient fill for
inner glow (the legacy `Grow`/`Grad` key does not exist), so those typed fields
stay `None`. A `sofi` whose version is not 2 is skipped. A bevel whose decoded
style is not `Inner` (or whose technique would not be `Smooth`) is decoded but
the shipped renderer returns immediately — a no-op, matching the existing bevel
ceiling. None of these reject the block.

### D7. Malformed vs. missing

- Missing `lrFX` → `None`.
- A structural fault — bad `EffectsLayer` version, a truncated body, a record
  whose signature is not `8BIM`, or a count that overruns the payload — → the
  whole `decode_legacy_effects` returns `None`.
- An individually unknown or malformed record (unknown `ostype`, a `sofi` with
  version ≠ 2, a wrong `Color` space) is skipped; the remaining records still
  decode.
- Every read is bounds-checked and every conversion is total; the parser never
  panics, matching the shipped decoders' contract.

### D8. GPU: reject through the same resolved set

`check_supported` (`src/gpu/mod.rs:273`) stops calling the per-effect `lfx2`
decoders in sequence and instead resolves `decode_layer_effects(layer)` once,
then returns `GpuError::UnsupportedLayerEffect` when a renderable effect is
enabled and present: `drop_shadow`/`outer_glow`/`inner_shadow`/`inner_glow`/
`satin`/`color_overlay`/`gradient_overlay`/`pattern_overlay`/solid `stroke`, and
a bevel only when `Inner + Smooth` (the existing predicate). A legacy-only
layer therefore rejects the same way an `lfx2` layer does. No new error variant.

### D9. Fixture and oracle

`scripts/generate-fixtures.py` gains `legacy_effects()`: a `Base` pixel layer
plus a signed layer whose record carries
`TaggedBlock(Tag.EFFECTS_LAYER, EffectsLayer({...}))` with a `CommonStateInfo`,
a `dsdw` `ShadowInfo` and an `oglw` `OuterGlowInfo`, authored through psd-tools'
low-level classes and a 16-bit `Color`. Register `"legacy_effects.psd"`.
Regenerating must leave every existing fixture byte-identical (`git status`).

The codec oracle (`tests/oracle.rs` + a `tests/oracle/legacy.rs` declared with
`#[path]`) asserts the `lrFX` key survives in `extra_blocks`, the whole
`Document` round-trips `write_psd`/`read_psd`, and a self-skipping psd-tools
check re-reads the fixture as an `EffectsLayer` with the authored shadow and glow
values. Render tests load the fixture with `include_bytes!`, assert
`decode_legacy_effects` yields the authored params, and composite a result that
differs from the no-effect composite.

### D10. No app change

The canvas composites through `pictura_render::composite_rgba` /
`composite_active`; a decoded legacy effect renders there with no command, panel
or `CMakeLists.txt` change, and no C++ self-test check. No new dependency.

## Risks / Trade-offs

- **`pfx` blur width is ambiguous in the wild** (psd-tools reads `blur` as a
  `u32`; libpsd reads a `u16` + a spare `u16`). The total body size is identical
  either way, and this design follows psd-tools (the fixture's author/reader).
  Mitigation: the fixture pins the layout; a future CS-era capture can confirm.
- **Legacy bevel only renders `Inner`.** A legacy bevel in another style is a
  no-op. Mitigation: the existing renderer's ceiling is documented; the fixture
  does not require a non-Inner bevel to render.
- **Opacity is rescaled 0..=255 → percent.** A crafted byte lands on a
  fractional percent. Mitigation: the spec asserts a byte of 255 (100 %) for the
  equivalence scenario, so no rounding drift enters the composite assertion.
- **16→8-bit colour truncation.** `value >> 8` is lossy for a value that is not
  a multiple of `256`. Mitigation: the fixture authors `v << 8`; the mapping is
  stated.
- **Block-level precedence discards a legacy effect when `lfx2` is present but
  omits it.** Mitigation: this is the intended semantics (the newer block is
  authoritative), and it is the only rule that cannot double-apply; the spec
  states it.
- **Untrusted input.** All reads are bounds-checked and all arithmetic is total;
  a truncated or oversized block yields `None`, never a panic.

## Migration Plan

Not applicable: additive read support behind the existing `composite_rgba` /
`composite_active` path. No data migration, no flag, no rollback step beyond
reverting the change. The new golden fixture is regenerated with the existing
script.

## Open Questions

- Whether Photoshop ever writes `lrFX` and `lfx2` with disjoint effect sets in
  one file; this design treats `lfx2` as complete.
- The exact legacy `bevel_style` byte values (grounded from libpsd's
  `psd_bevel_*` order); a CS6-authored legacy bevel would settle 0 vs 1 for
  `Outer`/`Inner`.
- Whether the legacy `intensity` for a shadow is ever meaningfully different
  from the spread; libpsd treats the slot as `spread`, and this design follows.
