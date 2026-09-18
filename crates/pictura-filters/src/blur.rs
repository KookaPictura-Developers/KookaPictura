//! Blur family: Gaussian, Box, Motion, Radial, Average, Blur/Blur More, Surface.
//!
//! Planar 8-bit buffers (channels 3 or 4); only the color planes are touched,
//! alpha is never modified. Borders clamp-to-edge per `FILT-010`.

use pictura_core::PixelBuffer;

use crate::kernel::{clamp_index, convolve3x3_planes, gaussian_blur_planes, sigma_from_radius};
use crate::luma::luma;
use crate::{validate, FilterError, Quality, RadialMethod};

pub fn gaussian(buf: &mut PixelBuffer, radius: f64) -> Result<(), FilterError> {
    validate(buf)?;
    if !radius.is_finite() || radius < 0.0 {
        return Err(FilterError::InvalidParams(format!(
            "gaussian radius {radius} must be finite and non-negative"
        )));
    }
    if radius == 0.0 {
        return Ok(());
    }
    gaussian_blur_planes(buf, sigma_from_radius(radius));
    Ok(())
}

pub fn r#box(buf: &mut PixelBuffer, radius: u32) -> Result<(), FilterError> {
    validate(buf)?;
    if radius == 0 {
        return Ok(());
    }
    box_blur_planes(buf, radius);
    Ok(())
}

