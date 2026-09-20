## MODIFIED Requirements

### Requirement: Committed adjustment payloads decode to typed parameters

`decode_adjustment` SHALL decode the committed PSD adjustment payloads into the
matching `pictura_adjust::Adjustment` variant: `expA` to
`Adjustment::Exposure(ExposureParams)`, `vibA` to
`Adjustment::Vibrance(VibranceParams)`, `blwh` to
`Adjustment::BlackWhite(BlackWhiteParams)`, `phfl` to
`Adjustment::PhotoFilter(PhotoFilterParams)`, `grdm` to
`Adjustment::GradientMap(GradientMapParams)`, `blnc` to
`Adjustment::ColorBalance(ColorBalanceParams)`, `mixr` to
`Adjustment::ChannelMixer(ChannelMixerParams)`, `curv` to
`Adjustment::Curves(CurvesParams)`, `SoCo` to
`Adjustment::SolidFill([u8; 4])`, `GdFl` to
`Adjustment::GradientFill(GradientFillParams)`, and `PtFl` to
`Adjustment::PatternFill(PatternFillParams)`. The fixed structs `expA` and
`phfl` SHALL be read as a `u16` version followed by fixed-width fields. The
descriptor payloads `vibA` and `blwh` SHALL be read as a 4-byte descriptor
version equal to 16 followed by a descriptor object whose keys carry the slider
values. A version-2 `phfl` payload SHALL be read as a `u16` colour space, four
`u16` colour components, a `u32` density, and a `u8` luminosity flag; the first
three components SHALL be the filter colour, each in `0..=255`, and the fourth
and the colour space SHALL be ignored. A `grdm` payload SHALL be read as a `u16`
version (`1` or `3`, a `4`-byte method following the two flag bytes when the
version is 3), a `u8` reverse flag, a `u8` dither flag, a unicode gradient name,
and a colour-stop list; each stop SHALL be read as a `u32` location, a `u32`
midpoint, a `u16` mode, and four `u16` colour components whose first three are
reduced from the 16-bit scale to the stop's RGB colour, and the midpoint, mode,
fourth component, and every field after the colour stops SHALL be ignored. A
`blnc` payload SHALL be read as nine big-endian `i16` shifts in the order
shadows, midtones, highlights (three per band, each in `-100..=100`) followed by
a `u8` luminosity flag, and every byte after the flag SHALL be ignored; a
truncated payload or a shift outside `-100..=100` SHALL decode to `None`. A
`mixr` payload SHALL be read as a `u16` version equal to `1`, a `u16`
monochrome flag, then the payload's channels: the `red`, `green`, and `blue`
channels when the flag is clear, and the `gray` channel always. Every channel
SHALL be read as three big-endian `i16` source percentages, two reserved bytes,
and one big-endian `i16` constant percentage, and every byte after the last
channel SHALL be ignored. A non-monochrome payload SHALL decode to
`Adjustment::ChannelMixer` whose `monochrome` is false, whose `red`, `green`,
and `blue` rows are the three channels' source triples, and whose `constant` is
their constants; a monochrome payload SHALL decode with `monochrome` true,
`red` the `gray` source triple, and `constant[0]` the `gray` constant. A `mixr`
payload whose version is not 1, that is shorter than the channels it declares,
or that carries a source percentage or constant outside `-200..=200` SHALL
decode to `None`. A `curv` payload SHALL be read as a `u8` ignored byte, a `u16`
version equal to `1`, a `u16` ignored word, and a `u16` channel bitmask (`1`
rgb, `2` red, `4` green, `8` blue); for each set bit, in the order rgb, red,
green, blue, a `u16` node count followed by that many `(i16 output, i16 input)`
pairs. A bitmask equal to zero or carrying a bit outside `0b1111`, a nonzero lead
`is_map` byte, a node count outside `2..=14`, a coordinate outside `0..=255`, or
non-strictly-increasing inputs SHALL decode to `None`. The `2..=14` bound is the
engine-op contract; psd-tools additionally permits up to 19, so a 15–19-point
legacy file SHALL decode to a no-op. The version-4 duplicate `Crv ` section and
every trailing byte SHALL be ignored. A `curv` payload SHALL decode to
`Adjustment::Curves` whose `points` is the rgb channel's points converted from
`(output, input)` to `(input, output)` order and whose `red`, `green`, and
`blue` are the corresponding channels when present; when the rgb bit is clear,
`points` SHALL be the identity endpoints `[(0, 0), (255, 255)]` and only the
per-channel curves SHALL be present. A `SoCo` payload SHALL be read in one of
two forms: a 4-byte straight-alpha RGBA tuple, or a version-16 descriptor whose
`Clr ` object carries `Rd  `, `Grn `, and `Bl  ` `doub` values on the `0..=255`
scale. The descriptor form SHALL decode to `Adjustment::SolidFill([r, g, b,
255])`, rounding and clamping each component to `0..=255` and forcing alpha to
255; the 4-byte form SHALL keep its four stored components. A `GdFl` payload
SHALL be read as a version-16 descriptor whose `Angl` `doub` is the angle in
degrees, whose `Type` enum (typeID `GrdT`) is the gradient kind (`Lnr ` Linear,
`Rdl ` Radial, `Angl` Angle, `Rflc` Reflected, `Dmnd` Diamond), and whose `Grad`
object (class `Grdn`) carries the gradient stops. The `Grad` object's `GrdF`
enum (typeID `GrdF`) SHALL be the custom-stops form (`CstS`); a colour-noise
form (`ClNs`) SHALL NOT decode. Each stop in the `Clrs` list SHALL be an object
whose `Clr ` `RGBC` object carries `Rd  `, `Grn `, and `Bl  ` finite `doub`
values on the `0..=255` scale and whose `Lctn` value is a `doub` or `long` in
`0..=4096`. Stops SHALL be kept in stored order. When present, the top-level
`Rvrs` `bool` SHALL be the reverse flag and the `Scl ` `doub` SHALL be the scale
percent; absent, they SHALL default to false and 100. A `PtFl` payload SHALL be
read as a version-16 descriptor whose `Ptrn` object (class `Ptrn`) carries the
pattern id in its `Idnt` text and whose top-level `Scl ` value (`doub` or unit
float) is the scale percent and whose `Algn` `bool` is the Link With Layer flag;
a missing or wrong-typed `Ptrn`/`Idnt`, a non-finite scale, or a malformed
descriptor SHALL decode to `None`. The pattern id SHALL have trailing NUL bytes
stripped; an absent `Scl ` SHALL default to 100 and an absent `Algn` SHALL
default to true.

