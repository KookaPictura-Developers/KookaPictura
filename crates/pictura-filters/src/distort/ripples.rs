//! Ripple warps (`FILT-040`): ZigZag, Ocean Ripple.
//!
//! Inverse-mapping warps with bilinear color sampling; alpha is never touched.
//! ZigZag is deterministic; Ocean Ripple takes a seed. The reference's generators are
//! closed (`docs/dev/m11-distort2.md`), so the models below are approximations.

use pictura_core::PixelBuffer;
use rand_chacha::{rand_core::SeedableRng, ChaCha8Rng};

use crate::kernel::{clamp_index, to_u8, unit_f64};
use crate::{validate, FilterError};

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

    // ponytail: the reference's ripple placement is closed. This sums a few random
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

    /// Deterministic high-frequency pattern so a warp cannot be invariant.
    fn ramp(w: u32, h: u32) -> PixelBuffer {
        let px: Vec<[u8; 3]> = (0..(w * h) as usize)
            .map(|i| [i as u8, (i * 5) as u8, (i * 11) as u8])
            .collect();
        buf3(w, h, &px)
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
}
