use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaCha8Rng,
};

use crate::artistic::{reduce, texture};
use crate::kernel::clamp_index;
use crate::{FilterError, TextureOptions, TextureSurface};

/// One uniform `f64` in `[0, 1)` from 53 random bits.
pub(crate) fn unit_f64(rng: &mut ChaCha8Rng) -> f64 {
    (rng.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
}

/// Separable box sum over `(2*radius+1)^2` with clamp-to-edge. Not normalized:
/// callers divide by the window area when they want a mean.
pub(crate) fn box_sum(src: &[f64], w: usize, h: usize, radius: usize) -> Vec<f64> {
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

/// One oriented, soft-edged daub stamped onto a planar buffer. `aspect` is the
/// cross-stroke to along-stroke radius ratio, `power` the edge crispness
/// (larger = harder), and `roughness` jitters coverage with `jitter_field`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_daub(
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

/// Coarse per-pixel stroke directions: one seeded angle per `cell`-sized block.
pub(crate) fn stroke_angles(w: usize, h: usize, cell: usize, seed: u64) -> Vec<f64> {
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

/// Shared range check for the Dry Brush / Fresco pair.
pub(crate) fn validate_brush(
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

pub(crate) fn surface_for(texture: u8) -> TextureSurface {
    match texture {
        1 => TextureSurface::Canvas,
        2 => TextureSurface::Burlap,
        _ => TextureSurface::Sandstone,
    }
}

/// Add an embossed procedural surface term to every colour plane. `texture`
/// selects the preset and scales the relief.
pub(crate) fn apply_surface(
    data: &mut [u8],
    w: usize,
    h: usize,
    n: usize,
    planes: usize,
    texture: u8,
) {
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

/// Add an embossed procedural surface term from full [`TextureOptions`], scaled
/// by `strength`. Rough Pastels and Underpainting expose the surface's scaling,
/// relief, light direction and invert, unlike the fixed preset of
/// [`apply_surface`].
pub(crate) fn apply_surface_options(
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
