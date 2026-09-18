use super::*;

use crate::common::COMPRESSION_RAW;
use pictura_core::*;

fn header(version: u16, channels: u16, width: u32, height: u32, mode: u16) -> Vec<u8> {
    let mut v = Vec::new();
    v.extend_from_slice(b"8BPS");
    v.extend_from_slice(&version.to_be_bytes());
    v.extend_from_slice(&[0u8; 6]);
    v.extend_from_slice(&channels.to_be_bytes());
    v.extend_from_slice(&height.to_be_bytes());
    v.extend_from_slice(&width.to_be_bytes());
    v.extend_from_slice(&8u16.to_be_bytes());
    v.extend_from_slice(&mode.to_be_bytes());
    v
}

fn psd_sections() -> Vec<u8> {
    let mut v = Vec::new();
    v.extend_from_slice(&0u32.to_be_bytes());
    v.extend_from_slice(&0u32.to_be_bytes());
    v.extend_from_slice(&0u32.to_be_bytes());
    v
}

#[test]
fn parse_hand_constructed_raw_rgb() {
    let mut p = header(1, 3, 2, 2, 3);
    p.extend_from_slice(&psd_sections());
    p.extend_from_slice(&0u16.to_be_bytes());
    p.extend_from_slice(&[1, 2, 3, 4]); // R plane
    p.extend_from_slice(&[5, 6, 7, 8]); // G plane
    p.extend_from_slice(&[9, 10, 11, 12]); // B plane

    let doc = read_psd(&p).unwrap();
    assert_eq!(doc.width, 2);
    assert_eq!(doc.height, 2);
    assert_eq!(doc.mode, ColorMode::Rgb);
    assert_eq!(doc.depth, BitDepth::Eight);
    assert_eq!(doc.composite.channels, 3);
    assert!(doc.layers.is_empty());
    assert_eq!(
        doc.composite.data,
        vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]
    );
}

#[test]
fn round_trip_rgb_and_grayscale() {
    for (mode, channels) in [(ColorMode::Rgb, 3u8), (ColorMode::Grayscale, 1u8)] {
        let mut doc = Document::new(3, 2, mode, BitDepth::Eight);
        for (i, b) in doc.composite.data.iter_mut().enumerate() {
            *b = (i * 7 + 3) as u8;
        }
        assert_eq!(doc.composite.channels, channels);
        let bytes = write_psd(&doc).unwrap();
        assert_eq!(read_psd(&bytes).unwrap(), doc);
    }
}

#[test]
fn extra_selection_channel_round_trips() {
    let mut doc = Document::new(4, 3, ColorMode::Rgb, BitDepth::Eight);
    for (i, b) in doc.composite.data.iter_mut().enumerate() {
        *b = (i * 3) as u8;
    }
    doc.channels = vec![Channel {
        id: 0,
        data: (0..12).map(|i| 255 - (i * 5) as u8).collect(),
    }];

    let bytes = write_psd(&doc).unwrap();
    assert_eq!(u16::from_be_bytes(bytes[12..14].try_into().unwrap()), 4);

    let back = read_psd(&bytes).unwrap();
    assert_eq!(
        back.composite.channels, 3,
        "composite keeps only color planes"
    );
    assert_eq!(back.channels, doc.channels);
    assert_eq!(back, doc);
}

#[test]
fn multiple_extra_channels_round_trip() {
    let mut doc = Document::new(2, 2, ColorMode::Grayscale, BitDepth::Eight);
    doc.composite.data.copy_from_slice(&[1, 2, 3, 4]);
    doc.channels = vec![
        Channel {
            id: 0,
            data: vec![10, 20, 30, 40],
        },
        Channel {
            id: 1,
            data: vec![50, 60, 70, 80],
        },
    ];

    let bytes = write_psd(&doc).unwrap();
    assert_eq!(u16::from_be_bytes(bytes[12..14].try_into().unwrap()), 3);

    let back = read_psd(&bytes).unwrap();
    assert_eq!(back.composite.channels, 1);
    assert_eq!(back, doc);
}

