## Why

Every downstream adjustment, filter, and export assumes a defined working space,
but the repository had no color engine: nothing could tag or transform a document
between sRGB, Adobe RGB (1998), and ProPhoto RGB. M3 adds the ICC core so later
milestones can rely on a fixed color model instead of ad-hoc channel math.

## What Changes

- New crate `pictura-color` implementing ICC color management over the system
  Little CMS 2 (`lcms2` crate).
- Built-in working-space profiles **sRGB**, **Adobe RGB (1998)**, and
  **ProPhoto RGB**, synthesized from primaries and tone curves at runtime — no
  bundled Adobe profile files.
- ICC profile load from raw bytes (`from_icc`) and serialize back (`to_icc`),
  with malformed input returning a `ColorError`, never panicking.
- `assign` (retag, pixels unchanged) versus `convert` (pixel transform between
  two profiles).
- Four rendering intents (Perceptual, Relative Colorimetric, Saturation,
  Absolute Colorimetric) plus black point compensation on/off.
- 8-bit and 16-bit samples; 1/3/4-channel (gray / RGB / RGBA) interleaved input.
- Independent ImageMagick differential oracle (`scripts/color_oracle.py`) with a
  known-value cross-check for the sRGB → Adobe RGB primaries.

## Capabilities

### New Capabilities

- `color-management`: ICC profile construction, load, and serialization;
  profile assignment versus conversion; rendering intents and black point
  compensation; 8/16-bit multi-channel transforms; identity no-op and malformed
  input behavior; ImageMagick differential oracle tolerance.

### Modified Capabilities

- None.

## Impact

- New crate `crates/pictura-color` (depends on `thiserror` and `lcms2 = "6"`;
  dev-dependency on `pictura-testkit`).
- New oracle script `scripts/color_oracle.py` and tests under
  `crates/pictura-color/tests/`.
- Adds the `lcms2` C-library dependency to the workspace (system Little CMS 2);
  this is the one justified new dependency for M3.
- No changes to `docs/`, `pictura-core`, or the Qt shell in this change.
