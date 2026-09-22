## Why

An opened PSD can carry an embedded non-sRGB ICC profile, but the engine ignores
it: pixels are interpreted as sRGB, so a wide-gamut file (Adobe RGB, ProPhoto,
…) displays and saves with wrong colours. The image-resource parser now exposes
the profile; this change applies it.

## What Changes

- On read, when the embedded ICC profile (resource 1039) parses and is not
  sRGB, convert the composite and every layer's color channels to the sRGB
  working space and record the source profile in `Document.source_icc`.
- Remove resource 1039 from the document's preserved image resources when a
  document is ICC-normalized, so a normal (untagged, sRGB) save is not
  mis-tagged with the source profile. A document that is already sRGB (or has no
  decodable profile) is untouched and its resources stay byte-for-byte.
- Add `pictura_codec::encode_image_resources(&[ImageResource]) -> Vec<u8>`, a
  raw re-emitter used for that rewrite.
- Add `pictura-color::Profile::description()` / `is_srgb()` (lcms2 profile
  description), and a `pictura-codec → pictura-color` dependency for the lcms2
  transform.
- The app shows a status-bar "Converted from embedded ICC profile" notice.
- **BREAKING** (spec-level): `psd-opaque-preservation` gains a carve-out for the
  image-resource section of an ICC-normalized document; the byte-for-byte
  guarantee still holds for every other document.

## Capabilities

### New Capabilities

- `psd-icc-conversion`: apply the embedded ICC profile on read and keep the
  saved file consistently tagged.

### Modified Capabilities

- `psd-opaque-preservation`: the image-resource section is no longer re-emitted
  byte-for-byte when the document was normalized from a non-sRGB ICC profile.
- `psd-image-resources`: `ImageResource` gains a raw view and an encoder so the
  section can be re-serialized.

## Impact

- `crates/pictura-color`: `Profile::description`/`is_srgb`; a new
  `examples/dump_adobe_rgb.rs` that regenerates the fixture profile.
- `crates/pictura-core`: `Document.source_icc`.
- `crates/pictura-codec`: new `pictura-color` dependency, a planar ICC pass in
  `read.rs`, `image_resources.rs` raw records + encoder.
- `crates/pictura-app`: a status notice.
- Fixture `psd_icc_rgb.icc` (synthesized from `adobe_rgb()`, not Adobe's file)
  and `icc_profile.psd`; oracles.
- `docs/dev/psd-support-roadmap.md` / `STATE.md` in a separate docs commit.
