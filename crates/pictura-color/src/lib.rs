//! ICC color management for Kooka Pictura.
//!
//! M3 scope: working-space profiles (sRGB / Adobe RGB / ProPhoto), profile
//! assignment vs conversion, rendering intents, and black point compensation.
//! Spec: `docs/01-architecture/color-management.md`.
//!
//! Contract owned by task M3-A. This is a stub; `convert` is unimplemented.

/// Rendering intent (ICC).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent {
    Perceptual,
    RelativeColorimetric,
    Saturation,
    AbsoluteColorimetric,
}

#[derive(Debug, thiserror::Error)]
pub enum ColorError {
    #[error("invalid ICC profile")]
    InvalidProfile,
    #[error("unsupported: {0}")]
    Unsupported(String),
}

/// An ICC profile handle.
#[derive(Debug)]
pub struct Profile(/* lcms2 profile */ ());

/// Convert interleaved 8- or 16-bit samples between two profiles.
pub fn convert(
    _src: &Profile,
    _dst: &Profile,
    _data: &[u8],
    _width: u32,
    _height: u32,
    _channels: u8,
    _bits: u8,
    _intent: Intent,
    _black_point_compensation: bool,
) -> Result<Vec<u8>, ColorError> {
    unimplemented!("M3-A: implement ICC conversion")
}
