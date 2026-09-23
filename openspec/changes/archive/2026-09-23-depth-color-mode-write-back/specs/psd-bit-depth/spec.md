## MODIFIED Requirements

### Requirement: A normalized document records its source depth and preserves it on save

`read_psd` SHALL set `Document.source_depth` to `Some(BitDepth::Sixteen)` or
`Some(BitDepth::ThirtyTwo)` for any 16/32-bit document it accepts (so the
application can report the conversion), and to `None` for an 8-bit document, a
depth-1 Bitmap document, and a constructed document. For a 16/32-bit Grayscale,
RGB, Lab, or CMYK document it SHALL retain the decoded source-depth samples of
the composite color planes, the document extra channels, and every layer
channel. `write_psd` SHALL write the output header at the document's source depth
when the document retained samples, else 8. For each plane, when the retained
source-depth samples narrow to the plane's current 8-bit bytes the writer SHALL
re-encode the retained samples at the source depth; when they do not (the plane
was edited, or the layer moved) the writer SHALL widen the current 8-bit plane
to the source depth and encode it, so a save of a 16/32-bit Grayscale, RGB, Lab,
or CMYK document is not a silent downgrade to 8-bit. Widening SHALL be `v * 257`
at 16-bit and the 8-bit value scaled to `[0, 1]` at 32-bit and SHALL be
documented as an approximation that cannot recover the source low bits or HDR
range. Compression SHALL follow the document's recorded kinds at the source
depth: raw, PackBits RLE, ZIP, and ZIP-with-prediction, whose forward predictor
SHALL be the depth-specific inverse of the read predictor (a per-`u16` difference
at 16-bit and the byte difference plus the four-byte-plane shuffle at 32-bit). A
document with no recorded source depth, and a mode with no retained native
store, SHALL save as 8-bit as before.

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

#### Scenario: A normalized CMYK or Lab color mode preserves its depth

- **WHEN** a 16-bit CMYK or Lab document (whose planes the read path converts to the working RGB) is read and written unchanged
- **THEN** the output header declares bit depth 16 and header color mode CMYK (4) or Lab (9), and the retained native color planes round-trip byte-identically

#### Scenario: The psd-tools oracle reads the output depth and samples

- **WHEN** a written 16/32-bit file is decoded by `psd-tools`
- **THEN** the header depth and the native samples agree with the retained source-depth samples, and the oracle self-skips when `psd-tools` is unavailable
