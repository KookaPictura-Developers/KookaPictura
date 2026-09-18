use pictura_core::PixelBuffer;

use crate::artistic::{noise, reduce};
use crate::kernel::clamp_index;
use crate::luma::luma;
use crate::{validate, FilterError};

use super::common::box_sum;

/// Cutout: quantize every colour plane to `levels` bands, then flatten contours
/// with a box mean that grows with `edge_simplicity` and shrinks with
/// `edge_fidelity`.
///
/// ponytail: the box mean followed by a second posterize is a contour smoother,
/// not Adobe's edge-guided region merge (closed). Good enough for flat bands.
pub fn cutout(
    buf: &mut PixelBuffer,
    levels: u8,
    edge_simplicity: u8,
    edge_fidelity: u8,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if !(2..=8).contains(&levels) {
        return Err(FilterError::InvalidParams(format!(
            "cutout levels {levels} is outside 2..=8"
        )));
    }
    if edge_simplicity > 10 {
        return Err(FilterError::InvalidParams(format!(
            "cutout edge simplicity {edge_simplicity} is outside 0..=10"
        )));
    }
    if !(1..=3).contains(&edge_fidelity) {
        return Err(FilterError::InvalidParams(format!(
            "cutout edge fidelity {edge_fidelity} is outside 1..=3"
        )));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);

    for c in 0..planes {
        let base = c * n;
        for v in &mut buf.data[base..base + n] {
            *v = reduce::posterize(*v, levels);
        }
    }

    let radius = edge_simplicity as usize * (4 - edge_fidelity as usize) / 4;
    if radius > 0 {
        let area = ((2 * radius + 1) * (2 * radius + 1)) as f64;
        for c in 0..planes {
            let base = c * n;
            let src: Vec<f64> = buf.data[base..base + n].iter().map(|&v| v as f64).collect();
            let sums = box_sum(&src, w, h, radius);
            for (i, &s) in sums.iter().enumerate() {
                buf.data[base + i] = reduce::posterize(reduce::clamp_u8(s / area), levels);
            }
        }
    }
    Ok(())
}

