use super::*;
use pictura_adjust::{
    BlackWhiteParams, ExposureParams, GradientMapParams, GradientStop, PhotoFilterParams,
    VibranceParams,
};
use pictura_codec::{write_descriptor, DescValue};

fn exposure_payload(exposure: f32, offset: f32, gamma: f32) -> Vec<u8> {
    let mut data = 1u16.to_be_bytes().to_vec();
    data.extend_from_slice(&exposure.to_be_bytes());
    data.extend_from_slice(&offset.to_be_bytes());
    data.extend_from_slice(&gamma.to_be_bytes());
    data
}

fn phfl_payload(version: u16, components: [u16; 4], density: u32, luminosity: u8) -> Vec<u8> {
    let mut data = version.to_be_bytes().to_vec();
    data.extend_from_slice(&0u16.to_be_bytes());
    for c in components {
        data.extend_from_slice(&c.to_be_bytes());
    }
    data.extend_from_slice(&density.to_be_bytes());
    data.push(luminosity);
    data.extend_from_slice(&[0, 0, 0]);
    data
}

fn grdm_payload(
    version: u16,
    reverse: u8,
    dither: u8,
    name: &str,
    stops: &[(u32, [u16; 4])],
) -> Vec<u8> {
    let mut data = version.to_be_bytes().to_vec();
    data.push(reverse);
    data.push(dither);
    if version == 3 {
        data.extend_from_slice(b"Gcls");
    }
    let utf16: Vec<u16> = name.encode_utf16().collect();
    data.extend_from_slice(&(utf16.len() as u32).to_be_bytes());
    for u in utf16 {
        data.extend_from_slice(&u.to_be_bytes());
    }
    data.extend_from_slice(&(stops.len() as u16).to_be_bytes());
    for (location, color) in stops {
        data.extend_from_slice(&location.to_be_bytes());
        data.extend_from_slice(&50u32.to_be_bytes());
        data.extend_from_slice(&0u16.to_be_bytes());
        for c in color {
            data.extend_from_slice(&c.to_be_bytes());
        }
        data.extend_from_slice(&[0, 0]);
    }
    data
}

fn desc_object(name: &str, class_id: &[u8], items: Vec<(Vec<u8>, DescValue)>) -> DescValue {
    DescValue::Object {
        name: name.to_string(),
        class_id: class_id.to_vec(),
        items,
    }
}

#[test]
fn decode_adjustment_subset_and_unknown() {
    assert_eq!(
        decode_adjustment(&adjdata(*b"nvrt", vec![])),
        Some(Adjustment::Invert)
    );
    assert_eq!(
        decode_adjustment(&adjdata(*b"invr", vec![])),
        Some(Adjustment::Invert)
    );
    assert_eq!(
        decode_adjustment(&adjdata(*b"post", vec![0, 4, 0, 0])),
        Some(Adjustment::Posterize(4))
    );
    assert_eq!(
        decode_adjustment(&adjdata(*b"thrs", vec![0, 128, 0, 0])),
        Some(Adjustment::Threshold(128))
    );
    assert_eq!(
        decode_adjustment(&adjdata(*b"brit", vec![0, 10, 0, 20, 0, 0, 0, 0])),
        Some(Adjustment::BrightnessContrast(BrightnessContrastParams {
            brightness: 10,
            contrast: 20,
            use_legacy: false,
        }))
    );

    let mut levels = vec![0, 2];
    for v in [5u16, 250, 10, 240, 120] {
        levels.extend_from_slice(&v.to_be_bytes());
    }
    assert_eq!(
        decode_adjustment(&adjdata(*b"levl", levels)),
        Some(Adjustment::Levels(LevelsParams {
            input_black: 5,
            input_white: 250,
            gamma: 1.2,
            output_black: 10,
            output_white: 240,
        }))
    );

    let hue = vec![0, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0, 20, 0, 30];
    assert_eq!(
        decode_adjustment(&adjdata(*b"hue2", hue)),
        Some(Adjustment::HueSaturation(HueSaturationParams {
            hue: 10,
            saturation: 20,
            lightness: 30,
        }))
    );

    // Unknown key and undecodable payloads are a no-op, never an error.
    assert_eq!(decode_adjustment(&adjdata(*b"zzzz", vec![1, 2, 3])), None);
    assert_eq!(decode_adjustment(&adjdata(*b"clrL", vec![1, 2, 3])), None);
    assert_eq!(decode_adjustment(&adjdata(*b"post", vec![0, 0])), None);
    assert_eq!(decode_adjustment(&adjdata(*b"levl", vec![0, 3])), None);
}

