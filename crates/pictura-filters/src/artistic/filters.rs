//! Artistic filters (`m22-artistic-filters`): Cutout, Film Grain, Neon Glow and
//! Poster Edges. Behavioural models; Adobe's closed algorithms are approximated
//! and each shortcut is marked with a `ponytail:` note.

use pictura_core::PixelBuffer;
use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaCha8Rng,
};

use crate::kernel::clamp_index;
use crate::luma::luma;
use crate::{validate, BrushType, FilterError, TextureOptions, TextureSurface};

use super::{noise, reduce, texture};

/// One uniform `f64` in `[0, 1)` from 53 random bits.
fn unit_f64(rng: &mut ChaCha8Rng) -> f64 {
    (rng.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
}

/// Separable box sum over `(2*radius+1)^2` with clamp-to-edge. Not normalized:
/// callers divide by the window area when they want a mean.
fn box_sum(src: &[f64], w: usize, h: usize, radius: usize) -> Vec<f64> {
    if radius == 0 {
        return src.to_vec();
    }
    let mut tmp = vec![0.0f64; src.len()];
    for y in 0..h {
        for x in 0..w {
            let mut s = 0.0;
            for d in 0..=2 * radius {
                let sx = clamp_index(x as isize + d as isize - radius as isize, w);
                s += src[y * w + sx];
            }
            tmp[y * w + x] = s;
        }
    }
    let mut out = vec![0.0f64; src.len()];
    for y in 0..h {
        for x in 0..w {
            let mut s = 0.0;
            for d in 0..=2 * radius {
                let sy = clamp_index(y as isize + d as isize - radius as isize, h);
                s += tmp[sy * w + x];
            }
            out[y * w + x] = s;
        }
    }
    out
}

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

/// One oriented, soft-edged daub stamped onto a planar buffer. `aspect` is the
/// cross-stroke to along-stroke radius ratio, `power` the edge crispness
/// (larger = harder), and `roughness` jitters coverage with `jitter_field`.
#[allow(clippy::too_many_arguments)]
fn paint_daub(
    dst: &mut [u8],
    w: usize,
    h: usize,
    planes: usize,
    center: (f64, f64),
    radius: f64,
    aspect: f64,
    angle: f64,
    color: [f64; 3],
    power: f64,
    roughness: f64,
    jitter_field: &[f64],
) {
    if radius <= 0.0 {
        return;
    }
    let n = w * h;
    let (cx, cy) = center;
    let rx = radius;
    let ry = (radius * aspect).max(0.5);
    let (sin, cos) = angle.sin_cos();
    let reach = rx.max(ry).ceil() as isize + 1;
    let x0 = (cx as isize - reach).clamp(0, w as isize - 1) as usize;
    let x1 = (cx as isize + reach).clamp(0, w as isize - 1) as usize;
    let y0 = (cy as isize - reach).clamp(0, h as isize - 1) as usize;
    let y1 = (cy as isize + reach).clamp(0, h as isize - 1) as usize;
    for py in y0..=y1 {
        for px in x0..=x1 {
            let dx = px as f64 + 0.5 - cx;
            let dy = py as f64 + 0.5 - cy;
            let u = (dx * cos + dy * sin) / rx;
            let v = (-dx * sin + dy * cos) / ry;
            let d = (u * u + v * v).sqrt();
            if d >= 1.0 {
                continue;
            }
            let mut cov = (1.0 - d).powf(power);
            if roughness > 0.0 {
                cov = (cov + (jitter_field[py * w + px] - 0.5) * roughness).clamp(0.0, 1.0);
            }
            let idx = py * w + px;
            for c in 0..planes {
                let o = dst[c * n + idx] as f64;
                dst[c * n + idx] = reduce::clamp_u8(o + (color[c] - o) * cov);
            }
        }
    }
}

/// Paint Daubs: oriented daubs on a `brush_size`-related grid, each filled with
/// the source colour at its centre. `sharpness` hardens the daub edge and the
/// six [`BrushType`]s vary shape, hardness and tonal skew.
///
/// ponytail: one daub per grid cell with a hard elliptical falloff; Adobe's
/// per-type bristle footprints are closed. An oriented multi-lobe stamp is the
/// upgrade path if the single ellipse reads too uniform.
pub fn paint_daubs(
    buf: &mut PixelBuffer,
    brush_size: u8,
    sharpness: u8,
    brush_type: BrushType,
    seed: u64,
) -> Result<(), FilterError> {
    validate(buf)?;
    if !(1..=50).contains(&brush_size) {
        return Err(FilterError::InvalidParams(format!(
            "paint daubs brush size {brush_size} is outside 1..=50"
        )));
    }
    if sharpness > 40 {
        return Err(FilterError::InvalidParams(format!(
            "paint daubs sharpness {sharpness} is outside 0..=40"
        )));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let n = w * h;
    let planes = (buf.channels as usize).min(3);
    let src = buf.data.clone();
    let jitter_field = noise::value_noise(w, h, seed);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    let radius = brush_size as f64 * 0.6 + 0.5;
    let spacing = (brush_size as usize / 2).max(1);
    let power = 1.0 + sharpness as f64 / 10.0;
    let (aspect, power_mul, roughness, tint, radius_scale, density) = match brush_type {
        BrushType::Simple => (1.0, 1.0, 0.0, 0.0, 1.0, 1.0),
        BrushType::LightRough => (1.0, 1.0, 0.4, 16.0, 1.0, 1.0),
        BrushType::DarkRough => (1.0, 1.0, 0.4, -16.0, 1.0, 1.0),
        BrushType::WideSharp => (0.5, 1.6, 0.0, 0.0, 1.2, 1.0),
        BrushType::WideBlurry => (0.5, 0.5, 0.0, 0.0, 1.2, 1.0),
        BrushType::Sparkle => (1.0, 2.0, 0.0, 40.0, 0.25, 0.6),
    };

    for cy in (0..h).step_by(spacing) {
        for cx in (0..w).step_by(spacing) {
            let jx = unit_f64(&mut rng) * spacing as f64;
            let jy = unit_f64(&mut rng) * spacing as f64;
            let angle = unit_f64(&mut rng) * std::f64::consts::PI;
            if unit_f64(&mut rng) > density {
                continue;
            }
            let center = (
                (cx as f64 + jx).clamp(0.0, (w - 1) as f64),
                (cy as f64 + jy).clamp(0.0, (h - 1) as f64),
            );
            let sx = clamp_index(center.0 as isize, w);
            let sy = clamp_index(center.1 as isize, h);
            let idx = sy * w + sx;
            let mut color = [0.0f64; 3];
            for (c, col) in color.iter_mut().enumerate() {
                *col = (src[c * n + idx] as f64 + tint).clamp(0.0, 255.0);
            }
            paint_daub(
                &mut buf.data,
                w,
                h,
                planes,
                center,
                radius * radius_scale,
                aspect,
                angle,
                color,
                power * power_mul,
                roughness,
                &jitter_field,
            );
        }
    }
    Ok(())
}

/// Coarse per-pixel stroke directions: one seeded angle per `cell`-sized block.
fn stroke_angles(w: usize, h: usize, cell: usize, seed: u64) -> Vec<f64> {
    let cell = cell.max(1);
    let cw = w.div_ceil(cell).max(1);
    let ch = h.div_ceil(cell).max(1);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let angles: Vec<f64> = (0..cw * ch)
        .map(|_| unit_f64(&mut rng) * std::f64::consts::PI)
        .collect();
    let mut out = vec![0.0f64; w * h];
    for y in 0..h {
        let cy = (y / cell).min(ch - 1);
        for x in 0..w {
            let cx = (x / cell).min(cw - 1);
            out[y * w + x] = angles[cy * cw + cx];
        }
    }
    out
}

/// Palette Knife: pull each sample toward the source colour a short distance
/// along a seeded stroke direction, repeated `stroke_detail` times and mixed by
/// `softness`.
///
/// ponytail: a directional neighbour pull, not Adobe's iterative paint-load
/// smear. The coarse angle block keeps strokes coherent; a carried running
/// colour is the upgrade path.
pub fn palette_knife(
    buf: &mut PixelBuffer,
    stroke_size: u8,
    stroke_detail: u8,
    softness: u8,
    seed: u64,
) -> Result<(), FilterError> {
    validate(buf)?;
    if !(1..=50).contains(&stroke_size) {
        return Err(FilterError::InvalidParams(format!(
            "palette knife stroke size {stroke_size} is outside 1..=50"
        )));
    }
    if !(1..=3).contains(&stroke_detail) {
        return Err(FilterError::InvalidParams(format!(
            "palette knife stroke detail {stroke_detail} is outside 1..=3"
        )));
    }
    if softness > 10 {
        return Err(FilterError::InvalidParams(format!(
            "palette knife softness {softness} is outside 0..=10"
        )));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let n = w * h;
    let planes = (buf.channels as usize).min(3);
    let len = stroke_size as f64;
    let blend = softness as f64 / 10.0 * 0.7;

    for it in 0..stroke_detail as u64 {
        let src = buf.data.clone();
        let angles = stroke_angles(
            w,
            h,
            stroke_size as usize,
            seed ^ (it + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15),
        );
        let offsets = noise::value_noise(
            w,
            h,
            seed.wrapping_add(it).wrapping_add(0x517C_C1B7_2722_0A95),
        );
        for y in 0..h {
            for x in 0..w {
                let p = y * w + x;
                let angle = angles[p];
                let (sin, cos) = angle.sin_cos();
                let t = (offsets[p] - 0.5) * len;
                let sx = clamp_index(x as isize + (t * cos).round() as isize, w);
                let sy = clamp_index(y as isize + (t * sin).round() as isize, h);
                for c in 0..planes {
                    let base = c * n;
                    let cur = src[base + p] as f64;
                    let samp = src[base + sy * w + sx] as f64;
                    buf.data[base + p] = reduce::clamp_u8(cur + (samp - cur) * blend);
                }
            }
        }
    }
    Ok(())
}

/// Plastic Wrap: box-smooth with radius `smoothness`, then add a highlight from
/// the smoothed edge magnitude (localized by `detail`) plus a weak specular
/// sheen, scaled by `highlight_strength`.
///
/// ponytail: a Sobel gate and a quadratic sheen, not Adobe's plastic-surface
/// simulation. Add specular lobes if the flat sheen reads too even.
pub fn plastic_wrap(
    buf: &mut PixelBuffer,
    highlight_strength: u8,
    detail: u8,
    smoothness: u8,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if highlight_strength > 20 {
        return Err(FilterError::InvalidParams(format!(
            "plastic wrap highlight strength {highlight_strength} is outside 0..=20"
        )));
    }
    if !(1..=15).contains(&detail) {
        return Err(FilterError::InvalidParams(format!(
            "plastic wrap detail {detail} is outside 1..=15"
        )));
    }
    if !(1..=15).contains(&smoothness) {
        return Err(FilterError::InvalidParams(format!(
            "plastic wrap smoothness {smoothness} is outside 1..=15"
        )));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let radius = smoothness as usize;
    let area = ((2 * radius + 1) * (2 * radius + 1)) as f64;
    for c in 0..planes {
        let base = c * n;
        let field: Vec<f64> = buf.data[base..base + n].iter().map(|&v| v as f64).collect();
        let sums = box_sum(&field, w, h, radius);
        for (i, &s) in sums.iter().enumerate() {
            buf.data[base + i] = reduce::clamp_u8(s / area);
        }
    }

    let lum: Vec<f64> = (0..n)
        .map(|i| {
            luma(
                buf.data[i] as f64,
                buf.data[n + i] as f64,
                buf.data[2 * n + i] as f64,
            )
        })
        .collect();
    let luma_at =
        |ax: isize, ay: isize| -> f64 { lum[clamp_index(ay, h) * w + clamp_index(ax, w)] };
    let threshold = 1200.0 / detail as f64;
    let gain = highlight_strength as f64 / 20.0;

    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let mag = reduce::edge_magnitude(luma_at, x, y);
            let gate = ((mag - threshold).max(0.0) / 900.0).clamp(0.0, 1.0);
            let sheen = (lum[i] / 255.0).powi(2) * 0.25;
            let amount = (gate + sheen).min(1.0) * gain * 180.0;
            for c in 0..planes {
                buf.data[c * n + i] = reduce::clamp_u8(buf.data[c * n + i] as f64 + amount);
            }
        }
    }
    Ok(())
}

/// Sponge: seeded soft daubs that push samples brighter or darker by
/// `definition`, then a `smoothness` box blend.
///
/// ponytail: signed soft discs over per-pixel sponge noise; Adobe's porous
/// texture is closed. A noise-modulated coverage is the upgrade path.
pub fn sponge(
    buf: &mut PixelBuffer,
    brush_size: u8,
    definition: u8,
    smoothness: u8,
    seed: u64,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if brush_size > 10 {
        return Err(FilterError::InvalidParams(format!(
            "sponge brush size {brush_size} is outside 0..=10"
        )));
    }
    if definition > 25 {
        return Err(FilterError::InvalidParams(format!(
            "sponge definition {definition} is outside 0..=25"
        )));
    }
    if !(1..=15).contains(&smoothness) {
        return Err(FilterError::InvalidParams(format!(
            "sponge smoothness {smoothness} is outside 1..=15"
        )));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let src = buf.data.clone();
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let r = brush_size as f64;
    let spacing = ((2.0 * r).max(1.0)) as usize;
    let mag = definition as f64 / 25.0 * 160.0;

    for cy in (0..h).step_by(spacing) {
        for cx in (0..w).step_by(spacing) {
            let jx = unit_f64(&mut rng) * spacing as f64;
            let jy = unit_f64(&mut rng) * spacing as f64;
            let sign = if unit_f64(&mut rng) < 0.5 { -1.0 } else { 1.0 };
            let bias = sign * mag * (0.4 + 0.6 * unit_f64(&mut rng));
            let radius = if r <= 0.0 {
                0.6
            } else {
                (r * (0.5 + 0.8 * unit_f64(&mut rng))).max(0.6)
            };
            let (ccx, ccy) = (cx as f64 + jx, cy as f64 + jy);
            let reach = radius.ceil() as isize;
            let x0 = (ccx as isize - reach).clamp(0, w as isize - 1) as usize;
            let x1 = (ccx as isize + reach).clamp(0, w as isize - 1) as usize;
            let y0 = (ccy as isize - reach).clamp(0, h as isize - 1) as usize;
            let y1 = (ccy as isize + reach).clamp(0, h as isize - 1) as usize;
            for py in y0..=y1 {
                for px in x0..=x1 {
                    let dx = px as f64 + 0.5 - ccx;
                    let dy = py as f64 + 0.5 - ccy;
                    let d = (dx * dx + dy * dy).sqrt();
                    if d >= radius {
                        continue;
                    }
                    let cov = 1.0 - d / radius;
                    let idx = py * w + px;
                    for c in 0..planes {
                        let base = c * n;
                        let o = src[base + idx] as f64;
                        buf.data[base + idx] = reduce::clamp_u8(o + bias * cov);
                    }
                }
            }
        }
    }

    let sr = smoothness as usize;
    let area = ((2 * sr + 1) * (2 * sr + 1)) as f64;
    let mix = smoothness as f64 / 15.0 * 0.6;
    for c in 0..planes {
        let base = c * n;
        let field: Vec<f64> = buf.data[base..base + n].iter().map(|&v| v as f64).collect();
        let means = box_sum(&field, w, h, sr);
        for (i, &m) in means.iter().enumerate() {
            buf.data[base + i] = reduce::clamp_u8(field[i] + (m / area - field[i]) * mix);
        }
    }
    Ok(())
}

/// Shared range check for the Dry Brush / Fresco pair.
fn validate_brush(
    brush_size: u8,
    brush_detail: u8,
    texture: u8,
    name: &str,
) -> Result<(), FilterError> {
    if !(1..=50).contains(&brush_size) {
        return Err(FilterError::InvalidParams(format!(
            "{name} brush size {brush_size} is outside 1..=50"
        )));
    }
    if !(1..=12).contains(&brush_detail) {
        return Err(FilterError::InvalidParams(format!(
            "{name} brush detail {brush_detail} is outside 1..=12"
        )));
    }
    if !(1..=3).contains(&texture) {
        return Err(FilterError::InvalidParams(format!(
            "{name} texture {texture} is outside 1..=3"
        )));
    }
    Ok(())
}

fn surface_for(texture: u8) -> TextureSurface {
    match texture {
        1 => TextureSurface::Canvas,
        2 => TextureSurface::Burlap,
        _ => TextureSurface::Sandstone,
    }
}

/// Add an embossed procedural surface term to every colour plane. `texture`
/// selects the preset and scales the relief.
fn apply_surface(data: &mut [u8], w: usize, h: usize, n: usize, planes: usize, texture: u8) {
    let surface = surface_for(texture);
    let relief = texture * 4;
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let shade = texture::emboss(
                |hx, hy| texture::surface_height(surface, hx, hy, 100),
                x,
                y,
                0,
                relief,
                false,
            );
            let grain = (texture::surface_height(surface, x, y, 100) - 0.5) * 30.0;
            let delta = shade * 6.0 + grain;
            for c in 0..planes {
                data[c * n + i] = reduce::clamp_u8(data[c * n + i] as f64 + delta);
            }
        }
    }
}

/// Colored Pencil: posterize to a small palette, lay down crosshatch strokes
/// tinted by `foreground`, and blend smooth areas toward the `background` paper
/// lightened by `paper_brightness`. The Sobel edge weight keeps the reduced
/// source colour at strong edges, so the tonal step survives.
///
/// ponytail: geometric crosshatch with noise jitter, not Adobe's pressure-
/// modelled pencil; the hatch period is fixed relative to `pencil_width`.
pub fn colored_pencil(
    buf: &mut PixelBuffer,
    pencil_width: u8,
    stroke_pressure: u8,
    paper_brightness: u8,
    foreground: [u8; 3],
    background: [u8; 3],
    seed: u64,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if !(1..=24).contains(&pencil_width) {
        return Err(FilterError::InvalidParams(format!(
            "colored pencil width {pencil_width} is outside 1..=24"
        )));
    }
    if stroke_pressure > 15 {
        return Err(FilterError::InvalidParams(format!(
            "colored pencil stroke pressure {stroke_pressure} is outside 0..=15"
        )));
    }
    if paper_brightness > 50 {
        return Err(FilterError::InvalidParams(format!(
            "colored pencil paper brightness {paper_brightness} is outside 0..=50"
        )));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let src = buf.data.clone();

    let lum: Vec<f64> = (0..n)
        .map(|i| luma(src[i] as f64, src[n + i] as f64, src[2 * n + i] as f64))
        .collect();
    let luma_at =
        |ax: isize, ay: isize| -> f64 { lum[clamp_index(ay, h) * w + clamp_index(ax, w)] };

    let paper: [f64; 3] = std::array::from_fn(|c| {
        let bg = background[c] as f64;
        bg + (255.0 - bg) * (paper_brightness as f64 / 50.0 * 0.7)
    });
    let pressure = stroke_pressure as f64 / 15.0;
    let pw = pencil_width as usize;
    let period = (pw * 3).max(3);
    let jitter = noise::value_noise(w, h, seed);

    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let e = (reduce::edge_magnitude(luma_at, x, y) / 800.0).clamp(0.0, 1.0);
            let on = (x + y) % period < pw || (x + (h - 1 - y)) % period < pw;
            let j = 0.6 + (jitter[i] - 0.5) * 0.3;
            let dark = if on {
                (pressure * (0.4 + 0.6 * e) * j).clamp(0.0, 1.0)
            } else {
                0.0
            };
            for c in 0..planes {
                let reduced = reduce::posterize(src[c * n + i], 5) as f64;
                let base = reduced + (paper[c] - reduced) * (1.0 - e) * 0.7;
                buf.data[c * n + i] = reduce::clamp_u8(base + (foreground[c] as f64 - base) * dark);
            }
        }
    }
    Ok(())
}

