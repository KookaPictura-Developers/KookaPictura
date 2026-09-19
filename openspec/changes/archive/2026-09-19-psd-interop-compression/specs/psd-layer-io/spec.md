## MODIFIED Requirements

### Requirement: Channel image data supports raw and PackBits RLE
Layer channel image data SHALL support compression code `0` (raw), `1`
(PackBits/RLE), `2` (ZIP), and `3` (ZIP-with-prediction). RLE scanline
byte-count entries SHALL be 2 bytes in PSD and 4 bytes in PSB. RLE decoding
SHALL be lossless and SHALL reproduce the raw bytes exactly. ZIP is a
zlib-framed deflate stream (a raw-deflate stream SHALL also be accepted); for
code `3` the byte-wise per-scanline delta SHALL be inverted after inflating,
using the channel width as the scanline length. Any other compression code SHALL
return an unsupported error, and a malformed deflate stream SHALL return a typed
error without panicking.

#### Scenario: RLE decodes to the raw bytes
- **WHEN** a channel is encoded with PackBits literal and repeat runs
- **THEN** decoding yields the original byte sequence

#### Scenario: PSB uses four-byte scanline counts
- **WHEN** a PSB (`version == 2`) channel is RLE-decoded
- **THEN** the scanline count table is read as 4-byte entries and the row decodes correctly

#### Scenario: ZIP channel decodes
- **WHEN** a layer channel declares compression 2 and holds a deflate stream
- **THEN** the channel bytes are the inflated payload

#### Scenario: ZIP-with-prediction channel decodes
- **WHEN** a layer channel declares compression 3
- **THEN** the inflated bytes have the per-row delta inverted to recover the channel

#### Scenario: ZIP compression is rejected, not misread
- **WHEN** a layer channel declares an unknown compression code
- **THEN** `read_psd` returns `PsdError::Unsupported` and does not panic

## ADDED Requirements

### Requirement: Unknown blend keys degrade to Normal
A layer whose 4-byte blend key is not one of the 27 modes or `pass` SHALL be
read with `BlendMode::Normal` instead of failing the whole document. A blend
signature that is not `8BIM` SHALL still be an error. The original key is not
preserved by this change; opaque preservation is a later roadmap phase.

#### Scenario: Unknown key does not abort the file
- **WHEN** a layer record carries an unrecognized blend key such as `zzzz`
- **THEN** `read_psd` succeeds and that layer's blend is `BlendMode::Normal`

#### Scenario: Bad blend signature still errors
- **WHEN** a layer record's blend signature is not `8BIM`
- **THEN** `read_psd` returns a typed error