#[test]
fn solid_fill_decodes_both_payload_forms() {
    // The in-house 4-byte straight-alpha tuple is unchanged.
    assert_eq!(
        decode_adjustment(&adjdata(*b"SoCo", vec![10, 20, 30, 40])),
        Some(Adjustment::SolidFill([10, 20, 30, 40]))
    );
    // The standard descriptor decodes with alpha forced to 255.
    assert_eq!(
        decode_adjustment(&encode_solid_color_fill([10, 20, 30])),
        Some(Adjustment::SolidFill([10, 20, 30, 255]))
    );

    // A non-4-byte, non-descriptor payload is a no-op.
    for payload in [vec![], vec![1, 2, 3], vec![1, 2, 3, 4, 5]] {
        assert_eq!(decode_adjustment(&adjdata(*b"SoCo", payload)), None);
    }
    // A descriptor missing `Clr ` is a no-op.
    let no_clr = write_descriptor(&desc_object("", b"SoCo", vec![]));
    assert_eq!(decode_adjustment(&adjdata(*b"SoCo", no_clr)), None);
    // A `Clr ` object with a wrong-typed component is a no-op.
    let wrong_type = write_descriptor(&desc_object(
        "",
        b"SoCo",
        vec![(
            b"Clr ".to_vec(),
            desc_object(
                "",
                b"RGBC",
                vec![
                    (b"Rd  ".to_vec(), DescValue::Long(10)),
                    (b"Grn ".to_vec(), DescValue::Double(20.0)),
                    (b"Bl  ".to_vec(), DescValue::Double(30.0)),
                ],
            ),
        )],
    ));
    assert_eq!(decode_adjustment(&adjdata(*b"SoCo", wrong_type)), None);
    // A non-finite component is a no-op.
    let non_finite = write_descriptor(&desc_object(
        "",
        b"SoCo",
        vec![(
            b"Clr ".to_vec(),
            desc_object(
                "",
                b"RGBC",
                vec![
                    (b"Rd  ".to_vec(), DescValue::Double(f64::NAN)),
                    (b"Grn ".to_vec(), DescValue::Double(20.0)),
                    (b"Bl  ".to_vec(), DescValue::Double(30.0)),
                ],
            ),
        )],
    ));
    assert_eq!(decode_adjustment(&adjdata(*b"SoCo", non_finite)), None);
}

#[test]
fn encoded_solid_fill_is_version16_descriptor() {
    let encoded = encode_solid_color_fill([10, 20, 30]);
    assert_eq!(encoded.key, *b"SoCo");
    assert_eq!(
        decode_adjustment(&encoded),
        Some(Adjustment::SolidFill([10, 20, 30, 255]))
    );

    let desc = pictura_codec::read_descriptor(&encoded.data).expect("version-16 descriptor");
    let DescValue::Object { items, .. } = desc else {
        panic!("top level is an object");
    };
    let clr = items
        .iter()
        .find(|(k, _)| k.as_slice() == b"Clr ")
        .map(|(_, v)| v)
        .expect("Clr item");
    let DescValue::Object { items, .. } = clr else {
        panic!("Clr is an object");
    };
    let get = |key: &[u8]| {
        items
            .iter()
            .find(|(k, _)| k.as_slice() == key)
            .map(|(_, v)| v)
            .expect("component")
    };
    assert_eq!(get(b"Rd  "), &DescValue::Double(10.0));
    assert_eq!(get(b"Grn "), &DescValue::Double(20.0));
    assert_eq!(get(b"Bl  "), &DescValue::Double(30.0));
}

