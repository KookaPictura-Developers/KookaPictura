//! Replace Color (#164) kernel behaviour.

use pictura_adjust::{
    apply, replace_color_mask, replace_color_result, replace_color_shift_for, AdjustError,
    Adjustment, ReplaceColorParams, ReplaceColorSample,
};
use pictura_core::PixelBuffer;

fn buf3(px: &[[u8; 3]]) -> PixelBuffer {
    let n = px.len();
    let mut data = vec![0u8; n * 3];
    for (i, p) in px.iter().enumerate() {
        data[i] = p[0];
        data[n + i] = p[1];
        data[2 * n + i] = p[2];
    }
    PixelBuffer {
        width: n as u32,
        height: 1,
        channels: 3,
        data: data.into(),
    }
}

fn buf4(px: &[[u8; 4]]) -> PixelBuffer {
    let n = px.len();
    let mut data = vec![0u8; n * 4];
    for (i, p) in px.iter().enumerate() {
        data[i] = p[0];
        data[n + i] = p[1];
        data[2 * n + i] = p[2];
        data[3 * n + i] = p[3];
    }
    PixelBuffer {
        width: n as u32,
        height: 1,
        channels: 4,
        data: data.into(),
    }
}

fn px3(b: &PixelBuffer, i: usize) -> [u8; 3] {
    let n = b.pixel_count();
    [b.data[i], b.data[n + i], b.data[2 * n + i]]
}

fn sample(x: i32, rgb: [u8; 3]) -> ReplaceColorSample {
    ReplaceColorSample { x, y: 0, rgb }
}

fn params(samples: Vec<ReplaceColorSample>, fuzziness: f64, hue: f64) -> Adjustment {
    Adjustment::ReplaceColor(ReplaceColorParams {
        samples,
        fuzziness,
        localized: false,
        hue,
        saturation: 0.0,
        lightness: 0.0,
    })
}

#[test]
fn fuzziness_zero_selects_exact_samples_only() {
    let mut b = buf3(&[[255, 0, 0], [250, 0, 0], [0, 0, 255]]);
    apply(&params(vec![sample(0, [255, 0, 0])], 0.0, 120.0), &mut b).unwrap();
    assert_eq!(px3(&b, 0), [0, 255, 0], "the exact sample rotates to green");
    assert_eq!(
        px3(&b, 1),
        [250, 0, 0],
        "a near match stays put at fuzziness 0"
    );
    assert_eq!(px3(&b, 2), [0, 0, 255], "an unrelated colour stays put");
}

#[test]
fn positive_fuzziness_shifts_near_colours() {
    let mut b = buf3(&[[250, 0, 0]]);
    apply(&params(vec![sample(0, [255, 0, 0])], 40.0, 120.0), &mut b).unwrap();
    let [r, g, _] = px3(&b, 0);
    assert!(
        r < 250 && g > 0,
        "the near match should move toward green: {r},{g}"
    );
}

#[test]
fn empty_samples_is_identity() {
    let before = buf3(&[[10, 20, 30], [200, 100, 50]]);
    let mut b = before.clone();
    apply(&params(Vec::new(), 40.0, 120.0), &mut b).unwrap();
    assert_eq!(b.data, before.data);
}

#[test]
fn alpha_is_untouched() {
    let mut b = buf4(&[[255, 0, 0, 17], [0, 255, 0, 200]]);
    apply(&params(vec![sample(0, [255, 0, 0])], 40.0, 90.0), &mut b).unwrap();
    let n = b.pixel_count();
    assert_eq!(&b.data[3 * n..], &[17, 200]);
}

#[test]
fn invalid_params_are_rejected() {
    let sample = sample(0, [255, 0, 0]);
    let cases = [
        ReplaceColorParams {
            samples: vec![sample],
            fuzziness: 300.0,
            localized: false,
            hue: 0.0,
            saturation: 0.0,
            lightness: 0.0,
        },
        ReplaceColorParams {
            samples: vec![sample],
            fuzziness: 40.0,
            localized: false,
            hue: 200.0,
            saturation: 0.0,
            lightness: 0.0,
        },
        ReplaceColorParams {
            samples: vec![sample],
            fuzziness: 40.0,
            localized: false,
            hue: 0.0,
            saturation: 101.0,
            lightness: 0.0,
        },
        ReplaceColorParams {
            samples: vec![sample],
            fuzziness: 40.0,
            localized: false,
            hue: 0.0,
            saturation: 0.0,
            lightness: -101.0,
        },
    ];
    for p in cases {
        let mut b = buf3(&[[255, 0, 0]]);
        assert!(matches!(
            apply(&Adjustment::ReplaceColor(p), &mut b),
            Err(AdjustError::InvalidParams(_))
        ));
    }
}