pub fn motion(buf: &mut PixelBuffer, angle_deg: f64, distance: u32) -> Result<(), FilterError> {
    validate(buf)?;
    if !angle_deg.is_finite() || !(-360.0..=360.0).contains(&angle_deg) {
        return Err(FilterError::InvalidParams(format!(
            "motion angle {angle_deg} is outside -360..=360"
        )));
    }
    if !(1..=999).contains(&distance) {
        return Err(FilterError::InvalidParams(format!(
            "motion distance {distance} is outside 1..=999"
        )));
    }
    if distance == 1 {
        return Ok(());
    }
    let w = buf.width as usize;
    let h = buf.height as usize;
    let n = w * h;
    let planes = (buf.channels as usize).min(3);
    let angle = angle_deg.to_radians();
    let (dx, dy) = (angle.cos(), angle.sin());
    let taps = distance as isize;
    let half = (distance as f64 - 1.0) / 2.0;
    let divisor = distance as f64;
    for c in 0..planes {
        let base = c * n;
        let src = buf.data[base..base + n].to_vec();
        for y in 0..h {
            for x in 0..w {
                let mut acc = 0f64;
                for k in 0..taps {
                    let f = k as f64 - half;
                    let sx = clamp_index(x as isize + (f * dx).round() as isize, w);
                    let sy = clamp_index(y as isize + (f * dy).round() as isize, h);
                    acc += src[sy * w + sx] as f64;
                }
                buf.data[base + y * w + x] = (acc / divisor).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    Ok(())
}

pub fn radial(
    buf: &mut PixelBuffer,
    method: RadialMethod,
    amount: f64,
    quality: Quality,
) -> Result<(), FilterError> {
    validate(buf)?;
    if !amount.is_finite() || amount < 0.0 {
        return Err(FilterError::InvalidParams(format!(
            "radial amount {amount} must be finite and non-negative"
        )));
    }
    if amount == 0.0 {
        return Ok(());
    }
    if matches!(method, RadialMethod::Zoom) && amount > 100.0 {
        return Err(FilterError::InvalidParams(format!(
            "radial zoom amount {amount} is outside 1..=100"
        )));
    }
    let w = buf.width as usize;
    let h = buf.height as usize;
    let n = w * h;
    let planes = (buf.channels as usize).min(3);
    let samples: usize = match quality {
        Quality::Draft => 8,
        Quality::Good => 16,
        Quality::Best => 32,
    };
    let cx = (w as f64 - 1.0) / 2.0;
    let cy = (h as f64 - 1.0) / 2.0;
    // ponytail: polar resample + midpoint 1-D smear, not Adobe's kernel.
    // Upgrade to supersampled/jittered sampling if Draft banding is visible.
    for c in 0..planes {
        let base = c * n;
        let src = buf.data[base..base + n].to_vec();
        for y in 0..h {
            for x in 0..w {
                let px = x as f64 - cx;
                let py = y as f64 - cy;
                let radius = (px * px + py * py).sqrt();
                let theta = py.atan2(px);
                let mut acc = 0f64;
                match method {
                    RadialMethod::Spin => {
                        let dt = amount.to_radians() / samples as f64;
                        for k in 0..samples {
                            let off = (k as f64 - (samples as f64 - 1.0) / 2.0) * dt;
                            let t = theta + off;
                            acc +=
                                bilinear(&src, w, h, cx + radius * t.cos(), cy + radius * t.sin());
                        }
                    }
                    RadialMethod::Zoom => {
                        let k_amt = (amount / 100.0).clamp(0.0, 1.0);
                        for k in 0..samples {
                            let t = k as f64 / (samples as f64 - 1.0);
                            let frac = 1.0 - k_amt / 2.0 + k_amt * t;
                            let rr = radius * frac;
                            acc +=
                                bilinear(&src, w, h, cx + rr * theta.cos(), cy + rr * theta.sin());
                        }
                    }
                }
                buf.data[base + y * w + x] = (acc / samples as f64).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    Ok(())
}

pub fn average(buf: &mut PixelBuffer) -> Result<(), FilterError> {
    validate(buf)?;
    let n = buf.pixel_count();
    let planes = (buf.channels as usize).min(3);
    for c in 0..planes {
        let base = c * n;
        let sum: u64 = buf.data[base..base + n].iter().map(|&v| v as u64).sum();
        let mean = (sum as f64 / n as f64).round().clamp(0.0, 255.0) as u8;
        buf.data[base..base + n].fill(mean);
    }
    Ok(())
}

pub fn simple(buf: &mut PixelBuffer, more: bool) -> Result<(), FilterError> {
    validate(buf)?;
    let kernel = [[1.0, 2.0, 1.0], [2.0, 4.0, 2.0], [1.0, 2.0, 1.0]];
    for _ in 0..if more { 3 } else { 1 } {
        convolve3x3_planes(buf, &kernel, 16.0);
    }
    Ok(())
}

pub fn surface(buf: &mut PixelBuffer, radius: u32, threshold: u8) -> Result<(), FilterError> {
    validate(buf)?;
    if !(1..=100).contains(&radius) {
        return Err(FilterError::InvalidParams(format!(
            "surface radius {radius} is outside 1..=100"
        )));
    }
    if threshold == 0 {
        return Err(FilterError::InvalidParams(
            "surface threshold must be 1..=255".into(),
        ));
    }
    let w = buf.width as usize;
    let h = buf.height as usize;
    let n = w * h;
    let planes = (buf.channels as usize).min(3);
    let r = radius as isize;
    let sigma = sigma_from_radius(radius as f64);
    let two_sig_sq = 2.0 * sigma * sigma;
    let thr = threshold as f64;
    let two_thr_sq = 2.0 * thr * thr;
    // ponytail: direct O(n·r²) bilateral. Upgrade to a guided filter (or a
    // separable box-range approximation) only if large-radius cost bites.
    let src: Vec<Vec<f64>> = (0..planes)
        .map(|c| {
            let base = c * n;
            buf.data[base..base + n].iter().map(|&v| v as f64).collect()
        })
        .collect();
    let lum: Vec<f64> = (0..n)
        .map(|i| luma(src[0][i], src[1][i], src[2][i]))
        .collect();
    for y in 0..h {
        for x in 0..w {
            let ci = y * w + x;
            let center_luma = lum[ci];
            let mut weight_sum = 0f64;
            let mut acc = [0f64; 3];
            for ky in -r..=r {
                let sy = clamp_index(y as isize + ky, h);
                let dy2 = (ky * ky) as f64;
                for kx in -r..=r {
                    let sx = clamp_index(x as isize + kx, w);
                    let si = sy * w + sx;
                    let mut wgt = (-(((kx * kx) as f64) + dy2) / two_sig_sq).exp();
                    let dl = (lum[si] - center_luma).abs();
                    wgt *= (-(dl * dl) / two_thr_sq).exp();
                    weight_sum += wgt;
                    acc[0] += wgt * src[0][si];
                    acc[1] += wgt * src[1][si];
                    acc[2] += wgt * src[2][si];
                }
            }
            for (c, a) in acc.iter().enumerate() {
                buf.data[c * n + ci] = (a / weight_sum).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    Ok(())
}

fn bilinear(plane: &[u8], w: usize, h: usize, x: f64, y: f64) -> f64 {
    let x0 = x.floor();
    let y0 = y.floor();
    let fx = x - x0;
    let fy = y - y0;
    let xi = clamp_index(x0 as isize, w);
    let yi = clamp_index(y0 as isize, h);
    let xi1 = clamp_index(x0 as isize + 1, w);
    let yi1 = clamp_index(y0 as isize + 1, h);
    let p00 = plane[yi * w + xi] as f64;
    let p10 = plane[yi * w + xi1] as f64;
    let p01 = plane[yi1 * w + xi] as f64;
    let p11 = plane[yi1 * w + xi1] as f64;
    (p00 * (1.0 - fx) + p10 * fx) * (1.0 - fy) + (p01 * (1.0 - fx) + p11 * fx) * fy
}

fn box_blur_planes(buf: &mut PixelBuffer, radius: u32) {
    let w = buf.width as usize;
    let h = buf.height as usize;
    let n = w * h;
    let planes = (buf.channels as usize).min(3);
    let r = radius as isize;
    for c in 0..planes {
        let base = c * n;
        let src: Vec<f64> = buf.data[base..base + n].iter().map(|&v| v as f64).collect();
        let mut tmp = vec![0f64; n];
        box_blur_axis(&src, &mut tmp, w, h, r, true);
        let mut out = vec![0f64; n];
        box_blur_axis(&tmp, &mut out, w, h, r, false);
        for (dst, v) in buf.data[base..base + n].iter_mut().zip(out.iter()) {
            *dst = v.round().clamp(0.0, 255.0) as u8;
        }
    }
}

/// Separable moving average over `(2r+1)` taps, clamp-to-edge. Prefix sums keep
/// it O(1) per pixel even when the radius exceeds the image.
fn box_blur_axis(src: &[f64], dst: &mut [f64], w: usize, h: usize, r: isize, horizontal: bool) {
    let taps = (2 * r + 1) as f64;
    if horizontal {
        let mut line = vec![0f64; w];
        let mut pre = vec![0f64; w + 1];
        for y in 0..h {
            line.copy_from_slice(&src[y * w..y * w + w]);
            pre[0] = 0.0;
            for (k, &v) in line.iter().enumerate() {
                pre[k + 1] = pre[k] + v;
            }
            for x in 0..w {
                let a = x as isize - r;
                let b = x as isize + r;
                dst[y * w + x] = (prefix_at(&pre, &line, b) - prefix_at(&pre, &line, a - 1)) / taps;
            }
        }
    } else {
        let mut line = vec![0f64; h];
        let mut pre = vec![0f64; h + 1];
        for x in 0..w {
            for y in 0..h {
                line[y] = src[y * w + x];
            }
            pre[0] = 0.0;
            for (k, &v) in line.iter().enumerate() {
                pre[k + 1] = pre[k] + v;
            }
            for y in 0..h {
                let a = y as isize - r;
                let b = y as isize + r;
                dst[y * w + x] = (prefix_at(&pre, &line, b) - prefix_at(&pre, &line, a - 1)) / taps;
            }
        }
    }
}

/// `H(t) = Σ_{i=0..t} clamp(i, n)` where `pre[k] = Σ_{i<k} v[i]`; `H(-1) = 0`.
fn prefix_at(pre: &[f64], v: &[f64], t: isize) -> f64 {
    let n = v.len() as isize;
    if t < 0 {
        (t + 1) as f64 * v[0]
    } else if t >= n {
        pre[v.len()] + (t - n + 1) as f64 * v[v.len() - 1]
    } else {
        pre[t as usize + 1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buf3(w: u32, h: u32, px: &[[u8; 3]]) -> PixelBuffer {
        let n = (w * h) as usize;
        assert_eq!(px.len(), n);
        let mut data = vec![0u8; n * 3];
        for (i, p) in px.iter().enumerate() {
            data[i] = p[0];
            data[n + i] = p[1];
            data[2 * n + i] = p[2];
        }
        PixelBuffer {
            width: w,
            height: h,
            channels: 3,
            data,
        }
    }

    fn buf4(w: u32, h: u32, px: &[[u8; 4]]) -> PixelBuffer {
        let n = (w * h) as usize;
        assert_eq!(px.len(), n);
        let mut data = vec![0u8; n * 4];
        for (i, p) in px.iter().enumerate() {
            data[i] = p[0];
            data[n + i] = p[1];
            data[2 * n + i] = p[2];
            data[3 * n + i] = p[3];
        }
        PixelBuffer {
            width: w,
            height: h,
            channels: 4,
            data,
        }
    }

    fn px3(buf: &PixelBuffer, x: u32, y: u32) -> [u8; 3] {
        let w = buf.width as usize;
        let n = buf.pixel_count();
        let i = y as usize * w + x as usize;
        [buf.data[i], buf.data[n + i], buf.data[2 * n + i]]
    }

    fn solid4(w: u32, h: u32, rgb: [u8; 3], a: u8) -> PixelBuffer {
        let px: Vec<[u8; 4]> = (0..w * h).map(|_| [rgb[0], rgb[1], rgb[2], a]).collect();
        buf4(w, h, &px)
    }

    #[test]
    fn gaussian_box_simple_are_near_noop_on_solid() {
        let base = solid4(6, 5, [40, 90, 160], 123);
        let mut g = base.clone();
        gaussian(&mut g, 3.0).unwrap();
        let mut b = base.clone();
        r#box(&mut b, 3).unwrap();
        let mut s = base.clone();
        simple(&mut s, true).unwrap();
        for y in 0..5 {
            for x in 0..6 {
                for buf in [&g, &b, &s] {
                    let p = px3(buf, x, y);
                    assert!((p[0] as i32 - 40).abs() <= 1);
                    assert!((p[1] as i32 - 90).abs() <= 1);
                    assert!((p[2] as i32 - 160).abs() <= 1);
                }
            }
        }
    }

    #[test]
    fn gaussian_edge_transition_widens_with_radius() {
        let px: Vec<[u8; 3]> = (0..16)
            .map(|x| if x < 8 { [0, 0, 0] } else { [255, 255, 255] })
            .collect();
        let base = buf3(16, 1, &px);
        let mut small = base.clone();
        gaussian(&mut small, 1.0).unwrap();
        let mut large = base.clone();
        gaussian(&mut large, 3.0).unwrap();
        let partial = |b: &PixelBuffer| {
            (0..16)
                .filter(|&x| b.data[x] > 0 && b.data[x] < 255)
                .count()
        };
        assert!(
            partial(&large) > partial(&small),
            "large={} small={}",
            partial(&large),
            partial(&small)
        );
    }

    #[test]
    fn motion_at_zero_streaks_horizontally() {
        let mut b = PixelBuffer::new(9, 9, 3);
        b.data[4 * 9 + 4] = 255;
        motion(&mut b, 0.0, 5).unwrap();
        assert_eq!(px3(&b, 4, 4)[0], 51);
        assert_eq!(px3(&b, 2, 4)[0], 51);
        assert_eq!(px3(&b, 6, 4)[0], 51);
        assert_eq!(px3(&b, 4, 3)[0], 0);
        assert_eq!(px3(&b, 4, 5)[0], 0);
    }

    #[test]
    fn motion_at_ninety_streaks_vertically() {
        let mut b = PixelBuffer::new(9, 9, 3);
        b.data[4 * 9 + 4] = 255;
        motion(&mut b, 90.0, 5).unwrap();
        assert_eq!(px3(&b, 4, 4)[0], 51);
        assert_eq!(px3(&b, 4, 2)[0], 51);
        assert_eq!(px3(&b, 4, 6)[0], 51);
        assert_eq!(px3(&b, 3, 4)[0], 0);
        assert_eq!(px3(&b, 5, 4)[0], 0);
    }

    #[test]
    fn average_equals_region_mean() {
        let px = [[10, 20, 30], [20, 30, 40], [30, 40, 50], [40, 50, 60]];
        let mut b = buf3(2, 2, &px);
        average(&mut b).unwrap();
        for y in 0..2 {
            for x in 0..2 {
                assert_eq!(px3(&b, x, y), [25, 35, 45]);
            }
        }
    }

    #[test]
    fn simple_more_changes_more_than_simple() {
        let mut base = PixelBuffer::new(9, 9, 3);
        base.data[4 * 9 + 4] = 255;
        let mut less = base.clone();
        simple(&mut less, false).unwrap();
        let mut more = base.clone();
        simple(&mut more, true).unwrap();
        let diff = |a: &PixelBuffer, b: &PixelBuffer| {
            a.data
                .iter()
                .zip(b.data.iter())
                .map(|(&x, &y)| (x as i32 - y as i32).unsigned_abs() as u64)
                .sum::<u64>()
        };
        assert!(diff(&base, &more) > diff(&base, &less));
    }

    #[test]
    fn surface_preserves_edge_but_higher_threshold_smooths_across_it() {
        let px: Vec<[u8; 3]> = (0..16)
            .map(|x| if x < 8 { [50, 50, 50] } else { [200, 200, 200] })
            .collect();
        let base = buf3(16, 1, &px);
        let mut tight = base.clone();
        surface(&mut tight, 3, 10).unwrap();
        let mut loose = base.clone();
        surface(&mut loose, 3, 255).unwrap();
        assert!(
            (px3(&tight, 2, 0)[0] as i32 - 50).abs() <= 1,
            "flat region drifted"
        );
        assert!(
            px3(&tight, 7, 0)[0] < 60,
            "tight threshold leaked across edge"
        );
        assert!(px3(&loose, 7, 0)[0] > px3(&tight, 7, 0)[0]);
    }

    #[test]
    fn radial_changes_image_and_zero_amount_is_noop() {
        let mut base = PixelBuffer::new(11, 11, 3);
        base.data[2 * 11 + 2] = 255;
        let mut zoom = base.clone();
        radial(&mut zoom, RadialMethod::Zoom, 50.0, Quality::Good).unwrap();
        assert!(base.data.iter().zip(zoom.data.iter()).any(|(a, b)| a != b));
        let mut spin = base.clone();
        radial(&mut spin, RadialMethod::Spin, 20.0, Quality::Good).unwrap();
        assert!(base.data.iter().zip(spin.data.iter()).any(|(a, b)| a != b));
        let mut noop = base.clone();
        radial(&mut noop, RadialMethod::Spin, 0.0, Quality::Best).unwrap();
        assert_eq!(noop.data, base.data);
    }

    #[test]
    fn alpha_is_untouched_by_every_blur() {
        let expected: Vec<u8> = (0..20u32).map(|i| (i * 7) as u8).collect();
        let px: Vec<[u8; 4]> = expected.iter().map(|&a| [30, 60, 90, a]).collect();
        let base = buf4(5, 4, &px);
        let check = |b: &PixelBuffer| {
            let n = b.pixel_count();
            for (i, &a) in expected.iter().enumerate() {
                assert_eq!(b.data[3 * n + i], a);
            }
        };
        let mut b1 = base.clone();
        gaussian(&mut b1, 2.0).unwrap();
        check(&b1);
        let mut b2 = base.clone();
        r#box(&mut b2, 2).unwrap();
        check(&b2);
        let mut b3 = base.clone();
        motion(&mut b3, 30.0, 5).unwrap();
        check(&b3);
        let mut b4 = base.clone();
        radial(&mut b4, RadialMethod::Spin, 10.0, Quality::Good).unwrap();
        check(&b4);
        let mut b5 = base.clone();
        average(&mut b5).unwrap();
        check(&b5);
        let mut b6 = base.clone();
        simple(&mut b6, true).unwrap();
        check(&b6);
        let mut b7 = base.clone();
        surface(&mut b7, 2, 30).unwrap();
        check(&b7);
    }

    #[test]
    fn blur_parameter_ranges_are_validated() {
        let mut b = PixelBuffer::new(4, 4, 3);
        assert!(gaussian(&mut b, -1.0).is_err());
        assert!(gaussian(&mut b, f64::NAN).is_err());
        assert!(motion(&mut b, 400.0, 3).is_err());
        assert!(motion(&mut b, 0.0, 0).is_err());
        assert!(motion(&mut b, 0.0, 1000).is_err());
        assert!(radial(&mut b, RadialMethod::Zoom, 101.0, Quality::Good).is_err());
        assert!(radial(&mut b, RadialMethod::Spin, f64::NAN, Quality::Good).is_err());
        assert!(surface(&mut b, 0, 10).is_err());
        assert!(surface(&mut b, 101, 10).is_err());
        assert!(surface(&mut b, 2, 0).is_err());
        assert!(gaussian(&mut b, 0.0).is_ok());
        assert!(motion(&mut b, 0.0, 1).is_ok());
        assert!(radial(&mut b, RadialMethod::Spin, 0.0, Quality::Good).is_ok());
    }

    #[test]
    fn tiny_images_do_not_panic() {
        for (w, h) in [(1, 1), (1, 7), (7, 1)] {
            let n = (w * h) as usize;
            let px3v: Vec<[u8; 3]> = (0..n)
                .map(|i| [(i * 3) as u8, (i * 5) as u8, (i * 7) as u8])
                .collect();
            let mut a = buf3(w, h, &px3v);
            gaussian(&mut a, 5.0).unwrap();
            r#box(&mut a, 4).unwrap();
            simple(&mut a, true).unwrap();
            average(&mut a).unwrap();
            motion(&mut a, 45.0, 7).unwrap();
            radial(&mut a, RadialMethod::Zoom, 50.0, Quality::Draft).unwrap();
            surface(&mut a, 3, 20).unwrap();

            let px4v: Vec<[u8; 4]> = (0..n)
                .map(|i| [(i * 3) as u8, (i * 5) as u8, (i * 7) as u8, 200])
                .collect();
            let mut b = buf4(w, h, &px4v);
            gaussian(&mut b, 5.0).unwrap();
            r#box(&mut b, 4).unwrap();
            simple(&mut b, true).unwrap();
            average(&mut b).unwrap();
            motion(&mut b, 45.0, 7).unwrap();
            radial(&mut b, RadialMethod::Spin, 10.0, Quality::Best).unwrap();
            surface(&mut b, 3, 20).unwrap();
        }
    }
}