#[test]
fn extra_channel_length_mismatch_is_rejected() {
    let mut doc = Document::new(2, 2, ColorMode::Rgb, BitDepth::Eight);
    doc.channels = vec![Channel {
        id: 0,
        data: vec![0; 3],
    }];
    assert!(matches!(write_psd(&doc), Err(PsdError::Invalid(_))));
}

#[test]
fn parse_rle_psd() {
    let mut p = header(1, 1, 4, 2, 1);
    p.extend_from_slice(&psd_sections());
    p.extend_from_slice(&1u16.to_be_bytes()); // RLE
    p.extend_from_slice(&5u16.to_be_bytes()); // row 0 packed length
    p.extend_from_slice(&2u16.to_be_bytes()); // row 1 packed length
    p.push(3);
    p.extend_from_slice(&[0, 1, 2, 3]); // literal run of 4
    p.push(253);
    p.push(5); // repeat 5 four times

    let doc = read_psd(&p).unwrap();
    assert_eq!(doc.composite.data, vec![0, 1, 2, 3, 5, 5, 5, 5]);
}

#[test]
fn parse_rle_psb_uses_four_byte_counts() {
    let mut p = header(2, 1, 4, 1, 1);
    p.extend_from_slice(&0u32.to_be_bytes()); // color mode
    p.extend_from_slice(&0u32.to_be_bytes()); // image resources
    p.extend_from_slice(&0u64.to_be_bytes()); // PSB layer/mask is 8 bytes
    p.extend_from_slice(&1u16.to_be_bytes()); // RLE
    p.extend_from_slice(&5u32.to_be_bytes()); // 4-byte scanline count
    p.push(3);
    p.extend_from_slice(&[9, 8, 7, 6]);

    let doc = read_psd(&p).unwrap();
    assert_eq!(doc.composite.data, vec![9, 8, 7, 6]);
}

#[test]
fn bad_signature_is_error_not_panic() {
    let mut p = header(1, 3, 1, 1, 3);
    p[0] = b'X';
    assert!(matches!(read_psd(&p), Err(PsdError::BadSignature(_))));
}

#[test]
fn truncated_is_error_not_panic() {
    let mut doc = Document::new(2, 2, ColorMode::Rgb, BitDepth::Eight);
    doc.composite.data = (0..12).collect();
    let bytes = write_psd(&doc).unwrap();
    assert!(matches!(
        read_psd(&bytes[..bytes.len() - 1]),
        Err(PsdError::Truncated)
    ));
    assert!(matches!(read_psd(b"8BPS"), Err(PsdError::Truncated)));
}

#[test]
fn lcg_round_trip_property() {
    let mut state: u64 = 0x1234_5678_9abc_def0;
    let mut next = move || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (state >> 33) as u32
    };

    for _ in 0..40 {
        let mode = if next() % 2 == 0 {
            ColorMode::Rgb
        } else {
            ColorMode::Grayscale
        };
        let channels = if mode == ColorMode::Rgb { 3 } else { 1 };
        let width = 1 + next() % 17;
        let height = 1 + next() % 17;

        let mut doc = Document::new(width, height, mode, BitDepth::Eight);
        for b in doc.composite.data.iter_mut() {
            *b = next() as u8;
        }
        let bytes = write_psd(&doc).unwrap();
        let back = read_psd(&bytes).unwrap();
        assert_eq!(back, doc, "mismatch for {width}x{height} ch={channels}");
    }
}

// -- M1-B: layers ------------------------------------------------------

fn rect(top: i32, left: i32, bottom: i32, right: i32) -> PsdRect {
    PsdRect {
        top,
        left,
        bottom,
        right,
    }
}