#[test]
fn encode_decode_round_trips() {
    assert_eq!(
        decode_adjustment(&encode_invert()),
        Some(Adjustment::Invert)
    );
    assert_eq!(
        decode_adjustment(&encode_posterize(4)),
        Some(Adjustment::Posterize(4))
    );
    assert_eq!(
        decode_adjustment(&encode_threshold(128)),
        Some(Adjustment::Threshold(128))
    );
    assert_eq!(
        decode_adjustment(&encode_brightness_contrast(10, 20)),
        Some(Adjustment::BrightnessContrast(BrightnessContrastParams {
            brightness: 10,
            contrast: 20,
            use_legacy: false,
        }))
    );
    assert_eq!(
        decode_adjustment(&encode_hue_saturation(10, 20, 30)),
        Some(Adjustment::HueSaturation(HueSaturationParams {
            hue: 10,
            saturation: 20,
            lightness: 30,
        }))
    );
    assert_eq!(
        decode_adjustment(&encode_photo_filter([255, 180, 80], 25.0, true)),
        Some(Adjustment::PhotoFilter(PhotoFilterParams {
            color: [255, 180, 80],
            density: 25.0,
            preserve_luminosity: true,
        }))
    );

    // Byte formats match the psd-tools fixtures (`H2x`, `3HBx`).
    assert_eq!(encode_invert().key, *b"nvrt");
    assert!(encode_invert().data.is_empty());
    assert_eq!(encode_posterize(4).data, [0, 4, 0, 0]);
    assert_eq!(encode_threshold(128).data, [0, 128, 0, 0]);
    assert_eq!(
        encode_brightness_contrast(10, 20).data,
        [0, 10, 0, 20, 0, 0, 0, 0]
    );
    assert_eq!(encode_hue_saturation(10, 20, 30).data.len(), 100);
    assert_eq!(encode_hue_saturation(10, 20, 30).data[0..2], [0, 2]);
    let phfl = encode_photo_filter([255, 180, 80], 25.0, true);
    assert_eq!(phfl.key, *b"phfl");
    assert_eq!(phfl.data.len(), 20);
    assert_eq!(
        phfl.data,
        [0, 2, 0, 0, 0, 255, 0, 180, 0, 80, 0, 0, 0, 0, 0, 25, 1, 0, 0, 0]
    );

    let gm_stops = [
        GradientStop {
            location: 0,
            color: [0, 0, 0],
        },
        GradientStop {
            location: 2048,
            color: [255, 0, 0],
        },
        GradientStop {
            location: 4096,
            color: [255, 255, 255],
        },
    ];
    let gm = encode_gradient_map(&gm_stops, true);
    assert_eq!(gm.key, *b"grdm");
    assert_eq!(
        decode_adjustment(&gm),
        Some(Adjustment::GradientMap(GradientMapParams {
            stops: gm_stops.to_vec(),
            reverse: true,
        }))
    );
    assert_eq!(gm.data.len() % 4, 0, "block is padded to 4 bytes");

    // Out-of-range inputs are clamped to what the decoder accepts.
    assert_eq!(
        decode_adjustment(&encode_posterize(0)),
        Some(Adjustment::Posterize(2))
    );
    assert_eq!(
        decode_adjustment(&encode_threshold(0)),
        Some(Adjustment::Threshold(1))
    );
    assert_eq!(
        decode_adjustment(&encode_brightness_contrast(999, -999)),
        Some(Adjustment::BrightnessContrast(BrightnessContrastParams {
            brightness: 150,
            contrast: -50,
            use_legacy: false,
        }))
    );
    assert_eq!(
        decode_adjustment(&encode_photo_filter([255, 180, 80], 150.0, false)),
        Some(Adjustment::PhotoFilter(PhotoFilterParams {
            color: [255, 180, 80],
            density: 100.0,
            preserve_luminosity: false,
        }))
    );
    assert_eq!(
        decode_adjustment(&encode_photo_filter([255, 180, 80], -5.0, false)),
        Some(Adjustment::PhotoFilter(PhotoFilterParams {
            color: [255, 180, 80],
            density: 0.0,
            preserve_luminosity: false,
        }))
    );
}

