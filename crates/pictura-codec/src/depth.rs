//! Depth-aware row stride, 16/32-bit sample narrowing, and ZIP-with-prediction
//! inversion. Kept out of `read.rs` only to respect the file-size cap; the
//! encodings mirror `psd-tools`' `compression/__init__.py`.

/// One channel row's byte stride at `depth` bits per sample: `ceil(width *
/// depth / 8)`, so a depth-1 Bitmap row is `ceil(width / 8)`, a depth-16 row is
/// `2 * width`, and a depth-32 row is `4 * width`.
pub(crate) fn row_bytes(width: usize, depth: u16) -> usize {
    (width * depth as usize).div_ceil(8)
}

/// Narrow a 16/32-bit planar buffer to the 8-bit layout: for each of `channels`
/// planes, each big-endian `u16`/`f32` sample becomes one byte. `row_bytes` is
/// the source row stride, so `data` is exactly `channels * row_bytes * height`.
pub(crate) fn narrow_planes(
    data: &[u8],
    channels: usize,
    row_bytes: usize,
    width: usize,
    height: usize,
    depth: u16,
) -> Vec<u8> {
    let src_plane = row_bytes * height;
    let mut out = Vec::with_capacity(channels * width * height);
    for c in 0..channels {
        out.extend_from_slice(&narrow_channel(
            &data[c * src_plane..(c + 1) * src_plane],
            width,
            height,
            row_bytes,
            depth,
        ));
    }
    out
}

/// Narrow one 16/32-bit plane to `width * height` 8-bit samples.
pub(crate) fn narrow_channel(
    plane: &[u8],
    width: usize,
    height: usize,
    row_bytes: usize,
    depth: u16,
) -> Vec<u8> {
    use crate::color_mode::{narrow_f32_to_8, narrow_u16_to_8};
    let sample = (depth / 8) as usize;
    let mut out = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            let i = y * row_bytes + x * sample;
            out.push(match depth {
                16 => narrow_u16_to_8(u16::from_be_bytes([plane[i], plane[i + 1]])),
                _ => narrow_f32_to_8(f32::from_be_bytes([
                    plane[i],
                    plane[i + 1],
                    plane[i + 2],
                    plane[i + 3],
                ])),
            });
        }
    }
    out
}

/// Invert the delta applied by ZIP-with-prediction over `rows` scanlines of
/// `width` pixels. Depth 8 is byte-wise; depth 16 is a per-row running sum of
/// big-endian `u16` values mod 2^16; depth 32 first undoes the byte-wise delta
/// over each `4 * width`-byte row, then un-shuffles the four byte planes back
/// into big-endian floats. Depth 1 never reaches ZIP (rejected earlier).
pub(crate) fn undo_prediction(data: &mut [u8], width: usize, rows: usize, depth: u16) {
    if width == 0 {
        return;
    }
    match depth {
        16 => {
            let row = 2 * width;
            for r in 0..rows {
                let start = r * row;
                if start + row > data.len() {
                    break;
                }
                let mut prev = 0u16;
                for x in 0..width {
                    let i = start + x * 2;
                    let v = u16::from_be_bytes([data[i], data[i + 1]]).wrapping_add(prev);
                    data[i..i + 2].copy_from_slice(&v.to_be_bytes());
                    prev = v;
                }
            }
        }
        32 => {
            let row = 4 * width;
            for r in 0..rows {
                let row_start = r * row;
                let row_end = (row_start + row).min(data.len());
                for i in (row_start + 1)..row_end {
                    data[i] = data[i].wrapping_add(data[i - 1]);
                }
            }
            unshuffle_32(data, width, rows);
        }
        _ => {
            for row_start in (0..data.len()).step_by(width) {
                let row_end = (row_start + width).min(data.len());
                for i in (row_start + 1)..row_end {
                    data[i] = data[i].wrapping_add(data[i - 1]);
                }
            }
        }
    }
}

/// Undo the depth-32 four-byte-plane shuffle in place: psd-tools packs each
/// row's four byte planes together (`12341234` -> `111222333444`) before the
/// byte-wise delta, so this restores the big-endian float order.
fn unshuffle_32(data: &mut [u8], width: usize, rows: usize) {
    let row = 4 * width;
    let src = data.to_vec();
    for r in 0..rows {
        let base = r * row;
        if base + row > data.len() {
            break;
        }
        let mut dst = base;
        for offset in base..base + width {
            let mut x = offset;
            while x < base + row {
                data[dst] = src[x];
                dst += 1;
                x += width;
            }
        }
    }
}
