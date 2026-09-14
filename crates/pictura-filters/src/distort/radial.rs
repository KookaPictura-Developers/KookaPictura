//! Radial warps (`FILT-040`): Twirl, Pinch, Spherize.
//!
//! Inverse mapping: each output pixel's source offset is computed from the image
//! center `((w-1)/2, (h-1)/2)`, resampled bilinearly, clamp-to-edge. Only color
//! planes are written; alpha is untouched.

use pictura_core::PixelBuffer;

use crate::kernel::clamp_index;
use crate::{validate, FilterError, SpherizeMode};

fn check(name: &str, v: f64, range: std::ops::RangeInclusive<f64>) -> Result<(), FilterError> {
    if !v.is_finite() || !range.contains(&v) {
        return Err(FilterError::InvalidParams(format!(
            "{name} {v} must be finite and within {range:?}"
        )));
    }
    Ok(())
}

pub fn twirl(buf: &mut PixelBuffer, angle: f64) -> Result<(), FilterError> {
    validate(buf)?;
    check("twirl angle", angle, -999.0..=999.0)?;
    if angle == 0.0 {
        return Ok(());
    }
    let radians = angle.to_radians();
    // ponytail: linear angular falloff, not Adobe's closed falloff/resampling
    // (behavioral parity only). Upgrade to a fitted falloff once CS6 renders exist.
    warp(buf, |px, py, rx, ry| {
        let rmax = rx.max(ry);
        if rmax <= 0.0 {
            return (px, py);
        }
        let r = (px * px + py * py).sqrt();
        let (s, c) = (radians * (1.0 - r / rmax).clamp(0.0, 1.0)).sin_cos();
        (px * c - py * s, px * s + py * c)
    })
}

pub fn pinch(buf: &mut PixelBuffer, amount: f64) -> Result<(), FilterError> {
    validate(buf)?;
    check("pinch amount", amount, -100.0..=100.0)?;
    if amount == 0.0 {
        return Ok(());
    }
    let k = amount / 100.0;
    // ponytail: monotone linear radial remap, not Adobe's closed falloff and
    // resampling (behavioral parity only). Upgrade once CS6 renders exist.
    warp(buf, |px, py, rx, ry| {
        let rmax = rx.max(ry);
        let r = (px * px + py * py).sqrt();
        if r == 0.0 || rmax <= 0.0 {
            return (px, py);
        }
        let f = 1.0 + k * (1.0 - (r / rmax).clamp(0.0, 1.0));
        (px * f, py * f)
    })
}

pub fn spherize(buf: &mut PixelBuffer, amount: f64, mode: SpherizeMode) -> Result<(), FilterError> {
    validate(buf)?;
    check("spherize amount", amount, -100.0..=100.0)?;
    if amount == 0.0 {
        return Ok(());
    }
    let p = amount / 100.0;
    // ponytail: arc-length sphere warp on the axis extents, not Adobe's closed
    // falloff/resampling (behavioral parity only). Upgrade once CS6 renders exist.
    warp(buf, move |px, py, rx, ry| match mode {
        SpherizeMode::Normal => {
            let rmax = rx.max(ry);
            let r = (px * px + py * py).sqrt();
            if r == 0.0 || rmax <= 0.0 {
                return (px, py);
            }
            let f = rmax * sphere_map((r / rmax).clamp(0.0, 1.0), p) / r;
            (px * f, py * f)
        }
        SpherizeMode::HorizontalOnly => {
            if rx <= 0.0 {
                return (px, py);
            }
            let mapped = rx * sphere_map((px.abs() / rx).clamp(0.0, 1.0), p);
            (px.signum() * mapped, py)
        }
        SpherizeMode::VerticalOnly => {
            if ry <= 0.0 {
                return (px, py);
            }
            let mapped = ry * sphere_map((py.abs() / ry).clamp(0.0, 1.0), p);
            (px, py.signum() * mapped)
        }
    })
}

