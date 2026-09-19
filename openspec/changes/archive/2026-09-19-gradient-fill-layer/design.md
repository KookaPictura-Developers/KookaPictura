## Context

A Photoshop gradient fill layer stores a version-16 `DescriptorBlock` under the
`GdFl` additional-layer-info key. `pictura-codec` preserves that block verbatim
but does not list `GdFl` in `ADJUSTMENT_KEYS`, so it is not routed to
`layer.adjustment`; `pictura-render`'s `decode_adjustment` has no `GdFl` arm; and
the app's `Layer > New Fill Layer > Gradient…` leaf is a disabled placeholder
with no command. The result is that a real gradient fill renders as a no-op.

The pieces that exist: `pictura-codec`'s public descriptor DOM
(`read_descriptor`/`write_descriptor`, `Objc`/`VlLs`/`doub`/`enum`/`long`/`TEXT`),
`pictura-adjust`'s `GradientStop { location, color }` and `gradient_map` sampler,
and `pictura-render`'s generative fill composite (`composite_solid_fill`) plus
the fill-content rasterizer. psd-tools 1.19 registers `GdFl` as a
`DescriptorBlock`, exposes `GradientFill.angle` / `.gradient_kind` / `.data`, and
its `composite/paint.py::draw_gradient_fill` is a public, testable reference for
the geometry and sampling.

## Goals / Non-Goals

**Goals:**

- Recognise `GdFl` as an adjustment key and decode a custom-stop descriptor into
  `Adjustment::GradientFill(GradientFillParams)`.
- Composite a gradient fill generatively over the layer rect for all five kinds,
  so a Photoshop gradient fill renders instead of being a no-op.
- Treat a decodable `GdFl` layer as fill content for the Rasterize subset.
- Provide `encode_gradient_fill` and app wiring so the app can author one.
- Add a psd-tools-authored fixture and prove the block survives read,
  whole-`Document` round-trip, and decode.

**Non-Goals:**

- Pattern fill (`PtFl`).
- Noise gradients (`ClNs`), transparency/opacity stops, stop midpoint, and
  non-linear interpolation modes.
- Non-RGB colour models.
- A `GradientFill` GPU shader; documents with one keep falling back to CPU.
- `curv`, `mixr`, `clrL`, `selc`, and a version-3 `phfl`.

## Decisions

### D1. Reuse `GradientStop`; add `GradientKind` and `GradientFillParams`

`GradientFillParams { stops: Vec<GradientStop>, reverse: bool, kind:
GradientKind, angle_deg: f32, scale: f32 }` and `GradientKind { Linear,
Radial, Angle, Reflected, Diamond }` live in `pictura-adjust::types`, next to
`GradientMapParams`, and reuse the existing `GradientStop`. A fill is not a
destructive adjustment, so `apply` returns `AdjustError::Unsupported` exactly as
it does for `SolidFill`; the variant is otherwise only a carrier for the
generative composite. This mirrors the `SolidFill` precedent instead of inventing
a parallel fill model in `pictura-render`.

### D2. Optionally read `Rvrs` and `Scl `

psd-tools' own `draw_gradient_fill` reads `Key.Reverse` (`Rvrs`) and `Key.Scale`
(`Scl `) from the top-level `GdFl` descriptor with defaults `false` / `100`. Our
decoder reads them when present and otherwise defaults the same way, so `reverse`
and `scale` in `GradientFillParams` are populated from a real file. The verified
authoring recipe does not write them, so a psd-tools fixture uses the defaults.
The encoder writes only the grounded `Angl`/`Type`/`Grad` keys (see D6); a
`ponytail:` ceiling records that an authored fill is forward at scale 100.

### D3. Decode is strict and never panics

