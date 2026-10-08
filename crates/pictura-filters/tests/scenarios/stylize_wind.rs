//! Spec scenarios from the replaced `stylize/wind.rs` unit tests, run through
//! `apply` against the ported photorust engine.

use super::shims::*;
use pictura_core::PixelBuffer;
use pictura_filters::*;

const W: u32 = 24;
const H: u32 = 16;

fn block(channels: u8, alpha: impl Fn(usize) -> u8) -> PixelBuffer {
    let n = (W * H) as usize;
    let mut data = vec![0u8; n * channels as usize];
    for y in 0..H {
        for x in 0..W {
            let i = (y * W + x) as usize;
            let v: u8 = if (8..16).contains(&x) && (4..12).contains(&y) {
                220
            } else {
                0
            };
            data[i] = v;
            data[n + i] = v;
            data[2 * n + i] = v;
            if channels == 4 {
                data[3 * n + i] = alpha(i);
            }
        }
    }
    PixelBuffer {
        width: W,
        height: H,
        channels,
        data: data.into(),
    }
}

fn changed_before(base: &PixelBuffer, out: &PixelBuffer, limit: u32) -> bool {
    let w = W as usize;
    (0..limit as usize)
        .any(|x| (0..H as usize).any(|y| base.data[y * w + x] != out.data[y * w + x]))
}

fn changed_after(base: &PixelBuffer, out: &PixelBuffer, start: u32) -> bool {
    let w = W as usize;
    (start as usize..W as usize)
        .any(|x| (0..H as usize).any(|y| base.data[y * w + x] != out.data[y * w + x]))
}

#[test]
fn wind_streaks_downwind_and_leaves_the_upwind_side() {
    let base = block(3, |_| 0);
    let mut right = base.clone();
    wind(&mut right, WindMethod::Blast, false).unwrap();
    assert!(
        changed_after(&base, &right, 16),
        "blows streaks to the right"
    );
    assert!(
        !changed_before(&base, &right, 8),
        "the upwind side stays clean"
    );

    let mut left = base.clone();
    wind(&mut left, WindMethod::Blast, true).unwrap();
    assert!(changed_before(&base, &left, 8), "from the right blows left");
    assert!(!changed_after(&base, &left, 16), "the far side stays clean");
}

#[test]
fn stagger_drifts_rows_away_from_plain_wind() {
    let base = block(3, |_| 0);
    let mut plain = base.clone();
    wind(&mut plain, WindMethod::Wind, false).unwrap();
    let mut stagger = base.clone();
    wind(&mut stagger, WindMethod::Stagger, false).unwrap();
    assert_ne!(stagger.data, base.data, "stagger still blows streaks");
    assert_ne!(stagger.data, plain.data, "stagger drifts the sampled rows");
}

#[test]
fn wind_flat_ground_is_untouched() {
    let n = (W * H) as usize;
    let mut flat = PixelBuffer::new(W, H, 3);
    for v in flat.data[..n].iter_mut() {
        *v = 90;
    }
    for v in flat.data[n..2 * n].iter_mut() {
        *v = 90;
    }
    for v in flat.data[2 * n..].iter_mut() {
        *v = 90;
    }
    let base = flat.clone();
    for method in [WindMethod::Wind, WindMethod::Blast, WindMethod::Stagger] {
        let mut out = base.clone();
        wind(&mut out, method, false).unwrap();
        assert_eq!(out, base, "{method:?} must not touch flat ground");
    }
}

#[test]
fn wind_is_deterministic_and_preserves_alpha() {
    let base = block(4, |i| (i * 7) as u8);
    let n = base.pixel_count();
    let alpha = base.data[3 * n..].to_vec();
    let mut a = base.clone();
    wind(&mut a, WindMethod::Stagger, false).unwrap();
    let mut b = base.clone();
    wind(&mut b, WindMethod::Stagger, false).unwrap();
    assert_eq!(a.data, b.data, "streaks come from the coordinate hash");
    assert_eq!(&a.data[3 * n..], &alpha[..]);
}

#[test]
fn wind_rejects_an_empty_buffer() {
    let mut empty = PixelBuffer::new(0, 0, 3);
    assert!(wind(&mut empty, WindMethod::Wind, false).is_err());
}
