//! Stylize ▸ Extrude: break the picture into towers standing out of the frame.
//!
//! Ported from photorust's `core/src/filters/stylize.rs` (GPL-3.0-or-later;
//! see the change proposal).

use pictura_core::PixelBuffer;

use crate::kernel::{bilinear, Edge};
use crate::luma::luma;
use crate::{validate, ExtrudeType, FilterError};

/// How far the furthest tower is thrown outwards, as a fraction of its
/// distance from the middle of the frame, at full Depth.
const THROW: f32 = 0.85;
/// Light on each face in the order [top, right, bottom, left].
const FACES: [f32; 4] = [1.30, 0.78, 0.60, 1.12];

#[derive(Clone, Copy)]
struct Rect {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

pub fn extrude(
    buf: &mut PixelBuffer,
    kind: ExtrudeType,
    size: u32,
    depth: f32,
    level_based: bool,
    solid_front: bool,
    mask_incomplete: bool,
) -> Result<(), FilterError> {
    validate(buf)?;
    if !(2..=255).contains(&size) {
        return Err(FilterError::InvalidParams(format!(
            "extrude size {size} is outside 2..=255"
        )));
    }
    if !depth.is_finite() || depth < 1.0 || depth > 255.0 {
        return Err(FilterError::InvalidParams(format!(
            "extrude depth {depth} is outside 1..=255"
        )));
    }
    let size = size as i32;
    let w = buf.width as usize;
    let h = buf.height as usize;
    let n = w * h;
    let planes = (buf.channels as usize).min(3);
    let source = buf.clone();
    let middle = (w as f32 / 2.0, h as f32 / 2.0);
    let throw = depth / 255.0 * THROW;

    // Every tile with how far it stands, drawn back to front so a nearer tower
    // covers what is behind it.
    let mut towers: Vec<(f32, Rect, [u8; 3])> = Vec::new();
    let mut y = 0u32;
    while y < buf.height {
        let mut x = 0u32;
        while x < buf.width {
            let rect = Rect {
                x: x as i32,
                y: y as i32,
                w: size.min((buf.width - x) as i32),
                h: size.min((buf.height - y) as i32),
            };
            let whole = rect.w == size && rect.h == size;
            if whole || !mask_incomplete {
                let average = average_of(&source, n, rect);
                let stands = if level_based {
                    (luma(average[0] as f64, average[1] as f64, average[2] as f64) / 255.0) as f32
                } else {
                    hash(x, y) as f32 / u32::MAX as f32
                };
                towers.push((stands, rect, average));
            }
            x += size as u32;
        }
        y += size as u32;
    }
    towers.sort_by(|a, b| a.0.total_cmp(&b.0));

    for (stands, rect, average) in towers {
        let grow = 1.0 + throw * stands;
        let out = |px: f32, py: f32| {
            (
                middle.0 + (px - middle.0) * grow,
                middle.1 + (py - middle.1) * grow,
            )
        };
        let (x0, y0) = (rect.x as f32, rect.y as f32);
        let (x1, y1) = (x0 + rect.w as f32, y0 + rect.h as f32);
        let base = [(x0, y0), (x1, y0), (x1, y1), (x0, y1)];
        let front = base.map(|(px, py)| out(px, py));

        match kind {
            ExtrudeType::Blocks => {
                for edge in 0..4 {
                    let next = (edge + 1) % 4;
                    fill_convex(
                        buf,
                        w,
                        h,
                        n,
                        planes,
                        &[base[edge], base[next], front[next], front[edge]],
                        shade(average, FACES[edge]),
                    );
                }
                if solid_front {
                    fill_convex(buf, w, h, n, planes, &front, average);
                } else {
                    face_of_picture(buf, &source, n, planes, rect, front[0], front[2]);
                }
            }
            ExtrudeType::Pyramids => {
                let apex = out((x0 + x1) / 2.0, (y0 + y1) / 2.0);
                for edge in 0..4 {
                    let next = (edge + 1) % 4;
                    fill_convex(
                        buf,
                        w,
                        h,
                        n,
                        planes,
                        &[base[edge], base[next], apex],
                        shade(average, FACES[edge]),
                    );
                }
            }
        }
    }
    Ok(())
}

fn set_rgb(buf: &mut PixelBuffer, n: usize, planes: usize, x: usize, y: usize, c: [u8; 3]) {
    let i = y * buf.width as usize + x;
    for (p, &v) in c.iter().enumerate().take(planes) {
        buf.data[p * n + i] = v;
    }
}

fn average_of(source: &PixelBuffer, n: usize, rect: Rect) -> [u8; 3] {
    let mut sum = [0u64; 3];
    let mut count = 0u64;
    for y in rect.y..rect.y + rect.h {
        for x in rect.x..rect.x + rect.w {
            let i = y as usize * source.width as usize + x as usize;
            for (c, s) in sum.iter_mut().enumerate() {
                *s += source.data[c * n + i] as u64;
            }
            count += 1;
        }
    }
    if count == 0 {
        return [0, 0, 0];
    }
    [
        (sum[0] / count) as u8,
        (sum[1] / count) as u8,
        (sum[2] / count) as u8,
    ]
}

fn shade(c: [u8; 3], by: f32) -> [u8; 3] {
    let level = |v: u8| (v as f32 * by).clamp(0.0, 255.0) as u8;
    [level(c[0]), level(c[1]), level(c[2])]
}

fn face_of_picture(
    buf: &mut PixelBuffer,
    source: &PixelBuffer,
    n: usize,
    planes: usize,
    rect: Rect,
    top_left: (f32, f32),
    bottom_right: (f32, f32),
) {
    let (fx0, fy0) = top_left;
    let (fx1, fy1) = bottom_right;
    let (span_x, span_y) = (fx1 - fx0, fy1 - fy0);
    if span_x <= 0.0 || span_y <= 0.0 {
        return;
    }
    let w = buf.width as usize;
    let h = buf.height as usize;
    let from = (fy0.floor().max(0.0) as i32).max(0);
    let to = (fy1.ceil() as i32).min(h as i32);
    let left = (fx0.floor().max(0.0) as i32).max(0);
    let right = (fx1.ceil() as i32).min(w as i32);
    for y in from..to {
        let v = (y as f32 + 0.5 - fy0) / span_y;
        if !(0.0..1.0).contains(&v) {
            continue;
        }
        for x in left..right {
            let u = (x as f32 + 0.5 - fx0) / span_x;
            if !(0.0..1.0).contains(&u) {
                continue;
            }
            let sx = rect.x as f32 + u * rect.w as f32 - 0.5;
            let sy = rect.y as f32 + v * rect.h as f32 - 0.5;
            let mut c = [0u8; 3];
            for (p, slot) in c.iter_mut().enumerate().take(planes) {
                let plane = &source.data[p * n..p * n + n];
                *slot = bilinear(plane, w, h, sx as f64, sy as f64, Edge::Clamp)
                    .round()
                    .clamp(0.0, 255.0) as u8;
            }
            set_rgb(buf, n, planes, x as usize, y as usize, c);
        }
    }
}

fn fill_convex(
    buf: &mut PixelBuffer,
    w: usize,
    h: usize,
    n: usize,
    planes: usize,
    points: &[(f32, f32)],
    colour: [u8; 3],
) {
    if points.len() < 3 {
        return;
    }
    let top = points.iter().fold(f32::MAX, |a, p| a.min(p.1));
    let bottom = points.iter().fold(f32::MIN, |a, p| a.max(p.1));
    let first = (top.floor() as i32).max(0);
    let last = (bottom.ceil() as i32).min(h as i32);
    for y in first..last {
        let scan = y as f32 + 0.5;
        let (mut left, mut right) = (f32::MAX, f32::MIN);
        for (i, &(ax, ay)) in points.iter().enumerate() {
            let (bx, by) = points[(i + 1) % points.len()];
            if (ay <= scan) == (by <= scan) {
                continue;
            }
            let t = (scan - ay) / (by - ay);
            let x = ax + (bx - ax) * t;
            left = left.min(x);
            right = right.max(x);
        }
        if left > right {
            continue;
        }
        let from = (left.round() as i32).max(0);
        let to = (right.round() as i32).min(w as i32);
        for x in from..to {
            set_rgb(buf, n, planes, x as usize, y as usize, colour);
        }
    }
}

fn hash(x: u32, y: u32) -> u32 {
    let mut h = x.wrapping_mul(0x27d4_eb2d) ^ y.wrapping_mul(0x1656_67b1);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2545_f491);
    h ^= h >> 13;
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(
        w: u32,
        h: u32,
        channels: u8,
        px: &[[u8; 3]],
        alpha: impl Fn(usize) -> u8,
    ) -> PixelBuffer {
        let n = (w * h) as usize;
        assert_eq!(px.len(), n);
        let mut data = vec![0u8; n * channels as usize];
        for (i, p) in px.iter().enumerate() {
            data[i] = p[0];
            data[n + i] = p[1];
            data[2 * n + i] = p[2];
            if channels == 4 {
                data[3 * n + i] = alpha(i);
            }
        }
        PixelBuffer {
            width: w,
            height: h,
            channels,
            data: data.into(),
        }
    }

