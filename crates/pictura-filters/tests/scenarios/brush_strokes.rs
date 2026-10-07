//! Spec scenarios from the replaced `brush_strokes.rs` unit tests, run through
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

fn alpha_plane(buf: &PixelBuffer) -> Vec<u8> {
    let n = buf.pixel_count();
    buf.data[3 * n..4 * n].to_vec()
}

fn distinct(plane: &[u8]) -> usize {
    plane
        .iter()
        .collect::<std::collections::BTreeSet<_>>()
        .len()
}

fn brush_filters(seed: u64) -> Vec<Filter> {
    vec![
        Filter::AccentedEdges {
            edge_width: 2,
            edge_brightness: 0,
            smoothness: 5,
        },
        Filter::AngledStrokes {
            direction_balance: 50,
            stroke_length: 15,
            sharpness: 5,
        },
        Filter::Crosshatch {
            stroke_length: 9,
            sharpness: 6,
            strength: 2,
        },
        Filter::DarkStrokes {
            balance: 5,
            black_intensity: 6,
            white_intensity: 5,
        },
        Filter::InkOutlines {
            stroke_length: 10,
            dark_intensity: 25,
            light_intensity: 25,
        },
        Filter::Spatter {
            spray_radius: 8,
            smoothness: 5,
            seed,
        },
        Filter::SprayedStrokes {
            stroke_length: 10,
            spray_radius: 7,
            direction: StrokeDirection::RightDiagonal,
            seed,
        },
        Filter::SumiE {
            stroke_width: 8,
            stroke_pressure: 5,
            contrast: 20,
        },
    ]
}

#[test]
fn each_filter_changes_the_colour_planes() {
    let base = gradient(32, 8, 3);
    let n = base.pixel_count();
    for filter in brush_filters(3) {
        let mut out = base.clone();
        apply(&filter, &mut out).unwrap();
        assert_ne!(
            out.data[..3 * n],
            base.data[..3 * n],
            "{filter:?} did not change colour"
        );
    }
}

#[test]
fn every_filter_preserves_alpha_via_apply() {
    let base = gradient(16, 6, 4);
    let n = base.pixel_count();
    let expected = alpha_plane(&base);
    for filter in brush_filters(9) {
        let mut out = base.clone();
        apply(&filter, &mut out).unwrap();
        assert_eq!(alpha_plane(&out), expected, "{filter:?} touched alpha");
        assert_ne!(
            out.data[..3 * n],
            base.data[..3 * n],
            "{filter:?} did not change colour"
        );
    }
}

