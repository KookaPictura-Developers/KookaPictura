//! Depth-16/32 read tests: big-endian sample layouts, their 8-bit narrowing,
//! the depth-aware row stride and ZIP-with-prediction, and layer narrowing.

use super::color_modes::{flat_psd, layered_psd_depth};
use super::*;
use crate::depth::{predict16, predict32};

fn be16(values: &[u16]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_be_bytes()).collect()
}

fn be32(values: &[f32]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_be_bytes()).collect()
}

#[test]
fn depth16_raw_composite_narrows_high_byte() {
    let values = [0u16, 1, 255, 256, 257, 32768, 65534, 65535];
    let plane = be16(&values);
    let p = flat_psd(16, 3, 3, 8, 1, &[&plane, &plane, &plane]);
    let doc = read_psd(&p).unwrap();
    assert_eq!(doc.mode, ColorMode::Rgb);
    assert_eq!(doc.depth, BitDepth::Eight);
    assert_eq!(doc.source_depth, Some(BitDepth::Sixteen));
    let expected = [0u8, 0, 0, 1, 1, 128, 255, 255];
    assert_eq!(&doc.composite.data[0..8], &expected, "R narrows v >> 8");
    assert_eq!(&doc.composite.data[8..16], &expected, "G");
    assert_eq!(&doc.composite.data[16..24], &expected, "B");
}

#[test]
fn depth16_non_multiple_of_eight_width_narrows() {
    let plane = be16(&[0x0100, 0x0200, 0x0300]);
    let p = flat_psd(16, 3, 3, 3, 1, &[&plane, &plane, &plane]);
    let doc = read_psd(&p).unwrap();
    assert_eq!(&doc.composite.data[0..3], &[1, 2, 3]);
}

#[test]
fn depth16_rle_row_uses_two_byte_stride() {
    let plane = be16(&[0x0100, 0x0200, 0x0300, 0x0400]);
    let mut p = header_depth(1, 1, 4, 1, 16, 1);
    p.extend_from_slice(&psd_sections());
    p.extend_from_slice(&COMPRESSION_RLE.to_be_bytes());
    p.extend_from_slice(&9u16.to_be_bytes()); // packed row length
    p.push(7); // literal 8 bytes
    p.extend_from_slice(&plane);
    let doc = read_psd(&p).unwrap();
    assert_eq!(&doc.composite.data[0..4], &[1, 2, 3, 4]);
}

#[test]
fn depth32_raw_composite_narrows_clamped() {
    let values = [0.0f32, 0.001, 0.5, 1.0, 1.5, -0.5, 0.99609375, 255.0];
    let plane = be32(&values);
    let p = flat_psd(32, 3, 3, 8, 1, &[&plane, &plane, &plane]);
    let doc = read_psd(&p).unwrap();
    assert_eq!(doc.source_depth, Some(BitDepth::ThirtyTwo));
    let expected = [0u8, 0, 128, 255, 255, 0, 255, 255];
    assert_eq!(
        &doc.composite.data[0..8],
        &expected,
        "clamp(trunc(f * 256))"
    );
}

#[test]
fn depth16_zip_prediction_is_per_u16() {
    let values = [0u16, 256, 32768, 65535, 257, 512];
    let encoded = predict16(&values, 3);
    let mut p = header_depth(1, 1, 3, 2, 16, 1);
    p.extend_from_slice(&psd_sections());
    p.extend_from_slice(&COMPRESSION_ZIP_PREDICTION.to_be_bytes());
    p.extend_from_slice(&zlib(&encoded));
    let doc = read_psd(&p).unwrap();
    assert_eq!(&doc.composite.data, &[0, 1, 128, 255, 1, 2]);
}

#[test]
fn depth32_zip_prediction_unshuffles() {
    let values = [0.0f32, 0.5, 1.0, 1.5, -0.5, 0.00390625];
    let encoded = predict32(&be32(&values), 3, 2);
    let mut p = header_depth(1, 1, 3, 2, 32, 1);
    p.extend_from_slice(&psd_sections());
    p.extend_from_slice(&COMPRESSION_ZIP_PREDICTION.to_be_bytes());
    p.extend_from_slice(&zlib(&encoded));
    let doc = read_psd(&p).unwrap();
    assert_eq!(&doc.composite.data, &[0, 128, 255, 255, 0, 1]);
}

