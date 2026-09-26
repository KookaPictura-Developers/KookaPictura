//! Brush Strokes filter family (`m25-filter-families`): Accented Edges,
//! Angled Strokes, Crosshatch, Dark Strokes, Ink Outlines, Spatter, Sprayed
//! Strokes and Sumi-e.
//!
//! Behavioural models only: the reference's kernels are closed. Each filter is a
//! variation on edge/gradient detection + directional stroke rendering + tonal
//! gating, and every deliberate shortcut carries a `ponytail:` note.

use pictura_core::PixelBuffer;
use rand_chacha::{rand_core::SeedableRng, ChaCha8Rng};

use crate::artistic::{noise, reduce};
use crate::kernel::{box_mean, clamp_index, unit_f64};
use crate::luma::luma_plane;
use crate::{validate, FilterError, StrokeDirection};

/// Sobel magnitude of a full black-to-white step; normalizes edge responses.
const EDGE_MAX: f64 = 1020.0;

/// Widening pass: each sample takes the max of its `(2*radius+1)^2` window.
///
/// ponytail: naive max filter; make it separable if `edge_width` 14 on large
/// documents measures slow.
fn dilate_max(src: &[f64], w: usize, h: usize, radius: usize) -> Vec<f64> {
    if radius == 0 {
        return src.to_vec();
    }
    let mut out = vec![0.0f64; src.len()];
    for y in 0..h {
        for x in 0..w {
            let mut m = f64::NEG_INFINITY;
            for dy in 0..=2 * radius {
                let sy = clamp_index(y as isize + dy as isize - radius as isize, h);
                for dx in 0..=2 * radius {
                    let sx = clamp_index(x as isize + dx as isize - radius as isize, w);
                    m = m.max(src[sy * w + sx]);
                }
            }
            out[y * w + x] = m;
        }
    }
    out
}

/// Un-normalized Sobel magnitude per pixel from a luma plane.
fn edge_field(lum: &[f64], w: usize, h: usize) -> Vec<f64> {
    let mut out = vec![0.0f64; w * h];
    for y in 0..h {
        for x in 0..w {
            out[y * w + x] = reduce::edge_magnitude(
                |ax, ay| lum[clamp_index(ay, h) * w + clamp_index(ax, w)],
                x,
                y,
            );
        }
    }
    out
}

/// Averaged sample along a fixed integer direction, `len` steps each way.
fn direction_smear(src: &[f64], w: usize, h: usize, dx: isize, dy: isize, len: usize) -> Vec<f64> {
    let mut out = vec![0.0f64; src.len()];
    let count = (2 * len + 1) as f64;
    for y in 0..h {
        for x in 0..w {
            let mut s = 0.0;
            for t in -(len as isize)..=(len as isize) {
                let sx = clamp_index(x as isize + t * dx, w);
                let sy = clamp_index(y as isize + t * dy, h);
                s += src[sy * w + sx];
            }
            out[y * w + x] = s / count;
        }
    }
    out
}

/// Paint a flat-colored disc with hard coverage.
#[allow(clippy::too_many_arguments)]
fn paint_disc(
    data: &mut [u8],
    w: usize,
    h: usize,
    planes: usize,
    cx: f64,
    cy: f64,
    radius: f64,
    color: [u8; 3],
) {
    if radius <= 0.0 {
        return;
    }
    let n = w * h;
    let reach = radius.ceil() as isize;
    let x0 = clamp_index(cx as isize - reach, w);
    let x1 = clamp_index(cx as isize + reach, w);
    let y0 = clamp_index(cy as isize - reach, h);
    let y1 = clamp_index(cy as isize + reach, h);
    let r2 = radius * radius;
    for py in y0..=y1 {
        for px in x0..=x1 {
            let dx = px as f64 + 0.5 - cx;
            let dy = py as f64 + 0.5 - cy;
            if dx * dx + dy * dy > r2 {
                continue;
            }
            let idx = py * w + px;
            for (c, &col) in color.iter().enumerate().take(planes) {
                data[c * n + idx] = col;
            }
        }
    }
}

