//! Ripple warps (`FILT-040`): ZigZag, Ocean Ripple.
//!
//! Inverse-mapping warps with bilinear color sampling; alpha is never touched.
//! ZigZag is deterministic; Ocean Ripple takes a seed. Adobe's generators are
//! closed (`docs/dev/m11-distort2.md`), so the models below are approximations.

use pictura_core::PixelBuffer;
use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaCha8Rng,
};

use crate::kernel::clamp_index;
use crate::{validate, FilterError, ZigZagStyle};

/// Radial displacement about the image center.
///
/// `amount` is a percent in `-100..=100` (sign flips direction, `0` is a
/// bit-exact no-op); `ridges` in `0..=20` sets the direction reversals from the
/// center to the edge (`0` is a single direction). `AroundCenter` displaces
/// tangentially, `OutFromCenter` radially, `PondRipples` diagonally.
pub fn zigzag(
    buf: &mut PixelBuffer,
    amount: f64,
    ridges: u32,
    style: ZigZagStyle,
) -> Result<(), FilterError> {
    validate(buf)?;
    if !amount.is_finite() || !(-100.0..=100.0).contains(&amount) {
        return Err(FilterError::InvalidParams(format!(
            "zigzag amount {amount} must be finite and within -100..=100"
        )));
    }
    if ridges > 20 {
        return Err(FilterError::InvalidParams(format!(
            "zigzag ridges {ridges} outside 0..=20"
        )));
    }
    if amount == 0.0 {
        return Ok(());
    }

    // ponytail: Adobe's ridge falloff is closed. This uses a cosine radial
    // profile pinned to zero at the edge, so reversals == ridges. Ceiling: no
    // CS6 pixel parity. Upgrade by fitting reference renders (oracle hook M11-B).
    let (w, h) = (buf.width as usize, buf.height as usize);
    let n = w * h;
    let planes = (buf.channels as usize).min(3);
    let cx = (w as f64 - 1.0) / 2.0;
    let cy = (h as f64 - 1.0) / 2.0;
    let rmax = cx.max(cy).max(1.0);
    let amp = amount / 100.0;
    let pi = std::f64::consts::PI;
    let src = buf.data.clone();
    for y in 0..h {
        for x in 0..w {
            let vx = x as f64 - cx;
            let vy = y as f64 - cy;
            let r = vx.hypot(vy);
            let rn = (r / rmax).clamp(0.0, 1.0);
            let mag = amp * rmax * (1.0 - rn) * (ridges as f64 * pi * rn).cos();
            let (ux, uy) = if r > 0.0 {
                (vx / r, vy / r)
            } else {
                (0.0, 0.0)
            };
            let (dx, dy) = match style {
                ZigZagStyle::AroundCenter => (-uy * mag, ux * mag),
                ZigZagStyle::OutFromCenter => (ux * mag, uy * mag),
                ZigZagStyle::PondRipples => {
                    let s = std::f64::consts::FRAC_1_SQRT_2;
                    ((ux - uy) * s * mag, (ux + uy) * s * mag)
                }
            };
            let (sx, sy) = (x as f64 + dx, y as f64 + dy);
            for c in 0..planes {
                let plane = &src[c * n..c * n + n];
                buf.data[c * n + y * w + x] = to_u8(sample(plane, w, h, sx, sy));
            }
        }
    }
    Ok(())
}

