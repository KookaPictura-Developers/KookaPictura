use super::*;
use pictura_codec::{read_psd, write_psd};
use pictura_core::{Channel, LayerBlock, SmartObject, SmartObjectKind};

fn section_has_uuid(bytes: &[u8], uuid: &str) -> bool {
    !uuid.is_empty() && bytes.windows(uuid.len()).any(|w| w == uuid.as_bytes())
}

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

#[test]
fn convert_authors_embedded_object_and_keeps_proxy() {
    let layer = solid(
        "Raster",
        full(4, 4),
        (10, 20, 30),
        255,
        BlendMode::Normal,
        255,
    );
    let mut d = doc(4, 4, vec![layer]);
    let before = d.clone();
    let before_comp = composite_rgba(&d);

    assert!(can_convert_to_smart_object(&d, "0"));
    assert!(convert_to_smart_object(&mut d, "0"));

    let so = d.layers[0].smart_object.as_ref().expect("smart object");
    assert_eq!(so.kind, SmartObjectKind::Embedded);
    assert!(so.payload.as_deref().is_some_and(|p| !p.is_empty()));
    assert_eq!(so.filename, "Raster.psd");
    assert_eq!(so.filetype, *b"8BPB");
    assert_eq!(so.creator, *b"8BIM");
    assert_eq!(d.layers[0].channels, before.layers[0].channels);
    assert_eq!(composite_rgba(&d), before_comp);

    let embedded = read_psd(so.payload.as_deref().unwrap()).expect("payload reads");
    assert_eq!(embedded.width, 4);
    assert_eq!(embedded.height, 4);
    assert_eq!(embedded.composite.channels, 3);
    for i in 0..16 {
        assert_eq!(embedded.composite.data[i], 10);
        assert_eq!(embedded.composite.data[16 + i], 20);
        assert_eq!(embedded.composite.data[32 + i], 30);
    }
}

#[test]
fn converted_layer_round_trips() {
    let layer = solid(
        "Raster",
        full(4, 4),
        (10, 20, 30),
        255,
        BlendMode::Normal,
        255,
    );
    let mut d = doc(4, 4, vec![layer]);
    assert!(convert_to_smart_object(&mut d, "0"));
    let payload = d.layers[0]
        .smart_object
        .as_ref()
        .unwrap()
        .payload
        .clone()
        .unwrap();

    let bytes = write_psd(&d).expect("host writes");
    let back = read_psd(&bytes).expect("host reads");
    let so = back.layers[0].smart_object.as_ref().expect("smart object");
    assert_eq!(so.kind, SmartObjectKind::Embedded);
    assert_eq!(so.payload.as_deref(), Some(payload.as_slice()));
    assert_eq!(back.layers[0].channels, d.layers[0].channels);
    assert_eq!(composite_rgba(&back), composite_rgba(&d));
}

#[test]
fn convert_refuses_ineligible_targets() {
    let cases: Vec<(&str, Document)> = vec![
        (
            "group",
            doc(4, 4, vec![group("G", BlendMode::Normal, 255, None, vec![])]),
        ),
        (
            "adjustment",
            doc(
                4,
                4,
                vec![adjustment_layer("Adj", *b"inv ", vec![0], 255, None)],
            ),
        ),
        ("background", {
            let mut d = doc(
                4,
                4,
                vec![solid(
                    "Background",
                    full(4, 4),
                    (0, 0, 0),
                    255,
                    BlendMode::Normal,
                    255,
                )],
            );
            d.layers[0].background = true;
            d
        }),
        (
            "zero-size",
            doc(
                4,
                4,
                vec![solid(
                    "Zero",
                    rect(0, 0, 0, 0),
                    (1, 2, 3),
                    255,
                    BlendMode::Normal,
                    255,
                )],
            ),
        ),
    ];
    for (name, mut d) in cases {
        let before = d.clone();
        assert!(!can_convert_to_smart_object(&d, "0"), "can: {name}");
        assert!(!convert_to_smart_object(&mut d, "0"), "convert: {name}");
        assert_eq!(d, before, "mutated: {name}");
    }

    let mut d = doc(
        4,
        4,
        vec![solid(
            "Raster",
            full(4, 4),
            (1, 2, 3),
            255,
            BlendMode::Normal,
            255,
        )],
    );
    let before = d.clone();
    assert!(!convert_to_smart_object(&mut d, "99"));
    assert!(!convert_to_smart_object(&mut d, "bad"));
    assert_eq!(d, before);

    assert!(convert_to_smart_object(&mut d, "0"));
    let converted = d.clone();
    assert!(!convert_to_smart_object(&mut d, "0"));
    assert_eq!(d, converted);
}