/// Apply a scalar luminance delta to every color plane of one sample.
fn shift_sample(data: &mut [u8], n: usize, planes: usize, i: usize, delta: f64) {
    for c in 0..planes {
        data[c * n + i] = reduce::clamp_u8(data[c * n + i] as f64 + delta);
    }
}

/// Accented Edges: Sobel edges stamped as dark (brightness < 25) or light
/// (brightness > 25) accent lines, widened by `edge_width` and softened by
/// `smoothness`. Brightness 25 is a no-op (edges not outlined).
///
/// ponytail: a dilated, smoothed Sobel band, not the reference's edge-following chalk /
/// ink brush. Brightness maps linearly around 25.
pub fn accented_edges(
    buf: &mut PixelBuffer,
    edge_width: u8,
    edge_brightness: u8,
    smoothness: u8,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if !(1..=14).contains(&edge_width) {
        return Err(FilterError::InvalidParams(format!(
            "accented edges edge width {edge_width} is outside 1..=14"
        )));
    }
    if edge_brightness > 50 {
        return Err(FilterError::InvalidParams(format!(
            "accented edges edge brightness {edge_brightness} is outside 0..=50"
        )));
    }
    if !(1..=15).contains(&smoothness) {
        return Err(FilterError::InvalidParams(format!(
            "accented edges smoothness {smoothness} is outside 1..=15"
        )));
    }
    if edge_brightness == 25 {
        return Ok(());
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let lum = luma_plane(&buf.data, n);
    let mag = box_mean(&edge_field(&lum, w, h), w, h, (smoothness as usize - 1) / 3);
    let band = dilate_max(&mag, w, h, (edge_width as usize - 1) / 2);
    let amount = (edge_brightness as f64 - 25.0) / 25.0;

    for (i, &m) in band.iter().enumerate() {
        let e = (m / EDGE_MAX).clamp(0.0, 1.0);
        if e > 0.0 {
            shift_sample(&mut buf.data, n, planes, i, amount * e * 200.0);
        }
    }
    Ok(())
}

/// Angled Strokes: two opposing diagonal smears blended by `direction_balance`;
/// `sharpness` unsharpens the blend around the source.
///
/// ponytail: 45-degree directional box smears, not the reference's per-direction paint
/// strokes over a gradient field.
pub fn angled_strokes(
    buf: &mut PixelBuffer,
    direction_balance: u8,
    stroke_length: u8,
    sharpness: u8,
) -> Result<(), FilterError> {
    validate(buf)?;
    if direction_balance > 100 {
        return Err(FilterError::InvalidParams(format!(
            "angled strokes direction balance {direction_balance} is outside 0..=100"
        )));
    }
    if !(3..=50).contains(&stroke_length) {
        return Err(FilterError::InvalidParams(format!(
            "angled strokes stroke length {stroke_length} is outside 3..=50"
        )));
    }
    if sharpness > 10 {
        return Err(FilterError::InvalidParams(format!(
            "angled strokes sharpness {sharpness} is outside 0..=10"
        )));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let n = w * h;
    let planes = (buf.channels as usize).min(3);
    let src = buf.data.clone();
    let balance = direction_balance as f64 / 100.0;
    let sharp = sharpness as f64 / 10.0;
    let len = stroke_length as usize;

    for c in 0..planes {
        let base = c * n;
        let field: Vec<f64> = src[base..base + n].iter().map(|&v| v as f64).collect();
        let down = direction_smear(&field, w, h, 1, 1, len);
        let up = direction_smear(&field, w, h, 1, -1, len);
        for i in 0..n {
            let mix = down[i] * (1.0 - balance) + up[i] * balance;
            buf.data[base + i] = reduce::clamp_u8(mix + sharp * (mix - field[i]));
        }
    }
    Ok(())
}

/// Crosshatch: detail-preserving base with one pencil-hatch overlay per pass;
/// `strength` (1..=3) is the pass count, each pass at a different angle.
///
/// ponytail: geometric line gratings, not the reference's pressure-modelled pencil
/// hatching; period and width derive from `stroke_length` / `sharpness`.
pub fn crosshatch(
    buf: &mut PixelBuffer,
    stroke_length: u8,
    sharpness: u8,
    strength: u8,
) -> Result<(), FilterError> {
    validate(buf)?;
    if !(3..=50).contains(&stroke_length) {
        return Err(FilterError::InvalidParams(format!(
            "crosshatch stroke length {stroke_length} is outside 3..=50"
        )));
    }
    if sharpness > 20 {
        return Err(FilterError::InvalidParams(format!(
            "crosshatch sharpness {sharpness} is outside 0..=20"
        )));
    }
    if !(1..=3).contains(&strength) {
        return Err(FilterError::InvalidParams(format!(
            "crosshatch strength {strength} is outside 1..=3"
        )));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let n = w * h;
    let planes = (buf.channels as usize).min(3);
    let period = stroke_length as f64;
    let width = 1 + (sharpness as usize) / 10;
    let dark = 0.30 + 0.35 * (sharpness as f64 / 20.0);
    let angles = [
        0.0,
        std::f64::consts::FRAC_PI_4,
        -std::f64::consts::FRAC_PI_4,
    ];

    for (k, &theta) in angles.iter().enumerate().take(strength as usize) {
        let _ = k;
        let (sin, cos) = theta.sin_cos();
        for y in 0..h {
            for x in 0..w {
                let proj = -(x as f64) * sin + (y as f64) * cos;
                if proj.rem_euclid(period) < width as f64 {
                    let i = y * w + x;
                    for c in 0..planes {
                        let v = buf.data[c * n + i] as f64;
                        buf.data[c * n + i] = reduce::clamp_u8(v * (1.0 - dark));
                    }
                }
            }
        }
    }
    Ok(())
}

/// Dark Strokes: tonal gate on luma. `balance` widens the dark gate (more dark
/// strokes), while `black_intensity` / `white_intensity` independently deepen
/// shadows and lift highlights.
///
/// ponytail: tonal gating only; the short-dark / long-light stroke direction is
/// folded into the gate rather than rasterized as separate strokes.
pub fn dark_strokes(
    buf: &mut PixelBuffer,
    balance: u8,
    black_intensity: u8,
    white_intensity: u8,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if balance > 10 {
        return Err(FilterError::InvalidParams(format!(
            "dark strokes balance {balance} is outside 0..=10"
        )));
    }
    if black_intensity > 10 {
        return Err(FilterError::InvalidParams(format!(
            "dark strokes black intensity {black_intensity} is outside 0..=10"
        )));
    }
    if white_intensity > 10 {
        return Err(FilterError::InvalidParams(format!(
            "dark strokes white intensity {white_intensity} is outside 0..=10"
        )));
    }

    let planes = (buf.channels as usize).min(3);
    let lum = luma_plane(&buf.data, n);
    let thr = 0.3 + 0.7 * (balance as f64 / 10.0);
    let black = black_intensity as f64 / 10.0;
    let white = white_intensity as f64 / 10.0;

    for (i, &l) in lum.iter().enumerate() {
        let t = (l / 255.0).clamp(0.0, 1.0);
        let dark_gate = ((thr - t) / thr).clamp(0.0, 1.0);
        let light_gate = ((t - 0.5) / 0.5).clamp(0.0, 1.0);
        let delta = (white * light_gate - black * dark_gate) * 255.0 * 0.8;
        shift_sample(&mut buf.data, n, planes, i, delta);
    }
    Ok(())
}

/// Ink Outlines: fine pen-and-ink lines over the source. `dark_intensity`
/// darkens the shadow side of an edge, `light_intensity` lifts the lit side, so
/// the two components act on separate pixels; `stroke_length` widens the edge
/// response.
///
/// ponytail: a Sobel edge response split by local luma, not the reference's line
/// tracing; the split keeps the dark/light components independent.
pub fn ink_outlines(
    buf: &mut PixelBuffer,
    stroke_length: u8,
    dark_intensity: u8,
    light_intensity: u8,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if !(1..=50).contains(&stroke_length) {
        return Err(FilterError::InvalidParams(format!(
            "ink outlines stroke length {stroke_length} is outside 1..=50"
        )));
    }
    if dark_intensity > 50 {
        return Err(FilterError::InvalidParams(format!(
            "ink outlines dark intensity {dark_intensity} is outside 0..=50"
        )));
    }
    if light_intensity > 50 {
        return Err(FilterError::InvalidParams(format!(
            "ink outlines light intensity {light_intensity} is outside 0..=50"
        )));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let lum = luma_plane(&buf.data, n);
    let edge = box_mean(
        &edge_field(&lum, w, h),
        w,
        h,
        (stroke_length as usize / 10).max(1),
    );
    let dark = dark_intensity as f64 / 50.0;
    let light = light_intensity as f64 / 50.0;

    for (i, &l) in lum.iter().enumerate() {
        let t = (l / 255.0).clamp(0.0, 1.0);
        let e = (edge[i] / EDGE_MAX).clamp(0.0, 1.0);
        let delta = (light * t - dark * (1.0 - t)) * e * 200.0;
        shift_sample(&mut buf.data, n, planes, i, delta);
    }
    Ok(())
}

/// Spatter: seeded airbrush scatter. Each grid cell stamps a flat-colored disc
/// of random radius up to `spray_radius`, its color sampled from a scattered
/// source pixel; `smoothness` box-merges the spots.
///
/// ponytail: flat discs over a per-pixel jitter field, not the reference's ink
/// atomization; disc radius grows with `spray_radius` so coverage widens.
pub fn spatter(
    buf: &mut PixelBuffer,
    spray_radius: u8,
    smoothness: u8,
    seed: u64,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if spray_radius > 25 {
        return Err(FilterError::InvalidParams(format!(
            "spatter spray radius {spray_radius} is outside 0..=25"
        )));
    }
    if !(1..=15).contains(&smoothness) {
        return Err(FilterError::InvalidParams(format!(
            "spatter smoothness {smoothness} is outside 1..=15"
        )));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let src = buf.data.clone();
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let r = spray_radius as f64;
    let step = (spray_radius as usize).max(3);
    let mottle = noise::value_noise(w, h, seed ^ 0x9E37_79B9_7F4A_7C15);

    for cy in (0..h).step_by(step) {
        for cx in (0..w).step_by(step) {
            let jx = unit_f64(&mut rng) * step as f64;
            let jy = unit_f64(&mut rng) * step as f64;
            let cxr = (cx as f64 + jx).clamp(0.0, (w - 1) as f64);
            let cyr = (cy as f64 + jy).clamp(0.0, (h - 1) as f64);
            let radius = 0.5 + r * (0.5 + 0.5 * unit_f64(&mut rng));
            let ox = (unit_f64(&mut rng) * 2.0 - 1.0) * (r + 1.0);
            let oy = (unit_f64(&mut rng) * 2.0 - 1.0) * (r + 1.0);
            let sx = clamp_index((cxr + ox) as isize, w);
            let sy = clamp_index((cyr + oy) as isize, h);
            let q = sy * w + sx;
            let jitter = (mottle[q] - 0.5) * 40.0;
            let color = [
                reduce::clamp_u8(src[q] as f64 + jitter),
                reduce::clamp_u8(src[n + q] as f64 + jitter),
                reduce::clamp_u8(src[2 * n + q] as f64 + jitter),
            ];
            paint_disc(&mut buf.data, w, h, planes, cxr, cyr, radius, color);
        }
    }

    let sr = (smoothness as usize / 3).max(1);
    for c in 0..planes {
        let base = c * n;
        let field: Vec<f64> = buf.data[base..base + n].iter().map(|&v| v as f64).collect();
        let means = box_mean(&field, w, h, sr);
        for (i, &m) in means.iter().enumerate() {
            buf.data[base + i] = reduce::clamp_u8(field[i] + (m - field[i]) * 0.6);
        }
    }
    Ok(())
}

/// Sprayed Strokes: seeded short strokes along `direction`, each surrounded by
/// scattered spray dots; `spray_radius` sets both the scatter distance and the
/// dot count, so a larger radius spreads the effect wider.
///
/// ponytail: flat line segments plus uniform dots, not the reference's dominant-color
/// sprayed bristles; the direction picks one of four integer vectors.
pub fn sprayed_strokes(
    buf: &mut PixelBuffer,
    stroke_length: u8,
    spray_radius: u8,
    direction: StrokeDirection,
    seed: u64,
) -> Result<(), FilterError> {
    validate(buf)?;
    if stroke_length > 20 {
        return Err(FilterError::InvalidParams(format!(
            "sprayed strokes stroke length {stroke_length} is outside 0..=20"
        )));
    }
    if spray_radius > 25 {
        return Err(FilterError::InvalidParams(format!(
            "sprayed strokes spray radius {spray_radius} is outside 0..=25"
        )));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let n = w * h;
    let src = buf.data.clone();
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let (dx, dy): (isize, isize) = match direction {
        StrokeDirection::RightDiagonal => (1, 1),
        StrokeDirection::Horizontal => (1, 0),
        StrokeDirection::LeftDiagonal => (1, -1),
        StrokeDirection::Vertical => (0, 1),
    };
    let len = stroke_length as isize;
    let r = spray_radius as f64;
    let step = (stroke_length as usize / 2).max(2);
    let dots = spray_radius as usize / 3 + 1;

    for cy in (0..h).step_by(step) {
        for cx in (0..w).step_by(step) {
            let jx = unit_f64(&mut rng) * step as f64;
            let jy = unit_f64(&mut rng) * step as f64;
            let cxr = (cx as f64 + jx).clamp(0.0, (w - 1) as f64);
            let cyr = (cy as f64 + jy).clamp(0.0, (h - 1) as f64);
            let ox = (unit_f64(&mut rng) * 2.0 - 1.0) * r;
            let oy = (unit_f64(&mut rng) * 2.0 - 1.0) * r;
            let sx = clamp_index((cxr + ox) as isize, w);
            let sy = clamp_index((cyr + oy) as isize, h);
            let q = sy * w + sx;
            let color = [src[q], src[n + q], src[2 * n + q]];

            for t in -len..=len {
                let px = clamp_index(cxr as isize + t * dx, w);
                let py = clamp_index(cyr as isize + t * dy, h);
                let idx = py * w + px;
                for (c, &col) in color.iter().enumerate() {
                    buf.data[c * n + idx] = col;
                }
            }

            for _ in 0..dots {
                let ang = unit_f64(&mut rng) * std::f64::consts::TAU;
                let rr = unit_f64(&mut rng) * r;
                let px = clamp_index(cxr as isize + (rr * ang.cos()) as isize, w);
                let py = clamp_index(cyr as isize + (rr * ang.sin()) as isize, h);
                let idx = py * w + px;
                for (c, &col) in color.iter().enumerate() {
                    buf.data[c * n + idx] = col;
                }
            }
        }
    }
    Ok(())
}

/// Sumi-e: a box-softened luma drives wide ink coverage on a lightened ground,
/// with `stroke_pressure` deepening the ink and `contrast` steepening the tone.
///
/// ponytail: monochrome ink coverage from a smoothed luma, not the reference's brush
/// dynamics; `stroke_width` is the smoothing radius.
pub fn sumi_e(
    buf: &mut PixelBuffer,
    stroke_width: u8,
    stroke_pressure: u8,
    contrast: u8,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if !(3..=15).contains(&stroke_width) {
        return Err(FilterError::InvalidParams(format!(
            "sumi-e stroke width {stroke_width} is outside 3..=15"
        )));
    }
    if stroke_pressure > 15 {
        return Err(FilterError::InvalidParams(format!(
            "sumi-e stroke pressure {stroke_pressure} is outside 0..=15"
        )));
    }
    if contrast > 40 {
        return Err(FilterError::InvalidParams(format!(
            "sumi-e contrast {contrast} is outside 0..=40"
        )));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let soft = box_mean(&luma_plane(&buf.data, n), w, h, stroke_width as usize / 2);
    let pressure = stroke_pressure as f64 / 15.0;
    let gain = 1.0 + contrast as f64 / 40.0 * 1.5;

    for (i, &s) in soft.iter().enumerate() {
        let t = (s / 255.0).clamp(0.0, 1.0);
        let paper = 0.55 + 0.45 * t;
        let ink = ((1.0 - t) * (0.35 + 0.65 * pressure)).clamp(0.0, 1.0);
        let v = ((paper * (1.0 - ink) - 0.5) * gain + 0.5) * 255.0;
        let out = reduce::clamp_u8(v);
        for c in 0..planes {
            buf.data[c * n + i] = out;
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

    fn distinct(plane: &[u8]) -> usize {
        plane
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    }

    fn brush_filters(seed: u64) -> Vec<Filter> {
        vec![
            Filter::AccentedEdges {
                edge_width: 2,
                edge_brightness: 0,
                smoothness: 5,
            },
            Filter::AngledStrokes {
                direction_balance: 50,
                stroke_length: 15,
                sharpness: 5,
            },
            Filter::Crosshatch {
                stroke_length: 9,
                sharpness: 6,
                strength: 2,
            },
            Filter::DarkStrokes {
                balance: 5,
                black_intensity: 6,
                white_intensity: 5,
            },
            Filter::InkOutlines {
                stroke_length: 10,
                dark_intensity: 25,
                light_intensity: 25,
            },
            Filter::Spatter {
                spray_radius: 8,
                smoothness: 5,
                seed,
            },
            Filter::SprayedStrokes {
                stroke_length: 10,
                spray_radius: 7,
                direction: StrokeDirection::RightDiagonal,
                seed,
            },
            Filter::SumiE {
                stroke_width: 8,
                stroke_pressure: 5,
                contrast: 20,
            },
        ]
    }

    #[test]
    fn each_filter_changes_the_colour_planes() {
        let base = gradient(32, 8, 3);
        let n = base.pixel_count();
        for filter in brush_filters(3) {
            let mut out = base.clone();
            apply(&filter, &mut out).unwrap();
            assert_ne!(
                out.data[..3 * n],
                base.data[..3 * n],
                "{filter:?} did not change colour"
            );
        }
    }

    #[test]
    fn every_filter_preserves_alpha_via_apply() {
        let base = gradient(16, 6, 4);
        let n = base.pixel_count();
        let expected = alpha_plane(&base);
        for filter in brush_filters(9) {
            let mut out = base.clone();
            apply(&filter, &mut out).unwrap();
            assert_eq!(alpha_plane(&out), expected, "{filter:?} touched alpha");
            assert_ne!(
                out.data[..3 * n],
                base.data[..3 * n],
                "{filter:?} did not change colour"
            );
        }
    }

    #[test]
    fn boundaries_are_accepted_and_out_of_range_rejected() {
        let base = gradient(8, 4, 3);

        assert!(accented_edges(&mut base.clone(), 1, 0, 1).is_ok());
        assert!(accented_edges(&mut base.clone(), 14, 50, 15).is_ok());
        for (ew, eb, sm) in [(0u8, 25u8, 5u8), (15, 25, 5), (1, 51, 5), (1, 25, 0)] {
            let mut out = base.clone();
            assert!(matches!(
                accented_edges(&mut out, ew, eb, sm),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected accented edges modified the buffer");
        }

        assert!(angled_strokes(&mut base.clone(), 0, 3, 0).is_ok());
        assert!(angled_strokes(&mut base.clone(), 100, 50, 10).is_ok());
        for (db, sl, sh) in [(101u8, 15u8, 5u8), (50, 2, 5), (50, 51, 5), (50, 15, 11)] {
            let mut out = base.clone();
            assert!(matches!(
                angled_strokes(&mut out, db, sl, sh),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected angled strokes modified the buffer");
        }

        assert!(crosshatch(&mut base.clone(), 3, 0, 1).is_ok());
        assert!(crosshatch(&mut base.clone(), 50, 20, 3).is_ok());
        for sl in [2u8, 51] {
            assert!(matches!(
                crosshatch(&mut base.clone(), sl, 6, 2),
                Err(FilterError::InvalidParams(_))
            ));
        }
        assert!(matches!(
            crosshatch(&mut base.clone(), 9, 21, 2),
            Err(FilterError::InvalidParams(_))
        ));
        for st in [0u8, 4] {
            assert!(matches!(
                crosshatch(&mut base.clone(), 9, 6, st),
                Err(FilterError::InvalidParams(_))
            ));
        }

        assert!(dark_strokes(&mut base.clone(), 0, 0, 0).is_ok());
        assert!(dark_strokes(&mut base.clone(), 10, 10, 10).is_ok());
        for (b, bl, wh) in [(11u8, 5u8, 5u8), (5, 11, 5), (5, 5, 11)] {
            let mut out = base.clone();
            assert!(matches!(
                dark_strokes(&mut out, b, bl, wh),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected dark strokes modified the buffer");
        }

        assert!(ink_outlines(&mut base.clone(), 1, 0, 0).is_ok());
        assert!(ink_outlines(&mut base.clone(), 50, 50, 50).is_ok());
        for (sl, di, li) in [(0u8, 25u8, 25u8), (51, 25, 25), (10, 51, 25), (10, 25, 51)] {
            let mut out = base.clone();
            assert!(matches!(
                ink_outlines(&mut out, sl, di, li),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected ink outlines modified the buffer");
        }

        assert!(spatter(&mut base.clone(), 0, 1, 1).is_ok());
        assert!(spatter(&mut base.clone(), 25, 15, 1).is_ok());
        for (sr, sm) in [(26u8, 5u8), (5, 0), (5, 16)] {
            let mut out = base.clone();
            assert!(matches!(
                spatter(&mut out, sr, sm, 1),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected spatter modified the buffer");
        }

        assert!(sprayed_strokes(&mut base.clone(), 0, 0, StrokeDirection::Horizontal, 1).is_ok());
        assert!(sprayed_strokes(&mut base.clone(), 20, 25, StrokeDirection::Vertical, 1).is_ok());
        for (sl, sr) in [(21u8, 7u8), (10, 26)] {
            let mut out = base.clone();
            assert!(matches!(
                sprayed_strokes(&mut out, sl, sr, StrokeDirection::Horizontal, 1),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected sprayed strokes modified the buffer");
        }

        assert!(sumi_e(&mut base.clone(), 3, 0, 0).is_ok());
        assert!(sumi_e(&mut base.clone(), 15, 15, 40).is_ok());
        for (sw, sp, ct) in [(2u8, 5u8, 20u8), (16, 5, 20), (8, 16, 20), (8, 5, 41)] {
            let mut out = base.clone();
            assert!(matches!(
                sumi_e(&mut out, sw, sp, ct),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected sumi-e modified the buffer");
        }
    }

    #[test]
    fn stochastic_filters_are_seed_deterministic() {
        let base = gradient(32, 8, 3);

        let (mut sa, mut sb, mut sc) = (base.clone(), base.clone(), base.clone());
        spatter(&mut sa, 12, 5, 7).unwrap();
        spatter(&mut sb, 12, 5, 7).unwrap();
        spatter(&mut sc, 12, 5, 8).unwrap();
        assert_eq!(sa.data, sb.data, "spatter same seed must be bit-identical");
        assert_ne!(sa.data, sc.data, "spatter different seed must differ");

        let (mut xa, mut xb, mut xc) = (base.clone(), base.clone(), base.clone());
        sprayed_strokes(&mut xa, 10, 7, StrokeDirection::RightDiagonal, 7).unwrap();
        sprayed_strokes(&mut xb, 10, 7, StrokeDirection::RightDiagonal, 7).unwrap();
        sprayed_strokes(&mut xc, 10, 7, StrokeDirection::RightDiagonal, 8).unwrap();
        assert_eq!(
            xa.data, xb.data,
            "sprayed strokes same seed must be bit-identical"
        );
        assert_ne!(
            xa.data, xc.data,
            "sprayed strokes different seed must differ"
        );
    }

    #[test]
    fn accented_edges_brightness_polarity_and_neutrality() {
        let base = gradient(32, 8, 3);
        let n = base.pixel_count();
        let sum = |b: &PixelBuffer| b.data[..3 * n].iter().map(|&v| v as u64).sum::<u64>();

        let mut neutral = base.clone();
        accented_edges(&mut neutral, 2, 25, 5).unwrap();
        assert_eq!(neutral.data, base.data, "brightness 25 must be neutral");

        let mut dark = base.clone();
        accented_edges(&mut dark, 2, 0, 5).unwrap();
        let mut light = base.clone();
        accented_edges(&mut light, 2, 50, 5).unwrap();
        assert!(
            sum(&dark) < sum(&base),
            "brightness 0 ({}) must darken below input ({})",
            sum(&dark),
            sum(&base)
        );
        assert!(
            sum(&light) > sum(&base),
            "brightness 50 ({}) must lighten above input ({})",
            sum(&light),
            sum(&base)
        );
    }

    #[test]
    fn crosshatch_strength_and_spatter_radius_change_coverage() {
        let base = gradient(32, 8, 3);
        let n = base.pixel_count();

        let mut one = base.clone();
        crosshatch(&mut one, 9, 6, 1).unwrap();
        let mut three = base.clone();
        crosshatch(&mut three, 9, 6, 3).unwrap();
        assert_ne!(one.data, three.data, "strength 1 vs 3 must differ");
        assert!(
            distinct(&one.data[..n]) > 1,
            "crosshatch must preserve tonal detail"
        );

        let changed = |b: &PixelBuffer| {
            b.data
                .iter()
                .zip(&base.data)
                .filter(|(a, c)| a != c)
                .count()
        };
        let mut r0 = base.clone();
        spatter(&mut r0, 0, 5, 3).unwrap();
        let mut r25 = base.clone();
        spatter(&mut r25, 25, 5, 3).unwrap();
        assert!(changed(&r0) > 0, "radius 0 must still scatter");
        assert!(
            changed(&r25) > changed(&r0),
            "radius 25 ({}) must spread wider than radius 0 ({})",
            changed(&r25),
            changed(&r0)
        );
    }

    #[test]
    fn sprayed_strokes_direction_changes_the_result() {
        let base = gradient(32, 8, 3);
        let mut right = base.clone();
        sprayed_strokes(&mut right, 10, 7, StrokeDirection::RightDiagonal, 4).unwrap();
        let mut horiz = base.clone();
        sprayed_strokes(&mut horiz, 10, 7, StrokeDirection::Horizontal, 4).unwrap();
        assert_ne!(right.data, horiz.data, "direction must change the result");
    }
}
