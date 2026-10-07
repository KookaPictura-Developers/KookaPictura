//! Spec scenarios from the replaced `sketch/relief.rs` unit tests, run through
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

fn planes_of(buf: &PixelBuffer, n: usize) -> usize {
    (buf.channels as usize).min(3) * n
}

fn alpha_plane(buf: &PixelBuffer) -> Vec<u8> {
    let n = buf.pixel_count();
    buf.data[3 * n..4 * n].to_vec()
}

const FG: [u8; 3] = [10, 10, 10];
const BG: [u8; 3] = [240, 240, 240];

fn sketch_cases(seed: u64) -> Vec<Filter> {
    vec![
        Filter::BasRelief {
            detail: 6,
            smoothness: 3,
            light_direction: LightDirection::Top,
            foreground: FG,
            background: BG,
        },
        Filter::ChalkCharcoal {
            charcoal_area: 6,
            chalk_area: 6,
            stroke_pressure: 1,
            foreground: FG,
            background: BG,
            seed,
        },
        Filter::Charcoal {
            thickness: 3,
            detail: 3,
            light_dark_balance: 50,
            foreground: FG,
            background: BG,
            seed,
        },
        Filter::Chrome {
            detail: 4,
            smoothness: 7,
        },
        Filter::ConteCrayon {
            foreground_level: 8,
            background_level: 7,
            texture: TextureOptions::default(),
            foreground: FG,
            background: BG,
            seed,
        },
        Filter::GraphicPen {
            stroke_length: 6,
            light_dark_balance: 50,
            direction: StrokeDirection::RightDiagonal,
            foreground: FG,
            background: BG,
        },
        Filter::HalftonePattern {
            size: 5,
            contrast: 5,
            pattern: HalftoneType::Dot,
        },
    ]
}

