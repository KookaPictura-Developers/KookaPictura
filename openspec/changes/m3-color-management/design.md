## Context

M3 sits between the document model (M1) and the adjustment/render stack
(M4+). Every downstream operation needs a defined working space, so the crate
must be able to tag (assign) and transform (convert) pixel data between sRGB,
Adobe RGB (1998), and ProPhoto RGB. The long-form contract is
`docs/01-architecture/color-management.md` (`ARCH-007`) and
`docs/04-image-ops/color-profiles-and-assignment.md` (`IMG-006`); the milestone
brief is `docs/dev/m3-color-management.md`.

The previous state had no color engine at all. ICC profile parsing and
transform math are large, subtle, and standardized; the project's independent-creation
rule forbids shipping Adobe profile files, so built-ins must be synthesized.

## Goals / Non-Goals

**Goals:**

- A small ICC core: build working-space profiles, load/serialize ICC bytes,
  assign versus convert, all four rendering intents, black point compensation,
  8/16-bit, and 1/3/4-channel input.
- Built-in sRGB, Adobe RGB (1998), and ProPhoto RGB generated from primaries and
  tone curves at runtime (no bundled Adobe files).
- A panic-free API: malformed input and unsupported formats return `ColorError`.
- An independent ImageMagick differential oracle plus engine-independent known
  values, so plumbing (channel order, stride, depth, flags) is verified.

**Non-Goals:**

- Soft-proofing UI, gamut warning, display/monitor profiles, and CMYK/Lab
  accuracy (later milestones).
- 32-bit float conversion and PSD profile embedding (M9).
- Bit-exact parity with Adobe ACE. ACE's gamut mapping and BPC numerics are
  closed; parity is behavioral only.
- LUT-based (A2B/B2A) profiles for the built-ins, which is why intent/BPC are
  inert here (see Risks).

## Decisions

### Use the system `lcms2` crate over a hand-rolled or pure-Rust CMS

Implementation wraps Little CMS 2 through the `lcms2` crate. It is the standard
ICC engine, is already installed system-wide, and supplies profile construction,
transform, and flag handling. Re-implementing ICC parsing and transforms is out
of scope and would not be verifiable against a reference. Alternative
considered: `moxcms` (the `image` crate's pure-Rust CMS) — rejected because it
does not replace a full CMS and would force re-deriving built-in profile bytes.
This is the one new dependency for M3 and is recorded in the proposal.

### Synthesize built-ins from primaries and tone curves

sRGB is delegated to `lcms2::Profile::new_srgb()`. Adobe RGB (1998) is built
with `new_rgb` from its D65 white point, primaries, and gamma 563/256. ProPhoto
RGB uses the ROMM D50 white point, ROMM primaries, and the ROMM tone curve,
which has a linear toe below 1/32 followed by gamma 1.8; lcms2 has no
parametric type for it, so the encoded→linear mapping is tabulated into a 4096
entry `ToneCurve`.

### `assign` returns pixels plus profile; `convert` uses lcms2 pixel formats

`assign(data, profile) -> (Vec<u8>, Profile)` copies the bytes and returns the
new profile, documenting explicitly that no transform runs. `convert` maps
`(channels, bits)` to lcms2 `PixelFormat` values (`GRAY_8/16`, `RGB_8/16`,
`RGBA_8/16`), validates the buffer length up front (with checked arithmetic to
avoid overflow), sets `BLACKPOINT_COMPENSATION` when requested and `COPY_ALPHA`
for 4-channel input, and runs `Transform<u8, u8>` directly on the byte buffer so
no endianness or alignment juggling is needed.

### ImageMagick differential oracle with an explicit tolerance

ImageMagick links lcms2, so `magick … -profile src -profile dst` validates our
plumbing rather than the transform algorithm. The differential test allows 3
8-bit LSB per sample (observed divergence is 0 on the tested profiles; the
margin covers 16-bit intermediate rounding in other lcms2 builds). A separate
known-value test pins the sRGB → Adobe RGB primaries, an engine-independent
check. The oracle script handles only 8-bit RGBA because ImageMagick's raw coder
writes 16-bit big-endian, which the M3 fixture contract does not exercise.

## Risks / Trade-offs

- **Intent and BPC are inert for the built-in RGB profiles.** sRGB, Adobe RGB,
  and ProPhoto are matrix/TRC profiles with matching white points and zero black
  points, so Perceptual, Relative, Saturation, Absolute, and BPC all produce
  identical bytes. Intent-sensitive behavior requires LUT-based (A2B/B2A)
  profiles, which the RGB-only M3 scope excludes. This is recorded in
  `crates/pictura-color/tests/README.md`. → Mitigation: the flags are still
  built and exercised (they must parse and not corrupt pixels); a LUT-profile
  test is future work when CMYK/print support lands.
- **New C dependency (`lcms2` / system Little CMS 2).** Build and runtime now
  depend on the system library. → Accepted: it is already installed and is the
  reference engine; the crate feature-gates nothing and pins `lcms2 = "6"`.
- **Differential oracle is not fully independent.** Both engines delegate to
  lcms2. → Mitigation: the known-value test provides engine-independent
  ground truth, and the README documents the measured divergence.
- **16-bit oracle gap.** The oracle only checks 8-bit RGBA; 16-bit and
  1/3-channel paths are covered by unit tests in `lib.rs`, not differentially.
  → Accepted for this milestone.
- **Absolute Colorimetric + BPC is legal but semantically wrong for proofing.**
  The API does not auto-disable BPC; callers own that policy (documented in
  `ARCH-007`). → Mitigation: a later document-operation layer enforces it.
