//! Texture filter family (`m25-*`).
//!
//! The CS6 Texture submenu: Craquelure, Grain, Mosaic Tiles, Patchwork, Stained
//! Glass, Texturizer. Adobe's kernels are closed, so each is a behavioural model
//! and carries a `ponytail:` note naming its ceiling. Seeded kernels take a
//! `seed: u64` and are bit-repeatable; alpha is never modified.

use pictura_core::PixelBuffer;
use rand_chacha::{rand_core::SeedableRng, ChaCha8Rng};

use crate::artistic::noise::value_noise;
use crate::artistic::reduce::clamp_u8;
use crate::artistic::texture::{emboss, surface_height, texture_options_valid};
use crate::kernel::{clamp_index, unit_f64};
use crate::luma::luma;
use crate::{validate, FilterError, GrainType, TextureOptions};

/// Craquelure takes no seed, but its crack network must be repeatable.
const CRAQUELURE_SEED: u64 = 0x0C7A_9F3E_5B12_34D6;

fn in_range(name: &str, v: u8, lo: u8, hi: u8) -> Result<(), FilterError> {
    if (lo..=hi).contains(&v) {
        Ok(())
    } else {
        Err(FilterError::InvalidParams(format!(
            "{name} {v} is outside {lo}..={hi}"
        )))
    }
}

/// Jittered grid Voronoi: `assign[p]` is the owning site, `edge[p]` is the
/// distance to the second-nearest site minus the nearest (0 on a cell border).
/// Only the 3x3 grid neighbourhood is scanned, matching the bounded-apron rule.
fn tessellate(w: usize, h: usize, cs: usize, seed: u64) -> (Vec<usize>, Vec<f64>) {
    let gw = w.div_ceil(cs);
    let gh = h.div_ceil(cs);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut sites = vec![(0.0f64, 0.0f64); gw * gh];
    for (g, site) in sites.iter_mut().enumerate() {
        let (gx, gy) = (g % gw, g / gw);
        *site = (
            gx as f64 * cs as f64 + unit_f64(&mut rng) * cs as f64,
            gy as f64 * cs as f64 + unit_f64(&mut rng) * cs as f64,
        );
    }
    let mut assign = vec![0usize; w * h];
    let mut edge = vec![0.0f64; w * h];
    for y in 0..h {
        let cy = (y / cs).min(gh - 1);
        let y0 = cy.saturating_sub(1);
        let y1 = (cy + 1).min(gh - 1);
        for x in 0..w {
            let cx = (x / cs).min(gw - 1);
            let x0 = cx.saturating_sub(1);
            let x1 = (cx + 1).min(gw - 1);
            let (px, py) = (x as f64 + 0.5, y as f64 + 0.5);
            let (mut d1, mut d2, mut id) = (f64::INFINITY, f64::INFINITY, 0usize);
            for gy in y0..=y1 {
                for gx in x0..=x1 {
                    let (qx, qy) = sites[gy * gw + gx];
                    let d = (px - qx).powi(2) + (py - qy).powi(2);
                    if d < d1 {
                        d2 = d1;
                        d1 = d;
                        id = gy * gw + gx;
                    } else if d < d2 {
                        d2 = d;
                    }
                }
            }
            let p = y * w + x;
            assign[p] = id;
            edge[p] = (d2.sqrt() - d1.sqrt()).max(0.0);
        }
    }
    (assign, edge)
}

/// Mean colour per Voronoi cell and the cell population count.
fn cell_means(data: &[u8], n: usize, planes: usize, assign: &[usize]) -> (Vec<[f64; 3]>, Vec<u64>) {
    let cells = assign.iter().copied().max().map_or(0, |m| m + 1);
    let mut acc = vec![[0u64; 3]; cells];
    let mut cnt = vec![0u64; cells];
    for (p, &g) in assign.iter().enumerate() {
        cnt[g] += 1;
        for (c, a) in acc[g].iter_mut().enumerate().take(planes) {
            *a += data[c * n + p] as u64;
        }
    }
    let means = acc
        .iter()
        .zip(&cnt)
        .map(|(a, &k)| std::array::from_fn(|c| a[c] as f64 / k.max(1) as f64))
        .collect();
    (means, cnt)
}

