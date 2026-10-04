//! Stylize family: Emboss, Find Edges, Solarize (`FILT-050`). Implemented by
//! the M7-B task.

use pictura_core::PixelBuffer;

use crate::kernel::{clamp_index, gaussian_blur_planes};
use crate::luma::luma;
use crate::other::maximum;
use crate::{validate, FilterError};

mod extrude;
mod tiles;
mod trace_contour;
mod wind;

pub use extrude::extrude;
pub use tiles::tiles;
pub use trace_contour::trace_contour;
pub use wind::wind;

/// CS6's four Diffuse modes, in the order its dialog lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiffuseMode {
    #[default]
    Normal,
    DarkenOnly,
    LightenOnly,
    Anisotropic,
}

impl DiffuseMode {
    pub fn from_i32(value: i32) -> DiffuseMode {
        match value {
            1 => DiffuseMode::DarkenOnly,
            2 => DiffuseMode::LightenOnly,
            3 => DiffuseMode::Anisotropic,
            _ => DiffuseMode::Normal,
        }
    }
}

/// How many passes Anisotropic makes, and so how far a pixel there reaches.
pub const ANISOTROPIC_REACH: u32 = 4;

/// The eight neighbours of a pixel, in no order that matters.
const AROUND: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

/// Filter ▸ Stylize ▸ Diffuse: shuffle each pixel with one of its neighbours.
///
/// ponytail: the neighbour is picked from the pixel's own coordinates rather
/// than re-rolled per apply, so a preview, the commit and an undo/redo replay
/// agree. CS6 re-rolls; add a seed to the Filter variant if that matters.
pub fn diffuse(buf: &mut PixelBuffer, mode: DiffuseMode) -> Result<(), FilterError> {
    let n = validate(buf)?;
    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);

    if mode == DiffuseMode::Anisotropic {
        // Diffusion is iterative: each pass reads what the last one left.
        for _ in 0..ANISOTROPIC_REACH {
            let src = buf.data.clone();
            for y in 0..h {
                for x in 0..w {
                    let colour = along_the_edge(&src, n, x, y, w, h);
                    let i = y * w + x;
                    for (c, &value) in colour.iter().enumerate().take(planes) {
                        buf.data[c * n + i] = value.round().clamp(0.0, 255.0) as u8;
                    }
                }
            }
        }
        return Ok(());
    }

    // Every output pixel reads its neighbours as they *were*.
    let src = buf.data.clone();
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let here = [src[i] as f64, src[n + i] as f64, src[2 * n + i] as f64];
            let (dx, dy) = AROUND[(hash(x as u32, y as u32) % 8) as usize];
            let tx = clamp_index(x as isize + dx as isize, w);
            let ty = clamp_index(y as isize + dy as isize, h);
            let j = ty * w + tx;
            let there = [src[j] as f64, src[n + j] as f64, src[2 * n + j] as f64];
            let take = match mode {
                DiffuseMode::Normal => true,
                DiffuseMode::DarkenOnly => {
                    luma(there[0], there[1], there[2]) < luma(here[0], here[1], here[2])
                }
                DiffuseMode::LightenOnly => {
                    luma(there[0], there[1], there[2]) > luma(here[0], here[1], here[2])
                }
                DiffuseMode::Anisotropic => unreachable!(),
            };
            for c in 0..planes {
                buf.data[c * n + i] = if take { there[c] as u8 } else { here[c] as u8 };
            }
        }
    }
    Ok(())
}