/// Dry Brush: posterize to a reduced palette, smear along seeded directional
/// strokes, then modulate by a procedural surface (`texture` 1..=3).
///
/// ponytail: one directional sample per pixel plus an embossed surface; Adobe's
/// bristle-resolved dry stroke is closed.
pub fn dry_brush(
    buf: &mut PixelBuffer,
    brush_size: u8,
    brush_detail: u8,
    texture: u8,
    seed: u64,
) -> Result<(), FilterError> {
    validate(buf)?;
    validate_brush(brush_size, brush_detail, texture, "dry brush")?;

    let w = buf.width as usize;
    let h = buf.height as usize;
    let n = w * h;
    let planes = (buf.channels as usize).min(3);
    let levels = 2 + brush_detail / 2;
    for c in 0..planes {
        let base = c * n;
        for v in &mut buf.data[base..base + n] {
            *v = reduce::posterize(*v, levels);
        }
    }
    let src = buf.data.clone();

    let angles = stroke_angles(w, h, brush_size as usize, seed);
    let flow = noise::value_noise(w, h, seed ^ 0x9E37_79B9_7F4A_7C15);
    let len = brush_size as f64 * 0.5;
    for y in 0..h {
        for x in 0..w {
            let p = y * w + x;
            let (sin, cos) = angles[p].sin_cos();
            let t = (flow[p] - 0.5) * len;
            let sx = clamp_index(x as isize + (t * cos).round() as isize, w);
            let sy = clamp_index(y as isize + (t * sin).round() as isize, h);
            let q = sy * w + sx;
            for c in 0..planes {
                let base = c * n;
                let cur = src[base + p] as f64;
                let samp = src[base + q] as f64;
                buf.data[base + p] = reduce::clamp_u8(cur + (samp - cur) * 0.6);
            }
        }
    }

    apply_surface(&mut buf.data, w, h, n, planes, texture);
    Ok(())
}

