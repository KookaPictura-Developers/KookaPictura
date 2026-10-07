//! Spec scenarios from the replaced `sketch/paper.rs` unit tests, run through
//! `apply` against the ported photorust engine.

use super::shims::*;
use pictura_core::PixelBuffer;
use pictura_filters::*;

fn gradient(w: u32, h: u32, channels: u8) -> PixelBuffer {
    let n = (w * h) as usize;
    let mut data = vec![0u8; n * channels as usize];
    let denom = (w.max(2) - 1) as f64;
    for y in 0..h as usize {
        for x in 0..w as usize {
            let i = y * w as usize + x;
            let step = if x < (w / 2) as usize { 0.0 } else { 100.0 };
            let v = (255.0 * x as f64 / denom + step).min(255.0).round() as u8;
            for c in 0..(channels as usize).min(3) {
                data[c * n + i] = v;
            }
            if channels == 4 {
                data[3 * n + i] = 137;
            }
        }
    }
    PixelBuffer {
        width: w,
        height: h,
        channels,
        data: data.into(),
    }
}

fn flat(w: u32, h: u32, value: u8) -> PixelBuffer {
    let n = (w * h) as usize;
    PixelBuffer {
        width: w,
        height: h,
        channels: 3,
        data: vec![value; n * 3].into(),
    }
}

fn block(w: u32, h: u32, block_w: u32, value: u8) -> PixelBuffer {
    let n = (w * h) as usize;
    let mut buf = flat(w, h, 255);
    for y in 0..h as usize {
        for x in 0..block_w as usize {
            for c in 0..3 {
                buf.data[c * n + y * w as usize + x] = value;
            }
        }
    }
    buf
}

fn planes_of(buf: &PixelBuffer, n: usize) -> usize {
    (buf.channels as usize).min(3) * n
}

fn alpha_plane(buf: &PixelBuffer) -> Vec<u8> {
    let n = buf.pixel_count();
    buf.data[3 * n..4 * n].to_vec()
}

const FG: [u8; 3] = [10, 10, 10];
const BG: [u8; 3] = [240, 240, 240];

fn paper_cases(seed: u64) -> Vec<Filter> {
    vec![
        Filter::NotePaper {
            image_balance: 25,
            graininess: 10,
            relief: 11,
            seed,
        },
        Filter::Photocopy {
            detail: 5,
            darkness: 20,
        },
        Filter::Plaster {
            image_balance: 25,
            smoothness: 2,
            light_direction: LightDirection::Bottom,
            foreground: FG,
            background: BG,
        },
        Filter::Reticulation {
            density: 13,
            black_level: 10,
            white_level: 40,
            foreground: FG,
            background: BG,
            seed,
        },
        Filter::Stamp {
            light_dark_balance: 25,
            smoothness: 5,
            foreground: FG,
            background: BG,
        },
        Filter::TornEdges {
            image_balance: 25,
            smoothness: 1,
            contrast: 8,
            foreground: FG,
            background: BG,
        },
        Filter::WaterPaper {
            fiber_length: 15,
            brightness: 45,
            contrast: 60,
            seed,
        },
    ]
}

#[test]
fn each_filter_changes_the_colour_planes() {
    let base = gradient(32, 8, 3);
    let n = base.pixel_count();
    for filter in paper_cases(3) {
        let mut out = base.clone();
        apply(&filter, &mut out).unwrap();
        assert_ne!(
            out.data[..planes_of(&out, n)],
            base.data[..planes_of(&base, n)],
            "{filter:?} did not change colour"
        );
    }
}

#[test]
fn every_filter_preserves_alpha_via_apply() {
    let base = gradient(16, 6, 4);
    let expected = alpha_plane(&base);
    for filter in paper_cases(9) {
        let mut out = base.clone();
        apply(&filter, &mut out).unwrap();
        assert_eq!(alpha_plane(&out), expected, "{filter:?} touched alpha");
    }
}

