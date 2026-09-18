//! Noise family: Add Noise, Median, Despeckle (`FILT-030`).
//!
//! Add Noise is the only randomized filter in the crate; the seed is part of
//! the parameters so a re-apply is bit-identical. Median and Despeckle are
//! deterministic. Every filter touches the color planes only; alpha is never
//! modified. Adobe's kernels are closed, so the choices marked `ponytail:`
//! below are documented approximations, not verified parity.

use pictura_core::PixelBuffer;
use rand_chacha::{rand_core::SeedableRng, ChaCha8Rng};

use crate::{
    kernel::{clamp_index, unit_f64},
    validate, FilterError, NoiseDistribution,
};

/// 8-bit full scale; Add Noise Amount is a percentage of it (`FILT-030`).
const FULL_SCALE: f64 = 255.0;
/// Despeckle: how far outside a flat neighborhood the center must sit before
/// it reads as an isolated outlier rather than texture.
const DESPECKLE_SPIKE: i32 = 12;
/// Despeckle: neighborhood spread at or below this counts as "flat" (no edge).
const DESPECKLE_BAND: i32 = 12;

/// Add seeded zero-mean noise to the color planes.
///
/// `amount` is a percentage of full scale (0 no-op, 0..=400 accepted).
/// Uniform uses a half-width of `amount`; Gaussian uses `amount` as its
/// standard deviation, so Gaussian reads stronger at the same Amount
/// (matches the "subtle / speckled" CS6 description). `monochromatic` draws
/// one delta per pixel and applies it to every channel (hue-preserving);
/// otherwise each channel gets its own delta.
pub fn add(
    buf: &mut PixelBuffer,
    amount: f64,
    distribution: NoiseDistribution,
    monochromatic: bool,
    seed: u64,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if amount == 0.0 {
        return Ok(());
    }
    if !(0.0..=400.0).contains(&amount) {
        return Err(FilterError::InvalidParams(format!(
            "add noise amount {amount} out of range 0..=400"
        )));
    }
    let scale = FULL_SCALE * amount / 100.0;
    let planes = (buf.channels as usize).min(3);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    for p in 0..n {
        if monochromatic {
            let delta = sample_delta(&mut rng, distribution, scale);
            for plane in 0..planes {
                let idx = plane * n + p;
                buf.data[idx] = apply_delta(buf.data[idx], delta);
            }
        } else {
            for plane in 0..planes {
                let idx = plane * n + p;
                let delta = sample_delta(&mut rng, distribution, scale);
                buf.data[idx] = apply_delta(buf.data[idx], delta);
            }
        }
    }
    Ok(())
}

/// Replace each pixel with the per-channel median of its `(2r+1)²` window.
///
/// `radius == 0` is a no-op. Clamp-to-edge at borders.
pub fn median(buf: &mut PixelBuffer, radius: u32) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if radius == 0 {
        return Ok(());
    }
    if radius > 100 {
        return Err(FilterError::InvalidParams(format!(
            "median radius {radius} is outside 0..=100"
        )));
    }
    let w = buf.width as usize;
    let h = buf.height as usize;
    // Cap the radius so an oversized request cannot allocate a giant window;
    // beyond max(w, h) every clamped sample is interior anyway.
    let r = (radius as usize).min(w.max(h));
    let k = 2 * r + 1;
    let planes = (buf.channels as usize).min(3);
    // ponytail: sort-as-we-go is O(pixels · k² log k); swap in the sliding
    // histogram from Huang et al. 1979 (cited by FILT-030) if Median gets hot.
    let mut window: Vec<u8> = Vec::with_capacity(k * k);
    for plane in 0..planes {
        let base = plane * n;
        let src = buf.data[base..base + n].to_vec();
        for y in 0..h {
            for x in 0..w {
                window.clear();
                for dy in 0..k {
                    let sy = clamp_index(y as isize + dy as isize - r as isize, h);
                    for dx in 0..k {
                        let sx = clamp_index(x as isize + dx as isize - r as isize, w);
                        window.push(src[sy * w + sx]);
                    }
                }
                window.sort_unstable();
                buf.data[base + y * w + x] = window[window.len() / 2];
            }
        }
    }
    Ok(())
}

