//! The built-in patterns the Pattern Stamp paints with.
//!
//! CS6 ships its patterns as Adobe artwork in `.pat` files; these are
//! **generated** instead and named for what they are. Every tile is square,
//! opaque, greyscale, and seamless: the generators wrap by construction
//! (positions taken modulo the tile), so a repeat shows no join.
//!
//! Ported from photorust's `core/src/pattern.rs`
//! (<https://github.com/perfecto25/photorust>).

use crate::healing::RgbaImage;

/// Side of every generated tile, in pixels.
pub const TILE: i32 = 64;

/// The built-in patterns, in picker order.
pub const PATTERN_NAMES: [&str; 8] = [
    "Checkerboard",
    "Grid",
    "Diagonal Stripes",
    "Horizontal Lines",
    "Polka Dots",
    "Woven",
    "Bricks",
    "Grain",
];

/// The pattern at `index`, or `None` past the end of [`PATTERN_NAMES`].
pub fn tile(index: usize) -> Option<RgbaImage> {
    let shade: fn(i32, i32) -> u8 = match index {
        0 => checkerboard,
        1 => grid,
        2 => diagonal_stripes,
        3 => horizontal_lines,
        4 => return Some(polka_dots()),
        5 => woven,
        6 => bricks,
        7 => grain,
        _ => return None,
    };
    let mut img = blank();
    for y in 0..TILE {
        for x in 0..TILE {
            let v = shade(x, y);
            img.set(x, y, [v, v, v, 255]);
        }
    }
    Some(img)
}

fn blank() -> RgbaImage {
    RgbaImage {
        width: TILE,
        height: TILE,
        data: vec![[0, 0, 0, 255]; (TILE * TILE) as usize],
    }
}

fn checkerboard(x: i32, y: i32) -> u8 {
    let half = TILE / 2;
    if (x < half) ^ (y < half) {
        90
    } else {
        215
    }
}

// Lines on two edges only: the neighbouring tile supplies the other two, so a
// repeated grid has single-width lines.
fn grid(x: i32, y: i32) -> u8 {
    if x < 2 || y < 2 {
        105
    } else {
        220
    }
}

// The period divides the tile, so the 45° stripe meets itself at every edge.
fn diagonal_stripes(x: i32, y: i32) -> u8 {
    let period = 16;
    if (x + y).rem_euclid(period) < period / 2 {
        110
    } else {
        210
    }
}

fn horizontal_lines(_x: i32, y: i32) -> u8 {
    if y.rem_euclid(8) < 3 {
        115
    } else {
        220
    }
}

// Alternating cells put the horizontal thread on top, which reads as weaving
// rather than as a grid.
fn woven(x: i32, y: i32) -> u8 {
    let cell = 16;
    let (cx, cy) = (x.rem_euclid(cell), y.rem_euclid(cell));
    let horizontal_on_top = (x / cell + y / cell) % 2 == 0;
    let in_horizontal = cy >= 3 && cy < cell - 3;
    let in_vertical = cx >= 3 && cx < cell - 3;
    if (horizontal_on_top && in_horizontal) || (!horizontal_on_top && in_vertical) {
        190
    } else if in_horizontal || in_vertical {
        140
    } else {
        95
    }
}

// Running bond: every other course shifts half a brick, a whole number of
// tiles over two courses, so the tile still meets itself.
fn bricks(x: i32, y: i32) -> u8 {
    let (bw, bh, mortar) = (32, 16, 3);
    let shift = if (y / bh) % 2 == 0 { 0 } else { bw / 2 };
    if y.rem_euclid(bh) < mortar || (x + shift).rem_euclid(bw) < mortar {
        205
    } else {
        120
    }
}

// A fixed hash, not a generator: the tile must come out the same every time or
// a repainted stroke would not match itself.
fn grain(x: i32, y: i32) -> u8 {
    let mut h = (x as u32).wrapping_mul(0x9E37_79B9) ^ (y as u32).wrapping_mul(0x85EB_CA6B);
    h ^= h >> 13;
    h = h.wrapping_mul(0xC2B2_AE35);
    h ^= h >> 16;
    150 + (h % 80) as u8
}

// Four dots on a staggered grid, measured on the torus so a dot on an edge
// appears on both sides.
fn polka_dots() -> RgbaImage {
    let mut img = blank();
    let radius = 9.0;
    let centres = [(16.0, 16.0), (48.0, 16.0), (32.0, 48.0), (0.0, 48.0)];
    for y in 0..TILE {
        for x in 0..TILE {
            let mut v = 220.0f32;
            for (cx, cy) in centres {
                let dx = wrapped_delta(x as f32 + 0.5 - cx);
                let dy = wrapped_delta(y as f32 + 0.5 - cy);
                let cover = (radius + 0.5 - (dx * dx + dy * dy).sqrt()).clamp(0.0, 1.0);
                v += (100.0 - v) * cover;
            }
            let v = v.round() as u8;
            img.set(x, y, [v, v, v, 255]);
        }
    }
    img
}

/// The shorter way round the tile between two coordinates.
fn wrapped_delta(d: f32) -> f32 {
    let side = TILE as f32;
    let d = d % side;
    if d > side / 2.0 {
        d - side
    } else if d < -side / 2.0 {
        d + side
    } else {
        d
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pattern_is_an_opaque_seamless_textured_tile() {
        for (index, name) in PATTERN_NAMES.iter().enumerate() {
            let t = tile(index).expect("a listed pattern renders");
            assert_eq!((t.width, t.height), (TILE, TILE));
            let (lo, hi) = t.data.iter().fold((255u8, 0u8), |(lo, hi), px| {
                assert_eq!(px[3], 255, "{name} has a transparent pixel");
                (lo.min(px[0]), hi.max(px[0]))
            });
            assert!(hi - lo > 30, "{name} is nearly flat: {lo}..{hi}");
            // A seam may be no sharper than the steps the tile already makes one
            // pixel in (a hard-edged checkerboard jumps at its own boundaries).
            let r = |x: i32, y: i32| t.get(x, y)[0] as i32;
            for i in 0..TILE {
                let across = (r(0, i) - r(TILE - 1, i)).abs() - (r(1, i) - r(0, i)).abs();
                let down = (r(i, 0) - r(i, TILE - 1)).abs() - (r(i, 1) - r(i, 0)).abs();
                assert!(across.max(down) <= 130, "{name} does not wrap");
            }
        }
        assert!(tile(PATTERN_NAMES.len()).is_none());
    }
}