/// Monotone radial map on `x in [0, 1]`; `x = 1` is a fixed edge point. `p > 0`
/// magnifies the center (bulge), `p < 0` compresses it (pincushion).
fn sphere_map(x: f64, p: f64) -> f64 {
    let a = p.abs();
    let wrapped = if p >= 0.0 {
        (2.0 / std::f64::consts::PI) * x.asin()
    } else {
        (x * std::f64::consts::FRAC_PI_2).sin()
    };
    x + a * (wrapped - x)
}

/// Shared inverse-mapping loop: `map` turns an output offset into a source
/// offset, then bilinear + clamp-to-edge resampling.
fn warp(
    buf: &mut PixelBuffer,
    map: impl Fn(f64, f64, f64, f64) -> (f64, f64),
) -> Result<(), FilterError> {
    let w = buf.width as usize;
    let h = buf.height as usize;
    let n = w * h;
    let planes = (buf.channels as usize).min(3);
    let cx = (w as f64 - 1.0) / 2.0;
    let cy = (h as f64 - 1.0) / 2.0;
    for c in 0..planes {
        let base = c * n;
        let src = buf.data[base..base + n].to_vec();
        for y in 0..h {
            for x in 0..w {
                let (px, py) = (x as f64 - cx, y as f64 - cy);
                let (ox, oy) = map(px, py, cx, cy);
                let v = bilinear(&src, w, h, cx + ox, cy + oy);
                buf.data[base + y * w + x] = v.round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    Ok(())
}

fn bilinear(plane: &[u8], w: usize, h: usize, x: f64, y: f64) -> f64 {
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);
    let (xi, yi) = (clamp_index(x0 as isize, w), clamp_index(y0 as isize, h));
    let (xi1, yi1) = (
        clamp_index(x0 as isize + 1, w),
        clamp_index(y0 as isize + 1, h),
    );
    let at = |x: usize, y: usize| plane[y * w + x] as f64;
    let (p00, p10, p01, p11) = (at(xi, yi), at(xi1, yi), at(xi, yi1), at(xi1, yi1));
    (p00 * (1.0 - fx) + p10 * fx) * (1.0 - fy) + (p01 * (1.0 - fx) + p11 * fx) * fy
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buf<const C: usize>(w: u32, h: u32, px: &[[u8; C]]) -> PixelBuffer {
        let n = (w * h) as usize;
        assert_eq!(px.len(), n);
        let mut data = vec![0u8; n * C];
        for (i, p) in px.iter().enumerate() {
            for (c, &v) in p.iter().enumerate() {
                data[c * n + i] = v;
            }
        }
        PixelBuffer {
            width: w,
            height: h,
            channels: C as u8,
            data,
        }
    }

    use SpherizeMode::{HorizontalOnly, Normal, VerticalOnly};
    const MODES: [SpherizeMode; 3] = [Normal, HorizontalOnly, VerticalOnly];

    #[test]
    fn warps_change_patterns_and_zero_is_bit_exact_noop() {
        let px: Vec<[u8; 3]> = (0..81)
            .map(|i| {
                let (x, y) = ((i % 9) as u8, (i / 9) as u8);
                [
                    x.wrapping_mul(20),
                    y.wrapping_mul(30),
                    x.wrapping_add(y).wrapping_mul(10),
                ]
            })
            .collect();
        let base = buf(9, 9, &px);
        let solid = buf(9, 9, &vec![[40u8, 90, 160]; 81]);
        let check_solid = |b: &PixelBuffer| {
            for (g, w) in b.data.iter().zip(&solid.data) {
                assert!((*g as i32 - *w as i32).abs() <= 1, "solid drifted");
            }
        };

        let mut t0 = base.clone();
        twirl(&mut t0, 0.0).unwrap();
        assert_eq!(t0.data, base.data);
        let mut p0 = base.clone();
        pinch(&mut p0, 0.0).unwrap();
        assert_eq!(p0.data, base.data);
        for m in MODES {
            let mut s0 = base.clone();
            spherize(&mut s0, 0.0, m).unwrap();
            assert_eq!(s0.data, base.data, "{m:?} no-op at 0");
        }

        let mut p = base.clone();
        pinch(&mut p, 60.0).unwrap();
        assert_ne!(p.data, base.data, "pinch changes pattern");
        let mut n = base.clone();
        spherize(&mut n, 60.0, SpherizeMode::Normal).unwrap();
        assert_ne!(n.data, base.data, "spherize changes pattern");
        let mut h = base.clone();
        spherize(&mut h, 70.0, SpherizeMode::HorizontalOnly).unwrap();
        assert_ne!(n.data, h.data, "HorizontalOnly differs from Normal");

        let mut ps = solid.clone();
        pinch(&mut ps, -80.0).unwrap();
        check_solid(&ps);
        for m in MODES {
            let mut ss = solid.clone();
            spherize(&mut ss, 80.0, m).unwrap();
            check_solid(&ss);
        }
    }

    #[test]
    fn twirl_rotates_pattern_and_pins_center() {
        let mut px = vec![[0u8; 3]; 81];
        px[2 * 9 + 2] = [255; 3];
        px[4 * 9 + 4] = [200; 3];
        let base = buf(9, 9, &px);
        let mut t = base.clone();
        twirl(&mut t, 90.0).unwrap();
        assert_ne!(t.data, base.data, "off-center pixel moves");
        let (n, i) = (t.pixel_count(), 4 * 9 + 4);
        assert_eq!([t.data[i], t.data[n + i], t.data[2 * n + i]], [200; 3]);
    }

    #[test]
    fn params_are_validated() {
        let mut b = PixelBuffer::new(4, 4, 3);
        assert!(twirl(&mut b, f64::NAN).is_err());
        assert!(twirl(&mut b, 1000.0).is_err());
        assert!(pinch(&mut b, f64::INFINITY).is_err());
        assert!(pinch(&mut b, 101.0).is_err());
        assert!(spherize(&mut b, f64::NAN, SpherizeMode::Normal).is_err());
        assert!(spherize(&mut b, 100.1, SpherizeMode::Normal).is_err());
        assert!(twirl(&mut b, 999.0).is_ok());
        assert!(pinch(&mut b, -100.0).is_ok());
        assert!(spherize(&mut b, 100.0, SpherizeMode::VerticalOnly).is_ok());
    }

    #[test]
    fn alpha_is_untouched_by_every_radial_warp() {
        let alpha: Vec<u8> = (0..49).map(|i| (i * 5) as u8).collect();
        let px: Vec<[u8; 4]> = alpha
            .iter()
            .enumerate()
            .map(|(i, &a)| [(i * 3) as u8, (i * 4) as u8, (i * 5) as u8, a])
            .collect();
        let base = buf(7, 7, &px);
        let check = |b: &PixelBuffer| {
            let n = b.pixel_count();
            for (i, &a) in alpha.iter().enumerate() {
                assert_eq!(b.data[3 * n + i], a, "alpha changed at {i}");
            }
        };
        let mut t = base.clone();
        twirl(&mut t, 45.0).unwrap();
        check(&t);
        let mut p = base.clone();
        pinch(&mut p, -40.0).unwrap();
        check(&p);
        for m in MODES {
            let mut s = base.clone();
            spherize(&mut s, 40.0, m).unwrap();
            check(&s);
        }
    }

    #[test]
    fn tiny_and_thin_images_do_not_panic() {
        for (w, h) in [(1u32, 1u32), (1, 7), (7, 1)] {
            let n = (w * h) as usize;
            let px: Vec<[u8; 3]> = (0..n)
                .map(|i| [(i * 3) as u8, (i * 5) as u8, (i * 7) as u8])
                .collect();
            let mut b = buf(w, h, &px);
            twirl(&mut b, 33.0).unwrap();
            pinch(&mut b, 80.0).unwrap();
            for m in MODES {
                spherize(&mut b, 80.0, m).unwrap();
            }
        }
    }
}