#[test]
fn boundaries_accept_and_out_of_range_rejects() {
    let base = gradient(8, 4, 3);

    assert!(note_paper(&mut base.clone(), 0, 0, 0, 1).is_ok());
    assert!(note_paper(&mut base.clone(), 50, 20, 25, 1).is_ok());
    for (b, g, r) in [(51u8, 0u8, 0u8), (0, 21, 0), (0, 0, 26)] {
        let mut out = base.clone();
        assert!(matches!(
            note_paper(&mut out, b, g, r, 1),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected note paper modified the buffer");
    }

    assert!(photocopy(&mut base.clone(), 0, 1).is_ok());
    assert!(photocopy(&mut base.clone(), 24, 50).is_ok());
    for (d, k) in [(25u8, 20u8), (5, 0), (5, 51)] {
        let mut out = base.clone();
        assert!(matches!(
            photocopy(&mut out, d, k),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected photocopy modified the buffer");
    }

    assert!(plaster(&mut base.clone(), 0, 0, LightDirection::Bottom, FG, BG).is_ok());
    assert!(plaster(
        &mut base.clone(),
        50,
        15,
        LightDirection::BottomRight,
        FG,
        BG
    )
    .is_ok());
    for (b, s) in [(51u8, 0u8), (25, 16)] {
        let mut out = base.clone();
        assert!(matches!(
            plaster(&mut out, b, s, LightDirection::Top, FG, BG),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected plaster modified the buffer");
    }

    assert!(reticulation(&mut base.clone(), 0, 0, 0, FG, BG, 1).is_ok());
    assert!(reticulation(&mut base.clone(), 50, 50, 50, FG, BG, 1).is_ok());
    for (d, bl, wl) in [(51u8, 0u8, 0u8), (0, 51, 0), (0, 0, 51)] {
        let mut out = base.clone();
        assert!(matches!(
            reticulation(&mut out, d, bl, wl, FG, BG, 1),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected reticulation modified the buffer");
    }

    assert!(stamp(&mut base.clone(), 0, 1, FG, BG).is_ok());
    assert!(stamp(&mut base.clone(), 50, 50, FG, BG).is_ok());
    for (b, s) in [(51u8, 5u8), (25, 0), (25, 51)] {
        let mut out = base.clone();
        assert!(matches!(
            stamp(&mut out, b, s, FG, BG),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected stamp modified the buffer");
    }

    assert!(torn_edges(&mut base.clone(), 0, 1, 1, FG, BG).is_ok());
    assert!(torn_edges(&mut base.clone(), 50, 15, 25, FG, BG).is_ok());
    for (b, s, c) in [
        (51u8, 1u8, 1u8),
        (25, 0, 1),
        (25, 16, 1),
        (25, 1, 0),
        (25, 1, 26),
    ] {
        let mut out = base.clone();
        assert!(matches!(
            torn_edges(&mut out, b, s, c, FG, BG),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected torn edges modified the buffer");
    }

    assert!(water_paper(&mut base.clone(), 3, 0, 0, 1).is_ok());
    assert!(water_paper(&mut base.clone(), 50, 100, 100, 1).is_ok());
    for (f, b, c) in [(2u8, 0u8, 0u8), (51, 0, 0), (15, 101, 0), (15, 0, 101)] {
        let mut out = base.clone();
        assert!(matches!(
            water_paper(&mut out, f, b, c, 1),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected water paper modified the buffer");
    }
}

#[test]
fn seeded_filters_are_deterministic() {
    let base = gradient(32, 8, 3);

    let (mut a, mut b, mut c) = (base.clone(), base.clone(), base.clone());
    note_paper(&mut a, 25, 10, 11, 7).unwrap();
    note_paper(&mut b, 25, 10, 11, 7).unwrap();
    note_paper(&mut c, 25, 10, 11, 8).unwrap();
    assert_eq!(a.data, b.data, "note paper same seed must match");
    assert_ne!(a.data, c.data, "note paper different seed must differ");

    let (mut d, mut e, mut f) = (base.clone(), base.clone(), base.clone());
    reticulation(&mut d, 13, 10, 40, FG, BG, 7).unwrap();
    reticulation(&mut e, 13, 10, 40, FG, BG, 7).unwrap();
    reticulation(&mut f, 13, 10, 40, FG, BG, 8).unwrap();
    assert_eq!(d.data, e.data, "reticulation same seed must match");
    assert_ne!(d.data, f.data, "reticulation different seed must differ");

    let (mut g, mut hh, mut j) = (base.clone(), base.clone(), base.clone());
    water_paper(&mut g, 15, 45, 60, 7).unwrap();
    water_paper(&mut hh, 15, 45, 60, 7).unwrap();
    water_paper(&mut j, 15, 45, 60, 8).unwrap();
    assert_eq!(g.data, hh.data, "water paper same seed must match");
    assert_ne!(g.data, j.data, "water paper different seed must differ");
}

#[test]
fn photocopy_darkness_shrinks_the_dark_area() {
    let base = block(32, 8, 16, 60);
    let n = base.pixel_count();
    let dark_pixels =
        |buf: &PixelBuffer| -> usize { buf.data[..n].iter().filter(|&&v| v == 0).count() };

    let mut dark = base.clone();
    photocopy(&mut dark, 12, 10).unwrap();
    let mut light = base.clone();
    photocopy(&mut light, 12, 50).unwrap();

    assert!(
        dark_pixels(&light) >= dark_pixels(&dark),
        "raising darkness must not shrink the toner ({} vs {})",
        dark_pixels(&dark),
        dark_pixels(&light)
    );
}

#[test]
fn reticulation_levels_change_the_result() {
    let base = gradient(32, 8, 3);
    let (mut a, mut b, mut c) = (base.clone(), base.clone(), base.clone());
    reticulation(&mut a, 13, 10, 40, FG, BG, 4).unwrap();
    reticulation(&mut b, 13, 50, 40, FG, BG, 4).unwrap();
    reticulation(&mut c, 13, 10, 50, FG, BG, 4).unwrap();
    assert_ne!(a.data, b.data, "black level must change the result");
    assert_ne!(a.data, c.data, "white level must change the result");
}

#[test]
fn torn_edges_contrast_and_balance_change_the_result() {
    let base = gradient(32, 8, 3);
    let (mut a, mut b, mut c) = (base.clone(), base.clone(), base.clone());
    torn_edges(&mut a, 25, 1, 8, FG, BG).unwrap();
    torn_edges(&mut b, 25, 1, 25, FG, BG).unwrap();
    torn_edges(&mut c, 50, 1, 8, FG, BG).unwrap();
    assert_ne!(a.data, b.data, "contrast must change the result");
    assert_ne!(a.data, c.data, "image balance must change the result");
}

#[test]
fn equal_foreground_and_background_are_safe() {
    let base = gradient(16, 6, 3);
    let same = [7, 7, 7];
    assert!(plaster(&mut base.clone(), 25, 5, LightDirection::Top, same, same).is_ok());
    assert!(reticulation(&mut base.clone(), 13, 10, 40, same, same, 1).is_ok());
    assert!(stamp(&mut base.clone(), 25, 5, same, same).is_ok());
    assert!(torn_edges(&mut base.clone(), 25, 1, 8, same, same).is_ok());
}

#[test]
fn tiny_buffers_do_not_panic() {
    let tiny = PixelBuffer {
        width: 1,
        height: 1,
        channels: 4,
        data: vec![10, 20, 30, 40].into(),
    };
    assert!(note_paper(&mut tiny.clone(), 50, 20, 25, 1).is_ok());
    assert!(photocopy(&mut tiny.clone(), 24, 50).is_ok());
    assert!(plaster(&mut tiny.clone(), 50, 15, LightDirection::Top, FG, BG).is_ok());
    assert!(reticulation(&mut tiny.clone(), 50, 50, 50, FG, BG, 1).is_ok());
    assert!(stamp(&mut tiny.clone(), 50, 50, FG, BG).is_ok());
    assert!(torn_edges(&mut tiny.clone(), 50, 15, 25, FG, BG).is_ok());
    assert!(water_paper(&mut tiny.clone(), 50, 100, 100, 1).is_ok());
}
