//! Stylize family: Emboss, Find Edges, Solarize (`FILT-050`). Implemented by
//! the M7-B task.

use pictura_core::PixelBuffer;

use crate::FilterError;

pub fn emboss(
    _buf: &mut PixelBuffer,
    _angle: f64,
    _height: f64,
    _amount: f64,
) -> Result<(), FilterError> {
    Err(FilterError::Unsupported(
        "emboss not implemented yet".into(),
    ))
}

pub fn find_edges(_buf: &mut PixelBuffer) -> Result<(), FilterError> {
    Err(FilterError::Unsupported(
        "find edges not implemented yet".into(),
    ))
}

pub fn solarize(_buf: &mut PixelBuffer) -> Result<(), FilterError> {
    Err(FilterError::Unsupported(
        "solarize not implemented yet".into(),
    ))
}
