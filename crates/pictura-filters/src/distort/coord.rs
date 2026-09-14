//! Coordinate transforms (`FILT-040`): Polar Coordinates, Shear.
//!
//! Scaffold for M11-A1. Both are inverse-mapping warps with bilinear
//! resampling; the functions are stubs so the `Filter` contract and dispatch
//! compile before the math lands.

use pictura_core::PixelBuffer;

use crate::{FilterError, PolarKind, ShearFill};

pub fn polar_coordinates(buf: &mut PixelBuffer, kind: PolarKind) -> Result<(), FilterError> {
    let _ = (buf, kind);
    Err(FilterError::Unsupported(
        "polar_coordinates not implemented yet".into(),
    ))
}

pub fn shear(
    buf: &mut PixelBuffer,
    curve: &[(f64, f64)],
    fill: ShearFill,
) -> Result<(), FilterError> {
    let _ = (buf, curve, fill);
    Err(FilterError::Unsupported("shear not implemented yet".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stubs_report_unsupported() {
        let mut b = PixelBuffer::new(4, 4, 3);
        assert!(matches!(
            polar_coordinates(&mut b, PolarKind::RectangularToPolar),
            Err(FilterError::Unsupported(_))
        ));
        assert!(matches!(
            shear(&mut b, &[(0.0, 0.0), (1.0, 1.0)], ShearFill::WrapAround),
            Err(FilterError::Unsupported(_))
        ));
    }
}
