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
`Adjustment::ChannelMixer(ChannelMixerParams)`, `SoCo` to
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
decode to `None`. A `SoCo` payload SHALL be read in one of two forms: a 4-byte
straight-alpha RGBA tuple, or a version-16 descriptor whose `Clr ` object
carries `Rd  `, `Grn `, and `Bl  ` `doub` values on the `0..=255` scale. The
descriptor form SHALL decode to `Adjustment::SolidFill([r, g, b, 255])`,
rounding and clamping each component to `0..=255` and forcing alpha to 255; the
4-byte form SHALL keep its four stored components. A `GdFl` payload SHALL be
read as a version-16 descriptor whose `Angl` `doub` is the angle in degrees,
whose `Type` enum (typeID `GrdT`) is the gradient kind (`Lnr ` Linear, `Rdl `
Radial, `Angl` Angle, `Rflc` Reflected, `Dmnd` Diamond), and whose `Grad`
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

#### Scenario: Malformed solid-color descriptor is a no-op

- **WHEN** a `SoCo` payload is neither a 4-byte RGBA tuple nor a version-16 descriptor object whose `Clr ` object carries `Rd  `, `Grn `, and `Bl  ` finite doubles
- **THEN** `decode_adjustment` returns `None` and does not panic

#### Scenario: Photo Filter version 3 is deferred

- **WHEN** a `phfl` payload has version 3
- **THEN** `decode_adjustment` returns `None`

### Requirement: Deferred adjustment payloads remain no-ops

The renderer SHALL return `None` from `decode_adjustment` for the deferred
payloads `curv`, `selc`, `clrL`, and a version-3 `phfl`. These payloads SHALL
remain preserved on disk and their layers SHALL leave the backdrop unchanged.

#### Scenario: Deferred keys return None

- **WHEN** a `curv`, `selc`, `clrL`, or version-3 `phfl` payload is decoded
- **THEN** `decode_adjustment` returns `None`

#### Scenario: Deferred layer does not change the composite

- **WHEN** a document contains an adjustment layer with a deferred payload over a backdrop
- **THEN** the composite equals the backdrop-only composite

## ADDED Requirements

### Requirement: Channel Mixer payloads encode and round-trip

`pictura-render` SHALL expose `encode_channel_mixer(monochrome: bool, red:
[f64; 3], green: [f64; 3], blue: [f64; 3], constant: [f64; 3]) ->
AdjustmentData` that builds the `mixr` block `decode_adjustment` reads: a `u16`
version equal to 1, a `u16` monochrome flag, the `red`/`green`/`blue` rows when
not monochrome followed by a `gray` row writing `rgb = red` and
`constant = constant[0]`, or the `gray` row (`rgb = red`,
`constant = constant[0]`) followed by 30 zero
bytes when monochrome, so the block is 44 bytes in both forms. Each channel
SHALL write its three source percentages, two zero reserved bytes, and its
constant. The encoder SHALL clamp each source percentage and constant to
`-200..=200`, so its output always decodes. `decode_adjustment` on the encoder's
output SHALL equal `Adjustment::ChannelMixer` with the input monochrome flag,
rows, and constants (the monochrome rows' unused `green`/`blue` values decoded
from Photoshop defaults).

#### Scenario: Encoded non-monochrome Channel Mixer decodes back

- **WHEN** `encode_channel_mixer(false, [30, -10, 50], [10, 90, 0], [0, 20, 110], [5, -20, 40])` is passed to `decode_adjustment`
- **THEN** it returns `Adjustment::ChannelMixer` whose `monochrome` is false, whose rows are those triples, and whose `constant` is `[5, -20, 40]`

#### Scenario: Encoded monochrome Channel Mixer decodes back

- **WHEN** `encode_channel_mixer(true, [20, 40, 60], [0, 100, 0], [0, 0, 100], [-15, 0, 0])` is passed to `decode_adjustment`
- **THEN** it returns `Adjustment::ChannelMixer` whose `monochrome` is true, whose `red` is `[20, 40, 60]`, and whose `constant[0]` is `-15`

#### Scenario: Out-of-range weights are clamped

- **WHEN** `encode_channel_mixer` is called with any source percentage or constant above 200 or below -200
- **THEN** the encoded value is clamped into `-200..=200` and the block still decodes

### Requirement: Channel Mixer fixtures match an independent ag-psd decoder

The committed fixture `crates/pictura-codec/tests/fixtures/channel_mixer.psd` SHALL carry
a non-monochrome `mixr` block and a monochrome `mixr` block with
fixed values, and a test SHALL read the fixture with the independent `ag-psd` npm
package through `node` and SHALL assert the decoded `monochrome` flag and the
`red`, `green`, `blue`, and `gray` channels equal the authored values. The test
SHALL self-skip with a clear message when `node` or `ag-psd` is unavailable, and
SHALL NOT fail the suite in that case.

#### Scenario: ag-psd reads the non-monochrome fixture block

- **WHEN** the fixture's non-monochrome `mixr` layer is read by ag-psd
- **THEN** `monochrome` is false and its `red`, `green`, and `blue` channels carry the authored triples and constants, and its `gray` channel carries the authored (renderer-ignored) values

#### Scenario: ag-psd reads the monochrome fixture block

- **WHEN** the fixture's monochrome `mixr` layer is read by ag-psd
- **THEN** `monochrome` is true and its `gray` channel carries the authored triple and constant

#### Scenario: The oracle self-skips without ag-psd

- **WHEN** `node` or the `ag-psd` package is not available
- **THEN** the test reports a skip and the suite passes

### Requirement: The app can create a Channel Mixer adjustment layer

The app SHALL map the adjustment kind `channel-mixer` to a `mixr` adjustment
layer carrying the neutral identity default (monochrome off, the rows
`[100,0,0]`/`[0,100,0]`/`[0,0,100]`, constants `[0,0,0]`), and the Adjustments
panel menu SHALL offer a `Channel Mixer` entry that dispatches
`adjustment:channel-mixer`. Because the identity rows leave the matrix
unchanged, the default layer SHALL leave the backdrop unchanged. A `mixr` layer
with a non-identity row SHALL change the composite over a non-uniform backdrop.

#### Scenario: The channel-mixer kind becomes an adjustment layer

- **WHEN** the app adds an adjustment layer of kind `channel-mixer`
- **THEN** the new layer carries a `mixr` block and is reported as an adjustment layer

#### Scenario: The default Channel Mixer layer is neutral

- **WHEN** a `channel-mixer` adjustment layer with the default parameters is composited over a backdrop
- **THEN** the result equals the backdrop-only composite

#### Scenario: The panel menu offers Channel Mixer

- **WHEN** the Adjustments panel menu is built
- **THEN** it contains a `Channel Mixer` row dispatching `adjustment:channel-mixer`

#### Scenario: A non-neutral Channel Mixer layer changes the composite

- **WHEN** a Channel Mixer adjustment layer with a non-identity row is composited over a non-uniform backdrop
- **THEN** the result differs from the backdrop-only composite
