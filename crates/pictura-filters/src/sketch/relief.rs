//! Sketch relief filters (`m25-filter-families`): Bas Relief, Chalk &
//! Charcoal, Charcoal, Chrome, Conté Crayon, Graphic Pen and Halftone Pattern.
//!
//! Behavioural models only: the reference's kernels are closed. Each deliberate
//! shortcut carries a `ponytail:` note.

use pictura_core::PixelBuffer;

use crate::artistic::noise;
use crate::artistic::reduce;
use crate::artistic::texture::{emboss, surface_height, texture_options_valid};
use crate::kernel::{box_mean, clamp_index};
use crate::luma::luma_plane;
use crate::{validate, FilterError, HalftoneType, LightDirection, StrokeDirection, TextureOptions};

/// Bas Relief: luminance as a height field, lit by the shared directional
/// gradient. Recessed (dark) response lerps toward `foreground`, raised (light)
/// response toward `background`.
///
/// ponytail: the signed emboss is added to the tonal term and the result is a
/// per-channel fg→bg lerp, not the reference's true low-relief lighting. Swap in
/// normal-based directional shading if the relief needs real specular falloff.
pub fn bas_relief(
    buf: &mut PixelBuffer,
    detail: u8,
    smoothness: u8,
    light_direction: LightDirection,
    foreground: [u8; 3],
    background: [u8; 3],
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if !(1..=15).contains(&detail) {
        return Err(FilterError::InvalidParams(format!(
            "bas relief detail {detail} is outside 1..=15"
        )));
    }
    if !(1..=15).contains(&smoothness) {
        return Err(FilterError::InvalidParams(format!(
            "bas relief smoothness {smoothness} is outside 1..=15"
        )));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let field = luma_plane(&buf.data, n);
    let smooth = box_mean(&field, w, h, smoothness as usize / 3);
    let scale = detail as f64 / 255.0;

    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let lit = emboss(
                |hx, hy| {
                    smooth[clamp_index(hy as isize, h) * w + clamp_index(hx as isize, w)] / 255.0
                },
                x,
                y,
                light_direction as u8,
                detail,
                false,
            );
            let t = (smooth[i] / 255.0 + lit * scale).clamp(0.0, 1.0);
            for c in 0..planes {
                let dark = foreground[c] as f64;
                let light = background[c] as f64;
                buf.data[c * n + i] = reduce::clamp_u8(dark + (light - dark) * t);
            }
        }
    }
    Ok(())
}

/// Chalk & Charcoal: coarse chalk over a solid midtone ground with diagonal
/// charcoal shadows. `charcoal_area` drives shadow coverage, `chalk_area` the
/// light/midtone coverage, and `stroke_pressure` darkens the charcoal lines.
///
/// ponytail: a seeded noise jitter over a rotated band mask, not the reference's chalk
/// rub. A second, finer grain octave is the upgrade path if it reads too even.
pub fn chalk_charcoal(
    buf: &mut PixelBuffer,
    charcoal_area: u8,
    chalk_area: u8,
    stroke_pressure: u8,
    foreground: [u8; 3],
    background: [u8; 3],
    seed: u64,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if charcoal_area > 20 {
        return Err(FilterError::InvalidParams(format!(
            "chalk & charcoal area {charcoal_area} is outside 0..=20"
        )));
    }
    if chalk_area > 20 {
        return Err(FilterError::InvalidParams(format!(
            "chalk & charcoal chalk area {chalk_area} is outside 0..=20"
        )));
    }
    if stroke_pressure > 5 {
        return Err(FilterError::InvalidParams(format!(
            "chalk & charcoal stroke pressure {stroke_pressure} is outside 0..=5"
        )));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let field = luma_plane(&buf.data, n);
    let grain = noise::value_noise(w, h, seed);
    let charcoal = charcoal_area as f64 / 20.0;
    let chalk = chalk_area as f64 / 20.0;
    let pressure = stroke_pressure as f64 / 5.0;
    let period = 6usize;

    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let t = (field[i] / 255.0).clamp(0.0, 1.0);
            let chalk_gain = (t * chalk * (0.7 + 0.6 * grain[i])).clamp(0.0, 1.0);
            let band = ((x + y) % period) as f64 + (grain[i] - 0.5);
            let on_line = band < 1.0 + pressure * 2.0;
            let charcoal_gain =
                ((1.0 - t) * charcoal * (1.0 + pressure) * on_line as u8 as f64).clamp(0.0, 1.0);
            for c in 0..planes {
                let mut v = 128.0;
                v += (background[c] as f64 - v) * chalk_gain;
                v += (foreground[c] as f64 - v) * charcoal_gain;
                buf.data[c * n + i] = reduce::clamp_u8(v);
            }
        }
    }
    Ok(())
}

