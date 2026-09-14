//! Ripple warps (`FILT-040`): ZigZag, Ocean Ripple.
//!
//! Scaffold for M11-A2. ZigZag is a deterministic radial displacement; Ocean
//! Ripple is a seeded random ripple. Both are stubs for now.

use pictura_core::PixelBuffer;

use crate::{FilterError, ZigZagStyle};

pub fn zigzag(
    buf: &mut PixelBuffer,
    amount: f64,
    ridges: u32,
    style: ZigZagStyle,
) -> Result<(), FilterError> {
    let _ = (buf, amount, ridges, style);
    Err(FilterError::Unsupported(
        "zigzag not implemented yet".into(),
    ))
}

pub fn ocean_ripple(
    buf: &mut PixelBuffer,
    size: u32,
    magnitude: u32,
    seed: u64,
) -> Result<(), FilterError> {
    let _ = (buf, size, magnitude, seed);
    Err(FilterError::Unsupported(
        "ocean_ripple not implemented yet".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stubs_report_unsupported() {
        let mut b = PixelBuffer::new(4, 4, 3);
        assert!(matches!(
            zigzag(&mut b, 50.0, 5, ZigZagStyle::AroundCenter),
            Err(FilterError::Unsupported(_))
        ));
        assert!(matches!(
            ocean_ripple(&mut b, 9, 5, 1),
            Err(FilterError::Unsupported(_))
        ));
    }
}
