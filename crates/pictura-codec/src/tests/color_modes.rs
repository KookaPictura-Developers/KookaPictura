//! Non-RGB color-mode read tests, split out of `tests.rs` to stay under
//! the file-size cap.

use super::*;

// -- color modes --------------------------------------------------------

/// A flat (no-layer) document with an explicit `color_mode_data` section and a
/// raw-compressed composite of `planes`.
pub(super) fn flat_psd_with_data(
    depth: u16,
    mode: u16,
    channels: u16,
    width: u32,
    height: u32,
    color_mode_data: &[u8],
    planes: &[&[u8]],
) -> Vec<u8> {
    let mut p = header_depth(1, channels, width, height, depth, mode);
    p.extend_from_slice(&(color_mode_data.len() as u32).to_be_bytes());
    p.extend_from_slice(color_mode_data);
    p.extend_from_slice(&0u32.to_be_bytes()); // image resources
    p.extend_from_slice(&0u32.to_be_bytes()); // layer/mask section
    p.extend_from_slice(&COMPRESSION_RAW.to_be_bytes());
    for plane in planes {
        p.extend_from_slice(plane);
    }
    p
}

pub(super) fn flat_psd(
    depth: u16,
    mode: u16,
    channels: u16,
    width: u32,
    height: u32,
    planes: &[&[u8]],
) -> Vec<u8> {
    flat_psd_with_data(depth, mode, channels, width, height, &[], planes)
}

/// A 1x1 document with one raw-compressed layer carrying `(id, plane)` channels.
fn layered_psd(
    mode: u16,
    header_channels: u16,
    composite: &[&[u8]],
    layer_channels: &[(i16, &[u8])],
) -> Vec<u8> {
    layered_psd_depth(8, mode, header_channels, composite, layer_channels)
}

/// As [`layered_psd`] at an explicit bit depth (1 for a Bitmap layer channel).
pub(super) fn layered_psd_depth(
    depth: u16,
    mode: u16,
    header_channels: u16,
    composite: &[&[u8]],
    layer_channels: &[(i16, &[u8])],
) -> Vec<u8> {
    let mut rec = Vec::new();
    for v in [0i32, 0, 1, 1] {
        rec.extend_from_slice(&v.to_be_bytes());
    }
    rec.extend_from_slice(&(layer_channels.len() as u16).to_be_bytes());
    for (id, data) in layer_channels {
        rec.extend_from_slice(&id.to_be_bytes());
        rec.extend_from_slice(&((2 + data.len()) as u32).to_be_bytes());
    }
    rec.extend_from_slice(b"8BIM");
    rec.extend_from_slice(b"norm");
    rec.push(255);
    rec.push(0);
    rec.push(0);
    rec.push(0);
    let mut extra = Vec::new();
    extra.extend_from_slice(&0u32.to_be_bytes()); // no mask
    extra.extend_from_slice(&0u32.to_be_bytes()); // blending ranges
    extra.extend_from_slice(&[1, b'L', 0, 0]); // pascal name "L"
    rec.extend_from_slice(&(extra.len() as u32).to_be_bytes());
    rec.extend_from_slice(&extra);

    let mut info = Vec::new();
    info.extend_from_slice(&1i16.to_be_bytes());
    info.extend_from_slice(&rec);
    for (_, data) in layer_channels {
        info.extend_from_slice(&COMPRESSION_RAW.to_be_bytes());
        info.extend_from_slice(data);
    }
    while info.len() % 4 != 0 {
        info.push(0);
    }

    let mut out = header_depth(1, header_channels, 1, 1, depth, mode);
    out.extend_from_slice(&0u32.to_be_bytes()); // color mode data
    out.extend_from_slice(&0u32.to_be_bytes()); // image resources
    let section_len = 4 + info.len() + 4;
    out.extend_from_slice(&(section_len as u32).to_be_bytes());
    out.extend_from_slice(&(info.len() as u32).to_be_bytes());
    out.extend_from_slice(&info);
    out.extend_from_slice(&0u32.to_be_bytes()); // global layer mask
    out.extend_from_slice(&COMPRESSION_RAW.to_be_bytes());
    for plane in composite {
        out.extend_from_slice(plane);
    }
    out
}

