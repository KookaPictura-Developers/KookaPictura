//! Oil Paint filter (`m25-*`).
//!
//! CS6 runs Oil Paint only on a supported GPU; this is a CPU behavioural model
//! and therefore a deliberate non-parity divergence. It is deterministic (no
//! seed) and never touches alpha.

use std::f64::consts::PI;

use pictura_core::PixelBuffer;

use crate::artistic::reduce::clamp_u8;
use crate::kernel::clamp_index;
use crate::luma::luma;
use crate::{validate, FilterError};

fn check_unit(name: &str, v: f64) -> Result<(), FilterError> {
    if v.is_finite() && (0.0..=10.0).contains(&v) {
        Ok(())
    } else {
        Err(FilterError::InvalidParams(format!(
            "{name} {v} is outside 0..=10"
        )))
    }
}

fn normalize(v: (f64, f64, f64)) -> (f64, f64, f64) {
    let len = (v.0 * v.0 + v.1 * v.1 + v.2 * v.2).sqrt();
    if len < 1e-9 {
        (0.0, 0.0, 1.0)
    } else {
        (v.0 / len, v.1 / len, v.2 / len)
    }
}

/// ponytail: CS6's OpenCL shader is closed and GPU-only; this CPU model is
/// edge-aware directional smoothing plus Lambert/Blinn-Phong relief shading.
/// The stroke tangent comes from the Sobel gradient, `cleanliness` scales the
/// along-stroke reach, `stylization` the edge-stop threshold and cross-stroke
/// width, and `scale`/`bristle_detail` drive the height field's lighting.
pub fn oil_paint(
    buf: &mut PixelBuffer,
    stylization: f64,
    cleanliness: f64,
    scale: f64,
    bristle_detail: f64,
    angular_direction: f64,
    shine: f64,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    check_unit("oil paint stylization", stylization)?;
    check_unit("oil paint cleanliness", cleanliness)?;
    check_unit("oil paint scale", scale)?;
    check_unit("oil paint bristle detail", bristle_detail)?;
    check_unit("oil paint shine", shine)?;
    if !angular_direction.is_finite() || !(0.0..=360.0).contains(&angular_direction) {
        return Err(FilterError::InvalidParams(format!(
            "oil paint angular direction {angular_direction} is outside 0..=360"
        )));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let src = buf.data.clone();

    let lum: Vec<f64> = (0..n)
        .map(|i| luma(src[i] as f64, src[n + i] as f64, src[2 * n + i] as f64))
        .collect();
    let at = |x: isize, y: isize| lum[clamp_index(y, h) * w + clamp_index(x, w)];

    // 1-2. structure orientation (gradient tangent) + edge-stopping aggregation.
    let along = (1.0 + cleanliness * 0.9).round().max(1.0) as isize;
    let perp = (stylization * 0.4).round() as isize;
    let stop = 10.0 + stylization * 8.0;
    let mut agg = vec![[0.0f64; 3]; n];
    for y in 0..h {
        for x in 0..w {
            let xi = x as isize;
            let yi = y as isize;
            let li = at(xi, yi);
            let gx = (at(xi + 1, yi - 1) + 2.0 * at(xi + 1, yi) + at(xi + 1, yi + 1))
                - (at(xi - 1, yi - 1) + 2.0 * at(xi - 1, yi) + at(xi - 1, yi + 1));
            let gy = (at(xi - 1, yi + 1) + 2.0 * at(xi, yi + 1) + at(xi + 1, yi + 1))
                - (at(xi - 1, yi - 1) + 2.0 * at(xi, yi - 1) + at(xi + 1, yi - 1));
            let theta = gy.atan2(gx) + PI / 2.0;
            let (tx, ty) = (theta.cos(), theta.sin());
            let (nx, ny) = (-ty, tx);
            let mut wsum = 0.0;
            let mut acc = [0.0f64; 3];
            for d in -along..=along {
                for k in -perp..=perp {
                    let ox = (tx * d as f64 + nx * k as f64).round() as isize;
                    let oy = (ty * d as f64 + ny * k as f64).round() as isize;
                    let sx = clamp_index(xi + ox, w);
                    let sy = clamp_index(yi + oy, h);
                    let q = sy * w + sx;
                    let dl = lum[q] - li;
                    let edge = 1.0 / (1.0 + (dl / stop).powi(2));
                    let space = 1.0 / (1.0 + (d * d + k * k) as f64 * 0.12);
                    let wt = edge * space;
                    wsum += wt;
                    for (c, a) in acc.iter_mut().enumerate().take(planes) {
                        *a += src[c * n + q] as f64 * wt;
                    }
                }
            }
            let p = y * w + x;
            for (c, a) in acc.iter().enumerate().take(planes) {
                agg[p][c] = a / wsum;
            }
        }
    }

    // 3-5. height field (luma thickness + bristle ridge) + Lambert/Blinn-Phong.
    let scale_t = scale / 10.0;
    let bristle = bristle_detail / 10.0;
    let ridge = |x: usize, y: usize| -> f64 {
        let r = ((x * 37 + y * 17) as f64 * 0.11).sin();
        r * r
    };
    let mut height = vec![0.0f64; n];
    for y in 0..h {
        for x in 0..w {
            let p = y * w + x;
            let al = luma(agg[p][0], agg[p][1], agg[p][2]) / 255.0;
            height[p] = al * (0.2 + 0.8 * scale_t) + bristle * 0.35 * ridge(x, y);
        }
    }

    let relief_mag = 1.0 + scale * 1.5;
    let azimuth = angular_direction.to_radians();
    let light = normalize((azimuth.cos(), azimuth.sin(), 0.6));
    let half = normalize((light.0, light.1, light.2 + 1.0));
    let shine_t = shine / 10.0;

    for y in 0..h {
        for x in 0..w {
            let p = y * w + x;
            let gx = height[y * w + clamp_index(x as isize + 1, w)]
                - height[y * w + clamp_index(x as isize - 1, w)];
            let gy = height[clamp_index(y as isize + 1, h) * w + x]
                - height[clamp_index(y as isize - 1, h) * w + x];
            let nrm = normalize((-gx * relief_mag, -gy * relief_mag, 1.0));
            let lambert = (nrm.0 * light.0 + nrm.1 * light.1 + nrm.2 * light.2).max(0.0);
            let spec = (nrm.0 * half.0 + nrm.1 * half.1 + nrm.2 * half.2)
                .max(0.0)
                .powf(28.0)
                * shine_t;
            let lightf = 0.55 + 0.45 * lambert;
            for (c, &a) in agg[p].iter().enumerate().take(planes) {
                buf.data[c * n + p] = clamp_u8(a * lightf + 255.0 * spec * 0.7);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{apply, Filter};

    fn gradient(w: u32, h: u32, channels: u8) -> PixelBuffer {
        let n = (w * h) as usize;
        let mut data = vec![0u8; n * channels as usize];
        let denom = (w.max(2) - 1) as f64;
        for y in 0..h as usize {
            for x in 0..w as usize {
                let i = y * w as usize + x;
                let step = if x < (w / 2) as usize { 0.0 } else { 100.0 };
                let v = (255.0 * x as f64 / denom + step).min(255.0).round() as u8;
                for c in 0..(channels as usize).min(3) {
                    data[c * n + i] = v;
                }
                if channels == 4 {
                    data[3 * n + i] = 137;
                }
            }
        }
        PixelBuffer {
            width: w,
            height: h,
            channels,
            data,
        }
    }

    fn alpha_plane(buf: &PixelBuffer) -> Vec<u8> {
        let n = buf.pixel_count();
        buf.data[3 * n..4 * n].to_vec()
    }

    #[test]
    fn oil_paint_changes_the_colour_planes() {
        let base = gradient(32, 8, 3);
        let mut b = base.clone();
        oil_paint(&mut b, 4.0, 5.0, 6.0, 4.0, 135.0, 3.0).unwrap();
        assert_ne!(b.data, base.data, "oil paint must change the image");
    }

    #[test]
    fn oil_paint_preserves_alpha_through_apply() {
        let base = gradient(32, 8, 4);
        let alpha = alpha_plane(&base);
        let mut b = base.clone();
        apply(
            &Filter::OilPaint {
                stylization: 4.0,
                cleanliness: 5.0,
                scale: 6.0,
                bristle_detail: 4.0,
                angular_direction: 135.0,
                shine: 3.0,
            },
            &mut b,
        )
        .unwrap();
        assert_eq!(alpha_plane(&b), alpha, "alpha must be untouched");
    }

    #[test]
    fn boundaries_are_accepted_and_bad_values_rejected() {
        let base = gradient(8, 8, 3);
        for &(styl, clean, sc, bristle, angle, shine) in &[
            (0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            (10.0, 10.0, 10.0, 10.0, 360.0, 10.0),
            (5.0, 5.0, 5.0, 5.0, 180.0, 5.0),
        ] {
            let mut b = base.clone();
            assert!(
                oil_paint(&mut b, styl, clean, sc, bristle, angle, shine).is_ok(),
                "boundary ({styl},{clean},{sc},{bristle},{angle},{shine}) must be accepted"
            );
        }
        let mut b = base.clone();
        assert!(oil_paint(&mut b, 10.5, 5.0, 5.0, 5.0, 180.0, 5.0).is_err());
        assert!(oil_paint(&mut b, 5.0, -0.1, 5.0, 5.0, 180.0, 5.0).is_err());
        assert!(oil_paint(&mut b, 5.0, 5.0, 5.0, 5.0, 361.0, 5.0).is_err());
        assert!(oil_paint(&mut b, 5.0, 5.0, 5.0, 5.0, 180.0, 10.1).is_err());
        assert!(oil_paint(&mut b, f64::NAN, 5.0, 5.0, 5.0, 180.0, 5.0).is_err());
        assert!(oil_paint(&mut b, 5.0, 5.0, f64::INFINITY, 5.0, 180.0, 5.0).is_err());
        assert!(oil_paint(&mut b, 5.0, 5.0, 5.0, 5.0, f64::NAN, 5.0).is_err());
    }

    #[test]
    fn oil_paint_is_deterministic() {
        let base = gradient(24, 12, 3);
        let (mut a, mut b) = (base.clone(), base.clone());
        oil_paint(&mut a, 3.0, 4.0, 5.0, 2.0, 85.0, 1.0).unwrap();
        oil_paint(&mut b, 3.0, 4.0, 5.0, 2.0, 85.0, 1.0).unwrap();
        assert_eq!(a.data, b.data, "two identical applies must match");
    }

    #[test]
    fn scale_and_stylization_change_the_result() {
        let base = gradient(32, 32, 3);
        let (mut thin, mut thick) = (base.clone(), base.clone());
        oil_paint(&mut thin, 4.0, 5.0, 0.0, 3.0, 135.0, 2.0).unwrap();
        oil_paint(&mut thick, 4.0, 5.0, 10.0, 3.0, 135.0, 2.0).unwrap();
        assert_ne!(thin.data, thick.data, "scale must change the result");

        let (mut daubed, mut smooth) = (base.clone(), base.clone());
        oil_paint(&mut daubed, 0.0, 5.0, 5.0, 3.0, 135.0, 2.0).unwrap();
        oil_paint(&mut smooth, 10.0, 5.0, 5.0, 3.0, 135.0, 2.0).unwrap();
        assert_ne!(
            daubed.data, smooth.data,
            "stylization must change the result"
        );
    }
}
