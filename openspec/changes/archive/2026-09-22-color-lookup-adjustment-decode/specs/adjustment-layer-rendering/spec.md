## ADDED Requirements

### Requirement: Color Lookup payloads decode to ColorLookupParams

`decode_adjustment` SHALL decode a `clrL` payload into a new
`Adjustment::ColorLookup(ColorLookupParams)`. The payload SHALL be read as a
`u16` version equal to `1`, followed by a version-16 `DescriptorBlock` whose
`lookupType` enum (typeID `3DLUT`) selects a 3-D LUT, `LUTFormat` enum
(`LUTFormatCUBE`, `LUTFormat3DL`, `LUTFormatLOOK`) names the embedded format,
and `LUT3DFileData` carries the embedded LUT file bytes. A payload that is
truncated, carries a version other than `1`, does not parse as a descriptor
object, or lacks a `lookupType` SHALL decode to `None` and SHALL NOT panic.
`ColorLookupParams` SHALL carry the decoded `kind` (3-D LUT, abstract profile,
or device link) and, when `kind` is a 3-D LUT whose format is `.CUBE`, a parsed
`Lut3d` with its cube size and RGB points in the `.CUBE` order (red index
fastest, then green, then blue); if the embedded data does not parse as a valid
`.CUBE` (bad or absent `LUT_3D_SIZE`, size outside `2..=64`, wrong point count,
a non-finite component, or a format other than `.CUBE`), `lookup` SHALL be
`None`. An identity three-dimensional lookup SHALL be the exact identity within
one LSB per channel.

The `dataOrder` and `tableOrder` enums and the `Dthr` dither flag SHALL be read
as block metadata but SHALL NOT change how an embedded `.CUBE` is sampled, since
the `.CUBE` point order is intrinsic to the file.

#### Scenario: A 3-D LUT payload decodes to ColorLookupParams

- **WHEN** a `clrL` payload has version 1 and a descriptor with `lookupType` `3DLUT`, `LUTFormat` `LUTFormatCUBE`, and a valid `LUT_3D_SIZE 2` `LUT3DFileData`
- **THEN** `decode_adjustment` returns `Adjustment::ColorLookup` whose `kind` is 3-D LUT and whose `lookup` is `Some` with size 2 and eight points

#### Scenario: An abstract-profile lookup decodes without a LUT

- **WHEN** a `clrL` payload has `lookupType` `abstractProfile`
- **THEN** `decode_adjustment` returns `Adjustment::ColorLookup` whose `kind` is abstract profile and whose `lookup` is `None`

#### Scenario: Malformed Color Lookup is a no-op

- **WHEN** a `clrL` payload is truncated, has a version other than 1, does not parse as a descriptor object, or lacks `lookupType`
- **THEN** `decode_adjustment` returns `None` and does not panic

#### Scenario: A non-CUBE or malformed embedded LUT decodes without a lookup

- **WHEN** the embedded `LUT3DFileData` is empty, is not a `.CUBE`, declares a `LUT_3D_SIZE` outside `2..=64`, or carries the wrong number of points
- **THEN** `decode_adjustment` returns `Adjustment::ColorLookup` with `lookup` `None` rather than a panic

### Requirement: Color Lookup renders the embedded cube

`apply` SHALL render `Adjustment::ColorLookup` by sampling the parsed `.cube`
trilinearly: it SHALL normalize each pixel channel to `0.0..=1.0`, place it on
the cube's `size`-point grid, take the eight surrounding nodes, and interpolate
with the fractional part; it SHALL write the sampled RGB back to the color
channels and SHALL leave alpha untouched. A `lookup` of `None` (abstract
profile, device link, non-`.CUBE` format, or malformed data) SHALL leave the
buffer bit-exactly unchanged. An identity cube SHALL leave the buffer within one
LSB per channel; a cube with distinct corner colours SHALL map a node exactly to
that node's colour.

Oracle expectation: no ImageMagick operator applies an arbitrary `.cube`, and
Photoshop's `.cube` sampling is not independently reproducible here, so there is
no Adobe pixel-parity claim; known-value tests cover the identity, exact node
mapping (which fixes the red-fastest point order), and a mid-cube trilinear
blend.

#### Scenario: An identity cube is the identity

- **WHEN** Color Lookup is applied with an identity `LUT_3D_SIZE 2` cube to any buffer
- **THEN** every colour channel is within 1 LSB of its input and alpha is bit-identical

#### Scenario: A cube corner maps exactly to its node

- **WHEN** a cube's corner node for a grey input is a distinct colour and the pixel sits exactly on that corner
- **THEN** the output is that node's colour

#### Scenario: The red-fastest point order is honored

- **WHEN** a `LUT_3D_SIZE 3` cube colours only the node at red 1, green 0, blue 0 and a pixel maps exactly to that node
- **THEN** the output is that node's colour, proving the point order is red-fastest

#### Scenario: A missing lookup is a no-op

