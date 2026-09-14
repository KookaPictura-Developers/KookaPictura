//! Undulating warps (`FILT-040`): Ripple, Wave.

use pictura_core::PixelBuffer;

use crate::{FilterError, RippleSize, WaveType};

pub fn ripple(_buf: &mut PixelBuffer, _amount: f64, _size: RippleSize) -> Result<(), FilterError> {
    Err(FilterError::Unsupported(
        "ripple not implemented yet".into(),
    ))
}

#[allow(clippy::too_many_arguments)]
pub fn wave(
    _buf: &mut PixelBuffer,
    _generators: u32,
    _wavelength: (f64, f64),
    _amplitude: (f64, f64),
    _kind: WaveType,
    _scale: (f64, f64),
    _seed: u64,
    _repeat_edge: bool,
) -> Result<(), FilterError> {
    Err(FilterError::Unsupported("wave not implemented yet".into()))
}