/// 3x3 clamp-to-edge box average of a scalar field.
fn smoothed(field: &[f64], w: usize, h: usize) -> Vec<f64> {
    let mut out = vec![0.0f64; field.len()];
    for y in 0..h {
        for x in 0..w {
            let mut s = 0.0;
            for dy in -1isize..=1 {
                for dx in -1isize..=1 {
                    let sx = clamp_index(x as isize + dx, w);
                    let sy = clamp_index(y as isize + dy, h);
                    s += field[sy * w + sx];
                }
            }
            out[y * w + x] = s / 9.0;
        }
    }
    out
}

/// The scalar fields a Grain draw reads from.
struct GrainNoise<'a> {
    field: &'a [f64],
    field2: &'a [f64],
    soft: &'a [f64],
    w: usize,
    h: usize,
}

impl GrainNoise<'_> {
    /// One grain sample: signed perturbation plus a 0..1 background-composite
    /// weight (non-zero only for Sprinkles / Stippled, which draw from `background`).
    fn sample(&self, gt: GrainType, x: usize, y: usize) -> (f64, f64) {
        let p = y * self.w + x;
        let field = self.field;
        match gt {
            GrainType::Regular => (field[p] - 0.5, 0.0),
            GrainType::Soft => (self.soft[p] - 0.5, 0.0),
            GrainType::Sprinkles => {
                let f = field[p];
                if f > 0.80 {
                    (0.0, ((f - 0.80) / 0.20).min(1.0))
                } else {
                    (0.0, 0.0)
                }
            }
            GrainType::Clumped => {
                let c = 1.0 - (2.0 * field[p] - 1.0).abs();
                (c - 0.5, 0.0)
            }
            GrainType::Contrasty => {
                let f = field[p] - 0.5;
                (f.signum() * (f.abs() * 2.0).min(0.5), 0.0)
            }
            GrainType::Enlarged => {
                let sx = (x / 3).min(self.w - 1);
                let sy = (y / 3).min(self.h - 1);
                (field[sy * self.w + sx] - 0.5, 0.0)
            }
            GrainType::Stippled => {
                if field[p] > 0.62 && self.field2[p] > 0.40 {
                    (0.0, 0.7)
                } else {
                    (0.0, 0.0)
                }
            }
            GrainType::Horizontal => (field[y * self.w + (x / 4).min(self.w - 1)] - 0.5, 0.0),
            GrainType::Vertical => (field[(y / 4).min(self.h - 1) * self.w + x] - 0.5, 0.0),
            GrainType::Speckle => {
                if field[p] > 0.5 {
                    (0.5, 0.0)
                } else {
                    (-0.5, 0.0)
                }
            }
        }
    }
}

/// Craquelure: a cellular crack network generated from image luminance as a
/// height field, then embossed so crack edges catch the light.
///
/// ponytail: the closed crack generator is approximated by a luma-modulated
/// jittered-Voronoi valley field; `crack_spacing` is the cell size in pixels.
pub fn craquelure(
    buf: &mut PixelBuffer,
    crack_spacing: u8,
    crack_depth: u8,
    crack_brightness: u8,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    in_range("craquelure crack spacing", crack_spacing, 2, 100)?;
    in_range("craquelure crack depth", crack_depth, 0, 10)?;
    in_range("craquelure crack brightness", crack_brightness, 0, 10)?;

    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let (_, edge) = tessellate(w, h, crack_spacing as usize, CRAQUELURE_SEED);

    let lum: Vec<f64> = (0..n)
        .map(|i| {
            luma(
                buf.data[i] as f64,
                buf.data[n + i] as f64,
                buf.data[2 * n + i] as f64,
            )
        })
        .collect();
    let cw = (crack_spacing as f64 * 0.25).max(1.0);
    let depth = crack_depth as f64 / 10.0;
    let height: Vec<f64> = (0..n)
        .map(|p| {
            let crack = (1.0 - edge[p] / cw).clamp(0.0, 1.0);
            lum[p] / 255.0 * 0.7 - crack * (0.3 + 0.5 * depth)
        })
        .collect();

    let relief = 3 + crack_depth * 4;
    let bright = crack_brightness as f64 / 10.0 * 110.0 - 25.0;
    for y in 0..h {
        for x in 0..w {
            let p = y * w + x;
            let shade = emboss(
                |hx, hy| height[clamp_index(hy as isize, h) * w + clamp_index(hx as isize, w)],
                x,
                y,
                5,
                relief,
                false,
            );
            let crack = (1.0 - edge[p] / cw).clamp(0.0, 1.0);
            let delta = shade * 9.0 + crack * bright * depth.max(0.3);
            for c in 0..planes {
                buf.data[c * n + p] = clamp_u8(buf.data[c * n + p] as f64 + delta);
            }
        }
    }
    Ok(())
}