/// Seeded random ripple displacement.
///
/// `size` in `1..=15` scales the ripple wavelength (larger = broader ripples);
/// `magnitude` in `0..=20` is the amplitude (pixels); `0` is a no-op. Same seed
/// and parameters are bit-identical.
pub fn ocean_ripple(
    buf: &mut PixelBuffer,
    size: u32,
    magnitude: u32,
    seed: u64,
) -> Result<(), FilterError> {
    validate(buf)?;
    if !(1..=15).contains(&size) {
        return Err(FilterError::InvalidParams(format!(
            "ocean ripple size {size} outside 1..=15"
        )));
    }
    if magnitude > 20 {
        return Err(FilterError::InvalidParams(format!(
            "ocean ripple magnitude {magnitude} outside 0..=20"
        )));
    }
    if magnitude == 0 {
        return Ok(());
    }

    // ponytail: Adobe's ripple placement is closed. This sums a few random
    // direction sinusoids, an approximation of "randomly spaced ripples".
    // Ceiling: no CS6 pixel parity. Upgrade by fitting reference renders (M11-B).
    let (w, h) = (buf.width as usize, buf.height as usize);
    let n = w * h;
    let planes = (buf.channels as usize).min(3);
    let src = buf.data.clone();
    let tau = std::f64::consts::TAU;
    let base = tau / (size as f64 * 4.0);
    let components = 8;
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut comps = Vec::with_capacity(components);
    for _ in 0..components {
        let ang = tau * unit_f64(&mut rng);
        let k = base * (0.6 + 0.8 * unit_f64(&mut rng));
        let phase = tau * unit_f64(&mut rng);
        comps.push((k * ang.cos(), k * ang.sin(), phase));
    }
    let amp = magnitude as f64 * 0.4 / components as f64;
    for y in 0..h {
        for x in 0..w {
            let mut hx = 0.0;
            let mut hy = 0.0;
            for &(kx, ky, phase) in &comps {
                let t = kx * x as f64 + ky * y as f64 + phase;
                hx += t.sin();
                hy += t.cos();
            }
            let (sx, sy) = (x as f64 + amp * hx, y as f64 + amp * hy);
            for c in 0..planes {
                let plane = &src[c * n..c * n + n];
                buf.data[c * n + y * w + x] = to_u8(sample(plane, w, h, sx, sy));
            }
        }
    }
    Ok(())
}

/// Bilinear sample at `(x, y)`, clamp-to-edge.
fn sample(plane: &[u8], w: usize, h: usize, x: f64, y: f64) -> f64 {
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);
    let (xi, yi) = (clamp_index(x0 as isize, w), clamp_index(y0 as isize, h));
    let (xi1, yi1) = (
        clamp_index(x0 as isize + 1, w),
        clamp_index(y0 as isize + 1, h),
    );
    let p00 = plane[yi * w + xi] as f64;
    let p10 = plane[yi * w + xi1] as f64;
    let p01 = plane[yi1 * w + xi] as f64;
    let p11 = plane[yi1 * w + xi1] as f64;
    (p00 * (1.0 - fx) + p10 * fx) * (1.0 - fy) + (p01 * (1.0 - fx) + p11 * fx) * fy
}