/// Charcoal: posterized tonal zones crossed by bold Sobel edges and a seeded
/// smudge, rendered as foreground ink on background paper. `thickness` weights
/// the edges, `detail` the posterization, `light_dark_balance` 0 = dark to
/// 100 = light.
///
/// ponytail: posterize + Sobel coverage over a noise jitter, not the reference's smudge
/// brush; `thickness` scales edge weight instead of dilating a stroke field.
pub fn charcoal(
    buf: &mut PixelBuffer,
    thickness: u8,
    detail: u8,
    light_dark_balance: u8,
    foreground: [u8; 3],
    background: [u8; 3],
    seed: u64,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if !(1..=7).contains(&thickness) {
        return Err(FilterError::InvalidParams(format!(
            "charcoal thickness {thickness} is outside 1..=7"
        )));
    }
    if detail > 5 {
        return Err(FilterError::InvalidParams(format!(
            "charcoal detail {detail} is outside 0..=5"
        )));
    }
    if light_dark_balance > 100 {
        return Err(FilterError::InvalidParams(format!(
            "charcoal light/dark balance {light_dark_balance} is outside 0..=100"
        )));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let field = luma_plane(&buf.data, n);
    let grain = noise::value_noise(w, h, seed);
    let levels = 2 + detail;
    let darkness = (100 - light_dark_balance) as f64 / 100.0;
    let edge_gain = 0.35 + thickness as f64 / 7.0 * 0.65;
    let luma_at = |ax: isize, ay: isize| field[clamp_index(ay, h) * w + clamp_index(ax, w)];

    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let t = reduce::posterize(reduce::clamp_u8(field[i]), levels) as f64 / 255.0;
            let edge = (reduce::edge_magnitude(luma_at, x, y) / 1020.0).clamp(0.0, 1.0);
            let ink =
                ((1.0 - t) * (0.35 + 0.65 * darkness) + edge * edge_gain + (grain[i] - 0.5) * 0.2)
                    .clamp(0.0, 1.0);
            for c in 0..planes {
                let paper = background[c] as f64;
                let pen = foreground[c] as f64;
                buf.data[c * n + i] = reduce::clamp_u8(paper + (pen - paper) * ink);
            }
        }
    }
    Ok(())
}

/// Chrome: polished-surface highlights/shadows. The smoothed luma gradient
/// drives a low-contrast specular ramp and `detail` adds a high-pass sharpen.
///
/// ponytail: a low-contrast ramp plus a detail high-pass, not the reference's polished
/// metal mapping. Add a Levels-style contrast pass if it reads too flat.
pub fn chrome(buf: &mut PixelBuffer, detail: u8, smoothness: u8) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if detail > 10 {
        return Err(FilterError::InvalidParams(format!(
            "chrome detail {detail} is outside 0..=10"
        )));
    }
    if smoothness > 10 {
        return Err(FilterError::InvalidParams(format!(
            "chrome smoothness {smoothness} is outside 0..=10"
        )));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let field = luma_plane(&buf.data, n);
    let smooth = box_mean(&field, w, h, smoothness as usize / 2);
    let sharp = 0.3 + detail as f64 / 10.0 * 0.9;

    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let xl = clamp_index(x as isize - 1, w);
            let xr = clamp_index(x as isize + 1, w);
            let yu = clamp_index(y as isize - 1, h);
            let yd = clamp_index(y as isize + 1, h);
            let gx = smooth[y * w + xr] - smooth[y * w + xl];
            let gy = smooth[yd * w + x] - smooth[yu * w + x];
            let spec = -(gx + gy) * 0.05;
            let hf = field[i] - smooth[i];
            let v = reduce::clamp_u8(128.0 + (smooth[i] - 128.0) * 0.45 + hf * sharp + spec);
            for c in 0..planes {
                buf.data[c * n + i] = v;
            }
        }
    }
    Ok(())
}