    fn rgb_at(buf: &PixelBuffer, x: usize, y: usize) -> [u8; 3] {
        let n = buf.pixel_count();
        let i = y * buf.width as usize + x;
        [buf.data[i], buf.data[n + i], buf.data[2 * n + i]]
    }

    #[test]
    fn extrude_changes_the_image_and_is_deterministic() {
        let px: Vec<[u8; 3]> = (0..64)
            .map(|i| {
                let v = (i * 4) as u8;
                [v, v.wrapping_add(20), v.wrapping_add(40)]
            })
            .collect();
        let base = image(8, 8, 3, &px, |_| 0);
        let mut a = base.clone();
        extrude(&mut a, ExtrudeType::Blocks, 3, 30.0, false, false, false).unwrap();
        let mut b = base.clone();
        extrude(&mut b, ExtrudeType::Blocks, 3, 30.0, false, false, false).unwrap();
        assert_eq!(a.data, b.data, "random heights come from the position hash");
        assert_ne!(a.data, base.data, "extrude must redraw the picture");
    }

    #[test]
    fn extrude_level_based_stands_bright_tiles_tallest() {
        // Left tile dark, middle tile bright, right tile mid. Level-based
        // stands them by brightness, so the bright tile is drawn last and its
        // solid front covers the overlap it makes with the dark neighbour.
        let mut px = Vec::new();
        for _ in 0..4 {
            px.extend_from_slice(&[
                [50, 50, 50],
                [50, 50, 50],
                [220, 220, 220],
                [220, 220, 220],
                [130, 130, 130],
                [130, 130, 130],
            ]);
        }
        let base = image(6, 4, 3, &px, |_| 0);
        let mut out = base.clone();
        extrude(&mut out, ExtrudeType::Blocks, 2, 255.0, true, true, false).unwrap();
        assert_eq!(
            rgb_at(&out, 1, 1),
            [220, 220, 220],
            "the taller bright tower paints over its dark neighbour"
        );
    }

