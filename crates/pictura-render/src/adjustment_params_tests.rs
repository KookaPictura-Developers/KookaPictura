use super::*;
use crate::{
    encode_brightness_contrast, encode_channel_mixer, encode_color_balance, encode_curves,
    encode_hue_saturation, encode_invert, encode_photo_filter, encode_posterize,
    encode_selective_color,
};
use pictura_adjust::{SelectiveColorMethod, SelectiveRange};

fn value(data: &AdjustmentData, key: &str) -> f64 {
    adjustment_editor(data)
        .unwrap()
        .params
        .into_iter()
        .find(|p| p.key == key)
        .unwrap_or_else(|| panic!("no {key}"))
        .value
}

fn levels(input_black: u16, input_white: u16, gamma: u16) -> AdjustmentData {
    let mut data = 2u16.to_be_bytes().to_vec();
    for v in [input_black, input_white, 0, 255, gamma] {
        data.extend_from_slice(&v.to_be_bytes());
    }
    // The remaining 28 per-channel records the decoder does not read.
    data.extend(std::iter::repeat_n(7u8, 280));
    AdjustmentData {
        key: *b"levl",
        data,
    }
}

#[test]
fn a_patch_keeps_what_the_model_does_not_carry() {
    // brit's mean (offset 4) and Lab flag (offset 6) are not modelled.
    let mut brit = encode_brightness_contrast(20, 0);
    brit.data[4..7].copy_from_slice(&[0, 9, 1]);
    let edited = set_adjustment_param(&brit, "brightness", 55.0).unwrap();
    assert_eq!(value(&edited, "brightness"), 55.0);
    assert_eq!(&edited.data[4..7], &[0, 9, 1]);

    // hue2's Colorize byte and a per-range record survive a Master edit.
    let mut hue = encode_hue_saturation(30, 0, 0);
    hue.data[2] = 1;
    hue.data[40] = 0x2a;
    let edited = set_adjustment_param(&hue, "hue", -40.0).unwrap();
    assert_eq!(value(&edited, "hue"), -40.0);
    assert_eq!((edited.data[2], edited.data[40]), (1, 0x2a));
    assert_eq!(edited.data.len(), hue.data.len());

    let edited = set_adjustment_param(&levels(0, 255, 100), "gamma", 1.5).unwrap();
    assert_eq!(value(&edited, "gamma"), 1.5);
    assert!(edited.data[12..].iter().all(|&b| b == 7));
}

#[test]
fn values_clamp_and_invalid_results_are_refused() {
    let brit = encode_brightness_contrast(0, 0);
    let clamped = set_adjustment_param(&brit, "contrast", 500.0).unwrap();
    assert_eq!(value(&clamped, "contrast"), 100.0);
    assert!(set_adjustment_param(&brit, "nonsense", 1.0).is_none());
    assert!(set_adjustment_param(&encode_invert(), "anything", 1.0).is_none());

    // Levels needs input black below input white.
    let narrow = set_adjustment_param(&levels(0, 255, 100), "inputWhite", 10.0).unwrap();
    assert!(set_adjustment_param(&narrow, "inputBlack", 20.0).is_none());
    let posterize = set_adjustment_param(&encode_posterize(4), "levels", 1.0).unwrap();
    assert_eq!(value(&posterize, "levels"), 2.0);
}

#[test]
fn descriptor_items_are_edited_and_added_in_place() {
    let obj = DescValue::Object {
        name: String::new(),
        class_id: b"null".to_vec(),
        items: vec![
            (b"vibrance".to_vec(), DescValue::Long(10)),
            (b"Xtra".to_vec(), DescValue::Bool(true)),
        ],
    };
    let vib = AdjustmentData {
        key: *b"vibA",
        data: pictura_codec::write_descriptor(&obj),
    };
    assert_eq!(
        value(&vib, "saturation"),
        0.0,
        "missing reads as the default"
    );
    let edited = set_adjustment_param(&vib, "saturation", 25.0).unwrap();
    assert_eq!(value(&edited, "saturation"), 25.0);
    assert_eq!(value(&edited, "vibrance"), 10.0);
    let back = pictura_codec::read_descriptor(&edited.data).unwrap();
    assert_eq!(desc_item(&back, b"Xtra"), Some(&DescValue::Bool(true)));
}

