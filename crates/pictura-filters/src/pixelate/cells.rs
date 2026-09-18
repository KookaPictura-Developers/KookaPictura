//! Pixelate cell filters: Crystallize, Pointillize, Color Halftone.
//! Implemented by the M8-A2 task. Adobe's kernels are closed; `ponytail:`
//! marks the artistic approximations. Seeds make the randomized ones
//! bit-reproducible; alpha is never touched.

use pictura_core::PixelBuffer;
use rand_chacha::{rand_core::SeedableRng, ChaCha8Rng};

use crate::kernel::{clamp_index, unit_f64};
use crate::{validate, FilterError};

const MIN_CELL: u32 = 3;
const MAX_CELL: u32 = 300;
const MIN_RADIUS: u32 = 4;
const MAX_RADIUS: u32 = 127;

fn check_cell_size(cell_size: u32) -> Result<(), FilterError> {
    if (MIN_CELL..=MAX_CELL).contains(&cell_size) {
        Ok(())
    } else {
        Err(FilterError::InvalidParams(format!(
            "cell size {cell_size} out of range {MIN_CELL}..={MAX_CELL}"
        )))
    }
}

/// Voronoi assignment: one jittered grit per `cell_size` grid cell, nearest wins.
fn nearest_cells(w: usize, h: usize, cs: usize, seed: u64) -> Vec<usize> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut pts = Vec::new();
    for cy in 0..h.div_ceil(cs) {
        for cx in 0..w.div_ceil(cs) {
            let (gx, gy) = (cx as f64 * cs as f64, cy as f64 * cs as f64);
            let jx = unit_f64(&mut rng) * cs as f64;
            let jy = unit_f64(&mut rng) * cs as f64;
            pts.push((gx + jx, gy + jy));
        }
    }
    // ponytail: brute-force nearest is O(pixels · cells); scan only the 3×3
    // cell neighbourhood if Crystallize ever gets hot on large documents.
    let mut out = vec![0usize; w * h];
    for y in 0..h {
        for x in 0..w {
            let (px, py) = (x as f64 + 0.5, y as f64 + 0.5);
            let (mut best, mut best_d) = (0usize, f64::INFINITY);
            for (g, &(qx, qy)) in pts.iter().enumerate() {
                let d = (px - qx).powi(2) + (py - qy).powi(2);
                if d < best_d {
                    best_d = d;
                    best = g;
                }
            }
            out[y * w + x] = best;
        }
    }
    out
}

/// Voronoi tessellation: fill each grit's region with the mean of its members.
pub fn crystallize(buf: &mut PixelBuffer, cell_size: u32, seed: u64) -> Result<(), FilterError> {
    let n = validate(buf)?;
    check_cell_size(cell_size)?;
    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let assign = nearest_cells(w, h, cell_size as usize, seed);
    let grits = assign.iter().copied().max().map_or(0, |m| m + 1);
    let mut acc = vec![([0u64; 3], 0u64); grits];
    for (p, &g) in assign.iter().enumerate() {
        acc[g].1 += 1;
        for (c, a) in acc[g].0.iter_mut().enumerate().take(planes) {
            *a += buf.data[c * n + p] as u64;
        }
    }
    for (p, &g) in assign.iter().enumerate() {
        let (s, k) = acc[g];
        for (c, a) in s.iter().enumerate().take(planes) {
            buf.data[c * n + p] = ((a + k / 2) / k) as u8;
        }
    }
    Ok(())
}

/// Pointillize: scatter seeded dots, each filled with the local source color.
pub fn pointillize(
    buf: &mut PixelBuffer,
    cell_size: u32,
    background: [u8; 3],
    seed: u64,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    check_cell_size(cell_size)?;
    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let cs = cell_size as usize;
    let src = buf.data.clone();
    for (c, &bg) in background.iter().enumerate().take(planes) {
        for p in 0..n {
            buf.data[c * n + p] = bg;
        }
    }
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut color = [0u8; 3];
    for cy in 0..h.div_ceil(cs) {
        for cx in 0..w.div_ceil(cs) {
            let center = (
                cx as f64 * cs as f64 + unit_f64(&mut rng) * cs as f64,
                cy as f64 * cs as f64 + unit_f64(&mut rng) * cs as f64,
            );
            // ponytail: radius law is an artistic guess; CS6's exact scatter
            // radius/density is closed. One dot per cell keeps it reproducible.
            let radius = (cs as f64 / 4.0 + unit_f64(&mut rng) * cs as f64 / 4.0).max(1.0);
            let sx = clamp_index(center.0 as isize, w);
            let sy = clamp_index(center.1 as isize, h);
            for (c, col) in color.iter_mut().enumerate().take(planes) {
                *col = src[c * n + sy * w + sx];
            }
            dot(buf, w, h, planes, center, radius, color);
        }
    }
    Ok(())
}

