//! Pixelate texture filters: Mosaic, Facet, Fragment, Mezzotint (`FILT-084`).
//!
//! Color planes only; alpha untouched. Mosaic is an exact block mean; Facet and
//! Mezzotint are documented approximations (`ponytail:`).

use pictura_core::PixelBuffer;
use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaCha8Rng,
};

use crate::{kernel::clamp_index, luma::luma, validate, FilterError, MezzotintType};

/// Facet passes, the spread counted as "similar", and the quantization step
/// that turns a banded local mean into flat patches.
const FACET_PASSES: usize = 3;
const FACET_BAND: i32 = 16;
const FACET_STEP: i32 = 16;
/// Fragment offset, fixed per `FILT-084` (the exact amount is closed).
const FRAGMENT_DX: isize = 1;
const FRAGMENT_DY: isize = 1;

/// Average each `cell_size` block and write the mean back to every pixel in it.
pub fn mosaic(buf: &mut PixelBuffer, cell_size: u32) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if !(2..=200).contains(&cell_size) {
        return Err(FilterError::InvalidParams(format!(
            "mosaic cell size {cell_size} is outside 2..=200"
        )));
    }
    let w = buf.width as usize;
    let h = buf.height as usize;
    let cell = cell_size as usize;
    for by in (0..h).step_by(cell) {
        let y1 = (by + cell).min(h);
        for bx in (0..w).step_by(cell) {
            let x1 = (bx + cell).min(w);
            let count = (y1 - by) * (x1 - bx);
            for plane in 0..(buf.channels as usize).min(3) {
                let base = plane * n;
                let mut sum = 0u64;
                for y in by..y1 {
                    for x in bx..x1 {
                        sum += buf.data[base + y * w + x] as u64;
                    }
                }
                let mean = (sum as f64 / count as f64).round() as u8;
                for y in by..y1 {
                    for x in bx..x1 {
                        buf.data[base + y * w + x] = mean;
                    }
                }
            }
        }
    }
    Ok(())
}

/// Facet: replace a pixel with its 3x3 mean when the neighborhood is similar
/// (spread <= `FACET_BAND`); flat regions stay bit-exact and strong edges are
/// kept. The mean is quantized to `FACET_STEP` bands, which collapses a
/// gradient into patches.
///
/// ponytail: CS6's clump radius and pass count are closed; the banded 3x3 mean
/// is the behavioral model, not verified parity.
pub fn facet(buf: &mut PixelBuffer) -> Result<(), FilterError> {
    let n = validate(buf)?;
    let w = buf.width as usize;
    let h = buf.height as usize;
    for plane in 0..(buf.channels as usize).min(3) {
        let base = plane * n;
        for _ in 0..FACET_PASSES {
            let src = buf.data[base..base + n].to_vec();
            for y in 0..h {
                for x in 0..w {
                    let (mut lo, mut hi, mut sum) = (255i32, 0i32, 0i32);
                    for dy in 0..3 {
                        let sy = clamp_index(y as isize + dy as isize - 1, h);
                        for dx in 0..3 {
                            let sx = clamp_index(x as isize + dx as isize - 1, w);
                            let v = src[sy * w + sx] as i32;
                            (lo, hi, sum) = (lo.min(v), hi.max(v), sum + v);
                        }
                    }
                    if hi - lo == 0 || hi - lo > FACET_BAND {
                        continue;
                    }
                    let q = (((sum + 4) / 9 + FACET_STEP / 2) / FACET_STEP) * FACET_STEP;
                    buf.data[base + y * w + x] = q.clamp(0, 255) as u8;
                }
            }
        }
    }
    Ok(())
}

