//! Other family: Maximum, Minimum, Offset, High Pass, Custom
//! (`FILT-070`). Implemented by the M7-A task.

use pictura_core::PixelBuffer;

use crate::FilterError;

pub fn maximum(_buf: &mut PixelBuffer, _radius: u32) -> Result<(), FilterError> {
    Err(FilterError::Unsupported(
        "maximum not implemented yet".into(),
    ))
}

pub fn minimum(_buf: &mut PixelBuffer, _radius: u32) -> Result<(), FilterError> {
    Err(FilterError::Unsupported(
        "minimum not implemented yet".into(),
    ))
}

pub fn offset(
    _buf: &mut PixelBuffer,
    _horizontal: i32,
    _vertical: i32,
    _wrap: bool,
    _background: [u8; 3],
) -> Result<(), FilterError> {
    Err(FilterError::Unsupported(
        "offset not implemented yet".into(),
    ))
}

pub fn high_pass(_buf: &mut PixelBuffer, _radius: f64) -> Result<(), FilterError> {
    Err(FilterError::Unsupported(
        "high pass not implemented yet".into(),
    ))
}

pub fn custom(
    _buf: &mut PixelBuffer,
    _kernel: &[[f64; 5]; 5],
    _scale: f64,
    _offset: f64,
) -> Result<(), FilterError> {
    Err(FilterError::Unsupported(
        "custom not implemented yet".into(),
    ))
}