#[test]
fn sixteen_bit_document_refuses_conversion() {
    let mut d = Document::new(4, 4, ColorMode::Rgb, BitDepth::Sixteen);
    d.layers = vec![solid(
        "Raster",
        full(4, 4),
        (10, 20, 30),
        255,
        BlendMode::Normal,
        255,
    )];
    let before = d.clone();
    assert!(!can_convert_to_smart_object(&d, "0"));
    assert!(!convert_to_smart_object(&mut d, "0"));
    assert_eq!(d, before);
}

#[test]
fn grayscale_layer_converts_to_grayscale_source() {
    let mut d = Document::new(2, 2, ColorMode::Grayscale, BitDepth::Eight);
    d.layers = vec![Layer {
        name: "Gray".into(),
        rect: full(2, 2),
        channels: vec![
            Channel {
                id: 0,
                data: vec![128; 4],
            },
            Channel {
                id: -1,
                data: vec![255; 4],
            },
        ],
        ..Default::default()
    }];
    assert!(convert_to_smart_object(&mut d, "0"));
    let payload = d.layers[0]
        .smart_object
        .as_ref()
        .unwrap()
        .payload
        .clone()
        .unwrap();
    let embedded = read_psd(&payload).expect("payload reads");
    assert_eq!(embedded.mode, ColorMode::Grayscale);
    assert!(embedded.composite.data.iter().all(|&b| b == 128));
}

#[test]
fn rasterize_keeps_proxy_channels_and_drops_config() {
    let mut layer = solid(
        "Raster",
        full(4, 4),
        (10, 20, 30),
        255,
        BlendMode::Normal,
        255,
    );
    layer.smart_object = Some(embedded(payload_of([1, 2, 3])));
    layer.extra_blocks = vec![LayerBlock {
        key: *b"SoLd",
        data: vec![9, 9, 9, 9],
    }];
    let mut d = doc(4, 4, vec![layer]);
    let before = d.clone();
    let before_comp = composite_rgba(&d);

    assert!(can_rasterize_smart_object(&d, "0"));
    assert!(rasterize_smart_object(&mut d, "0"));

    assert_eq!(d.layers[0].channels, before.layers[0].channels);
    assert!(d.layers[0].smart_object.is_none());
    assert!(!d.layers[0].extra_blocks.iter().any(|b| &b.key == b"SoLd"));
    assert!(!can_rasterize_smart_object(&d, "0"));
    assert_eq!(composite_rgba(&d), before_comp);
}

#[test]
fn rasterize_materializes_source_into_channels() {
    let layer = smart_layer(
        "smart",
        full(4, 4),
        embedded(payload_of([255, 0, 0])),
        Vec::new(),
    );
    let mut d = doc(4, 4, vec![layer]);
    let before_comp = composite_rgba(&d);

    assert!(can_rasterize_smart_object(&d, "0"));
    assert!(rasterize_smart_object(&mut d, "0"));

    let channel = |id: i16| d.layers[0].channels.iter().find(|c| c.id == id).unwrap();
    assert_eq!(channel(0).data, vec![255u8; 16]);
    assert_eq!(channel(1).data, vec![0u8; 16]);
    assert_eq!(channel(2).data, vec![0u8; 16]);
    assert_eq!(channel(-1).data, vec![255u8; 16]);
    assert!(d.layers[0].smart_object.is_none());
    assert_eq!(composite_rgba(&d), before_comp);
}