#[test]
fn invert_adjustment_layer_matches_flattened() {
    // Varied RGB backdrop so Invert is not a uniform all-zero/all-one case.
    let base = solid(
        "base",
        full(4, 2),
        (30, 90, 210),
        255,
        BlendMode::Normal,
        255,
    );
    let with_adj = doc(
        4,
        2,
        vec![
            base.clone(),
            adjustment_layer("invert", *b"nvrt", Vec::new(), 255, None),
        ],
    );
    let out = composite_rgba(&with_adj);
    // Expected: apply the destructive Adjustment to the flattened composite.
    let mut flat = composite_rgba(&doc(4, 2, vec![base]));
    pictura_adjust::apply(&Adjustment::Invert, &mut flat).unwrap();
    for y in 0..2 {
        for x in 0..4 {
            let want = rgb(&flat, x, y);
            let got = rgb(&out, x, y);
            for c in 0..3 {
                let d = got[c] as i16 - want[c] as i16;
                assert!(d.abs() <= 1, "at {x},{y}.{c}: got {got:?} want {want:?}");
            }
        }
    }
}

#[test]
fn adjustment_layer_mask_and_opacity_gate() {
    // 2x1: mask fully reveals x=0, hides x=1.
    let base = solid(
        "base",
        full(2, 1),
        (100, 100, 100),
        255,
        BlendMode::Normal,
        255,
    );
    let mut masked = adjustment_layer("invert", *b"nvrt", Vec::new(), 255, None);
    masked.mask = Some(LayerMask {
        rect: full(2, 1),
        default_color: 0,
        disabled: false,
        flags: 0,
        data: Some(vec![255, 0]),
        ..Default::default()
    });
    let out = composite_rgba(&doc(2, 1, vec![base, masked]));
    assert_eq!(rgb(&out, 0, 0), [155, 155, 155], "unmasked pixel inverts");
    assert_eq!(
        rgb(&out, 1, 0),
        [100, 100, 100],
        "masked-out pixel unchanged"
    );

    // Opacity 128 lerps about halfway to the inverted value.
    let base = solid(
        "base",
        full(1, 1),
        (100, 100, 100),
        255,
        BlendMode::Normal,
        255,
    );
    let out = composite_rgba(&doc(
        1,
        1,
        vec![
            base,
            adjustment_layer("invert", *b"nvrt", Vec::new(), 128, None),
        ],
    ));
    let got = rgb(&out, 0, 0)[0] as i16;
    assert!(
        (got - 128).abs() <= 1,
        "50% opacity should land near 128, got {got}"
    );
}

#[test]
fn unknown_adjustment_key_is_noop() {
    let base = solid(
        "base",
        full(2, 2),
        (10, 200, 60),
        255,
        BlendMode::Normal,
        255,
    );
    let plain = composite_rgba(&doc(2, 2, vec![base.clone()]));
    let with_unknown = composite_rgba(&doc(
        2,
        2,
        vec![
            base,
            adjustment_layer("lookup", *b"clrL", vec![1, 2, 3], 255, None),
        ],
    ));
    assert_eq!(
        with_unknown.data, plain.data,
        "undecodable key must be a no-op"
    );
}