#### Scenario: Exposure payload decodes to ExposureParams

- **WHEN** an `expA` payload has version 1 and finite exposure, offset, and gamma values
- **THEN** `decode_adjustment` returns `Adjustment::Exposure` carrying those three values

#### Scenario: Vibrance payload decodes to VibranceParams

- **WHEN** a `vibA` descriptor contains the `vibrance` and `Strt` integer keys in range
- **THEN** `decode_adjustment` returns `Adjustment::Vibrance` with those vibrance and saturation values

#### Scenario: Black and white payload decodes to BlackWhiteParams

- **WHEN** a `blwh` descriptor contains the `Rd  `, `Yllw`, `Grn `, `Cyn `, `Bl  `, `Mgnt`, `useTint`, and `tintColor` keys
- **THEN** `decode_adjustment` returns `Adjustment::BlackWhite` with the six channel percentages, the tint flag, and the tint colour

#### Scenario: Photo Filter payload decodes to PhotoFilterParams

- **WHEN** a `phfl` payload has version 2, four colour components whose first three are each `0..=255`, and a density in `0..=100`
- **THEN** `decode_adjustment` returns `Adjustment::PhotoFilter` whose `color` is the first three components, whose `density` is the stored percent, and whose `preserve_luminosity` is the luminosity byte interpreted as a boolean

#### Scenario: Gradient Map payload decodes to GradientMapParams

