//! Sketch paper filters (`m25-filter-families`): Note Paper, Photocopy, Plaster,
//! Reticulation, Stamp, Torn Edges and Water Paper.
//!
//! Behavioural models only: Adobe's kernels are closed. Each deliberate
//! shortcut carries a `ponytail:` note.

use pictura_core::PixelBuffer;

use crate::artistic::noise;
use crate::artistic::reduce;
use crate::artistic::texture::emboss;
use crate::kernel::clamp_index;
use crate::luma::luma;
use crate::{validate, FilterError, LightDirection};

/// Rec.601 luma plane for a planar 3/4-channel buffer.
fn luma_plane(data: &[u8], n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| luma(data[i] as f64, data[n + i] as f64, data[2 * n + i] as f64))
        .collect()
}

/// Separable box mean with clamp-to-edge, for the smoothness radii.
///
/// ponytail: duplicated from `sketch::relief` (private there) instead of
/// widening `kernel` for seven call sites; move it if a third family needs it.
fn blur_field(src: &[f64], w: usize, h: usize, radius: usize) -> Vec<f64> {
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
    let area = ((2 * radius + 1) * (2 * radius + 1)) as f64;
    let mut out = vec![0.0f64; src.len()];
    for y in 0..h {
        for x in 0..w {
            let mut s = 0.0;
            for d in 0..=2 * radius {
                let sy = clamp_index(y as isize + d as isize - radius as isize, h);
                s += tmp[sy * w + x];
            }
            out[y * w + x] = s / area;
        }
    }
    out
}

/// Integer-hash sample in `0..1`, deterministic from `(seed, x, y)`.
///
/// ponytail: stateless coordinate hash (same model as `artistic::texture`), so
/// the fibre lattice never materializes a canvas-sized field.
fn hash2(seed: u64, x: i64, y: i64) -> f64 {
    let mut h = seed;
    h = h.wrapping_add((x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    h = h.wrapping_add((y as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F));
    h ^= h >> 33;
    h = h.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    h ^= h >> 33;
    h = h.wrapping_mul(0xC4CE_B9FE_1A85_EC53);
    h ^= h >> 33;
    (h >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
}

/// Bilinear-interpolated value noise over the integer lattice.
fn lattice(x: f64, y: f64, seed: u64) -> f64 {
    let (xi, yi) = (x.floor(), y.floor());
    let (xf, yf) = (x - xi, y - yi);
    let u = xf * xf * (3.0 - 2.0 * xf);
    let v = yf * yf * (3.0 - 2.0 * yf);
    let (ix, iy) = (xi as i64, yi as i64);
    let a = hash2(seed, ix, iy);
    let b = hash2(seed, ix + 1, iy);
    let c = hash2(seed, ix, iy + 1);
    let d = hash2(seed, ix + 1, iy + 1);
    let top = a + (b - a) * u;
    let bot = c + (d - c) * u;
    top + (bot - top) * v
}

/// Note Paper: Emboss + Grain. The luminance is a height field lit from the top
/// and perturbed by the seeded surface grain; the result is achromatic paper.
///
/// ponytail: a fixed top light plus `image_balance` tonal bias, not Adobe's
/// foreground/background hole model (this signature carries no fg/bg). Swap in
/// the fg/bg ink mapping if the note-paper holes need to reveal a paper colour.
pub fn note_paper(
    buf: &mut PixelBuffer,
    image_balance: u8,
    graininess: u8,
    relief: u8,
    seed: u64,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if image_balance > 50 {
        return Err(FilterError::InvalidParams(format!(
            "note paper image balance {image_balance} is outside 0..=50"
        )));
    }
    if graininess > 20 {
        return Err(FilterError::InvalidParams(format!(
            "note paper graininess {graininess} is outside 0..=20"
        )));
    }
    if relief > 25 {
        return Err(FilterError::InvalidParams(format!(
            "note paper relief {relief} is outside 0..=25"
        )));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let field = luma_plane(&buf.data, n);
    let grain = noise::value_noise(w, h, seed);
    let balance = (image_balance as f64 / 50.0 - 0.5) * 0.4;
    let grain_amp = graininess as f64 / 20.0;

    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let lit = emboss(
                |hx, hy| {
                    field[clamp_index(hy as isize, h) * w + clamp_index(hx as isize, w)] / 255.0
                },
                x,
                y,
                LightDirection::Top as u8,
                relief,
                false,
            );
            let tone = (field[i] / 255.0 + balance + lit * 0.5).clamp(0.0, 1.0);
            let surface = (grain[i] - 0.5) * grain_amp;
            let v = reduce::clamp_u8((tone + surface) * 255.0);
            for c in 0..planes {
                buf.data[c * n + i] = v;
            }
        }
    }
    Ok(())
}

/// Photocopy: hard black/white threshold where only high-gradient pixels survive
/// as ink, so large dark areas reduce to their edges and midtones collapse to
/// black or white. `darkness` raises the cut and shrinks the dark area; `detail`
/// lowers the edge requirement.
///
/// ponytail: Sobel-gated threshold, not Adobe's halftone/toner simulation. Add
/// a dither screen if the flat areas read too clean.
pub fn photocopy(buf: &mut PixelBuffer, detail: u8, darkness: u8) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if detail > 24 {
        return Err(FilterError::InvalidParams(format!(
            "photocopy detail {detail} is outside 0..=24"
        )));
    }
    if !(1..=50).contains(&darkness) {
        return Err(FilterError::InvalidParams(format!(
            "photocopy darkness {darkness} is outside 1..=50"
        )));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let field = luma_plane(&buf.data, n);
    let luma_at = |ax: isize, ay: isize| field[clamp_index(ay, h) * w + clamp_index(ax, w)];
    let cut = 1.0 - darkness as f64 / 50.0 * 0.85;
    let edge_req = 0.35 - detail as f64 / 24.0 * 0.30;

    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let t = field[i] / 255.0;
            let edge = (reduce::edge_magnitude(luma_at, x, y) / 1020.0).clamp(0.0, 1.0);
            let ink = t < cut && edge >= edge_req;
            let v = if ink { 0u8 } else { 255u8 };
            for c in 0..planes {
                buf.data[c * n + i] = v;
            }
        }
    }
    Ok(())
}