/// Fresco: coarse short daubs on a `brush_size` grid over a reduced palette,
/// then embossed by a procedural surface.
///
/// ponytail: reuses the Paint Daubs single-ellipse stamp with a short aspect;
/// Adobe's plaster daub is closed.
pub fn fresco(
    buf: &mut PixelBuffer,
    brush_size: u8,
    brush_detail: u8,
    texture: u8,
    seed: u64,
) -> Result<(), FilterError> {
    validate(buf)?;
    validate_brush(brush_size, brush_detail, texture, "fresco")?;

    let w = buf.width as usize;
    let h = buf.height as usize;
    let n = w * h;
    let planes = (buf.channels as usize).min(3);
    let levels = 2 + brush_detail / 2;
    for c in 0..planes {
        let base = c * n;
        for v in &mut buf.data[base..base + n] {
            *v = reduce::posterize(*v, levels);
        }
    }
    let src = buf.data.clone();
    let jitter = noise::value_noise(w, h, seed);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let radius = brush_size as f64 * 0.6 + 0.5;
    let spacing = (brush_size as usize / 2).max(1);

    for cy in (0..h).step_by(spacing) {
        for cx in (0..w).step_by(spacing) {
            let jx = unit_f64(&mut rng) * spacing as f64;
            let jy = unit_f64(&mut rng) * spacing as f64;
            let angle = unit_f64(&mut rng) * std::f64::consts::PI;
            let center = (
                (cx as f64 + jx).clamp(0.0, (w - 1) as f64),
                (cy as f64 + jy).clamp(0.0, (h - 1) as f64),
            );
            let idx = clamp_index(center.1 as isize, h) * w + clamp_index(center.0 as isize, w);
            let color = [
                src[idx] as f64,
                src[n + idx] as f64,
                src[2 * n + idx] as f64,
            ];
            paint_daub(
                &mut buf.data,
                w,
                h,
                planes,
                center,
                radius,
                0.5,
                angle,
                color,
                2.0,
                0.0,
                &jitter,
            );
        }
    }

    apply_surface(&mut buf.data, w, h, n, planes, texture);
    Ok(())
}

/// Add an embossed procedural surface term from full [`TextureOptions`], scaled
/// by `strength`. Rough Pastels and Underpainting expose the surface's scaling,
/// relief, light direction and invert, unlike the fixed preset of
/// [`apply_surface`].
fn apply_surface_options(
    data: &mut [u8],
    w: usize,
    h: usize,
    n: usize,
    planes: usize,
    options: &TextureOptions,
    strength: f64,
) {
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let shade = texture::emboss(
                |hx, hy| texture::surface_height(options.surface, hx, hy, options.scaling),
                x,
                y,
                options.light_direction,
                options.relief,
                options.invert,
            );
            let grain =
                (texture::surface_height(options.surface, x, y, options.scaling) - 0.5) * 25.0;
            let delta = (shade * 12.0 + grain) * strength;
            for c in 0..planes {
                data[c * n + i] = reduce::clamp_u8(data[c * n + i] as f64 + delta);
            }
        }
    }
}