/// Film Grain: seeded per-pixel perturbation, stronger in shadows/midtones and
/// rolled off in highlights. `grain` 0 is an exact no-op.
///
/// ponytail: monochromatic white-noise grain; Adobe's multi-scale grain
/// clumping is closed. An interpolated field is the upgrade path.
pub fn film_grain(
    buf: &mut PixelBuffer,
    grain: u8,
    highlight_area: u8,
    intensity: u8,
    seed: u64,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if grain > 20 {
        return Err(FilterError::InvalidParams(format!(
            "film grain {grain} is outside 0..=20"
        )));
    }
    if highlight_area > 20 {
        return Err(FilterError::InvalidParams(format!(
            "film grain highlight area {highlight_area} is outside 0..=20"
        )));
    }
    if intensity > 10 {
        return Err(FilterError::InvalidParams(format!(
            "film grain intensity {intensity} is outside 0..=10"
        )));
    }
    if grain == 0 {
        return Ok(());
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let field = noise::value_noise(w, h, seed);
    let amp = grain as f64;
    let contrast = 0.5 + intensity as f64 / 10.0;
    let rolloff = 1.0 + highlight_area as f64 * 0.3;

    for (i, &f) in field.iter().enumerate() {
        let l = luma(
            buf.data[i] as f64,
            buf.data[n + i] as f64,
            buf.data[2 * n + i] as f64,
        );
        let t = (l / 255.0).clamp(0.0, 1.0);
        let weight = (-t * rolloff).exp();
        let delta = (f * 2.0 - 1.0) * amp * weight * contrast;
        for c in 0..planes {
            buf.data[c * n + i] = reduce::clamp_u8(buf.data[c * n + i] as f64 + delta);
        }
    }
    Ok(())
}

/// Neon Glow: tinted additive glow of a blurred luminance field. `glow_size > 0`
/// glows the highlights, `glow_size < 0` confines the glow to shadows.
///
/// ponytail: highlight selection is a hard threshold and the glow is a box sum
/// (bloom-like accumulation), not Adobe's soft rolloff / Gaussian halo.
pub fn neon_glow(
    buf: &mut PixelBuffer,
    glow_size: i32,
    glow_brightness: u8,
    glow_color: [u8; 3],
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if !(-24..=24).contains(&glow_size) {
        return Err(FilterError::InvalidParams(format!(
            "neon glow size {glow_size} is outside -24..=24"
        )));
    }
    if glow_brightness > 50 {
        return Err(FilterError::InvalidParams(format!(
            "neon glow brightness {glow_brightness} is outside 0..=50"
        )));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;

    let mut field = vec![0.0f64; n];
    for (i, f) in field.iter_mut().enumerate() {
        let l = luma(
            buf.data[i] as f64,
            buf.data[n + i] as f64,
            buf.data[2 * n + i] as f64,
        );
        *f = if glow_size >= 0 {
            ((l - 128.0) / 127.0).clamp(0.0, 1.0)
        } else {
            ((128.0 - l) / 127.0).clamp(0.0, 1.0)
        };
    }

    let radius = glow_size.unsigned_abs() as usize;
    if radius > 0 {
        let sums = box_sum(&field, w, h, radius);
        for (f, s) in field.iter_mut().zip(sums) {
            *f = s.min(1.0);
        }
    }

    let gain = glow_brightness as f64 / 50.0;
    for (i, &f) in field.iter().enumerate() {
        let amount = f * gain;
        for (c, &color) in glow_color.iter().enumerate() {
            buf.data[c * n + i] =
                reduce::clamp_u8(buf.data[c * n + i] as f64 + amount * color as f64);
        }
    }
    Ok(())
}

/// Poster Edges: posterize to `posterization + 1` bands, then darken pixels on
/// a Sobel edge, widened by a dilation of radius `edge_thickness`.
pub fn poster_edges(
    buf: &mut PixelBuffer,
    edge_thickness: u8,
    edge_intensity: u8,
    posterization: u8,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if edge_thickness > 10 {
        return Err(FilterError::InvalidParams(format!(
            "poster edges thickness {edge_thickness} is outside 0..=10"
        )));
    }
    if edge_intensity > 10 {
        return Err(FilterError::InvalidParams(format!(
            "poster edges intensity {edge_intensity} is outside 0..=10"
        )));
    }
    if posterization > 10 {
        return Err(FilterError::InvalidParams(format!(
            "poster edges posterization {posterization} is outside 0..=10"
        )));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
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

    let levels = posterization + 1;
    for c in 0..planes {
        let base = c * n;
        for v in &mut buf.data[base..base + n] {
            *v = reduce::posterize(*v, levels);
        }
    }

    let threshold = 200.0 - edge_intensity as f64 * 15.0;
    let luma_at =
        |ax: isize, ay: isize| -> f64 { lum[clamp_index(ay, h) * w + clamp_index(ax, w)] };

    let mut edge = vec![false; n];
    for y in 0..h {
        for x in 0..w {
            if reduce::edge_magnitude(luma_at, x, y) > threshold {
                edge[y * w + x] = true;
            }
        }
    }

    let radius = edge_thickness as usize;
    if radius > 0 {
        let mut dilated = vec![false; n];
        for y in 0..h {
            for x in 0..w {
                if !edge[y * w + x] {
                    continue;
                }
                for dy in 0..=2 * radius {
                    let ny = clamp_index(y as isize + dy as isize - radius as isize, h);
                    for dx in 0..=2 * radius {
                        let nx = clamp_index(x as isize + dx as isize - radius as isize, w);
                        dilated[ny * w + nx] = true;
                    }
                }
            }
        }
        edge = dilated;
    }

    for (i, &is_edge) in edge.iter().enumerate() {
        if is_edge {
            for c in 0..planes {
                buf.data[c * n + i] = 0;
            }
        }
    }
    Ok(())
}
