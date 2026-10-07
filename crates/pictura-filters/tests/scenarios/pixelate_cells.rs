//! Spec scenarios from the replaced `pixelate/cells.rs` unit tests, run through
//! `apply` against the ported photorust engine.

use super::shims::*;
use pictura_core::PixelBuffer;

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
    // Cells are flat: far fewer colours than pixels, but more than one.
    let colours: std::collections::HashSet<[u8; 3]> = (0..24 * 24).map(|p| px(&out, p)).collect();
    assert!(colours.len() > 1, "varied input should give several cells");
    assert!(
        colours.len() < 24 * 24 / 6,
        "cells must be flat: {} colours",
        colours.len()
    );
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
