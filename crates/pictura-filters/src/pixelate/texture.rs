//! Pixelate texture filters: Mosaic, Facet, Fragment, Mezzotint.
//! Implemented by the M8-A1 task.

use pictura_core::PixelBuffer;

use crate::{FilterError, MezzotintType};

pub fn mosaic(_buf: &mut PixelBuffer, _cell_size: u32) -> Result<(), FilterError> {
    Err(FilterError::Unsupported(
        "mosaic not implemented yet".into(),
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
