# Specs delta: phfl-v3-xyz-decode

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
`Adjustment::ChannelMixer(ChannelMixerParams)`, `selc` to
`Adjustment::SelectiveColor(SelectiveColorParams)`, `curv` to
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
and the colour space SHALL be ignored. A version-3 `phfl` payload SHALL be read
as three big-endian `u32` CIE XYZ values, a `u32` density, and a `u8` luminosity
flag; the XYZ values SHALL be interpreted as 16.16 fixed-point relative to D50
white and converted to an sRGB filter colour with the same profile-free D50→sRGB
matrix class used for Lab document read; density SHALL be in `0..=100`. A
`grdm` payload SHALL be read as a `u16`
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
decode to `None`. A `selc` payload SHALL be read as a `u16` version equal to
`1`, a `u16` correction method (`0` relative, nonzero absolute), and ten plates of
four big-endian `i16` corrections in cyan, magenta, yellow, black order. The first
plate SHALL be ignored without validation (Photoshop reserves it and writes
zeroes); the remaining nine plates SHALL be the ranges reds, yellows, greens,
cyans, blues, magentas, whites, neutrals, and blacks, and every one of their
corrections SHALL be in `-100..=100` or the payload SHALL decode to `None`. Every
byte after the tenth plate SHALL be ignored. A `selc` payload SHALL decode to
`Adjustment::SelectiveColor` with that method and those nine range records. A
`curv` payload SHALL be read as a `u8` ignored byte, a `u16`
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

#### Scenario: Photo Filter version 3 decodes from XYZ

- **WHEN** a `phfl` payload has version 3, three big-endian `u32` XYZ values, a density in `0..=100`, and a luminosity flag
- **THEN** `decode_adjustment` returns `Adjustment::PhotoFilter` whose `color` is the sRGB conversion of those XYZ values, whose `density` is the stored percent, and whose `preserve_luminosity` is the luminosity byte interpreted as a boolean

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

#### Scenario: Selective Color payload decodes to SelectiveColorParams

- **WHEN** a `selc` payload has version 1, a relative method, a zero reserved plate, and nine non-zero range plates
- **THEN** `decode_adjustment` returns `Adjustment::SelectiveColor` with the relative method and those nine ranges in reds through blacks order

#### Scenario: Selective Color reserved plate and trailing bytes are ignored

- **WHEN** a `selc` payload carries non-zero values in its reserved first plate and trailing bytes after the tenth plate
- **THEN** `decode_adjustment` returns the same `Adjustment::SelectiveColor` as the payload with a zero reserved plate and no trailing bytes

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

- **WHEN** a `phfl` payload is truncated, has a version other than 2 or 3, has a version-2 colour component above 255, or has a density above 100
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

#### Scenario: Malformed Selective Color payload is a no-op

- **WHEN** a `selc` payload is shorter than 84 bytes, has a version other than 1, or carries a range correction outside `-100..=100`
- **THEN** `decode_adjustment` returns `None` and does not panic

#### Scenario: Malformed solid-color descriptor is a no-op

- **WHEN** a `SoCo` payload is neither a 4-byte RGBA tuple nor a version-16 descriptor object whose `Clr ` object carries `Rd  `, `Grn `, and `Bl  ` finite doubles
- **THEN** `decode_adjustment` returns `None` and does not panic

### Requirement: Photo Filter payloads encode and round-trip

`pictura-render` SHALL expose `encode_photo_filter(color: [u8; 3], density:
f64, preserve_luminosity: bool) -> AdjustmentData` that builds the version-2
`phfl` block `decode_adjustment` reads. The encoder SHALL clamp `density` to
`0..=100` and clamp each colour component to `0..=255`, so its output always
decodes. `decode_adjustment` on the encoder's output SHALL equal
`Adjustment::PhotoFilter` with the input colour, the clamped density, and the
input luminosity flag.
Encoding SHALL remain version 2 (the app authoring
path); a decoded version-3 payload re-emitted through this encoder becomes
version 2 with the decoded colour.

#### Scenario: Encoded Photo Filter decodes back

- **WHEN** `encode_photo_filter([255, 180, 80], 25.0, true)` is passed to `decode_adjustment`
- **THEN** it returns `Adjustment::PhotoFilter` with `color` `[255, 180, 80]`, `density` `25.0`, and `preserve_luminosity` true

#### Scenario: Out-of-range density is clamped

- **WHEN** `encode_photo_filter` is called with a density above 100 or below 0
- **THEN** the encoded density is clamped into `0..=100` and the block still decodes

#### Scenario: Version-3 open-save re-emits version 2 with decoded colour

- **WHEN** a well-formed version-3 `phfl` decodes and is re-encoded via `encode_photo_filter` using the decoded colour, density, and flag
- **THEN** the new payload has version 2 and `decode_adjustment` returns the same `PhotoFilterParams` colour, density, and flag (up to 8-bit colour quantisation)

## REMOVED Requirements

### Requirement: Deferred adjustment payloads remain no-ops

**Reason**: Version-3 `phfl` was the only deferred adjustment payload. It now
decodes to `Adjustment::PhotoFilter`, so the deferred no-op contract has no
remaining subject.

**Migration**: Decode a version-3 `phfl` through the committed Photo Filter
path; unknown keys continue to return `None` via the existing unknown-key
behaviour in the committed-decode requirement.
