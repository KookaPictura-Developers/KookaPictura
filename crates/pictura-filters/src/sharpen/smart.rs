//! Smart Sharpen: unsharp-mask against the blur the softness came from, with a
//! smooth noise floor, a higher-fidelity "More Accurate" blur estimate, and
//! Shadow/Highlight tonal damping.

use pictura_core::PixelBuffer;

use crate::kernel::{
    clamp_index, gaussian_blur_planes, gaussian_blur_planes_with_support, sigma_from_radius,
};
use crate::luma::luma_plane;
use crate::{validate, FilterError, SharpenRemove, TonalFade};

/// Smart Sharpen: unsharp-mask against the blur the softness came from, with a
/// smooth noise floor holding back the finest detail.
///
/// Ported from photorust's core/src/filters/convolve.rs (GPL-3.0-or-later; see
/// the change proposal).
///
/// `amount` and `reduce_noise` are percentages, `radius` in pixels (0.1..=64),
/// `angle` only read for [`SharpenRemove::MotionBlur`]. Color planes only.
///
/// ponytail: Adobe's "More Accurate" and Shadow/Highlight use closed
/// algorithms. `more_accurate` swaps in wider/denser blur estimates (4σ
/// Gaussian, half-pixel disc rows, bilinear motion taps); Shadow/Highlight
/// damps the sharpening delta by a soft tonal mask. Both are fidelity
/// approximations, not bit-exact ports.
#[allow(clippy::too_many_arguments)]
pub fn smart_sharpen(
    buf: &mut PixelBuffer,
    amount: f64,
    radius: f64,
    reduce_noise: f64,
    remove: SharpenRemove,
    angle: f64,
    more_accurate: bool,
    shadow: TonalFade,
    highlight: TonalFade,
) -> Result<(), FilterError> {
    validate(buf)?;
    if !amount.is_finite() || !(1.0..=500.0).contains(&amount) {
        return Err(FilterError::InvalidParams(format!(
            "smart sharpen amount {amount} is outside 1..=500"
        )));
    }
    if !radius.is_finite() || radius < 0.1 || radius > 64.0 {
        return Err(FilterError::InvalidParams(format!(
            "smart sharpen radius {radius} is outside 0.1..=64"
        )));
    }
    if !reduce_noise.is_finite() || !(0.0..=100.0).contains(&reduce_noise) {
        return Err(FilterError::InvalidParams(format!(
            "smart sharpen reduce noise {reduce_noise} is outside 0..=100"
        )));
    }
    if !angle.is_finite() || !(-360.0..=360.0).contains(&angle) {
        return Err(FilterError::InvalidParams(format!(
            "smart sharpen angle {angle} is outside -360..=360"
        )));
    }
    validate_fade(shadow, "shadow")?;
    validate_fade(highlight, "highlight")?;

    let n = buf.pixel_count();
    let planes = (buf.channels as usize).min(3);
    let orig = buf.data.clone();
    let mut blurred = buf.clone();
    // ponytail: the reference blurs premultiplied RGBA and skips fully
    // transparent pixels; here the color planes blur with clamp-to-edge and
    // alpha is never read or written, and radius maps through the shared
    // 3-sigma convention rather than photorust's own sigma scale.
    match remove {
        SharpenRemove::GaussianBlur => {
            let sigma = sigma_from_radius(radius);
            if more_accurate {
                gaussian_blur_planes_with_support(&mut blurred, sigma, 4.0);
            } else {
                gaussian_blur_planes(&mut blurred, sigma);
            }
        }
        SharpenRemove::LensBlur => {
            disc_blur_planes(&mut blurred, radius.round().max(1.0) as u32, more_accurate);
        }
        SharpenRemove::MotionBlur => {
            let distance = (radius * 2.0).round().max(2.0) as u32;
            if more_accurate {
                crate::blur::motion_accurate(&mut blurred, angle, distance)?;
            } else {
                crate::blur::motion(&mut blurred, angle, distance)?;
            }
        }
    }

    let fade = tonal_fade_mask(&orig[..], buf.width, buf.height, shadow, highlight);
    let strength = amount / 100.0;
    // Smooth roll-off: detail below the floor fades away instead of being
    // cut, so a flat area cannot break along a hard threshold.
    let floor = reduce_noise / 100.0 * 26.0;
    let floor_sq = floor * floor;
    for c in 0..planes {
        let base = c * n;
        for i in 0..n {
            let o = orig[base + i] as f64;
            let diff = o - blurred.data[base + i] as f64;
            let gain = if floor > 0.0 {
                let d = diff * diff;
                d / (d + floor_sq)
            } else {
                1.0
            };
            let out = match &fade {
                Some(f) => o + diff * strength * gain * (1.0 - f[i]),
                None => o + diff * strength * gain,
            };
            buf.data[base + i] = out.round().clamp(0.0, 255.0) as u8;
        }
    }
    Ok(())
}

