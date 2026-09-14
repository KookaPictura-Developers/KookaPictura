# M3-B — ImageMagick color-conversion oracle

These tests diff `pictura_color::convert` against **ImageMagick**, which links
Little CMS 2. ImageMagick therefore validates our *plumbing* — channel order,
stride, bit depth, and the intent/BPC flags — rather than the transform
algorithm (both engines delegate to lcms2). An engine-independent set of known
values for the sRGB→AdobeRGB primaries backs up the differential.

- Oracle script: `scripts/color_oracle.py`
- Tests: `crates/pictura-color/tests/oracle.rs`
- Verified against: ImageMagick **7.1.2-29 Q16-HDRI**, Little CMS **2.19**
  (system `transicc` 5.1).

## Exact ImageMagick flags

```bash
magick -size 8x8 -depth 8 rgba:IN.rgba \
    -intent <intent> [-black-point-compensation] \
    -profile SRC.icc -profile DST.icc \
    -depth 8 rgba:OUT.rgba
```

- Input is raw interleaved **RGBA8**, untagged. `-size`/`-depth 8` describe it.
- The first `-profile SRC.icc` **attaches** SRC (no pixel change); the second
  **transforms** to DST.
- `-intent` and `-black-point-compensation` are *settings*: they must precede
  the transforming `-profile`. The script places them before the first
  `-profile`, which is equivalent (the attach does not consume them).
- `-intent` accepts `perceptual`, `relative`, `saturation`, `absolute`
  (case-insensitive in IM); the script can also omit it (`undefined`).
- `-depth 8` on the output coder writes 8-bit samples. Alpha is preserved
  unchanged by the transform.

## Intent and black point compensation

ImageMagick and the M3 contract agree on the flag names/mapping. **For the
available RGB profiles the intent and BPC flags have no effect on output**:
`sRGB.icc`, `AdobeRGB1998.icc` and `ProPhotoRGB.icc` are matrix/TRC profiles
(no A2B/B2A LUTs) with matching white points and zero black points, so
perceptual, relative, saturation, absolute and BPC all produce identical bytes.
The flags are still exercised (they must parse and not corrupt pixels);
intent-sensitive coverage needs a LUT-based profile, which the RGB-only M3
scope does not include.

## Measured divergence: ImageMagick vs lcms2

Measured on the 64-pixel test image in `oracle.rs` (primaries, white/black/
gray, ramp), comparing IM's 8-bit output to `transicc -t1` (lcms2 2.19):

| Conversion | Intent | max Δ (8-bit) |
|---|---|---|
| sRGB → AdobeRGB1998 | relative, +BPC | 0 |
| sRGB → ProPhotoRGB | relative, +BPC | 0 |
| sRGB → ProPhotoRGB | absolute | 0 |
| sRGB → sRGB (identity) | relative / absolute | 0 |

ImageMagick's Q16-HDRI pipeline and lcms2's 8-bit path rounded identically on
every sample here. The test tolerance is **3 LSB** to leave room for other
lcms2 builds' 16-bit intermediate rounding; the panic message reports the
observed max delta.

## Known values (sRGB → AdobeRGB, relative colorimetric)

Derived from lcms2 2.19 (`transicc -t1`), rounded. Tolerance 3 LSB.

| sRGB in | AdobeRGB out |
|---|---|
| (0, 0, 0) | (0, 0, 0) |
| (255, 255, 255) | (255, 255, 255) |
| (255, 0, 0) | (219, 2, 0) |
| (0, 255, 0) | (144, 255, 60) |
| (0, 0, 255) | (0, 2, 250) |
| (128, 128, 128) | (127, 127, 127) |

## Running

The smoke tests (script present, `magick -version` runs, a conversion through
system ICC profiles) always run:

```bash
cargo test -p pictura-color
```

The differential and known-value tests need the M3-A API, hidden behind the
non-default `m3a` feature and `#[ignore]`d:

```bash
cargo test -p pictura-color --features m3a -- --ignored
```

Manual inspection:

```bash
python3 scripts/color_oracle.py version         # magick + lcms version
python3 scripts/color_oracle.py convert --help
python3 scripts/color_oracle.py convert --src SRC.icc --dst DST.icc \
    [--intent relative] [--bpc] [--size 8x8] IN.rgba OUT.rgba
```

## Not expressed by this oracle

- **16-bit.** Only 8-bit RGBA is handled: IM's raw coder writes 16-bit samples
  big-endian and the M3 contract's raw fixtures do not exercise that yet.
- **1/3-channel input.** The oracle always uses 4-channel RGBA (alpha is a
  passthrough check); gray/RGB→RGB plumbing is covered by M3-A unit tests.
- **LUT-based intent differences.** See "Intent" above.
