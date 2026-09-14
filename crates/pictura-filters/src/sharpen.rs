//! Sharpen family. Stubs only; filter math lands in M6-C.

use pictura_core::PixelBuffer;

use crate::FilterError;

pub fn sharpen(_buf: &mut PixelBuffer) -> Result<(), FilterError> {
    Err(FilterError::Unsupported(
        "sharpen not implemented yet".into(),
    ))
}

pub fn sharpen_more(_buf: &mut PixelBuffer) -> Result<(), FilterError> {
    Err(FilterError::Unsupported(
        "sharpen_more not implemented yet".into(),
    ))
}

pub fn edges(_buf: &mut PixelBuffer) -> Result<(), FilterError> {
    Err(FilterError::Unsupported("edges not implemented yet".into()))
}

pub fn unsharp_mask(
    _buf: &mut PixelBuffer,
    _amount: f64,
    _radius: f64,
    _threshold: u8,
) -> Result<(), FilterError> {
    Err(FilterError::Unsupported(
        "unsharp_mask not implemented yet".into(),
    ))
}