/// One uniform `f64` in `[0, 1)` from 53 random bits (as in `noise.rs`).
fn unit_f64(rng: &mut ChaCha8Rng) -> f64 {
    (rng.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
}

fn to_u8(v: f64) -> u8 {
    v.round().clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buf3(w: u32, h: u32, px: &[[u8; 3]]) -> PixelBuffer {
        let mut b = PixelBuffer::new(w, h, 3);
        let n = px.len();
        for (i, p) in px.iter().enumerate() {
            for (c, &v) in p.iter().enumerate() {
                b.data[c * n + i] = v;
            }
        }
        b
    }

    fn buf4(w: u32, h: u32, px: &[[u8; 4]]) -> PixelBuffer {
        let mut b = PixelBuffer::new(w, h, 4);
        let n = px.len();
        for (i, p) in px.iter().enumerate() {
            for (c, &v) in p.iter().enumerate() {
                b.data[c * n + i] = v;
            }
        }
        b
    }

    /// Deterministic high-frequency pattern so a warp cannot be invariant.
    fn ramp(w: u32, h: u32) -> PixelBuffer {
        let px: Vec<[u8; 3]> = (0..(w * h) as usize)
            .map(|i| [i as u8, (i * 5) as u8, (i * 11) as u8])
            .collect();
        buf3(w, h, &px)
    }

    #[test]
    fn zigzag_zero_is_noop_and_nonzero_displaces() {
        let base = ramp(24, 24);
        let mut noop = base.clone();
        zigzag(&mut noop, 0.0, 5, ZigZagStyle::AroundCenter).unwrap();
        assert_eq!(noop.data, base.data, "amount 0 must be bit-exact");

        for style in [
            ZigZagStyle::AroundCenter,
            ZigZagStyle::OutFromCenter,
            ZigZagStyle::PondRipples,
        ] {
            let mut out = base.clone();
            zigzag(&mut out, 80.0, 5, style).unwrap();
            assert_ne!(out.data, base.data, "{style:?} must displace");
        }
    }

    #[test]
    fn zigzag_styles_differ() {
        let base = ramp(24, 24);
        let run = |style| {
            let mut b = base.clone();
            zigzag(&mut b, 80.0, 5, style).unwrap();
            b.data
        };
        let around = run(ZigZagStyle::AroundCenter);
        let out = run(ZigZagStyle::OutFromCenter);
        let pond = run(ZigZagStyle::PondRipples);
        assert_ne!(around, out);
        assert_ne!(around, pond);
        assert_ne!(out, pond);
    }

    #[test]
    fn zigzag_rejects_out_of_range_amount_and_ridges() {
        let base = ramp(8, 8);
        for bad in [-101.0, 101.0, f64::NAN, f64::INFINITY] {
            let mut b = base.clone();
            assert!(zigzag(&mut b, bad, 5, ZigZagStyle::AroundCenter).is_err());
        }
        let mut b = base.clone();
        assert!(zigzag(&mut b, 50.0, 21, ZigZagStyle::AroundCenter).is_err());
    }

    #[test]
    fn ocean_ripple_is_seed_deterministic() {
        let base = ramp(32, 32);
        let run = |seed| {
            let mut b = base.clone();
            ocean_ripple(&mut b, 9, 20, seed).unwrap();
            b.data
        };
        assert_eq!(run(7), run(7));
        assert_ne!(run(7), run(8));
        assert_ne!(run(7), base.data);
    }

    #[test]
    fn ocean_ripple_zero_magnitude_is_noop_and_validates() {
        let base = ramp(8, 8);
        let mut b = base.clone();
        ocean_ripple(&mut b, 9, 0, 1).unwrap();
        assert_eq!(b.data, base.data);

        assert!(ocean_ripple(&mut b, 0, 5, 1).is_err());
        assert!(ocean_ripple(&mut b, 16, 5, 1).is_err());
        assert!(ocean_ripple(&mut b, 9, 21, 1).is_err());
    }

    #[test]
    fn alpha_preserved_and_tiny_images_do_not_panic() {
        let px: Vec<[u8; 4]> = (0..24 * 24)
            .map(|i| [i as u8, (i * 3) as u8, (i * 7) as u8, (i * 11) as u8])
            .collect();
        let base = buf4(24, 24, &px);
        let n = base.pixel_count();
        let alpha = base.data[3 * n..].to_vec();

        let mut z = base.clone();
        zigzag(&mut z, 90.0, 7, ZigZagStyle::PondRipples).unwrap();
        assert_eq!(&z.data[3 * n..], &alpha[..]);
        let mut o = base.clone();
        ocean_ripple(&mut o, 9, 15, 3).unwrap();
        assert_eq!(&o.data[3 * n..], &alpha[..]);

        for &(w, h) in &[(1u32, 1u32), (1, 8), (8, 1)] {
            let px: Vec<[u8; 3]> = (0..(w * h) as usize)
                .map(|i| [i as u8, (i * 2) as u8, 255 - i as u8])
                .collect();
            let base = buf3(w, h, &px);
            let mut z = base.clone();
            assert!(zigzag(&mut z, 100.0, 20, ZigZagStyle::AroundCenter).is_ok());
            let mut o = base.clone();
            assert!(ocean_ripple(&mut o, 15, 20, 5).is_ok());
        }
    }
}