- **WHEN** a `grdm` payload has version 1 or 3, a two-flag header, a unicode name, and at least two colour stops whose locations strictly increase within `0..=4096`
- **THEN** `decode_adjustment` returns `Adjustment::GradientMap` whose `stops` carry those locations and the 16-bit-reduced RGB colours and whose `reverse` is the reverse flag interpreted as a boolean

#### Scenario: Gradient Map version 3 skips its method

- **WHEN** a `grdm` payload has version 3 and a `4`-byte method before the name
- **THEN** `decode_adjustment` returns `Adjustment::GradientMap` with the same stops as the equivalent version-1 payload

#### Scenario: Color Balance payload decodes to ColorBalanceParams

- **WHEN** a `blnc` payload carries nine `i16` shifts in `-100..=100`, a luminosity byte, and trailing pad bytes
- **THEN** `decode_adjustment` returns `Adjustment::ColorBalance` whose `shadows`, `midtones`, and `highlights` are the three shifts per band in order and whose `preserve_luminosity` is the luminosity byte interpreted as a boolean

#### Scenario: Channel Mixer payload decodes to ChannelMixerParams

- **WHEN** a non-monochrome `mixr` payload carries version 1, the monochrome flag clear, the `red`/`green`/`blue`/`gray` channels in order, and trailing bytes
- **THEN** `decode_adjustment` returns `Adjustment::ChannelMixer` with `monochrome` false, `red`/`green`/`blue` the three RGB channels' source triples, and `constant` their constants, ignoring the `gray` channel and the trailing bytes

#### Scenario: Monochrome Channel Mixer payload decodes to ChannelMixerParams

- **WHEN** a monochrome `mixr` payload carries version 1, the monochrome flag set, and a `gray` channel
- **THEN** `decode_adjustment` returns `Adjustment::ChannelMixer` with `monochrome` true, `red` the `gray` source triple, and `constant[0]` the `gray` constant

#### Scenario: Composite Curves payload decodes to CurvesParams

- **WHEN** a `curv` payload carries version 1, the bitmask `1`, and an rgb node list in `(output, input)` order
- **THEN** `decode_adjustment` returns `Adjustment::Curves` whose `points` is that list in `(input, output)` order and whose `red`, `green`, and `blue` are `None`

#### Scenario: Per-channel Curves payload decodes to CurvesParams

- **WHEN** a `curv` payload sets the rgb, red, green, and blue bits and carries a node list for each
- **THEN** `decode_adjustment` returns `Adjustment::Curves` whose `points` is the rgb curve and whose `red`, `green`, and `blue` are the corresponding per-channel curves

#### Scenario: Curves without a composite channel decodes to per-channel curves

- **WHEN** a `curv` payload sets only the red, green, or blue bits
- **THEN** `decode_adjustment` returns `Adjustment::Curves` whose `points` is the identity `[(0, 0), (255, 255)]` and whose present per-channel curves carry the stored nodes

#### Scenario: Curves duplicate section and trailing bytes are ignored

- **WHEN** a `curv` payload carries the version-4 duplicate `Crv ` section and pad bytes after its last channel
- **THEN** `decode_adjustment` returns the same `Adjustment::Curves` as the payload without that section

#### Scenario: Solid-color fill payload decodes to SolidFill

- **WHEN** a `SoCo` payload is a 4-byte RGBA tuple, or a version-16 descriptor whose `Clr ` object carries `Rd  `/`Grn `/`Bl  ` doubles on the `0..=255` scale
- **THEN** `decode_adjustment` returns `Adjustment::SolidFill` with those components; the descriptor form forces alpha to 255

#### Scenario: Gradient fill payload decodes to GradientFill

- **WHEN** a `GdFl` descriptor has a finite `Angl`, a `Type` kind, and a `Grad` `Grdn` object whose `GrdF` is `CstS` and whose `Clrs` list holds two `RGBC` stops with strictly increasing `Lctn` in `0..=4096`
- **THEN** `decode_adjustment` returns `Adjustment::GradientFill` whose `kind` is the decoded kind, whose `angle_deg` is the `Angl` value, whose `reverse` is false and `scale` is 100 when absent, and whose `stops` carry those locations and colours

