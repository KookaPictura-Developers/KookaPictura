# M3 — Color Management

Goal: ICC color management core — working spaces, profile assignment vs
conversion, rendering intents, and black point compensation — so every
adjustment/filter/export downstream operates in a defined color space.

Spec: `docs/01-architecture/color-management.md` and
`docs/04-image-ops/color-profiles-and-assignment.md`.

## Scope

In:
- New crate `crates/pictura-color`.
- ICC profiles: load from bytes, serialize; built-in **sRGB**, **Adobe RGB
  (1998)**, **ProPhoto RGB** generated programmatically (no bundled Adobe files).
- `assign` (retag only, no pixel change) vs `convert` (transform pixels).
- Rendering intents: Perceptual, Relative Colorimetric, Saturation, Absolute
  Colorimetric; **black point compensation** on/off.
- 8-bit and 16-bit, 1/3/4-channel input (gray/RGB/RGBA) → RGB.
- Engine: **`lcms2`** crate (wraps the system Little CMS 2.19). This is the one
  justified new dependency: it is the standard, is already installed system-wide,
  and re-implementing ICC transforms is out of scope.

Out (later):
- Soft-proofing UI, full CMYK/Lab editing accuracy, 32-bit float color, PSD
  profile embedding (M9), display profiles / monitor calibration.

## Contract (M3-A authoritative — suggested)

```rust
// crates/pictura-color
pub enum ColorError { ... }
pub enum Intent { Perceptual, RelativeColorimetric, Saturation, AbsoluteColorimetric }

pub struct Profile(/* lcms2 handle */);
impl Profile {
    pub fn srgb() -> Profile;
    pub fn adobe_rgb() -> Profile;
    pub fn pro_photo() -> Profile;
    pub fn from_icc(bytes: &[u8]) -> Result<Profile, ColorError>;
    pub fn to_icc(&self) -> Vec<u8>;
}

/// Convert interleaved 8- or 16-bit samples from one profile to another.
pub fn convert(
    src: &Profile, dst: &Profile,
    data: &[u8], width: u32, height: u32, channels: u8, bits: u8,
    intent: Intent, black_point_compensation: bool,
) -> Result<Vec<u8>, ColorError>;

/// Retag: returns the same pixels with a new profile attached (no transform).
pub fn assign(_data: &[u8], profile: Profile) -> Profile { profile }
```

- Identity conversion (same profile) must be a no-op within ±1 LSB.
- Do not panic on malformed ICC bytes; return `ColorError`.

## Task DAG

| ID | Task | Owner | Owns | Acceptance |
|---|---|---|---|---|
| M3-A | `pictura-color` core | agent | `crates/pictura-color` | profiles + convert/assign; 8/16-bit; intent + BPC; tests |
| M3-B | Independent oracle | agent | `scripts/**`, `crates/pictura-color/tests/**` | ImageMagick `-profile` differential + known-value tests within tolerance |
| M3-C | Integrate + review | orchestrator | — | workspace green |

## Oracle

ImageMagick (`magick … -profile src.icc -profile dst.icc`) uses lcms2 too, so it
validates **our plumbing** (channel order, stride, bit depth, intent flags) rather
than the transform itself. Add known-value tests (e.g. pure red sRGB → Adobe RGB
primaries) for an engine-independent check. Report where IM and lcms2 diverge.

## Exit gate

- `cargo test --workspace` green; identity, round-trip, intent, and 16-bit tests pass.
- ImageMagick differential within tolerance for at least sRGB↔AdobeRGB and sRGB↔ProPhoto.
- `scripts/guard.sh` green.
