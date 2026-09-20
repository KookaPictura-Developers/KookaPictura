//! Depth-16/32 read tests: big-endian sample layouts, their 8-bit narrowing,
//! the depth-aware row stride and ZIP-with-prediction, and layer narrowing.

use super::color_modes::{flat_psd, layered_psd_depth};
use super::*;

fn be16(values: &[u16]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_be_bytes()).collect()
}

fn be32(values: &[f32]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_be_bytes()).collect()
}

/// Forward of the depth-16 per-`u16` running sum: each row stores its first
/// sample then big-endian differences, matching psd-tools `encode_prediction`.
fn predict16(samples: &[u16], width: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(samples.len() * 2);
    for (i, &v) in samples.iter().enumerate() {
        let encoded = if i % width == 0 {
            v
        } else {
            v.wrapping_sub(samples[i - 1])
        };
        out.extend_from_slice(&encoded.to_be_bytes());
    }
    out
}

/// Forward of the depth-32 codec: shuffle the four byte planes of each row
/// together, then byte-wise delta. Matches psd-tools `encode_prediction`.
fn predict32(be: &[u8], width: usize, rows: usize) -> Vec<u8> {
    let row = 4 * width;
    let mut shuffled = vec![0u8; be.len()];
    let mut k = 0;
    for r in 0..rows {
        let base = r * row;
        for offset in base..base + width {
            let mut x = offset;
            while x < base + row {
                shuffled[x] = be[k];
                k += 1;
                x += width;
            }
        }
    }
    let mut out = shuffled.clone();
    for r in 0..rows {
        let base = r * row;
        for i in (1..row).rev() {
            out[base + i] = shuffled[base + i].wrapping_sub(shuffled[base + i - 1]);
        }
    }
    out
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
fn depth16_unmodeled_layer_channel_narrows_and_saves_as_8bit() {
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

    // The narrowed stream round-trips; the writer never re-emits 16-bit payload.
    let back = read_psd(&write_psd(&doc).unwrap()).unwrap();
    assert_eq!(back.layers[0].raw_channels[0].data, vec![0, 0, 0xab]);
}

#[test]
fn depth32_unmodeled_layer_channel_narrows_and_saves_as_8bit() {
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
fn depth16_cmyk_narrows_then_converts() {
    let c = be16(&[128 << 8, 0, 255 << 8]);
    let m = be16(&[64 << 8, 0, 255 << 8]);
    let y = be16(&[32 << 8, 0, 255 << 8]);
    let k = be16(&[200 << 8, 0, 255 << 8]);
    let p = flat_psd(16, 4, 4, 3, 1, &[&c, &m, &y, &k]);
    let doc = read_psd(&p).unwrap();
    assert_eq!(doc.mode, ColorMode::Rgb);
    assert_eq!(doc.source_depth, Some(BitDepth::Sixteen));
    assert_eq!(
        doc.composite.data,
        vec![100, 0, 255, 50, 0, 255, 25, 0, 255]
    );
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
