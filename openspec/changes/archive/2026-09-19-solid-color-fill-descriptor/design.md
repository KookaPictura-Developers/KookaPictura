## Context

`pictura-codec` preserves the `SoCo` additional-layer-info block verbatim, and
`pictura-adjust` already implements `Adjustment::SolidFill([u8; 4])`. Two things
keep a real Photoshop solid-color fill layer from rendering: `decode_adjustment`
recognises only the in-house 4-byte `[r, g, b, a]` payload, and the app authors
only that 4-byte form. Photoshop instead writes a version-16 `DescriptorBlock`
whose `Clr ` object of class `RGBC` carries `Rd  ` / `Grn ` / `Bl  ` `doub`
components on the 0-255 scale. psd-tools 1.19 registers `SoCo` as a
`DescriptorBlock` and exposes `SolidColorFill.data` returning that `Clr `
object, and `pictura_codec::read_descriptor` already parses the block, so both
the decoder and an independent oracle are in place.

## Goals / Non-Goals

**Goals:**

- Decode a descriptor-form `SoCo` payload into
  `Adjustment::SolidFill([r, g, b, 255])`, so a Photoshop-authored fill layer
  renders through the existing solid-fill composite path.
- Keep the 4-byte in-house form readable, so documents we already wrote do not
  regress.
- Make `is_fill_content_layer` accept both forms, so `Rasterize Fill Content`
  works on a Photoshop-authored fill.
- Provide `encode_solid_color_fill` so the app authors the standard descriptor.
- Add a psd-tools-authored `solid_fill.psd` fixture and prove the block survives
  read, whole-`Document` round-trip, and decode.

**Non-Goals:**

- Gradient (`GdFl`) and pattern (`PtFl`) fills.
- `curv`, `mixr`, `clrL`, `selc`, and a version-3 `phfl`.
- Colour-space conversion. The `Clr ` object is always `RGBC`.
- Non-opaque solid fills. The standard descriptor has no alpha.
- A `SolidFill` GPU shader. Documents with one keep falling back to the CPU
  path, as today.

## Decisions

### D1. The descriptor is read through the shared DOM

`read_descriptor` returns the version-16 block as a `DescValue::Object` with a
`Clr ` item that is itself an `Object` carrying `DescValue::Double` components.
`decode_solid_fill` requires the top level to be an object, the `Clr ` item to
be an object, and each of `Rd  ` / `Grn ` / `Bl  ` to be a present, finite
`Double`. Each component is rounded and clamped to `0..=255`; alpha is 255. The
`RGBC` class id is not enforced — the three keys are the contract — so a
slightly different but structurally valid producer still decodes.

### D2. Prefer the 4-byte arm, then the descriptor

The `SoCo` arm distinguishes the forms by length first: exactly four bytes is the
in-house `[r, g, b, a]` tuple, and anything longer is tried as a version-16
descriptor. Every descriptor also begins with `00 00 00 10`, so a corrupt
exactly-four-byte payload is spec-legal as the tuple `SolidFill([0, 0, 0, 16])`;
length is the discriminator, not the bytes. `decode_solid_fill` returning `None`
still means "not understood" and leaves the backdrop unchanged. This preserves
the existing no-op contract for corrupt bytes.

### D3. Malformed payloads are `None`, never a panic

A payload that does not parse as a descriptor, lacks `Clr `, lacks a component,
or carries a non-`Double` or non-finite component returns `None`. The reader is
already bounded (nesting cap, checked lengths), so truncation surfaces as an
`Err` that becomes `None`. This matches the existing decoders' range-check
posture.

### D4. `encode_solid_color_fill` takes `[u8; 3]`

The signature mirrors `encode_photo_filter(color: [u8; 3], ...)` and the
decoded RGB (minus alpha). The standard descriptor is RGB-only, so a three-byte
parameter makes the dropped alpha explicit at the single app call site rather
than hiding it behind a four-byte tuple that quietly ignores its last element.
`add_solid_fill` keeps its `[u8; 4]` signature (its bridge callers pass an
`0xAARRGGBB` value) and passes the first three bytes to the encoder with a
`ponytail:` note naming the alpha ceiling.

