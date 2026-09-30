# Spec Delta

## MODIFIED Requirements

### Requirement: The application decodes images to RGBA8888 at the Qt boundary

The application SHALL decode a supported raster file to packed RGBA8888 using
Qt (`QImage`/`QImageReader`/`QImage::fromData` or their cxx-qt binding) and SHALL
NOT pass a Qt type into the engine. Qt's runtime decoders are the authoritative
format allow-list; consulting `QImageReader::supportedImageFormats()` is
optional. The header probe is a fast-path guard only: it can pre-empt a
recognized header that exceeds the budget, but a container the probe does not
recognize SHALL still be attempted through Qt. The decode edge SHALL refuse —
returning failure without constructing engine structures — when the file is
missing or unreadable, when Qt cannot decode it, when a *recognized* header
probe refuses it, or when the actual decoded allocation
(`decoded width * height * 4`) exceeds the budget. The decode edge SHALL copy the
decoded RGBA rows to the engine buffer in bulk — a row or the whole frame at a
time — and MUST NOT copy one byte at a time across the bridge. Only frame 0 of a
multi-frame file SHALL be imported. For v1 an undecodable file SHALL be refused,
not replaced by a placeholder.

#### Scenario: A supported file decodes to packed RGBA8888

- **WHEN** the decode edge is given a supported image file
- **THEN** it returns packed RGBA8888 bytes matching the decoded width and height

#### Scenario: A Qt-decodable container unknown to the probe is imported

- **WHEN** the decode edge is given a valid image whose container the probe does not recognize but Qt can decode
- **THEN** Qt decodes it, the actual decoded allocation is capped, and the import succeeds

#### Scenario: An undecodable file is refused

- **WHEN** the decode edge is given a file Qt cannot read
- **THEN** it returns failure and no document or layer is constructed

#### Scenario: The actual decoded allocation is capped

- **WHEN** a file's declared header is within budget but its decoded allocation exceeds the budget
- **THEN** the decode edge refuses before any engine structure is constructed

#### Scenario: The decoded rows are copied in bulk

- **WHEN** a decoded image is handed to the engine
- **THEN** its rows are copied with bulk copies and the import's cost does not grow with a per-byte copy loop

#### Scenario: Only the first frame is imported

- **WHEN** a multi-frame GIF or APNG is decoded
- **THEN** only frame 0 is imported
