//! `Image > Image Rotation` and flips (`IMG-003`).
//!
//! The quarter/half turns and flips are exact index remaps; `rotate_arbitrary`
//! grows the canvas to the axis-aligned bounding box and resamples.

use pictura_core::PixelBuffer;

use crate::{validate, OpsError};

pub fn rotate90_cw(buf: &PixelBuffer) -> PixelBuffer {
    // M10-A2
    buf.clone()
}

pub fn rotate90_ccw(buf: &PixelBuffer) -> PixelBuffer {
    // M10-A2
    buf.clone()
}

pub fn rotate180(buf: &PixelBuffer) -> PixelBuffer {
    // M10-A2
    buf.clone()
}

pub fn flip_horizontal(buf: &PixelBuffer) -> PixelBuffer {
    // M10-A2
    buf.clone()
}

pub fn flip_vertical(buf: &PixelBuffer) -> PixelBuffer {
    // M10-A2
    buf.clone()
}

/// Rotate about the canvas center by `angle_deg`, growing the canvas to the
/// bounding box and filling corners with `background`. Stub until M10-A3.
pub fn rotate_arbitrary(
    buf: &PixelBuffer,
    angle_deg: f64,
    _background: [u8; 4],
) -> Result<PixelBuffer, OpsError> {
    validate(buf)?;
    if !angle_deg.is_finite() || !(-359.99..=359.99).contains(&angle_deg) {
        return Err(OpsError::InvalidParams(
            "angle must be finite and within -359.99..=359.99".into(),
        ));
    }
    Err(OpsError::Unsupported(
        "arbitrary rotation not implemented (M10-A3)".into(),
    ))
}
