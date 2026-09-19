use super::*;
use pictura_codec::{read_psd, write_psd};
use pictura_core::{SmartObject, SmartObjectKind};

fn solid_doc(w: u32, h: u32, rgb: [u8; 3]) -> Document {
    let n = (w * h) as usize;
    let mut d = Document::new(w, h, ColorMode::Rgb, BitDepth::Eight);
    for i in 0..n {
        d.composite.data[i] = rgb[0];
        d.composite.data[n + i] = rgb[1];
        d.composite.data[2 * n + i] = rgb[2];
    }
    d
}

fn payload_of(rgb: [u8; 3]) -> Vec<u8> {
    write_psd(&solid_doc(2, 2, rgb)).expect("payload writes")
}

fn payload_2x2(colors: [[u8; 3]; 4]) -> Vec<u8> {
    let mut d = Document::new(2, 2, ColorMode::Rgb, BitDepth::Eight);
    for (i, c) in colors.iter().enumerate() {
        d.composite.data[i] = c[0];
        d.composite.data[4 + i] = c[1];
        d.composite.data[8 + i] = c[2];
    }
    write_psd(&d).expect("payload writes")
}

fn absent_composite_layers_payload(rgb: (u8, u8, u8)) -> Vec<u8> {
    let mut d = Document::new(4, 4, ColorMode::Rgb, BitDepth::Eight);
    d.layers = vec![solid("green", full(4, 4), rgb, 255, BlendMode::Normal, 255)];
    let bytes = write_psd(&d).expect("payload writes");
    let section_len = u32::from_be_bytes(bytes[34..38].try_into().unwrap()) as usize;
    bytes[..38 + section_len].to_vec()
}

fn embedded(payload: Vec<u8>) -> SmartObject {
    SmartObject {
        filename: "source.psd".into(),
        kind: SmartObjectKind::Embedded,
        payload: Some(payload),
        ..Default::default()
    }
}

fn smart_layer(name: &str, r: PsdRect, so: SmartObject, channels: Vec<Channel>) -> Layer {
    Layer {
        name: name.into(),
        rect: r,
        channels,
        smart_object: Some(so),
        ..Default::default()
    }
}

fn green_proxy() -> Vec<Channel> {
    vec![
        Channel {
            id: 0,
            data: vec![0; 16],
        },
        Channel {
            id: 1,
            data: vec![255; 16],
        },
        Channel {
            id: 2,
            data: vec![0; 16],
        },
        Channel {
            id: -1,
            data: vec![255; 16],
        },
    ]
}

fn assert_close(buf: &PixelBuffer, x: u32, y: u32, expected: [u8; 4]) {
    let got = px(buf, x, y);
    for i in 0..4 {
        assert!(
            (got[i] as i32 - expected[i] as i32).abs() <= 1,
            "at {x},{y}: got {got:?}, want {expected:?}"
        );
    }
}

fn assert_all_alpha_zero(buf: &PixelBuffer) {
    let plane = (buf.width * buf.height) as usize;
    for i in 0..plane {
        assert_eq!(buf.data[3 * plane + i], 0, "pixel {i} is not transparent");
    }
}

#[test]
fn embedded_source_renders_without_proxy() {
    let layer = smart_layer(
        "smart",
        full(4, 4),
        embedded(payload_of([255, 0, 0])),
        Vec::new(),
    );
    let out = composite_rgba(&doc(4, 4, vec![layer]));
    for y in 0..4 {
        for x in 0..4 {
            assert_close(&out, x, y, [255, 0, 0, 255]);
        }
    }
}

#[test]
fn proxy_wins_over_source() {
    let layer = smart_layer(
        "smart",
        full(4, 4),
        embedded(payload_of([255, 0, 0])),
        green_proxy(),
    );
    let out = composite_rgba(&doc(4, 4, vec![layer]));
    for y in 0..4 {
        for x in 0..4 {
            assert_close(&out, x, y, [0, 255, 0, 255]);
        }
    }
}

#[test]
fn undecodable_payload_is_a_noop() {
    let so = SmartObject {
        kind: SmartObjectKind::Embedded,
        payload: Some(vec![0, 1, 2]),
        ..Default::default()
    };
    let layer = smart_layer("smart", full(4, 4), so, Vec::new());
    let out = composite_rgba(&doc(4, 4, vec![layer]));
    assert_all_alpha_zero(&out);
}

#[test]
fn empty_payload_is_a_noop() {
    let so = SmartObject {
        kind: SmartObjectKind::Embedded,
        payload: Some(Vec::new()),
        ..Default::default()
    };
    let layer = smart_layer("smart", full(4, 4), so, Vec::new());
    let out = composite_rgba(&doc(4, 4, vec![layer]));
    assert_all_alpha_zero(&out);
}

#[test]
fn absent_composite_falls_back_to_layers() {
    let layer = smart_layer(
        "smart",
        full(4, 4),
        embedded(absent_composite_layers_payload((0, 255, 0))),
        Vec::new(),
    );
    let backdrop = solid("blue", full(4, 4), (0, 0, 255), 255, BlendMode::Normal, 255);
    let out = composite_rgba(&doc(4, 4, vec![backdrop, layer]));
    for y in 0..4 {
        for x in 0..4 {
            assert_close(&out, x, y, [0, 255, 0, 255]);
        }
    }
}

#[test]
fn channel_less_non_smart_layer_still_paints_black() {
    let layer = Layer {
        name: "bare".into(),
        rect: full(4, 4),
        channels: Vec::new(),
        ..Default::default()
    };
    let out = composite_rgba(&doc(4, 4, vec![layer]));
    for y in 0..4 {
        for x in 0..4 {
            assert_close(&out, x, y, [0, 0, 0, 255]);
        }
    }
}

#[test]
fn external_and_unresolved_do_not_render() {
    for kind in [
        SmartObjectKind::External,
        SmartObjectKind::Alias,
        SmartObjectKind::Unresolved,
    ] {
        let so = SmartObject {
            kind,
            payload: Some(payload_of([255, 0, 0])),
            ..Default::default()
        };
        let layer = smart_layer("smart", full(4, 4), so, Vec::new());
        let out = composite_rgba(&doc(4, 4, vec![layer]));
        assert_all_alpha_zero(&out);
    }
}

#[test]
fn source_scales_into_layer_rect() {
    let colors = [[255, 0, 0], [0, 255, 0], [0, 0, 255], [255, 255, 255]];
    let layer = smart_layer(
        "smart",
        full(4, 4),
        embedded(payload_2x2(colors)),
        Vec::new(),
    );
    let out = composite_rgba(&doc(4, 4, vec![layer]));
    let expected = [
        (0, 0, [255, 0, 0, 255]),
        (2, 0, [0, 255, 0, 255]),
        (0, 2, [0, 0, 255, 255]),
        (2, 2, [255, 255, 255, 255]),
    ];
    for (x, y, want) in expected {
        assert_close(&out, x, y, want);
    }
}

#[test]
fn author_and_render_round_trip() {
    let layer = smart_layer(
        "smart",
        full(4, 4),
        embedded(payload_of([255, 0, 0])),
        Vec::new(),
    );
    let bytes = write_psd(&doc(4, 4, vec![layer])).expect("host writes");
    let back = read_psd(&bytes).expect("host reads");
    let out = composite_rgba(&back);
    for y in 0..4 {
        for x in 0..4 {
            assert_close(&out, x, y, [255, 0, 0, 255]);
        }
    }
}