#[test]
fn grouped_pages_address_each_group() {
    let balance = encode_color_balance([0.0; 3], [0.0; 3], [0.0; 3], true);
    let editor = adjustment_editor(&balance).unwrap();
    assert_eq!(editor.groups, ["Shadows", "Midtones", "Highlights"]);
    assert_eq!(editor.params.len(), 10);
    let edited = set_adjustment_param(&balance, "midtones.cyanRed", 40.0).unwrap();
    match decode_adjustment(&edited) {
        Some(Adjustment::ColorBalance(p)) => assert_eq!(p.midtones, [40.0, 0.0, 0.0]),
        other => panic!("{other:?}"),
    }

    let selective = encode_selective_color(
        SelectiveColorMethod::Relative,
        &[SelectiveRange::default(); 9],
    );
    let editor = adjustment_editor(&selective).unwrap();
    assert_eq!(editor.groups.len(), 9);
    let edited = set_adjustment_param(&selective, "blues.black", -30.0).unwrap();
    let edited = set_adjustment_param(&edited, "method", 1.0).unwrap();
    match decode_adjustment(&edited) {
        Some(Adjustment::SelectiveColor(p)) => {
            assert_eq!(p.ranges[4].k, -30);
            assert_eq!(p.method, SelectiveColorMethod::Absolute);
        }
        other => panic!("{other:?}"),
    }

    let mixer = encode_channel_mixer(
        false,
        [100.0, 0.0, 0.0],
        [0.0, 100.0, 0.0],
        [0.0, 0.0, 100.0],
        [0.0; 3],
    );
    let edited = set_adjustment_param(&mixer, "1.red", 30.0).unwrap();
    match decode_adjustment(&edited) {
        Some(Adjustment::ChannelMixer(p)) => assert_eq!(p.green, [30.0, 100.0, 0.0]),
        other => panic!("{other:?}"),
    }
}

#[test]
fn photo_filter_colour_is_editable_in_version_two_only() {
    let filter = encode_photo_filter([255, 180, 80], 25.0, true);
    assert_eq!(value(&filter, "color"), f64::from(0xff_b4_50u32));
    let edited = set_adjustment_param(&filter, "color", f64::from(0x10_20_30u32)).unwrap();
    match decode_adjustment(&edited) {
        Some(Adjustment::PhotoFilter(p)) => assert_eq!(p.color, [0x10, 0x20, 0x30]),
        other => panic!("{other:?}"),
    }
    let mut v3 = vec![0, 3];
    for v in [0u32, 0, 0, 40] {
        v3.extend_from_slice(&v.to_be_bytes());
    }
    v3.extend_from_slice(&[1, 0, 0]);
    let v3 = AdjustmentData {
        key: *b"phfl",
        data: v3,
    };
    let editor = adjustment_editor(&v3).unwrap();
    assert!(editor.params.iter().all(|p| p.key != "color"));
    assert_eq!(value(&v3, "density"), 40.0);
}

#[test]
fn curves_edit_per_channel() {
    let curves = encode_curves(&[(0, 0), (255, 255)], None, None, None);
    assert!(adjustment_editor(&curves).unwrap().curves);
    let red = [(0, 0), (128, 160), (255, 255)];
    let edited = set_curve_points(&curves, 1, &red).unwrap();
    assert_eq!(curve_points(&edited, 1).unwrap(), red);
    assert_eq!(curve_points(&edited, 2).unwrap(), [(0, 0), (255, 255)]);
    // Back to the identity, the channel curve is dropped.
    let flat = set_curve_points(&edited, 1, &[(0, 0), (255, 255)]).unwrap();
    assert!(matches!(decode_adjustment(&flat), Some(Adjustment::Curves(c)) if c.red.is_none()));
    assert!(set_curve_points(&curves, 0, &[(10, 0), (5, 255)]).is_none());
    assert!(set_curve_points(&curves, 4, &red).is_none());
}

#[test]
fn reset_restores_every_default() {
    let brit = encode_brightness_contrast(60, -20);
    let reset = reset_adjustment(&brit).unwrap();
    assert_eq!(
        (value(&reset, "brightness"), value(&reset, "contrast")),
        (0.0, 0.0)
    );
    let curves = encode_curves(&[(0, 30), (255, 200)], None, None, None);
    let reset = reset_adjustment(&curves).unwrap();
    assert_eq!(curve_points(&reset, 0).unwrap(), [(0, 0), (255, 255)]);
    let note = adjustment_editor(&encode_invert()).unwrap();
    assert!(note.params.is_empty() && note.note.is_some());
}
