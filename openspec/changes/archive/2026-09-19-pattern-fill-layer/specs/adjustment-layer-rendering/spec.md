## MODIFIED Requirements

### Requirement: Committed adjustment payloads decode to typed parameters

`decode_adjustment` SHALL decode the committed PSD adjustment payloads into the
matching `pictura_adjust::Adjustment` variant: `expA` to
`Adjustment::Exposure(ExposureParams)`, `vibA` to
`Adjustment::Vibrance(VibranceParams)`, `blwh` to
`Adjustment::BlackWhite(BlackWhiteParams)`, `phfl` to
`Adjustment::PhotoFilter(PhotoFilterParams)`, `grdm` to
`Adjustment::GradientMap(GradientMapParams)`, `blnc` to
`Adjustment::ColorBalance(ColorBalanceParams)`, `SoCo` to
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
`SoCo` payload SHALL be read in one of two forms: a 4-byte straight-alpha RGBA
tuple, or a version-16 descriptor whose `Clr ` object carries `Rd  `, `Grn `,
and `Bl  ` `doub` values on the `0..=255` scale. The descriptor form SHALL
decode to `Adjustment::SolidFill([r, g, b, 255])`, rounding and clamping each
component to `0..=255` and forcing alpha to 255; the 4-byte form SHALL keep its
four stored components. A `GdFl` payload SHALL be read as a version-16
descriptor whose `Angl` `doub` is the angle in degrees, whose `Type` enum (typeID
`GrdT`) is the gradient kind (`Lnr ` Linear, `Rdl ` Radial, `Angl` Angle, `Rflc`
Reflected, `Dmnd` Diamond), and whose `Grad` object (class `Grdn`) carries the
gradient stops. The `Grad` object's `GrdF` enum (typeID `GrdF`) SHALL be the
custom-stops form (`CstS`); a colour-noise form (`ClNs`) SHALL NOT decode. Each
stop in the `Clrs` list SHALL be an object whose `Clr ` `RGBC` object carries
`Rd  `, `Grn `, and `Bl  ` finite `doub` values on the `0..=255` scale and whose
`Lctn` value is a `doub` or `long` in `0..=4096`. Stops SHALL be kept in stored
order. When present, the top-level `Rvrs` `bool` SHALL be the reverse flag and
the `Scl ` `doub` SHALL be the scale percent; absent, they SHALL default to false
and 100. A `PtFl` payload SHALL be read as a version-16 descriptor whose `Ptrn`
object (class `Ptrn`) carries the pattern id in its `Idnt` text and whose
top-level `Scl ` value (`doub` or unit float) is the scale percent and whose
`Algn` `bool` is the Link With Layer flag; a missing or wrong-typed
`Ptrn`/`Idnt`, a non-finite scale, or a malformed descriptor SHALL decode to
`None`. The pattern id SHALL have trailing NUL bytes stripped; an absent `Scl `
SHALL default to 100 and an absent `Algn` SHALL default to true.

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

#### Scenario: Malformed solid-color descriptor is a no-op

- **WHEN** a `SoCo` payload is neither a 4-byte RGBA tuple nor a version-16 descriptor object whose `Clr ` object carries `Rd  `, `Grn `, and `Bl  ` finite doubles
- **THEN** `decode_adjustment` returns `None` and does not panic

#### Scenario: Photo Filter version 3 is deferred

- **WHEN** a `phfl` payload has version 3
- **THEN** `decode_adjustment` returns `None`

## ADDED Requirements

### Requirement: The document pattern library is decoded from the Patt resource

`pictura-codec` SHALL expose `decode_patterns(&Document) -> Vec<PatternPixels>`
that decodes the document's Patterns resource into row-major RGBA pattern
pixels carrying each pattern's id, width, and height. The resource SHALL be read
from the `Patt`, `Pat2`, and `Pat3` additional-layer-information tagged blocks
in the Layer and Mask Information section (preserved in
`Document.layer_section_extra`), NOT from `Document.image_resources` (image
resource 1039 is the ICC profile). Each block SHALL be parsed as a `Patterns`
list of length-prefixed version-1 patterns: image mode, width/height point,
Unicode name, Pascal pattern id, and a `VirtualMemoryArrayList` whose
`num_channels + 2` channel arrays are each decoded with the existing 8-bit
channel decompression. The written colour channels SHALL be the pattern colour
(the first written channel replicated for a grayscale pattern and the first
three taken as R, G, B for an RGB pattern), and a written channel in the alpha
region SHALL be the pattern's transparency with 255 when absent. Only 8-bit
planes SHALL be decoded: a written channel whose declared depth or pixel depth
is not 8 SHALL cause the pattern to be skipped. An RGB pattern SHALL have
exactly three written colour planes and a grayscale pattern exactly one; a
pattern with a missing or extra colour plane, or whose declared rectangle
disagrees with its written channel rectangles or exceeds a bounded pixel cap,
SHALL be skipped. A malformed or unrecognised block SHALL be skipped, keeping
the patterns parsed so far, and the decoder SHALL NOT panic or allocate without
bound.