#[test]
fn depth16_layer_and_mask_channels_are_narrowed() {
    let composite = [be16(&[0x1234]), be16(&[0x5678]), be16(&[0x9abc])];
    let p = layered_psd_depth(
        16,
        3,
        3,
        &[&composite[0], &composite[1], &composite[2]],
        &[
            (0, &be16(&[0x1234])),
            (1, &be16(&[0x5678])),
            (2, &be16(&[0x9abc])),
            (-2, &be16(&[0x4000])),
        ],
    );
    let doc = read_psd(&p).unwrap();
    assert_eq!(doc.source_depth, Some(BitDepth::Sixteen));
    let layer = &doc.layers[0];
    assert_eq!(
        layer
            .channels
            .iter()
            .map(|c| (c.id, c.data[0]))
            .collect::<Vec<_>>(),
        vec![(0, 0x12), (1, 0x56), (2, 0x9a)],
        "color channels narrow to 8-bit"
    );
    assert_eq!(
        layer.mask.as_ref().and_then(|m| m.data.as_deref()),
        Some(&[0x40u8][..]),
        "-2 mask narrows too"
    );
}

#[test]
fn depth16_unmodeled_layer_channel_round_trips_at_source_depth() {
    let composite = [be16(&[0x0100]), be16(&[0x0200]), be16(&[0x0300])];
    let spot = be16(&[0xab00]);
    let p = layered_psd_depth(
        16,
        3,
        3,
        &[&composite[0], &composite[1], &composite[2]],
        &[(3, &spot)],
    );
    let doc = read_psd(&p).unwrap();
    assert_eq!(doc.source_depth, Some(BitDepth::Sixteen));
    let raw = &doc.layers[0].raw_channels;
    assert_eq!(raw.len(), 1);
    assert_eq!(raw[0].id, 3);
    assert_eq!(
        raw[0].data,
        vec![0, 0, 0xab],
        "spot channel is decoded at 16 bits and re-wrapped as an 8-bit raw stream"
    );
    assert_eq!(
        doc.layers[0].source_channels.as_ref().unwrap().planes,
        vec![(3, spot.clone())],
        "the native spot samples are retained"
    );

    // The narrowed stream round-trips; the save re-emits the native samples, so
    // reading the output narrows back to the same 8-bit stream.
    let back = read_psd(&write_psd(&doc).unwrap()).unwrap();
    assert_eq!(back.layers[0].raw_channels[0].data, vec![0, 0, 0xab]);
}

#[test]
fn depth32_unmodeled_layer_channel_round_trips_at_source_depth() {
    let composite = [be32(&[0.0]), be32(&[0.5]), be32(&[1.0])];
    let spot = be32(&[0.5]);
    let p = layered_psd_depth(
        32,
        3,
        3,
        &[&composite[0], &composite[1], &composite[2]],
        &[(-3, &spot)],
    );
    let doc = read_psd(&p).unwrap();
    assert_eq!(doc.source_depth, Some(BitDepth::ThirtyTwo));
    assert_eq!(doc.layers[0].raw_channels[0].data, vec![0, 0, 128]);
    let back = read_psd(&write_psd(&doc).unwrap()).unwrap();
    assert_eq!(back.layers[0].raw_channels[0].data, vec![0, 0, 128]);
}

#[test]
fn depth16_document_extra_channel_is_narrowed() {
    let planes = [
        be16(&[0x0100]),
        be16(&[0x0200]),
        be16(&[0x0300]),
        be16(&[0xab00]),
    ];
    let p = flat_psd(
        16,
        3,
        4,
        1,
        1,
        &[&planes[0], &planes[1], &planes[2], &planes[3]],
    );
    let doc = read_psd(&p).unwrap();
    assert_eq!(doc.channels.len(), 1, "one document extra channel");
    assert_eq!(doc.channels[0].data, vec![0xab], "extra channel narrowed");
}

