//! Noise family. Stubs only; filter math lands in M6-D.

use pictura_core::PixelBuffer;

use crate::{FilterError, NoiseDistribution};

pub fn add(
    _buf: &mut PixelBuffer,
    _amount: f64,
    _distribution: NoiseDistribution,
    _monochromatic: bool,
    _seed: u64,
) -> Result<(), FilterError> {
    Err(FilterError::Unsupported("add not implemented yet".into()))
}

pub fn median(_buf: &mut PixelBuffer, _radius: u32) -> Result<(), FilterError> {
    Err(FilterError::Unsupported(
        "median not implemented yet".into(),
    ))
}

pub fn despeckle(_buf: &mut PixelBuffer) -> Result<(), FilterError> {
    Err(FilterError::Unsupported(
        "despeckle not implemented yet".into(),
    ))
}
