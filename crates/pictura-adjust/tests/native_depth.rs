//! Native-depth tonal application: the 8-bit parity pin, the precision rules
//! for `u16`/`f32` stores, and the unsupported ceiling.

use pictura_adjust::{
    apply, apply_native, AdjustError, Adjustment, BrightnessContrastParams, CurvesParams,
    ExposureParams, GradientMapParams, GradientStop, HueSaturationParams, LevelsParams,
};
use pictura_core::{PixelBuffer, Samples};

const W: u32 = 8;
const H: u32 = 8;

fn fixed_rgb() -> PixelBuffer {
    let n = (W * H) as usize;
    let mut data = vec![0u8; n * 3];
    for i in 0..n {
        data[i] = ((i * 37 + 11) % 256) as u8;
        data[n + i] = ((i * 53 + 29) % 256) as u8;
        data[2 * n + i] = ((i * 91 + 7) % 256) as u8;
    }
    // A 1:2:3 pixel makes the desaturate average land on 1.5, a rounding edge.
    data[0] = 1;
    data[n] = 2;
    data[2 * n] = 3;
    PixelBuffer {
        width: W,
        height: H,
        channels: 3,
        data,
    }
}

fn covered() -> Vec<Adjustment> {
    vec![
        Adjustment::Invert,
        Adjustment::Desaturate,
        Adjustment::Levels(LevelsParams {
            input_black: 10,
            input_white: 240,
            gamma: 1.3,
            output_black: 5,
            output_white: 250,
        }),
        Adjustment::Curves(CurvesParams {
            points: vec![(0, 0), (64, 80), (192, 180), (255, 255)],
            red: Some(vec![(0, 0), (128, 100), (255, 255)]),
            green: None,
            blue: Some(vec![(0, 10), (255, 245)]),
        }),
        Adjustment::BrightnessContrast(BrightnessContrastParams {
            brightness: 30,
            contrast: 20,
            use_legacy: false,
        }),
        Adjustment::BrightnessContrast(BrightnessContrastParams {
            brightness: -40,
            contrast: -15,
            use_legacy: true,
        }),
        Adjustment::Exposure(ExposureParams {
            exposure: 0.8,
            offset: 0.05,
            gamma: 1.1,
        }),
        Adjustment::Posterize(7),
        Adjustment::Threshold(120),
        Adjustment::GradientMap(GradientMapParams {
            stops: vec![
                GradientStop {
                    location: 0,
                    color: [10, 0, 40],
                },
                GradientStop {
                    location: 2048,
                    color: [200, 100, 0],
                },
                GradientStop {
                    location: 4096,
                    color: [255, 255, 255],
                },
            ],
            reverse: false,
        }),
        Adjustment::GradientMap(GradientMapParams {
            stops: vec![
                GradientStop {
                    location: 0,
                    color: [0, 0, 0],
                },
                GradientStop {
                    location: 4096,
                    color: [255, 255, 255],
                },
            ],
            reverse: true,
        }),
    ]
}

#[test]
fn native_u8_equals_apply_for_every_covered_adjustment() {
    let base = fixed_rgb();
    for adjustment in covered() {
        let mut expected = base.clone();
        apply(&adjustment, &mut expected).unwrap();
        let mut store = Samples::U8(base.data.clone());
        apply_native(&adjustment, &mut store, W as usize, H as usize, 3).unwrap();
        assert_eq!(store, Samples::U8(expected.data), "{adjustment:?}");
    }
}

#[test]
fn native_u8_leaves_alpha_untouched() {
    let n = (W * H) as usize;
    let mut data = fixed_rgb().data;
    data.extend((0..n).map(|i| (i * 17 + 3) as u8));
    let mut expected = PixelBuffer {
        width: W,
        height: H,
        channels: 4,
        data: data.clone(),
    };
    apply(&Adjustment::Invert, &mut expected).unwrap();
    let mut store = Samples::U8(data);
    apply_native(&Adjustment::Invert, &mut store, W as usize, H as usize, 4).unwrap();
    assert_eq!(store, Samples::U8(expected.data));
}

#[test]
fn depth16_native_edit_is_not_the_widened_byte() {
    let mut store = Samples::U16(vec![
        1000, 5000, 20000, 40000, 60000, 12345, 1, 255, 256, 257, 32768, 65535,
    ]);
    let levels = Adjustment::Levels(LevelsParams {
        input_black: 0,
        input_white: 255,
        gamma: 1.0,
        output_black: 0,
        output_white: 200,
    });
    apply_native(&levels, &mut store, 4, 1, 3).unwrap();
    let Samples::U16(v) = &store else { panic!() };
    assert!(
        v.iter().any(|s| (s >> 8) != (s & 0xff)),
        "expected a sample whose low byte is not the high byte: {v:?}"
    );
}

#[test]
fn depth32_native_edit_keeps_values_above_one() {
    let mut store = Samples::F32(vec![1.0, 0.5, 0.25, 0.0, 0.75, 1.0]);
    let exposure = Adjustment::Exposure(ExposureParams {
        exposure: 2.0,
        offset: 0.0,
        gamma: 1.0,
    });
    apply_native(&exposure, &mut store, 2, 1, 3).unwrap();
    let Samples::F32(v) = &store else { panic!() };
    assert!(
        v.iter().any(|&s| s > 1.0),
        "expected an HDR sample above 1.0: {v:?}"
    );
}

#[test]
fn unsupported_adjustment_is_refused_without_mutation() {
    let mut store = Samples::U16(vec![10, 20, 30, 40, 50, 60]);
    let before = store.clone();
    let hue = Adjustment::HueSaturation(HueSaturationParams {
        hue: 10,
        saturation: 20,
        lightness: -5,
    });
    let err = apply_native(&hue, &mut store, 2, 1, 3).unwrap_err();
    assert!(matches!(err, AdjustError::Unsupported(_)));
    assert_eq!(store, before);
}