/// Conté Crayon: foreground/background level emphasis over a lit procedural
/// surface (`TextureOptions`), with a seeded crayon grain.
///
/// ponytail: per-pixel lerp between the two ink colours plus the shared emboss
/// surface, not the reference's dense crayon stroke. A bristle stamp is the upgrade path
/// if the texture reads too fine.
pub fn conte_crayon(
    buf: &mut PixelBuffer,
    foreground_level: u8,
    background_level: u8,
    texture: TextureOptions,
    foreground: [u8; 3],
    background: [u8; 3],
    seed: u64,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if !(1..=15).contains(&foreground_level) {
        return Err(FilterError::InvalidParams(format!(
            "conte crayon foreground level {foreground_level} is outside 1..=15"
        )));
    }
    if !(1..=15).contains(&background_level) {
        return Err(FilterError::InvalidParams(format!(
            "conte crayon background level {background_level} is outside 1..=15"
        )));
    }
    if !texture_options_valid(&texture) {
        return Err(FilterError::InvalidParams(
            "conte crayon texture options are out of range".into(),
        ));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let field = luma_plane(&buf.data, n);
    let grain = noise::value_noise(w, h, seed);
    let fg_w = foreground_level as f64 / 15.0;
    let bg_w = background_level as f64 / 15.0;

    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let t = (field[i] / 255.0).clamp(0.0, 1.0);
            let shade = emboss(
                |hx, hy| surface_height(texture.surface, hx, hy, texture.scaling),
                x,
                y,
                texture.light_direction,
                texture.relief,
                texture.invert,
            );
            let surf = surface_height(texture.surface, x, y, texture.scaling) - 0.5;
            let dark = ((1.0 - t) * fg_w * 0.3).clamp(0.0, 1.0);
            let light = (t * bg_w * 0.3).clamp(0.0, 1.0);
            let tex = shade * 0.5 + surf + (grain[i] - 0.5) * 0.3;
            let m = (t + light - dark + tex * 0.3).clamp(0.0, 1.0);
            for c in 0..planes {
                let dark_ink = foreground[c] as f64;
                let light_ink = background[c] as f64;
                buf.data[c * n + i] = reduce::clamp_u8(dark_ink + (light_ink - dark_ink) * m);
            }
        }
    }
    Ok(())
}

/// Graphic Pen: fine directional ink strokes. The luma is smeared along
/// `direction`, thresholded by `light_dark_balance` (0 = dark, 100 = light) and
/// gated onto parallel lines whose density grows with `stroke_length`.
///
/// ponytail: a luma smear on one-pixel lines, not the reference's nib model. Widen with
/// a brush footprint if the strokes read too thin.
pub fn graphic_pen(
    buf: &mut PixelBuffer,
    stroke_length: u8,
    light_dark_balance: u8,
    direction: StrokeDirection,
    foreground: [u8; 3],
    background: [u8; 3],
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if !(1..=15).contains(&stroke_length) {
        return Err(FilterError::InvalidParams(format!(
            "graphic pen stroke length {stroke_length} is outside 1..=15"
        )));
    }
    if light_dark_balance > 100 {
        return Err(FilterError::InvalidParams(format!(
            "graphic pen light/dark balance {light_dark_balance} is outside 0..=100"
        )));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let field = luma_plane(&buf.data, n);
    let (dx, dy): (isize, isize) = match direction {
        StrokeDirection::RightDiagonal => (1, 1),
        StrokeDirection::Horizontal => (1, 0),
        StrokeDirection::LeftDiagonal => (1, -1),
        StrokeDirection::Vertical => (0, 1),
    };
    let len = stroke_length as isize;
    let bias = (100 - light_dark_balance) as f64 / 100.0;
    let threshold = (0.18 + 0.64 * bias) * 255.0;
    let spacing = ((18 - stroke_length as usize) / 2).max(2);

    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let mut acc = 0.0;
            for t in -len..=len {
                let sx = clamp_index(x as isize + t * dx, w);
                let sy = clamp_index(y as isize + t * dy, h);
                acc += field[sy * w + sx];
            }
            let avg = acc / (2 * len + 1) as f64;
            let perp: isize = match direction {
                StrokeDirection::RightDiagonal => x as isize - y as isize,
                StrokeDirection::LeftDiagonal => x as isize + y as isize,
                StrokeDirection::Horizontal => y as isize,
                StrokeDirection::Vertical => x as isize,
            };
            let on_line = perp.rem_euclid(spacing as isize) == 0;
            let ink = if on_line && avg < threshold { 1.0 } else { 0.0 };
            for c in 0..planes {
                let paper = background[c] as f64;
                let pen = foreground[c] as f64;
                buf.data[c * n + i] = reduce::clamp_u8(paper + (pen - paper) * ink);
            }
        }
    }
    Ok(())
}