/// Edge-gated smoothing: replace isolated outliers, leave edges alone.
///
/// A pixel is an outlier when its 8 neighbors are flat (spread <= band) and
/// the center lies beyond them, in which case it takes the 3×3 median. A
/// genuine edge makes the neighborhood spread wide, so its pixels are kept.
pub fn despeckle(buf: &mut PixelBuffer) -> Result<(), FilterError> {
    let n = validate(buf)?;
    let w = buf.width as usize;
    let h = buf.height as usize;
    if w < 3 || h < 3 {
        return Ok(());
    }
    let planes = (buf.channels as usize).min(3);
    // ponytail: fixed internal thresholds; FILT-030 leaves the detector closed.
    // A per-tile adaptive threshold or a real gradient gate is the upgrade path.
    for plane in 0..planes {
        let base = plane * n;
        let src = buf.data[base..base + n].to_vec();
        for y in 0..h {
            for x in 0..w {
                let mut vals = [0i32; 9];
                let mut nmin = 255i32;
                let mut nmax = 0i32;
                for dy in 0..3usize {
                    let sy = clamp_index(y as isize + dy as isize - 1, h);
                    for dx in 0..3usize {
                        let sx = clamp_index(x as isize + dx as isize - 1, w);
                        let v = src[sy * w + sx] as i32;
                        vals[dy * 3 + dx] = v;
                        if dy != 1 || dx != 1 {
                            nmin = nmin.min(v);
                            nmax = nmax.max(v);
                        }
                    }
                }
                let center = vals[4];
                if nmax - nmin <= DESPECKLE_BAND
                    && (center > nmax + DESPECKLE_SPIKE || center < nmin - DESPECKLE_SPIKE)
                {
                    vals.sort_unstable();
                    buf.data[base + y * w + x] = vals[4] as u8;
                }
            }
        }
    }
    Ok(())
}

