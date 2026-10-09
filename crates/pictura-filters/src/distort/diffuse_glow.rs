//! Diffuse Glow (`FILT-040`): the picture as seen through a soft diffusion
//! filter. Highlights bloom toward the glow colour and spill past their
//! edges, a veil of it lifts the whole picture as Clear Amount falls, and
//! grain speckles the glow so the picture shows through it.
//!
//! CS6's algorithm is closed. The model below is fitted by least squares to
//! CS6 Filter Gallery renders at (Graininess, Glow, Clear) = (6, 10, 15),
//! (2, 2, 2), (9, 18, 16) on one photograph and (8, 1, 6) on another: every
//! pixel mixes toward the glow colour by a fraction driven by its blurred
//! luminance. It is an approximation, not pixel parity.

use pictura_core::PixelBuffer;
use rayon::prelude::*;

use crate::kernel::{blur_plane, to_u8};
use crate::luma::luma_plane;
use crate::{validate, FilterError};

// The halo: σ of the luminance blur, in pixels.
const HALO_SIGMA: f32 = 2.37;
const HALO_SIGMA_PER_GLOW: f32 = 0.54;
// Where the glow ramp is half on, as a 0..1 luminance.
const THRESHOLD: f32 = 0.56;
const THRESHOLD_PER_CLEAR: f32 = 0.016;
const THRESHOLD_PER_GLOW: f32 = -0.028;
// How steeply the ramp climbs.
const GAIN: f32 = 0.71;
const GAIN_PER_GLOW: f32 = 0.186;
// The most the glow covers; the picture always shows a little. It eases in
// over the first couple of Glow steps, so Glow 0 is no glow.
const COVER: f32 = 0.93;
const COVER_GLOW_SCALE: f32 = 0.71;
// The veil: the whole picture lifts by VEIL at Clear 0, falling steeply to
// none at VEIL_CLEAR_END. CS6 washes out at Clear 2 but barely lifts at 6.
const VEIL: f32 = 0.98;
const VEIL_CLEAR_END: f32 = 10.0;
const VEIL_FALLOFF: f32 = 2.5;

/// Bloom the highlights of `buf` toward white.
///
/// `graininess` in `0..=10` speckles the glow, `glow_amount` in `0..=20`
/// lowers the threshold, steepens the ramp and widens the halo (0 is no
/// glow), and `clear_amount` in `0..=20` thins the veil (none from 10) and
/// raises the threshold. Same seed and parameters are bit-identical; alpha is untouched.
pub fn diffuse_glow(
    buf: &mut PixelBuffer,
    graininess: u32,
    glow_amount: u32,
    clear_amount: u32,
    seed: u64,
) -> Result<(), FilterError> {
    validate(buf)?;
    for (name, value, max) in [
        ("graininess", graininess, 10),
        ("glow amount", glow_amount, 20),
        ("clear amount", clear_amount, 20),
    ] {
        if value > max {
            return Err(FilterError::InvalidParams(format!(
                "diffuse glow {name} {value} outside 0..={max}"
            )));
        }
    }
    let (w, h) = (buf.width as usize, buf.height as usize);
    let n = w * h;
    if n == 0 {
        return Ok(());
    }
    // ponytail: white stands in for CS6's glow colour, the Background swatch;
    // the dialog shows no swatch and the variant carries no colour yet.
    const GLOW: f32 = 255.0;

    let (grain, glow, clear) = (graininess as f32, glow_amount as f32, clear_amount as f32);
    let luma: Vec<f32> = luma_plane(&buf.data, n)
        .into_iter()
        .map(|l| l as f32 / 255.0)
        .collect();
    let lit = blur_plane(&luma, w, h, HALO_SIGMA + HALO_SIGMA_PER_GLOW * glow);
    let threshold = THRESHOLD + THRESHOLD_PER_CLEAR * clear + THRESHOLD_PER_GLOW * glow;
    let gain = GAIN + GAIN_PER_GLOW * glow;
    let cover = COVER * (1.0 - (-glow / COVER_GLOW_SCALE).exp());
    let veil = VEIL * (1.0 - clear / VEIL_CLEAR_END).max(0.0).powf(VEIL_FALLOFF);
    let speckle = 0.0126 * grain + 0.0016 * grain * grain;
    let salt = (seed ^ (seed >> 32)) as u32;

    let mut mix = vec![0f32; n];
    mix.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
        for (x, m) in row.iter_mut().enumerate() {
            let t = (0.5 + gain * (lit[y * w + x] - threshold)).clamp(0.0, 1.0);
            let ramp = t * t * (3.0 - 2.0 * t);
            let mut v = 1.0 - (1.0 - veil) * (1.0 - cover * ramp);
            if speckle > 0.0 {
                // Two draws summed bunch the grain toward the middle.
                v += speckle
                    * (unit(x as u32, y as u32, salt) + unit(x as u32, y as u32, !salt) - 1.0);
            }
            *m = v.clamp(0.0, 1.0);
        }
    });
    for c in 0..(buf.channels as usize).min(3) {
        let plane = &mut buf.data[c * n..c * n + n];
        plane.par_iter_mut().zip(&mix).for_each(|(p, &m)| {
            let v = *p as f32;
            *p = to_u8(f64::from(v + (GLOW - v) * m));
        });
    }
    Ok(())
}