#[test]
fn bitmap_depth1_raw_expands_black_white() {
    let p = flat_psd(1, 0, 1, 8, 2, &[&[0xAA, 0xF0]]);
    let doc = read_psd(&p).unwrap();
    assert_eq!(doc.mode, ColorMode::Rgb);
    assert_eq!(doc.depth, BitDepth::Eight);
    assert_eq!(doc.source_mode, Some(ColorMode::Bitmap));
    assert_eq!(doc.composite.channels, 3);
    assert_eq!(&doc.composite.data[0..8], &[0, 255, 0, 255, 0, 255, 0, 255]);
    assert_eq!(
        &doc.composite.data[8..16],
        &[0, 0, 0, 0, 255, 255, 255, 255]
    );
    assert_eq!(&doc.composite.data[16..32], &doc.composite.data[0..16]);
}

#[test]
fn bitmap_depth1_non_multiple_of_eight_pads_rows() {
    let p = flat_psd(1, 0, 1, 10, 1, &[&[0b1010_0000, 0b1100_0000]]);
    let doc = read_psd(&p).unwrap();
    assert_eq!(
        &doc.composite.data[0..10],
        &[0, 255, 0, 255, 255, 255, 255, 255, 0, 0]
    );
}

#[test]
fn bitmap_depth1_rle_decodes_padded_rows() {
    let mut p = header_depth(1, 1, 10, 2, 1, 0);
    p.extend_from_slice(&0u32.to_be_bytes());
    p.extend_from_slice(&0u32.to_be_bytes());
    p.extend_from_slice(&0u32.to_be_bytes());
    p.extend_from_slice(&COMPRESSION_RLE.to_be_bytes());
    p.extend_from_slice(&3u16.to_be_bytes()); // row 0 packed length
    p.extend_from_slice(&3u16.to_be_bytes()); // row 1 packed length
    p.extend_from_slice(&[1, 0xAA, 0xC0]); // literal 2 bytes
    p.extend_from_slice(&[1, 0x00, 0xFF]); // literal 2 bytes
    let doc = read_psd(&p).unwrap();
    let r = &doc.composite.data[0..20];
    assert_eq!(
        r,
        &[0, 255, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0]
    );
    assert_eq!(&doc.composite.data[20..40], r, "G mirrors R");
    assert_eq!(&doc.composite.data[40..60], r, "B mirrors R");
}

#[test]
fn indexed_depth8_maps_palette_indices() {
    let mut palette = vec![0u8; 768];
    palette[1] = 10;
    palette[256 + 1] = 20;
    palette[512 + 1] = 30;
    palette[2] = 200;
    palette[256 + 2] = 100;
    palette[512 + 2] = 50;
    let p = flat_psd_with_data(8, 2, 1, 3, 1, &palette, &[&[0, 1, 2]]);
    let doc = read_psd(&p).unwrap();
    assert_eq!(doc.mode, ColorMode::Rgb);
    assert_eq!(doc.source_mode, Some(ColorMode::Indexed));
    assert!(doc.color_mode_data.is_empty());
    assert_eq!(&doc.composite.data[0..3], &[0, 10, 200]);
    assert_eq!(&doc.composite.data[3..6], &[0, 20, 100]);
    assert_eq!(&doc.composite.data[6..9], &[0, 30, 50]);
}

#[test]
fn cmyk_depth8_uses_profile_free_formula() {
    let p = flat_psd(
        8,
        4,
        4,
        3,
        1,
        &[&[128, 0, 255], &[64, 0, 255], &[32, 0, 255], &[200, 0, 255]],
    );
    let doc = read_psd(&p).unwrap();
    assert_eq!(doc.mode, ColorMode::Rgb);
    assert_eq!(doc.source_mode, Some(ColorMode::Cmyk));
    assert_eq!(
        doc.composite.data,
        vec![100, 0, 255, 50, 0, 255, 25, 0, 255]
    );
}