    #[test]
    fn extrude_rejects_bad_depth_and_empty_buffer() {
        let base = image(4, 4, 3, &[[10, 20, 30]; 16], |_| 0);
        for depth in [f32::NAN, f32::INFINITY, 0.0, 0.5, -5.0, 256.0] {
            let mut out = base.clone();
            assert!(
                extrude(&mut out, ExtrudeType::Blocks, 3, depth, false, false, false).is_err(),
                "depth {depth} must be rejected"
            );
            assert_eq!(out, base, "rejected parameters must not modify the buffer");
        }
        for size in [1u32, 256] {
            let mut out = base.clone();
            assert!(
                extrude(
                    &mut out,
                    ExtrudeType::Blocks,
                    size,
                    30.0,
                    false,
                    false,
                    false
                )
                .is_err(),
                "size {size} must be rejected"
            );
            assert_eq!(out, base, "rejected parameters must not modify the buffer");
        }
        let mut empty = PixelBuffer::new(0, 0, 3);
        assert!(extrude(
            &mut empty,
            ExtrudeType::Blocks,
            3,
            30.0,
            false,
            false,
            false
        )
        .is_err());
    }

    #[test]
    fn extrude_survives_a_tower_thrown_past_the_left_edge() {
        // A bright tile in the lower-left corner grows 1.85x from the middle,
        // throwing its left face entirely to negative x. The face's scanline
        // has left < right but both negative, so the old usize cast exploded.
        let mut px = vec![[0u8, 0, 0]; 8 * 16];
        for y in 8..12usize {
            for x in 0..4usize {
                px[y * 8 + x] = [255, 255, 255];
            }
        }
        let base = image(8, 16, 3, &px, |_| 0);
        let mut out = base.clone();
        extrude(&mut out, ExtrudeType::Blocks, 4, 255.0, true, true, false).unwrap();
    }