/// Plaster: luminance as a smoothed height field lit in `light_direction`, mapped
/// from recessed `foreground` to raised `background`. `image_balance` shifts the
/// raised/recessed emphasis.
///
/// ponytail: the shared signed-gradient emboss at a fixed relief, not a true
/// surface-normal model. Swap in normal-based shading if plaster needs real
/// cast falloff.
pub fn plaster(
    buf: &mut PixelBuffer,
    image_balance: u8,
    smoothness: u8,
    light_direction: LightDirection,
    foreground: [u8; 3],
    background: [u8; 3],
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if image_balance > 50 {
        return Err(FilterError::InvalidParams(format!(
            "plaster image balance {image_balance} is outside 0..=50"
        )));
    }
    if smoothness > 15 {
        return Err(FilterError::InvalidParams(format!(
            "plaster smoothness {smoothness} is outside 0..=15"
        )));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let field = luma_plane(&buf.data, n);
    let smooth = blur_field(&field, w, h, smoothness as usize / 3);
    let balance = (image_balance as f64 / 50.0 - 0.5) * 0.5;

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
                10,
                false,
            );
            let t = (smooth[i] / 255.0 + balance + lit * 0.6).clamp(0.0, 1.0);
            for c in 0..planes {
                let recessed = foreground[c] as f64;
                let raised = background[c] as f64;
                buf.data[c * n + i] = reduce::clamp_u8(recessed + (raised - recessed) * t);
            }
        }
    }
    Ok(())
}