#[test]
fn lab_depth8_neutral_gray_and_white_within_two() {
    let p = flat_psd(8, 9, 3, 2, 1, &[&[200, 255], &[128, 128], &[128, 128]]);
    let doc = read_psd(&p).unwrap();
    assert_eq!(doc.mode, ColorMode::Rgb);
    assert_eq!(doc.source_mode, Some(ColorMode::Lab));
    for i in 0..2 {
        let expected = if i == 0 { 194 } else { 255 };
        for c in 0..3 {
            let value = doc.composite.data[c * 2 + i] as i32;
            assert!(
                (value - expected).abs() <= 2,
                "pixel {i} channel {c} = {value}"
            );
        }
    }
}

#[test]
fn indexed_palette_length_must_be_768() {
    for len in [767usize, 769] {
        let p = flat_psd_with_data(8, 2, 1, 1, 1, &vec![0u8; len], &[&[0]]);
        assert!(matches!(read_psd(&p), Err(PsdError::Invalid(_))));
    }
}

#[test]
fn bitmap_depth1_zip_is_unsupported() {
    let mut p = header_depth(1, 1, 8, 1, 1, 0);
    p.extend_from_slice(&0u32.to_be_bytes());
    p.extend_from_slice(&0u32.to_be_bytes());
    p.extend_from_slice(&0u32.to_be_bytes());
    p.extend_from_slice(&COMPRESSION_ZIP.to_be_bytes());
    assert!(matches!(read_psd(&p), Err(PsdError::Unsupported(_))));
}

#[test]
fn bitmap_depth1_truncated_rle_row_is_typed_error() {
    let mut p = header_depth(1, 1, 8, 1, 1, 0);
    p.extend_from_slice(&0u32.to_be_bytes());
    p.extend_from_slice(&0u32.to_be_bytes());
    p.extend_from_slice(&0u32.to_be_bytes());
    p.extend_from_slice(&COMPRESSION_RLE.to_be_bytes());
    p.extend_from_slice(&5u16.to_be_bytes()); // claims 5 packed bytes
    p.extend_from_slice(&[3, 1, 2, 3, 4]); // literal 4 into a 1-byte row
    assert!(matches!(read_psd(&p), Err(PsdError::Invalid(_))));
}

#[test]
fn cmyk_layer_converts_to_rgb_and_keeps_transparency() {
    let p = layered_psd(
        4,
        4,
        &[&[128], &[64], &[32], &[200]],
        &[
            (0, &[128]),
            (1, &[64]),
            (2, &[32]),
            (3, &[200]),
            (-1, &[200]),
        ],
    );
    let doc = read_psd(&p).unwrap();
    let layer = &doc.layers[0];
    assert_eq!(
        layer.channels.iter().map(|c| c.id).collect::<Vec<_>>(),
        vec![0, 1, 2, -1]
    );
    assert_eq!(layer.channels[0].data, vec![100]);
    assert_eq!(layer.channels[1].data, vec![50]);
    assert_eq!(layer.channels[2].data, vec![25]);
    assert_eq!(layer.channels[3].data, vec![200]);
}

