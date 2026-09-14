//! Radial warps (`FILT-040`): Twirl, Pinch, Spherize.

use pictura_core::PixelBuffer;

use crate::{FilterError, SpherizeMode};

pub fn twirl(_buf: &mut PixelBuffer, _angle: f64) -> Result<(), FilterError> {
    Err(FilterError::Unsupported("twirl not implemented yet".into()))
}

pub fn pinch(_buf: &mut PixelBuffer, _amount: f64) -> Result<(), FilterError> {
    Err(FilterError::Unsupported("pinch not implemented yet".into()))
}

pub fn spherize(
    _buf: &mut PixelBuffer,
    _amount: f64,
    _mode: SpherizeMode,
) -> Result<(), FilterError> {
    Err(FilterError::Unsupported(
        "spherize not implemented yet".into(),
    ))
}