#### Scenario: A Patt block decodes to pattern pixels

- **WHEN** a document carries a `Patt` tagged block with a version-1 RGB pattern whose channels hold known 8-bit planes
- **THEN** `decode_patterns` returns one `PatternPixels` with that pattern's id, size, and RGBA pixels in row-major order

#### Scenario: A grayscale pattern replicates its single channel

- **WHEN** a pattern's image mode is Grayscale with one written colour channel
- **THEN** the decoded pixel's R, G, and B components all equal that channel

#### Scenario: A malformed Patt block is skipped

- **WHEN** a `Patt` block is truncated or one pattern's channel list is malformed
- **THEN** `decode_patterns` returns the patterns parsed before the failure and does not panic

#### Scenario: Non-8-bit pattern planes are skipped

- **WHEN** a `Patt` block's written channel declares a depth or pixel depth other than 8
- **THEN** `decode_patterns` skips that pattern and returns the patterns parsed before it, without decoding the plane or panicking

#### Scenario: A crafted oversized pattern rectangle is skipped

- **WHEN** a pattern's declared rectangle is inconsistent with its written channel rectangles or exceeds the decoder's pixel cap
- **THEN** `decode_patterns` skips the pattern without an unbounded allocation and does not panic

#### Scenario: The trailing alpha slot is the pattern transparency

- **WHEN** both slots of a pattern's two-slot alpha region are written
- **THEN** the decoded alpha is the last written slot, matching psd-tools' trailing-plane rule

#### Scenario: A pattern with a missing colour plane is skipped

- **WHEN** an RGB pattern does not have exactly three written colour planes, or a grayscale pattern does not have exactly one
- **THEN** `decode_patterns` skips it rather than filling the absent channel with zero

### Requirement: Pattern fill layers render by tiling the document pattern

`decode_adjustment` SHALL cause the renderer to composite a `PtFl` pattern fill
(`Adjustment::PatternFill`) by tiling the referenced pattern over the layer's
rectangle clamped to the canvas, blending
each in-rect sample at the pattern's alpha through the normal layer path, so the
layer's mask, opacity, fill, and blend mode apply and pixels outside the
rectangle are untouched. The pattern SHALL be resolved by `pattern_id` from the
document's decoded pattern pixels. The tile SHALL use the params' `scale`
(percent; the tile is resampled when the scale is not 100),
`link_with_layer` (the tile is anchored at the layer's top-left when set and at
the document origin when clear) and `origin` (a pixel offset added to the
sample). When no decoded pattern matches `pattern_id`, the layer SHALL composite
a grey placeholder over its rectangle rather than a no-op, and SHALL NOT panic.

#### Scenario: A pattern fill layer tiles its pattern

- **WHEN** a layer references a 2x2 RGB pattern at scale 100 with Link With Layer on and is composited over a backdrop
- **THEN** the in-rect pixels repeat the 2x2 tile and the result differs from the backdrop-only composite

#### Scenario: Adjacent tile cells differ

- **WHEN** the composite is sampled at horizontally adjacent in-rect pixels and again one tile width across
- **THEN** the adjacent pixels carry different tile colours and the pixel one tile width across repeats the first

#### Scenario: Link With Layer anchors the tile to the layer rect

- **WHEN** `link_with_layer` is true and the layer rectangle starts at a non-zero offset
- **THEN** the tile's first cell is aligned with the layer rectangle's top-left, so the pattern moves with the layer

#### Scenario: A missing pattern falls back to the placeholder

- **WHEN** a layer's `pattern_id` does not match any pattern decoded from the document
- **THEN** the in-rect composite is the opaque grey placeholder and the composite is not a no-op

#### Scenario: A masked-out pattern fill is a no-op

- **WHEN** a pattern fill layer's mask hides the whole canvas
- **THEN** the composite equals the backdrop-only composite