fn validate_fade(fade: TonalFade, which: &str) -> Result<(), FilterError> {
    if fade.amount > 100 {
        return Err(FilterError::InvalidParams(format!(
            "smart sharpen {which} amount {} is outside 0..=100",
            fade.amount
        )));
    }
    if fade.width > 100 {
        return Err(FilterError::InvalidParams(format!(
            "smart sharpen {which} width {} is outside 0..=100",
            fade.width
        )));
    }
    if !(1..=100).contains(&fade.radius) {
        return Err(FilterError::InvalidParams(format!(
            "smart sharpen {which} radius {} is outside 1..=100",
            fade.radius
        )));
    }
    Ok(())
}

/// Per-pixel tonal fade amount in `0..=1`, or `None` when both tabs are
/// inactive. The sharpening delta is scaled by `1 - fade`.
fn tonal_fade_mask(
    data: &[u8],
    width: u32,
    height: u32,
    shadow: TonalFade,
    highlight: TonalFade,
) -> Option<Vec<f64>> {
    let shadow_active = shadow.amount > 0 && shadow.width > 0;
    let highlight_active = highlight.amount > 0 && highlight.width > 0;
    if !shadow_active && !highlight_active {
        return None;
    }
    let n = width as usize * height as usize;
    let shadow_l = shadow_active.then(|| blurred_luma(data, width, height, shadow.radius));
    let highlight_l = highlight_active.then(|| blurred_luma(data, width, height, highlight.radius));
    let mut fade = vec![0.0f64; n];
    for (i, f) in fade.iter_mut().enumerate() {
        let mut value = 0.0f64;
        if let Some(l) = &shadow_l {
            let mask = 1.0 - smoothstep(0.0, 0.5 * shadow.width as f64 / 100.0, l[i]);
            value = value.max(shadow.amount as f64 / 100.0 * mask);
        }
        if let Some(l) = &highlight_l {
            let mask = smoothstep(1.0 - 0.5 * highlight.width as f64 / 100.0, 1.0, l[i]);
            value = value.max(highlight.amount as f64 / 100.0 * mask);
        }
        *f = value;
    }
    Some(fade)
}

/// Luminance of the original in `0..=1`, Gaussian-blurred with the shared
/// radius convention.
fn blurred_luma(data: &[u8], width: u32, height: u32, radius: u32) -> Vec<f64> {
    let n = width as usize * height as usize;
    let luma = luma_plane(data, n);
    let bytes: Vec<u8> = luma
        .iter()
        .map(|&v| v.round().clamp(0.0, 255.0) as u8)
        .collect();
    let mut mask = PixelBuffer {
        width,
        height,
        channels: 1,
        data: bytes.into(),
    };
    gaussian_blur_planes(&mut mask, sigma_from_radius(radius as f64));
    mask.data.iter().map(|&b| b as f64 / 255.0).collect()
}