#[test]
fn rasterize_refuses_ineligible_targets() {
    let mut plain = doc(
        4,
        4,
        vec![solid(
            "Raster",
            full(4, 4),
            (1, 2, 3),
            255,
            BlendMode::Normal,
            255,
        )],
    );
    let before = plain.clone();
    assert!(!can_rasterize_smart_object(&plain, "0"));
    assert!(!rasterize_smart_object(&mut plain, "0"));
    assert!(!rasterize_smart_object(&mut plain, "bad"));
    assert_eq!(plain, before);

    let mut grouped = doc(
        4,
        4,
        vec![{
            let mut g = group("G", BlendMode::Normal, 255, None, vec![]);
            g.smart_object = Some(embedded(payload_of([1, 2, 3])));
            g
        }],
    );
    let before = grouped.clone();
    assert!(!can_rasterize_smart_object(&grouped, "0"));
    assert!(!rasterize_smart_object(&mut grouped, "0"));
    assert_eq!(grouped, before);

    let mut adjustment = doc(
        4,
        4,
        vec![{
            let mut a = adjustment_layer("Adj", *b"inv ", vec![0], 255, None);
            a.smart_object = Some(embedded(payload_of([1, 2, 3])));
            a
        }],
    );
    let before = adjustment.clone();
    assert!(!can_rasterize_smart_object(&adjustment, "0"));
    assert!(!rasterize_smart_object(&mut adjustment, "0"));
    assert_eq!(adjustment, before);

    let undecodable = smart_layer("smart", full(4, 4), embedded(vec![0, 1, 2]), Vec::new());
    let mut d = doc(4, 4, vec![undecodable]);
    let before = d.clone();
    assert!(can_rasterize_smart_object(&d, "0"));
    assert!(!rasterize_smart_object(&mut d, "0"));
    assert_eq!(d, before);

    let empty = smart_layer("smart", full(4, 4), embedded(Vec::new()), Vec::new());
    let mut d = doc(4, 4, vec![empty]);
    let before = d.clone();
    assert!(!rasterize_smart_object(&mut d, "0"));
    assert_eq!(d, before);
}

#[test]
fn oversized_rect_source_is_clipped_to_canvas() {
    let layer = smart_layer(
        "smart",
        rect(0, 0, 100, 100),
        embedded(payload_of([7, 8, 9])),
        Vec::new(),
    );
    let out = composite_rgba(&doc(4, 4, vec![layer]));
    for y in 0..4 {
        for x in 0..4 {
            assert_close(&out, x, y, [7, 8, 9, 255]);
        }
    }

    // The output buffer is bounded by the requested region, not the layer rect.
    let so = embedded(payload_of([7, 8, 9]));
    let buf = render_smart_source(&so, rect(-50, -50, 100_000, 100_000), rect(0, 0, 4, 4))
        .expect("region renders");
    assert_eq!((buf.width, buf.height), (4, 4));
}

#[test]
fn rasterize_preserves_layout_fields() {
    let mut layer = smart_layer(
        "smart",
        rect(-1, -2, 4, 5),
        embedded(payload_of([255, 0, 0])),
        Vec::new(),
    );
    layer.blend = BlendMode::Multiply;
    layer.opacity = 200;
    layer.fill = 128;
    layer.mask = Some(LayerMask::default());
    let mut d = doc(8, 8, vec![layer]);
    let before = d.layers[0].clone();

    assert!(rasterize_smart_object(&mut d, "0"));

    let after = &d.layers[0];
    assert_eq!(after.name, before.name);
    assert_eq!(after.rect, before.rect);
    assert_eq!(after.blend, before.blend);
    assert_eq!(after.opacity, before.opacity);
    assert_eq!(after.fill, before.fill);
    assert_eq!(after.mask, before.mask);
    assert!(after.smart_object.is_none());
}

#[test]
fn rasterize_drops_preserved_config_and_linked_record_round_trip() {
    let layer = solid(
        "Raster",
        full(4, 4),
        (10, 20, 30),
        255,
        BlendMode::Normal,
        255,
    );
    let mut d = doc(4, 4, vec![layer]);
    assert!(convert_to_smart_object(&mut d, "0"));
    let saved = write_psd(&d).expect("first write");

    let mut back = read_psd(&saved).expect("first read");
    let uuid = back.layers[0]
        .smart_object
        .as_ref()
        .expect("resolved")
        .uuid
        .clone();
    assert_eq!(uuid.len(), 36);
    assert!(back.layers[0]
        .extra_blocks
        .iter()
        .any(|b| &b.key == b"SoLd"));
    assert!(section_has_uuid(&back.layer_section_extra, &uuid));

    assert!(rasterize_smart_object(&mut back, "0"));
    assert!(back.layers[0].smart_object.is_none());
    assert!(!back.layers[0]
        .extra_blocks
        .iter()
        .any(|b| matches!(&b.key, b"SoLd" | b"SoLE" | b"plLd" | b"PlLd")));
    assert!(!section_has_uuid(&back.layer_section_extra, &uuid));

    let resaved = write_psd(&back).expect("second write");
    let reread = read_psd(&resaved).expect("second read");
    assert!(reread.layers[0].smart_object.is_none());
    assert!(!section_has_uuid(&reread.layer_section_extra, &uuid));
}