/// Average the source with three copies offset by `(DX, DY)` — a deterministic
/// 4-tap ghost. A flat region is a bit-exact no-op.
pub fn fragment(buf: &mut PixelBuffer) -> Result<(), FilterError> {
    let n = validate(buf)?;
    let w = buf.width as usize;
    let h = buf.height as usize;
    let offsets = [
        (0isize, 0isize),
        (FRAGMENT_DX, 0),
        (0, FRAGMENT_DY),
        (FRAGMENT_DX, FRAGMENT_DY),
    ];
    for plane in 0..(buf.channels as usize).min(3) {
        let base = plane * n;
        let src = buf.data[base..base + n].to_vec();
        for y in 0..h {
            for x in 0..w {
                let mut sum = 0u32;
                for &(ox, oy) in &offsets {
                    let sx = clamp_index(x as isize + ox, w);
                    let sy = clamp_index(y as isize + oy, h);
                    sum += src[sy * w + sx] as u32;
                }
                buf.data[base + y * w + x] = ((sum + 2) / 4) as u8;
            }
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum Shape {
    Dots,
    Lines,
    Strokes,
}

/// Dot scales 2/4/8/16; lines and strokes 3/6/12.
fn kind_params(kind: MezzotintType) -> (usize, Shape) {
    use MezzotintType::*;
    match kind {
        FineDots => (2, Shape::Dots),
        MediumDots => (4, Shape::Dots),
        GrainyDots => (8, Shape::Dots),
        CoarseDots => (16, Shape::Dots),
        ShortLines => (3, Shape::Lines),
        MediumLines => (6, Shape::Lines),
        LongLines => (12, Shape::Lines),
        ShortStrokes => (3, Shape::Strokes),
        MediumStrokes => (6, Shape::Strokes),
        LongStrokes => (12, Shape::Strokes),
    }
}

fn triangle(phase: usize, period: usize) -> u8 {
    let p = phase % period;
    let half = (period - 1).max(1);
    ((if p <= half { p } else { period - p } * 255) / half).min(255) as u8
}

/// Structured threshold field for the dot / line / stroke families.
fn base_threshold(x: usize, y: usize, cell: usize, shape: Shape) -> u8 {
    match shape {
        Shape::Dots => {
            let cx = (x / cell) * cell + cell / 2;
            let cy = (y / cell) * cell + cell / 2;
            let d = (x as isize - cx as isize)
                .unsigned_abs()
                .max((y as isize - cy as isize).unsigned_abs());
            ((d * 255) / (cell / 2).max(1)).min(255) as u8
        }
        Shape::Lines => triangle(x + y, cell),
        Shape::Strokes => triangle(x + 2 * y, cell),
    }
}

/// Seeded dot / line / stroke pattern: black or white driven by source luma
/// over a procedural threshold. The same seed is bit-identical.
///
/// ponytail: outputs a grayscale pattern (all planes equal) rather than CS6's
/// saturated-color variant; swap in a per-plane threshold if color matters.
pub fn mezzotint(buf: &mut PixelBuffer, kind: MezzotintType, seed: u64) -> Result<(), FilterError> {
    let n = validate(buf)?;
    let w = buf.width as usize;
    let h = buf.height as usize;
    let (cell, shape) = kind_params(kind);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    for y in 0..h {
        for x in 0..w {
            let p = y * w + x;
            let l = luma(
                buf.data[p] as f64,
                buf.data[n + p] as f64,
                buf.data[2 * n + p] as f64,
            );
            let threshold = base_threshold(x, y, cell, shape).wrapping_add(rng.next_u32() as u8);
            let v = if l as i32 >= threshold as i32 { 255 } else { 0 };
            for plane in 0..(buf.channels as usize).min(3) {
                buf.data[plane * n + p] = v;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn mk(w: u32, h: u32, ch: u8, f: impl Fn(usize, usize) -> [u8; 3]) -> PixelBuffer {
        let n = (w * h) as usize;
        let mut data = vec![0u8; n * ch as usize];
        for y in 0..h as usize {
            for x in 0..w as usize {
                for (k, &v) in f(x, y).iter().enumerate() {
                    data[k * n + y * w as usize + x] = v;
                }
                if ch == 4 {
                    data[3 * n + y * w as usize + x] = 200;
                }
            }
        }
        PixelBuffer {
            width: w,
            height: h,
            channels: ch,
            data,
        }
    }

    fn buf3(w: u32, h: u32, f: impl Fn(usize, usize) -> [u8; 3]) -> PixelBuffer {
        mk(w, h, 3, f)
    }

    #[test]
    fn mosaic_blocks_are_uniform_and_equal_the_exact_mean() {
        let w = 4usize;
        let mut b = buf3(4, 4, |x, y| [(x + 10 * y) as u8; 3]);
        let sd = b.data.clone();
        mosaic(&mut b, 2).unwrap();
        for (by, bx) in [(0, 0), (0, 2), (2, 0), (2, 2)] {
            let mut sum = 0u64;
            for y in by..by + 2 {
                for x in bx..bx + 2 {
                    sum += sd[y * w + x] as u64;
                }
            }
            let mean = (sum as f64 / 4.0).round() as u8;
            for y in by..by + 2 {
                for x in bx..bx + 2 {
                    assert_eq!(b.data[y * w + x], mean, "block ({bx},{by})");
                }
            }
        }
        let mut tiny = buf3(2, 2, |_, _| [0; 3]);
        assert!(mosaic(&mut tiny, 1).is_err() && mosaic(&mut tiny, 201).is_err());
        assert!(mosaic(&mut tiny, 2).is_ok());
    }

    #[test]
    fn fragment_averages_the_four_offset_copies() {
        let w = 4usize;
        let mut b = buf3(4, 4, |x, y| [(x * 10 + y) as u8; 3]);
        let src = b.data.clone();
        fragment(&mut b).unwrap();
        for y in 0..4 {
            for x in 0..4 {
                let sum: u32 = [(0isize, 0isize), (1, 0), (0, 1), (1, 1)]
                    .iter()
                    .map(|&(ox, oy)| {
                        src[clamp_index(y as isize + oy, 4) * w + clamp_index(x as isize + ox, w)]
                            as u32
                    })
                    .sum();
                assert_eq!(b.data[y * w + x], ((sum + 2) / 4) as u8, "at ({x},{y})");
            }
        }
        let mut flat = buf3(4, 4, |_, _| [77, 88, 99]);
        let before = flat.data.clone();
        fragment(&mut flat).unwrap();
        assert_eq!(flat.data, before, "flat region must be a no-op");
    }

    #[test]
    fn facet_flattens_a_gradient_and_keeps_a_strong_edge() {
        let w = 64usize;
        let distinct = |b: &PixelBuffer| b.data[..w].iter().collect::<BTreeSet<_>>().len();
        let mut grad = buf3(w as u32, 1, |x, _| {
            [(255.0 * x as f64 / (w - 1) as f64).round() as u8; 3]
        });
        let before = distinct(&grad);
        facet(&mut grad).unwrap();
        let after = distinct(&grad);
        assert!(after < before, "facet must flatten ({before} -> {after})");
        let mut edge = buf3(16, 1, |x, _| if x < 8 { [30; 3] } else { [220; 3] });
        facet(&mut edge).unwrap();
        assert!(
            edge.data.contains(&30) && edge.data.contains(&220),
            "both sides of a strong edge must survive"
        );
    }

    #[test]
    fn mezzotint_is_seed_deterministic_and_kind_sensitive() {
        let base = buf3(8, 8, |_, _| [128, 128, 128]);
        let (mut a, mut b, mut c, mut d) = (base.clone(), base.clone(), base.clone(), base.clone());
        mezzotint(&mut a, MezzotintType::FineDots, 7).unwrap();
        mezzotint(&mut b, MezzotintType::FineDots, 7).unwrap();
        mezzotint(&mut c, MezzotintType::FineDots, 8).unwrap();
        mezzotint(&mut d, MezzotintType::CoarseDots, 7).unwrap();
        assert_eq!(a.data, b.data, "same seed must be bit-identical");
        assert_ne!(a.data, c.data, "different seed must differ");
        assert_ne!(a.data, d.data, "different kind must differ");
        let n = a.pixel_count();
        assert!((0..n).all(|p| a.data[p] == a.data[n + p] && a.data[p] == a.data[2 * n + p]));
    }

    #[test]
    fn all_four_filters_preserve_alpha() {
        let base = mk(6, 5, 4, |x, y| [(x * 10 + y) as u8; 3]);
        let n = base.pixel_count();
        let alpha = base.data[3 * n..].to_vec();
        let (mut m, mut f, mut g, mut z) = (base.clone(), base.clone(), base.clone(), base.clone());
        mosaic(&mut m, 3).unwrap();
        facet(&mut f).unwrap();
        fragment(&mut g).unwrap();
        mezzotint(&mut z, MezzotintType::MediumLines, 5).unwrap();
        for b in [&m, &f, &g, &z] {
            assert_eq!(&b.data[3 * n..], &alpha[..], "alpha must be untouched");
        }
    }

    #[test]
    fn tiny_and_oversized_cells_do_not_panic() {
        for &(w, h) in &[(1u32, 1u32), (2, 3)] {
            let base = buf3(w, h, |x, y| [(x + y) as u8; 3]);
            for cell in [2, 200] {
                let mut m = base.clone();
                assert!(mosaic(&mut m, cell).is_ok(), "{w}x{h} mosaic {cell}");
            }
            let (mut f, mut g, mut z) = (base.clone(), base.clone(), base.clone());
            assert!(facet(&mut f).is_ok());
            assert!(fragment(&mut g).is_ok());
            assert!(mezzotint(&mut z, MezzotintType::LongStrokes, 1).is_ok());
        }
    }
}