/// Rough Pastels: chalk daubs over a lit procedural surface. `stroke_length`
/// sets the daub reach, `stroke_detail` the retained tonal bands, `foreground`
/// the chalk, and `background` the paper that light areas settle toward.
///
/// ponytail: single-ellipse chalk stamps plus a per-pixel surface emboss;
/// Adobe's scratchboard pastel is closed. A bristle-tipped stamp is the upgrade
/// path if the strokes read too soft.
pub fn rough_pastels(
    buf: &mut PixelBuffer,
    stroke_length: u8,
    stroke_detail: u8,
    texture: TextureOptions,
    foreground: [u8; 3],
    background: [u8; 3],
    seed: u64,
) -> Result<(), FilterError> {
    validate(buf)?;
    if stroke_length > 40 {
        return Err(FilterError::InvalidParams(format!(
            "rough pastels stroke length {stroke_length} is outside 0..=40"
        )));
    }
    if !(1..=20).contains(&stroke_detail) {
        return Err(FilterError::InvalidParams(format!(
            "rough pastels stroke detail {stroke_detail} is outside 1..=20"
        )));
    }
    if !texture::texture_options_valid(&texture) {
        return Err(FilterError::InvalidParams(
            "rough pastels texture options are out of range".into(),
        ));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let n = w * h;
    let planes = (buf.channels as usize).min(3);
    let levels = 2 + stroke_detail / 2;
    for c in 0..planes {
        let base = c * n;
        for v in &mut buf.data[base..base + n] {
            *v = reduce::posterize(*v, levels);
        }
    }
    let src = buf.data.clone();

    let lum: Vec<f64> = (0..n)
        .map(|i| luma(src[i] as f64, src[n + i] as f64, src[2 * n + i] as f64))
        .collect();
    let jitter = noise::value_noise(w, h, seed);
    let mut rng = ChaCha8Rng::seed_from_u64(seed ^ 0x9E37_79B9_7F4A_7C15);
    let radius = stroke_length as f64 * 0.7 + 0.8;
    let spacing = (stroke_length as usize / 3).max(1);

    for cy in (0..h).step_by(spacing) {
        for cx in (0..w).step_by(spacing) {
            let jx = unit_f64(&mut rng) * spacing as f64;
            let jy = unit_f64(&mut rng) * spacing as f64;
            let angle = unit_f64(&mut rng) * std::f64::consts::PI;
            let center = (
                (cx as f64 + jx).clamp(0.0, (w - 1) as f64),
                (cy as f64 + jy).clamp(0.0, (h - 1) as f64),
            );
            let idx = clamp_index(center.1 as isize, h) * w + clamp_index(center.0 as isize, w);
            let t = (lum[idx] / 255.0).clamp(0.0, 1.0);
            let mut color = [0.0f64; 3];
            for (c, col) in color.iter_mut().enumerate() {
                let chalk = foreground[c] as f64 * (1.0 - t) + background[c] as f64 * t;
                let s = src[c * n + idx] as f64;
                *col = (s * 0.5 + chalk * 0.5).clamp(0.0, 255.0);
            }
            paint_daub(
                &mut buf.data,
                w,
                h,
                planes,
                center,
                radius,
                0.7,
                angle,
                color,
                1.5,
                0.5,
                &jitter,
            );
        }
    }

    apply_surface_options(&mut buf.data, w, h, n, planes, &texture, 1.0);
    Ok(())
}

/// Smudge Stick: short diagonal smears. Dark areas pull toward a neighbouring
/// sample along the stroke; light areas brighten toward white as `highlight_area`
/// grows, all scaled by `intensity`.
///
/// ponytail: a single-neighbour directional pull, not Adobe's iterative smudge
/// brush. Carrying a running colour along the stroke is the upgrade path.
pub fn smudge_stick(
    buf: &mut PixelBuffer,
    stroke_length: u8,
    highlight_area: u8,
    intensity: u8,
    seed: u64,
) -> Result<(), FilterError> {
    validate(buf)?;
    if stroke_length > 10 {
        return Err(FilterError::InvalidParams(format!(
            "smudge stick stroke length {stroke_length} is outside 0..=10"
        )));
    }
    if highlight_area > 20 {
        return Err(FilterError::InvalidParams(format!(
            "smudge stick highlight area {highlight_area} is outside 0..=20"
        )));
    }
    if intensity > 10 {
        return Err(FilterError::InvalidParams(format!(
            "smudge stick intensity {intensity} is outside 0..=10"
        )));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let n = w * h;
    let planes = (buf.channels as usize).min(3);
    let src = buf.data.clone();
    let lum: Vec<f64> = (0..n)
        .map(|i| luma(src[i] as f64, src[n + i] as f64, src[2 * n + i] as f64))
        .collect();
    let angles = stroke_angles(w, h, 4, seed);
    let flow = noise::value_noise(w, h, seed ^ 0x517C_C1B7_2722_0A95);
    let len = stroke_length as f64;
    let smudge = intensity as f64 / 10.0;
    let highlight = highlight_area as f64 / 20.0;

    for y in 0..h {
        for x in 0..w {
            let p = y * w + x;
            let angle =
                std::f64::consts::FRAC_PI_4 + (angles[p] - std::f64::consts::FRAC_PI_2) * 0.5;
            let (sin, cos) = angle.sin_cos();
            let t = (flow[p] - 0.5) * len;
            let sx = clamp_index(x as isize + (t * cos).round() as isize, w);
            let sy = clamp_index(y as isize + (t * sin).round() as isize, h);
            let q = sy * w + sx;
            let l = (lum[p] / 255.0).clamp(0.0, 1.0);
            let dark = (1.0 - l) * smudge * 0.8;
            let gate = ((l - (1.0 - highlight * smudge)).max(0.0) / 0.5).clamp(0.0, 1.0);
            let brighten = gate * smudge;
            for c in 0..planes {
                let base = c * n;
                let cur = src[base + p] as f64;
                let samp = src[base + q] as f64;
                let mut v = cur + (samp - cur) * dark;
                v += (255.0 - v) * brighten;
                buf.data[base + p] = reduce::clamp_u8(v);
            }
        }
    }
    Ok(())
}

/// Underpainting: paint a lit procedural surface as the ground (visible in
/// proportion to `texture_coverage`, 0 = none to 40 = full) then lay the image
/// over it as `brush_size`-scaled daubs.
///
/// ponytail: the image is daubed first and the ground mixed back by coverage,
/// rather than Adobe's separate underpainting pass; closed. A per-stroke
/// occlusion mask is the upgrade path.
pub fn underpainting(
    buf: &mut PixelBuffer,
    brush_size: u8,
    texture_coverage: u8,
    texture: TextureOptions,
    seed: u64,
) -> Result<(), FilterError> {
    validate(buf)?;
    if brush_size > 40 {
        return Err(FilterError::InvalidParams(format!(
            "underpainting brush size {brush_size} is outside 0..=40"
        )));
    }
    if texture_coverage > 40 {
        return Err(FilterError::InvalidParams(format!(
            "underpainting texture coverage {texture_coverage} is outside 0..=40"
        )));
    }
    if !texture::texture_options_valid(&texture) {
        return Err(FilterError::InvalidParams(
            "underpainting texture options are out of range".into(),
        ));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let n = w * h;
    let planes = (buf.channels as usize).min(3);
    let src = buf.data.clone();
    let jitter = noise::value_noise(w, h, seed);
    let mut rng = ChaCha8Rng::seed_from_u64(seed ^ 0xC2B2_AE3D_27D4_EB4F);
    let radius = brush_size as f64 * 0.7 + 0.5;
    let spacing = (brush_size as usize / 2).max(1);

    for cy in (0..h).step_by(spacing) {
        for cx in (0..w).step_by(spacing) {
            let jx = unit_f64(&mut rng) * spacing as f64;
            let jy = unit_f64(&mut rng) * spacing as f64;
            let angle = unit_f64(&mut rng) * std::f64::consts::PI;
            let center = (
                (cx as f64 + jx).clamp(0.0, (w - 1) as f64),
                (cy as f64 + jy).clamp(0.0, (h - 1) as f64),
            );
            let idx = clamp_index(center.1 as isize, h) * w + clamp_index(center.0 as isize, w);
            let color = [
                src[idx] as f64,
                src[n + idx] as f64,
                src[2 * n + idx] as f64,
            ];
            paint_daub(
                &mut buf.data,
                w,
                h,
                planes,
                center,
                radius,
                0.6,
                angle,
                color,
                2.0,
                0.25,
                &jitter,
            );
        }
    }

    let coverage = texture_coverage as f64 / 40.0;
    if coverage > 0.0 {
        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                let shade = texture::emboss(
                    |hx, hy| texture::surface_height(texture.surface, hx, hy, texture.scaling),
                    x,
                    y,
                    texture.light_direction,
                    texture.relief,
                    texture.invert,
                );
                let grain =
                    (texture::surface_height(texture.surface, x, y, texture.scaling) - 0.5) * 30.0;
                let under = reduce::clamp_u8(128.0 + shade * 14.0 + grain) as f64;
                for c in 0..planes {
                    let v = buf.data[c * n + i] as f64;
                    buf.data[c * n + i] = reduce::clamp_u8(v + (under - v) * coverage * 0.7);
                }
            }
        }
    }
    Ok(())
}

/// Watercolor: posterize and soft-smooth the colours, saturate them toward the
/// `foreground` ink where tonal edges are strong, wash light flat areas toward
/// the `background` paper, deepen shadows by `shadow_intensity`, and add a
/// `texture` surface plus seeded paper mottle. `brush_detail` retains more
/// bands and less smoothing.
///
/// ponytail: a posterize + Sobel-saturate model, not Adobe's pigment diffusion.
/// A wet-edge bleed pass is the upgrade path.
pub fn watercolor(
    buf: &mut PixelBuffer,
    brush_detail: u8,
    shadow_intensity: u8,
    texture: u8,
    foreground: [u8; 3],
    background: [u8; 3],
    seed: u64,
) -> Result<(), FilterError> {
    validate(buf)?;
    if !(1..=14).contains(&brush_detail) {
        return Err(FilterError::InvalidParams(format!(
            "watercolor brush detail {brush_detail} is outside 1..=14"
        )));
    }
    if shadow_intensity > 10 {
        return Err(FilterError::InvalidParams(format!(
            "watercolor shadow intensity {shadow_intensity} is outside 0..=10"
        )));
    }
    if !(1..=3).contains(&texture) {
        return Err(FilterError::InvalidParams(format!(
            "watercolor texture {texture} is outside 1..=3"
        )));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let n = w * h;
    let planes = (buf.channels as usize).min(3);

    let levels = 2 + brush_detail / 2;
    for c in 0..planes {
        let base = c * n;
        for v in &mut buf.data[base..base + n] {
            *v = reduce::posterize(*v, levels);
        }
    }

    let radius = ((14 - brush_detail as usize) / 4).min(3);
    if radius > 0 {
        let area = ((2 * radius + 1) * (2 * radius + 1)) as f64;
        for c in 0..planes {
            let base = c * n;
            let field: Vec<f64> = buf.data[base..base + n].iter().map(|&v| v as f64).collect();
            let means = box_sum(&field, w, h, radius);
            for (i, &m) in means.iter().enumerate() {
                buf.data[base + i] = reduce::clamp_u8(field[i] + (m / area - field[i]) * 0.5);
            }
        }
    }

    apply_surface(&mut buf.data, w, h, n, planes, texture);

    let lum: Vec<f64> = (0..n)
        .map(|i| {
            luma(
                buf.data[i] as f64,
                buf.data[n + i] as f64,
                buf.data[2 * n + i] as f64,
            )
        })
        .collect();
    let luma_at =
        |ax: isize, ay: isize| -> f64 { lum[clamp_index(ay, h) * w + clamp_index(ax, w)] };
    let paper = noise::value_noise(w, h, seed);
    let shadow = shadow_intensity as f64 / 10.0;

    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let l = (lum[i] / 255.0).clamp(0.0, 1.0);
            let e = (reduce::edge_magnitude(luma_at, x, y) / 800.0).clamp(0.0, 1.0);
            let ink = (e * 0.85 + (1.0 - l) * shadow * 0.5).clamp(0.0, 1.0);
            let wash = ((1.0 - e) * l).powi(2) * 0.6 * (1.0 - shadow * 0.5);
            let mottle = (paper[i] - 0.5) * 18.0;
            for c in 0..planes {
                let base = c * n;
                let mut v = buf.data[base + i] as f64;
                v += (v - lum[i]) * e * 0.8;
                v += (foreground[c] as f64 - v) * ink;
                v += (background[c] as f64 - v) * wash;
                v += mottle;
                buf.data[base + i] = reduce::clamp_u8(v);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{apply, BrushType, Filter};

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

    fn ramp(w: u32) -> PixelBuffer {
        let n = w as usize;
        let mut data = vec![0u8; n * 3];
        let denom = (w.max(2) - 1) as f64;
        for x in 0..n {
            let v = (255.0 * x as f64 / denom).round() as u8;
            data[x] = v;
            data[n + x] = v;
            data[2 * n + x] = v;
        }
        PixelBuffer {
            width: w,
            height: 1,
            channels: 3,
            data,
        }
    }

    fn alpha_plane(buf: &PixelBuffer) -> Vec<u8> {
        let n = buf.pixel_count();
        buf.data[3 * n..4 * n].to_vec()
    }

    fn distinct(plane: &[u8]) -> usize {
        plane
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    }

    #[test]
    fn each_filter_changes_the_colour_planes() {
        let base = gradient(32, 8, 3);
        let n = base.pixel_count();
        let cases: Vec<Filter> = vec![
            Filter::Cutout {
                levels: 4,
                edge_simplicity: 0,
                edge_fidelity: 1,
            },
            Filter::FilmGrain {
                grain: 10,
                highlight_area: 5,
                intensity: 5,
                seed: 42,
            },
            Filter::NeonGlow {
                glow_size: 8,
                glow_brightness: 40,
                glow_color: [0, 0, 255],
            },
            Filter::PosterEdges {
                edge_thickness: 2,
                edge_intensity: 8,
                posterization: 4,
            },
            Filter::PaintDaubs {
                brush_size: 8,
                sharpness: 20,
                brush_type: BrushType::Simple,
                seed: 3,
            },
            Filter::PaletteKnife {
                stroke_size: 12,
                stroke_detail: 2,
                softness: 8,
                seed: 3,
            },
            Filter::PlasticWrap {
                highlight_strength: 12,
                detail: 6,
                smoothness: 3,
            },
            Filter::Sponge {
                brush_size: 6,
                definition: 18,
                smoothness: 4,
                seed: 3,
            },
            Filter::ColoredPencil {
                pencil_width: 6,
                stroke_pressure: 8,
                paper_brightness: 20,
                foreground: [10, 10, 10],
                background: [240, 240, 240],
                seed: 3,
            },
            Filter::DryBrush {
                brush_size: 8,
                brush_detail: 6,
                texture: 2,
                seed: 3,
            },
            Filter::Fresco {
                brush_size: 8,
                brush_detail: 6,
                texture: 2,
                seed: 3,
            },
        ];
        for filter in cases {
            let mut out = base.clone();
            apply(&filter, &mut out).unwrap();
            assert_ne!(
                out.data[..planes_of(&out, n)],
                base.data[..planes_of(&base, n)],
                "{filter:?} did not change colour"
            );
        }
    }

    fn planes_of(buf: &PixelBuffer, n: usize) -> usize {
        (buf.channels as usize).min(3) * n
    }

    #[test]
    fn every_artistic_filter_preserves_alpha_via_apply() {
        let base = gradient(16, 6, 4);
        let expected = alpha_plane(&base);
        let cases: Vec<Filter> = vec![
            Filter::Cutout {
                levels: 8,
                edge_simplicity: 3,
                edge_fidelity: 3,
            },
            Filter::FilmGrain {
                grain: 20,
                highlight_area: 20,
                intensity: 10,
                seed: 9,
            },
            Filter::NeonGlow {
                glow_size: -12,
                glow_brightness: 50,
                glow_color: [255, 0, 0],
            },
            Filter::PosterEdges {
                edge_thickness: 10,
                edge_intensity: 10,
                posterization: 10,
            },
            Filter::PaintDaubs {
                brush_size: 10,
                sharpness: 40,
                brush_type: BrushType::WideBlurry,
                seed: 9,
            },
            Filter::PaletteKnife {
                stroke_size: 20,
                stroke_detail: 3,
                softness: 10,
                seed: 9,
            },
            Filter::PlasticWrap {
                highlight_strength: 20,
                detail: 15,
                smoothness: 15,
            },
            Filter::Sponge {
                brush_size: 10,
                definition: 25,
                smoothness: 15,
                seed: 9,
            },
            Filter::ColoredPencil {
                pencil_width: 24,
                stroke_pressure: 15,
                paper_brightness: 50,
                foreground: [0, 0, 0],
                background: [255, 255, 255],
                seed: 9,
            },
            Filter::DryBrush {
                brush_size: 50,
                brush_detail: 12,
                texture: 3,
                seed: 9,
            },
            Filter::Fresco {
                brush_size: 50,
                brush_detail: 12,
                texture: 3,
                seed: 9,
            },
        ];
        for filter in cases {
            let mut out = base.clone();
            apply(&filter, &mut out).unwrap();
            assert_eq!(alpha_plane(&out), expected, "{filter:?} touched alpha");
        }
    }

    #[test]
    fn boundary_values_are_accepted_and_out_of_range_rejected() {
        let base = gradient(8, 4, 3);

        assert!(cutout(&mut base.clone(), 2, 0, 1).is_ok());
        assert!(cutout(&mut base.clone(), 8, 10, 3).is_ok());
        for (l, s, f) in [(1u8, 0u8, 1u8), (9, 0, 1), (2, 11, 1), (2, 0, 0), (2, 0, 4)] {
            let mut out = base.clone();
            assert!(
                matches!(
                    cutout(&mut out, l, s, f),
                    Err(FilterError::InvalidParams(_))
                ),
                "cutout {l},{s},{f} should reject"
            );
            assert_eq!(out, base, "rejected cutout modified the buffer");
        }

        assert!(film_grain(&mut base.clone(), 0, 0, 0, 1).is_ok());
        assert!(film_grain(&mut base.clone(), 20, 20, 10, 1).is_ok());
        for (g, ha, i) in [(21u8, 0u8, 0u8), (0, 21, 0), (0, 0, 11)] {
            let mut out = base.clone();
            assert!(matches!(
                film_grain(&mut out, g, ha, i, 1),
                Err(FilterError::InvalidParams(_))
            ));
        }

        assert!(neon_glow(&mut base.clone(), -24, 0, [0, 0, 0]).is_ok());
        assert!(neon_glow(&mut base.clone(), 24, 50, [255, 255, 255]).is_ok());
        for (size, bright) in [(-25i32, 0u8), (25, 0), (0, 51)] {
            let mut out = base.clone();
            assert!(matches!(
                neon_glow(&mut out, size, bright, [0, 0, 0]),
                Err(FilterError::InvalidParams(_))
            ));
        }

        assert!(poster_edges(&mut base.clone(), 0, 0, 0).is_ok());
        assert!(poster_edges(&mut base.clone(), 10, 10, 10).is_ok());
        for (t, i, p) in [(11u8, 0u8, 0u8), (0, 11, 0), (0, 0, 11)] {
            let mut out = base.clone();
            assert!(matches!(
                poster_edges(&mut out, t, i, p),
                Err(FilterError::InvalidParams(_))
            ));
        }
    }

    #[test]
    fn new_artistic_boundaries_accept_and_out_of_range_rejects() {
        let base = gradient(8, 4, 3);

        assert!(paint_daubs(&mut base.clone(), 1, 0, BrushType::Simple, 1).is_ok());
        assert!(paint_daubs(&mut base.clone(), 50, 40, BrushType::Sparkle, 1).is_ok());
        for (b, s) in [(0u8, 0u8), (51, 0), (1, 41)] {
            let mut out = base.clone();
            assert!(matches!(
                paint_daubs(&mut out, b, s, BrushType::Simple, 1),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected paint daubs modified the buffer");
        }

        assert!(palette_knife(&mut base.clone(), 1, 1, 0, 1).is_ok());
        assert!(palette_knife(&mut base.clone(), 50, 3, 10, 1).is_ok());
        for (sz, d, so) in [
            (0u8, 1u8, 0u8),
            (51, 1, 0),
            (1, 0, 0),
            (1, 4, 0),
            (1, 1, 11),
        ] {
            let mut out = base.clone();
            assert!(matches!(
                palette_knife(&mut out, sz, d, so, 1),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected palette knife modified the buffer");
        }

        assert!(plastic_wrap(&mut base.clone(), 0, 1, 1).is_ok());
        assert!(plastic_wrap(&mut base.clone(), 20, 15, 15).is_ok());
        for (hs, d, sm) in [
            (21u8, 1u8, 1u8),
            (0, 0, 1),
            (0, 16, 1),
            (0, 1, 0),
            (0, 1, 16),
        ] {
            let mut out = base.clone();
            assert!(matches!(
                plastic_wrap(&mut out, hs, d, sm),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected plastic wrap modified the buffer");
        }

        assert!(sponge(&mut base.clone(), 0, 0, 1, 1).is_ok());
        assert!(sponge(&mut base.clone(), 10, 25, 15, 1).is_ok());
        for (b, d, sm) in [(11u8, 0u8, 1u8), (0, 26, 1), (0, 0, 0), (0, 0, 16)] {
            let mut out = base.clone();
            assert!(matches!(
                sponge(&mut out, b, d, sm, 1),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected sponge modified the buffer");
        }

        assert!(colored_pencil(&mut base.clone(), 1, 0, 0, [0, 0, 0], [255, 255, 255], 1).is_ok());
        assert!(
            colored_pencil(&mut base.clone(), 24, 15, 50, [0, 0, 0], [255, 255, 255], 1).is_ok()
        );
        for (pw, sp, pb) in [(0u8, 0u8, 0u8), (25, 0, 0), (1, 16, 0), (1, 0, 51)] {
            let mut out = base.clone();
            assert!(matches!(
                colored_pencil(&mut out, pw, sp, pb, [0, 0, 0], [255, 255, 255], 1),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected colored pencil modified the buffer");
        }
        assert!(
            colored_pencil(&mut base.clone(), 8, 8, 20, [7, 7, 7], [7, 7, 7], 1).is_ok(),
            "equal fg/bg must not panic"
        );

        assert!(dry_brush(&mut base.clone(), 1, 1, 1, 1).is_ok());
        assert!(dry_brush(&mut base.clone(), 50, 12, 3, 1).is_ok());
        for (b, d, t) in [
            (0u8, 1u8, 1u8),
            (51, 1, 1),
            (1, 0, 1),
            (1, 13, 1),
            (1, 1, 0),
            (1, 1, 4),
        ] {
            let mut out = base.clone();
            assert!(matches!(
                dry_brush(&mut out, b, d, t, 1),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected dry brush modified the buffer");
        }

        assert!(fresco(&mut base.clone(), 1, 1, 1, 1).is_ok());
        assert!(fresco(&mut base.clone(), 50, 12, 3, 1).is_ok());
        for (b, d, t) in [
            (0u8, 1u8, 1u8),
            (51, 1, 1),
            (1, 0, 1),
            (1, 13, 1),
            (1, 1, 0),
            (1, 1, 4),
        ] {
            let mut out = base.clone();
            assert!(matches!(
                fresco(&mut out, b, d, t, 1),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected fresco modified the buffer");
        }
    }

    fn flat(w: u32, h: u32, value: u8) -> PixelBuffer {
        let n = (w * h) as usize;
        PixelBuffer {
            width: w,
            height: h,
            channels: 3,
            data: vec![value; n * 3],
        }
    }

    #[test]
    fn colored_pencil_background_shows_and_edges_survive() {
        let base = flat(16, 8, 120);
        let (mut light, mut dark) = (base.clone(), base.clone());
        colored_pencil(&mut light, 6, 6, 10, [0, 0, 0], [255, 255, 255], 5).unwrap();
        colored_pencil(&mut dark, 6, 6, 10, [0, 0, 0], [0, 0, 0], 5).unwrap();
        assert_ne!(
            light.data, dark.data,
            "two background colours must give different flat regions"
        );

        let (mut a, mut b, mut c) = (base.clone(), base.clone(), base.clone());
        colored_pencil(&mut a, 6, 6, 10, [0, 0, 0], [255, 255, 255], 5).unwrap();
        colored_pencil(&mut b, 6, 6, 10, [0, 0, 0], [255, 255, 255], 5).unwrap();
        colored_pencil(&mut c, 6, 6, 10, [0, 0, 0], [255, 255, 255], 6).unwrap();
        assert_eq!(a.data, b.data, "same seed must be bit-identical");
        assert_ne!(a.data, c.data, "different seed must differ");

        let (w, h) = (32u32, 8u32);
        let n = (w * h) as usize;
        let mut data = vec![0u8; n * 3];
        for y in 0..h as usize {
            for x in 0..w as usize {
                let v = if x < (w / 2) as usize { 0 } else { 255 };
                for ch in 0..3 {
                    data[ch * n + y * w as usize + x] = v;
                }
            }
        }
        let step = PixelBuffer {
            width: w,
            height: h,
            channels: 3,
            data,
        };
        let mut edged = step.clone();
        colored_pencil(&mut edged, 4, 4, 0, [0, 0, 0], [255, 255, 255], 1).unwrap();
        let half = (w / 2) as usize;
        let mut left = 0u64;
        let mut right = 0u64;
        for y in 0..h as usize {
            for x in 0..w as usize {
                let v = edged.data[y * w as usize + x] as u64;
                if x < half {
                    left += v;
                } else {
                    right += v;
                }
            }
        }
        assert!(
            right > left,
            "edge must retain a luminance change (left {left}, right {right})"
        );
    }

    #[test]
    fn dry_brush_and_fresco_texture_size_and_seed_change_the_result() {
        let base = gradient(32, 8, 3);

        let (mut d1, mut d3) = (base.clone(), base.clone());
        dry_brush(&mut d1, 10, 6, 1, 4).unwrap();
        dry_brush(&mut d3, 10, 6, 3, 4).unwrap();
        assert_ne!(d1.data, d3.data, "dry brush texture 1 vs 3 must differ");
        let (mut ds, mut dl) = (base.clone(), base.clone());
        dry_brush(&mut ds, 1, 6, 2, 4).unwrap();
        dry_brush(&mut dl, 50, 6, 2, 4).unwrap();
        assert_ne!(ds.data, dl.data, "dry brush size 1 vs 50 must differ");

        let (mut f1, mut f3) = (base.clone(), base.clone());
        fresco(&mut f1, 10, 6, 1, 4).unwrap();
        fresco(&mut f3, 10, 6, 3, 4).unwrap();
        assert_ne!(f1.data, f3.data, "fresco texture 1 vs 3 must differ");
        let (mut fs, mut fl) = (base.clone(), base.clone());
        fresco(&mut fs, 1, 6, 2, 4).unwrap();
        fresco(&mut fl, 50, 6, 2, 4).unwrap();
        assert_ne!(fs.data, fl.data, "fresco size 1 vs 50 must differ");

        let (mut a, mut b, mut c) = (base.clone(), base.clone(), base.clone());
        dry_brush(&mut a, 10, 6, 2, 7).unwrap();
        dry_brush(&mut b, 10, 6, 2, 7).unwrap();
        dry_brush(&mut c, 10, 6, 2, 8).unwrap();
        assert_eq!(a.data, b.data, "dry brush same seed must be bit-identical");
        assert_ne!(a.data, c.data, "dry brush different seed must differ");

        let (mut d, mut e, mut f) = (base.clone(), base.clone(), base.clone());
        fresco(&mut d, 10, 6, 2, 7).unwrap();
        fresco(&mut e, 10, 6, 2, 7).unwrap();
        fresco(&mut f, 10, 6, 2, 8).unwrap();
        assert_eq!(d.data, e.data, "fresco same seed must be bit-identical");
        assert_ne!(d.data, f.data, "fresco different seed must differ");
    }

    fn longest_run(plane: &[u8]) -> usize {
        let mut best = 0;
        let mut cur = 0;
        let mut prev = None;
        for &v in plane {
            if Some(v) == prev {
                cur += 1;
            } else {
                cur = 1;
            }
            prev = Some(v);
            best = best.max(cur);
        }
        best
    }

    #[test]
    fn paint_daubs_types_seed_and_size() {
        let base = gradient(32, 8, 3);
        let (mut simple, mut rough) = (base.clone(), base.clone());
        paint_daubs(&mut simple, 10, 20, BrushType::Simple, 7).unwrap();
        paint_daubs(&mut rough, 10, 20, BrushType::DarkRough, 7).unwrap();
        assert_ne!(simple.data, rough.data, "brush types must differ");

        let (mut a, mut b, mut c) = (base.clone(), base.clone(), base.clone());
        paint_daubs(&mut a, 10, 20, BrushType::Simple, 7).unwrap();
        paint_daubs(&mut b, 10, 20, BrushType::Simple, 7).unwrap();
        paint_daubs(&mut c, 10, 20, BrushType::Simple, 8).unwrap();
        assert_eq!(a.data, b.data, "same seed must be bit-identical");
        assert_ne!(a.data, c.data, "different seed must differ");

        let ramp = {
            let (rw, rh) = (64u32, 8u32);
            let rn = (rw * rh) as usize;
            let mut data = vec![0u8; rn * 3];
            for y in 0..rh as usize {
                for x in 0..rw as usize {
                    let v = (255.0 * x as f64 / (rw - 1) as f64).round() as u8;
                    for c in 0..3 {
                        data[c * rn + y * rw as usize + x] = v;
                    }
                }
            }
            PixelBuffer {
                width: rw,
                height: rh,
                channels: 3,
                data,
            }
        };
        let (mut fine, mut coarse) = (ramp.clone(), ramp.clone());
        paint_daubs(&mut fine, 1, 20, BrushType::Simple, 5).unwrap();
        paint_daubs(&mut coarse, 50, 20, BrushType::Simple, 5).unwrap();
        let n = ramp.pixel_count();
        let fine_run = longest_run(&fine.data[..n]);
        let coarse_run = longest_run(&coarse.data[..n]);
        assert!(
            coarse_run > fine_run,
            "brush 50 runs ({coarse_run}) must exceed brush 1 runs ({fine_run})"
        );
    }

    #[test]
    fn palette_knife_and_sponge_are_seeded() {
        let base = gradient(32, 8, 3);
        let (mut a, mut b, mut c) = (base.clone(), base.clone(), base.clone());
        palette_knife(&mut a, 12, 2, 6, 11).unwrap();
        palette_knife(&mut b, 12, 2, 6, 11).unwrap();
        palette_knife(&mut c, 12, 2, 6, 12).unwrap();
        assert_eq!(
            a.data, b.data,
            "palette knife same seed must be bit-identical"
        );
        assert_ne!(a.data, c.data, "palette knife different seed must differ");

        let (mut d, mut e, mut f) = (base.clone(), base.clone(), base.clone());
        sponge(&mut d, 6, 20, 4, 21).unwrap();
        sponge(&mut e, 6, 20, 4, 21).unwrap();
        sponge(&mut f, 6, 20, 4, 22).unwrap();
        assert_eq!(d.data, e.data, "sponge same seed must be bit-identical");
        assert_ne!(d.data, f.data, "sponge different seed must differ");
    }

    #[test]
    fn plastic_wrap_strength_raises_max_luminance() {
        let w = 32u32;
        let h = 8u32;
        let n = (w * h) as usize;
        let mid = PixelBuffer {
            width: w,
            height: h,
            channels: 3,
            data: vec![128u8; n * 3],
        };
        let (mut zero, mut full) = (mid.clone(), mid.clone());
        plastic_wrap(&mut zero, 0, 8, 4).unwrap();
        plastic_wrap(&mut full, 20, 8, 4).unwrap();

        let max_lum = |b: &PixelBuffer| -> f64 {
            (0..n)
                .map(|i| {
                    luma(
                        b.data[i] as f64,
                        b.data[n + i] as f64,
                        b.data[2 * n + i] as f64,
                    )
                })
                .fold(0.0, f64::max)
        };
        assert!(
            max_lum(&full) > max_lum(&zero),
            "highlight 20 ({}) must raise max luminance over 0 ({})",
            max_lum(&full),
            max_lum(&zero)
        );
    }

    #[test]
    fn film_grain_zero_is_noop_and_seed_is_deterministic() {
        let base = gradient(24, 8, 3);
        let mut zero = base.clone();
        film_grain(&mut zero, 0, 7, 7, 5).unwrap();
        assert_eq!(zero, base, "grain=0 must be an exact no-op");

        let (mut a, mut b, mut c) = (base.clone(), base.clone(), base.clone());
        film_grain(&mut a, 12, 4, 6, 77).unwrap();
        film_grain(&mut b, 12, 4, 6, 77).unwrap();
        film_grain(&mut c, 12, 4, 6, 78).unwrap();
        assert_eq!(a.data, b.data, "same seed must be bit-identical");
        assert_ne!(a.data, c.data, "different seed must differ");
    }

    #[test]
    fn cutout_more_levels_yield_more_bands_and_is_deterministic() {
        let base = ramp(64);
        let n = base.pixel_count();
        let (mut two, mut eight) = (base.clone(), base.clone());
        cutout(&mut two, 2, 0, 1).unwrap();
        cutout(&mut eight, 8, 0, 1).unwrap();
        let d2 = distinct(&two.data[..n]);
        let d8 = distinct(&eight.data[..n]);
        assert!(d8 > d2, "8 levels ({d8}) must exceed 2 levels ({d2})");

        let mut again = base.clone();
        cutout(&mut again, 8, 0, 1).unwrap();
        assert_eq!(again.data, eight.data, "cutout must be deterministic");
    }

    #[test]
    fn poster_edges_flattens_and_darkens_the_edge() {
        let flat = ramp(64);
        let n = flat.pixel_count();
        let before = distinct(&flat.data[..n]);
        let mut out = flat.clone();
        poster_edges(&mut out, 2, 0, 4).unwrap();
        assert!(
            distinct(&out.data[..n]) < before,
            "posterize must reduce interior levels ({before} -> {})",
            distinct(&out.data[..n])
        );

        let edged = gradient(32, 1, 3);
        let mut out = edged.clone();
        poster_edges(&mut out, 3, 10, 2).unwrap();
        assert!(
            out.data.iter().zip(&edged.data).any(|(a, b)| a < b),
            "edge band must darken at least one sample"
        );
    }

    #[test]
    fn neon_glow_tints_toward_the_colour_and_spreads_with_size() {
        let base = gradient(64, 8, 3);
        let n = base.pixel_count();

        let mut tinted = base.clone();
        neon_glow(&mut tinted, 6, 50, [255, 0, 0]).unwrap();
        let red_before: u64 = base.data[..n].iter().map(|&v| v as u64).sum();
        let red_after: u64 = tinted.data[..n].iter().map(|&v| v as u64).sum();
        assert!(red_after > red_before, "red channel must increase");
        assert_eq!(&tinted.data[n..3 * n], &base.data[n..3 * n], "tint leaked");

        let changed = |out: &PixelBuffer| {
            out.data
                .iter()
                .zip(&base.data)
                .filter(|(a, b)| a != b)
                .count()
        };
        let (mut small, mut large) = (base.clone(), base.clone());
        neon_glow(&mut small, 2, 50, [0, 0, 255]).unwrap();
        neon_glow(&mut large, 12, 50, [0, 0, 255]).unwrap();
        assert!(changed(&small) > 0, "small glow must change something");
        assert!(
            changed(&large) >= changed(&small),
            "larger glow must change at least as many samples ({} vs {})",
            changed(&large),
            changed(&small)
        );
    }

    #[test]
    fn tiny_and_three_channel_buffers_do_not_panic() {
        let tiny = PixelBuffer {
            width: 1,
            height: 1,
            channels: 4,
            data: vec![10, 20, 30, 40],
        };
        assert!(cutout(&mut tiny.clone(), 2, 0, 1).is_ok());
        assert!(film_grain(&mut tiny.clone(), 20, 20, 10, 3).is_ok());
        assert!(neon_glow(&mut tiny.clone(), 24, 50, [1, 2, 3]).is_ok());
        assert!(poster_edges(&mut tiny.clone(), 10, 10, 10).is_ok());
        assert!(paint_daubs(&mut tiny.clone(), 50, 40, BrushType::Sparkle, 3).is_ok());
        assert!(palette_knife(&mut tiny.clone(), 50, 3, 10, 3).is_ok());
        assert!(plastic_wrap(&mut tiny.clone(), 20, 15, 15).is_ok());
        assert!(sponge(&mut tiny.clone(), 10, 25, 15, 3).is_ok());

        let rgb = gradient(3, 3, 3);
        assert!(cutout(&mut rgb.clone(), 5, 10, 1).is_ok());
        assert!(film_grain(&mut rgb.clone(), 5, 5, 5, 1).is_ok());
        assert!(neon_glow(&mut rgb.clone(), -24, 10, [9, 9, 9]).is_ok());
        assert!(poster_edges(&mut rgb.clone(), 1, 1, 1).is_ok());
        assert!(paint_daubs(&mut rgb.clone(), 50, 40, BrushType::WideSharp, 1).is_ok());
        assert!(palette_knife(&mut rgb.clone(), 50, 3, 10, 1).is_ok());
        assert!(plastic_wrap(&mut rgb.clone(), 20, 15, 15).is_ok());
        assert!(sponge(&mut rgb.clone(), 10, 25, 15, 1).is_ok());
    }

    fn final_four_filters(seed: u64) -> Vec<Filter> {
        let opts = TextureOptions::default();
        vec![
            Filter::RoughPastels {
                stroke_length: 8,
                stroke_detail: 6,
                texture: opts,
                foreground: [20, 20, 20],
                background: [235, 235, 235],
                seed,
            },
            Filter::SmudgeStick {
                stroke_length: 4,
                highlight_area: 8,
                intensity: 6,
                seed,
            },
            Filter::Underpainting {
                brush_size: 10,
                texture_coverage: 24,
                texture: opts,
                seed,
            },
            Filter::Watercolor {
                brush_detail: 8,
                shadow_intensity: 6,
                texture: 2,
                foreground: [10, 10, 10],
                background: [245, 245, 245],
                seed,
            },
        ]
    }

    #[test]
    fn final_four_change_colour_and_preserve_alpha() {
        let base = gradient(16, 6, 4);
        let n = base.pixel_count();
        let expected = alpha_plane(&base);
        for filter in final_four_filters(7) {
            let mut out = base.clone();
            apply(&filter, &mut out).unwrap();
            assert_ne!(
                out.data[..3 * n],
                base.data[..3 * n],
                "{filter:?} did not change colour"
            );
            assert_eq!(alpha_plane(&out), expected, "{filter:?} touched alpha");
        }
    }

    #[test]
    fn final_four_seed_determinism() {
        let base = gradient(32, 8, 3);
        let a = final_four_filters(5);
        let b = final_four_filters(5);
        let c = final_four_filters(6);
        for i in 0..a.len() {
            let (mut oa, mut ob, mut oc) = (base.clone(), base.clone(), base.clone());
            apply(&a[i], &mut oa).unwrap();
            apply(&b[i], &mut ob).unwrap();
            apply(&c[i], &mut oc).unwrap();
            assert_eq!(oa.data, ob.data, "{:?} same seed must match", a[i]);
            assert_ne!(oa.data, oc.data, "{:?} different seed must differ", a[i]);
        }
    }

    #[test]
    fn final_four_validate_ranges_and_texture_options() {
        let base = gradient(8, 4, 3);
        let opts = TextureOptions::default();
        let (fg, bg) = ([0, 0, 0], [255, 255, 255]);

        assert!(rough_pastels(&mut base.clone(), 0, 1, opts, fg, bg, 1).is_ok());
        assert!(rough_pastels(&mut base.clone(), 40, 20, opts, fg, bg, 1).is_ok());
        for (l, d) in [(41u8, 6u8), (8, 0), (8, 21)] {
            let mut out = base.clone();
            assert!(matches!(
                rough_pastels(&mut out, l, d, opts, fg, bg, 1),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected rough pastels modified the buffer");
        }
        for bad in [
            TextureOptions {
                scaling: 49,
                ..opts
            },
            TextureOptions {
                scaling: 201,
                ..opts
            },
            TextureOptions { relief: 51, ..opts },
            TextureOptions {
                light_direction: 8,
                ..opts
            },
        ] {
            let mut out = base.clone();
            assert!(matches!(
                rough_pastels(&mut out, 8, 6, bad, fg, bg, 1),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected rough pastels texture modified buffer");
        }

        assert!(smudge_stick(&mut base.clone(), 0, 0, 0, 1).is_ok());
        assert!(smudge_stick(&mut base.clone(), 10, 20, 10, 1).is_ok());
        for (l, ha, i) in [(11u8, 0u8, 0u8), (0, 21, 0), (0, 0, 11)] {
            let mut out = base.clone();
            assert!(matches!(
                smudge_stick(&mut out, l, ha, i, 1),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected smudge stick modified the buffer");
        }

        assert!(underpainting(&mut base.clone(), 0, 0, opts, 1).is_ok());
        assert!(underpainting(&mut base.clone(), 40, 40, opts, 1).is_ok());
        for (b, tc) in [(41u8, 0u8), (0, 41)] {
            let mut out = base.clone();
            assert!(matches!(
                underpainting(&mut out, b, tc, opts, 1),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected underpainting modified the buffer");
        }
        let mut out = base.clone();
        assert!(matches!(
            underpainting(
                &mut out,
                10,
                20,
                TextureOptions {
                    scaling: 49,
                    ..opts
                },
                1
            ),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base);

        assert!(watercolor(&mut base.clone(), 1, 0, 1, fg, bg, 1).is_ok());
        assert!(watercolor(&mut base.clone(), 14, 10, 3, fg, bg, 1).is_ok());
        for (bd, si, t) in [
            (0u8, 0u8, 1u8),
            (15, 0, 1),
            (1, 11, 1),
            (1, 0, 0),
            (1, 0, 4),
        ] {
            let mut out = base.clone();
            assert!(matches!(
                watercolor(&mut out, bd, si, t, fg, bg, 1),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected watercolor modified the buffer");
        }
    }

    #[test]
    fn rough_pastels_and_underpainting_texture_options_change_output() {
        let base = gradient(32, 8, 3);
        let canvas = TextureOptions::default();
        let brick = TextureOptions {
            surface: TextureSurface::Brick,
            ..canvas
        };
        let lit = TextureOptions {
            light_direction: 4,
            relief: 20,
            ..canvas
        };

        let mut r_canvas = base.clone();
        let mut r_brick = base.clone();
        rough_pastels(
            &mut r_canvas,
            10,
            8,
            canvas,
            [10, 10, 10],
            [240, 240, 240],
            4,
        )
        .unwrap();
        rough_pastels(&mut r_brick, 10, 8, brick, [10, 10, 10], [240, 240, 240], 4).unwrap();
        assert_ne!(
            r_canvas.data, r_brick.data,
            "rough pastels surface must matter"
        );

        let mut r_lit = base.clone();
        rough_pastels(&mut r_lit, 10, 8, lit, [10, 10, 10], [240, 240, 240], 4).unwrap();
        assert_ne!(
            r_canvas.data, r_lit.data,
            "rough pastels light direction must matter"
        );

        let mut u_canvas = base.clone();
        let mut u_brick = base.clone();
        underpainting(&mut u_canvas, 10, 40, canvas, 4).unwrap();
        underpainting(&mut u_brick, 10, 40, brick, 4).unwrap();
        assert_ne!(
            u_canvas.data, u_brick.data,
            "underpainting surface must matter"
        );

        let mut u_lit = base.clone();
        underpainting(&mut u_lit, 10, 40, lit, 4).unwrap();
        assert_ne!(
            u_canvas.data, u_lit.data,
            "underpainting light direction must matter"
        );

        let mut u_zero = base.clone();
        underpainting(&mut u_zero, 10, 0, canvas, 4).unwrap();
        let mut u_full = base.clone();
        underpainting(&mut u_full, 10, 40, canvas, 4).unwrap();
        assert_ne!(u_zero.data, u_full.data, "coverage 0 vs 40 must differ");
    }

    #[test]
    fn final_four_equal_colours_and_tiny_buffer_are_safe() {
        let base = gradient(8, 4, 3);
        let opts = TextureOptions::default();
        assert!(
            rough_pastels(&mut base.clone(), 8, 6, opts, [7, 7, 7], [7, 7, 7], 1).is_ok(),
            "rough pastels equal fg/bg must not panic"
        );
        assert!(
            watercolor(&mut base.clone(), 8, 5, 2, [7, 7, 7], [7, 7, 7], 1).is_ok(),
            "watercolor equal fg/bg must not panic"
        );

        let tiny = PixelBuffer {
            width: 1,
            height: 1,
            channels: 4,
            data: vec![10, 20, 30, 40],
        };
        assert!(rough_pastels(
            &mut tiny.clone(),
            40,
            20,
            opts,
            [0, 0, 0],
            [255, 255, 255],
            1
        )
        .is_ok());
        assert!(smudge_stick(&mut tiny.clone(), 10, 20, 10, 1).is_ok());
        assert!(underpainting(&mut tiny.clone(), 40, 40, opts, 1).is_ok());
        assert!(watercolor(&mut tiny.clone(), 14, 10, 3, [0, 0, 0], [255, 255, 255], 1).is_ok());
    }
}