#[test]
fn exp_a_decodes_exposure() {
    let payload = exposure_payload(1.5, -0.25, 1.1);
    assert_eq!(
        decode_adjustment(&adjdata(*b"expA", payload.clone())),
        Some(Adjustment::Exposure(ExposureParams {
            exposure: 1.5f32 as f64,
            offset: -0.25f32 as f64,
            gamma: 1.1f32 as f64,
        }))
    );

    // Fewer than the 14 field bytes is a no-op, never an error.
    for cut in 0..payload.len() {
        assert_eq!(
            decode_adjustment(&adjdata(*b"expA", payload[..cut].to_vec())),
            None,
            "cut {cut}"
        );
    }
    assert_eq!(
        decode_adjustment(&adjdata(*b"expA", exposure_payload(1.5, -0.25, 0.0))),
        None,
        "gamma must be positive"
    );
    assert_eq!(
        decode_adjustment(&adjdata(*b"expA", exposure_payload(f32::NAN, 0.0, 1.0))),
        None,
        "non-finite exposure is rejected"
    );

    // The version word must be 1, per the committed payload layout.
    let mut wrong_version = 2u16.to_be_bytes().to_vec();
    wrong_version.extend_from_slice(&1.5f32.to_be_bytes());
    wrong_version.extend_from_slice(&(-0.25f32).to_be_bytes());
    wrong_version.extend_from_slice(&1.1f32.to_be_bytes());
    assert_eq!(
        decode_adjustment(&adjdata(*b"expA", wrong_version)),
        None,
        "version must be 1"
    );
}

#[test]
fn vib_a_decodes_descriptor() {
    let payload = write_descriptor(&desc_object(
        "",
        b"null",
        vec![
            (b"vibrance".to_vec(), DescValue::Long(40)),
            (b"Strt".to_vec(), DescValue::Long(-10)),
        ],
    ));
    assert_eq!(
        decode_adjustment(&adjdata(*b"vibA", payload)),
        Some(Adjustment::Vibrance(VibranceParams {
            vibrance: 40,
            saturation: -10,
        }))
    );

    // Garbage bytes are not a version-16 descriptor object: no-op.
    assert_eq!(decode_adjustment(&adjdata(*b"vibA", vec![1, 2, 3])), None);

    // A present key outside the slider range is rejected, not clamped.
    let out_of_range = write_descriptor(&desc_object(
        "",
        b"null",
        vec![(b"vibrance".to_vec(), DescValue::Long(200))],
    ));
    assert_eq!(decode_adjustment(&adjdata(*b"vibA", out_of_range)), None);
}

#[test]
fn blwh_decodes_descriptor() {
    let tint = desc_object(
        "tint",
        b"Clr ",
        vec![
            (b"Rd  ".to_vec(), DescValue::Double(0.5)),
            (b"Grn ".to_vec(), DescValue::Double(0.25)),
            (b"Bl  ".to_vec(), DescValue::Double(0.75)),
        ],
    );
    let payload = write_descriptor(&desc_object(
        "",
        b"null",
        vec![
            (b"Rd  ".to_vec(), DescValue::Long(40)),
            (b"Yllw".to_vec(), DescValue::Long(60)),
            (b"Grn ".to_vec(), DescValue::Long(40)),
            (b"Cyn ".to_vec(), DescValue::Long(60)),
            (b"Bl  ".to_vec(), DescValue::Long(20)),
            (b"Mgnt".to_vec(), DescValue::Long(80)),
            (b"useTint".to_vec(), DescValue::Bool(true)),
            (b"tintColor".to_vec(), tint),
        ],
    ));
    assert_eq!(
        decode_adjustment(&adjdata(*b"blwh", payload)),
        Some(Adjustment::BlackWhite(BlackWhiteParams {
            red: 40.0,
            yellow: 60.0,
            green: 40.0,
            cyan: 60.0,
            blue: 20.0,
            magenta: 80.0,
            tint: true,
            tint_color: [128, 64, 191],
        }))
    );

    // A non-descriptor payload is a no-op.
    assert_eq!(decode_adjustment(&adjdata(*b"blwh", vec![0, 16, 0])), None);

    // A truncated descriptor and an out-of-range percentage are both rejected.
    let truncated = write_descriptor(&desc_object("", b"null", vec![]));
    assert_eq!(
        decode_adjustment(&adjdata(
            *b"blwh",
            truncated[..truncated.len() - 1].to_vec()
        )),
        None
    );
    let out_of_range = write_descriptor(&desc_object(
        "",
        b"null",
        vec![(b"Rd  ".to_vec(), DescValue::Long(400))],
    ));
    assert_eq!(decode_adjustment(&adjdata(*b"blwh", out_of_range)), None);

    // A descriptor with no channel keys falls back to Photoshop's defaults.
    let empty = write_descriptor(&desc_object("", b"null", vec![]));
    assert_eq!(
        decode_adjustment(&adjdata(*b"blwh", empty)),
        Some(Adjustment::BlackWhite(BlackWhiteParams {
            red: 40.0,
            yellow: 60.0,
            green: 40.0,
            cyan: 60.0,
            blue: 20.0,
            magenta: 80.0,
            tint: false,
            tint_color: [0, 0, 0],
        }))
    );
}