fn pixel(name: &str, r: PsdRect, color_channels: u8, blend: BlendMode, opacity: u8) -> Layer {
    let width = r.width().max(0) as usize;
    let height = r.height().max(0) as usize;
    let channels = (0..color_channels)
        .map(|c| Channel {
            id: c as i16,
            data: vec![c * 40 + 17; width * height],
        })
        .chain(std::iter::once(Channel {
            id: -1,
            data: vec![255; width * height],
        }))
        .collect();
    Layer {
        name: name.to_string(),
        rect: r,
        blend,
        opacity,
        fill: 255,
        lock: LockFlags::default(),
        color: ColorLabel::None,
        clipping: false,
        visible: true,
        mask: None,
        adjustment: None,
        channels,
        children: Vec::new(),
        is_group: false,
    }
}

#[test]
fn round_trip_layers_group_and_mask() {
    let mut doc = Document::new(16, 16, ColorMode::Rgb, BitDepth::Eight);
    for (i, b) in doc.composite.data.iter_mut().enumerate() {
        *b = (i % 251) as u8;
    }

    let red = pixel("Red", rect(0, 0, 4, 4), 3, BlendMode::Multiply, 200);
    let green = pixel("Green", rect(4, 4, 8, 8), 3, BlendMode::Screen, 255);
    let blue = pixel("Blue", rect(8, 8, 12, 12), 3, BlendMode::Normal, 128);
    let group = Layer {
        name: "Group A".to_string(),
        rect: rect(0, 0, 0, 0),
        blend: BlendMode::Normal,
        opacity: 255,
        fill: 255,
        lock: LockFlags::default(),
        color: ColorLabel::None,
        clipping: false,
        visible: true,
        mask: None,
        adjustment: None,
        channels: Vec::new(),
        children: vec![green, blue],
        is_group: true,
    };

    let mut masked = pixel("Masked", rect(2, 2, 6, 6), 3, BlendMode::Overlay, 255);
    masked.mask = Some(LayerMask {
        rect: rect(2, 2, 6, 6),
        default_color: 0,
        disabled: true,
        flags: 0x02,
        data: Some(vec![7u8; 16]),
    });

    doc.layers = vec![red, group, masked];

    let bytes = write_psd(&doc).unwrap();
    let back = read_psd(&bytes).unwrap();

    assert_eq!(back.layers.len(), 3);
    assert_eq!(back.layers[0].name, "Red");
    assert_eq!(back.layers[0].blend, BlendMode::Multiply);
    assert_eq!(back.layers[0].opacity, 200);
    assert_eq!(back.layers[0].rect, rect(0, 0, 4, 4));
    assert!(back.layers[1].is_group());
    assert_eq!(back.layers[1].name, "Group A");
    assert_eq!(back.layers[1].children.len(), 2);
    assert_eq!(back.layers[1].children[0].name, "Green");
    assert_eq!(back.layers[1].children[1].name, "Blue");
    assert_eq!(back.layers[1].children[0].rect, rect(4, 4, 8, 8));
    let mask = back.layers[2].mask.as_ref().expect("mask round-trips");
    assert_eq!(mask.rect, rect(2, 2, 6, 6));
    assert!(mask.disabled);
    assert_eq!(mask.data.as_deref(), Some(&[7u8; 16][..]));

    // The strongest check: the whole document is equal.
    assert_eq!(back, doc);
}

#[test]
fn pass_through_group_round_trips() {
    let mut doc = Document::new(8, 8, ColorMode::Rgb, BitDepth::Eight);
    let child = pixel("Child", rect(0, 0, 4, 4), 3, BlendMode::Multiply, 255);
    doc.layers = vec![Layer {
        name: "Pass Group".to_string(),
        rect: rect(0, 0, 0, 0),
        blend: BlendMode::PassThrough,
        opacity: 255,
        fill: 255,
        lock: LockFlags::default(),
        color: ColorLabel::None,
        clipping: false,
        visible: true,
        mask: None,
        adjustment: None,
        channels: Vec::new(),
        children: vec![child],
        is_group: true,
    }];

    let bytes = write_psd(&doc).unwrap();
    let back = read_psd(&bytes).unwrap();
    assert!(back.layers[0].is_group());
    assert_eq!(back.layers[0].blend, BlendMode::PassThrough);
    assert_eq!(back, doc);
}