#[test]
fn depth16_cmyk_narrows_then_converts_and_saves_8bit() {
    let c = be16(&[128 << 8, 0, 255 << 8]);
    let m = be16(&[64 << 8, 0, 255 << 8]);
    let y = be16(&[32 << 8, 0, 255 << 8]);
    let k = be16(&[200 << 8, 0, 255 << 8]);
    let p = flat_psd(16, 4, 4, 3, 1, &[&c, &m, &y, &k]);
    let doc = read_psd(&p).unwrap();
    assert_eq!(doc.mode, ColorMode::Rgb);
    assert_eq!(
        doc.source_depth,
        Some(BitDepth::Sixteen),
        "a converted mode still records the source depth for the app"
    );
    assert!(
        !doc.retains_source_depth(),
        "a converted mode retains no native samples"
    );
    assert_eq!(
        doc.composite.data,
        vec![100, 0, 255, 50, 0, 255, 25, 0, 255]
    );

    // The save stays 8-bit because no samples were retained.
    let out = write_psd(&doc).unwrap();
    assert_eq!(
        u16::from_be_bytes(out[22..24].try_into().unwrap()),
        8,
        "a 16-bit converted mode saves 8-bit"
    );
    assert_eq!(
        u16::from_be_bytes(out[24..26].try_into().unwrap()),
        3,
        "a 16-bit CMYK source saves as RGB, not CMYK"
    );
    let back = read_psd(&out).unwrap();
    assert_eq!(back.source_depth, None);
    assert_eq!(back.composite, doc.composite);
}

#[test]
fn depth32_cmyk_narrows_then_converts_and_saves_rgb() {
    // A 32-bit CMYK source retains no samples like the 16-bit case, so its save
    // must write the working RGB, not synthesize a four-plane CMYK file.
    let plane = |v: f32| be32(&[v, 0.0, 1.0]);
    let p = flat_psd(
        32,
        4,
        4,
        3,
        1,
        &[&plane(0.5), &plane(0.25), &plane(0.125), &plane(0.75)],
    );
    let doc = read_psd(&p).unwrap();
    assert_eq!(doc.mode, ColorMode::Rgb);
    assert_eq!(doc.source_depth, Some(BitDepth::ThirtyTwo));
    assert!(
        !doc.retains_source_depth(),
        "a converted mode retains no native samples"
    );

    let out = write_psd(&doc).unwrap();
    assert_eq!(
        u16::from_be_bytes(out[22..24].try_into().unwrap()),
        8,
        "a 32-bit converted mode saves 8-bit"
    );
    assert_eq!(
        u16::from_be_bytes(out[24..26].try_into().unwrap()),
        3,
        "a 32-bit CMYK source saves as RGB, not CMYK"
    );
    let back = read_psd(&out).unwrap();
    assert_eq!(back.source_depth, None);
    assert_eq!(back.composite, doc.composite);
}

#[test]
fn depth16_malformed_inputs_are_typed_errors() {
    // Truncated raw composite.
    let mut p = header_depth(1, 3, 2, 2, 16, 3);
    p.extend_from_slice(&psd_sections());
    p.extend_from_slice(&COMPRESSION_RAW.to_be_bytes());
    p.extend_from_slice(&[0, 1, 2]);
    assert!(matches!(read_psd(&p), Err(PsdError::Truncated)));

    // Bitmap and Indexed are meaningless at 16 bits.
    for mode in [0u16, 2] {
        let p = header_depth(1, 1, 1, 1, 16, mode);
        assert!(
            matches!(read_psd(&p), Err(PsdError::Unsupported(_))),
            "16-bit mode {mode}"
        );
    }
}

#[test]
fn depth16_grayscale_preserves_source_depth() {
    let plane = be16(&[0x0100, 0x0200]);
    let p = flat_psd(16, 1, 1, 2, 1, &[&plane]);
    let doc = read_psd(&p).unwrap();
    assert_eq!(doc.mode, ColorMode::Grayscale);
    assert_eq!(doc.source_depth, Some(BitDepth::Sixteen));
    assert!(doc.source_planes.is_some(), "native gray plane retained");
    let out = write_psd(&doc).unwrap();
    assert_eq!(u16::from_be_bytes(out[22..24].try_into().unwrap()), 16);
    let back = read_psd(&out).unwrap();
    assert_eq!(back.composite, doc.composite);
    assert_eq!(back.source_depth, Some(BitDepth::Sixteen));
}