#[test]
fn phfl_decodes_version_two() {
    let payload = phfl_payload(2, [255, 180, 80, 0], 25, 1);
    assert_eq!(payload.len(), 20);
    assert_eq!(
        decode_adjustment(&adjdata(*b"phfl", payload.clone())),
        Some(Adjustment::PhotoFilter(PhotoFilterParams {
            color: [255, 180, 80],
            density: 25.0,
            preserve_luminosity: true,
        }))
    );
    // The fourth component and colour space are ignored, and luminosity is a
    // boolean.
    assert_eq!(
        decode_adjustment(&adjdata(*b"phfl", phfl_payload(2, [1, 2, 3, 999], 0, 0))),
        Some(Adjustment::PhotoFilter(PhotoFilterParams {
            color: [1, 2, 3],
            density: 0.0,
            preserve_luminosity: false,
        }))
    );

    // Fewer than the 17 field bytes is a no-op, never an error.
    for cut in 0..17 {
        assert_eq!(
            decode_adjustment(&adjdata(*b"phfl", payload[..cut].to_vec())),
            None,
            "cut {cut}"
        );
    }
    // Version 3 (CIE XYZ), a component above 255, and density above 100 are
    // all rejected.
    assert_eq!(
        decode_adjustment(&adjdata(*b"phfl", phfl_payload(3, [0, 0, 0, 0], 25, 1))),
        None,
        "version must be 2"
    );
    assert_eq!(
        decode_adjustment(&adjdata(
            *b"phfl",
            phfl_payload(2, [256, 180, 80, 0], 25, 1)
        )),
        None,
        "component must be 0..=255"
    );
    assert_eq!(
        decode_adjustment(&adjdata(
            *b"phfl",
            phfl_payload(2, [255, 180, 80, 0], 101, 1)
        )),
        None,
        "density must be 0..=100"
    );
}

#[test]
fn grdm_decodes_versions_and_rejects_malformed() {
    let stops = [
        (0u32, [0u16, 0, 0, 0]),
        (4096u32, [65535u16, 65535, 65535, 0]),
    ];
    let expected = Adjustment::GradientMap(GradientMapParams {
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
        reverse: false,
    });

    let payload = grdm_payload(1, 0, 0, "Black to White", &stops);
    assert_eq!(
        decode_adjustment(&adjdata(*b"grdm", payload.clone())),
        Some(expected.clone())
    );
    assert_eq!(
        decode_adjustment(&adjdata(
            *b"grdm",
            grdm_payload(1, 1, 0, "Black to White", &stops)
        )),
        Some(Adjustment::GradientMap(GradientMapParams {
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
        }))
    );
    // Version 3 carries a 4-byte method after the flags; the stops are the same.
    assert_eq!(
        decode_adjustment(&adjdata(
            *b"grdm",
            grdm_payload(3, 0, 1, "Black to White", &stops)
        )),
        Some(expected.clone())
    );

    // Truncation anywhere before the last stop's colour is a no-op, never an
    // error. The final 4 bytes (stop pad + transparency count) are ignored.
    for cut in 0..payload.len() - 4 {
        assert_eq!(
            decode_adjustment(&adjdata(*b"grdm", payload[..cut].to_vec())),
            None,
            "cut {cut}"
        );
    }
    // The in-house `gdrm` spelling is accepted as an alias.
    assert_eq!(
        decode_adjustment(&adjdata(*b"gdrm", payload.clone())),
        Some(expected.clone())
    );
    // Unsupported version.
    assert_eq!(
        decode_adjustment(&adjdata(*b"grdm", grdm_payload(2, 0, 0, "", &stops))),
        None
    );
    // Fewer than two stops.
    assert_eq!(
        decode_adjustment(&adjdata(*b"grdm", grdm_payload(1, 0, 0, "", &stops[..1]))),
        None
    );
    // Non-increasing locations.
    let decreasing = [(4096u32, [0u16, 0, 0, 0]), (0u32, [0u16, 0, 0, 0])];
    assert_eq!(
        decode_adjustment(&adjdata(*b"grdm", grdm_payload(1, 0, 0, "", &decreasing))),
        None
    );
    // A location above 4096.
    let out_of_range = [(0u32, [0u16, 0, 0, 0]), (5000u32, [0u16, 0, 0, 0])];
    assert_eq!(
        decode_adjustment(&adjdata(*b"grdm", grdm_payload(1, 0, 0, "", &out_of_range))),
        None
    );
}

