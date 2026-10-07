//! Spec scenarios from the replaced `stylize/extrude.rs` unit tests, run through
//! `apply` against the ported photorust engine.

use super::shims::*;
use pictura_core::PixelBuffer;
use pictura_filters::*;

fn image(w: u32, h: u32, channels: u8, px: &[[u8; 3]], alpha: impl Fn(usize) -> u8) -> PixelBuffer {
    let n = (w * h) as usize;
    assert_eq!(px.len(), n);
    let mut data = vec![0u8; n * channels as usize];
    for (i, p) in px.iter().enumerate() {
        data[i] = p[0];
        data[n + i] = p[1];
        data[2 * n + i] = p[2];
        if channels == 4 {
            data[3 * n + i] = alpha(i);
        }
    }
    PixelBuffer {
        width: w,
        height: h,
        channels,
        data: data.into(),
    }
}

fn rgb_at(buf: &PixelBuffer, x: usize, y: usize) -> [u8; 3] {
    let n = buf.pixel_count();
    let i = y * buf.width as usize + x;
    [buf.data[i], buf.data[n + i], buf.data[2 * n + i]]
}

#[test]
fn extrude_changes_the_image_and_is_deterministic() {
    let px: Vec<[u8; 3]> = (0..64)
        .map(|i| {
            let v = (i * 4) as u8;
            [v, v.wrapping_add(20), v.wrapping_add(40)]
        })
        .collect();
    let base = image(8, 8, 3, &px, |_| 0);
    let mut a = base.clone();
    extrude(&mut a, ExtrudeType::Blocks, 3, 30.0, false, false, false).unwrap();
    let mut b = base.clone();
    extrude(&mut b, ExtrudeType::Blocks, 3, 30.0, false, false, false).unwrap();
    assert_eq!(a.data, b.data, "random heights come from the position hash");
    assert_ne!(a.data, base.data, "extrude must redraw the picture");
}

#[test]
fn extrude_level_based_stands_bright_tiles_tallest() {
    // Left tile dark, middle tile bright, right tile mid. Level-based
    // stands them by brightness, so the bright tile is drawn last and its
    // solid front covers the overlap it makes with the dark neighbour.
    let mut px = Vec::new();
    for _ in 0..4 {
        px.extend_from_slice(&[
            [50, 50, 50],
            [50, 50, 50],
            [220, 220, 220],
            [220, 220, 220],
            [130, 130, 130],
            [130, 130, 130],
        ]);
    }
    let base = image(6, 4, 3, &px, |_| 0);
    let mut out = base.clone();
    extrude(&mut out, ExtrudeType::Blocks, 2, 255.0, true, true, false).unwrap();
    assert_eq!(
        rgb_at(&out, 1, 1),
        [220, 220, 220],
        "the taller bright tower paints over its dark neighbour"
    );
}

#[test]
fn extrude_rejects_bad_depth_and_empty_buffer() {
    let base = image(4, 4, 3, &[[10, 20, 30]; 16], |_| 0);
    for depth in [f32::NAN, f32::INFINITY, 0.0, 0.5, -5.0, 256.0] {
        let mut out = base.clone();
        assert!(
            extrude(&mut out, ExtrudeType::Blocks, 3, depth, false, false, false).is_err(),
            "depth {depth} must be rejected"
        );
        assert_eq!(out, base, "rejected parameters must not modify the buffer");
    }
    for size in [1u32, 256] {
        let mut out = base.clone();
        assert!(
            extrude(
                &mut out,
                ExtrudeType::Blocks,
                size,
                30.0,
                false,
                false,
                false
            )
            .is_err(),
            "size {size} must be rejected"
        );
        assert_eq!(out, base, "rejected parameters must not modify the buffer");
    }
    let mut empty = PixelBuffer::new(0, 0, 3);
    assert!(extrude(
        &mut empty,
        ExtrudeType::Blocks,
        3,
        30.0,
        false,
        false,
        false
    )
    .is_err());
}

#[test]
fn extrude_survives_a_tower_thrown_past_the_left_edge() {
    // A bright tile in the lower-left corner grows 1.85x from the middle,
    // throwing its left face entirely to negative x. The face's scanline
    // has left < right but both negative, so the old usize cast exploded.
    let mut px = vec![[0u8, 0, 0]; 8 * 16];
    for y in 8..12usize {
        for x in 0..4usize {
            px[y * 8 + x] = [255, 255, 255];
        }
    }
    let base = image(8, 16, 3, &px, |_| 0);
    let mut out = base.clone();
    extrude(&mut out, ExtrudeType::Blocks, 4, 255.0, true, true, false).unwrap();
}