/// One step of diffusion that flows along an edge rather than across it: each
/// neighbour is weighted by how much it differs from this pixel.
fn along_the_edge(src: &[u8], n: usize, x: usize, y: usize, w: usize, h: usize) -> [f64; 3] {
    const EDGE: f64 = 28.0;
    let p = y * w + x;
    let here = [src[p] as f64, src[n + p] as f64, src[2 * n + p] as f64];
    let mut total = 1.0f64;
    let mut sum = here;
    for (dx, dy) in AROUND {
        let tx = clamp_index(x as isize + dx as isize, w);
        let ty = clamp_index(y as isize + dy as isize, h);
        let q = ty * w + tx;
        let there = [src[q] as f64, src[n + q] as f64, src[2 * n + q] as f64];
        let difference = ((there[0] - here[0]).powi(2)
            + (there[1] - here[1]).powi(2)
            + (there[2] - here[2]).powi(2))
        .sqrt();
        let weight = (-(difference / EDGE).powi(2)).exp();
        for c in 0..3 {
            sum[c] += there[c] * weight;
        }
        total += weight;
    }
    [sum[0] / total, sum[1] / total, sum[2] / total]
}

/// Which neighbour a pixel reaches for, seeded from where the pixel is.
fn hash(x: u32, y: u32) -> u32 {
    let mut h = x.wrapping_mul(0x27d4_eb2d) ^ y.wrapping_mul(0x1656_67b1);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2545_f491);
    h ^= h >> 13;
    h
}

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
    // 255 - clamp(magnitude). Fitting the exact detector is out of scope.
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

/// CS6's ranges for Glowing Edges, which its three sliders run over.
pub const GLOW_WIDTH: std::ops::RangeInclusive<u32> = 1..=14;
pub const GLOW_BRIGHTNESS: std::ops::RangeInclusive<u32> = 0..=20;
pub const GLOW_SMOOTHNESS: std::ops::RangeInclusive<u32> = 1..=15;

/// How much Gaussian each step of Smoothness is worth, in pixels of radius.
const GLOW_SMOOTHING: f32 = 0.3;

/// How far a pixel of Glowing Edges reads: three sigma of the smoothing blur,
/// one more for the Sobel taken on top, and half the edge width.
pub fn glow_reach(width: u32, smoothness: u32) -> u32 {
    let smoothness = smoothness.clamp(*GLOW_SMOOTHNESS.start(), *GLOW_SMOOTHNESS.end());
    let width = width.clamp(*GLOW_WIDTH.start(), *GLOW_WIDTH.end());
    (smoothness as f32 * GLOW_SMOOTHING * 3.0).ceil() as u32 + 1 + width / 2
}

/// Glowing Edges: the picture's edges lit up on a black ground. Smoothness
/// blurs before the gradient is taken, Edge Width dilates the line, and Edge
/// Brightness is the gain on the result. The gradient runs channel by channel,
/// which is where the colour of an edge comes from.
pub fn glowing_edges(
    buf: &mut PixelBuffer,
    width: u32,
    brightness: u32,
    smoothness: u32,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if !GLOW_WIDTH.contains(&width)
        || !GLOW_BRIGHTNESS.contains(&brightness)
        || !GLOW_SMOOTHNESS.contains(&smoothness)
    {
        return Err(FilterError::InvalidParams(format!(
            "glowing edges width {width} brightness {brightness} smoothness {smoothness} out of range"
        )));
    }

    let mut smoothed = buf.clone();
    gaussian_blur_planes(&mut smoothed, smoothness as f64 * GLOW_SMOOTHING as f64);
    // ponytail: skip photorust's premultiply, so a soft-alpha edge can ring
    // (the Sobel reads the hidden colour of transparent pixels). Add a
    // premultiply pass if that ring ever shows.
    let mut edges = gradient(&smoothed);
    if width / 2 > 0 {
        maximum(&mut edges, width / 2)?;
    }

    let gain = brightness as f64 / 5.0;
    let planes = (buf.channels as usize).min(3);
    for i in 0..n {
        for c in 0..planes {
            buf.data[c * n + i] = (edges.data[c * n + i] as f64 * gain).min(255.0) as u8;
        }
    }
    Ok(())
}

