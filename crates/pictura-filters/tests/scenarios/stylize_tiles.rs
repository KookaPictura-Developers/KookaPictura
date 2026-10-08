//! Spec scenarios from the replaced `stylize/tiles.rs` unit tests, run through
//! `apply` against the ported photorust engine.

use super::shims::*;
use pictura_core::PixelBuffer;
use pictura_filters::*;

fn uniform(w: u32, h: u32, channels: u8, rgb: [u8; 3], alpha: impl Fn(usize) -> u8) -> PixelBuffer {
    let n = (w * h) as usize;
    let mut data = vec![0u8; n * channels as usize];
    for i in 0..n {
        data[i] = rgb[0];
        data[n + i] = rgb[1];
        data[2 * n + i] = rgb[2];
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

fn has_pixel(buf: &PixelBuffer, rgb: [u8; 3]) -> bool {
    let n = buf.pixel_count();
    (0..n).any(|i| {
        buf.data[i] == rgb[0] && buf.data[n + i] == rgb[1] && buf.data[2 * n + i] == rgb[2]
    })
}

#[test]
fn tiles_fill_gaps_with_the_background_colour() {
    let base = uniform(8, 8, 3, [10, 20, 30], |_| 0);
    let mut out = base.clone();
    tiles(
        &mut out,
        2,
        50,
        TileFill::BackgroundColor,
        [1, 2, 3],
        [255, 0, 0],
    )
    .unwrap();
    assert!(
        has_pixel(&out, [255, 0, 0]),
        "background fill must show in the gaps"
    );
    assert!(
        has_pixel(&out, [10, 20, 30]),
        "the tiles themselves must survive"
    );
}

#[test]
fn tiles_fill_gaps_with_the_foreground_colour() {
    let base = uniform(8, 8, 3, [10, 20, 30], |_| 0);
    let mut out = base.clone();
    tiles(
        &mut out,
        2,
        50,
        TileFill::ForegroundColor,
        [255, 0, 0],
        [0, 0, 0],
    )
    .unwrap();
    assert!(
        has_pixel(&out, [255, 0, 0]),
        "foreground fill must show in the gaps"
    );
    assert!(
        has_pixel(&out, [10, 20, 30]),
        "the tiles themselves must survive"
    );
}

#[test]
fn tiles_high_count_on_small_image_does_not_panic() {
    let n = 16usize;
    let mut data = vec![0u8; n * 3];
    for i in 0..n {
        data[i] = (i * 15) as u8;
        data[n + i] = (i * 7) as u8;
        data[2 * n + i] = (i * 3) as u8;
    }
    let base = PixelBuffer {
        width: 4,
        height: 4,
        channels: 3,
        data: data.into(),
    };
    let mut out = base.clone();
    tiles(
        &mut out,
        99,
        50,
        TileFill::BackgroundColor,
        [0, 0, 0],
        [255, 255, 255],
    )
    .unwrap();
    let mut again = base.clone();
    tiles(
        &mut again,
        99,
        50,
        TileFill::BackgroundColor,
        [0, 0, 0],
        [255, 255, 255],
    )
    .unwrap();
    assert_eq!(
        out.data, again.data,
        "a count above the tile size still produces deterministic output"
    );
}

#[test]
fn tiles_background_invariant_on_uniform_source() {
    let src = [10, 20, 30];
    let bg = [200, 210, 220];
    let base = uniform(8, 8, 3, src, |_| 0);
    let mut out = base.clone();
    tiles(&mut out, 3, 40, TileFill::BackgroundColor, [0, 0, 0], bg).unwrap();
    let n = out.pixel_count();
    for i in 0..n {
        let p = [out.data[i], out.data[n + i], out.data[2 * n + i]];
        assert!(
            p == src || p == bg,
            "pixel {i} is neither source nor background: {p:?}"
        );
    }
}

#[test]
fn tiles_inverse_fill_negates_the_ground() {
    let base = uniform(8, 8, 3, [10, 20, 30], |_| 0);
    let mut out = base.clone();
    tiles(&mut out, 3, 1, TileFill::InverseImage, [0, 0, 0], [0, 0, 0]).unwrap();
    assert!(
        has_pixel(&out, [245, 235, 225]),
        "ground is the inverted picture"
    );
}

#[test]
fn tiles_unaltered_fill_leaves_a_flat_picture_alone() {
    let base = uniform(8, 8, 3, [77, 88, 99], |_| 0);
    let mut out = base.clone();
    tiles(
        &mut out,
        3,
        20,
        TileFill::UnalteredImage,
        [0, 0, 0],
        [0, 0, 0],
    )
    .unwrap();
    assert_eq!(out, base, "every tile is a copy of the same flat colour");
}

#[test]
fn tiles_are_deterministic_and_preserve_alpha() {
    let base = uniform(11, 7, 4, [40, 90, 160], |i| (i * 13) as u8);
    let n = base.pixel_count();
    let alpha = base.data[3 * n..].to_vec();
    let mut a = base.clone();
    tiles(
        &mut a,
        4,
        40,
        TileFill::BackgroundColor,
        [0, 0, 0],
        [200, 200, 200],
    )
    .unwrap();
    let mut b = base.clone();
    tiles(
        &mut b,
        4,
        40,
        TileFill::BackgroundColor,
        [0, 0, 0],
        [200, 200, 200],
    )
    .unwrap();
    assert_eq!(a.data, b.data, "offsets come from the grid, not a seed");
    assert_eq!(&a.data[3 * n..], &alpha[..]);
}

#[test]
fn tiles_reject_bad_count_and_offset() {
    let base = uniform(6, 6, 3, [50, 60, 70], |_| 0);
    let mut out = base.clone();
    assert!(tiles(
        &mut out,
        0,
        10,
        TileFill::BackgroundColor,
        [0, 0, 0],
        [0, 0, 0]
    )
    .is_err());
    assert!(tiles(
        &mut out,
        100,
        10,
        TileFill::BackgroundColor,
        [0, 0, 0],
        [0, 0, 0]
    )
    .is_err());
    assert!(tiles(
        &mut out,
        3,
        0,
        TileFill::BackgroundColor,
        [0, 0, 0],
        [0, 0, 0]
    )
    .is_err());
    assert!(tiles(
        &mut out,
        3,
        100,
        TileFill::BackgroundColor,
        [0, 0, 0],
        [0, 0, 0]
    )
    .is_err());
    assert_eq!(out, base, "rejected parameters must not modify the buffer");
}