#[test]
fn descriptor_solid_fill_layer_composites_to_its_color() {
    let mut fill = adjustment_layer(
        "fill",
        *b"SoCo",
        encode_solid_color_fill([10, 20, 30]).data,
        255,
        None,
    );
    fill.rect = full(2, 2);
    let out = composite_rgba(&doc(2, 2, vec![fill]));
    assert_eq!(rgb(&out, 0, 0), [10, 20, 30]);
    assert_eq!(rgb(&out, 1, 1), [10, 20, 30]);
    assert_eq!(px(&out, 1, 1)[3], 255, "descriptor fills are opaque");
}

#[test]
fn fixture_solid_fill_decodes_descriptor() {
    let bytes = include_bytes!("../../../pictura-codec/tests/fixtures/solid_fill.psd");
    let d = pictura_codec::read_psd(bytes).expect("fixture parses");
    let fill = d
        .layers
        .iter()
        .find(|l| l.name == "Solid Fill")
        .expect("Solid Fill layer");
    assert_eq!(
        decode_adjustment(fill.adjustment.as_ref().expect("SoCo block")),
        Some(Adjustment::SolidFill([10, 20, 30, 255]))
    );
}

#[test]
fn deferred_keys_still_none() {
    for key in [*b"curv", *b"mixr", *b"selc", *b"clrL"] {
        assert_eq!(
            decode_adjustment(&adjdata(key, vec![1, 2, 3, 4])),
            None,
            "deferred key {key:?}"
        );
    }
    // Version-3 `phfl` (three u32 CIE XYZ values) is still deferred.
    let mut v3 = 3u16.to_be_bytes().to_vec();
    v3.extend_from_slice(&[0u8; 12]);
    v3.extend_from_slice(&25u32.to_be_bytes());
    v3.push(1);
    v3.extend_from_slice(&[0, 0, 0]);
    assert_eq!(
        decode_adjustment(&adjdata(*b"phfl", v3)),
        None,
        "version-3 phfl is deferred"
    );
}

#[test]
fn hidden_decoded_adjustment_is_noop() {
    // A committed key (expA) decodes, but a fully hidden mask must gate it off
    // exactly like the existing `nvrt` path.
    let base = solid(
        "base",
        full(2, 1),
        (120, 60, 200),
        255,
        BlendMode::Normal,
        255,
    );
    let plain = composite_rgba(&doc(2, 1, vec![base.clone()]));
    let mut hidden = adjustment_layer(
        "exposure",
        *b"expA",
        exposure_payload(2.0, 0.5, 0.5),
        255,
        None,
    );
    hidden.mask = Some(LayerMask {
        rect: full(2, 1),
        default_color: 0,
        disabled: false,
        flags: 0,
        data: Some(vec![0, 0]),
        ..Default::default()
    });
    let out = composite_rgba(&doc(2, 1, vec![base, hidden]));
    assert_eq!(
        out.data, plain.data,
        "a fully masked decoded adjustment layer must be a no-op"
    );
}

