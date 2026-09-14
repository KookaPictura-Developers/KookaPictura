//! Pixelate cell filters: Crystallize, Pointillize, Color Halftone.
//! Implemented by the M8-A2 task.

use pictura_core::PixelBuffer;

use crate::FilterError;

pub fn crystallize(_buf: &mut PixelBuffer, _cell_size: u32, _seed: u64) -> Result<(), FilterError> {
    Err(FilterError::Unsupported(
        "crystallize not implemented yet".into(),
    ))
}

pub fn pointillize(
    _buf: &mut PixelBuffer,
    _cell_size: u32,
    _background: [u8; 3],
    _seed: u64,
) -> Result<(), FilterError> {
    Err(FilterError::Unsupported(
        "pointillize not implemented yet".into(),
    ))
}

pub fn color_halftone(
    _buf: &mut PixelBuffer,
    _max_radius: u32,
    _angles: [f64; 4],
) -> Result<(), FilterError> {
    Err(FilterError::Unsupported(
        "color halftone not implemented yet".into(),
    ))
}