#[test]
fn gray_layer_round_trips() {
    let mut doc = Document::new(8, 8, ColorMode::Grayscale, BitDepth::Eight);
    doc.layers = vec![pixel("Gray", rect(0, 0, 8, 8), 1, BlendMode::Normal, 255)];
    let bytes = write_psd(&doc).unwrap();
    assert_eq!(read_psd(&bytes).unwrap(), doc);
}

#[test]
fn adjustment_layers_round_trip_key_and_bytes() {
    let mut doc = Document::new(8, 8, ColorMode::Rgb, BitDepth::Eight);
    for (i, b) in doc.composite.data.iter_mut().enumerate() {
        *b = (i % 251) as u8;
    }
    let base = pixel("Base", rect(0, 0, 8, 8), 3, BlendMode::Normal, 255);

    // Odd (`curv`) and even payloads, empty (Invert) and descriptor-ish.
    let cases: &[([u8; 4], Vec<u8>)] = &[
        (*b"nvrt", Vec::new()),
        (*b"post", vec![0, 4, 0, 0]),
        (*b"thrs", vec![0, 128, 0, 0]),
        (*b"brit", vec![0, 10, 0, 20, 0, 0, 0, 0]),
        (*b"hue2", vec![0; 16]),
        (*b"curv", vec![1, 2, 3]),
    ];

    let mut layers = vec![base];
    for (i, (key, data)) in cases.iter().enumerate() {
        layers.push(Layer {
            name: format!("adj{i}"),
            rect: rect(0, 0, 0, 0),
            blend: BlendMode::Normal,
            opacity: 255,
            fill: 255,
            lock: LockFlags::default(),
            color: ColorLabel::None,
            clipping: false,
            visible: true,
            mask: None,
            adjustment: Some(AdjustmentData {
                key: *key,
                data: data.clone(),
            }),
            channels: Vec::new(),
            children: Vec::new(),
            is_group: false,
        });
    }
    doc.layers = layers;

    let back = read_psd(&write_psd(&doc).unwrap()).unwrap();
    assert_eq!(back, doc);
    for (i, (key, data)) in cases.iter().enumerate() {
        let adj = back.layers[i + 1]
            .adjustment
            .as_ref()
            .expect("adjustment round-trips");
        assert_eq!(&adj.key, key);
        assert_eq!(&adj.data, data);
    }
}

#[test]
fn malformed_layer_section_is_error_not_panic() {
    let mut doc = Document::new(8, 8, ColorMode::Rgb, BitDepth::Eight);
    doc.layers = vec![pixel("Only", rect(0, 0, 8, 8), 3, BlendMode::Normal, 255)];
    let good = write_psd(&doc).unwrap();

    // Truncated inside the layer/mask section.
    assert!(read_psd(&good[..60]).is_err());

    // Bogus layer count: claims 100 records but the section holds one.
    let mut bogus = good.clone();
    bogus[42..44].copy_from_slice(&100i16.to_be_bytes());
    assert!(read_psd(&bogus).is_err());

    // Bogus channel data length: first channel info length at offset 64
    // (header 26 + section lengths 12 + count 2 + rect 16 + nch 2 + id 2).
    let mut bad_len = good.clone();
    bad_len[64..68].copy_from_slice(&u32::MAX.to_be_bytes());
    assert!(read_psd(&bad_len).is_err());
}

#[test]
fn zip_layer_compression_is_unsupported() {
    let mut doc = Document::new(4, 4, ColorMode::Rgb, BitDepth::Eight);
    doc.layers = vec![pixel("Only", rect(0, 0, 4, 4), 3, BlendMode::Normal, 255)];
    let mut bytes = write_psd(&doc).unwrap();
    // Layer channel data follows the first (and only) record's extra data.
    let extra_len = u32::from_be_bytes(bytes[98..102].try_into().unwrap()) as usize;
    let channel_data = 102 + extra_len;
    bytes[channel_data + 1] = 2; // compression 2 = ZIP
    assert!(matches!(read_psd(&bytes), Err(PsdError::Unsupported(_))));
}