#[test]
fn decoded_adjustment_changes_backdrop() {
    let base = solid(
        "base",
        full(2, 2),
        (120, 60, 200),
        255,
        BlendMode::Normal,
        255,
    );
    let plain = composite_rgba(&doc(2, 2, vec![base.clone()]));
    let adjusted = composite_rgba(&doc(
        2,
        2,
        vec![
            base,
            adjustment_layer(
                "exposure",
                *b"expA",
                exposure_payload(1.0, 0.0, 1.0),
                255,
                None,
            ),
        ],
    ));
    assert_ne!(
        adjusted.data, plain.data,
        "a decoded expA adjustment layer must change the backdrop"
    );
}

#[test]
fn photo_filter_layer_warms_and_preserves_luminance() {
    // Non-uniform backdrop: two different neutral grays.
    let base = solid(
        "base",
        full(2, 1),
        (120, 120, 120),
        255,
        BlendMode::Normal,
        255,
    );
    let patch = solid(
        "patch",
        rect(0, 0, 1, 1),
        (80, 80, 80),
        255,
        BlendMode::Normal,
        255,
    );
    let plain = composite_rgba(&doc(2, 1, vec![base.clone(), patch.clone()]));
    let filtered = composite_rgba(&doc(
        2,
        1,
        vec![
            base,
            patch,
            adjustment_layer(
                "photo-filter",
                *b"phfl",
                encode_photo_filter([255, 180, 80], 25.0, true).data,
                255,
                None,
            ),
        ],
    ));
    assert_ne!(
        filtered.data, plain.data,
        "a Photo Filter layer must change the backdrop"
    );
    for x in 0..2 {
        let got = rgb(&filtered, x, 0);
        assert!(
            got[0] > got[2],
            "warming filter must give red > blue: {got:?}"
        );
        let before = rgb(&plain, x, 0);
        let luma = |c: [u8; 3]| 0.299 * c[0] as f64 + 0.587 * c[1] as f64 + 0.114 * c[2] as f64;
        assert!(
            (luma(got) - luma(before)).abs() <= 2.0,
            "luminance must be preserved at {x}: {:?} -> {:?}",
            before,
            got
        );
    }
}

#[test]
fn gradient_map_layer_changes_non_uniform_backdrop() {
    // Non-uniform, non-neutral backdrop: black-to-white maps each pixel to the
    // grey of its luminance, changing the colour.
    let base = solid(
        "base",
        full(2, 1),
        (30, 90, 210),
        255,
        BlendMode::Normal,
        255,
    );
    let patch = solid(
        "patch",
        rect(0, 0, 1, 1),
        (200, 100, 50),
        255,
        BlendMode::Normal,
        255,
    );
    let plain = composite_rgba(&doc(2, 1, vec![base.clone(), patch.clone()]));
    let stops = [
        GradientStop {
            location: 0,
            color: [0, 0, 0],
        },
        GradientStop {
            location: 4096,
            color: [255, 255, 255],
        },
    ];
    let graded = composite_rgba(&doc(
        2,
        1,
        vec![
            base,
            patch,
            adjustment_layer(
                "gradient-map",
                *b"grdm",
                encode_gradient_map(&stops, false).data,
                255,
                None,
            ),
        ],
    ));
    assert_ne!(
        graded.data, plain.data,
        "a Gradient Map layer must change the backdrop"
    );
    for x in 0..2 {
        let before = rgb(&plain, x, 0);
        let got = rgb(&graded, x, 0);
        let luma = 0.299 * before[0] as f64 + 0.587 * before[1] as f64 + 0.114 * before[2] as f64;
        assert!(
            got[0].abs_diff(got[1]) <= 1 && got[1].abs_diff(got[2]) <= 1,
            "black-to-white output must be neutral at {x}: {got:?}"
        );
        assert!(
            (got[0] as f64 - luma).abs() <= 2.0,
            "the grey should follow the backdrop's luminance at {x}: {before:?} -> {got:?}"
        );
    }
}