#[test]
fn mask_weights_exact_samples_full_and_others_zero() {
    let b = buf3(&[[255, 0, 0], [0, 0, 255]]);
    let p = ReplaceColorParams {
        samples: vec![sample(0, [255, 0, 0])],
        fuzziness: 0.0,
        localized: false,
        hue: 0.0,
        saturation: 0.0,
        lightness: 0.0,
    };
    let mask = replace_color_mask(&b, &p, 2);
    assert_eq!((mask.width, mask.height, mask.channels), (2, 1, 1));
    assert_eq!(&mask.data[..], &[255, 0]);

    let p = ReplaceColorParams {
        samples: vec![sample(0, [255, 0, 0])],
        fuzziness: 40.0,
        localized: false,
        hue: 0.0,
        saturation: 0.0,
        lightness: 0.0,
    };
    let mask = replace_color_mask(&buf3(&[[250, 0, 0]]), &p, 1);
    assert_eq!(mask.data[0], (0.875f64 * 255.0 + 0.5) as u8);
}

#[test]
fn localized_ignores_a_sample_with_no_canvas_position() {
    // The foreground colour the dialog opens on has x = y = -1; Localized
    // must not pull its weight toward the canvas origin.
    let p = ReplaceColorParams {
        samples: vec![sample(-1, [200, 0, 0])],
        fuzziness: 40.0,
        localized: true,
        hue: 0.0,
        saturation: 0.0,
        lightness: 0.0,
    };
    let b = PixelBuffer {
        width: 64,
        height: 1,
        channels: 3,
        data: buf3(&[[200, 0, 0]; 64]).data,
    };
    let mask = replace_color_mask(&b, &p, 64);
    assert_eq!(mask.data[0], 255);
    assert_eq!(mask.data[63], 255, "the far pixel is not falloff-weighted");
}

#[test]
fn saturation_scales_chroma_so_dark_noise_does_not_turn_vivid() {
    // Near-black JPEG noise in two different hues, and a dark brown. Raising
    // saturation must keep the noise dark and close to neutral (the old
    // additive HSL boost made each pixel 81 % saturated in its noise hue).
    let mut b = buf3(&[[12, 10, 14], [10, 13, 10], [60, 40, 30]]);
    apply(
        &Adjustment::ReplaceColor(ReplaceColorParams {
            samples: vec![sample(-1, [0, 0, 0])],
            fuzziness: 200.0,
            localized: false,
            hue: 0.0,
            saturation: 81.0,
            lightness: 0.0,
        }),
        &mut b,
    )
    .unwrap();
    for i in 0..2 {
        let p = px3(&b, i);
        let spread = p.iter().max().unwrap() - p.iter().min().unwrap();
        assert!(spread <= 20, "noise pixel {i} stays near neutral: {p:?}");
    }
    let brown = px3(&b, 2);
    assert!(
        brown[0] > 60 && brown[2] < 30,
        "the brown deepens: {brown:?}"
    );
}

#[test]
fn lightness_blends_each_channel_toward_white() {
    let mut b = buf3(&[[0, 0, 0]]);
    apply(
        &Adjustment::ReplaceColor(ReplaceColorParams {
            samples: vec![sample(-1, [0, 0, 0])],
            fuzziness: 200.0,
            localized: false,
            hue: 0.0,
            saturation: 0.0,
            lightness: 50.0,
        }),
        &mut b,
    )
    .unwrap();
    assert_eq!(px3(&b, 0), [128, 128, 128]);
}

#[test]
fn the_shift_for_a_result_reaches_it() {
    // Each pair is reachable: a chromatic sample to a colour, lighter,
    // darker, or gray.
    for (sample, result) in [
        ([200, 40, 30], [30, 60, 200]),
        ([120, 90, 60], [220, 200, 180]),
        ([90, 140, 200], [20, 30, 50]),
        ([90, 140, 200], [128, 128, 128]),
    ] {
        let (hue, sat, light) = replace_color_shift_for(sample, result);
        let reached = replace_color_result(sample, hue, sat, light);
        assert!(
            reached.iter().zip(result).all(|(a, b)| a.abs_diff(b) <= 3),
            "{sample:?} -> {result:?}: ({hue}, {sat}, {light}) reaches {reached:?}"
        );
    }
}

#[test]
fn a_gray_sample_takes_lightness_but_no_colour() {
    // CS6 Help: pure gray, black, and white cannot be replaced with a colour.
    let (hue, sat, light) = replace_color_shift_for([0, 0, 0], [0, 0, 255]);
    assert_eq!((hue, sat, light), (0.0, 0.0, 50.0));
    assert_eq!(
        replace_color_result([0, 0, 0], 120.0, 100.0, 50.0),
        [128, 128, 128]
    );
}
