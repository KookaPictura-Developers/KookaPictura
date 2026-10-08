//! Spec scenarios from the replaced `pixelate/texture.rs` unit tests, run through
//! `apply` against the ported photorust engine.

use super::shims::*;
use pictura_core::PixelBuffer;
use pictura_filters::*;

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
        data: data.into(),
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
fn facet_flattens_a_gradient_and_keeps_a_strong_edge() {
    let w = 64usize;
    let distinct = |b: &PixelBuffer| b.data[..w].iter().collect::<BTreeSet<_>>().len();
    // CS6 clumps similarly coloured pixels: speckle collapses into patches.
    let mut grad = buf3(w as u32, 16, |x, y| {
        [(120 + ((x * 7 + y * 13) % 9) as i32 - 4) as u8; 3]
    });
    let before = distinct(&grad);
    facet(&mut grad).unwrap();
    let after = distinct(&grad);
    assert!(
        after < before,
        "facet must clump speckle ({before} -> {after})"
    );
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
