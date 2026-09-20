## MODIFIED Requirements

### Requirement: Non-RGB color modes open by normalizing to the working mode

`read_psd` SHALL accept header color modes Bitmap (0), Grayscale (1), Indexed
(2), RGB (3), CMYK (4), and Lab (9). It SHALL convert a non-RGB mode's color
planes into the engine's working mode (RGB for Bitmap/Indexed/CMYK/Lab,
Grayscale unchanged) and SHALL set the returned document's `mode` to `Rgb` or
`Grayscale` and its `depth` to `Eight`. A header color mode of Multichannel (7)
or Duotone (8), any other mode code, and any bit depth other than 8 (other than
depth 1 for Bitmap, and depths 16/32 for the Grayscale/RGB/CMYK/Lab modes read
per `psd-bit-depth`) SHALL return `PsdError::Unsupported`. A header channel
count below the mode's color-channel count SHALL return `PsdError::Invalid`.

#### Scenario: An Indexed document opens as RGB with palette colors

- **WHEN** an 8-bit Indexed PSD whose indices select distinct palette entries is read
- **THEN** `read_psd` returns a document whose `mode` is `Rgb` and whose composite pixels are the palette colors for those indices

#### Scenario: A CMYK document opens as RGB

- **WHEN** an 8-bit CMYK PSD is read
- **THEN** `read_psd` returns a document whose `mode` is `Rgb`, whose `depth` is `Eight`, and whose composite is the profile-free CMYK-to-RGB conversion of the stored planes

#### Scenario: A Lab document opens as RGB

- **WHEN** an 8-bit Lab PSD is read
- **THEN** `read_psd` returns a document whose `mode` is `Rgb` and whose composite is the profile-free CIELAB-to-sRGB conversion of the stored planes

#### Scenario: A Bitmap document opens as black and white RGB

- **WHEN** a depth-1 Bitmap PSD is read
- **THEN** `read_psd` returns an RGB document whose pixels are black where the bit is set and white where it is clear

#### Scenario: Grayscale and RGB are unchanged

- **WHEN** an 8-bit Grayscale or RGB PSD is read
- **THEN** `read_psd` returns a document whose mode is `Grayscale` or `Rgb` respectively, exactly as before this change

#### Scenario: Multichannel and Duotone remain unsupported

- **WHEN** the header color mode is Multichannel (7) or Duotone (8)
- **THEN** `read_psd` returns `PsdError::Unsupported` and does not panic

#### Scenario: Depth 16 and 32 are normalized with the mode

- **WHEN** a 16- or 32-bit CMYK or Lab document is read
- **THEN** the samples are narrowed to 8-bit and the color planes are converted to the working mode per this requirement and `psd-bit-depth`
