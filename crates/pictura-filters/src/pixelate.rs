//! Pixelate family: Color Halftone, Crystallize, Facet, Fragment, Mezzotint,
//! Mosaic, Pointillize (`FILT-084`). Implemented by the M8-A task.

use pictura_core::PixelBuffer;

use crate::{FilterError, MezzotintType};

pub fn mosaic(_buf: &mut PixelBuffer, _cell_size: u32) -> Result<(), FilterError> {
    Err(FilterError::Unsupported(
        "mosaic not implemented yet".into(),
    ))
}

pub fn crystallize(_buf: &mut PixelBuffer, _cell_size: u32, _seed: u64) -> Result<(), FilterError> {
    Err(FilterError::Unsupported(
        "crystallize not implemented yet".into(),
    ))
}

pub fn facet(_buf: &mut PixelBuffer) -> Result<(), FilterError> {
    Err(FilterError::Unsupported("facet not implemented yet".into()))
}

pub fn fragment(_buf: &mut PixelBuffer) -> Result<(), FilterError> {
    Err(FilterError::Unsupported(
        "fragment not implemented yet".into(),
    ))
}

pub fn mezzotint(
    _buf: &mut PixelBuffer,
    _kind: MezzotintType,
    _seed: u64,
) -> Result<(), FilterError> {
    Err(FilterError::Unsupported(
        "mezzotint not implemented yet".into(),
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
