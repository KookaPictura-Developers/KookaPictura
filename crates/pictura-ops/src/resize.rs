//! `Image > Image Size` resampling (`IMG-001`).

use pictura_core::PixelBuffer;

use crate::{validate, OpsError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resample {
    Nearest,
    Bilinear,
    Bicubic,
}

/// Resample every channel to `width × height`. Stub until M10-A1.
pub fn resize(
    buf: &PixelBuffer,
    width: u32,
    height: u32,
    _resample: Resample,
) -> Result<PixelBuffer, OpsError> {
    validate(buf)?;
    if width == 0 || height == 0 {
        return Err(OpsError::InvalidParams(
            "width and height must be >= 1".into(),
        ));
    }
    Err(OpsError::Unsupported(
        "resize kernel not implemented (M10-A1)".into(),
    ))
}