/// One standard normal sample via Box–Muller (`u1` in `(0, 1]` avoids `ln(0)`).
fn standard_normal(rng: &mut ChaCha8Rng) -> f64 {
    let u1 = 1.0 - unit_f64(rng);
    let u2 = unit_f64(rng);
    (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
}

fn sample_delta(rng: &mut ChaCha8Rng, distribution: NoiseDistribution, scale: f64) -> f64 {
    match distribution {
        NoiseDistribution::Uniform => (unit_f64(rng) * 2.0 - 1.0) * scale,
        NoiseDistribution::Gaussian => standard_normal(rng) * scale,
    }
}

fn apply_delta(v: u8, delta: f64) -> u8 {
    (v as f64 + delta).round().clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plane(w: u32, h: u32, channels: u8, fill: [u8; 3]) -> PixelBuffer {
        let n = w as usize * h as usize;
        let mut data = vec![0u8; n * channels as usize];
        for p in 0..n {
            for (c, &v) in fill.iter().enumerate() {
                data[c * n + p] = v;
            }
            if channels == 4 {
                data[3 * n + p] = 200;
            }
        }
        PixelBuffer {
            width: w,
            height: h,
            channels,
            data,
        }
    }

    fn stats(buf: &PixelBuffer, plane: usize) -> (f64, f64) {
        let n = buf.pixel_count();
        let vals = &buf.data[plane * n..plane * n + n];
        let mean = vals.iter().map(|&v| v as f64).sum::<f64>() / n as f64;
        let var = vals.iter().map(|&v| (v as f64 - mean).powi(2)).sum::<f64>() / n as f64;
        (mean, var.sqrt())
    }

    #[test]
    fn add_noise_keeps_mean_and_grows_std() {
        for distribution in [NoiseDistribution::Uniform, NoiseDistribution::Gaussian] {
            let mut low = plane(64, 64, 3, [128, 128, 128]);
            add(&mut low, 2.0, distribution, false, 1).unwrap();
            let (mean_low, std_low) = stats(&low, 0);
            assert!((mean_low - 128.0).abs() < 2.0, "mean drifted: {mean_low}");
            assert!(std_low > 1.0, "std too small: {std_low}");

            let mut high = plane(64, 64, 3, [128, 128, 128]);
            add(&mut high, 20.0, distribution, false, 1).unwrap();
            let (mean_high, std_high) = stats(&high, 0);
            assert!((mean_high - 128.0).abs() < 4.0, "mean drifted: {mean_high}");
            assert!(
                std_high > std_low * 3.0,
                "{distribution:?}: std should grow with amount ({std_low} -> {std_high})"
            );
        }
    }

    #[test]
    fn add_noise_monochromatic_preserves_hue() {
        let mut mono = plane(32, 32, 3, [100, 150, 200]);
        add(&mut mono, 5.0, NoiseDistribution::Uniform, true, 7).unwrap();
        let n = mono.pixel_count();
        for p in 0..n {
            let r = mono.data[p] as i32;
            let g = mono.data[n + p] as i32;
            let b = mono.data[2 * n + p] as i32;
            assert_eq!(r - g, -50);
            assert_eq!(g - b, -50);
        }

        let mut color = plane(32, 32, 3, [100, 150, 200]);
        add(&mut color, 5.0, NoiseDistribution::Uniform, false, 7).unwrap();
        let varied = (0..n).any(|p| {
            let r = color.data[p] as i32;
            let g = color.data[n + p] as i32;
            let b = color.data[2 * n + p] as i32;
            r - g != -50 || g - b != -50
        });
        assert!(varied, "per-channel noise should shift channel differences");
    }

    #[test]
    fn add_noise_same_seed_is_identical_and_different_seed_differs() {
        let base = plane(16, 16, 4, [128, 128, 128]);
        let mut a = base.clone();
        let mut b = base.clone();
        let mut c = base.clone();
        add(&mut a, 10.0, NoiseDistribution::Gaussian, false, 42).unwrap();
        add(&mut b, 10.0, NoiseDistribution::Gaussian, false, 42).unwrap();
        add(&mut c, 10.0, NoiseDistribution::Gaussian, false, 43).unwrap();
        assert_eq!(a.data, b.data);
        assert_ne!(a.data, c.data);
    }

    #[test]
    fn zero_amount_and_zero_radius_are_noops_bad_amount_errors() {
        let base = plane(4, 4, 3, [128, 128, 128]);
        let mut buf = base.clone();
        add(&mut buf, 0.0, NoiseDistribution::Uniform, false, 1).unwrap();
        assert_eq!(buf.data, base.data);
        median(&mut buf, 0).unwrap();
        assert_eq!(buf.data, base.data);
        assert!(add(&mut buf, -1.0, NoiseDistribution::Uniform, false, 1).is_err());
    }

    #[test]
    fn median_removes_salt_and_pepper_and_keeps_step_edge() {
        let mut salt = plane(8, 8, 3, [100, 100, 100]);
        let w = 8usize;
        let n = salt.pixel_count();
        salt.data[3 * w + 4] = 255;
        median(&mut salt, 1).unwrap();
        assert!(
            salt.data.iter().all(|&v| v == 100),
            "isolated pixel should be removed"
        );

        let mut step = plane(8, 8, 3, [50, 50, 50]);
        for p in 0..n {
            if p % w >= 4 {
                step.data[p] = 200;
                step.data[n + p] = 200;
                step.data[2 * n + p] = 200;
            }
        }
        median(&mut step, 1).unwrap();
        for y in 0..8usize {
            assert_eq!(step.data[y * w], 50);
            assert_eq!(step.data[y * w + w - 1], 200);
        }
        assert!(
            step.data.iter().all(|&v| v == 50 || v == 200),
            "step edge should stay sharp"
        );
    }

    #[test]
    fn despeckle_removes_isolated_pixel_and_keeps_strong_edge() {
        let mut b = plane(8, 8, 3, [100, 100, 100]);
        let w = 8usize;
        let n = b.pixel_count();
        b.data[3 * w + 4] = 255;
        despeckle(&mut b).unwrap();
        assert_eq!(b.data[3 * w + 4], 100);

        let mut e = plane(8, 8, 3, [0, 0, 0]);
        for p in 0..n {
            if p % w >= 4 {
                e.data[p] = 255;
                e.data[n + p] = 255;
                e.data[2 * n + p] = 255;
            }
        }
        let before = e.data.clone();
        despeckle(&mut e).unwrap();
        assert_eq!(e.data, before, "strong step edge must be preserved");
    }

    #[test]
    fn alpha_is_untouched_and_tiny_images_do_not_panic() {
        for &(w, h) in &[(1u32, 1u32), (2, 2), (3, 3), (8, 8)] {
            let base = plane(w, h, 4, [100, 100, 100]);
            let n = base.pixel_count();
            let alpha = base.data[3 * n..].to_vec();

            let mut a = base.clone();
            add(&mut a, 10.0, NoiseDistribution::Gaussian, true, 3).unwrap();
            assert_eq!(&a.data[3 * n..], &alpha[..]);

            let mut m = base.clone();
            median(&mut m, 1).unwrap();
            assert_eq!(&m.data[3 * n..], &alpha[..]);

            let mut d = base.clone();
            despeckle(&mut d).unwrap();
            assert_eq!(&d.data[3 * n..], &alpha[..]);
        }
    }
}
