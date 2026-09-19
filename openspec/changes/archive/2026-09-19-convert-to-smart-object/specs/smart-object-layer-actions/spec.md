## ADDED Requirements

### Requirement: Convert to Smart Object eligibility and refusal

The engine operation `pictura-render::convert_to_smart_object(doc, path)` SHALL
convert the layer at `path` only when it is a single raster pixel layer: the path
resolves, the layer is not a group, has no adjustment data, is not the Background
layer, has no existing smart object, and has a `rect` with positive width and
height. On any other target the operation SHALL return `false` and SHALL NOT
mutate the document.

#### Scenario: A raster pixel layer converts

- **WHEN** `convert_to_smart_object` is called on the path of a raster pixel layer that is not the Background and has no smart object
- **THEN** it returns `true` and the layer carries an embedded smart object

#### Scenario: A group is refused

- **WHEN** the target path resolves to a group layer
- **THEN** the operation returns `false` and the document is unchanged

#### Scenario: An adjustment layer is refused

- **WHEN** the target layer has adjustment data
- **THEN** the operation returns `false` and the document is unchanged

#### Scenario: The Background is refused

- **WHEN** the target layer is the Background layer
- **THEN** the operation returns `false` and the document is unchanged

#### Scenario: An existing smart object is refused

- **WHEN** the target layer already has a smart object
- **THEN** the operation returns `false` and the document is unchanged

#### Scenario: A zero-size layer is refused

- **WHEN** the target layer's `rect` has zero width or zero height
- **THEN** the operation returns `false` and the document is unchanged

#### Scenario: An unresolved path is refused

- **WHEN** `path` does not resolve to a layer
- **THEN** the operation returns `false` and the document is unchanged

### Requirement: Embedded smart-object source authoring

On success the operation SHALL author an embedded source document and attach it
to the layer as `SmartObject { kind: Embedded, payload, filename, filetype,
creator }`. The source document SHALL use the document's color mode and bit
depth and the layer's `rect` as its size, SHALL contain a copy of the layer
translated so its top-left is `(0, 0)`, and SHALL carry a merged composite equal
to the layer's raster over that rect. The operation SHALL serialize the source
with `write_psd` and store the bytes as the payload, with `filename` equal to
`"<layer name>.psd"`, `filetype` `8BPB`, and `creator` `8BIM`. The payload SHALL
be non-empty and SHALL be a valid PSD that the codec resolves as an embedded
smart object.

#### Scenario: The payload is a valid embedded source

- **WHEN** a raster layer is converted and the stored payload is read with `read_psd`
- **THEN** the payload parses, its document size equals the layer's `rect`, and its merged composite equals the layer's raster

#### Scenario: SmartObject metadata is stamped

- **WHEN** the conversion succeeds
- **THEN** the layer's smart object reports kind `Embedded`, a non-empty payload, filename `"<layer name>.psd"`, `filetype` `8BPB`, and `creator` `8BIM`

#### Scenario: Same mode and depth

- **WHEN** a Grayscale 8-bit document's layer is converted
- **THEN** the embedded source document is Grayscale 8-bit and `write_psd` accepts it

### Requirement: The raster proxy and rendering are unchanged

The operation SHALL keep the layer's existing pixel channels as a raster proxy
and SHALL NOT change the layer's name, rect, blend, opacity, fill, mask, or
position. Because a layer with a proxy renders from the proxy, the document
composite after conversion SHALL equal the composite before conversion.

#### Scenario: Pixel channels survive

- **WHEN** a raster layer with color channels is converted
- **THEN** the layer's channels are byte-for-byte the same as before

#### Scenario: The composite is unchanged

- **WHEN** the service composite is compared before and after a successful conversion
- **THEN** the two composites are equal

#### Scenario: Rejected destinations are untouched beyond the refusal

- **WHEN** conversion is refused
- **THEN** the layer's channels, rect, name, and smart-object state are unchanged

### Requirement: Save and load preserve the converted object

A document containing a converted layer SHALL round-trip through `write_psd`
and `read_psd` with the smart object preserved as embedded and its payload
unchanged.

#### Scenario: Round-trip preserves the object

- **WHEN** a document with a converted layer is written and read back
- **THEN** the layer resolves as an embedded smart object with a non-empty payload whose bytes equal the written payload

#### Scenario: The proxy still round-trips

- **WHEN** a converted document is written and read back
- **THEN** the layer's pixel channels still resolve and the composite is unchanged

### Requirement: The app command records exactly one undo state

The application SHALL expose the command `Layer > Smart Objects > Convert to
Smart Object` with the stable id `LayerSmartObjectConvertTo`. The command SHALL
be enabled only when the current layer is convertible, SHALL invoke the engine
operation on success, clear the link sets, recomposite, and record exactly one
history state labelled `"Convert to Smart Object"`. A refusal SHALL record no
history state.

#### Scenario: Success records one state

- **WHEN** the command runs on a convertible current layer
- **THEN** exactly one history state labelled `"Convert to Smart Object"` is added and the layer reports a smart object

#### Scenario: Refusal records nothing

- **WHEN** the command runs on a group, an adjustment layer, the Background, or an existing smart object
- **THEN** the history count is unchanged and the document is unchanged

#### Scenario: Enabled only when convertible

- **WHEN** the current layer is not convertible
- **THEN** the command is disabled and its availability predicate mutates nothing
