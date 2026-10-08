//! Coordinate transforms (`FILT-040`): Polar Coordinates, Shear.
//!
//! Inverse-mapping bilinear warps; alpha untouched. The exact resampling
//! and polar anchor are closed (`docs/dev/m11-distort2.md`).

use pictura_core::PixelBuffer;

use crate::kernel::{bilinear, invalid, to_u8, Edge};
use crate::{validate, FilterError, ShearFill};

/// Vertical column shift from a piecewise-linear control-point curve: per
/// column `t = (x + 0.5)/w * 2 - 1` indexes the curve and shifts down by
/// `y(t) * (h/2)` px. `fill` wraps or clamps rows off-canvas; all-zero `y` is a
/// bit-exact no-op.
pub fn shear(
    buf: &mut PixelBuffer,
    curve: &[(f64, f64)],
    fill: ShearFill,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    validate_curve(curve)?;
    let (w, h) = (buf.width as usize, buf.height as usize);
    let planes = (buf.channels as usize).min(3);
    let edge = match fill {
        ShearFill::WrapAround => Edge::Wrap,
        ShearFill::RepeatEdgePixels => Edge::Clamp,
    };
    // ponytail: the reference's shear curve interpolation/falloff is closed; this is
    // piecewise-linear between control points, clamped at the ends. Ceiling: no
    // CS6 pixel parity. Upgrade by fitting renders (oracle hook M11-B).
    let mut shifts = vec![0.0f64; w];
    for (x, s) in shifts.iter_mut().enumerate() {
        let t = (x as f64 + 0.5) / w as f64 * 2.0 - 1.0;
        *s = interp(curve, t) * (h as f64 / 2.0);
    }
    let src = buf.data.clone();
    for (x, &shift) in shifts.iter().enumerate() {
        for y in 0..h {
            let sy = y as f64 - shift;
            for c in 0..planes {
                let plane = &src[c * n..c * n + n];
                let v = bilinear(plane, w, h, x as f64, sy, edge);
                buf.data[c * n + y * w + x] = to_u8(v);
            }
        }
    }
    Ok(())
}

fn validate_curve(curve: &[(f64, f64)]) -> Result<(), FilterError> {
    if curve.len() < 2 {
        return Err(invalid(format!(
            "shear curve needs >= 2 points, got {}",
            curve.len()
        )));
    }
    for (i, &(x, y)) in curve.iter().enumerate() {
        if !x.is_finite() || !y.is_finite() {
            return Err(invalid(format!("shear curve point {i} must be finite")));
        }
        if !(-1.0..=1.0).contains(&x) || !(-1.0..=1.0).contains(&y) {
            return Err(invalid(format!(
                "shear curve point {i} must have x and y within -1.0..=1.0"
            )));
        }
        if i > 0 && x <= curve[i - 1].0 {
            return Err(invalid(format!(
                "shear curve x must strictly increase at point {i}"
            )));
        }
    }
    Ok(())
}

/// Piecewise-linear `y` at `t`, clamped outside the control-point span.
fn interp(curve: &[(f64, f64)], t: f64) -> f64 {
    let first = curve[0];
    let last = curve[curve.len() - 1];
    if t <= first.0 {
        return first.1;
    }
    if t >= last.0 {
        return last.1;
    }
    for seg in curve.windows(2) {
        let (x0, y0) = seg[0];
        let (x1, y1) = seg[1];
        if t <= x1 {
            return y0 + (y1 - y0) * (t - x0) / (x1 - x0);
        }
    }
    last.1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk<const C: usize>(w: u32, h: u32, px: &[[u8; C]]) -> PixelBuffer {
        let mut b = PixelBuffer::new(w, h, C as u8);
        let n = px.len();
        for (i, p) in px.iter().enumerate() {
            for (c, &v) in p.iter().enumerate() {
                b.data[c * n + i] = v;
            }
        }
        b
    }

    fn ramp(w: u32, h: u32) -> PixelBuffer {
        let px: Vec<[u8; 3]> = (0..(w * h) as usize)
            .map(|i| [i as u8, (i * 5) as u8, (i * 11) as u8])
            .collect();
        mk(w, h, &px)
    }

    #[test]
    fn shear_zero_curve_is_bit_exact_noop() {
        let base = ramp(16, 16);
        let mut b = base.clone();
        shear(&mut b, &[(-1.0, 0.0), (1.0, 0.0)], ShearFill::WrapAround).unwrap();
        assert_eq!(b.data, base.data);
    }

    #[test]
    fn shear_sloped_curve_shifts_and_fill_modes_differ() {
        let (w, h) = (9u32, 9u32);
        let mut px = vec![[0u8, 0, 0]; (w * h) as usize];
        for (y, chunk) in px.chunks_mut(w as usize).enumerate() {
            let v = (y * 255 / (h as usize - 1)) as u8;
            for p in chunk {
                *p = [v, v, v];
            }
        }
        let base = mk(w, h, &px);
        let curve = [(-1.0, -0.5), (1.0, 0.5)];
        let mut wrap = base.clone();
        let mut edge = base.clone();
        shear(&mut wrap, &curve, ShearFill::WrapAround).unwrap();
        shear(&mut edge, &curve, ShearFill::RepeatEdgePixels).unwrap();
        assert_ne!(wrap.data, base.data);
        assert_ne!(edge.data, base.data);
        assert_ne!(wrap.data, edge.data);
    }

    #[test]
    fn shear_validation_rejects_bad_curves() {
        let base = ramp(8, 8);
        let cases: &[&[(f64, f64)]] = &[
            &[(0.0, 0.0)],
            &[(0.0, 0.0), (0.0, 1.0)],
            &[(0.5, 0.0), (0.2, 1.0)],
            &[(0.0, 0.0), (f64::NAN, 1.0)],
            &[(0.0, 0.0), (1.0, f64::INFINITY)],
            &[(-1.5, 0.0), (1.0, 0.0)],
            &[(0.0, 0.0), (1.0, 1.5)],
        ];
        for c in cases {
            let mut b = base.clone();
            assert!(matches!(
                shear(&mut b, c, ShearFill::WrapAround),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(b.data, base.data, "buffer untouched on invalid curve");
        }
    }
}