/// Reticulation: film-emulsion clumping. Coarse seeded clumps darken shadows
/// toward `foreground`, fine seeded grain lights highlights toward `background`;
/// `black_level`/`white_level` place the shadow/highlight cuts and `density`
/// scales the clumping.
///
/// ponytail: box-mean-clumped white noise, not Adobe's emulsion shrink model.
/// Multi-octave clumping is the upgrade path if the grain reads too uniform.
pub fn reticulation(
    buf: &mut PixelBuffer,
    density: u8,
    black_level: u8,
    white_level: u8,
    foreground: [u8; 3],
    background: [u8; 3],
    seed: u64,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if density > 50 {
        return Err(FilterError::InvalidParams(format!(
            "reticulation density {density} is outside 0..=50"
        )));
    }
    if black_level > 50 {
        return Err(FilterError::InvalidParams(format!(
            "reticulation black level {black_level} is outside 0..=50"
        )));
    }
    if white_level > 50 {
        return Err(FilterError::InvalidParams(format!(
            "reticulation white level {white_level} is outside 0..=50"
        )));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let field = luma_plane(&buf.data, n);
    let fine = noise::value_noise(w, h, seed);
    let clump = blur_field(&fine, w, h, 1 + density as usize / 12);
    let black_cut = black_level as f64 / 50.0 * 0.6;
    let white_cut = 1.0 - white_level as f64 / 50.0 * 0.6;
    let span = (white_cut - black_cut).max(0.1);
    let clump_amp = density as f64 / 50.0;

    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let t = (field[i] / 255.0).clamp(0.0, 1.0);
            let tone = ((t - black_cut) / span).clamp(0.0, 1.0);
            let shadow = (clump[i] - 0.5) * clump_amp * (1.0 - t);
            let highlight = (fine[i] - 0.5) * clump_amp * 0.5 * t;
            let m = (tone + shadow + highlight).clamp(0.0, 1.0);
            for c in 0..planes {
                let dark = foreground[c] as f64;
                let light = background[c] as f64;
                buf.data[c * n + i] = reduce::clamp_u8(dark + (light - dark) * m);
            }
        }
    }
    Ok(())
}

/// Stamp: blur then threshold to foreground ink on background paper.
/// `light_dark_balance` sets the cut (0 = dark/heavy, 50 = light/sparse) and
/// `smoothness` is the pre-threshold blur radius.
///
/// ponytail: a single hard threshold, not Adobe's rubber-stamp edge roughening.
/// Add a nib-shaped jitter if the outline needs texture.
pub fn stamp(
    buf: &mut PixelBuffer,
    light_dark_balance: u8,
    smoothness: u8,
    foreground: [u8; 3],
    background: [u8; 3],
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if light_dark_balance > 50 {
        return Err(FilterError::InvalidParams(format!(
            "stamp light/dark balance {light_dark_balance} is outside 0..=50"
        )));
    }
    if !(1..=50).contains(&smoothness) {
        return Err(FilterError::InvalidParams(format!(
            "stamp smoothness {smoothness} is outside 1..=50"
        )));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let field = luma_plane(&buf.data, n);
    let smooth = blur_field(&field, w, h, smoothness as usize / 8);
    let threshold = 200.0 - light_dark_balance as f64 / 50.0 * 160.0;

    for (i, &s) in smooth.iter().enumerate() {
        let ink = if s < threshold { 1.0 } else { 0.0 };
        for c in 0..planes {
            let paper = background[c] as f64;
            let pen = foreground[c] as f64;
            buf.data[c * n + i] = reduce::clamp_u8(paper + (pen - paper) * ink);
        }
    }
    Ok(())
}

/// Torn Edges: threshold with a ragged fibrous boundary. `image_balance` sets the
/// cut, `contrast` the tonal gain, `smoothness` the pre-threshold blur; the
/// seeded lattice roughens the tear. Foreground ink on background paper.
///
/// ponytail: one lattice-perturbed sigmoid, not Adobe's fibre-cell tear. Layer a
/// directional fibre octave if the edges need real torn strands.
pub fn torn_edges(
    buf: &mut PixelBuffer,
    image_balance: u8,
    smoothness: u8,
    contrast: u8,
    foreground: [u8; 3],
    background: [u8; 3],
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if image_balance > 50 {
        return Err(FilterError::InvalidParams(format!(
            "torn edges image balance {image_balance} is outside 0..=50"
        )));
    }
    if !(1..=15).contains(&smoothness) {
        return Err(FilterError::InvalidParams(format!(
            "torn edges smoothness {smoothness} is outside 1..=15"
        )));
    }
    if !(1..=25).contains(&contrast) {
        return Err(FilterError::InvalidParams(format!(
            "torn edges contrast {contrast} is outside 1..=25"
        )));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let field = luma_plane(&buf.data, n);
    let smooth = blur_field(&field, w, h, smoothness as usize / 3);
    let cut = 0.25 + image_balance as f64 / 50.0 * 0.5;
    let gain = 0.8 + contrast as f64 / 25.0 * 2.0;

    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let base = smooth[i] / 255.0;
            let ragged = lattice(x as f64 / 3.0, y as f64 / 6.0, 0x705E) - 0.5;
            let d = (base - cut + ragged * 0.5) * gain;
            let ink = 1.0 / (1.0 + (d * 6.0).exp());
            for c in 0..planes {
                let paper = background[c] as f64;
                let pen = foreground[c] as f64;
                buf.data[c * n + i] = reduce::clamp_u8(paper + (pen - paper) * ink);
            }
        }
    }
    Ok(())
}