### D5. One fill-content predicate, in `layer-management`

`is_fill_content_layer` becomes `adjustment key == SoCo && decode_adjustment(..)
== Some(Adjustment::SolidFill(..))`, and `rasterize_fill_content` bakes the RGBA
that same decode returned. Routing both through `decode_adjustment` means a
future `SoCo` spelling is recognised in one place and cannot disagree between
composite and rasterize. The behavior requirement lives in `layer-management`
(which owns the `Rasterize subset` requirement), not
`adjustment-layer-rendering`.

### D6. The fixture and its oracle

`scripts/generate-fixtures.py` gains a `solid_fill()` builder using psd-tools'
`DescriptorBlock`:

```python
DescriptorBlock(
    Descriptor(
        {b"Clr ": Descriptor(
            {b"Rd  ": Double(10.0), b"Grn ": Double(20.0), b"Bl  ": Double(30.0)},
            classID=b"RGBC")},
        classID=b"SoCo"),
)
```

attached under `Tag.SOLID_COLOR_SHEET_SETTING` on a channel-stripped pixel
layer, matching the existing `_adj_layer` recipe. psd-tools' `DescriptorBlock`
takes the passed `Descriptor` as its item map and keeps its own default class id,
so the fixture's tagged-block key is `SoCo` but its serialized top-level
descriptor class is `null` (`6e756c6c`). Our own encoder writes `SoCo` as the
top-level class; psd-tools dispatches on the tag and accepts either. The codec
oracle asserts the `SoCo` key, that the payload is a version-16 descriptor whose
`Clr ` `RGBC` object's components are `Double(10.0/20.0/30.0)` (the value
psd-tools'
`SolidColorFill.data` reports), and that the whole `Document` round-trips. The
render test loads the same fixture with `include_bytes!` and asserts
`decode_adjustment` yields `SolidFill([10, 20, 30, 255])`. `pictura-codec`
cannot call `decode_adjustment` (that would invert the crate dependency), so the
decode assertion lives in `pictura-render`.

### D7. Authoring switches to the descriptor

`add_solid_fill` builds `AdjustmentData` from `encode_solid_color_fill`, not the
raw four bytes. New files are Photoshop-readable. Existing files with the 4-byte
form still decode (D2). Because the descriptor has no alpha, an authored fill is
opaque; the app callers all pass `0xFF` alpha, so no visible behavior changes.
The `rasterize_bakes_color_and_clears_fill` unit test, which used a non-opaque
alpha to exercise the in-house form, is updated to opaque and a new case covers
the descriptor form's alpha 255.

## Risks / Trade-offs

- **Alpha is dropped on authoring.** A non-opaque solid fill cannot be written
  as a standard descriptor. Mitigation: all app callers pass opaque (`0xFF`);
  the encoder's three-byte signature makes the loss explicit; the 4-byte reader
  still supports legacy files.
- **The decoded colour is assumed sRGB.** The `Clr ` object is always `RGBC`,
  so a non-RGB document's fill is decoded as if sRGB rather than converted. This
  is recorded in the proposal; a colour-managed fill is future work.
- **A wrong-but-parseable descriptor renders a wrong colour** instead of a
  no-op. The component checks (`Double`, finite, clamped) bound the damage; the
  fixture pins the known value.
- **The committed golden fixture is new, not modified.** No existing fixture
  changes, so the regeneration cannot churn the existing oracle.
- **A GPU document with a solid fill decodes further before rejection.** It now
  decodes to `SolidFill` and is still rejected by `adjustment_params` (no
  shader), so the fallback outcome is unchanged.

## Open Questions

- Whether a real CS6 solid fill ever writes a colour space other than RGB. The
  file-format excerpt and psd-tools both say RGB-only; a CS6 capture would
  settle it.