`decode_gradient_fill` parses with `pictura_codec::read_descriptor`, requires an
object, reads `Angl` as a finite `Double`, reads the `Type` `Enum` kind, and
requires the `Grad` item to be an object. It requires `GrdF` to be `CstS` (a
`ClNs` noise gradient is `None`), reads `Clrs` as a list of at least two stop
objects, and for each stop reads the `Clr ` `RGBC` object's `Rd  `/`Grn `/`Bl  `
finite `Double`s (rounded and clamped to `0..=255`) and `Lctn` as a `Double` or
`Long` in `0..=4096`. Locations must strictly increase in stored order (the same
invariant `gradient_map` enforces). Any missing key, wrong type, non-finite
value, colour-noise form, or too-few/non-monotone stop returns `None`. This
matches the existing decoders' posture and keeps a corrupt file a no-op rather
than a panic or a wrong render.

Note: `ADJUSTMENT_KEYS` must gain `GdFl` (19→20) first, or the codec never puts
the block in `layer.adjustment` and no decoder can see it. This is the one codec
change the earlier `SoCo` precedent did not need, because `SoCo` was already
listed.

### D4. Generative composite mirrors psd-tools' geometry

`composite_adjustment` routes `Adjustment::GradientFill` to
`composite_gradient_fill`, like `SolidFill` routes to `composite_solid_fill`:
it generates a colour for every pixel inside the layer rect (clamped to the
canvas) and calls `blend_into`, so the layer's mask, opacity, fill, and blend
still apply and pixels outside the rect are untouched.

For each pixel `(x, y)` in the rect of size `w × h`, following
`psd_tools/composite/paint.py`:

- Normalize `X ∈ [-w/s, w/s]` across columns and `Y ∈ [-h/s, h/s]` across rows,
  where `s = (scale/100) × (((90 − (angle mod 90))/90)·w + ((angle mod 90)/90)·h)`.
- Compute the index `Z` for the kind: linear `0.5(cos θ · X − sin θ · Y + 1)`,
  radial `√(X² + Y²)`, angle `((180·atan2(Y, X)/π) + angle·180/π mod 360)/360`,
  reflected `|cos θ · X − sin θ · Y|`, diamond
  `|cos θ · X − sin θ · Y| + |sin θ · X + cos θ · Y|`.
- Clamp `Z` to `0..=1`, then `Z ← 1 − Z` when `reverse`.
- Sample the stop list at `Z × 4096` with the gradient-map sampler (linear
  between bracketing stops, clamped outside; stop colours scaled `× 257` on
  decode).

Output alpha is 255 (opaque); the transparency/opacity stops are a recorded
ceiling.

### D5. One fill-content predicate

`is_fill_content_layer` becomes `decode_adjustment(..)` matching
`Some(Adjustment::SolidFill(_) | Adjustment::GradientFill(_))`, and
`rasterize_fill_content` bakes the decoded content with the same decoder. A
solid fill bakes its RGBA as before; a gradient fill generates the five-kind
ramp over the layer rect and bakes the RGB with alpha 255. Routing both through
`decode_adjustment` means a future fill spelling is recognised in one place and
cannot disagree between composite and rasterize. The behavior requirement lives
in `layer-management` (which owns `Rasterize subset`).

### D6. Encoder builds the standard descriptor

`encode_gradient_fill(kind: GradientKind, stops: &[GradientStop], angle_deg:
f32) -> AdjustmentData` builds a version-16 `GdFl` object: `Angl` `Double(angle)`,
`Type` `Enum(b"GrdT", kind_enum)`, and `Grad` an object of class `Grdn` carrying
`Nm  ` text, `GrdF` `Enum(b"GrdF", b"CstS")`, `Intr` `Enum(b"Intp", b"Lnr ")`,
and a `Clrs` list whose entries are `Objc`/`RGBC` stop objects with `Clr `,
`Typ `, `Lctn` `Double`, and `Mdpn` `Double(50)`. It is serialized with
`pictura_codec::write_descriptor`. This is exactly the shape psd-tools reads and
the verified authoring recipe produces. `reverse`/`scale` are not written (D2).

### D7. The app authors a fixed default