#[test]
fn mask_incomplete_leaves_partial_tiles_as_source() {
    let px: Vec<[u8; 3]> = (0..77)
        .map(|i| {
            let v = (i * 3) as u8;
            [v, v.wrapping_add(40), v.wrapping_add(80)]
        })
        .collect();
    let base = image(11, 7, 3, &px, |_| 0);
    let n = base.pixel_count();
    let mut masked = base.clone();
    extrude(&mut masked, ExtrudeType::Blocks, 4, 1.0, false, true, true).unwrap();
    let mut unmasked = base.clone();
    extrude(
        &mut unmasked,
        ExtrudeType::Blocks,
        4,
        1.0,
        false,
        true,
        false,
    )
    .unwrap();
    let mut changed = false;
    for y in 0..7usize {
        for x in 0..11usize {
            if x >= 8 || y >= 4 {
                let i = y * 11 + x;
                assert_eq!(masked.data[i], base.data[i], "masked R at ({x},{y})");
                assert_eq!(
                    masked.data[n + i],
                    base.data[n + i],
                    "masked G at ({x},{y})"
                );
                assert_eq!(
                    masked.data[2 * n + i],
                    base.data[2 * n + i],
                    "masked B at ({x},{y})"
                );
                if unmasked.data[i] != base.data[i] {
                    changed = true;
                }
            }
        }
    }
    assert!(
        changed,
        "mask_incomplete=false must throw the partial tiles"
    );
}

#[test]
fn solid_front_paints_the_cell_average() {
    let cell = [
        [10u8, 20, 30],
        [90, 100, 110],
        [130, 140, 150],
        [170, 180, 190],
    ];
    let mut px = vec![[0u8, 0, 0]; 16];
    px[0] = cell[0];
    px[1] = cell[1];
    px[4] = cell[2];
    px[5] = cell[3];
    let base = image(4, 4, 3, &px, |_| 0);
    let avg = [100u8, 110, 120];
    let n = base.pixel_count();
    assert!(
        !(0..n).any(|i| base.data[i] == avg[0]
            && base.data[n + i] == avg[1]
            && base.data[2 * n + i] == avg[2]),
        "the cell mean must not already exist in the source"
    );
    let mut out = base.clone();
    extrude(&mut out, ExtrudeType::Blocks, 2, 30.0, true, true, false).unwrap();
    assert!(
        (0..n).any(|i| out.data[i] == avg[0]
            && out.data[n + i] == avg[1]
            && out.data[2 * n + i] == avg[2]),
        "solid front must paint the exact per-cell mean"
    );
}

#[test]
fn pyramids_geometry_differs_from_blocks_and_keeps_alpha() {
    let px: Vec<[u8; 3]> = (0..64)
        .map(|i| {
            let v = (i * 3) as u8;
            [v, v.wrapping_add(20), v.wrapping_add(40)]
        })
        .collect();
    let base = image(8, 8, 4, &px, |i| (i * 5) as u8);
    let n = base.pixel_count();
    let alpha = base.data[3 * n..].to_vec();
    let mut blocks = base.clone();
    extrude(
        &mut blocks,
        ExtrudeType::Blocks,
        4,
        200.0,
        false,
        true,
        false,
    )
    .unwrap();
    let mut pyramids = base.clone();
    extrude(
        &mut pyramids,
        ExtrudeType::Pyramids,
        4,
        200.0,
        false,
        true,
        false,
    )
    .unwrap();
    assert_ne!(blocks.data, pyramids.data, "pyramids are not blocks");
    assert_eq!(&pyramids.data[3 * n..], &alpha[..]);
}

#[test]
fn extrude_preserves_alpha_and_survives_tiny_and_ragged_tiles() {
    let px: Vec<[u8; 3]> = (0..77)
        .map(|i| [(i * 3) as u8, (i * 5) as u8, (i * 7) as u8])
        .collect();
    let base = image(11, 7, 4, &px, |i| (i * 11) as u8);
    let n = base.pixel_count();
    let alpha = base.data[3 * n..].to_vec();
    for (kind, mask) in [(ExtrudeType::Blocks, false), (ExtrudeType::Pyramids, true)] {
        let mut out = base.clone();
        extrude(&mut out, kind, 4, 200.0, true, true, mask).unwrap();
        assert_eq!(&out.data[3 * n..], &alpha[..]);
    }
}
