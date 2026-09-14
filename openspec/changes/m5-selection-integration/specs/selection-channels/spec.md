## ADDED Requirements

### Requirement: Document-level extra channels

`pictura-core` MUST expose `Document.channels: Vec<Channel>` for document-level
extra channels (saved selections / spot channels), separate from the per-layer
channel list. A `Channel` MUST carry an `id: i16` and a planar 8-bit `data`
plane whose length equals `width * height` in bytes. A document created with
`Document::new` MUST start with an empty `channels` list.

#### Scenario: New document has no extra channels
- **WHEN** a document is created with `Document::new`
- **THEN** `Document.channels` is empty while available for later saves

#### Scenario: Extra channels are distinct from layer channels
- **WHEN** a document holds both per-layer channels and `Document.channels`
- **THEN** mutating one MUST NOT change the other

### Requirement: Extra channels round-trip through PSD

`pictura-codec` MUST write document-level extra channels into the PSD header
channel count and the image-data section, placing the extra planes after the
color planes. The header channel count MUST equal the color channel count plus
the number of extra channels. On read, the extra planes MUST be restored to
`Document.channels` while `Document.composite` MUST keep only the color planes.
Writing or reading extra channels MUST NOT regress composite or layer handling.

#### Scenario: Single saved selection round-trips
- **WHEN** a document with one extra channel is written and read back
- **THEN** the read document equals the written document and `channels` is restored

#### Scenario: Multiple extra channels round-trip
- **WHEN** a document with two extra channels is written and read back
- **THEN** both channels are restored in order

#### Scenario: Header count includes extra channels
- **WHEN** an RGB document with one extra channel is written
- **THEN** the header channel count is 4 and the composite keeps 3 color planes

#### Scenario: Layer and composite handling is unchanged
- **WHEN** a layered document with no extra channels is written and read back
- **THEN** composite pixels and the layer tree are unchanged

### Requirement: Extra channel validation

`pictura-codec` MUST reject a document whose extra channel data length does not
equal `width * height`, and MUST reject a header channel count that is zero or
exceeds the maximum supported channel count. Invalid input MUST return an error
rather than panicking.

#### Scenario: Length mismatch is rejected
- **WHEN** an extra channel has fewer bytes than `width * height`
- **THEN** `write_psd` returns an error

#### Scenario: Channel count over the cap is rejected
- **WHEN** a file header declares more than the maximum supported channels
- **THEN** `read_psd` returns an error

### Requirement: Selection to and from channel

`pictura-select` MUST provide `Selection::to_channel(id) -> Channel` and
`Selection::from_channel(&Channel, width, height) -> Result<Selection>`. Round-
tripping a selection through `to_channel` then `from_channel` at the same
dimensions MUST reproduce the selection coverage exactly. `from_channel` MUST
return an error when the channel length does not match `width * height`.

#### Scenario: Selection round-trips through a channel
- **WHEN** a selection is converted with `to_channel` and back with `from_channel`
- **THEN** the resulting selection equals the original

#### Scenario: Wrong channel length errors
- **WHEN** `from_channel` is given a channel whose data length is not `width * height`
- **THEN** it returns a size-mismatch error

### Requirement: PSD alpha channel is visible to psd-tools

A PSD written by `pictura-codec` with an extra channel MUST be readable by
`psd-tools`, which reports the bumped header channel count and exposes the extra
plane as the composite alpha channel. This is the independent oracle for the
alternate-channel layout.

#### Scenario: psd-tools reports the written alpha channel
- **WHEN** psd-tools opens a written PSD with one extra channel
- **THEN** it reports color channels plus one, and the alpha bytes equal the extra plane
