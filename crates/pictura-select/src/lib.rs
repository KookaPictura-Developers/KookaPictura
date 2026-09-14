//! Selection and mask math for Kooka Pictura.
//!
//! M5 scope: an 8-bit selection coverage mask and the boolean/modify operations
//! masked edits depend on. Spec: `docs/08-selection/*.md`.
//!
//! Contract owned by task M5-A. This is a stub.

use pictura_core::PixelBuffer;

#[derive(Debug, thiserror::Error)]
pub enum SelectError {
    #[error("size mismatch: {0}")]
    SizeMismatch(String),
    #[error("invalid parameters: {0}")]
    InvalidParams(String),
}

/// A document-sized 8-bit selection coverage mask (0 = outside, 255 = inside).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

/// How a new selection combines with the existing one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectOp {
    Replace,
    Add,
    Subtract,
    Intersect,
}

impl Selection {
    pub fn none(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0; width as usize * height as usize],
        }
    }

    pub fn all(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![255; width as usize * height as usize],
        }
    }

    /// Combine `other` into `self` with `op` (same dimensions required).
    pub fn combine(&mut self, _other: &Selection, _op: SelectOp) {
        unimplemented!("M5-A: selection combine")
    }

    pub fn invert(&self) -> Selection {
        unimplemented!("M5-A: selection invert")
    }

    pub fn feather(&self, _radius: f64) -> Selection {
        unimplemented!("M5-A: selection feather")
    }

    pub fn expand(&self, _radius: u32) -> Selection {
        unimplemented!("M5-A: selection expand")
    }

    pub fn contract(&self, _radius: u32) -> Selection {
        unimplemented!("M5-A: selection contract")
    }

    pub fn border(&self, _width: u32) -> Selection {
        unimplemented!("M5-A: selection border")
    }

    pub fn smooth(&self, _radius: u32) -> Selection {
        unimplemented!("M5-A: selection smooth")
    }
}

/// Flood/global selection by color tolerance (Magic Wand).
pub fn magic_wand(
    _img: &PixelBuffer,
    _x: u32,
    _y: u32,
    _tolerance: u8,
    _contiguous: bool,
) -> Selection {
    unimplemented!("M5-A: magic wand")
}

/// Expand a selection to include adjacent similar-colored pixels.
pub fn grow(_sel: &Selection, _img: &PixelBuffer, _tolerance: u8) -> Selection {
    unimplemented!("M5-A: grow")
}

/// Add all similar-colored pixels in the image to the selection.
pub fn similar(_sel: &Selection, _img: &PixelBuffer, _tolerance: u8) -> Selection {
    unimplemented!("M5-A: similar")
}

/// Select pixels within `fuzziness` of `target`.
pub fn color_range(_img: &PixelBuffer, _target: [u8; 3], _fuzziness: u8) -> Selection {
    unimplemented!("M5-A: color range")
}