#[test]
fn layer_with_too_many_channels_is_rejected() {
    let mut doc = Document::new(1, 1, ColorMode::Rgb, BitDepth::Eight);
    let mut layer = pixel("TooMany", rect(0, 0, 1, 1), 3, BlendMode::Normal, 255);
    layer.channels = (0..=crate::common::MAX_CHANNELS as i16)
        .map(|id| Channel {
            id,
            data: vec![0; 1],
        })
        .collect();
    doc.layers = vec![layer];

    assert!(matches!(
        write_psd(&doc),
        Err(PsdError::Invalid(msg)) if msg.contains("layer channel count")
    ));
}

// -- M36: lspf / lclr / iOpa -------------------------------------------

/// The fixed default document captured before M36; its serialization must
/// stay byte-identical because every new tag is omitted at its default.
fn default_document() -> Document {
    let mut doc = Document::new(4, 4, ColorMode::Rgb, BitDepth::Eight);
    for (i, b) in doc.composite.data.iter_mut().enumerate() {
        *b = (i * 7 + 1) as u8;
    }
    let mut masked = pixel("Masked", rect(0, 0, 2, 2), 3, BlendMode::Normal, 200);
    masked.mask = Some(LayerMask {
        rect: rect(0, 0, 2, 2),
        default_color: 0,
        disabled: false,
        flags: 0,
        data: Some(vec![9u8; 4]),
    });
    let adj = Layer {
        name: "Invert".to_string(),
        rect: rect(0, 0, 0, 0),
        blend: BlendMode::Normal,
        opacity: 255,
        fill: 255,
        lock: LockFlags::default(),
        color: ColorLabel::None,
        clipping: false,
        visible: true,
        mask: None,
        adjustment: Some(AdjustmentData {
            key: *b"nvrt",
            data: Vec::new(),
        }),
        channels: Vec::new(),
        children: Vec::new(),
        is_group: false,
    };
    let group = Layer {
        name: "Group".to_string(),
        rect: rect(0, 0, 0, 0),
        blend: BlendMode::PassThrough,
        opacity: 255,
        fill: 255,
        lock: LockFlags::default(),
        color: ColorLabel::None,
        clipping: false,
        visible: true,
        mask: None,
        adjustment: None,
        channels: Vec::new(),
        children: vec![pixel(
            "Inner",
            rect(1, 1, 3, 3),
            3,
            BlendMode::Multiply,
            128,
        )],
        is_group: true,
    };
    doc.layers = vec![masked, adj, group];
    doc
}

#[test]
fn default_document_bytes_are_unchanged() {
    let bytes = write_psd(&default_document()).unwrap();
    let before = include_bytes!("../tests/fixtures/default_before.psd");
    assert_eq!(
        bytes.as_slice(),
        before.as_slice(),
        "default documents must serialize byte-identically to the original baseline"
    );
}

#[test]
fn layer_attributes_round_trip() {
    let mut doc = Document::new(4, 4, ColorMode::Rgb, BitDepth::Eight);
    for (i, b) in doc.composite.data.iter_mut().enumerate() {
        *b = (i * 5) as u8;
    }
    let mut layer = pixel("Attrs", rect(0, 0, 2, 2), 3, BlendMode::Normal, 255);
    layer.fill = 128;
    layer.color = ColorLabel::Violet;
    layer.lock = LockFlags::default()
        .with(LockFlags::TRANSPARENCY, true)
        .with(LockFlags::POSITION, true);
    doc.layers = vec![layer];

    let bytes = write_psd(&doc).unwrap();
    let back = read_psd(&bytes).unwrap();
    assert_eq!(back, doc, "whole document round-trips");
    assert_eq!(back.layers[0].fill, 128);
    assert_eq!(back.layers[0].color, ColorLabel::Violet);
    assert_eq!(back.layers[0].lock.bits(), 0x05);
}

