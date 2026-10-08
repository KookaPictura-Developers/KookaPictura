//! Spec scenarios from the replaced `artistic/filters/tests.rs` unit tests, run through
//! `apply` against the ported photorust engine.

use super::shims::*;
use pictura_filters::*;

use pictura_core::PixelBuffer;

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

fn ramp(w: u32) -> PixelBuffer {
    let n = w as usize;
    let mut data = vec![0u8; n * 3];
    let denom = (w.max(2) - 1) as f64;
    for x in 0..n {
        let v = (255.0 * x as f64 / denom).round() as u8;
        data[x] = v;
        data[n + x] = v;
        data[2 * n + x] = v;
    }
    PixelBuffer {
        width: w,
        height: 1,
        channels: 3,
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

#[test]
fn each_filter_changes_the_colour_planes() {
    let base = gradient(32, 8, 3);
    let n = base.pixel_count();
    let cases: Vec<Filter> = vec![
        Filter::Cutout {
            levels: 4,
            edge_simplicity: 0,
            edge_fidelity: 1,
        },
        Filter::FilmGrain {
            grain: 10,
            highlight_area: 5,
            intensity: 5,
            seed: 42,
        },
        Filter::NeonGlow {
            glow_size: 8,
            glow_brightness: 40,
            glow_color: [0, 0, 255],
        },
        Filter::PosterEdges {
            edge_thickness: 2,
            edge_intensity: 8,
            posterization: 4,
        },
        Filter::PaintDaubs {
            brush_size: 8,
            sharpness: 20,
            brush_type: BrushType::Simple,
            seed: 3,
        },
        Filter::PaletteKnife {
            stroke_size: 12,
            stroke_detail: 2,
            softness: 8,
        },
        Filter::PlasticWrap {
            highlight_strength: 12,
            detail: 6,
            smoothness: 3,
        },
        Filter::Sponge {
            brush_size: 6,
            definition: 18,
            smoothness: 4,
            seed: 3,
        },
        Filter::ColoredPencil {
            pencil_width: 6,
            stroke_pressure: 8,
            paper_brightness: 20,
            background: [240, 240, 240],
            seed: 3,
        },
        Filter::DryBrush {
            brush_size: 8,
            brush_detail: 6,
            texture: 2,
            seed: 3,
        },
        Filter::Fresco {
            brush_size: 8,
            brush_detail: 6,
            texture: 2,
            seed: 3,
        },
    ];
    for filter in cases {
        let mut out = base.clone();
        apply(&filter, &mut out).unwrap();
        assert_ne!(
            out.data[..planes_of(&out, n)],
            base.data[..planes_of(&base, n)],
            "{filter:?} did not change colour"
        );
    }
}

fn planes_of(buf: &PixelBuffer, n: usize) -> usize {
    (buf.channels as usize).min(3) * n
}

#[test]
fn every_artistic_filter_preserves_alpha_via_apply() {
    let base = gradient(16, 6, 4);
    let expected = alpha_plane(&base);
    let cases: Vec<Filter> = vec![
        Filter::Cutout {
            levels: 8,
            edge_simplicity: 3,
            edge_fidelity: 3,
        },
        Filter::FilmGrain {
            grain: 20,
            highlight_area: 20,
            intensity: 10,
            seed: 9,
        },
        Filter::NeonGlow {
            glow_size: -12,
            glow_brightness: 50,
            glow_color: [255, 0, 0],
        },
        Filter::PosterEdges {
            edge_thickness: 10,
            edge_intensity: 10,
            posterization: 10,
        },
        Filter::PaintDaubs {
            brush_size: 10,
            sharpness: 40,
            brush_type: BrushType::WideBlurry,
            seed: 9,
        },
        Filter::PaletteKnife {
            stroke_size: 20,
            stroke_detail: 3,
            softness: 10,
        },
        Filter::PlasticWrap {
            highlight_strength: 20,
            detail: 15,
            smoothness: 15,
        },
        Filter::Sponge {
            brush_size: 10,
            definition: 25,
            smoothness: 15,
            seed: 9,
        },
        Filter::ColoredPencil {
            pencil_width: 24,
            stroke_pressure: 15,
            paper_brightness: 50,
            background: [255, 255, 255],
            seed: 9,
        },
        Filter::DryBrush {
            brush_size: 10,
            brush_detail: 10,
            texture: 3,
            seed: 9,
        },
        Filter::Fresco {
            brush_size: 10,
            brush_detail: 10,
            texture: 3,
            seed: 9,
        },
    ];
    for filter in cases {
        let mut out = base.clone();
        apply(&filter, &mut out).unwrap();
        assert_eq!(alpha_plane(&out), expected, "{filter:?} touched alpha");
    }
}

#[test]
fn boundary_values_are_accepted_and_out_of_range_rejected() {
    let base = gradient(8, 4, 3);

    assert!(cutout(&mut base.clone(), 2, 0, 1).is_ok());
    assert!(cutout(&mut base.clone(), 8, 10, 3).is_ok());
    for (l, s, f) in [(1u8, 0u8, 1u8), (9, 0, 1), (2, 11, 1), (2, 0, 0), (2, 0, 4)] {
        let mut out = base.clone();
        assert!(
            matches!(
                cutout(&mut out, l, s, f),
                Err(FilterError::InvalidParams(_))
            ),
            "cutout {l},{s},{f} should reject"
        );
        assert_eq!(out, base, "rejected cutout modified the buffer");
    }

    assert!(film_grain(&mut base.clone(), 0, 0, 0, 1).is_ok());
    assert!(film_grain(&mut base.clone(), 20, 20, 10, 1).is_ok());
    for (g, ha, i) in [(21u8, 0u8, 0u8), (0, 21, 0), (0, 0, 11)] {
        let mut out = base.clone();
        assert!(matches!(
            film_grain(&mut out, g, ha, i, 1),
            Err(FilterError::InvalidParams(_))
        ));
    }

    assert!(neon_glow(&mut base.clone(), -24, 0, [0, 0, 0]).is_ok());
    assert!(neon_glow(&mut base.clone(), 24, 50, [255, 255, 255]).is_ok());
    for (size, bright) in [(-25i32, 0u8), (25, 0), (0, 51)] {
        let mut out = base.clone();
        assert!(matches!(
            neon_glow(&mut out, size, bright, [0, 0, 0]),
            Err(FilterError::InvalidParams(_))
        ));
    }

    assert!(poster_edges(&mut base.clone(), 0, 0, 0).is_ok());
    assert!(poster_edges(&mut base.clone(), 10, 10, 10).is_ok());
    for (t, i, p) in [(11u8, 0u8, 0u8), (0, 11, 0), (0, 0, 11)] {
        let mut out = base.clone();
        assert!(matches!(
            poster_edges(&mut out, t, i, p),
            Err(FilterError::InvalidParams(_))
        ));
    }
}

#[test]
fn new_artistic_boundaries_accept_and_out_of_range_rejects() {
    let base = gradient(8, 4, 3);

    assert!(paint_daubs(&mut base.clone(), 1, 0, BrushType::Simple, 1).is_ok());
    assert!(paint_daubs(&mut base.clone(), 50, 40, BrushType::Sparkle, 1).is_ok());
    for (b, s) in [(0u8, 0u8), (51, 0), (1, 41)] {
        let mut out = base.clone();
        assert!(matches!(
            paint_daubs(&mut out, b, s, BrushType::Simple, 1),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected paint daubs modified the buffer");
    }

    assert!(palette_knife(&mut base.clone(), 1, 1, 0).is_ok());
    assert!(palette_knife(&mut base.clone(), 50, 3, 10).is_ok());
    for (sz, d, so) in [
        (0u8, 1u8, 0u8),
        (51, 1, 0),
        (1, 0, 0),
        (1, 4, 0),
        (1, 1, 11),
    ] {
        let mut out = base.clone();
        assert!(matches!(
            palette_knife(&mut out, sz, d, so),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected palette knife modified the buffer");
    }

    assert!(plastic_wrap(&mut base.clone(), 0, 1, 1).is_ok());
    assert!(plastic_wrap(&mut base.clone(), 20, 15, 15).is_ok());
    for (hs, d, sm) in [
        (21u8, 1u8, 1u8),
        (0, 0, 1),
        (0, 16, 1),
        (0, 1, 0),
        (0, 1, 16),
    ] {
        let mut out = base.clone();
        assert!(matches!(
            plastic_wrap(&mut out, hs, d, sm),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected plastic wrap modified the buffer");
    }

    assert!(sponge(&mut base.clone(), 0, 0, 1, 1).is_ok());
    assert!(sponge(&mut base.clone(), 10, 25, 15, 1).is_ok());
    for (b, d, sm) in [(11u8, 0u8, 1u8), (0, 26, 1), (0, 0, 0), (0, 0, 16)] {
        let mut out = base.clone();
        assert!(matches!(
            sponge(&mut out, b, d, sm, 1),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected sponge modified the buffer");
    }

    assert!(colored_pencil(&mut base.clone(), 1, 0, 0, [255, 255, 255], 1).is_ok());
    assert!(colored_pencil(&mut base.clone(), 24, 15, 50, [255, 255, 255], 1).is_ok());
    for (pw, sp, pb) in [(0u8, 0u8, 0u8), (25, 0, 0), (1, 16, 0), (1, 0, 51)] {
        let mut out = base.clone();
        assert!(matches!(
            colored_pencil(&mut out, pw, sp, pb, [255, 255, 255], 1),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected colored pencil modified the buffer");
    }

    assert!(dry_brush(&mut base.clone(), 0, 0, 1, 1).is_ok());
    assert!(dry_brush(&mut base.clone(), 10, 10, 3, 1).is_ok());
    for (b, d, t) in [(11u8, 1u8, 1u8), (1, 11, 1), (1, 1, 0), (1, 1, 4)] {
        let mut out = base.clone();
        assert!(matches!(
            dry_brush(&mut out, b, d, t, 1),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected dry brush modified the buffer");
    }

    assert!(fresco(&mut base.clone(), 0, 0, 1, 1).is_ok());
    assert!(fresco(&mut base.clone(), 10, 10, 3, 1).is_ok());
    for (b, d, t) in [(11u8, 1u8, 1u8), (1, 11, 1), (1, 1, 0), (1, 1, 4)] {
        let mut out = base.clone();
        assert!(matches!(
            fresco(&mut out, b, d, t, 1),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected fresco modified the buffer");
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

#[test]
fn colored_pencil_background_shows_and_edges_survive() {
    let base = flat(16, 8, 120);
    let (mut light, mut dark) = (base.clone(), base.clone());
    colored_pencil(&mut light, 6, 6, 10, [255, 255, 255], 5).unwrap();
    colored_pencil(&mut dark, 6, 6, 10, [0, 0, 0], 5).unwrap();
    assert_ne!(
        light.data, dark.data,
        "two background colours must give different flat regions"
    );

    let (mut a, mut b, mut c) = (base.clone(), base.clone(), base.clone());
    colored_pencil(&mut a, 6, 6, 10, [255, 255, 255], 5).unwrap();
    colored_pencil(&mut b, 6, 6, 10, [255, 255, 255], 5).unwrap();
    colored_pencil(&mut c, 6, 6, 10, [255, 255, 255], 6).unwrap();
    assert_eq!(a.data, b.data, "same seed must be bit-identical");
    assert_ne!(a.data, c.data, "different seed must differ");

    let (w, h) = (32u32, 8u32);
    let n = (w * h) as usize;
    let mut data = vec![0u8; n * 3];
    for y in 0..h as usize {
        for x in 0..w as usize {
            let v = if x < (w / 2) as usize { 0 } else { 255 };
            for ch in 0..3 {
                data[ch * n + y * w as usize + x] = v;
            }
        }
    }
    let step = PixelBuffer {
        width: w,
        height: h,
        channels: 3,
        data: data.into(),
    };
    let mut edged = step.clone();
    colored_pencil(&mut edged, 4, 4, 0, [255, 255, 255], 1).unwrap();
    let half = (w / 2) as usize;
    let mut left = 0u64;
    let mut right = 0u64;
    for y in 0..h as usize {
        for x in 0..w as usize {
            let v = edged.data[y * w as usize + x] as u64;
            if x < half {
                left += v;
            } else {
                right += v;
            }
        }
    }
    assert!(
        right > left,
        "edge must retain a luminance change (left {left}, right {right})"
    );
}

#[test]
fn dry_brush_and_fresco_texture_size_and_seed_change_the_result() {
    let base = gradient(32, 8, 3);

    let (mut d1, mut d3) = (base.clone(), base.clone());
    dry_brush(&mut d1, 10, 6, 1, 4).unwrap();
    dry_brush(&mut d3, 10, 6, 3, 4).unwrap();
    assert_ne!(d1.data, d3.data, "dry brush texture 1 vs 3 must differ");
    let (mut ds, mut dl) = (base.clone(), base.clone());
    dry_brush(&mut ds, 1, 6, 2, 4).unwrap();
    dry_brush(&mut dl, 10, 6, 2, 4).unwrap();
    assert_ne!(ds.data, dl.data, "dry brush size 1 vs 10 must differ");

    let (mut f1, mut f3) = (base.clone(), base.clone());
    fresco(&mut f1, 10, 6, 1, 4).unwrap();
    fresco(&mut f3, 10, 6, 3, 4).unwrap();
    assert_ne!(f1.data, f3.data, "fresco texture 1 vs 3 must differ");
    let (mut fs, mut fl) = (base.clone(), base.clone());
    fresco(&mut fs, 1, 6, 2, 4).unwrap();
    fresco(&mut fl, 10, 6, 2, 4).unwrap();
    assert_ne!(fs.data, fl.data, "fresco size 1 vs 10 must differ");

    let (mut a, mut b, mut c) = (base.clone(), base.clone(), base.clone());
    dry_brush(&mut a, 10, 6, 2, 7).unwrap();
    dry_brush(&mut b, 10, 6, 2, 7).unwrap();
    dry_brush(&mut c, 10, 6, 2, 8).unwrap();
    assert_eq!(a.data, b.data, "dry brush same seed must be bit-identical");
    assert_ne!(a.data, c.data, "dry brush different seed must differ");

    let (mut d, mut e, mut f) = (base.clone(), base.clone(), base.clone());
    fresco(&mut d, 10, 6, 2, 7).unwrap();
    fresco(&mut e, 10, 6, 2, 7).unwrap();
    fresco(&mut f, 10, 6, 2, 8).unwrap();
    assert_eq!(d.data, e.data, "fresco same seed must be bit-identical");
    assert_ne!(d.data, f.data, "fresco different seed must differ");
}

#[test]
fn paint_daubs_types_seed_and_size() {
    let base = gradient(32, 8, 3);
    let (mut simple, mut rough) = (base.clone(), base.clone());
    paint_daubs(&mut simple, 10, 20, BrushType::Simple, 7).unwrap();
    paint_daubs(&mut rough, 10, 20, BrushType::DarkRough, 7).unwrap();
    assert_ne!(simple.data, rough.data, "brush types must differ");

    let (mut a, mut b, mut c) = (base.clone(), base.clone(), base.clone());
    // Sparkle is the brush with randomness in it; the seed re-rolls it.
    paint_daubs(&mut a, 10, 20, BrushType::Sparkle, 7).unwrap();
    paint_daubs(&mut b, 10, 20, BrushType::Sparkle, 7).unwrap();
    paint_daubs(&mut c, 10, 20, BrushType::Sparkle, 8).unwrap();
    assert_eq!(a.data, b.data, "same seed must be bit-identical");
    assert_ne!(a.data, c.data, "different seed must differ");

    let ramp = {
        let (rw, rh) = (64u32, 8u32);
        let rn = (rw * rh) as usize;
        let mut data = vec![0u8; rn * 3];
        for y in 0..rh as usize {
            for x in 0..rw as usize {
                let v = (255.0 * x as f64 / (rw - 1) as f64).round() as u8;
                for c in 0..3 {
                    data[c * rn + y * rw as usize + x] = v;
                }
            }
        }
        PixelBuffer {
            width: rw,
            height: rh,
            channels: 3,
            data: data.into(),
        }
    };
    let (mut fine, mut coarse) = (ramp.clone(), ramp.clone());
    paint_daubs(&mut fine, 1, 20, BrushType::Simple, 5).unwrap();
    paint_daubs(&mut coarse, 50, 20, BrushType::Simple, 5).unwrap();
    // A bigger brush lays wider daubs: fewer distinct tones survive.
    let n = ramp.pixel_count();
    let tones = |b: &PixelBuffer| {
        b.data[..n]
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    };
    assert!(
        tones(&coarse) < tones(&fine),
        "brush 50 tones ({}) must be fewer than brush 1 tones ({})",
        tones(&coarse),
        tones(&fine)
    );
}

#[test]
fn sponge_is_seeded() {
    let base = gradient(32, 8, 3);
    let (mut d, mut e, mut f) = (base.clone(), base.clone(), base.clone());
    sponge(&mut d, 6, 20, 4, 21).unwrap();
    sponge(&mut e, 6, 20, 4, 21).unwrap();
    sponge(&mut f, 6, 20, 4, 22).unwrap();
    assert_eq!(d.data, e.data, "sponge same seed must be bit-identical");
    assert_ne!(d.data, f.data, "sponge different seed must differ");
}

#[test]
fn plastic_wrap_strength_raises_max_luminance() {
    let w = 32u32;
    let h = 8u32;
    let n = (w * h) as usize;
    // A bumpy surface: a flat one has no relief to catch the light.
    let mut mid = PixelBuffer::new(w, h, 3);
    for y in 0..h as usize {
        for x in 0..w as usize {
            let v = (128.0 + 60.0 * ((x as f64 / 3.0).sin() * (y as f64 / 3.0).cos())) as u8;
            for c in 0..3 {
                mid.data[c * n + y * w as usize + x] = v;
            }
        }
    }
    let (mut zero, mut full) = (mid.clone(), mid.clone());
    plastic_wrap(&mut zero, 0, 8, 4).unwrap();
    plastic_wrap(&mut full, 20, 8, 4).unwrap();

    let max_lum = |b: &PixelBuffer| -> f64 {
        (0..n)
            .map(|i| {
                luma(
                    b.data[i] as f64,
                    b.data[n + i] as f64,
                    b.data[2 * n + i] as f64,
                )
            })
            .fold(0.0, f64::max)
    };
    assert!(
        max_lum(&full) > max_lum(&zero),
        "highlight 20 ({}) must raise max luminance over 0 ({})",
        max_lum(&full),
        max_lum(&zero)
    );
}

#[test]
fn film_grain_zero_is_noop_and_seed_is_deterministic() {
    let base = gradient(24, 8, 3);
    let mut zero = base.clone();
    // No grain and no highlight area: CS6's highlight smoothing has nothing
    // above its line, so the picture comes back as it was.
    film_grain(&mut zero, 0, 0, 7, 5).unwrap();
    assert_eq!(
        zero, base,
        "grain 0, highlight area 0 must be an exact no-op"
    );

    let (mut a, mut b, mut c) = (base.clone(), base.clone(), base.clone());
    film_grain(&mut a, 12, 4, 6, 77).unwrap();
    film_grain(&mut b, 12, 4, 6, 77).unwrap();
    film_grain(&mut c, 12, 4, 6, 78).unwrap();
    assert_eq!(a.data, b.data, "same seed must be bit-identical");
    assert_ne!(a.data, c.data, "different seed must differ");
}

#[test]
fn cutout_more_levels_yield_more_bands_and_is_deterministic() {
    let base = ramp(64);
    let n = base.pixel_count();
    let (mut two, mut eight) = (base.clone(), base.clone());
    cutout(&mut two, 2, 0, 1).unwrap();
    cutout(&mut eight, 8, 0, 1).unwrap();
    let d2 = distinct(&two.data[..n]);
    let d8 = distinct(&eight.data[..n]);
    assert!(d8 > d2, "8 levels ({d8}) must exceed 2 levels ({d2})");

    let mut again = base.clone();
    cutout(&mut again, 8, 0, 1).unwrap();
    assert_eq!(again.data, eight.data, "cutout must be deterministic");
}

#[test]
fn poster_edges_flattens_and_darkens_the_edge() {
    let flat = ramp(64);
    let n = flat.pixel_count();
    let before = distinct(&flat.data[..n]);
    let mut out = flat.clone();
    poster_edges(&mut out, 2, 0, 4).unwrap();
    assert!(
        distinct(&out.data[..n]) < before,
        "posterize must reduce interior levels ({before} -> {})",
        distinct(&out.data[..n])
    );

    let edged = gradient(32, 1, 3);
    let mut out = edged.clone();
    poster_edges(&mut out, 3, 10, 2).unwrap();
    assert!(
        out.data.iter().zip(&edged.data).any(|(a, b)| a < b),
        "edge band must darken at least one sample"
    );
}

#[test]
fn neon_glow_tints_toward_the_colour_and_spreads_with_size() {
    let base = gradient(64, 8, 3);
    let n = base.pixel_count();

    let mut tinted = base.clone();
    neon_glow(&mut tinted, 6, 50, [255, 0, 0]).unwrap();
    // CS6 renders the picture between the document colours and lights it
    // with the tube: a red tube leaves more red than blue in the result.
    let red: u64 = tinted.data[..n].iter().map(|&v| v as u64).sum();
    let blue: u64 = tinted.data[2 * n..3 * n].iter().map(|&v| v as u64).sum();
    assert!(
        red > blue,
        "the glow colour must show ({red} red vs {blue} blue)"
    );

    let changed = |out: &PixelBuffer| {
        out.data
            .iter()
            .zip(&base.data)
            .filter(|(a, b)| a != b)
            .count()
    };
    let (mut small, mut large) = (base.clone(), base.clone());
    neon_glow(&mut small, 2, 50, [0, 0, 255]).unwrap();
    neon_glow(&mut large, 12, 50, [0, 0, 255]).unwrap();
    assert!(changed(&small) > 0, "small glow must change something");
    assert!(
        changed(&large) >= changed(&small),
        "larger glow must change at least as many samples ({} vs {})",
        changed(&large),
        changed(&small)
    );
}

#[test]
fn tiny_and_three_channel_buffers_do_not_panic() {
    let tiny = PixelBuffer {
        width: 1,
        height: 1,
        channels: 4,
        data: vec![10, 20, 30, 40].into(),
    };
    assert!(cutout(&mut tiny.clone(), 2, 0, 1).is_ok());
    assert!(film_grain(&mut tiny.clone(), 20, 20, 10, 3).is_ok());
    assert!(neon_glow(&mut tiny.clone(), 24, 50, [1, 2, 3]).is_ok());
    assert!(poster_edges(&mut tiny.clone(), 10, 10, 10).is_ok());
    assert!(paint_daubs(&mut tiny.clone(), 50, 40, BrushType::Sparkle, 3).is_ok());
    assert!(palette_knife(&mut tiny.clone(), 50, 3, 10).is_ok());
    assert!(plastic_wrap(&mut tiny.clone(), 20, 15, 15).is_ok());
    assert!(sponge(&mut tiny.clone(), 10, 25, 15, 3).is_ok());

    let rgb = gradient(3, 3, 3);
    assert!(cutout(&mut rgb.clone(), 5, 10, 1).is_ok());
    assert!(film_grain(&mut rgb.clone(), 5, 5, 5, 1).is_ok());
    assert!(neon_glow(&mut rgb.clone(), -24, 10, [9, 9, 9]).is_ok());
    assert!(poster_edges(&mut rgb.clone(), 1, 1, 1).is_ok());
    assert!(paint_daubs(&mut rgb.clone(), 50, 40, BrushType::WideSharp, 1).is_ok());
    assert!(palette_knife(&mut rgb.clone(), 50, 3, 10).is_ok());
    assert!(plastic_wrap(&mut rgb.clone(), 20, 15, 15).is_ok());
    assert!(sponge(&mut rgb.clone(), 10, 25, 15, 1).is_ok());
}

fn final_four_filters(seed: u64) -> Vec<Filter> {
    let opts = TextureOptions::default();
    vec![
        Filter::RoughPastels {
            stroke_length: 8,
            stroke_detail: 6,
            texture: opts,
            seed,
        },
        Filter::SmudgeStick {
            stroke_length: 4,
            highlight_area: 8,
            intensity: 6,
            seed,
        },
        Filter::Underpainting {
            brush_size: 10,
            texture_coverage: 24,
            texture: opts,
            seed,
        },
        Filter::Watercolor {
            brush_detail: 8,
            shadow_intensity: 6,
            texture: 2,
            seed,
        },
    ]
}

#[test]
fn final_four_change_colour_and_preserve_alpha() {
    let base = gradient(16, 6, 4);
    let n = base.pixel_count();
    let expected = alpha_plane(&base);
    for filter in final_four_filters(7) {
        let mut out = base.clone();
        apply(&filter, &mut out).unwrap();
        assert_ne!(
            out.data[..3 * n],
            base.data[..3 * n],
            "{filter:?} did not change colour"
        );
        assert_eq!(alpha_plane(&out), expected, "{filter:?} touched alpha");
    }
}

#[test]
fn final_four_seed_determinism() {
    let base = gradient(32, 8, 3);
    let a = final_four_filters(5);
    let b = final_four_filters(5);
    let c = final_four_filters(6);
    for i in 0..a.len() {
        let (mut oa, mut ob, mut oc) = (base.clone(), base.clone(), base.clone());
        apply(&a[i], &mut oa).unwrap();
        apply(&b[i], &mut ob).unwrap();
        apply(&c[i], &mut oc).unwrap();
        assert_eq!(oa.data, ob.data, "{:?} same seed must match", a[i]);
        assert_ne!(oa.data, oc.data, "{:?} different seed must differ", a[i]);
    }
}

#[test]
fn final_four_validate_ranges_and_texture_options() {
    let base = gradient(8, 4, 3);
    let opts = TextureOptions::default();

    assert!(rough_pastels(&mut base.clone(), 0, 1, opts, 1).is_ok());
    assert!(rough_pastels(&mut base.clone(), 40, 20, opts, 1).is_ok());
    for (l, d) in [(41u8, 6u8), (8, 0), (8, 21)] {
        let mut out = base.clone();
        assert!(matches!(
            rough_pastels(&mut out, l, d, opts, 1),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected rough pastels modified the buffer");
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
            rough_pastels(&mut out, 8, 6, bad, 1),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected rough pastels texture modified buffer");
    }

    assert!(smudge_stick(&mut base.clone(), 0, 0, 0, 1).is_ok());
    assert!(smudge_stick(&mut base.clone(), 10, 20, 10, 1).is_ok());
    for (l, ha, i) in [(11u8, 0u8, 0u8), (0, 21, 0), (0, 0, 11)] {
        let mut out = base.clone();
        assert!(matches!(
            smudge_stick(&mut out, l, ha, i, 1),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected smudge stick modified the buffer");
    }

    assert!(underpainting(&mut base.clone(), 0, 0, opts, 1).is_ok());
    assert!(underpainting(&mut base.clone(), 40, 40, opts, 1).is_ok());
    for (b, tc) in [(41u8, 0u8), (0, 41)] {
        let mut out = base.clone();
        assert!(matches!(
            underpainting(&mut out, b, tc, opts, 1),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected underpainting modified the buffer");
    }
    let mut out = base.clone();
    assert!(matches!(
        underpainting(
            &mut out,
            10,
            20,
            TextureOptions {
                scaling: 49,
                ..opts
            },
            1
        ),
        Err(FilterError::InvalidParams(_))
    ));
    assert_eq!(out, base);

    assert!(watercolor(&mut base.clone(), 1, 0, 1, 1).is_ok());
    assert!(watercolor(&mut base.clone(), 14, 10, 3, 1).is_ok());
    for (bd, si, t) in [
        (0u8, 0u8, 1u8),
        (15, 0, 1),
        (1, 11, 1),
        (1, 0, 0),
        (1, 0, 4),
    ] {
        let mut out = base.clone();
        assert!(matches!(
            watercolor(&mut out, bd, si, t, 1),
            Err(FilterError::InvalidParams(_))
        ));
        assert_eq!(out, base, "rejected watercolor modified the buffer");
    }
}

#[test]
fn rough_pastels_and_underpainting_texture_options_change_output() {
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

    let mut r_canvas = base.clone();
    let mut r_brick = base.clone();
    rough_pastels(&mut r_canvas, 10, 8, canvas, 4).unwrap();
    rough_pastels(&mut r_brick, 10, 8, brick, 4).unwrap();
    assert_ne!(
        r_canvas.data, r_brick.data,
        "rough pastels surface must matter"
    );

    let mut r_lit = base.clone();
    rough_pastels(&mut r_lit, 10, 8, lit, 4).unwrap();
    assert_ne!(
        r_canvas.data, r_lit.data,
        "rough pastels light direction must matter"
    );

    let mut u_canvas = base.clone();
    let mut u_brick = base.clone();
    underpainting(&mut u_canvas, 10, 40, canvas, 4).unwrap();
    underpainting(&mut u_brick, 10, 40, brick, 4).unwrap();
    assert_ne!(
        u_canvas.data, u_brick.data,
        "underpainting surface must matter"
    );

    let mut u_lit = base.clone();
    underpainting(&mut u_lit, 10, 40, lit, 4).unwrap();
    assert_ne!(
        u_canvas.data, u_lit.data,
        "underpainting light direction must matter"
    );

    let mut u_zero = base.clone();
    underpainting(&mut u_zero, 10, 0, canvas, 4).unwrap();
    let mut u_full = base.clone();
    underpainting(&mut u_full, 10, 40, canvas, 4).unwrap();
    assert_ne!(u_zero.data, u_full.data, "coverage 0 vs 40 must differ");
}

#[test]
fn final_four_tiny_buffer_is_safe() {
    let opts = TextureOptions::default();
    let tiny = PixelBuffer {
        width: 1,
        height: 1,
        channels: 4,
        data: vec![10, 20, 30, 40].into(),
    };
    assert!(rough_pastels(&mut tiny.clone(), 40, 20, opts, 1).is_ok());
    assert!(smudge_stick(&mut tiny.clone(), 10, 20, 10, 1).is_ok());
    assert!(underpainting(&mut tiny.clone(), 40, 40, opts, 1).is_ok());
    assert!(watercolor(&mut tiny.clone(), 14, 10, 3, 1).is_ok());
}