/// Water Paper: fibrous damp-paper texture. `fiber_length` sets the fibre scale,
/// `brightness`/`contrast` grade the result, and a seeded lattice + grain make
/// colours pool and blend.
///
/// ponytail: an anisotropic lattice over per-pixel grain, not Adobe's fluid
/// daub simulation. A direction field is the upgrade path if the flow reads flat.
pub fn water_paper(
    buf: &mut PixelBuffer,
    fiber_length: u8,
    brightness: u8,
    contrast: u8,
    seed: u64,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if !(3..=50).contains(&fiber_length) {
        return Err(FilterError::InvalidParams(format!(
            "water paper fiber length {fiber_length} is outside 3..=50"
        )));
    }
    if brightness > 100 {
        return Err(FilterError::InvalidParams(format!(
            "water paper brightness {brightness} is outside 0..=100"
        )));
    }
    if contrast > 100 {
        return Err(FilterError::InvalidParams(format!(
            "water paper contrast {contrast} is outside 0..=100"
        )));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let grain = noise::value_noise(w, h, seed ^ 0x9E37);
    let fiber_scale = fiber_length as f64;
    let shift = (brightness as f64 / 100.0 - 0.5) * 0.7;
    let gain = 0.4 + contrast as f64 / 100.0 * 1.6;

    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let fiber = lattice(x as f64 / fiber_scale, y as f64 / 2.0, seed) - 0.5;
            let tex = fiber * 0.35 + (grain[i] - 0.5) * 0.12;
            for c in 0..planes {
                let base = buf.data[c * n + i] as f64 / 255.0;
                let v = ((base - 0.5) * gain + 0.5 + shift + tex).clamp(0.0, 1.0);
                buf.data[c * n + i] = reduce::clamp_u8(v * 255.0);
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

    fn flat(w: u32, h: u32, value: u8) -> PixelBuffer {
        let n = (w * h) as usize;
        PixelBuffer {
            width: w,
            height: h,
            channels: 3,
            data: vec![value; n * 3],
        }
    }

    fn block(w: u32, h: u32, block_w: u32, value: u8) -> PixelBuffer {
        let n = (w * h) as usize;
        let mut buf = flat(w, h, 255);
        for y in 0..h as usize {
            for x in 0..block_w as usize {
                for c in 0..3 {
                    buf.data[c * n + y * w as usize + x] = value;
                }
            }
        }
        buf
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

    fn paper_cases(seed: u64) -> Vec<Filter> {
        vec![
            Filter::NotePaper {
                image_balance: 25,
                graininess: 10,
                relief: 11,
                seed,
            },
            Filter::Photocopy {
                detail: 5,
                darkness: 20,
            },
            Filter::Plaster {
                image_balance: 25,
                smoothness: 2,
                light_direction: LightDirection::Bottom,
                foreground: FG,
                background: BG,
            },
            Filter::Reticulation {
                density: 13,
                black_level: 10,
                white_level: 40,
                foreground: FG,
                background: BG,
                seed,
            },
            Filter::Stamp {
                light_dark_balance: 25,
                smoothness: 5,
                foreground: FG,
                background: BG,
            },
            Filter::TornEdges {
                image_balance: 25,
                smoothness: 1,
                contrast: 8,
                foreground: FG,
                background: BG,
            },
            Filter::WaterPaper {
                fiber_length: 15,
                brightness: 45,
                contrast: 60,
                seed,
            },
        ]
    }

    #[test]
    fn each_filter_changes_the_colour_planes() {
        let base = gradient(32, 8, 3);
        let n = base.pixel_count();
        for filter in paper_cases(3) {
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
        for filter in paper_cases(9) {
            let mut out = base.clone();
            apply(&filter, &mut out).unwrap();
            assert_eq!(alpha_plane(&out), expected, "{filter:?} touched alpha");
        }
    }

    #[test]
    fn boundaries_accept_and_out_of_range_rejects() {
        let base = gradient(8, 4, 3);

        assert!(note_paper(&mut base.clone(), 0, 0, 0, 1).is_ok());
        assert!(note_paper(&mut base.clone(), 50, 20, 25, 1).is_ok());
        for (b, g, r) in [(51u8, 0u8, 0u8), (0, 21, 0), (0, 0, 26)] {
            let mut out = base.clone();
            assert!(matches!(
                note_paper(&mut out, b, g, r, 1),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected note paper modified the buffer");
        }

        assert!(photocopy(&mut base.clone(), 0, 1).is_ok());
        assert!(photocopy(&mut base.clone(), 24, 50).is_ok());
        for (d, k) in [(25u8, 20u8), (5, 0), (5, 51)] {
            let mut out = base.clone();
            assert!(matches!(
                photocopy(&mut out, d, k),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected photocopy modified the buffer");
        }

        assert!(plaster(&mut base.clone(), 0, 0, LightDirection::Bottom, FG, BG).is_ok());
        assert!(plaster(
            &mut base.clone(),
            50,
            15,
            LightDirection::BottomRight,
            FG,
            BG
        )
        .is_ok());
        for (b, s) in [(51u8, 0u8), (25, 16)] {
            let mut out = base.clone();
            assert!(matches!(
                plaster(&mut out, b, s, LightDirection::Top, FG, BG),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected plaster modified the buffer");
        }

        assert!(reticulation(&mut base.clone(), 0, 0, 0, FG, BG, 1).is_ok());
        assert!(reticulation(&mut base.clone(), 50, 50, 50, FG, BG, 1).is_ok());
        for (d, bl, wl) in [(51u8, 0u8, 0u8), (0, 51, 0), (0, 0, 51)] {
            let mut out = base.clone();
            assert!(matches!(
                reticulation(&mut out, d, bl, wl, FG, BG, 1),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected reticulation modified the buffer");
        }

        assert!(stamp(&mut base.clone(), 0, 1, FG, BG).is_ok());
        assert!(stamp(&mut base.clone(), 50, 50, FG, BG).is_ok());
        for (b, s) in [(51u8, 5u8), (25, 0), (25, 51)] {
            let mut out = base.clone();
            assert!(matches!(
                stamp(&mut out, b, s, FG, BG),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected stamp modified the buffer");
        }

        assert!(torn_edges(&mut base.clone(), 0, 1, 1, FG, BG).is_ok());
        assert!(torn_edges(&mut base.clone(), 50, 15, 25, FG, BG).is_ok());
        for (b, s, c) in [
            (51u8, 1u8, 1u8),
            (25, 0, 1),
            (25, 16, 1),
            (25, 1, 0),
            (25, 1, 26),
        ] {
            let mut out = base.clone();
            assert!(matches!(
                torn_edges(&mut out, b, s, c, FG, BG),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected torn edges modified the buffer");
        }

        assert!(water_paper(&mut base.clone(), 3, 0, 0, 1).is_ok());
        assert!(water_paper(&mut base.clone(), 50, 100, 100, 1).is_ok());
        for (f, b, c) in [(2u8, 0u8, 0u8), (51, 0, 0), (15, 101, 0), (15, 0, 101)] {
            let mut out = base.clone();
            assert!(matches!(
                water_paper(&mut out, f, b, c, 1),
                Err(FilterError::InvalidParams(_))
            ));
            assert_eq!(out, base, "rejected water paper modified the buffer");
        }
    }

    #[test]
    fn seeded_filters_are_deterministic() {
        let base = gradient(32, 8, 3);

        let (mut a, mut b, mut c) = (base.clone(), base.clone(), base.clone());
        note_paper(&mut a, 25, 10, 11, 7).unwrap();
        note_paper(&mut b, 25, 10, 11, 7).unwrap();
        note_paper(&mut c, 25, 10, 11, 8).unwrap();
        assert_eq!(a.data, b.data, "note paper same seed must match");
        assert_ne!(a.data, c.data, "note paper different seed must differ");

        let (mut d, mut e, mut f) = (base.clone(), base.clone(), base.clone());
        reticulation(&mut d, 13, 10, 40, FG, BG, 7).unwrap();
        reticulation(&mut e, 13, 10, 40, FG, BG, 7).unwrap();
        reticulation(&mut f, 13, 10, 40, FG, BG, 8).unwrap();
        assert_eq!(d.data, e.data, "reticulation same seed must match");
        assert_ne!(d.data, f.data, "reticulation different seed must differ");

        let (mut g, mut hh, mut j) = (base.clone(), base.clone(), base.clone());
        water_paper(&mut g, 15, 45, 60, 7).unwrap();
        water_paper(&mut hh, 15, 45, 60, 7).unwrap();
        water_paper(&mut j, 15, 45, 60, 8).unwrap();
        assert_eq!(g.data, hh.data, "water paper same seed must match");
        assert_ne!(g.data, j.data, "water paper different seed must differ");
    }

    #[test]
    fn photocopy_darkness_shrinks_the_dark_area() {
        let base = block(32, 8, 16, 60);
        let n = base.pixel_count();
        let dark_pixels =
            |buf: &PixelBuffer| -> usize { buf.data[..n].iter().filter(|&&v| v == 0).count() };

        let mut dark = base.clone();
        photocopy(&mut dark, 12, 10).unwrap();
        let mut light = base.clone();
        photocopy(&mut light, 12, 50).unwrap();

        assert!(
            dark_pixels(&dark) > dark_pixels(&light),
            "raising darkness must shrink the dark area ({} vs {})",
            dark_pixels(&dark),
            dark_pixels(&light)
        );
    }

    #[test]
    fn reticulation_levels_change_the_result() {
        let base = gradient(32, 8, 3);
        let (mut a, mut b, mut c) = (base.clone(), base.clone(), base.clone());
        reticulation(&mut a, 13, 10, 40, FG, BG, 4).unwrap();
        reticulation(&mut b, 13, 50, 40, FG, BG, 4).unwrap();
        reticulation(&mut c, 13, 10, 50, FG, BG, 4).unwrap();
        assert_ne!(a.data, b.data, "black level must change the result");
        assert_ne!(a.data, c.data, "white level must change the result");
    }

    #[test]
    fn torn_edges_contrast_and_balance_change_the_result() {
        let base = gradient(32, 8, 3);
        let (mut a, mut b, mut c) = (base.clone(), base.clone(), base.clone());
        torn_edges(&mut a, 25, 1, 8, FG, BG).unwrap();
        torn_edges(&mut b, 25, 1, 25, FG, BG).unwrap();
        torn_edges(&mut c, 50, 1, 8, FG, BG).unwrap();
        assert_ne!(a.data, b.data, "contrast must change the result");
        assert_ne!(a.data, c.data, "image balance must change the result");
    }

    #[test]
    fn equal_foreground_and_background_are_safe() {
        let base = gradient(16, 6, 3);
        let same = [7, 7, 7];
        assert!(plaster(&mut base.clone(), 25, 5, LightDirection::Top, same, same).is_ok());
        assert!(reticulation(&mut base.clone(), 13, 10, 40, same, same, 1).is_ok());
        assert!(stamp(&mut base.clone(), 25, 5, same, same).is_ok());
        assert!(torn_edges(&mut base.clone(), 25, 1, 8, same, same).is_ok());
    }

    #[test]
    fn tiny_buffers_do_not_panic() {
        let tiny = PixelBuffer {
            width: 1,
            height: 1,
            channels: 4,
            data: vec![10, 20, 30, 40],
        };
        assert!(note_paper(&mut tiny.clone(), 50, 20, 25, 1).is_ok());
        assert!(photocopy(&mut tiny.clone(), 24, 50).is_ok());
        assert!(plaster(&mut tiny.clone(), 50, 15, LightDirection::Top, FG, BG).is_ok());
        assert!(reticulation(&mut tiny.clone(), 50, 50, 50, FG, BG, 1).is_ok());
        assert!(stamp(&mut tiny.clone(), 50, 50, FG, BG).is_ok());
        assert!(torn_edges(&mut tiny.clone(), 50, 15, 25, FG, BG).is_ok());
        assert!(water_paper(&mut tiny.clone(), 50, 100, 100, 1).is_ok());
    }
}