/// A uniform `[0, 1)` draw keyed on pixel position, so the grain does not
/// depend on the order rows are visited in.
fn unit(x: u32, y: u32, salt: u32) -> f32 {
    let mut h = x.wrapping_mul(0x9E37_79B1) ^ y.wrapping_mul(0x85EB_CA77) ^ salt;
    h ^= h >> 16;
    h = h.wrapping_mul(0x7FEB_352D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846C_A68B);
    h ^= h >> 16;
    (h >> 8) as f32 / (1u32 << 24) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(w: u32, h: u32, rgb: [u8; 3], alpha: u8) -> PixelBuffer {
        let n = (w * h) as usize;
        let mut data = Vec::with_capacity(n * 4);
        for v in rgb {
            data.extend(std::iter::repeat_n(v, n));
        }
        data.extend(std::iter::repeat_n(alpha, n));
        PixelBuffer {
            width: w,
            height: h,
            channels: 4,
            data: data.into(),
        }
    }

    /// Dark on the left half, light on the right.
    fn step(w: u32, h: u32) -> PixelBuffer {
        let mut buf = flat(w, h, [40, 40, 40], 255);
        let n = (w * h) as usize;
        for i in 0..n {
            if (i as u32 % w) >= w / 2 {
                for c in 0..3 {
                    buf.data[c * n + i] = 220;
                }
            }
        }
        buf
    }

    fn at(buf: &PixelBuffer, x: u32, y: u32) -> u8 {
        buf.data[(y * buf.width + x) as usize]
    }

    #[test]
    fn no_glow_full_clear_and_no_grain_is_identity() {
        let mut buf = step(32, 8);
        let before = buf.data.clone();
        diffuse_glow(&mut buf, 0, 0, 20, 1).unwrap();
        assert_eq!(buf.data, before);
    }

    #[test]
    fn highlights_bloom_and_spill_past_their_edge() {
        let mut buf = step(64, 8);
        diffuse_glow(&mut buf, 0, 10, 15, 1).unwrap();
        let (far_dark, near_dark, light) = (at(&buf, 2, 4), at(&buf, 30, 4), at(&buf, 50, 4));
        assert!(light > 245, "the highlight blooms toward white: {light}");
        assert!(far_dark < 60, "deep shadow stays dark: {far_dark}");
        assert!(
            near_dark > far_dark + 20,
            "the halo spills: {near_dark} vs {far_dark}"
        );
    }

    #[test]
    fn clear_amount_thins_the_veil() {
        let mut veiled = flat(16, 16, [40, 60, 30], 255);
        let mut clear = veiled.clone();
        diffuse_glow(&mut veiled, 0, 2, 2, 1).unwrap();
        diffuse_glow(&mut clear, 0, 2, 15, 1).unwrap();
        assert!(at(&veiled, 8, 8) > 150, "low Clear washes the shadows out");
        assert!(at(&clear, 8, 8) < 60, "high Clear keeps them");
        // CS6 barely lifts the shadows by Clear 6 (about a tenth of the way).
        let mut faint = flat(16, 16, [40, 60, 30], 255);
        diffuse_glow(&mut faint, 0, 1, 6, 1).unwrap();
        let lifted = at(&faint, 8, 8);
        assert!((55..80).contains(&lifted), "Clear 6 barely veils: {lifted}");
    }

    #[test]
    fn grain_is_seeded_never_darkens_and_leaves_alpha() {
        let src = flat(32, 32, [60, 90, 50], 77);
        let run = |seed| {
            let mut buf = src.clone();
            diffuse_glow(&mut buf, 10, 10, 15, seed).unwrap();
            buf
        };
        let (a, b, other) = (run(3), run(3), run(4));
        assert_eq!(a.data, b.data);
        assert_ne!(a.data, other.data);
        let n = 32 * 32;
        assert!(a.data[3 * n..].iter().all(|&v| v == 77), "alpha untouched");
        for c in 0..3 {
            assert!(a.data[c * n..(c + 1) * n]
                .iter()
                .all(|&v| v >= src.data[c * n]));
        }
        let red = &a.data[..n];
        let spread = red.iter().max().unwrap() - red.iter().min().unwrap();
        assert!(spread > 20, "graininess speckles a flat area: {spread}");
    }

    #[test]
    fn out_of_range_parameters_are_rejected_untouched() {
        for (grain, glow, clear) in [(11, 10, 15), (6, 21, 15), (6, 10, 21)] {
            let mut buf = step(8, 8);
            let before = buf.data.clone();
            assert!(matches!(
                diffuse_glow(&mut buf, grain, glow, clear, 1),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(buf.data, before);
        }
    }
}