- **WHEN** `apply` is called with an `Adjustment::ColorLookup` whose `lookup` is `None`
- **THEN** the buffer is bit-exactly unchanged

#### Scenario: Alpha is preserved

- **WHEN** Color Lookup is applied to an RGBA buffer
- **THEN** the alpha plane is bit-identical to the input

### Requirement: Color Lookup payloads encode and round-trip

`pictura-render` SHALL expose `encode_color_lookup(file_data: &[u8], name: &str)
-> AdjustmentData` that builds the `clrL` block `decode_adjustment` reads: a
`u16` version equal to `1`, then a version-16 descriptor whose `lookupType` is
`3DLUT`, whose `LUTFormat` is `LUTFormatCUBE`, whose `dataOrder` and `tableOrder`
are `rgbOrder`, whose `Dthr` is false, whose `Nm  ` and `LUT3DFileName` are
`name`, and whose `LUT3DFileData` is `file_data`. `decode_adjustment` on the
encoder's output for a valid `<N> 2..=64` `.CUBE` SHALL equal
`Adjustment::ColorLookup` whose `kind` is 3-D LUT and whose `lookup` carries the
cube's size and points. `pictura-render` SHALL also expose
`identity_cube() -> Vec<u8>` returning a `LUT_3D_SIZE 2` identity `.cube`, so a
default layer renders as a no-op.

#### Scenario: Encoded Color Lookup decodes back

- **WHEN** the bytes of a valid identity `.cube` are passed to `encode_color_lookup` and the block to `decode_adjustment`
- **THEN** it returns `Adjustment::ColorLookup` whose `kind` is 3-D LUT and whose `lookup` has size 2

#### Scenario: The encoded block is a version-1 descriptor

- **WHEN** the output of `encode_color_lookup` is parsed by `pictura_codec::read_descriptor` after the 2-byte version
- **THEN** it is an object whose `lookupType` enum is `3DLUT` and whose `LUT3DFileData` carries the input bytes

### Requirement: Color Lookup fixtures match an independent ag-psd decoder

The committed fixture SHALL carry a `clrL` block at
`crates/pictura-codec/tests/fixtures/color_lookup.psd` whose embedded
`LUT3DFileData` is a known `.cube`,
and a test SHALL read the fixture with the independent `ag-psd` npm package
through `node` and SHALL assert `lookupType`, `lutFormat`, `dataOrder`,
`tableOrder`, `dither`, and the exact `LUT3DFileData` bytes. The fixture SHALL
be regenerated byte-stably by `scripts/generate-fixtures.py`. The test SHALL
self-skip with a clear message when `node` or `ag-psd` is unavailable, and SHALL
NOT fail the suite in that case.

#### Scenario: ag-psd reads the fixture block

- **WHEN** the fixture's `clrL` layer is read by ag-psd
- **THEN** `lookupType` is `3DLUT`, `lutFormat` is `LUTFormatCUBE`, `dataOrder` and `tableOrder` are `rgbOrder`, `dither` is false, and the `LUT3DFileData` bytes equal the authored `.cube`

#### Scenario: The oracle self-skips without ag-psd

- **WHEN** `node` or the `ag-psd` package is not available
- **THEN** the test reports a skip and the suite passes

### Requirement: The app can create a Color Lookup adjustment layer

The app SHALL map the adjustment kind `color-lookup` to a `clrL` adjustment
layer carrying an identity cube, and the Adjustments panel menu SHALL offer a
`Color Lookup` entry that dispatches `adjustment:color-lookup`. Because the
identity cube is exact, the default layer SHALL leave the backdrop unchanged. A
`clrL` layer whose cube is not the identity SHALL change the composite over a
non-uniform backdrop.

#### Scenario: The color-lookup kind becomes an adjustment layer

- **WHEN** the app adds an adjustment layer of kind `color-lookup`
- **THEN** the new layer carries a `clrL` block and is reported as an adjustment layer

#### Scenario: The default Color Lookup layer is neutral

- **WHEN** a `color-lookup` adjustment layer with the default identity cube is composited over a backdrop
- **THEN** the result equals the backdrop-only composite

#### Scenario: The panel menu offers Color Lookup

- **WHEN** the Adjustments panel menu is built
- **THEN** it contains a `Color Lookup` row dispatching `adjustment:color-lookup`

## MODIFIED Requirements

### Requirement: Deferred adjustment payloads remain no-ops

The renderer SHALL return `None` from `decode_adjustment` for the deferred
payload version-3 `phfl`. This payload SHALL remain preserved on disk and its
layer SHALL leave the backdrop unchanged.

#### Scenario: Deferred keys return None

- **WHEN** a version-3 `phfl` payload is decoded
- **THEN** `decode_adjustment` returns `None`

#### Scenario: Deferred layer does not change the composite

- **WHEN** a document contains an adjustment layer with a deferred payload over a backdrop
- **THEN** the composite equals the backdrop-only composite
