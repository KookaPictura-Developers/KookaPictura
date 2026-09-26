use pictura_core::PixelBuffer;
use rand_chacha::{rand_core::SeedableRng, ChaCha8Rng};

use crate::artistic::{noise, reduce, texture};
use crate::kernel::{clamp_index, unit_f64};
use crate::luma::luma;
use crate::{validate, BrushType, FilterError, TextureOptions};

use super::common::{
    apply_surface, apply_surface_options, box_sum, paint_daub, stroke_angles, validate_brush,
};

/// Paint Daubs: oriented daubs on a `brush_size`-related grid, each filled with
/// the source colour at its centre. `sharpness` hardens the daub edge and the
/// six [`BrushType`]s vary shape, hardness and tonal skew.
///
/// ponytail: one daub per grid cell with a hard elliptical falloff; the reference's
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

/// Palette Knife: pull each sample toward the source colour a short distance
/// along a seeded stroke direction, repeated `stroke_detail` times and mixed by
/// `softness`.
///
/// ponytail: a directional neighbour pull, not the reference's iterative paint-load
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
/// ponytail: a Sobel gate and a quadratic sheen, not the reference's plastic-surface
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
/// ponytail: signed soft discs over per-pixel sponge noise; the reference's porous
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

/// Colored Pencil: posterize to a small palette, lay down crosshatch strokes
/// tinted by `foreground`, and blend smooth areas toward the `background` paper
/// lightened by `paper_brightness`. The Sobel edge weight keeps the reduced
/// source colour at strong edges, so the tonal step survives.
///
/// ponytail: geometric crosshatch with noise jitter, not the reference's pressure-
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
/// ponytail: one directional sample per pixel plus an embossed surface; the reference's
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
/// The reference's plaster daub is closed.
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

/// Rough Pastels: chalk daubs over a lit procedural surface. `stroke_length`
/// sets the daub reach, `stroke_detail` the retained tonal bands, `foreground`
/// the chalk, and `background` the paper that light areas settle toward.
///
/// ponytail: single-ellipse chalk stamps plus a per-pixel surface emboss;
/// The reference's scratchboard pastel is closed. A bristle-tipped stamp is the upgrade
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
/// ponytail: a single-neighbour directional pull, not the reference's iterative smudge
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
/// rather than the reference's separate underpainting pass; closed. A per-stroke
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
/// ponytail: a posterize + Sobel-saturate model, not the reference's pigment diffusion.
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