fn smoothstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    if edge0 == edge1 {
        return if x < edge0 { 0.0 } else { 1.0 };
    }
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Circular (disc) mean of the color planes, clamp-to-edge. A per-plane summed
/// area table makes the disc `O(r)` per pixel instead of `O(r²)`. With `fine`,
/// rows are sampled on a half-pixel grid (vertical bilinear) at twice the row
/// count, the "More Accurate" estimate.
fn disc_blur_planes(buf: &mut PixelBuffer, radius: u32, fine: bool) {
    if radius == 0 {
        return;
    }
    let w = buf.width as usize;
    let h = buf.height as usize;
    let n = w * h;
    if n == 0 {
        return;
    }
    let reach = radius as i32;
    let reach_f = radius as f64;
    let steps = if fine { 2 * reach } else { reach };
    let spans: Vec<(f64, i32)> = (-steps..=steps)
        .map(|k| {
            let dy = if fine { k as f64 / 2.0 } else { k as f64 };
            (dy, (reach_f * reach_f - dy * dy).max(0.0).sqrt() as i32)
        })
        .collect();
    let planes = (buf.channels as usize).min(3);
    let stride = w + 1;
    for c in 0..planes {
        let base = c * n;
        let src = buf.data[base..base + n].to_vec();
        let mut sat = vec![0u64; (h + 1) * stride];
        for y in 0..h {
            let mut running = 0u64;
            for x in 0..w {
                running += src[y * w + x] as u64;
                sat[(y + 1) * stride + (x + 1)] = sat[y * stride + (x + 1)] + running;
            }
        }
        let row_sum = |row: isize, x0: usize, x1: usize| -> u64 {
            let row = clamp_index(row, h);
            let plus = sat[(row + 1) * stride + x1 + 1] + sat[row * stride + x0];
            let minus = sat[(row + 1) * stride + x0] + sat[row * stride + x1 + 1];
            plus - minus
        };
        for y in 0..h {
            for x in 0..w {
                let mut total = 0f64;
                let mut count = 0u64;
                for &(dy, half) in &spans {
                    let x0 = (x as i32 - half).max(0) as usize;
                    let x1 = (x as i32 + half).min(w as i32 - 1) as usize;
                    let yf = y as f64 + dy;
                    let contribution = if fine {
                        let y0 = yf.floor();
                        let fy = yf - y0;
                        let r0 = y0 as isize;
                        row_sum(r0, x0, x1) as f64 * (1.0 - fy)
                            + row_sum(r0 + 1, x0, x1) as f64 * fy
                    } else {
                        let yy = yf as isize;
                        if yy < 0 || yy >= h as isize {
                            continue;
                        }
                        row_sum(yy, x0, x1) as f64
                    };
                    total += contribution;
                    count += (x1 - x0 + 1) as u64;
                }
                let value = if fine {
                    (total / count.max(1) as f64).round()
                } else {
                    (total / count.max(1) as f64).floor()
                };
                buf.data[base + y * w + x] = value.clamp(0.0, 255.0) as u8;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sharpen::tests::{gray_row, planar, plane_range, total_deviation};
    use crate::sharpen::unsharp_mask;

    fn both_removes() -> [(SharpenRemove, f64); 3] {
        [
            (SharpenRemove::GaussianBlur, 0.0),
            (SharpenRemove::LensBlur, 0.0),
            (SharpenRemove::MotionBlur, 30.0),
        ]
    }

    fn run(
        base: &PixelBuffer,
        remove: SharpenRemove,
        angle: f64,
        more_accurate: bool,
        shadow: TonalFade,
        highlight: TonalFade,
    ) -> PixelBuffer {
        let mut out = base.clone();
        smart_sharpen(
            &mut out,
            200.0,
            3.0,
            0.0,
            remove,
            angle,
            more_accurate,
            shadow,
            highlight,
        )
        .unwrap();
        out
    }

    #[test]
    fn smart_sharpen_increases_edge_contrast() {
        let base = gray_row(&[100, 100, 100, 100, 140, 140, 140, 140]);
        let out = run(
            &base,
            SharpenRemove::GaussianBlur,
            0.0,
            false,
            TonalFade::default(),
            TonalFade::default(),
        );
        assert!(plane_range(&out) > plane_range(&base));
    }

    #[test]
    fn smart_sharpen_remove_changes_the_result() {
        let base = gray_row(&[100, 100, 100, 100, 140, 140, 140, 140]);
        let g = run(
            &base,
            SharpenRemove::GaussianBlur,
            0.0,
            false,
            TonalFade::default(),
            TonalFade::default(),
        );
        let l = run(
            &base,
            SharpenRemove::LensBlur,
            0.0,
            false,
            TonalFade::default(),
            TonalFade::default(),
        );
        let m = run(
            &base,
            SharpenRemove::MotionBlur,
            30.0,
            false,
            TonalFade::default(),
            TonalFade::default(),
        );
        assert!(
            total_deviation(&base, &l) > 0,
            "lens blur must change the image"
        );
        assert!(
            total_deviation(&base, &m) > 0,
            "motion blur must change the image"
        );
        assert_ne!(g.data, l.data, "different removals differ");
    }

    #[test]
    fn more_accurate_differs_and_is_deterministic() {
        // The 4σ-vs-3σ Gaussian shift is sub-LSB for most 8-bit input; this
        // signal puts blur values on rounding boundaries so both paths differ.
        let base = gray_row(&[
            16, 166, 37, 161, 234, 239, 6, 82, 72, 215, 225, 144, 10, 77, 243, 57,
        ]);
        for (remove, angle) in both_removes() {
            let coarse = run(
                &base,
                remove,
                angle,
                false,
                TonalFade::default(),
                TonalFade::default(),
            );
            let accurate = run(
                &base,
                remove,
                angle,
                true,
                TonalFade::default(),
                TonalFade::default(),
            );
            assert_ne!(
                coarse.data, accurate.data,
                "{remove:?}: More Accurate must change the estimate"
            );
            let again = run(
                &base,
                remove,
                angle,
                true,
                TonalFade::default(),
                TonalFade::default(),
            );
            assert_eq!(
                accurate.data, again.data,
                "{remove:?}: More Accurate must be deterministic"
            );
        }
    }

    #[test]
    fn smart_sharpen_gaussian_path_equals_unsharp_mask() {
        let r = vec![10, 60, 120, 200, 250, 30, 80, 90];
        let g = vec![20, 70, 130, 210, 240, 40, 90, 100];
        let b = vec![30, 80, 140, 220, 230, 50, 100, 110];
        let a = vec![5, 95, 128, 170, 200, 255, 60, 70];
        let base = planar(8, 1, 4, &[r, g, b, a]);
        let smart = run(
            &base,
            SharpenRemove::GaussianBlur,
            0.0,
            false,
            TonalFade::default(),
            TonalFade::default(),
        );
        let mut unsharp = base.clone();
        unsharp_mask(&mut unsharp, 200.0, 3.0, 0).unwrap();
        assert_eq!(smart.data, unsharp.data);
    }

    #[test]
    fn smart_sharpen_reduce_noise_holds_back_low_contrast() {
        let base = gray_row(&[100, 100, 100, 100, 104, 104, 104, 104]);
        let mut open = base.clone();
        smart_sharpen(
            &mut open,
            200.0,
            2.0,
            0.0,
            SharpenRemove::GaussianBlur,
            0.0,
            false,
            TonalFade::default(),
            TonalFade::default(),
        )
        .unwrap();
        let mut gated = base.clone();
        smart_sharpen(
            &mut gated,
            200.0,
            2.0,
            100.0,
            SharpenRemove::GaussianBlur,
            0.0,
            false,
            TonalFade::default(),
            TonalFade::default(),
        )
        .unwrap();
        assert!(total_deviation(&base, &gated) < total_deviation(&base, &open));
    }

    #[test]
    fn smart_sharpen_rejects_invalid_params() {
        let base = gray_row(&[10, 20, 30, 40]);
        for (amount, radius, noise, angle) in [
            (-1.0, 2.0, 0.0, 0.0),
            (0.0, 2.0, 0.0, 0.0),
            (0.5, 2.0, 0.0, 0.0),
            (600.0, 2.0, 0.0, 0.0),
            (100.0, 0.0, 0.0, 0.0),
            (100.0, 0.05, 0.0, 0.0),
            (100.0, 65.0, 0.0, 0.0),
            (100.0, 2.0, 101.0, 0.0),
            (100.0, 2.0, 0.0, 500.0),
        ] {
            let mut out = base.clone();
            assert!(smart_sharpen(
                &mut out,
                amount,
                radius,
                noise,
                SharpenRemove::GaussianBlur,
                angle,
                false,
                TonalFade::default(),
                TonalFade::default(),
            )
            .is_err());
            assert_eq!(out, base, "rejected parameters must not modify the buffer");
        }
    }

    #[test]
    fn tonal_fade_rejects_invalid_params() {
        let base = gray_row(&[10, 20, 30, 40]);
        let good = TonalFade {
            amount: 100,
            width: 100,
            radius: 1,
        };
        for bad in [
            TonalFade {
                amount: 101,
                width: 50,
                radius: 1,
            },
            TonalFade {
                amount: 50,
                width: 101,
                radius: 1,
            },
            TonalFade {
                amount: 50,
                width: 50,
                radius: 0,
            },
            TonalFade {
                amount: 50,
                width: 50,
                radius: 101,
            },
        ] {
            for (shadow, highlight) in [(bad, good), (good, bad)] {
                let mut out = base.clone();
                assert!(smart_sharpen(
                    &mut out,
                    100.0,
                    2.0,
                    0.0,
                    SharpenRemove::GaussianBlur,
                    0.0,
                    false,
                    shadow,
                    highlight,
                )
                .is_err());
                assert_eq!(out, base, "rejected tonal fade must not modify the buffer");
            }
        }
    }

    fn side_deviation(base: &PixelBuffer, out: &PixelBuffer, range: std::ops::Range<usize>) -> i64 {
        range
            .map(|i| (base.data[i] as i64 - out.data[i] as i64).abs())
            .sum()
    }

    #[test]
    fn shadow_fade_damps_the_dark_side() {
        let base = gray_row(&[10, 10, 10, 10, 60, 60, 60, 60]);
        let open = run(
            &base,
            SharpenRemove::GaussianBlur,
            0.0,
            false,
            TonalFade::default(),
            TonalFade::default(),
        );
        let faded = run(
            &base,
            SharpenRemove::GaussianBlur,
            0.0,
            false,
            TonalFade {
                amount: 100,
                width: 100,
                radius: 1,
            },
            TonalFade::default(),
        );
        let dark_open = side_deviation(&base, &open, 0..4);
        let dark_faded = side_deviation(&base, &faded, 0..4);
        assert!(dark_open > 0, "baseline sharpen must change the dark side");
        assert!(
            dark_faded < dark_open,
            "shadow fade must damp the dark side: {dark_faded} !< {dark_open}"
        );
    }

    #[test]
    fn highlight_fade_damps_the_bright_side() {
        let base = gray_row(&[200, 200, 200, 200, 250, 250, 250, 250]);
        let open = run(
            &base,
            SharpenRemove::GaussianBlur,
            0.0,
            false,
            TonalFade::default(),
            TonalFade::default(),
        );
        let faded = run(
            &base,
            SharpenRemove::GaussianBlur,
            0.0,
            false,
            TonalFade::default(),
            TonalFade {
                amount: 100,
                width: 100,
                radius: 1,
            },
        );
        let bright_open = side_deviation(&base, &open, 4..8);
        let bright_faded = side_deviation(&base, &faded, 4..8);
        assert!(
            bright_open > 0,
            "baseline sharpen must change the bright side"
        );
        assert!(
            bright_faded < bright_open,
            "highlight fade must damp the bright side: {bright_faded} !< {bright_open}"
        );
    }

    #[test]
    fn zero_tonal_amount_is_transparent() {
        let base = gray_row(&[
            10, 90, 30, 200, 50, 170, 20, 220, 60, 130, 40, 180, 70, 110, 90, 150,
        ]);
        let default = run(
            &base,
            SharpenRemove::LensBlur,
            0.0,
            true,
            TonalFade::default(),
            TonalFade::default(),
        );
        let zeroed = run(
            &base,
            SharpenRemove::LensBlur,
            0.0,
            true,
            TonalFade {
                amount: 0,
                width: 100,
                radius: 50,
            },
            TonalFade {
                amount: 0,
                width: 100,
                radius: 50,
            },
        );
        assert_eq!(default.data, zeroed.data);
    }

    #[test]
    fn smart_sharpen_uniform_is_bit_identical() {
        let base = gray_row(&[123; 16]);
        let out = run(
            &base,
            SharpenRemove::GaussianBlur,
            0.0,
            false,
            TonalFade::default(),
            TonalFade::default(),
        );
        assert_eq!(
            out.data, base.data,
            "a flat blur difference is exactly zero"
        );
    }

    #[test]
    fn disc_blur_matches_naive_circular_mean() {
        let (w, h, r) = (9usize, 9usize, 2i32);
        let mut data = vec![0u8; w * h];
        data[4 * w + 4] = 200;
        let base: Vec<u8> = data.clone();
        let mut buf = PixelBuffer {
            width: w as u32,
            height: h as u32,
            channels: 1,
            data: data.into(),
        };
        disc_blur_planes(&mut buf, r as u32, false);

        let mut expected = vec![0u8; w * h];
        for y in 0..h as i32 {
            for x in 0..w as i32 {
                let (mut total, mut count) = (0u64, 0u64);
                for dy in -r..=r {
                    let yy = y + dy;
                    if yy < 0 || yy >= h as i32 {
                        continue;
                    }
                    let half = ((r * r - dy * dy) as f64).sqrt() as i32;
                    let x0 = (x - half).max(0) as usize;
                    let x1 = (x + half).min(w as i32 - 1) as usize;
                    for xx in x0..=x1 {
                        total += base[yy as usize * w + xx] as u64;
                    }
                    count += (x1 - x0 + 1) as u64;
                }
                expected[y as usize * w + x as usize] = (total / count) as u8;
            }
        }
        assert_eq!(buf.data.as_ref(), expected.as_slice());
    }

    #[test]
    fn smart_sharpen_preserves_alpha_and_is_deterministic() {
        let r = vec![10, 60, 120, 200, 250, 30, 80, 90];
        let g = vec![20, 70, 130, 210, 240, 40, 90, 100];
        let b = vec![30, 80, 140, 220, 230, 50, 100, 110];
        let a = vec![5, 95, 128, 170, 200, 255, 60, 70];
        let base = planar(8, 1, 4, &[r, g, b, a.clone()]);
        let n = base.pixel_count();
        let shadow = TonalFade {
            amount: 100,
            width: 50,
            radius: 2,
        };
        let highlight = TonalFade {
            amount: 80,
            width: 50,
            radius: 2,
        };
        let x = run(
            &base,
            SharpenRemove::MotionBlur,
            30.0,
            true,
            shadow,
            highlight,
        );
        let y = run(
            &base,
            SharpenRemove::MotionBlur,
            30.0,
            true,
            shadow,
            highlight,
        );
        assert_eq!(x.data, y.data, "no seed, so re-apply is byte-identical");
        assert_eq!(&x.data[3 * n..4 * n], &a[..]);
    }
}
