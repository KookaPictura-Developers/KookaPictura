## ADDED Requirements

### Requirement: The legacy effects block decodes into the typed effect set

`pictura-render` SHALL derive the existing typed effect params
(`DropShadow`, `InnerShadow`, `OuterGlow`, `InnerGlow`, `BevelEmboss` and
`ColorOverlay`) from a layer's legacy `lrFX` (`EFFECTS_LAYER`) tagged block via
`decode_legacy_effects(&Layer) -> Option<LegacyEffects>`. The block SHALL be
parsed as the fixed `EffectsLayer` binary struct — a `u16` version (0) and a
`u16` count, then `count` records each beginning `8BIM` followed by a 4-byte
`ostype` (`cmnS`, `dsdw`, `isdw`, `oglw`, `iglw`, `bevl`, `sofi`) and a `u32`
body length — with all integers big-endian and no alignment padding. The decoder
SHALL read the shared `CommonStateInfo` (`cmnS`: `u32` version, `u8` visible) and
AND its `visible` flag into every record's `enabled` (`visible` defaults to true
when `cmnS` is absent). Every present record SHALL decode with `present` true,
because a record exists only when written.

The decoder SHALL map the fields `lrFX` carries and take the typed effect
defaults for the `lfx2`-only fields it does not:

- shadow records (`dsdw`/`isdw`): `blend_mode` from the stored 4-byte layer
  blend key through `BlendMode::from_psd_key` (not the capitalized `BlnM`
  vocabulary), `color` from the 16-bit `Color` channels (`value >> 8`),
  `opacity` from the `u8` scaled `0..=255 → 0..=100`, `angle` as `angle_deg`,
  `distance`, `blur` as `size`, `intensity` as `spread` (drop) or `choke`
  (inner), and `use_global_angle`; a shadow's `knocks_out` SHALL be false and
  `present` true;
- glow records (`oglw`/`iglw`): `blend_mode`, `color`, `opacity`, `blur` as
  `size` and `intensity` as `spread` (outer) or `choke` (inner), with `technique`
  `Softer`; an inner glow's `source` SHALL be `Center` when a version-2 `invert`
  byte is non-zero and `Edge` otherwise;
- bevel (`bevl`): `angle` as `angle_deg`, `depth`, `blur` as `size`,
  `soften` 0, `altitude_deg` 30, `use_global_angle`, the `bevel_style` byte as
  the `BevelStyle` (0 `Outer`, 1 `Inner`, 2 `Emboss`, 3 `Pillow`, 4 `Stroke`),
  the `direction` byte as the `BevelDirection`, `technique` `Smooth`, and both
  halves' blend mode, colour and opacity;
- solid fill (`sofi`): `blend_mode`, `color` and `opacity`; a `sofi` whose
  version is not 2 SHALL be skipped.

A finite value outside its typed range SHALL be clamped to the same bounds the
shipped `lfx2` decoders use (`opacity` `0..=100`, `distance` `0..=30000`, `size`
and `soften` `0..=250`, `choke`/`spread` `0..=100`, bevel `depth` `0..=1000` and
`altitude_deg` `0..=90`). A missing `lrFX` block, an unsupported `EffectsLayer`
version, a truncated body, a record whose signature is not `8BIM`, or a count
that overruns the payload SHALL return `None`; an individually unknown or
malformed record SHALL be skipped without failing the block. The decoder SHALL
NOT panic.

#### Scenario: A legacy drop shadow and outer glow decode to typed parameters

- **WHEN** an `lrFX` block contains `cmnS` visible 1, a `dsdw` with `blur` 6, `intensity` 10, `angle` 45, `distance` 8, a non-black `Color`, blend key `mul `, enabled 1, use-global-angle 0 and opacity 255, and an `oglw` with `blur` 10, `intensity` 20, a non-white `Color`, blend key `scrn`, enabled 1 and opacity 128
- **THEN** `decode_legacy_effects` returns a `LegacyEffects` whose `drop_shadow` has `enabled` true, `present` true, `blend_mode` `Multiply`, the stored colour, `opacity` 100, `angle_deg` 45, `distance` 8, `spread` 10 and `size` 6, and whose `outer_glow` has `enabled` true, `present` true, `blend_mode` `Screen`, the stored colour, `spread` 20, `size` 10, `technique` `Softer` and an opacity equal to `128 · 100 / 255`

#### Scenario: A legacy inner shadow and inner glow decode with their choke and source

- **WHEN** an `lrFX` block contains an `isdw` with `intensity` 15 and an `iglw` version 2 with a non-zero `invert` byte and `intensity` 25
- **THEN** the `inner_shadow` has `choke` 15 and the `inner_glow` has `choke` 25, `source` `Center` and `technique` `Softer`
- **AND WHEN** the `iglw` has a zero `invert` byte
- **THEN** its `source` is `Edge`

#### Scenario: A legacy bevel and solid fill decode to typed parameters

- **WHEN** an `lrFX` block contains a `bevl` with `angle` 120, `depth` 100, `blur` 5, `bevel_style` 1, `direction` 0, highlight and shadow blend keys, colours and opacities, and enabled 1, and a version-2 `sofi` with a blend key, colour, opacity 191 and enabled 1
- **THEN** the `bevel` has `style` `Inner`, `technique` `Smooth`, `direction` `Up`, `size` 5, `depth` 100, `altitude_deg` 30 and `soften` 0, and the `color_overlay` has the stored blend mode, colour and an opacity equal to `191 · 100 / 255`

#### Scenario: The common state gates every legacy effect