/// Grain: a seeded noise field shaped into ten distributions. Sprinkles and
/// Stippled composite from `background`; `intensity` 0 is an exact no-op.
///
/// ponytail: per-pixel white noise reshaped per type, not Adobe's correlated
/// grain model; `Enlarged`/`Horizontal`/`Vertical` decimate the same field.
pub fn grain(
    buf: &mut PixelBuffer,
    intensity: u8,
    contrast: u8,
    grain_type: GrainType,
    background: [u8; 3],
    seed: u64,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    in_range("grain intensity", intensity, 0, 100)?;
    in_range("grain contrast", contrast, 0, 100)?;
    if intensity == 0 {
        return Ok(());
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let field = value_noise(w, h, seed);
    let field2 = value_noise(w, h, seed ^ 0x9E37_79B9_7F4A_7C15);
    let soft = smoothed(&field, w, h);
    let noise = GrainNoise {
        field: &field,
        field2: &field2,
        soft: &soft,
        w,
        h,
    };
    let amp = intensity as f64 / 100.0 * (0.5 + contrast as f64 / 100.0) * 110.0;
    let bg_t = intensity as f64 / 100.0;

    for y in 0..h {
        for x in 0..w {
            let p = y * w + x;
            let (shaped, bg) = noise.sample(grain_type, x, y);
            for (c, &bc) in background.iter().enumerate().take(planes) {
                let mut v = buf.data[c * n + p] as f64;
                if bg > 0.0 {
                    v += (bc as f64 - v) * (bg * bg_t).clamp(0.0, 1.0);
                } else {
                    v += shaped * amp;
                }
                buf.data[c * n + p] = clamp_u8(v);
            }
        }
    }
    Ok(())
}

/// Mosaic Tiles: a jittered cell tessellation, each cell flat-filled with its
/// averaged colour and separated by a grout band that `lighten_grout` lifts.
///
/// ponytail: Voronoi chips with a distance-band grout, not Adobe's rectangular
/// chip cutter; a rectangular variant would be a small swap in `tessellate`.
pub fn mosaic_tiles(
    buf: &mut PixelBuffer,
    tile_size: u8,
    grout_width: u8,
    lighten_grout: u8,
    seed: u64,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    in_range("mosaic tiles tile size", tile_size, 2, 100)?;
    in_range("mosaic tiles grout width", grout_width, 1, 15)?;
    in_range("mosaic tiles lighten grout", lighten_grout, 0, 10)?;

    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let (assign, edge) = tessellate(w, h, tile_size as usize, seed);
    let (means, _) = cell_means(&buf.data, n, planes, &assign);

    let grout_lum = 40.0 + lighten_grout as f64 * 18.0;
    let gw = grout_width as f64;
    for (p, &g) in assign.iter().enumerate() {
        let cell = means[g];
        if edge[p] < gw {
            let t = 1.0 - edge[p] / gw;
            for (c, &m) in cell.iter().enumerate().take(planes) {
                buf.data[c * n + p] = clamp_u8(m + (grout_lum - m) * t);
            }
        } else {
            for (c, &m) in cell.iter().enumerate().take(planes) {
                buf.data[c * n + p] = clamp_u8(m);
            }
        }
    }
    Ok(())
}

/// Patchwork: square blocks filled with the block mean and raised or lowered by
/// `relief` with a diagonal bevel.
///
/// ponytail: block mean stands in for Adobe's closed "predominant colour"
/// statistic; a mode/median would drop in where `mean` is computed.
pub fn patchwork(
    buf: &mut PixelBuffer,
    square_size: u8,
    relief: u8,
    seed: u64,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    in_range("patchwork square size", square_size, 0, 10)?;
    in_range("patchwork relief", relief, 0, 25)?;

    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let cell = (square_size as usize).max(1);
    let rel = relief as f64;
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    for by in (0..h).step_by(cell) {
        let y1 = (by + cell).min(h);
        for bx in (0..w).step_by(cell) {
            let x1 = (bx + cell).min(w);
            let count = ((y1 - by) * (x1 - bx)) as f64;
            let mut mean = [0.0f64; 3];
            for (c, m) in mean.iter_mut().enumerate().take(planes) {
                let mut sum = 0u64;
                for y in by..y1 {
                    for x in bx..x1 {
                        sum += buf.data[c * n + y * w + x] as u64;
                    }
                }
                *m = sum as f64 / count;
            }
            let depth = unit_f64(&mut rng) * 2.0 - 1.0;
            let (bw, bh) = (x1 - bx, y1 - by);
            for y in by..y1 {
                for x in bx..x1 {
                    let nx = if bw > 1 {
                        2.0 * (x - bx) as f64 / (bw - 1) as f64 - 1.0
                    } else {
                        0.0
                    };
                    let ny = if bh > 1 {
                        2.0 * (y - by) as f64 / (bh - 1) as f64 - 1.0
                    } else {
                        0.0
                    };
                    let delta = depth * rel * 2.5 + (nx + ny) * 0.5 * depth * rel * 1.2;
                    for (c, &m) in mean.iter().enumerate().take(planes) {
                        buf.data[c * n + y * w + x] = clamp_u8(m + delta);
                    }
                }
            }
        }
    }
    Ok(())
}

/// Stained Glass: a jittered cell segmentation, each cell flat-filled with its
/// averaged colour and outlined in `foreground`; `light_intensity` lifts the
/// cells.
///
/// ponytail: Voronoi segmentation rather than Adobe's closed watershed; the
/// `cell_size`-scaled 3x3-site lookup keeps it tile-local and bounded.
pub fn stained_glass(
    buf: &mut PixelBuffer,
    cell_size: u8,
    border_thickness: u8,
    light_intensity: u8,
    foreground: [u8; 3],
    seed: u64,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    in_range("stained glass cell size", cell_size, 2, 50)?;
    in_range("stained glass border thickness", border_thickness, 1, 20)?;
    in_range("stained glass light intensity", light_intensity, 0, 10)?;

    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let (assign, edge) = tessellate(w, h, cell_size as usize, seed);
    let (means, _) = cell_means(&buf.data, n, planes, &assign);

    let lit = 1.0 + light_intensity as f64 * 0.05;
    let border = border_thickness as f64;
    for (p, &g) in assign.iter().enumerate() {
        if edge[p] < border {
            for (c, &fg) in foreground.iter().enumerate().take(planes) {
                buf.data[c * n + p] = clamp_u8(fg as f64 * lit);
            }
        } else {
            for (c, &m) in means[g].iter().enumerate().take(planes) {
                buf.data[c * n + p] = clamp_u8(m * lit);
            }
        }
    }
    Ok(())
}

/// Texturizer: tile a procedural [`TextureOptions`] height map under the image
/// and light it with the shared emboss model.
///
/// ponytail: reuses the Artistic surface model; a loaded-texture path (the CS6
/// "Load Texture" option) is out of scope until a texture resource exists.
pub fn texturizer(buf: &mut PixelBuffer, texture: TextureOptions) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if !texture_options_valid(&texture) {
        return Err(FilterError::InvalidParams(
            "texturizer texture options are out of range".into(),
        ));
    }

    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let shade = emboss(
                |hx, hy| surface_height(texture.surface, hx, hy, texture.scaling),
                x,
                y,
                texture.light_direction,
                texture.relief,
                texture.invert,
            );
            let grain = (surface_height(texture.surface, x, y, texture.scaling) - 0.5) * 25.0;
            let delta = shade * 12.0 + grain;
            for c in 0..planes {
                buf.data[c * n + i] = clamp_u8(buf.data[c * n + i] as f64 + delta);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{apply, Filter, TextureSurface};
    use std::collections::BTreeSet;

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

    fn mean_luma(buf: &PixelBuffer) -> f64 {
        let n = buf.pixel_count();
        (0..n)
            .map(|i| {
                luma(
                    buf.data[i] as f64,
                    buf.data[n + i] as f64,
                    buf.data[2 * n + i] as f64,
                )
            })
            .sum::<f64>()
            / n as f64
    }

    #[test]
    fn each_texture_filter_changes_the_colour_planes() {
        let base = gradient(32, 8, 3);

        let mut b = base.clone();
        craquelure(&mut b, 20, 6, 9).unwrap();
        assert_ne!(b.data, base.data, "craquelure must change the image");

        let mut b = base.clone();
        grain(&mut b, 60, 50, GrainType::Regular, [255, 255, 255], 7).unwrap();
        assert_ne!(b.data, base.data, "grain must change the image");

        let mut b = base.clone();
        mosaic_tiles(&mut b, 8, 3, 5, 7).unwrap();
        assert_ne!(b.data, base.data, "mosaic tiles must change the image");

        let mut b = base.clone();
        patchwork(&mut b, 4, 10, 7).unwrap();
        assert_ne!(b.data, base.data, "patchwork must change the image");

        let mut b = base.clone();
        stained_glass(&mut b, 8, 3, 5, [0, 0, 0], 7).unwrap();
        assert_ne!(b.data, base.data, "stained glass must change the image");

        let mut b = base.clone();
        texturizer(&mut b, TextureOptions::default()).unwrap();
        assert_ne!(b.data, base.data, "texturizer must change the image");
    }

    #[test]
    fn every_texture_filter_preserves_alpha_through_apply() {
        let base = gradient(32, 8, 4);
        let n = base.pixel_count();
        let alpha = alpha_plane(&base);
        let filters = [
            Filter::Craquelure {
                crack_spacing: 20,
                crack_depth: 6,
                crack_brightness: 9,
            },
            Filter::Grain {
                intensity: 60,
                contrast: 50,
                grain_type: GrainType::Clumped,
                background: [10, 20, 30],
                seed: 7,
            },
            Filter::MosaicTiles {
                tile_size: 8,
                grout_width: 3,
                lighten_grout: 5,
                seed: 7,
            },
            Filter::Patchwork {
                square_size: 4,
                relief: 10,
                seed: 7,
            },
            Filter::StainedGlass {
                cell_size: 8,
                border_thickness: 3,
                light_intensity: 5,
                foreground: [0, 0, 0],
                seed: 7,
            },
            Filter::Texturizer {
                texture: TextureOptions::default(),
            },
        ];
        for filter in &filters {
            let mut b = base.clone();
            apply(filter, &mut b).unwrap();
            assert_ne!(
                b.data[..3 * n],
                base.data[..3 * n],
                "{filter:?} must change colours"
            );
            assert_eq!(alpha_plane(&b), alpha, "{filter:?} must preserve alpha");
        }
    }

    #[test]
    fn boundaries_are_accepted_and_out_of_range_rejected() {
        let mut b = gradient(8, 8, 3);

        assert!(craquelure(&mut b, 2, 0, 0).is_ok());
        assert!(craquelure(&mut b, 100, 10, 10).is_ok());
        assert!(craquelure(&mut b, 1, 5, 5).is_err());
        assert!(craquelure(&mut b, 50, 11, 5).is_err());
        assert!(craquelure(&mut b, 50, 5, 11).is_err());

        assert!(grain(&mut b, 0, 0, GrainType::Regular, [0, 0, 0], 1).is_ok());
        assert!(grain(&mut b, 100, 100, GrainType::Speckle, [255; 3], 1).is_ok());
        assert!(grain(&mut b, 101, 50, GrainType::Regular, [0, 0, 0], 1).is_err());
        assert!(grain(&mut b, 50, 101, GrainType::Regular, [0, 0, 0], 1).is_err());

        assert!(mosaic_tiles(&mut b, 2, 1, 0, 1).is_ok());
        assert!(mosaic_tiles(&mut b, 100, 15, 10, 1).is_ok());
        assert!(mosaic_tiles(&mut b, 1, 3, 5, 1).is_err());
        assert!(mosaic_tiles(&mut b, 50, 0, 5, 1).is_err());
        assert!(mosaic_tiles(&mut b, 50, 16, 5, 1).is_err());
        assert!(mosaic_tiles(&mut b, 50, 5, 11, 1).is_err());

        assert!(patchwork(&mut b, 0, 0, 1).is_ok());
        assert!(patchwork(&mut b, 10, 25, 1).is_ok());
        assert!(patchwork(&mut b, 11, 10, 1).is_err());
        assert!(patchwork(&mut b, 5, 26, 1).is_err());

        assert!(stained_glass(&mut b, 2, 1, 0, [0, 0, 0], 1).is_ok());
        assert!(stained_glass(&mut b, 50, 20, 10, [255; 3], 1).is_ok());
        assert!(stained_glass(&mut b, 1, 3, 5, [0, 0, 0], 1).is_err());
        assert!(stained_glass(&mut b, 50, 0, 5, [0, 0, 0], 1).is_err());
        assert!(stained_glass(&mut b, 50, 21, 5, [0, 0, 0], 1).is_err());
        assert!(stained_glass(&mut b, 50, 5, 11, [0, 0, 0], 1).is_err());

        assert!(texturizer(&mut b, TextureOptions::default()).is_ok());
        let bad_scaling = TextureOptions {
            scaling: 49,
            ..TextureOptions::default()
        };
        assert!(texturizer(&mut b, bad_scaling).is_err());
        let bad_relief = TextureOptions {
            relief: 51,
            ..TextureOptions::default()
        };
        assert!(texturizer(&mut b, bad_relief).is_err());
        let bad_light = TextureOptions {
            light_direction: 8,
            ..TextureOptions::default()
        };
        assert!(texturizer(&mut b, bad_light).is_err());
    }

    #[test]
    fn stochastic_filters_are_seed_deterministic() {
        let base = gradient(24, 24, 3);

        let (mut a, mut c) = (base.clone(), base.clone());
        grain(&mut a, 60, 50, GrainType::Regular, [0, 0, 0], 77).unwrap();
        grain(&mut c, 60, 50, GrainType::Regular, [0, 0, 0], 78).unwrap();
        let mut b = base.clone();
        grain(&mut b, 60, 50, GrainType::Regular, [0, 0, 0], 77).unwrap();
        assert_eq!(a.data, b.data, "grain same seed must be bit-identical");
        assert_ne!(a.data, c.data, "grain different seed must differ");

        let (mut a, mut c) = (base.clone(), base.clone());
        mosaic_tiles(&mut a, 8, 3, 5, 5).unwrap();
        mosaic_tiles(&mut c, 8, 3, 5, 6).unwrap();
        let mut b = base.clone();
        mosaic_tiles(&mut b, 8, 3, 5, 5).unwrap();
        assert_eq!(a.data, b.data, "mosaic same seed must be bit-identical");
        assert_ne!(a.data, c.data, "mosaic different seed must differ");

        let (mut a, mut c) = (base.clone(), base.clone());
        patchwork(&mut a, 4, 10, 5).unwrap();
        patchwork(&mut c, 4, 10, 6).unwrap();
        let mut b = base.clone();
        patchwork(&mut b, 4, 10, 5).unwrap();
        assert_eq!(a.data, b.data, "patchwork same seed must be bit-identical");
        assert_ne!(a.data, c.data, "patchwork different seed must differ");

        let (mut a, mut c) = (base.clone(), base.clone());
        stained_glass(&mut a, 6, 2, 5, [0, 0, 0], 5).unwrap();
        stained_glass(&mut c, 6, 2, 5, [0, 0, 0], 6).unwrap();
        let mut b = base.clone();
        stained_glass(&mut b, 6, 2, 5, [0, 0, 0], 5).unwrap();
        assert_eq!(a.data, b.data, "stained same seed must be bit-identical");
        assert_ne!(a.data, c.data, "stained different seed must differ");
    }

    #[test]
    fn grain_types_are_distinguishable() {
        let base = gradient(32, 32, 3);
        let kinds = [
            GrainType::Regular,
            GrainType::Soft,
            GrainType::Sprinkles,
            GrainType::Clumped,
            GrainType::Contrasty,
            GrainType::Enlarged,
            GrainType::Stippled,
            GrainType::Horizontal,
            GrainType::Vertical,
            GrainType::Speckle,
        ];
        let mut seen: BTreeSet<Vec<u8>> = BTreeSet::new();
        for kind in kinds {
            let mut b = base.clone();
            grain(&mut b, 70, 60, kind, [200, 150, 80], 21).unwrap();
            seen.insert(b.data);
        }
        assert!(
            seen.len() >= 8,
            "the ten grain types must produce distinguishable output, got {}",
            seen.len()
        );
    }

    #[test]
    fn grain_intensity_zero_is_a_noop() {
        let base = gradient(16, 8, 3);
        let mut b = base.clone();
        grain(&mut b, 0, 100, GrainType::Contrasty, [9, 9, 9], 3).unwrap();
        assert_eq!(b.data, base.data, "intensity 0 must be an exact no-op");
    }

    #[test]
    fn craquelure_spacing_and_brightness_matter() {
        let base = gradient(64, 16, 3);
        let (mut fine, mut coarse) = (base.clone(), base.clone());
        craquelure(&mut fine, 2, 6, 9).unwrap();
        craquelure(&mut coarse, 100, 6, 9).unwrap();
        assert_ne!(fine.data, coarse.data, "crack spacing must matter");

        let (mut dark, mut bright) = (base.clone(), base.clone());
        craquelure(&mut dark, 20, 6, 0).unwrap();
        craquelure(&mut bright, 20, 6, 10).unwrap();
        assert_ne!(dark.data, bright.data, "crack brightness must matter");
    }

    #[test]
    fn mosaic_lighten_grout_lifts_luminance() {
        let base = gradient(32, 32, 3);
        let (mut dark, mut light) = (base.clone(), base.clone());
        mosaic_tiles(&mut dark, 8, 3, 0, 5).unwrap();
        mosaic_tiles(&mut light, 8, 3, 10, 5).unwrap();
        assert!(
            mean_luma(&light) > mean_luma(&dark),
            "lighten grout 10 ({}) must lift over 0 ({})",
            mean_luma(&light),
            mean_luma(&dark)
        );
    }

    #[test]
    fn stained_glass_foreground_and_cell_size_matter() {
        let base = gradient(32, 32, 3);
        let (mut black, mut red) = (base.clone(), base.clone());
        stained_glass(&mut black, 8, 3, 5, [0, 0, 0], 7).unwrap();
        stained_glass(&mut red, 8, 3, 5, [255, 0, 0], 7).unwrap();
        assert_ne!(black.data, red.data, "foreground border colour must matter");

        let (mut small, mut large) = (base.clone(), base.clone());
        stained_glass(&mut small, 2, 3, 5, [0, 0, 0], 7).unwrap();
        stained_glass(&mut large, 50, 3, 5, [0, 0, 0], 7).unwrap();
        assert_ne!(small.data, large.data, "cell size must matter");
    }

    #[test]
    fn texturizer_surfaces_and_light_directions_differ() {
        let base = gradient(32, 8, 3);
        let mut default_surface = base.clone();
        texturizer(&mut default_surface, TextureOptions::default()).unwrap();
        let mut brick = base.clone();
        texturizer(
            &mut brick,
            TextureOptions {
                surface: TextureSurface::Brick,
                ..TextureOptions::default()
            },
        )
        .unwrap();
        let mut lit = base.clone();
        texturizer(
            &mut lit,
            TextureOptions {
                light_direction: 4,
                ..TextureOptions::default()
            },
        )
        .unwrap();
        assert_ne!(default_surface.data, brick.data, "surface must matter");
        assert_ne!(
            default_surface.data, lit.data,
            "light direction must matter"
        );
    }

    #[test]
    fn cell_means_empty_cell_is_finite() {
        let data = [10u8, 20, 30, 40, 50, 60];
        let assign = [0usize, 2];
        let (means, cnt) = cell_means(&data, 2, 3, &assign);
        assert_eq!(cnt, vec![1, 0, 1], "cell 1 must have no pixels");
        assert!(
            means.iter().all(|m| m.iter().all(|v| v.is_finite())),
            "an empty cell mean must not be NaN: {means:?}"
        );
    }

    #[test]
    fn degenerate_sizes_do_not_panic() {
        for &(w, h) in &[(1u32, 1u32), (2, 3)] {
            let base = gradient(w, h, 3);
            let mut b = base.clone();
            craquelure(&mut b, 2, 10, 10).unwrap();
            let mut b = base.clone();
            mosaic_tiles(&mut b, 2, 1, 10, 1).unwrap();
            let mut b = base.clone();
            patchwork(&mut b, 0, 25, 1).unwrap();
            let mut b = base.clone();
            stained_glass(&mut b, 2, 20, 10, [3, 3, 3], 1).unwrap();
        }
    }
}