fn dot(
    buf: &mut PixelBuffer,
    w: usize,
    h: usize,
    planes: usize,
    center: (f64, f64),
    radius: f64,
    color: [u8; 3],
) {
    let n = w * h;
    let r = radius.ceil() as isize;
    let (cx, cy) = center;
    for iy in cy.floor() as isize - r..=cy.ceil() as isize + r {
        if iy < 0 || iy >= h as isize {
            continue;
        }
        for ix in cx.floor() as isize - r..=cx.ceil() as isize + r {
            if ix < 0 || ix >= w as isize {
                continue;
            }
            let dx = ix as f64 + 0.5 - cx;
            let dy = iy as f64 + 0.5 - cy;
            if dx * dx + dy * dy <= radius * radius {
                for (c, &col) in color.iter().enumerate().take(planes) {
                    buf.data[c * n + iy as usize * w + ix as usize] = col;
                }
            }
        }
    }
}

/// Color Halftone: a rotated screening grid per color channel.
pub fn color_halftone(
    buf: &mut PixelBuffer,
    max_radius: u32,
    angles: [f64; 4],
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if !(MIN_RADIUS..=MAX_RADIUS).contains(&max_radius) {
        return Err(FilterError::InvalidParams(format!(
            "max radius {max_radius} out of range {MIN_RADIUS}..={MAX_RADIUS}"
        )));
    }
    if angles.iter().any(|a| !a.is_finite()) {
        return Err(FilterError::InvalidParams(
            "screen angles must be finite".into(),
        ));
    }
    let w = buf.width as usize;
    let h = buf.height as usize;
    let planes = (buf.channels as usize).min(3);
    let spacing = max_radius as f64 * 2.0;
    let src = buf.data.clone();
    for (c, &angle) in angles.iter().enumerate().take(planes) {
        let (sin, cos) = angle.to_radians().sin_cos();
        let base = c * n;
        for y in 0..h {
            for x in 0..w {
                let u = x as f64 * cos + y as f64 * sin;
                let v = -(x as f64) * sin + y as f64 * cos;
                let cell = (u / spacing).floor();
                let cell_v = (v / spacing).floor();
                let (ccx, ccy) = ((cell + 0.5) * spacing, (cell_v + 0.5) * spacing);
                // Sample the source at the cell centre (rotate back to image space).
                let sx = clamp_index((ccx * cos - ccy * sin).round() as isize, w);
                let sy = clamp_index((ccx * sin + ccy * cos).round() as isize, h);
                let bright = src[base + sy * w + sx] as f64 / 255.0;
                let radius = max_radius as f64 * bright;
                let dist = ((u - ccx).powi(2) + (v - ccy).powi(2)).sqrt();
                // ponytail: hard dot threshold; coverage-based AA if edges matter.
                buf.data[base + y * w + x] = if dist <= radius { 255 } else { 0 };
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn buf3(w: u32, h: u32, f: impl Fn(u32, u32) -> [u8; 3]) -> PixelBuffer {
        let n = (w * h) as usize;
        let mut b = PixelBuffer::new(w, h, 3);
        for y in 0..h {
            for x in 0..w {
                let c = f(x, y);
                let p = (y * w + x) as usize;
                b.data[p] = c[0];
                b.data[n + p] = c[1];
                b.data[2 * n + p] = c[2];
            }
        }
        b
    }

    fn buf4(w: u32, h: u32, f: impl Fn(u32, u32) -> [u8; 3]) -> PixelBuffer {
        let base = buf3(w, h, f);
        let n = base.pixel_count();
        let mut data = base.data;
        data.extend(std::iter::repeat_n(200, n));
        PixelBuffer {
            width: w,
            height: h,
            channels: 4,
            data,
        }
    }

    fn px(b: &PixelBuffer, p: usize) -> [u8; 3] {
        let n = b.pixel_count();
        [b.data[p], b.data[n + p], b.data[2 * n + p]]
    }

    fn varied(x: u32, y: u32) -> [u8; 3] {
        [
            ((x * 13 + y * 7) % 256) as u8,
            ((x * 5 + y * 17) % 256) as u8,
            ((x * 29 + y * 3) % 256) as u8,
        ]
    }

    #[test]
    fn crystallize_is_seeded_and_piecewise_constant() {
        let src = buf3(16, 16, varied);
        let (mut a, mut b, mut c) = (src.clone(), src.clone(), src.clone());
        crystallize(&mut a, 4, 1).unwrap();
        crystallize(&mut b, 4, 1).unwrap();
        crystallize(&mut c, 4, 2).unwrap();
        assert_eq!(a.data, b.data, "same seed must be bit-identical");
        assert_ne!(a.data, c.data, "different seed must differ");

        let src = buf3(24, 24, varied);
        let mut out = src.clone();
        crystallize(&mut out, 6, 7).unwrap();
        let mut groups: HashMap<usize, [u8; 3]> = HashMap::new();
        for (p, &g) in nearest_cells(24, 24, 6, 7).iter().enumerate() {
            let col = px(&out, p);
            match groups.get(&g) {
                Some(&e) => assert_eq!(e, col, "cell must be flat"),
                None => {
                    groups.insert(g, col);
                }
            }
        }
        assert!(groups.len() > 1, "varied input should give several cells");
    }

    #[test]
    fn pointillize_seed_is_reproducible_and_leaves_background() {
        let src = buf3(16, 16, |_, _| [255, 0, 0]);
        let (mut a, mut b, mut c) = (src.clone(), src.clone(), src.clone());
        pointillize(&mut a, 3, [0, 0, 255], 9).unwrap();
        pointillize(&mut b, 3, [0, 0, 255], 9).unwrap();
        pointillize(&mut c, 3, [0, 0, 255], 10).unwrap();
        assert_eq!(a.data, b.data, "same seed must be bit-identical");
        assert_ne!(a.data, c.data, "different seed must differ");
        let (mut red, mut blue) = (0, 0);
        for p in 0..a.pixel_count() {
            match px(&a, p) {
                [255, 0, 0] => red += 1,
                [0, 0, 255] => blue += 1,
                _ => {}
            }
        }
        assert!(red > 0, "expected source-colored dots");
        assert!(blue > 0, "expected background between dots");
    }

    #[test]
    fn color_halftone_flat_gray_is_regular_and_deterministic() {
        let src = buf3(32, 32, |_, _| [128, 128, 128]);
        let (mut a, mut b) = (src.clone(), src.clone());
        color_halftone(&mut a, 5, [0.0; 4]).unwrap();
        color_halftone(&mut b, 5, [0.0; 4]).unwrap();
        assert_eq!(a.data, b.data, "color halftone must be deterministic");
        assert!(a.data.contains(&255) && a.data.contains(&0));
        let (w, d, n) = (32usize, 10usize, 32usize * 32);
        for c in 0..3 {
            for i in 0..w - d {
                for j in 0..w - d {
                    assert_eq!(a.data[c * n + i * w + j], a.data[c * n + i * w + j + d]);
                    assert_eq!(a.data[c * n + i * w + j], a.data[c * n + (i + d) * w + j]);
                }
            }
        }
    }

    #[test]
    fn filters_preserve_alpha() {
        let base = buf4(12, 12, varied);
        let n = base.pixel_count();
        let alpha = base.data[3 * n..].to_vec();
        let (mut cr, mut pt, mut ch) = (base.clone(), base.clone(), base.clone());
        crystallize(&mut cr, 4, 1).unwrap();
        pointillize(&mut pt, 4, [1, 2, 3], 2).unwrap();
        color_halftone(&mut ch, 6, [10.0, 20.0, 30.0, 40.0]).unwrap();
        for b in [&cr, &pt, &ch] {
            assert_eq!(&b.data[3 * n..], &alpha[..], "alpha must be untouched");
        }
    }

    #[test]
    fn invalid_parameters_are_rejected() {
        let mut b = buf3(8, 8, |_, _| [128, 128, 128]);
        assert!(crystallize(&mut b, 2, 1).is_err());
        assert!(crystallize(&mut b, 301, 1).is_err());
        assert!(pointillize(&mut b, 2, [0, 0, 0], 1).is_err());
        assert!(color_halftone(&mut b, 3, [0.0; 4]).is_err());
        assert!(color_halftone(&mut b, 128, [0.0; 4]).is_err());
        assert!(color_halftone(&mut b, 4, [f64::NAN, 0.0, 0.0, 0.0]).is_err());
    }

    #[test]
    fn one_by_one_and_oversized_cells_do_not_panic() {
        let mut b = buf3(1, 1, |_, _| [200, 100, 50]);
        crystallize(&mut b, 300, 5).unwrap();
        pointillize(&mut b, 300, [0, 0, 0], 5).unwrap();
        color_halftone(&mut b, 127, [10.0, 20.0, 30.0, 40.0]).unwrap();
    }
}