- **WHEN** an `lrFX` block has a `cmnS` whose `visible` is 0 and a `dsdw` whose own enabled byte is 1
- **THEN** the decoded `drop_shadow` has `enabled` false, and compositing the layer leaves the output byte-identical to the same document without the effect
- **AND WHEN** the block has no `cmnS` record
- **THEN** the record's own enabled byte alone decides `enabled`

#### Scenario: Fields lrFX does not carry take the effect defaults

- **WHEN** a legacy drop shadow, outer glow, inner glow and bevel are decoded
- **THEN** the drop shadow takes the `lfx2` default blend mode and colour for an absent key, the glows take `technique` `Softer`, the inner glow defaults `source` to `Edge`, and the bevel takes `technique` `Smooth`, `soften` 0 and `altitude_deg` 30

#### Scenario: A malformed block is a no-op and never panics

- **WHEN** a layer has no `lrFX` block, or its `lrFX` block has an unsupported version, a truncated record body, a record signature other than `8BIM`, or a count that overruns the payload
- **THEN** `decode_legacy_effects` returns `None` and does not panic
- **AND WHEN** a block contains an unknown `ostype` record, a `sofi` whose version is not 2, or a `Color` with an unknown space alongside a valid `dsdw`
- **THEN** the unknown or malformed record is skipped and the valid `dsdw` still decodes

### Requirement: The compositor resolves lfx2 over the legacy effects block

The decode path the compositor uses SHALL be a single resolver,
`decode_layer_effects(&Layer) -> LayerEffects`, which SHALL be authoritative over
both effect encodings. When a layer has an `lfx2` block, the resolver SHALL read
every effect from `lfx2` and SHALL ignore the `lrFX` block entirely; when the
layer has no `lfx2` block, the resolver SHALL derive the mapped set from `lrFX`
and SHALL leave the effects with no legacy record (`satin`, `stroke`, gradient
overlay, pattern overlay) absent. The resolver SHALL NOT combine the two encodings
and SHALL NOT apply the same effect twice. Both compositor passes
(`composite_layer_effects` and `composite_layer_effects_above`) SHALL consume the
resolved set, with the same group and destructive-adjustment skip as today.

#### Scenario: lfx2 wins when both blocks are present

- **WHEN** a layer carries an `lfx2` block whose `DrSh` is enabled and present and a legacy `lrFX` block whose `dsdw` describes a different shadow
- **THEN** the layer's resolved drop shadow is the `lfx2` one, and the composite is byte-identical to the same document with the `lrFX` block removed

#### Scenario: The legacy block is used when lfx2 is absent

- **WHEN** a layer carries only a legacy `lrFX` block with an enabled and present `dsdw`
- **THEN** the resolved drop shadow is the decoded legacy shadow and the layer composite differs from the same document without the effect

#### Scenario: An lfx2 block that omits an effect suppresses its legacy record

- **WHEN** a layer carries an `lfx2` block with an outer glow but no `DrSh`, and a legacy `lrFX` block with a `dsdw`
- **THEN** the resolved set has no drop shadow and only the outer glow renders

#### Scenario: The legacy block supplies only the effects it maps

- **WHEN** a layer carries only a legacy `lrFX` block with a `dsdw` and no other record
- **THEN** the resolved `satin`, `stroke`, gradient overlay and pattern overlay are absent and no effect other than the drop shadow renders

### Requirement: A legacy effect renders through the shipped layer-effect renderers

An enabled and present effect resolved from a legacy `lrFX` block SHALL render
through the same shipped `composite_*` functions an equivalent `lfx2` effect
uses, with no change to those functions: a drop shadow and outer glow composite
below the layer content, and an inner shadow, inner glow, bevel and color overlay
composite above it. Fields the legacy record carries SHALL drive the render, and
a legacy effect SHALL composite identically to an `lfx2` effect whose typed
params are equal for the render-affecting fields. A resolved effect that maps to
a renderer the shipped code declines (for example a bevel whose style is not
`Inner`, or a decoded glow whose size and offsets make it a no-op) SHALL leave
the composite byte-identical to the same document without the effect. A disabled
or not-present resolved effect, an effect absent from both blocks, or a legacy
extension with no record SHALL be a no-op. The composite SHALL be clipped to the
canvas, SHALL NOT panic, and SHALL NOT add a dependency.

#### Scenario: A legacy drop shadow renders like the equivalent lfx2 shadow

- **WHEN** two documents carry an identically masked content layer, one with an `lfx2` `DrSh` and one with a legacy `lrFX` `dsdw` whose decoded `DropShadow` has the same blend mode, colour, opacity, angle, distance, spread and size
- **THEN** the two composites are byte-identical

#### Scenario: A legacy outer glow renders around the content

- **WHEN** an opaque square layer over an opaque backdrop has a legacy `oglw` with a non-zero `size` and a non-white colour
- **THEN** backdrop pixels adjacent to the square are tinted toward the glow colour and pixels beyond the glow's reach are byte-identical to the same document without the effect

#### Scenario: A legacy interior effect renders above the content

- **WHEN** an opaque square layer has a legacy `isdw`, `iglw` or version-2 `sofi`
- **THEN** at least one pixel inside the square's coverage changes toward the effect colour when compared with the same document without the effect, and every pixel outside the coverage is byte-identical

#### Scenario: An unmapped or declined legacy effect is a no-op

- **WHEN** a layer carries a legacy `bevl` whose decoded style is `Outer`, or a `sofi` whose version is not 2
- **THEN** no effect renders and the composite is byte-identical to the same document without the legacy block

#### Scenario: A disabled legacy effect is a no-op

- **WHEN** a layer's only legacy effect record has an enabled byte of 0, or the block's common state is not visible
- **THEN** the composite is byte-identical to the same document without the legacy block