#### Scenario: Pattern fill payload decodes to PatternFill

- **WHEN** a `PtFl` descriptor has a `Ptrn` object whose `Idnt` is a pattern id, a finite `Scl `, and an `Algn` flag
- **THEN** `decode_adjustment` returns `Adjustment::PatternFill` whose `pattern_id` is the id with trailing NULs stripped, whose `scale` is the `Scl ` value, and whose `link_with_layer` is the `Algn` value

#### Scenario: Pattern fill defaults are applied

- **WHEN** a `PtFl` descriptor carries a `Ptrn`/`Idnt` but no `Scl ` or `Algn`
- **THEN** the decoded `PatternFill` has `scale` 100 and `link_with_layer` true

#### Scenario: Every gradient fill kind decodes

- **WHEN** the `Type` enum is `Lnr `, `Rdl `, `Angl`, `Rflc`, or `Dmnd`
- **THEN** the decoded `GradientKind` is Linear, Radial, Angle, Reflected, or Diamond respectively

#### Scenario: Malformed gradient fill descriptor is a no-op

- **WHEN** a `GdFl` payload does not parse as a descriptor object, carries a colour-noise `GrdF` (`ClNs`), lacks `Angl`/`Type`/`Grad`/`Clrs`, has fewer than two stops, has non-increasing or out-of-range stop locations, or carries a non-finite angle or colour component
- **THEN** `decode_adjustment` returns `None` and does not panic

#### Scenario: Malformed pattern fill descriptor is a no-op

- **WHEN** a `PtFl` payload does not parse as a descriptor object, lacks a `Ptrn` object, has a missing or wrong-typed `Idnt`, or has a non-finite `Scl `
- **THEN** `decode_adjustment` returns `None` and does not panic

#### Scenario: Malformed exposure payload is a no-op

- **WHEN** an `expA` payload is truncated, has a version other than 1, carries a non-finite value, or has gamma at or below zero
- **THEN** `decode_adjustment` returns `None`

#### Scenario: Malformed vibrance payload is a no-op

- **WHEN** a `vibA` payload is truncated, does not parse as a descriptor object, or carries an out-of-range or wrong-typed key
- **THEN** `decode_adjustment` returns `None`

#### Scenario: Malformed black and white payload is a no-op

- **WHEN** a `blwh` payload is truncated, does not parse as a descriptor object, or carries an out-of-range or wrong-typed key
- **THEN** `decode_adjustment` returns `None`

#### Scenario: Malformed Photo Filter payload is a no-op

- **WHEN** a `phfl` payload is truncated, has a version other than 2, has a colour component above 255, or has a density above 100
- **THEN** `decode_adjustment` returns `None`

#### Scenario: Malformed Gradient Map payload is a no-op

- **WHEN** a `grdm` payload is truncated, has a version other than 1 or 3, has fewer than two colour stops, or has non-increasing or out-of-range stop locations
- **THEN** `decode_adjustment` returns `None` and does not panic

#### Scenario: Malformed Color Balance payload is a no-op

- **WHEN** a `blnc` payload is truncated or carries a shift outside `-100..=100`
- **THEN** `decode_adjustment` returns `None` and does not panic

#### Scenario: Malformed Channel Mixer payload is a no-op

- **WHEN** a `mixr` payload has a version other than 1, is shorter than the channels it declares, or carries a source percentage or constant outside `-200..=200`
- **THEN** `decode_adjustment` returns `None` and does not panic

#### Scenario: Malformed Curves payload is a no-op

- **WHEN** a `curv` payload has a version other than 1, a zero bitmask, a bit outside `0b1111`, a node count outside `2..=14`, a coordinate outside `0..=255`, non-increasing inputs, or is truncated mid-channel
- **THEN** `decode_adjustment` returns `None` and does not panic

#### Scenario: Malformed solid-color descriptor is a no-op

- **WHEN** a `SoCo` payload is neither a 4-byte RGBA tuple nor a version-16 descriptor object whose `Clr ` object carries `Rd  `, `Grn `, and `Bl  ` finite doubles
- **THEN** `decode_adjustment` returns `None` and does not panic