#[test]
fn each_filter_changes_the_colour_planes() {
    let base = gradient(32, 8, 3);
    let n = base.pixel_count();
    for filter in sketch_cases(3) {
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
    for filter in sketch_cases(9) {
        let mut out = base.clone();
        apply(&filter, &mut out).unwrap();
        assert_eq!(alpha_plane(&out), expected, "{filter:?} touched alpha");
    }
}

#[test]
fn boundaries_accept_and_out_of_range_rejects() {
    let base = gradient(8, 4, 3);
    let opts = TextureOptions::default();

    assert!(bas_relief(&mut base.clone(), 1, 1, LightDirection::Bottom, FG, BG).is_ok());
    assert!(bas_relief(
        &mut base.clone(),
        15,
        15,
        LightDirection::BottomRight,
        FG,
        BG
    )
    .is_ok());
    for (d, s) in [(0u8, 1u8), (16, 1), (1, 0), (1, 16)] {
        let mut out = base.clone();
        assert!(matches!(
            bas_relief(&mut out, d, s, LightDirection::Top, FG, BG),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected bas relief modified the buffer");
    }

    assert!(chalk_charcoal(&mut base.clone(), 0, 0, 0, FG, BG, 1).is_ok());
    assert!(chalk_charcoal(&mut base.clone(), 20, 20, 5, FG, BG, 1).is_ok());
    for (ca, cha, sp) in [(21u8, 0u8, 0u8), (0, 21, 0), (0, 0, 6)] {
        let mut out = base.clone();
        assert!(matches!(
            chalk_charcoal(&mut out, ca, cha, sp, FG, BG, 1),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected chalk & charcoal modified the buffer");
    }

    assert!(charcoal(&mut base.clone(), 1, 0, 0, FG, BG, 1).is_ok());
    assert!(charcoal(&mut base.clone(), 7, 5, 100, FG, BG, 1).is_ok());
    for (t, d, b) in [(0u8, 0u8, 0u8), (8, 0, 0), (1, 6, 0), (1, 0, 101)] {
        let mut out = base.clone();
        assert!(matches!(
            charcoal(&mut out, t, d, b, FG, BG, 1),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected charcoal modified the buffer");
    }

    assert!(chrome(&mut base.clone(), 0, 0).is_ok());
    assert!(chrome(&mut base.clone(), 10, 10).is_ok());
    for (d, s) in [(11u8, 0u8), (0, 11)] {
        let mut out = base.clone();
        assert!(matches!(
            chrome(&mut out, d, s),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected chrome modified the buffer");
    }

    assert!(conte_crayon(&mut base.clone(), 1, 1, opts, FG, BG, 1).is_ok());
    assert!(conte_crayon(&mut base.clone(), 15, 15, opts, FG, BG, 1).is_ok());
    for (f, b) in [(0u8, 1u8), (16, 1), (1, 0), (1, 16)] {
        let mut out = base.clone();
        assert!(matches!(
            conte_crayon(&mut out, f, b, opts, FG, BG, 1),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected conte crayon modified the buffer");
    }
    for bad in [
        TextureOptions {
            scaling: 49,
            ..opts
        },
        TextureOptions {
            scaling: 201,
            ..opts
        },
        TextureOptions { relief: 51, ..opts },
        TextureOptions {
            light_direction: 8,
            ..opts
        },
    ] {
        let mut out = base.clone();
        assert!(matches!(
            conte_crayon(&mut out, 8, 7, bad, FG, BG, 1),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected conte texture modified the buffer");
    }

    assert!(graphic_pen(&mut base.clone(), 1, 0, StrokeDirection::Horizontal, FG, BG).is_ok());
    assert!(graphic_pen(
        &mut base.clone(),
        15,
        100,
        StrokeDirection::Vertical,
        FG,
        BG
    )
    .is_ok());
    for (l, b) in [(0u8, 0u8), (16, 0), (1, 101)] {
        let mut out = base.clone();
        assert!(matches!(
            graphic_pen(&mut out, l, b, StrokeDirection::Horizontal, FG, BG),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected graphic pen modified the buffer");
    }

    assert!(halftone_pattern(&mut base.clone(), 1, 0, HalftoneType::Dot).is_ok());
    assert!(halftone_pattern(&mut base.clone(), 12, 50, HalftoneType::Circle).is_ok());
    for (sz, c) in [(0u8, 0u8), (13, 0), (1, 51)] {
        let mut out = base.clone();
        assert!(matches!(
            halftone_pattern(&mut out, sz, c, HalftoneType::Dot),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected halftone pattern modified the buffer");
    }
}

#[test]
fn seeded_filters_are_deterministic() {
    let base = gradient(32, 8, 3);
    let opts = TextureOptions::default();

    let (mut a, mut b, mut c) = (base.clone(), base.clone(), base.clone());
    chalk_charcoal(&mut a, 6, 6, 1, FG, BG, 7).unwrap();
    chalk_charcoal(&mut b, 6, 6, 1, FG, BG, 7).unwrap();
    chalk_charcoal(&mut c, 6, 6, 1, FG, BG, 8).unwrap();
    assert_eq!(a.data, b.data, "chalk & charcoal same seed must match");
    assert_ne!(
        a.data, c.data,
        "chalk & charcoal different seed must differ"
    );

    let (mut d, mut e, mut f) = (base.clone(), base.clone(), base.clone());
    charcoal(&mut d, 3, 3, 50, FG, BG, 7).unwrap();
    charcoal(&mut e, 3, 3, 50, FG, BG, 7).unwrap();
    charcoal(&mut f, 3, 3, 50, FG, BG, 8).unwrap();
    assert_eq!(d.data, e.data, "charcoal same seed must match");
    assert_ne!(d.data, f.data, "charcoal different seed must differ");

    let (mut g, mut hh, mut j) = (base.clone(), base.clone(), base.clone());
    conte_crayon(&mut g, 8, 7, opts, FG, BG, 7).unwrap();
    conte_crayon(&mut hh, 8, 7, opts, FG, BG, 7).unwrap();
    conte_crayon(&mut j, 8, 7, opts, FG, BG, 8).unwrap();
    assert_eq!(g.data, hh.data, "conte crayon same seed must match");
    assert_ne!(g.data, j.data, "conte crayon different seed must differ");
}

#[test]
fn bas_relief_tone_mapping_light_direction_and_flat_input() {
    let (w, h) = (32u32, 8u32);
    let n = (w * h) as usize;
    let mut data = vec![0u8; n * 3];
    for y in 0..h as usize {
        for x in 0..w as usize {
            let v = if x < (w / 2) as usize { 40 } else { 210 };
            for c in 0..3 {
                data[c * n + y * w as usize + x] = v;
            }
        }
    }
    let step = PixelBuffer {
        width: w,
        height: h,
        channels: 3,
        data: data.into(),
    };
    let half = (w / 2) as usize;
    let mean = |buf: &PixelBuffer, plane: usize, from: usize, to: usize| -> f64 {
        let mut s = 0.0;
        let mut count = 0.0;
        for y in 0..h as usize {
            for x in from..to {
                s += buf.data[plane * n + y * w as usize + x] as f64;
                count += 1.0;
            }
        }
        s / count
    };

    let fg = [200, 10, 10];
    let bg = [10, 10, 200];
    let mut out = step.clone();
    bas_relief(&mut out, 6, 3, LightDirection::Left, fg, bg).unwrap();
    // CS6 / photorust: a flat surface catches no light and sits at the
    // midpoint of the swatches; the carved step pulls toward them.
    let mid_r = (fg[0] as f64 + bg[0] as f64) / 2.0;
    let far = mean(&out, 0, 0, 4);
    assert!(
        (far - mid_r).abs() <= 2.0,
        "flat area at the midpoint ({far} vs {mid_r})"
    );
    let edge = |buf: &PixelBuffer, x: usize| buf.data[3 * w as usize + x] as f64;
    let swing = (edge(&out, half - 1) - mid_r)
        .abs()
        .max((edge(&out, half) - mid_r).abs());
    assert!(
        swing > 20.0,
        "the step must be carved toward a swatch ({swing})"
    );
    let _ = (half, mean(&out, 2, half, w as usize));

    let mut other = step.clone();
    bas_relief(&mut other, 6, 3, LightDirection::Top, fg, bg).unwrap();
    assert_ne!(
        out.data, other.data,
        "light direction must change the result"
    );

    let mut uniform = flat(8, 4, 120);
    assert!(bas_relief(&mut uniform, 6, 3, LightDirection::Top, fg, bg).is_ok());

    let same = [7, 7, 7];
    let mut equal = flat(8, 4, 120);
    assert!(bas_relief(&mut equal, 6, 3, LightDirection::Top, same, same).is_ok());
}

#[test]
fn conte_crayon_texture_options_change_the_result() {
    let base = gradient(32, 8, 3);
    let canvas = TextureOptions::default();
    let brick = TextureOptions {
        surface: TextureSurface::Brick,
        ..canvas
    };
    let lit = TextureOptions {
        light_direction: 4,
        relief: 20,
        ..canvas
    };

    let (mut c, mut b, mut l) = (base.clone(), base.clone(), base.clone());
    conte_crayon(&mut c, 8, 7, canvas, FG, BG, 4).unwrap();
    conte_crayon(&mut b, 8, 7, brick, FG, BG, 4).unwrap();
    conte_crayon(&mut l, 8, 7, lit, FG, BG, 4).unwrap();
    assert_ne!(c.data, b.data, "surface preset must matter");
    assert_ne!(c.data, l.data, "light direction must matter");
}

#[test]
fn halftone_patterns_and_size_differ() {
    let base = gradient(32, 8, 3);
    let (mut dot, mut line, mut circle) = (base.clone(), base.clone(), base.clone());
    halftone_pattern(&mut dot, 4, 5, HalftoneType::Dot).unwrap();
    halftone_pattern(&mut line, 4, 5, HalftoneType::Line).unwrap();
    halftone_pattern(&mut circle, 4, 5, HalftoneType::Circle).unwrap();
    assert_ne!(dot.data, line.data, "dot vs line must differ");
    assert_ne!(dot.data, circle.data, "dot vs circle must differ");
    assert_ne!(line.data, circle.data, "line vs circle must differ");

    let (mut one, mut twelve) = (base.clone(), base.clone());
    halftone_pattern(&mut one, 1, 5, HalftoneType::Dot).unwrap();
    halftone_pattern(&mut twelve, 12, 5, HalftoneType::Dot).unwrap();
    assert_ne!(one.data, twelve.data, "cell size must change the result");
}

#[test]
fn equal_foreground_and_background_are_safe() {
    let base = gradient(16, 6, 3);
    let same = [7, 7, 7];
    let opts = TextureOptions::default();
    assert!(chalk_charcoal(&mut base.clone(), 6, 6, 1, same, same, 1).is_ok());
    assert!(charcoal(&mut base.clone(), 3, 3, 50, same, same, 1).is_ok());
    assert!(conte_crayon(&mut base.clone(), 8, 7, opts, same, same, 1).is_ok());
    assert!(graphic_pen(
        &mut base.clone(),
        6,
        50,
        StrokeDirection::RightDiagonal,
        same,
        same
    )
    .is_ok());
}

#[test]
fn tiny_buffers_do_not_panic() {
    let tiny = PixelBuffer {
        width: 1,
        height: 1,
        channels: 4,
        data: vec![10, 20, 30, 40].into(),
    };
    let opts = TextureOptions::default();
    assert!(bas_relief(&mut tiny.clone(), 15, 15, LightDirection::Top, FG, BG).is_ok());
    assert!(chalk_charcoal(&mut tiny.clone(), 20, 20, 5, FG, BG, 1).is_ok());
    assert!(charcoal(&mut tiny.clone(), 7, 5, 100, FG, BG, 1).is_ok());
    assert!(chrome(&mut tiny.clone(), 10, 10).is_ok());
    assert!(conte_crayon(&mut tiny.clone(), 15, 15, opts, FG, BG, 1).is_ok());
    assert!(graphic_pen(
        &mut tiny.clone(),
        15,
        100,
        StrokeDirection::Vertical,
        FG,
        BG
    )
    .is_ok());
    assert!(halftone_pattern(&mut tiny.clone(), 12, 50, HalftoneType::Circle).is_ok());
}
