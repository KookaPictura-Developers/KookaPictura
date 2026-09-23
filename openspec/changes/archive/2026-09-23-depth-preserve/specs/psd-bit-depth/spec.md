## MODIFIED Requirements

### Requirement: Layer channels are narrowed with the document

When a 16- or 32-bit document carries layers, `read_psd` SHALL decode each layer
channel — its color channels, the `-1` transparency channel, the `-2` mask
channel, and unmodeled channels — at the document depth and SHALL narrow it to
an 8-bit plane before assembling the layer, so the layer path agrees with the
narrowed composite. The layer's color channels SHALL then be converted by the
document's color-mode normalization. For a Grayscale or RGB document the
read path SHALL also retain the decoded source-depth samples of each layer
channel so an unchanged channel can be re-emitted at the source depth on save
(see the source-depth-on-save requirement).

#### Scenario: A 16-bit layer color channel is narrowed

- **WHEN** a depth-16 document carries a pixel layer whose color channel stores `u16` samples
- **THEN** the returned layer's color channel holds the 8-bit narrowing of those samples

#### Scenario: A 16-bit mask channel is narrowed

- **WHEN** a depth-16 document carries a `-2` mask channel
- **THEN** the returned mask plane is the 8-bit narrowing of the stored samples and is not decoded as an 8-bit row

#### Scenario: A 16-bit unmodeled channel is narrowed for the engine and restored on save

- **WHEN** a depth-16 document carries a layer channel the engine does not model (for example a `-3` or positive spot channel)
- **THEN** the channel is decoded at the document depth and narrowed to an 8-bit plane for the engine, and a save of the unedited document re-emits it at the source depth

### Requirement: The application reports a normalized bit depth

The application SHALL expose the source bit depth of an opened document and
SHALL present a conversion notice to the user when a 16/32-bit file is
normalized, so the user knows the document is edited at 8-bit working precision
and, for a Grayscale or RGB document, that a save preserves the source depth
(with unchanged planes exact and edited planes widened). For a mode the read
path converted (such as CMYK or Lab) the notice SHALL report the conversion
without claiming the save preserves the depth, because it saves 8-bit.

#### Scenario: Opening a 16-bit file shows a notice

- **WHEN** the application opens a depth-16 Grayscale or RGB PSD
- **THEN** the view reports that the document was converted from 16-bit, the document's depth is 8-bit, and the notice does not claim the save will be 8-bit

#### Scenario: Opening a 16-bit converted-mode file still shows a notice

- **WHEN** the application opens a depth-16 CMYK or Lab PSD
- **THEN** the view reports the conversion from 16-bit and does not claim the save preserves the depth

#### Scenario: Opening an 8-bit file shows no depth notice

- **WHEN** the application opens an 8-bit PSD
- **THEN** no depth-conversion notice is shown

## REMOVED Requirements

### Requirement: A normalized document records its source depth and saves as 8-bit

**Reason**: The codec now preserves the source bit depth on save instead of
always writing 8-bit, so the "saves as 8-bit" half of this requirement no longer
holds. The recording half is restated (with the save behavior) in the added
"A normalized document records its source depth and preserves it on save"
requirement.

**Migration**: No caller change is required for an 8-bit or constructed
document: `source_depth` stays `None` and the save is byte-identical to before.
A document read from a 16/32-bit Grayscale/RGB file now saves at its source
depth, so a caller that relied on the output being 8-bit must read the header
depth instead; the application notice is updated accordingly. The
`depth_oracle` expectations that asserted an 8-bit output are updated to the
source depth.

## ADDED Requirements

### Requirement: A normalized document records its source depth and preserves it on save

`read_psd` SHALL set `Document.source_depth` to `Some(BitDepth::Sixteen)` or
`Some(BitDepth::ThirtyTwo)` for any 16/32-bit document it accepts (so the
application can report the conversion), and to `None` for an 8-bit document, a
depth-1 Bitmap document, and a constructed document. For a 16/32-bit Grayscale
or RGB document — the modes the read path does not convert — it SHALL retain the
decoded source-depth samples of the composite color planes, the document extra
channels, and every layer channel. `write_psd` SHALL write the output header at
the document's source depth when the document retained samples, else 8. For each
plane, when the retained source-depth samples narrow to the plane's current
8-bit bytes the writer SHALL re-encode the retained samples at the source depth;
when they do not (the plane was edited, or the layer moved) the writer SHALL
widen the current 8-bit plane to the source depth and encode it, so a save of a
16/32-bit Grayscale or RGB document is not a silent downgrade to 8-bit. Widening SHALL be `v * 257` at 16-bit and the 8-bit
value scaled to `[0, 1]` at 32-bit and SHALL be documented as an approximation
that cannot recover the source low bits or HDR range. Compression SHALL follow
the document's recorded kinds at the source depth: raw, PackBits RLE, ZIP, and
ZIP-with-prediction, whose forward predictor SHALL be the depth-specific
inverse of the read predictor (a per-`u16` difference at 16-bit and the byte
difference plus the four-byte-plane shuffle at 32-bit). A document with no
recorded source depth, and a mode the read path converted, SHALL save as 8-bit
as before.

#### Scenario: A 16-bit document preserves its depth on save

- **WHEN** a depth-16 Grayscale or RGB document is read and written unchanged
- **THEN** the output header declares bit depth 16, the composite and layer channels round-trip to the same 8-bit pixels, and an independent decoder reads the source-depth samples

#### Scenario: An edited plane keeps the source depth, widened

- **WHEN** a plane of a depth-16 document is edited to new 8-bit bytes and the document is written
- **THEN** the output still declares bit depth 16, the edited plane is the 8-bit bytes widened by `v * 257`, and reading it back narrows to those 8-bit bytes

#### Scenario: A depth-32 document preserves its depth

- **WHEN** a depth-32 Grayscale or RGB document is read and written unchanged
- **THEN** the output header declares bit depth 32 and an independent decoder reads the source-depth samples

#### Scenario: An 8-bit or constructed document is unchanged

- **WHEN** an 8-bit or constructed document is written
- **THEN** the output header declares bit depth 8 and the bytes are as before

#### Scenario: A normalized color mode still saves 8-bit

- **WHEN** a 16-bit CMYK or Lab document (whose planes the read path converted to the working mode) is read and written
- **THEN** the output header declares bit depth 8 and the document still records `source_depth` as 16-bit, so the application reports the conversion

#### Scenario: The psd-tools oracle reads the output depth and samples

- **WHEN** a written 16/32-bit file is decoded by `psd-tools`
- **THEN** the header depth and the native samples agree with the retained source-depth samples, and the oracle self-skips when `psd-tools` is unavailable