`pictura-render::add_gradient_fill(doc, selection_path)` inserts a document-sized
`GdFl` layer over `encode_gradient_fill(GradientKind::Linear, &[black@0,
white@4096], 0.0)`, named `"Gradient Fill N"`, mirroring `add_solid_fill`. The
bridge method `PictureView::add_gradient_fill` recomposites and records one
state. `Layer > New Fill Layer > Gradient…` becomes an enabled command
(`LayerNewFillGradient`) and the Layers-panel fill menu gains a `Gradient…`
action; both call the bridge. Keeping the default fixed means no
gradient-editor/config surface is invented (YAGNI).

### D8. The fixture and its oracle

`scripts/generate-fixtures.py` gains a `gradient_fill()` builder using psd-tools'
`DescriptorBlock`/`Descriptor`/`Enumerated`/`Double`/`List`/`String`:

```python
DescriptorBlock(Descriptor({
    b"Angl": Double(0.0),
    b"Type": Enumerated(Type.GradientType, Enum.Linear),
    b"Grad": Descriptor({
        b"Nm  ": String("Black to White"),
        b"GrdF": Enumerated(Type.GradientForm, Enum.CustomStops),
        b"Intr": Enumerated(Type.Interpolation, b"Lnr "),
        b"Clrs": List([stop(0, (0, 0, 0)), stop(4096, (255, 255, 255))]),
    }, classID=b"Grdn"),
}, classID=b"GdFl"))
```

psd-tools' `DescriptorBlock` keeps the tagged-block key `GdFl` but serializes the
outer descriptor class as `null`, so the fixture's top-level classID is `null`,
not `GdFl`; this is the same latent quirk `solid_fill.psd` carries, and the codec
dispatches on the key and re-emits the bytes verbatim. Attached under
`Tag.GRADIENT_FILL_SETTING`. Unlike the adjustment helpers, the
fill layer keeps a document-sized rect (`top/left 0`, `bottom/right 8`) with
`channel_info = []` and no channels, so psd-tools' `composite()` actually paints
the ramp (a black→white 8-pixel row `0,36,72,109,145,182,218,255`). The codec
oracle asserts the `GdFl` key and that the descriptor's `Type`, `Angl`, and stop
components survive read and whole-`Document` round-trip; the render test asserts
`decode_adjustment` yields the expected params; and, where practical, the
rasterizer is compared against psd-tools' `composite()` ramp. `pictura-codec`
cannot call `decode_adjustment` (it would invert the crate dependency), so the
decode assertion lives in `pictura-render`.

## Risks / Trade-offs

- **`GdFl` was never in `ADJUSTMENT_KEYS`.** Without the codec change the fixture
  block never reaches `layer.adjustment`. Mitigation: D3/D8 and the codec task
  call it out explicitly; the fixture oracle fails loudly if it is missed.
- **`scale`/`reverse` keys are optional.** A file that stores the fill's reverse
  by permuting stops rather than via `Rvrs` would decode as forward. Mitigation:
  the defaults match psd-tools; the fields are carried and honored when present.
- **Stop midpoint and interpolation are ignored.** A gradient with a non-50
  midpoint or a non-linear mode renders slightly differently. This is a stated
  ceiling; the fixture uses `Mdpn 50` / `Lnr `.
- **A noise gradient is a no-op.** `ClNs` returns `None`, so the layer is
  preserved but unrendered rather than misrendered.
- **Opaque fills only.** Transparency stops are not modelled; a fill that should
  fade cannot. Recorded as a ceiling.
- **A wrong-but-parseable descriptor renders a wrong gradient.** The type checks
  and the two-stop monotonicity bound the damage; the fixture pins the known
  values.
- **The committed golden fixture is new, not modified.** No existing fixture
  changes, so regeneration cannot churn the existing oracle.
- **A GPU document with a gradient fill decodes further before rejection.** It
  now decodes to `GradientFill` and is still rejected by `adjustment_params` (no
  shader), so the CPU fallback outcome is unchanged.

## Open Questions

- Whether CS6 ever writes a `GdFl` without `Grad`/`Clrs`. The file-format
  excerpt and psd-tools both assume it does not; a CS6 capture would settle it.
- Whether CS6 stores the fill's Reverse as `Rvrs` or by reversing the stop list.
  psd-tools reads `Rvrs`; the encoder writes neither, so the question only
  affects decode of a real file.