/// The raw Sobel magnitude of each colour channel, as a 3-channel buffer.
fn gradient(source: &PixelBuffer) -> PixelBuffer {
    let w = source.width as usize;
    let h = source.height as usize;
    let n = w * h;
    let mut out = vec![0u8; n * 3];
    for y in 0..h {
        for x in 0..w {
            let xl = clamp_index(x as isize - 1, w);
            let xr = clamp_index(x as isize + 1, w);
            let yu = clamp_index(y as isize - 1, h);
            let yd = clamp_index(y as isize + 1, h);
            let i = y * w + x;
            for c in 0..3 {
                let base = c * n;
                let at = |sx: usize, sy: usize| source.data[base + sy * w + sx] as f64;
                let gx = -at(xl, yu) - 2.0 * at(xl, y) - at(xl, yd)
                    + at(xr, yu)
                    + 2.0 * at(xr, y)
                    + at(xr, yd);
                let gy = -at(xl, yu) - 2.0 * at(x, yu) - at(xr, yu)
                    + at(xl, yd)
                    + 2.0 * at(x, yd)
                    + at(xr, yd);
                out[c * n + i] = (gx * gx + gy * gy).sqrt().min(255.0) as u8;
            }
        }
    }
    PixelBuffer {
        width: source.width,
        height: source.height,
        channels: 3,
        data: out.into(),
    }
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
            data: data.into(),
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

    fn patch(w: u32, h: u32) -> PixelBuffer {
        let n = (w * h) as usize;
        let mut data = vec![0u8; n * 4];
        for y in 0..h as usize {
            for x in 0..w as usize {
                let p = y * w as usize + x;
                data[p] = (x * 37 + y * 11) as u8;
                data[n + p] = (x * 5 + y * 61) as u8;
                data[2 * n + p] = (x * 29 + y * 3) as u8;
                data[3 * n + p] = 200;
            }
        }
        PixelBuffer {
            width: w,
            height: h,
            channels: 4,
            data: data.into(),
        }
    }

    #[test]
    fn diffuse_modes_change_colour_and_preserve_alpha() {
        for mode in [
            DiffuseMode::Normal,
            DiffuseMode::DarkenOnly,
            DiffuseMode::LightenOnly,
            DiffuseMode::Anisotropic,
        ] {
            let base = patch(16, 16);
            let before = base.data.clone();
            let mut out = base.clone();
            diffuse(&mut out, mode).unwrap();
            let n = out.pixel_count();
            assert_ne!(out.data, before, "mode {mode:?} made no change");
            assert_eq!(
                out.data[3 * n..],
                before[3 * n..],
                "mode {mode:?} touched alpha"
            );
        }
    }

    #[test]
    fn diffuse_is_deterministic() {
        let base = patch(16, 16);
        let mut a = base.clone();
        let mut b = base.clone();
        diffuse(&mut a, DiffuseMode::Normal).unwrap();
        diffuse(&mut b, DiffuseMode::Normal).unwrap();
        assert_eq!(a.data, b.data);
    }

    #[test]
    fn glowing_edges_changes_colour_and_preserves_alpha() {
        let base = patch(24, 24);
        let before = base.data.clone();
        let mut out = base.clone();
        glowing_edges(&mut out, 2, 6, 1).unwrap();
        let n = out.pixel_count();
        assert_ne!(out.data, before);
        assert_eq!(out.data[3 * n..], before[3 * n..], "alpha untouched");
    }

    #[test]
    fn glowing_edges_rejects_out_of_range_params_without_mutation() {
        let base = patch(16, 16);
        for (w, b, s) in [
            (0u32, 6u32, 1u32),
            (15, 6, 1),
            (2, 21, 1),
            (2, 6, 0),
            (2, 6, 16),
        ] {
            let mut out = base.clone();
            assert!(
                matches!(
                    glowing_edges(&mut out, w, b, s),
                    Err(FilterError::InvalidParams(_))
                ),
                "expected rejection for width={w} brightness={b} smoothness={s}"
            );
            assert_eq!(out, base, "rejected parameters must not modify the buffer");
        }
    }
}