#[test]
fn cmyk_layer_with_wrong_channel_count_is_left_unchanged() {
    let p = layered_psd(
        4,
        4,
        &[&[128], &[64], &[32], &[200]],
        &[(0, &[10]), (1, &[20]), (2, &[30])],
    );
    let doc = read_psd(&p).unwrap();
    let layer = &doc.layers[0];
    assert_eq!(
        layer.channels.iter().map(|c| c.id).collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    assert_eq!(layer.channels[0].data, vec![10]);
}

#[test]
fn cmyk_layer_leaves_alpha_mask_and_spot_untouched() {
    let p = layered_psd(
        4,
        4,
        &[&[128], &[64], &[32], &[200]],
        &[
            (0, &[128]),
            (1, &[64]),
            (2, &[32]),
            (3, &[200]),
            (-1, &[111]),
            (-2, &[222]),
            (-3, &[123]),
        ],
    );
    let doc = read_psd(&p).unwrap();
    let layer = &doc.layers[0];
    // The four CMYK color planes become RGB; every non-color channel is intact.
    assert_eq!(
        layer.channels.iter().map(|c| c.id).collect::<Vec<_>>(),
        vec![0, 1, 2, -1]
    );
    assert_eq!(layer.channels[0].data, vec![100]);
    assert_eq!(layer.channels[1].data, vec![50]);
    assert_eq!(layer.channels[2].data, vec![25]);
    assert_eq!(layer.channels[3].data, vec![111], "transparency untouched");
    assert_eq!(
        layer.mask.as_ref().and_then(|m| m.data.as_deref()),
        Some(&[222u8][..]),
        "mask channel untouched"
    );
    assert_eq!(
        layer.raw_channels.iter().map(|c| c.id).collect::<Vec<_>>(),
        vec![-3]
    );
    assert_eq!(
        layer.raw_channels[0].data,
        vec![0, 0, 123],
        "spot untouched"
    );
}

#[test]
fn bitmap_depth1_layer_color_channel_expands() {
    // A 1-bit layer color channel is bit-packed (`0x80` is a byte-aligned pixel):
    // a set bit is black, a clear bit is white, and it becomes three RGB planes.
    for (packed, expected) in [(0x80u8, 0u8), (0x00, 255u8)] {
        let p = layered_psd_depth(1, 0, 1, &[&[packed]], &[(0, &[packed])]);
        let doc = read_psd(&p).unwrap();
        assert_eq!(doc.source_mode, Some(ColorMode::Bitmap));
        let layer = &doc.layers[0];
        assert_eq!(
            layer.channels.iter().map(|c| c.id).collect::<Vec<_>>(),
            vec![0, 1, 2],
            "bitmap layer becomes RGB"
        );
        for channel in &layer.channels {
            assert_eq!(channel.data, vec![expected], "packed {packed:#04x}");
        }
    }
}

#[test]
fn unsupported_depths_and_color_modes_are_rejected() {
    // An invalid depth and Multichannel (7) / Duotone (8) are typed Unsupported;
    // depth 16/32 for Bitmap (0) or Indexed (2) stays rejected too.
    for (depth, mode) in [(4u16, 3u16), (8, 7), (8, 8), (16, 0), (32, 2)] {
        let p = header_depth(1, 3, 1, 1, depth, mode);
        assert!(
            matches!(read_psd(&p), Err(PsdError::Unsupported(_))),
            "depth {depth} mode {mode} should be unsupported"
        );
    }
}

#[test]
fn native_and_constructed_documents_have_no_source_mode() {
    for (mode, code) in [(ColorMode::Rgb, 3u16), (ColorMode::Grayscale, 1u16)] {
        let channels = mode.color_channels() as u16;
        let plane = vec![7u8; channels as usize];
        let p = flat_psd(8, code, channels, 1, 1, &[&plane]);
        assert_eq!(read_psd(&p).unwrap().source_mode, None);
    }
    assert_eq!(
        Document::new(1, 1, ColorMode::Rgb, BitDepth::Eight).source_mode,
        None
    );
    assert_eq!(Document::from_rgba("x", 1, 1, &[0u8; 4]).source_mode, None);
}

#[test]
fn lab_document_writes_back_as_lab() {
    // An unedited flat Lab composite is retained on read and re-emitted
    // byte-identically on write, so re-reading reproduces the working RGB.
    let source_lab = vec![225, 200, 82, 110, 114, 100];
    let p = flat_psd(8, 9, 3, 2, 1, &[&[225, 200], &[82, 110], &[114, 100]]);
    let doc = read_psd(&p).unwrap();
    assert_eq!(doc.mode, ColorMode::Rgb);
    assert_eq!(doc.source_mode, Some(ColorMode::Lab));
    // The store is 8-bit Lab, deliberately not a native-depth store.
    assert!(doc.source_planes.is_some());
    assert!(!doc.retains_source_depth());
    assert_eq!(
        doc.source_planes
            .as_ref()
            .expect("Lab planes retained")
            .data,
        source_lab
    );

    let out = write_psd(&doc).unwrap();
    assert_eq!(
        u16::from_be_bytes(out[12..14].try_into().unwrap()),
        3,
        "channels stay three"
    );
    assert_eq!(
        u16::from_be_bytes(out[22..24].try_into().unwrap()),
        8,
        "output depth stays 8"
    );
    assert_eq!(
        u16::from_be_bytes(out[24..26].try_into().unwrap()),
        9,
        "output header color mode is Lab"
    );

    let back = read_psd(&out).unwrap();
    assert_eq!(back.mode, ColorMode::Rgb);
    assert_eq!(back.source_mode, Some(ColorMode::Lab));
    assert_eq!(
        back.source_planes
            .as_ref()
            .expect("re-read Lab planes")
            .data,
        source_lab,
        "the written Lab planes equal the source exactly"
    );
    assert_eq!(
        back.composite, doc.composite,
        "re-reading the retained planes reproduces the working RGB exactly"
    );
}

#[test]
fn lab_layer_color_channels_write_back_as_lab() {
    let p = layered_psd(
        9,
        3,
        &[&[225], &[82], &[114]],
        &[(0, &[225]), (1, &[82]), (2, &[114]), (-1, &[200])],
    );
    let doc = read_psd(&p).unwrap();
    assert_eq!(
        doc.layers[0]
            .channels
            .iter()
            .map(|c| c.id)
            .collect::<Vec<_>>(),
        vec![0, 1, 2, -1],
        "the Lab layer becomes three RGB channels plus transparency"
    );
    let retained = doc.layers[0]
        .source_channels
        .as_ref()
        .expect("Lab layer planes retained");
    assert_eq!(retained.depth, BitDepth::Eight);
    assert_eq!(
        retained.planes,
        vec![(0, vec![225]), (1, vec![82]), (2, vec![114])]
    );

    let out = write_psd(&doc).unwrap();
    assert_eq!(
        u16::from_be_bytes(out[24..26].try_into().unwrap()),
        9,
        "output header color mode is Lab"
    );

    let back = read_psd(&out).unwrap();
    let layer = &back.layers[0];
    assert_eq!(
        layer.channels.iter().map(|c| c.id).collect::<Vec<_>>(),
        vec![0, 1, 2, -1]
    );
    // The layer's Lab planes were re-emitted exactly, so its color channels
    // round-trip to the same RGB bytes.
    for c in 0..3 {
        assert_eq!(
            layer.channels[c].data, doc.layers[0].channels[c].data,
            "layer color channel {c} is exact"
        );
    }
    assert_eq!(layer.channels[3].data, vec![200], "transparency untouched");
}

#[test]
fn edited_lab_plane_uses_the_approximate_inverse() {
    // An edit replaces the retained plane, so the writer re-encodes with
    // `rgb_to_lab`. ponytail: 8-bit Lab cannot represent saturated colors
    // exactly, so the re-decoded RGB is approximate (the documented ceiling).
    let p = flat_psd(8, 9, 3, 1, 1, &[&[225], &[82], &[114]]);
    let mut doc = read_psd(&p).unwrap();
    let edited = [0u8, 255, 128];
    doc.composite.data[..3].copy_from_slice(&edited);

    let out = write_psd(&doc).unwrap();
    let back = read_psd(&out).unwrap();
    assert_eq!(
        back.source_planes.as_ref().unwrap().data,
        crate::color_mode::rgb_to_lab(&edited),
        "an edited plane is re-encoded with rgb_to_lab"
    );
    assert_ne!(
        back.composite.data[..3],
        edited[..],
        "the saturated Lab decode is approximate, not exact"
    );
}

/// A 1x1 grouped document: a bounding divider, one pixel layer, and its
/// open-folder group, in the writer's record order, each color plane
/// raw-compressed.
fn grouped_psd(
    mode: u16,
    header_channels: u16,
    composite: &[&[u8]],
    child: &[(i16, &[u8])],
) -> Vec<u8> {
    fn record(channels: &[(i16, &[u8])], section: Option<u32>) -> Vec<u8> {
        let mut rec = Vec::new();
        for v in [0i32, 0, 1, 1] {
            rec.extend_from_slice(&v.to_be_bytes());
        }
        rec.extend_from_slice(&(channels.len() as u16).to_be_bytes());
        for (id, data) in channels {
            rec.extend_from_slice(&id.to_be_bytes());
            rec.extend_from_slice(&((2 + data.len()) as u32).to_be_bytes());
        }
        rec.extend_from_slice(b"8BIM");
        rec.extend_from_slice(b"norm");
        rec.push(255);
        rec.push(0);
        rec.push(0);
        rec.push(0);
        let mut extra = Vec::new();
        extra.extend_from_slice(&0u32.to_be_bytes()); // no mask
        extra.extend_from_slice(&0u32.to_be_bytes()); // blending ranges
        extra.extend_from_slice(&[1, b'L', 0, 0]); // pascal name "L"
        if let Some(section) = section {
            let mut lsct = Vec::new();
            lsct.extend_from_slice(&section.to_be_bytes());
            lsct.extend_from_slice(b"8BIM");
            lsct.extend_from_slice(b"norm");
            crate::write::write_tag(&mut extra, b"lsct", &lsct, false);
        }
        rec.extend_from_slice(&(extra.len() as u32).to_be_bytes());
        rec.extend_from_slice(&extra);
        rec
    }

    let mut info = Vec::new();
    info.extend_from_slice(&3i16.to_be_bytes()); // divider + child + group
    info.extend_from_slice(&record(&[], Some(3))); // bounding divider
    info.extend_from_slice(&record(child, None)); // the nested pixel layer
    info.extend_from_slice(&record(&[], Some(1))); // open-folder group
    for (_, data) in child {
        info.extend_from_slice(&COMPRESSION_RAW.to_be_bytes());
        info.extend_from_slice(data);
    }
    while info.len() % 4 != 0 {
        info.push(0);
    }

    let mut out = header_depth(1, header_channels, 1, 1, 8, mode);
    out.extend_from_slice(&0u32.to_be_bytes()); // color mode data
    out.extend_from_slice(&0u32.to_be_bytes()); // image resources
    let section_len = 4 + info.len() + 4;
    out.extend_from_slice(&(section_len as u32).to_be_bytes());
    out.extend_from_slice(&(info.len() as u32).to_be_bytes());
    out.extend_from_slice(&info);
    out.extend_from_slice(&0u32.to_be_bytes()); // global layer mask
    out.extend_from_slice(&COMPRESSION_RAW.to_be_bytes());
    for plane in composite {
        out.extend_from_slice(plane);
    }
    out
}

#[test]
fn grouped_lab_layers_convert_on_read_and_round_trip() {
    // Regression: a pixel layer inside a group must have its Lab color channels
    // converted to RGB on read, not left as Lab bytes.
    let p = grouped_psd(
        9,
        3,
        &[&[225], &[82], &[114]],
        &[(0, &[225]), (1, &[82]), (2, &[114])],
    );
    let doc = read_psd(&p).unwrap();
    assert_eq!(doc.mode, ColorMode::Rgb);
    assert_eq!(doc.source_mode, Some(ColorMode::Lab));
    assert!(doc.layers[0].is_group());
    let child = &doc.layers[0].children[0];
    assert_eq!(
        child.channels.iter().map(|c| c.id).collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    for (c, expected) in [(0usize, 60u8), (1, 246), (2, 246)] {
        assert_eq!(child.channels[c].data, vec![expected], "nested channel {c}");
    }

    // The unedited nested plane survives a save exactly.
    let out = write_psd(&doc).unwrap();
    let back = read_psd(&out).unwrap();
    let back_child = &back.layers[0].children[0];
    for c in 0..3 {
        assert_eq!(
            back_child.channels[c].data, child.channels[c].data,
            "nested channel {c} after save"
        );
    }
}

#[test]
fn grouped_cmyk_layers_convert_on_read() {
    // The same recursion covers the pre-existing grouped-CMYK read bug.
    let p = grouped_psd(
        4,
        4,
        &[&[128], &[64], &[32], &[200]],
        &[(0, &[128]), (1, &[64]), (2, &[32]), (3, &[200])],
    );
    let doc = read_psd(&p).unwrap();
    assert_eq!(doc.mode, ColorMode::Rgb);
    let child = &doc.layers[0].children[0];
    assert_eq!(
        child.channels.iter().map(|c| c.id).collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    assert_eq!(child.channels[0].data, vec![100]);
    assert_eq!(child.channels[1].data, vec![50]);
    assert_eq!(child.channels[2].data, vec![25]);
}

#[test]
fn sixteen_bit_lab_saves_as_rgb() {
    // A 16-bit Lab source retains no planes and keeps writing the working RGB;
    // Lab write-back is scoped to an 8-bit source.
    let p = flat_psd(
        16,
        9,
        3,
        1,
        1,
        &[&[0x80, 0x00], &[0x80, 0x00], &[0x80, 0x00]],
    );
    let doc = read_psd(&p).unwrap();
    assert_eq!(doc.mode, ColorMode::Rgb);
    assert_eq!(doc.source_mode, Some(ColorMode::Lab));
    assert_eq!(doc.source_depth, Some(BitDepth::Sixteen));
    assert!(
        doc.source_planes.is_none(),
        "a 16-bit Lab read retains nothing"
    );

    let out = write_psd(&doc).unwrap();
    assert_eq!(
        u16::from_be_bytes(out[22..24].try_into().unwrap()),
        8,
        "output depth stays 8"
    );
    assert_eq!(
        u16::from_be_bytes(out[24..26].try_into().unwrap()),
        3,
        "16-bit Lab saves as RGB, not Lab"
    );
}

#[test]
fn lab_document_without_a_merged_composite_writes_lab() {
    // A layered Lab file with "Maximize Compatibility" off: the read yields no
    // merged composite, but the writer still selects Lab and re-emits the layer.
    let mut doc = Document::new(1, 1, ColorMode::Rgb, BitDepth::Eight);
    doc.source_mode = Some(ColorMode::Lab);
    doc.merged_composite_present = false;
    doc.layers = vec![Layer {
        rect: PsdRect {
            top: 0,
            left: 0,
            bottom: 1,
            right: 1,
        },
        channels: vec![
            Channel {
                id: 0,
                data: vec![60],
            },
            Channel {
                id: 1,
                data: vec![246],
            },
            Channel {
                id: 2,
                data: vec![246],
            },
        ],
        ..Default::default()
    }];

    let p = write_psd(&doc).unwrap();
    assert_eq!(
        u16::from_be_bytes(p[24..26].try_into().unwrap()),
        9,
        "no merged composite still writes Lab"
    );

    let back = read_psd(&p).unwrap();
    assert!(!back.merged_composite_present, "no merged composite");
    assert_eq!(back.mode, ColorMode::Rgb);
    assert_eq!(back.source_mode, Some(ColorMode::Lab));
    let layer = &back.layers[0];
    assert_eq!(layer.channels[0].data, vec![60]);
    assert_eq!(layer.channels[1].data, vec![246]);
    assert_eq!(layer.channels[2].data, vec![246]);

    let out = write_psd(&back).unwrap();
    assert_eq!(
        u16::from_be_bytes(out[24..26].try_into().unwrap()),
        9,
        "a re-save keeps Lab"
    );
}