#[test]
fn native_and_constructed_documents_have_no_source_depth() {
    let plane = vec![7u8; 3];
    let p = flat_psd(8, 3, 3, 1, 1, &[&plane, &plane, &plane]);
    assert_eq!(read_psd(&p).unwrap().source_depth, None);
    assert_eq!(
        Document::new(1, 1, ColorMode::Rgb, BitDepth::Eight).source_depth,
        None
    );
    assert_eq!(Document::from_rgba("x", 1, 1, &[0u8; 4]).source_depth, None);
}

#[test]
fn widening_narrows_back_to_the_8bit_byte() {
    for depth in [16u16, 32] {
        let plane: Vec<u8> = (0..=255).collect();
        let stride = crate::depth::row_bytes(256, depth);
        let native = crate::depth::widen_channel(&plane, 256, 1, depth);
        assert_eq!(
            crate::depth::narrow_channel(&native, 256, 1, stride, depth),
            plane,
            "widen_channel depth {depth}"
        );
        let planes = crate::depth::widen_planes(&plane, 1, 256, 1, depth);
        assert_eq!(planes, native, "widen_planes depth {depth}");
    }
}

#[test]
fn apply_prediction_inverts_undo_prediction() {
    for depth in [8u16, 16, 32] {
        let (width, rows) = (5usize, 3usize);
        let samples: Vec<u8> = (0..width * rows * (depth as usize / 8))
            .map(|i| (i * 37 + 11) as u8)
            .collect();
        let mut encoded = samples.clone();
        crate::depth::apply_prediction(&mut encoded, width, rows, depth);
        crate::depth::undo_prediction(&mut encoded, width, rows, depth);
        assert_eq!(encoded, samples, "depth {depth}");
    }
}

/// A high-depth layer plane whose length is not `width * height` must be a
/// typed error, not an out-of-bounds panic in `widen_channel`.
#[test]
fn short_high_depth_layer_plane_is_a_typed_error() {
    let mut doc = Document::new(2, 2, ColorMode::Rgb, BitDepth::Eight);
    doc.source_depth = Some(BitDepth::Sixteen);
    doc.source_planes = Some(SourcePlanes {
        depth: BitDepth::Sixteen,
        width: 2,
        height: 2,
        data: vec![0u8; 3 * 8],
    });
    doc.layers.push(Layer {
        rect: PsdRect {
            top: 0,
            left: 0,
            bottom: 2,
            right: 2,
        },
        ..Default::default()
    });

    // A modeled color channel with a one-byte plane.
    doc.layers[0].channels = vec![Channel {
        id: 0,
        data: vec![1],
    }];
    assert!(matches!(write_psd(&doc), Err(PsdError::Invalid(_))));

    // An unmodeled raw channel with a one-byte plane (after its header).
    doc.layers[0].channels.clear();
    doc.layers[0].raw_channels = vec![RawChannel {
        id: 3,
        data: vec![0, 0, 1],
    }];
    assert!(matches!(write_psd(&doc), Err(PsdError::Invalid(_))));
}

/// A hand-built document cannot pair a depth-1 retained store with a non-Bitmap
/// source mode: `write_psd` is public, so it must return a typed error rather
/// than narrow the packed plane at a sample width it cannot handle and panic.
#[test]
fn depth1_store_without_bitmap_write_back_is_a_typed_error() {
    let mut doc = Document::new(1, 1, ColorMode::Rgb, BitDepth::Eight);
    doc.source_depth = Some(BitDepth::One);
    doc.source_mode = Some(ColorMode::Rgb);
    doc.source_planes = Some(SourcePlanes {
        depth: BitDepth::One,
        width: 1,
        height: 1,
        data: vec![0],
    });
    assert!(matches!(write_psd(&doc), Err(PsdError::Invalid(_))));

    // A legitimate depth-1 Bitmap still writes back as mode 0 / depth 1.
    let p = flat_psd(1, 0, 1, 8, 1, &[&[0xAA]]);
    let bitmap = read_psd(&p).unwrap();
    let out = write_psd(&bitmap).unwrap();
    assert_eq!(
        u16::from_be_bytes(out[22..24].try_into().unwrap()),
        1,
        "a normal Bitmap output depth is 1"
    );
    assert_eq!(
        u16::from_be_bytes(out[24..26].try_into().unwrap()),
        0,
        "a normal Bitmap output header mode is Bitmap"
    );
}
