//! Procedural texture surfaces for the Artistic family (`m22-artistic-filters`):
//! coordinate-addressable height fields and a signed emboss term.

use std::f64::consts::TAU;

use crate::{TextureOptions, TextureSurface};

/// Integer-hash sample in `0..1`, deterministic from `(seed, x, y)`.
///
/// ponytail: a stateless coordinate hash, so no canvas-sized field is ever
/// materialized; swap in a real gradient-noise lattice if the mottling bands.
pub(crate) fn hash2(seed: u64, x: i64, y: i64) -> f64 {
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
pub(crate) fn lattice(x: f64, y: f64, seed: u64) -> f64 {
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

/// Procedural surface height in `0..=1` at `(x, y)`. `scaling` is a percentage:
/// larger values widen the pattern.
pub fn surface_height(surface: TextureSurface, x: usize, y: usize, scaling: u8) -> f64 {
    let s = scaling.clamp(50, 200) as f64 / 100.0;
    let fx = x as f64 / s;
    let fy = y as f64 / s;
    let h = match surface {
        TextureSurface::Brick => {
            let (bw, bh, m) = (24.0, 12.0, 0.12);
            let row = (fy / bh).floor();
            let off = if (row as i64) & 1 == 0 { 0.0 } else { bw * 0.5 };
            let bx = (fx + off) / bw;
            let bx = bx - bx.floor();
            let by = fy / bh - row;
            let face = |t: f64| ((t - m).min(1.0 - m - t)).max(0.0) / (0.5 - m);
            let g = face(bx).min(face(by));
            0.15 + 0.85 * g + (hash2(0xB71C, fx.floor() as i64, fy.floor() as i64) - 0.5) * 0.1
        }
        TextureSurface::Burlap => {
            let f = 4.0;
            0.5 + 0.4 * (fx * f).sin() * (fy * f).sin()
        }
        TextureSurface::Canvas => {
            let f = 12.0;
            let weave = (fx * f).sin() * 0.5 + (fy * f).sin() * 0.5;
            0.45 + 0.35 * weave + (lattice(fx * 0.3, fy * 0.3, 0x0CA5) - 0.5) * 0.15
        }
        TextureSurface::Sandstone => {
            lattice(fx * 0.35, fy * 0.35, 0x5A7D) * 0.6
                + lattice(fx * 0.9, fy * 0.9, 0x517D) * 0.3
                + lattice(fx * 2.2, fy * 2.2, 0x77AA) * 0.1
        }
    };
    h.clamp(0.0, 1.0)
}

/// Signed lighting from the local height gradient at `(x, y)`.
pub fn emboss(
    height: impl Fn(usize, usize) -> f64,
    x: usize,
    y: usize,
    light_direction: u8,
    relief: u8,
    invert: bool,
) -> f64 {
    let h = |dx: isize, dy: isize| -> f64 {
        let hx = (x as isize + dx).max(0) as usize;
        let hy = (y as isize + dy).max(0) as usize;
        height(hx, hy)
    };
    let dx = h(1, 0) - h(-1, 0);
    let dy = h(0, 1) - h(0, -1);
    let angle = light_direction.min(7) as f64 / 8.0 * TAU;
    let lit = -((dx * angle.cos() + dy * angle.sin()) * relief as f64 * 0.5);
    if invert {
        -lit
    } else {
        lit
    }
}

/// Whether `scaling`, `relief`, and `light_direction` are in range.
pub fn texture_options_valid(t: &TextureOptions) -> bool {
    (50..=200).contains(&t.scaling) && t.relief <= 50 && t.light_direction <= 7
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surfaces_differ_and_edges_are_safe() {
        let hs: Vec<f64> = [
            TextureSurface::Brick,
            TextureSurface::Burlap,
            TextureSurface::Canvas,
            TextureSurface::Sandstone,
        ]
        .iter()
        .map(|&s| surface_height(s, 0, 0, 100))
        .collect();
        let distinct: std::collections::BTreeSet<i64> =
            hs.iter().map(|v| (v * 1e6).round() as i64).collect();
        assert!(distinct.len() >= 2, "surfaces must differ: {hs:?}");
        let _ = surface_height(TextureSurface::Canvas, 0, 0, 50);
        let _ = surface_height(TextureSurface::Canvas, 0, 0, 200);
    }

    #[test]
    fn emboss_reacts_to_light_and_invert() {
        let step = |x: usize, _y: usize| if x < 2 { 0.0 } else { 1.0 };
        let a = emboss(step, 2, 1, 0, 10, false);
        let b = emboss(step, 2, 1, 4, 10, false);
        assert_ne!(a, b, "light direction must change the term");
        assert_eq!(emboss(step, 2, 1, 0, 10, true), -a, "invert must flip");
    }

    #[test]
    fn options_validate_ranges() {
        let ok = TextureOptions::default();
        assert!(texture_options_valid(&ok));
        assert!(texture_options_valid(&TextureOptions { scaling: 50, ..ok }));
        assert!(texture_options_valid(&TextureOptions {
            scaling: 200,
            ..ok
        }));
        assert!(texture_options_valid(&TextureOptions {
            relief: 50,
            light_direction: 7,
            ..ok
        }));
        assert!(!texture_options_valid(&TextureOptions {
            scaling: 49,
            ..ok
        }));
        assert!(!texture_options_valid(&TextureOptions {
            scaling: 201,
            ..ok
        }));
        assert!(!texture_options_valid(&TextureOptions { relief: 51, ..ok }));
        assert!(!texture_options_valid(&TextureOptions {
            light_direction: 8,
            ..ok
        }));
    }
}