    #[test]
    fn mask_incomplete_leaves_partial_tiles_as_source() {
        let px: Vec<[u8; 3]> = (0..77)
            .map(|i| {
                let v = (i * 3) as u8;
                [v, v.wrapping_add(40), v.wrapping_add(80)]
            })
            .collect();
        let base = image(11, 7, 3, &px, |_| 0);
        let n = base.pixel_count();
        let mut masked = base.clone();
        extrude(&mut masked, ExtrudeType::Blocks, 4, 1.0, false, true, true).unwrap();
        let mut unmasked = base.clone();
        extrude(
            &mut unmasked,
            ExtrudeType::Blocks,
            4,
            1.0,
            false,
            true,
            false,
        )
        .unwrap();
        let mut changed = false;
        for y in 0..7usize {
            for x in 0..11usize {
                if x >= 8 || y >= 4 {
                    let i = y * 11 + x;
                    assert_eq!(masked.data[i], base.data[i], "masked R at ({x},{y})");
                    assert_eq!(
                        masked.data[n + i],
                        base.data[n + i],
                        "masked G at ({x},{y})"
                    );
                    assert_eq!(
                        masked.data[2 * n + i],
                        base.data[2 * n + i],
                        "masked B at ({x},{y})"
                    );
                    if unmasked.data[i] != base.data[i] {
                        changed = true;
                    }
                }
            }
        }
        assert!(
            changed,
            "mask_incomplete=false must throw the partial tiles"
        );
    }

    #[test]
    fn solid_front_paints_the_cell_average() {
        let cell = [
            [10u8, 20, 30],
            [90, 100, 110],
            [130, 140, 150],
            [170, 180, 190],
        ];
        let mut px = vec![[0u8, 0, 0]; 16];
        px[0] = cell[0];
        px[1] = cell[1];
        px[4] = cell[2];
        px[5] = cell[3];
        let base = image(4, 4, 3, &px, |_| 0);
        let avg = [100u8, 110, 120];
        let n = base.pixel_count();
        assert!(
            !(0..n).any(|i| base.data[i] == avg[0]
                && base.data[n + i] == avg[1]
                && base.data[2 * n + i] == avg[2]),
            "the cell mean must not already exist in the source"
        );
        let mut out = base.clone();
        extrude(&mut out, ExtrudeType::Blocks, 2, 30.0, true, true, false).unwrap();
        assert!(
            (0..n).any(|i| out.data[i] == avg[0]
                && out.data[n + i] == avg[1]
                && out.data[2 * n + i] == avg[2]),
            "solid front must paint the exact per-cell mean"
        );
    }

    #[test]
    fn pyramids_geometry_differs_from_blocks_and_keeps_alpha() {
        let px: Vec<[u8; 3]> = (0..64)
            .map(|i| {
                let v = (i * 3) as u8;
                [v, v.wrapping_add(20), v.wrapping_add(40)]
            })
            .collect();
        let base = image(8, 8, 4, &px, |i| (i * 5) as u8);
        let n = base.pixel_count();
        let alpha = base.data[3 * n..].to_vec();
        let mut blocks = base.clone();
        extrude(
            &mut blocks,
            ExtrudeType::Blocks,
            4,
            200.0,
            false,
            true,
            false,
        )
        .unwrap();
        let mut pyramids = base.clone();
        extrude(
            &mut pyramids,
            ExtrudeType::Pyramids,
            4,
            200.0,
            false,
            true,
            false,
        )
        .unwrap();
        assert_ne!(blocks.data, pyramids.data, "pyramids are not blocks");
        assert_eq!(&pyramids.data[3 * n..], &alpha[..]);
    }

    #[test]
    fn extrude_preserves_alpha_and_survives_tiny_and_ragged_tiles() {
        let px: Vec<[u8; 3]> = (0..77)
            .map(|i| [(i * 3) as u8, (i * 5) as u8, (i * 7) as u8])
            .collect();
        let base = image(11, 7, 4, &px, |i| (i * 11) as u8);
        let n = base.pixel_count();
        let alpha = base.data[3 * n..].to_vec();
        for (kind, mask) in [(ExtrudeType::Blocks, false), (ExtrudeType::Pyramids, true)] {
            let mut out = base.clone();
            extrude(&mut out, kind, 4, 200.0, true, true, mask).unwrap();
            assert_eq!(&out.data[3 * n..], &alpha[..]);
        }
    }
}
