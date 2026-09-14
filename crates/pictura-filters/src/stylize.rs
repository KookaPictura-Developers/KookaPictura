//! Stylize family: Emboss, Find Edges, Solarize (`FILT-050`). Implemented by
//! the M7-B task.

use pictura_core::PixelBuffer;

use crate::kernel::clamp_index;
use crate::luma::luma;
use crate::{validate, FilterError};

pub fn emboss(
    buf: &mut PixelBuffer,
    angle: f64,
    height: f64,
    amount: f64,
) -> Result<(), FilterError> {
    validate(buf)?;
    if !angle.is_finite() || !(-360.0..=360.0).contains(&angle) {
        return Err(FilterError::InvalidParams(format!(
            "emboss angle {angle} is outside -360..=360"
        )));
    }
    if !height.is_finite() || height <= 0.0 {
        return Err(FilterError::InvalidParams(format!(
            "emboss height {height} must be finite and positive"
        )));
    }
    if !amount.is_finite() || amount <= 0.0 {
        return Err(FilterError::InvalidParams(format!(
            "emboss amount {amount} must be finite and positive"
        )));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let n = w * h;
    let planes = (buf.channels as usize).min(3);
    let lum: Vec<f64> = (0..n)
        .map(|i| {
            luma(
                buf.data[i] as f64,
                buf.data[n + i] as f64,
                buf.data[2 * n + i] as f64,
            )
        })
        .collect();

    let rad = angle.to_radians();
    let dx = rad.cos().round() as isize;
    let dy = rad.sin().round() as isize;
    let gain = height * amount / 100.0;

    // ponytail: directional second difference (centre minus its two neighbours
    // along the angle) is a high-pass, so it flips sign across one edge like PS
    // Emboss. A true first directional derivative does not flip. Swap in a
    // subpixel kernel only if angle precision beyond the 8 neighbours matters.
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let plus = lum[clamp_index(y as isize + dy, h) * w + clamp_index(x as isize + dx, w)];
            let minus = lum[clamp_index(y as isize - dy, h) * w + clamp_index(x as isize - dx, w)];
            let response = 2.0 * lum[i] - plus - minus;
            let v = (128.0 + response * gain).round().clamp(0.0, 255.0) as u8;
            for c in 0..planes {
                buf.data[c * n + i] = v;
            }
        }
    }
    Ok(())
}

pub fn find_edges(buf: &mut PixelBuffer) -> Result<(), FilterError> {
    validate(buf)?;
    let w = buf.width as usize;
    let h = buf.height as usize;
    let n = w * h;
    let planes = (buf.channels as usize).min(3);
    let src = buf.data.clone();

    // ponytail: PS renders edges dark on a light field; the sign/inversion
    // convention is approximate — un-normalized Sobel magnitude, then
    // 255 - clamp(magnitude). Fitting Adobe's exact detector is out of scope.
    for c in 0..planes {
        let base = c * n;
        let at = |x: usize, y: usize| src[base + y * w + x] as f64;
        for y in 0..h {
            for x in 0..w {
                let xl = clamp_index(x as isize - 1, w);
                let xr = clamp_index(x as isize + 1, w);
                let yu = clamp_index(y as isize - 1, h);
                let yd = clamp_index(y as isize + 1, h);
                let gx = (at(xr, yu) + 2.0 * at(xr, y) + at(xr, yd))
                    - (at(xl, yu) + 2.0 * at(xl, y) + at(xl, yd));
                let gy = (at(xl, yd) + 2.0 * at(x, yd) + at(xr, yd))
                    - (at(xl, yu) + 2.0 * at(x, yu) + at(xr, yu));
                let mag = (gx * gx + gy * gy).sqrt().clamp(0.0, 255.0);
                buf.data[base + y * w + x] = (255.0 - mag).round() as u8;
            }
        }
    }
    Ok(())
}

