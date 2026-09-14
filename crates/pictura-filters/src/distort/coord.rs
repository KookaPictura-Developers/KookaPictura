//! Coordinate transforms (`FILT-040`): Polar Coordinates, Shear.
//!
//! Inverse-mapping bilinear warps; alpha untouched. Adobe's exact resampling
//! and polar anchor are closed (`docs/dev/m11-distort2.md`).

use pictura_core::PixelBuffer;

use crate::kernel::clamp_index;
use crate::{validate, FilterError, PolarKind, ShearFill};

/// Edge policy for a source coordinate that falls outside the image.
#[derive(Clone, Copy)]
enum Edge {
    Clamp,
    Wrap,
}

/// Rectangular ⇄ polar transform about the center: `RectangularToPolar` turns a
/// horizontal source line into a constant-radius ring; `PolarToRectangular` is
/// its inverse. Clamp-to-edge; deterministic.
pub fn polar_coordinates(buf: &mut PixelBuffer, kind: PolarKind) -> Result<(), FilterError> {
    let n = validate(buf)?;
    let (w, h) = (buf.width as usize, buf.height as usize);
    let planes = (buf.channels as usize).min(3);
    let (cx, cy) = ((w as f64 - 1.0) / 2.0, (h as f64 - 1.0) / 2.0);
    // ponytail: Adobe's polar center/scale anchor is closed; center = image
    // midpoint, radius = corner distance. Ceiling: no CS6 pixel parity. Upgrade
    // by fitting renders (oracle hook M9-B/M11-B).
    let max_r = ((cx * cx + cy * cy).sqrt()).max(f64::MIN_POSITIVE);
    let src = buf.data.clone();
    for y in 0..h {
        for x in 0..w {
            let (sx, sy) = match kind {
                PolarKind::RectangularToPolar => {
                    let dx = x as f64 - cx;
                    let dy = y as f64 - cy;
                    let r = (dx * dx + dy * dy).sqrt();
                    let theta = dy.atan2(dx).rem_euclid(std::f64::consts::TAU);
                    (
                        theta / std::f64::consts::TAU * w as f64,
                        r / max_r * h as f64,
                    )
                }
                PolarKind::PolarToRectangular => {
                    let theta = (x as f64 + 0.5) / w as f64 * std::f64::consts::TAU;
                    let r = (y as f64 + 0.5) / h as f64 * max_r;
                    (cx + r * theta.cos(), cy + r * theta.sin())
                }
            };
            for c in 0..planes {
                let plane = &src[c * n..c * n + n];
                let v = sample(plane, w, h, sx, sy, Edge::Clamp);
                buf.data[c * n + y * w + x] = to_u8(v);
            }
        }
    }
    Ok(())
}

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
    // ponytail: Adobe's shear curve interpolation/falloff is closed; this is
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
                let v = sample(plane, w, h, x as f64, sy, edge);
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

/// Bilinear sample at `(x, y)` under `edge`.
fn sample(plane: &[u8], w: usize, h: usize, x: f64, y: f64, edge: Edge) -> f64 {
    let x0 = x.floor();
    let y0 = y.floor();
    let (fx, fy) = (x - x0, y - y0);
    let (i0, j0) = (x0 as isize, y0 as isize);
    let (xi, xi1) = (idx(i0, w, edge), idx(i0 + 1, w, edge));
    let (yi, yi1) = (idx(j0, h, edge), idx(j0 + 1, h, edge));
    let p00 = plane[yi * w + xi] as f64;
    let p10 = plane[yi * w + xi1] as f64;
    let p01 = plane[yi1 * w + xi] as f64;
    let p11 = plane[yi1 * w + xi1] as f64;
    (p00 * (1.0 - fx) + p10 * fx) * (1.0 - fy) + (p01 * (1.0 - fx) + p11 * fx) * fy
}

fn idx(i: isize, n: usize, edge: Edge) -> usize {
    match edge {
        Edge::Clamp => clamp_index(i, n),
        Edge::Wrap => i.rem_euclid(n as isize) as usize,
    }
}

fn invalid(msg: String) -> FilterError {
    FilterError::InvalidParams(msg)
}

fn to_u8(v: f64) -> u8 {
    v.round().clamp(0.0, 255.0) as u8
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
    fn polar_remaps_and_kinds_differ() {
        let base = ramp(17, 13);
        let mut r2p = base.clone();
        let mut p2r = base.clone();
        polar_coordinates(&mut r2p, PolarKind::RectangularToPolar).unwrap();
        polar_coordinates(&mut p2r, PolarKind::PolarToRectangular).unwrap();
        assert_ne!(r2p.data, base.data);
        assert_ne!(p2r.data, base.data);
        assert_ne!(r2p.data, p2r.data);
    }

    #[test]
    fn polar_horizontal_line_maps_to_constant_radius_ring() {
        let (w, h) = (65u32, 65u32);
        let mut px = vec![[0u8, 0, 0]; (w * h) as usize];
        let row = 34usize;
        let start = row * w as usize;
        for p in &mut px[start..start + w as usize] {
            *p = [255, 255, 255];
        }
        let mut b = mk(w, h, &px);
        polar_coordinates(&mut b, PolarKind::RectangularToPolar).unwrap();
        let c = 32i64;
        let at = |x: i64, y: i64| b.data[(y * w as i64 + x) as usize];
        let ring = at(c + 24, c);
        assert_eq!(ring, at(c - 24, c));
        assert_eq!(ring, at(c, c - 24));
        assert_eq!(ring, at(c, c + 24));
        assert!(
            ring > 100,
            "ring samples the bright source row (got {ring})"
        );
        assert_eq!(at(c, c), 0, "center maps to the dark top row");
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

    #[test]
    fn alpha_preserved_and_tiny_images_do_not_panic() {
        let px: Vec<[u8; 4]> = (0..12 * 12)
            .map(|i| [i as u8, (i * 3) as u8, (i * 7) as u8, (i * 11) as u8])
            .collect();
        let base = mk(12, 12, &px);
        let n = base.pixel_count();
        let alpha = base.data[3 * n..].to_vec();
        let mut p = base.clone();
        polar_coordinates(&mut p, PolarKind::RectangularToPolar).unwrap();
        assert_eq!(&p.data[3 * n..], &alpha[..]);
        let mut s = base.clone();
        shear(&mut s, &[(-1.0, -1.0), (1.0, 1.0)], ShearFill::WrapAround).unwrap();
        assert_eq!(&s.data[3 * n..], &alpha[..], "alpha untouched");

        let one = mk(1, 1, &[[100, 150, 200, 255]]);
        for kind in [PolarKind::RectangularToPolar, PolarKind::PolarToRectangular] {
            let mut b = one.clone();
            polar_coordinates(&mut b, kind).unwrap();
        }
        let mut sh = one.clone();
        shear(
            &mut sh,
            &[(-1.0, 0.0), (1.0, 1.0)],
            ShearFill::RepeatEdgePixels,
        )
        .unwrap();

        for &(w, h) in &[(1u32, 8u32), (8, 1)] {
            let px: Vec<[u8; 3]> = (0..(w * h) as usize)
                .map(|i| [i as u8, 0, 255 - i as u8])
                .collect();
            let base = mk(w, h, &px);
            let mut a = base.clone();
            assert!(polar_coordinates(&mut a, PolarKind::PolarToRectangular).is_ok());
            let mut b = base.clone();
            assert!(shear(
                &mut b,
                &[(-1.0, 0.5), (0.0, -0.5), (1.0, 0.5)],
                ShearFill::WrapAround,
            )
            .is_ok());
        }
    }
}
