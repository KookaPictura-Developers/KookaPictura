//! `Image > Canvas Size` (`IMG-002`).

use pictura_core::PixelBuffer;

use crate::{validate, OpsError};

/// 3×3 anchor grid, read row-major top-to-bottom, left-to-right.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

/// Grow or crop the canvas to `width × height`, placing the source per `anchor`.
/// Stub until M10-A2.
pub fn resize_canvas(
    buf: &PixelBuffer,
    width: u32,
    height: u32,
    _anchor: Anchor,
    _background: [u8; 4],
) -> Result<PixelBuffer, OpsError> {
    validate(buf)?;
    if width == 0 || height == 0 {
        return Err(OpsError::InvalidParams(
            "width and height must be >= 1".into(),
        ));
    }
    Err(OpsError::Unsupported(
        "canvas resize not implemented (M10-A2)".into(),
    ))
}