pub fn solarize(buf: &mut PixelBuffer) -> Result<(), FilterError> {
    validate(buf)?;
    let n = buf.pixel_count();
    let planes = (buf.channels as usize).min(3);
    for c in 0..planes {
        let base = c * n;
        for v in &mut buf.data[base..base + n] {
            if *v >= 128 {
                *v = 255 - *v;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn planar(width: u32, height: u32, channels: u8, planes: &[Vec<u8>]) -> PixelBuffer {
        let mut data = Vec::new();
        for p in planes {
            data.extend_from_slice(p);
        }
        PixelBuffer {
            width,
            height,
            channels,
            data,
        }
    }

    fn gray_row(values: &[u8]) -> PixelBuffer {
        planar(
            values.len() as u32,
            1,
            3,
            &[values.to_vec(), values.to_vec(), values.to_vec()],
        )
    }

    fn gray_at(buf: &PixelBuffer, x: usize, y: usize) -> u8 {
        let w = buf.width as usize;
        let n = buf.pixel_count();
        let i = y * w + x;
        assert_eq!(buf.data[i], buf.data[n + i], "R != G at {x},{y}");
        assert_eq!(buf.data[i], buf.data[2 * n + i], "R != B at {x},{y}");
        buf.data[i]
    }

    fn alpha_plane(buf: &PixelBuffer) -> Vec<u8> {
        let n = buf.pixel_count();
        buf.data[3 * n..4 * n].to_vec()
    }

    #[test]
    fn emboss_flat_field_is_neutral_gray() {
        let n = 5 * 4;
        let base = planar(
            5,
            4,
            4,
            &[vec![40; n], vec![90; n], vec![160; n], vec![37; n]],
        );
        let mut out = base.clone();
        emboss(&mut out, 135.0, 3.0, 100.0).unwrap();
        for i in 0..n {
            for c in 0..3 {
                let v = out.data[c * n + i] as i32;
                assert!((v - 128).abs() <= 1, "channel {c} pixel {i} = {v}");
            }
            assert_eq!(out.data[i], out.data[n + i], "not achromatic");
            assert_eq!(out.data[i], out.data[2 * n + i], "not achromatic");
        }
        assert_eq!(alpha_plane(&out), base.data[3 * n..4 * n].to_vec());
    }

    #[test]
    fn emboss_relief_flips_sign_across_a_step_edge() {
        let base = gray_row(&[50, 50, 50, 50, 90, 90, 90, 90]);
        let mut out = base.clone();
        emboss(&mut out, 0.0, 1.0, 100.0).unwrap();
        let dark = gray_at(&out, 3, 0) as i32;
        let light = gray_at(&out, 4, 0) as i32;
        assert!(dark < 128, "dark side of the step should be shadowed");
        assert!(light > 128, "light side of the step should be highlighted");
        assert_eq!(128 - dark, light - 128, "relief should be symmetric");
    }

    #[test]
    fn emboss_relief_is_directional() {
        let base = gray_row(&[50, 50, 50, 50, 90, 90, 90, 90]);
        let mut horizontal = base.clone();
        emboss(&mut horizontal, 0.0, 1.0, 100.0).unwrap();
        let mut vertical = base.clone();
        emboss(&mut vertical, 90.0, 1.0, 100.0).unwrap();
        let n = base.pixel_count();
        assert!(
            vertical.data[..n].iter().all(|&v| v == 128),
            "a horizontal step has no vertical relief"
        );
        assert!(gray_at(&horizontal, 3, 0) < 128);
        assert!(gray_at(&horizontal, 4, 0) > 128);
    }

    #[test]
    fn emboss_output_is_achromatic_on_colored_input() {
        let n = 16;
        let mut r = vec![200u8; n];
        let mut g = vec![30u8; n];
        let mut b = vec![60u8; n];
        for i in 8..n {
            r[i] = 20;
            g[i] = 180;
            b[i] = 220;
        }
        let mut buf = planar(16, 1, 3, &[r, g, b]);
        emboss(&mut buf, 0.0, 2.0, 150.0).unwrap();
        for i in 0..n {
            assert_eq!(buf.data[i], buf.data[n + i], "R != G at {i}");
            assert_eq!(buf.data[i], buf.data[2 * n + i], "R != B at {i}");
        }
    }

    #[test]
    fn emboss_rejects_invalid_parameters() {
        let base = gray_row(&[10, 20, 30, 40]);
        for (angle, height, amount) in [
            (400.0, 1.0, 100.0),
            (f64::NAN, 1.0, 100.0),
            (0.0, 0.0, 100.0),
            (0.0, -1.0, 100.0),
            (0.0, f64::INFINITY, 100.0),
            (0.0, 1.0, 0.0),
            (0.0, 1.0, -5.0),
            (0.0, 1.0, f64::NAN),
        ] {
            let mut out = base.clone();
            assert!(
                emboss(&mut out, angle, height, amount).is_err(),
                "expected reject for angle={angle} height={height} amount={amount}"
            );
            assert_eq!(out, base, "rejected parameters must not modify the buffer");
        }
        let mut ok = base.clone();
        assert!(emboss(&mut ok, 360.0, 1.0, 100.0).is_ok());
    }

    #[test]
    fn find_edges_is_light_on_a_flat_field() {
        let base = gray_row(&[77; 8]);
        let mut out = base.clone();
        find_edges(&mut out).unwrap();
        assert!(out.data[..base.pixel_count()].iter().all(|&v| v == 255));
    }

    #[test]
    fn find_edges_is_dark_at_a_step_edge() {
        let base = gray_row(&[0, 0, 0, 0, 255, 255, 255, 255]);
        let mut out = base.clone();
        find_edges(&mut out).unwrap();
        assert!(out.data[3] < 128, "edge column 3 = {}", out.data[3]);
        assert!(out.data[4] < 128, "edge column 4 = {}", out.data[4]);
        assert_eq!(out.data[0], 255, "flat area must stay light");
        assert_eq!(out.data[7], 255, "flat area must stay light");
    }

    #[test]
    fn solarize_known_values() {
        // FILT-050 fixed 50% curve: 255 >= 128 -> 255 - 255 = 0 (the task
        // brief's "255 -> 255" conflicts with the frozen curve; the curve wins).
        let values = [0u8, 100, 127, 128, 200, 254, 255];
        let expected = [0u8, 100, 127, 127, 55, 1, 0];
        let base = gray_row(&values);
        let mut out = base.clone();
        solarize(&mut out).unwrap();
        let n = values.len();
        for (i, &e) in expected.iter().enumerate() {
            assert_eq!(out.data[i], e, "solarize({})", values[i]);
            assert_eq!(out.data[n + i], e, "solarize({})", values[i]);
            assert_eq!(out.data[2 * n + i], e, "solarize({})", values[i]);
        }
    }

    #[test]
    fn solarize_is_idempotent_not_involutive() {
        let values: Vec<u8> = (0..=255).collect();
        let base = gray_row(&values);
        let mut once = base.clone();
        solarize(&mut once).unwrap();
        let mut twice = once.clone();
        solarize(&mut twice).unwrap();
        assert_ne!(once.data, base.data, "solarize must change the ramp");
        assert_eq!(twice.data, once.data, "solarize is idempotent");
        assert_eq!(once.data[200], 55);
        assert_eq!(twice.data[200], 55);
    }

    #[test]
    fn stylize_preserves_alpha() {
        let expected: Vec<u8> = (0..20u32).map(|i| (i * 11) as u8).collect();
        let n = expected.len();
        let r: Vec<u8> = (0..n).map(|i| (i * 13) as u8).collect();
        let g: Vec<u8> = (0..n).map(|i| (i * 17) as u8).collect();
        let b: Vec<u8> = (0..n).map(|i| (i * 19) as u8).collect();
        let base = planar(5, 4, 4, &[r, g, b, expected.clone()]);

        let mut e = base.clone();
        emboss(&mut e, 45.0, 2.0, 120.0).unwrap();
        assert_eq!(alpha_plane(&e), expected);

        let mut f = base.clone();
        find_edges(&mut f).unwrap();
        assert_eq!(alpha_plane(&f), expected);

        let mut s = base.clone();
        solarize(&mut s).unwrap();
        assert_eq!(alpha_plane(&s), expected);
    }

    #[test]
    fn tiny_images_do_not_panic() {
        for (w, h) in [(1u32, 1u32), (1, 5), (5, 1)] {
            let n = (w * h) as usize;
            let base = planar(
                w,
                h,
                4,
                &[vec![10; n], vec![20; n], vec![30; n], vec![40; n]],
            );
            let mut e = base.clone();
            assert!(emboss(&mut e, 90.0, 1.0, 100.0).is_ok());
            let mut f = base.clone();
            assert!(find_edges(&mut f).is_ok());
            let mut s = base.clone();
            assert!(solarize(&mut s).is_ok());
        }
    }
}