/// Halftone Pattern: a 45-degree screen whose Dot/Circle/Line cells threshold
/// the luma. `size` sets the cell edge, `contrast` the screen gain.
///
/// ponytail: three distance kernels on an axis-aligned rotated grid; the CS6
/// screen angle/curve is undocumented. Add an angle control if reference
/// renders need one.
pub fn halftone_pattern(
    buf: &mut PixelBuffer,
    size: u8,
    contrast: u8,
    pattern: HalftoneType,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if !(1..=12).contains(&size) {
        return Err(FilterError::InvalidParams(format!(
            "halftone pattern size {size} is outside 1..=12"
        )));
    }
    if contrast > 50 {
        return Err(FilterError::InvalidParams(format!(
            "halftone pattern contrast {contrast} is outside 0..=50"
        )));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let field = luma_plane(&buf.data, n);
    let cell = size as f64;
    let angle = std::f64::consts::FRAC_PI_4;
    let (sin, cos) = angle.sin_cos();
    let gain = 1.0 + contrast as f64 / 12.5;

    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let fx = x as f64 + 0.5;
            let fy = y as f64 + 0.5;
            let u = fx * cos - fy * sin;
            let v = fx * sin + fy * cos;
            let nx = (u.rem_euclid(cell) / cell) * 2.0 - 1.0;
            let ny = (v.rem_euclid(cell) / cell) * 2.0 - 1.0;
            let d = match pattern {
                HalftoneType::Dot => (nx * nx + ny * ny).sqrt(),
                HalftoneType::Circle => ((nx * nx + ny * ny).sqrt() - 0.55).abs() * 2.2,
                HalftoneType::Line => ny.abs(),
            };
            let l = (field[i] / 255.0).clamp(0.0, 1.0);
            let coverage = (((1.0 - l) - 0.5) * gain + 0.5).clamp(0.0, 1.0);
            let out = reduce::clamp_u8(if d <= coverage * 1.5 { 0.0 } else { 255.0 });
            for c in 0..planes {
                buf.data[c * n + i] = out;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{apply, Filter, TextureSurface};

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

    fn flat(w: u32, h: u32, value: u8) -> PixelBuffer {
        let n = (w * h) as usize;
        PixelBuffer {
            width: w,
            height: h,
            channels: 3,
            data: vec![value; n * 3],
        }
    }

    fn planes_of(buf: &PixelBuffer, n: usize) -> usize {
        (buf.channels as usize).min(3) * n
    }

    fn alpha_plane(buf: &PixelBuffer) -> Vec<u8> {
        let n = buf.pixel_count();
        buf.data[3 * n..4 * n].to_vec()
    }

    const FG: [u8; 3] = [10, 10, 10];
    const BG: [u8; 3] = [240, 240, 240];

    fn sketch_cases(seed: u64) -> Vec<Filter> {
        vec![
            Filter::BasRelief {
                detail: 6,
                smoothness: 3,
                light_direction: LightDirection::Top,
                foreground: FG,
                background: BG,
            },
            Filter::ChalkCharcoal {
                charcoal_area: 6,
                chalk_area: 6,
                stroke_pressure: 1,
                foreground: FG,
                background: BG,
                seed,
            },
            Filter::Charcoal {
                thickness: 3,
                detail: 3,
                light_dark_balance: 50,
                foreground: FG,
                background: BG,
                seed,
            },
            Filter::Chrome {
                detail: 4,
                smoothness: 7,
            },
            Filter::ConteCrayon {
                foreground_level: 8,
                background_level: 7,
                texture: TextureOptions::default(),
                foreground: FG,
                background: BG,
                seed,
            },
            Filter::GraphicPen {
                stroke_length: 6,
                light_dark_balance: 50,
                direction: StrokeDirection::RightDiagonal,
                foreground: FG,
                background: BG,
            },
            Filter::HalftonePattern {
                size: 5,
                contrast: 5,
                pattern: HalftoneType::Dot,
            },
        ]
    }

    #[test]
    fn each_filter_changes_the_colour_planes() {
        let base = gradient(32, 8, 3);
        let n = base.pixel_count();
        for filter in sketch_cases(3) {
            let mut out = base.clone();
            apply(&filter, &mut out).unwrap();
            assert_ne!(
                out.data[..planes_of(&out, n)],
                base.data[..planes_of(&base, n)],
                "{filter:?} did not change colour"
            );
        }
    }

    #[test]
    fn every_filter_preserves_alpha_via_apply() {
        let base = gradient(16, 6, 4);
        let expected = alpha_plane(&base);
        for filter in sketch_cases(9) {
            let mut out = base.clone();
            apply(&filter, &mut out).unwrap();
            assert_eq!(alpha_plane(&out), expected, "{filter:?} touched alpha");
        }
    }

    #[test]
    fn boundaries_accept_and_out_of_range_rejects() {
        let base = gradient(8, 4, 3);
        let opts = TextureOptions::default();

        assert!(bas_relief(&mut base.clone(), 1, 1, LightDirection::Bottom, FG, BG).is_ok());
        assert!(bas_relief(
            &mut base.clone(),
            15,
            15,
            LightDirection::BottomRight,
            FG,
            BG
        )
        .is_ok());
        for (d, s) in [(0u8, 1u8), (16, 1), (1, 0), (1, 16)] {
            let mut out = base.clone();
            assert!(matches!(
                bas_relief(&mut out, d, s, LightDirection::Top, FG, BG),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected bas relief modified the buffer");
        }

        assert!(chalk_charcoal(&mut base.clone(), 0, 0, 0, FG, BG, 1).is_ok());
        assert!(chalk_charcoal(&mut base.clone(), 20, 20, 5, FG, BG, 1).is_ok());
        for (ca, cha, sp) in [(21u8, 0u8, 0u8), (0, 21, 0), (0, 0, 6)] {
            let mut out = base.clone();
            assert!(matches!(
                chalk_charcoal(&mut out, ca, cha, sp, FG, BG, 1),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected chalk & charcoal modified the buffer");
        }

        assert!(charcoal(&mut base.clone(), 1, 0, 0, FG, BG, 1).is_ok());
        assert!(charcoal(&mut base.clone(), 7, 5, 100, FG, BG, 1).is_ok());
        for (t, d, b) in [(0u8, 0u8, 0u8), (8, 0, 0), (1, 6, 0), (1, 0, 101)] {
            let mut out = base.clone();
            assert!(matches!(
                charcoal(&mut out, t, d, b, FG, BG, 1),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected charcoal modified the buffer");
        }

        assert!(chrome(&mut base.clone(), 0, 0).is_ok());
        assert!(chrome(&mut base.clone(), 10, 10).is_ok());
        for (d, s) in [(11u8, 0u8), (0, 11)] {
            let mut out = base.clone();
            assert!(matches!(
                chrome(&mut out, d, s),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected chrome modified the buffer");
        }

        assert!(conte_crayon(&mut base.clone(), 1, 1, opts, FG, BG, 1).is_ok());
        assert!(conte_crayon(&mut base.clone(), 15, 15, opts, FG, BG, 1).is_ok());
        for (f, b) in [(0u8, 1u8), (16, 1), (1, 0), (1, 16)] {
            let mut out = base.clone();
            assert!(matches!(
                conte_crayon(&mut out, f, b, opts, FG, BG, 1),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected conte crayon modified the buffer");
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
                conte_crayon(&mut out, 8, 7, bad, FG, BG, 1),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected conte texture modified the buffer");
        }

        assert!(graphic_pen(&mut base.clone(), 1, 0, StrokeDirection::Horizontal, FG, BG).is_ok());
        assert!(graphic_pen(
            &mut base.clone(),
            15,
            100,
            StrokeDirection::Vertical,
            FG,
            BG
        )
        .is_ok());
        for (l, b) in [(0u8, 0u8), (16, 0), (1, 101)] {
            let mut out = base.clone();
            assert!(matches!(
                graphic_pen(&mut out, l, b, StrokeDirection::Horizontal, FG, BG),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected graphic pen modified the buffer");
        }

        assert!(halftone_pattern(&mut base.clone(), 1, 0, HalftoneType::Dot).is_ok());
        assert!(halftone_pattern(&mut base.clone(), 12, 50, HalftoneType::Circle).is_ok());
        for (sz, c) in [(0u8, 0u8), (13, 0), (1, 51)] {
            let mut out = base.clone();
            assert!(matches!(
                halftone_pattern(&mut out, sz, c, HalftoneType::Dot),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected halftone pattern modified the buffer");
        }
    }

    #[test]
    fn seeded_filters_are_deterministic() {
        let base = gradient(32, 8, 3);
        let opts = TextureOptions::default();

        let (mut a, mut b, mut c) = (base.clone(), base.clone(), base.clone());
        chalk_charcoal(&mut a, 6, 6, 1, FG, BG, 7).unwrap();
        chalk_charcoal(&mut b, 6, 6, 1, FG, BG, 7).unwrap();
        chalk_charcoal(&mut c, 6, 6, 1, FG, BG, 8).unwrap();
        assert_eq!(a.data, b.data, "chalk & charcoal same seed must match");
        assert_ne!(
            a.data, c.data,
            "chalk & charcoal different seed must differ"
        );

        let (mut d, mut e, mut f) = (base.clone(), base.clone(), base.clone());
        charcoal(&mut d, 3, 3, 50, FG, BG, 7).unwrap();
        charcoal(&mut e, 3, 3, 50, FG, BG, 7).unwrap();
        charcoal(&mut f, 3, 3, 50, FG, BG, 8).unwrap();
        assert_eq!(d.data, e.data, "charcoal same seed must match");
        assert_ne!(d.data, f.data, "charcoal different seed must differ");

        let (mut g, mut hh, mut j) = (base.clone(), base.clone(), base.clone());
        conte_crayon(&mut g, 8, 7, opts, FG, BG, 7).unwrap();
        conte_crayon(&mut hh, 8, 7, opts, FG, BG, 7).unwrap();
        conte_crayon(&mut j, 8, 7, opts, FG, BG, 8).unwrap();
        assert_eq!(g.data, hh.data, "conte crayon same seed must match");
        assert_ne!(g.data, j.data, "conte crayon different seed must differ");
    }

    #[test]
    fn bas_relief_tone_mapping_light_direction_and_flat_input() {
        let (w, h) = (32u32, 8u32);
        let n = (w * h) as usize;
        let mut data = vec![0u8; n * 3];
        for y in 0..h as usize {
            for x in 0..w as usize {
                let v = if x < (w / 2) as usize { 40 } else { 210 };
                for c in 0..3 {
                    data[c * n + y * w as usize + x] = v;
                }
            }
        }
        let step = PixelBuffer {
            width: w,
            height: h,
            channels: 3,
            data,
        };
        let half = (w / 2) as usize;
        let mean = |buf: &PixelBuffer, plane: usize, from: usize, to: usize| -> f64 {
            let mut s = 0.0;
            let mut count = 0.0;
            for y in 0..h as usize {
                for x in from..to {
                    s += buf.data[plane * n + y * w as usize + x] as f64;
                    count += 1.0;
                }
            }
            s / count
        };

        let fg = [200, 10, 10];
        let bg = [10, 10, 200];
        let mut out = step.clone();
        bas_relief(&mut out, 6, 3, LightDirection::Left, fg, bg).unwrap();
        let dark_r = mean(&out, 0, 0, half);
        let light_r = mean(&out, 0, half, w as usize);
        let dark_b = mean(&out, 2, 0, half);
        let light_b = mean(&out, 2, half, w as usize);
        assert!(
            dark_r > light_r,
            "dark side must take the foreground red (dark R {dark_r}, light R {light_r})"
        );
        assert!(
            dark_b < light_b,
            "light side must take the background blue (dark B {dark_b}, light B {light_b})"
        );

        let mut other = step.clone();
        bas_relief(&mut other, 6, 3, LightDirection::Top, fg, bg).unwrap();
        assert_ne!(
            out.data, other.data,
            "light direction must change the result"
        );

        let mut uniform = flat(8, 4, 120);
        assert!(bas_relief(&mut uniform, 6, 3, LightDirection::Top, fg, bg).is_ok());

        let same = [7, 7, 7];
        let mut equal = flat(8, 4, 120);
        assert!(bas_relief(&mut equal, 6, 3, LightDirection::Top, same, same).is_ok());
    }

    #[test]
    fn conte_crayon_texture_options_change_the_result() {
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

        let (mut c, mut b, mut l) = (base.clone(), base.clone(), base.clone());
        conte_crayon(&mut c, 8, 7, canvas, FG, BG, 4).unwrap();
        conte_crayon(&mut b, 8, 7, brick, FG, BG, 4).unwrap();
        conte_crayon(&mut l, 8, 7, lit, FG, BG, 4).unwrap();
        assert_ne!(c.data, b.data, "surface preset must matter");
        assert_ne!(c.data, l.data, "light direction must matter");
    }

    #[test]
    fn halftone_patterns_and_size_differ() {
        let base = gradient(32, 8, 3);
        let (mut dot, mut line, mut circle) = (base.clone(), base.clone(), base.clone());
        halftone_pattern(&mut dot, 4, 5, HalftoneType::Dot).unwrap();
        halftone_pattern(&mut line, 4, 5, HalftoneType::Line).unwrap();
        halftone_pattern(&mut circle, 4, 5, HalftoneType::Circle).unwrap();
        assert_ne!(dot.data, line.data, "dot vs line must differ");
        assert_ne!(dot.data, circle.data, "dot vs circle must differ");
        assert_ne!(line.data, circle.data, "line vs circle must differ");

        let (mut one, mut twelve) = (base.clone(), base.clone());
        halftone_pattern(&mut one, 1, 5, HalftoneType::Dot).unwrap();
        halftone_pattern(&mut twelve, 12, 5, HalftoneType::Dot).unwrap();
        assert_ne!(one.data, twelve.data, "cell size must change the result");
    }

    #[test]
    fn equal_foreground_and_background_are_safe() {
        let base = gradient(16, 6, 3);
        let same = [7, 7, 7];
        let opts = TextureOptions::default();
        assert!(chalk_charcoal(&mut base.clone(), 6, 6, 1, same, same, 1).is_ok());
        assert!(charcoal(&mut base.clone(), 3, 3, 50, same, same, 1).is_ok());
        assert!(conte_crayon(&mut base.clone(), 8, 7, opts, same, same, 1).is_ok());
        assert!(graphic_pen(
            &mut base.clone(),
            6,
            50,
            StrokeDirection::RightDiagonal,
            same,
            same
        )
        .is_ok());
    }

    #[test]
    fn tiny_buffers_do_not_panic() {
        let tiny = PixelBuffer {
            width: 1,
            height: 1,
            channels: 4,
            data: vec![10, 20, 30, 40],
        };
        let opts = TextureOptions::default();
        assert!(bas_relief(&mut tiny.clone(), 15, 15, LightDirection::Top, FG, BG).is_ok());
        assert!(chalk_charcoal(&mut tiny.clone(), 20, 20, 5, FG, BG, 1).is_ok());
        assert!(charcoal(&mut tiny.clone(), 7, 5, 100, FG, BG, 1).is_ok());
        assert!(chrome(&mut tiny.clone(), 10, 10).is_ok());
        assert!(conte_crayon(&mut tiny.clone(), 15, 15, opts, FG, BG, 1).is_ok());
        assert!(graphic_pen(
            &mut tiny.clone(),
            15,
            100,
            StrokeDirection::Vertical,
            FG,
            BG
        )
        .is_ok());
        assert!(halftone_pattern(&mut tiny.clone(), 12, 50, HalftoneType::Circle).is_ok());
    }
}
