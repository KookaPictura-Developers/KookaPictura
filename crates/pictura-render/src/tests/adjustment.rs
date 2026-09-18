use super::*;

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
fn solid_fill_decodes_only_the_four_byte_payload() {
    assert_eq!(
        decode_adjustment(&adjdata(*b"SoCo", vec![10, 20, 30, 40])),
        Some(Adjustment::SolidFill([10, 20, 30, 40]))
    );
    // A real Photoshop `'Clr '` descriptor is preserved on disk but not decoded.
    for payload in [vec![], vec![1, 2, 3], vec![1, 2, 3, 4, 5]] {
        assert_eq!(decode_adjustment(&adjdata(*b"SoCo", payload)), None);
    }
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