#[test]
fn boundaries_are_accepted_and_out_of_range_rejected() {
    let base = gradient(8, 4, 3);

    assert!(accented_edges(&mut base.clone(), 1, 0, 1).is_ok());
    assert!(accented_edges(&mut base.clone(), 14, 50, 15).is_ok());
    for (ew, eb, sm) in [(0u8, 25u8, 5u8), (15, 25, 5), (1, 51, 5), (1, 25, 0)] {
        let mut out = base.clone();
        assert!(matches!(
            accented_edges(&mut out, ew, eb, sm),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected accented edges modified the buffer");
    }

    assert!(angled_strokes(&mut base.clone(), 0, 3, 0).is_ok());
    assert!(angled_strokes(&mut base.clone(), 100, 50, 10).is_ok());
    for (db, sl, sh) in [(101u8, 15u8, 5u8), (50, 2, 5), (50, 51, 5), (50, 15, 11)] {
        let mut out = base.clone();
        assert!(matches!(
            angled_strokes(&mut out, db, sl, sh),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected angled strokes modified the buffer");
    }

    assert!(crosshatch(&mut base.clone(), 3, 0, 1).is_ok());
    assert!(crosshatch(&mut base.clone(), 50, 20, 3).is_ok());
    for sl in [2u8, 51] {
        assert!(matches!(
            crosshatch(&mut base.clone(), sl, 6, 2),
            Err(FilterError::InvalidParams(_))
        ));
    }
    assert!(matches!(
        crosshatch(&mut base.clone(), 9, 21, 2),
        Err(FilterError::InvalidParams(_))
    ));
    for st in [0u8, 4] {
        assert!(matches!(
            crosshatch(&mut base.clone(), 9, 6, st),
            Err(FilterError::InvalidParams(_))
        ));
    }

    assert!(dark_strokes(&mut base.clone(), 0, 0, 0).is_ok());
    assert!(dark_strokes(&mut base.clone(), 10, 10, 10).is_ok());
    for (b, bl, wh) in [(11u8, 5u8, 5u8), (5, 11, 5), (5, 5, 11)] {
        let mut out = base.clone();
        assert!(matches!(
            dark_strokes(&mut out, b, bl, wh),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected dark strokes modified the buffer");
    }

    assert!(ink_outlines(&mut base.clone(), 1, 0, 0).is_ok());
    assert!(ink_outlines(&mut base.clone(), 50, 50, 50).is_ok());
    for (sl, di, li) in [(0u8, 25u8, 25u8), (51, 25, 25), (10, 51, 25), (10, 25, 51)] {
        let mut out = base.clone();
        assert!(matches!(
            ink_outlines(&mut out, sl, di, li),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected ink outlines modified the buffer");
    }

    assert!(spatter(&mut base.clone(), 0, 1, 1).is_ok());
    assert!(spatter(&mut base.clone(), 25, 15, 1).is_ok());
    for (sr, sm) in [(26u8, 5u8), (5, 0), (5, 16)] {
        let mut out = base.clone();
        assert!(matches!(
            spatter(&mut out, sr, sm, 1),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected spatter modified the buffer");
    }

    assert!(sprayed_strokes(&mut base.clone(), 0, 0, StrokeDirection::Horizontal, 1).is_ok());
    assert!(sprayed_strokes(&mut base.clone(), 20, 25, StrokeDirection::Vertical, 1).is_ok());
    for (sl, sr) in [(21u8, 7u8), (10, 26)] {
        let mut out = base.clone();
        assert!(matches!(
            sprayed_strokes(&mut out, sl, sr, StrokeDirection::Horizontal, 1),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected sprayed strokes modified the buffer");
    }

    assert!(sumi_e(&mut base.clone(), 3, 0, 0).is_ok());
    assert!(sumi_e(&mut base.clone(), 15, 15, 40).is_ok());
    for (sw, sp, ct) in [(2u8, 5u8, 20u8), (16, 5, 20), (8, 16, 20), (8, 5, 41)] {
        let mut out = base.clone();
        assert!(matches!(
            sumi_e(&mut out, sw, sp, ct),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected sumi-e modified the buffer");
    }
}

#[test]
fn stochastic_filters_are_seed_deterministic() {
    let base = gradient(32, 8, 3);

    let (mut sa, mut sb, mut sc) = (base.clone(), base.clone(), base.clone());
    spatter(&mut sa, 12, 5, 7).unwrap();
    spatter(&mut sb, 12, 5, 7).unwrap();
    spatter(&mut sc, 12, 5, 8).unwrap();
    assert_eq!(sa.data, sb.data, "spatter same seed must be bit-identical");
    assert_ne!(sa.data, sc.data, "spatter different seed must differ");

    let (mut xa, mut xb, mut xc) = (base.clone(), base.clone(), base.clone());
    sprayed_strokes(&mut xa, 10, 7, StrokeDirection::RightDiagonal, 7).unwrap();
    sprayed_strokes(&mut xb, 10, 7, StrokeDirection::RightDiagonal, 7).unwrap();
    sprayed_strokes(&mut xc, 10, 7, StrokeDirection::RightDiagonal, 8).unwrap();
    assert_eq!(
        xa.data, xb.data,
        "sprayed strokes same seed must be bit-identical"
    );
    assert_ne!(
        xa.data, xc.data,
        "sprayed strokes different seed must differ"
    );
}

#[test]
fn accented_edges_brightness_polarity_and_neutrality() {
    let base = gradient(32, 8, 3);
    let n = base.pixel_count();
    let sum = |b: &PixelBuffer| b.data[..3 * n].iter().map(|&v| v as u64).sum::<u64>();

    let mut dark = base.clone();
    accented_edges(&mut dark, 2, 0, 5).unwrap();
    let mut light = base.clone();
    accented_edges(&mut light, 2, 50, 5).unwrap();
    assert!(
        sum(&dark) < sum(&base),
        "brightness 0 ({}) must darken below input ({})",
        sum(&dark),
        sum(&base)
    );
    assert!(
        sum(&light) > sum(&base),
        "brightness 50 ({}) must lighten above input ({})",
        sum(&light),
        sum(&base)
    );
}

#[test]
fn crosshatch_strength_and_spatter_radius_change_coverage() {
    let base = gradient(32, 8, 3);
    let n = base.pixel_count();

    let mut one = base.clone();
    crosshatch(&mut one, 9, 6, 1).unwrap();
    let mut three = base.clone();
    crosshatch(&mut three, 9, 6, 3).unwrap();
    assert_ne!(one.data, three.data, "strength 1 vs 3 must differ");
    assert!(
        distinct(&one.data[..n]) > 1,
        "crosshatch must preserve tonal detail"
    );

    let changed = |b: &PixelBuffer| {
        b.data
            .iter()
            .zip(&base.data)
            .filter(|(a, c)| a != c)
            .count()
    };
    let mut r0 = base.clone();
    spatter(&mut r0, 0, 5, 3).unwrap();
    let mut r25 = base.clone();
    spatter(&mut r25, 25, 5, 3).unwrap();
    assert_eq!(
        changed(&r0),
        0,
        "spray radius 0 leaves the picture as it was"
    );
    assert!(
        changed(&r25) > changed(&r0),
        "radius 25 ({}) must spread wider than radius 0 ({})",
        changed(&r25),
        changed(&r0)
    );
}

#[test]
fn sprayed_strokes_direction_changes_the_result() {
    let base = gradient(32, 8, 3);
    let mut right = base.clone();
    sprayed_strokes(&mut right, 10, 7, StrokeDirection::RightDiagonal, 4).unwrap();
    let mut horiz = base.clone();
    sprayed_strokes(&mut horiz, 10, 7, StrokeDirection::Horizontal, 4).unwrap();
    assert_ne!(right.data, horiz.data, "direction must change the result");
}