#[test]
fn nesting_lock_bit_round_trips_through_lspf() {
    let mut doc = Document::new(4, 4, ColorMode::Rgb, BitDepth::Eight);
    let mut layer = pixel("Nested", rect(0, 0, 2, 2), 3, BlendMode::Normal, 255);
    layer.lock = LockFlags::default().with(LockFlags::NESTING, true);
    doc.layers = vec![layer];

    let bytes = write_psd(&doc).unwrap();
    let back = read_psd(&bytes).unwrap();
    assert!(back.layers[0].lock.contains(LockFlags::NESTING));
    assert_eq!(back.layers[0].lock.bits(), 0x08);
    assert_eq!(back, doc, "whole document round-trips");
}

/// Assemble a 1x1 RGB PSD with one channel-less layer whose record carries
/// `flags` and the given hand-built tagged blocks.
fn tagged_layer_psd(flags: u8, tags: &[(&[u8; 4], &[u8])]) -> Vec<u8> {
    let mut extra = Vec::new();
    extra.extend_from_slice(&0u32.to_be_bytes()); // mask data length
    extra.extend_from_slice(&0u32.to_be_bytes()); // blending ranges
    extra.extend_from_slice(&[1, b'L', 0, 0]); // pascal name "L"
    for (key, data) in tags {
        extra.extend_from_slice(b"8BIM");
        extra.extend_from_slice(*key);
        extra.extend_from_slice(&(data.len() as u32).to_be_bytes());
        extra.extend_from_slice(data);
        if data.len() % 2 == 1 {
            extra.push(0);
        }
    }
    if extra.len() % 2 == 1 {
        extra.push(0);
    }

    let mut rec = Vec::new();
    for v in [0i32, 0, 1, 1] {
        rec.extend_from_slice(&v.to_be_bytes());
    }
    rec.extend_from_slice(&0u16.to_be_bytes()); // no channels
    rec.extend_from_slice(b"8BIM");
    rec.extend_from_slice(b"norm");
    rec.push(255);
    rec.push(0);
    rec.push(flags);
    rec.push(0);
    rec.extend_from_slice(&(extra.len() as u32).to_be_bytes());
    rec.extend_from_slice(&extra);

    let mut info = Vec::new();
    info.extend_from_slice(&1i16.to_be_bytes());
    info.extend_from_slice(&rec);
    while info.len() % 4 != 0 {
        info.push(0);
    }

    let mut out = header(1, 3, 1, 1, 3);
    out.extend_from_slice(&0u32.to_be_bytes()); // color mode data
    out.extend_from_slice(&0u32.to_be_bytes()); // image resources
    let section_len = 4 + info.len() + 4;
    out.extend_from_slice(&(section_len as u32).to_be_bytes());
    out.extend_from_slice(&(info.len() as u32).to_be_bytes());
    out.extend_from_slice(&info);
    out.extend_from_slice(&0u32.to_be_bytes()); // global layer mask
    out.extend_from_slice(&COMPRESSION_RAW.to_be_bytes());
    out.extend_from_slice(&[1, 2, 3]); // 1x1 RGB composite
    out
}

#[test]
fn reads_hand_built_attribute_tags_and_folds_legacy_flag() {
    let lspf = 2u32.to_be_bytes(); // lock image pixels
    let mut lclr = [0u8; 8];
    lclr[1] = 2; // Orange
    let iopa = [128u8, 0, 0, 0]; // 4-byte payload: reader takes byte 0

    let psd = tagged_layer_psd(
        0x01, // legacy transparency-protected bit
        &[(b"lspf", &lspf), (b"lclr", &lclr), (b"iOpa", &iopa)],
    );
    let doc = read_psd(&psd).unwrap();
    let layer = &doc.layers[0];
    assert_eq!(layer.fill, 128);
    assert_eq!(layer.color, ColorLabel::Orange);
    assert!(layer.lock.contains(LockFlags::PIXELS));
    assert!(
        layer.lock.contains(LockFlags::TRANSPARENCY),
        "legacy bit folds in"
    );
    assert!(!layer.lock.contains(LockFlags::POSITION));
}