#### Scenario: Photo Filter version 3 is deferred

- **WHEN** a `phfl` payload has version 3
- **THEN** `decode_adjustment` returns `None`

### Requirement: Deferred adjustment payloads remain no-ops

The renderer SHALL return `None` from `decode_adjustment` for the deferred
payloads `selc`, `clrL`, and a version-3 `phfl`. These payloads SHALL remain
preserved on disk and their layers SHALL leave the backdrop unchanged.

#### Scenario: Deferred keys return None

- **WHEN** a `selc`, `clrL`, or version-3 `phfl` payload is decoded
- **THEN** `decode_adjustment` returns `None`

#### Scenario: Deferred layer does not change the composite

- **WHEN** a document contains an adjustment layer with a deferred payload over a backdrop
- **THEN** the composite equals the backdrop-only composite

## ADDED Requirements

### Requirement: Curves payloads encode and round-trip

`pictura-render` SHALL expose `encode_curves(points: &[(u8, u8)], red:
Option<&[(u8, u8)]>, green: Option<&[(u8, u8)]>, blue: Option<&[(u8, u8)]>) ->
AdjustmentData` that builds the `curv` block `decode_adjustment` reads: a `u8`
ignored byte, a `u16` version equal to `1`, a `u16` ignored word, and a `u16`
bitmask setting a bit for each present non-empty channel (`1` rgb, `2` red, `4`
green, `8` blue), followed by each present channel's `u16` node count and its
`(output, input)` pairs in the order rgb, red, green, blue. The encoder SHALL
write only the version-1 bitmask section and SHALL NOT write the version-4
duplicate `Crv ` section. `decode_adjustment` on the encoder's output SHALL
equal `Adjustment::Curves` with the input composite curve as `points`, the input
per-channel curves as `red`/`green`/`blue`, and `None` for every absent channel
(when the rgb channel is absent, `points` decodes as the identity
`[(0, 0), (255, 255)]`).

#### Scenario: Encoded composite Curves decodes back

- **WHEN** `encode_curves(&[(0, 0), (64, 32), (192, 224), (255, 255)], None, None, None)` is passed to `decode_adjustment`
- **THEN** it returns `Adjustment::Curves` whose `points` is those points and whose `red`/`green`/`blue` are `None`

#### Scenario: Encoded per-channel Curves decodes back

- **WHEN** `encode_curves` is called with an rgb curve and all three per-channel curves
- **THEN** `decode_adjustment` returns `Adjustment::Curves` carrying all four curves

#### Scenario: Absent channels are omitted from the bitmask

- **WHEN** `encode_curves` is called with only a composite curve
- **THEN** the encoded bitmask is `1` and no per-channel curve is written

### Requirement: Curves fixtures match an independent ag-psd decoder

The committed fixture `crates/pictura-codec/tests/fixtures/curves.psd` SHALL
carry a composite-only `curv` block and a per-channel `curv` block with fixed
values, and a test SHALL read the fixture with the independent `ag-psd` npm
package through `node` and SHALL assert the decoded `rgb`, `red`, `green`, and
`blue` curves equal the authored points. The fixture SHALL be regenerated
byte-stably by `scripts/generate-fixtures.py`. The test SHALL self-skip with a
clear message when `node` or `ag-psd` is unavailable, and SHALL NOT fail the
suite in that case.

#### Scenario: ag-psd reads the composite fixture block

- **WHEN** the fixture's composite-only `curv` layer is read by ag-psd
- **THEN** its `rgb` curve carries the authored points and its `red`/`green`/`blue` curves are absent

#### Scenario: ag-psd reads the per-channel fixture block

- **WHEN** the fixture's per-channel `curv` layer is read by ag-psd
- **THEN** its `rgb`, `red`, `green`, and `blue` curves carry the authored points

#### Scenario: The oracle self-skips without ag-psd

- **WHEN** `node` or the `ag-psd` package is not available
- **THEN** the test reports a skip and the suite passes
