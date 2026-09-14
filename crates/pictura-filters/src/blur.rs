//! Blur family. Stubs only; filter math lands in M6-B.

use pictura_core::PixelBuffer;

use crate::{FilterError, Quality, RadialMethod};

pub fn gaussian(_buf: &mut PixelBuffer, _radius: f64) -> Result<(), FilterError> {
    Err(FilterError::Unsupported(
        "gaussian not implemented yet".into(),
    ))
}

pub fn r#box(_buf: &mut PixelBuffer, _radius: u32) -> Result<(), FilterError> {
    Err(FilterError::Unsupported("box not implemented yet".into()))
}

pub fn motion(_buf: &mut PixelBuffer, _angle_deg: f64, _distance: u32) -> Result<(), FilterError> {
    Err(FilterError::Unsupported(
        "motion not implemented yet".into(),
    ))
}

pub fn radial(
    _buf: &mut PixelBuffer,
    _method: RadialMethod,
    _amount: f64,
    _quality: Quality,
) -> Result<(), FilterError> {
    Err(FilterError::Unsupported(
        "radial not implemented yet".into(),
    ))
}

pub fn average(_buf: &mut PixelBuffer) -> Result<(), FilterError> {
    Err(FilterError::Unsupported(
        "average not implemented yet".into(),
    ))
}

pub fn simple(_buf: &mut PixelBuffer, _more: bool) -> Result<(), FilterError> {
    Err(FilterError::Unsupported(
        "simple not implemented yet".into(),
    ))
}

pub fn surface(_buf: &mut PixelBuffer, _radius: u32, _threshold: u8) -> Result<(), FilterError> {
    Err(FilterError::Unsupported(
        "surface not implemented yet".into(),
    ))
}
