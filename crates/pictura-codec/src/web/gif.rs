//! A GIF89a encoder for one indexed frame: a global colour table, a Graphic
//! Control Extension when there is a transparent entry, optional interlacing,
//! and LZW with a hashed dictionary (photorust's searched it linearly).

use std::collections::HashMap;

use super::Indexed;

/// Encode `image` as a GIF89a file.
pub fn encode_gif(image: &Indexed, interlaced: bool) -> Vec<u8> {
    let colors = image.palette.len().clamp(2, 256);
    let bits = (usize::BITS - (colors - 1).leading_zeros()).max(1) as u8;
    let table = 1usize << bits;
    let (w, h) = (image.width as u16, image.height as u16);
    let mut out = b"GIF89a".to_vec();
    out.extend_from_slice(&w.to_le_bytes());
    out.extend_from_slice(&h.to_le_bytes());
    out.push(0x80 | ((bits - 1) << 4) | (bits - 1));
    out.extend_from_slice(&[0, 0]);
    for i in 0..table {
        let c = image.palette.get(i).copied().unwrap_or([0, 0, 0, 255]);
        out.extend_from_slice(&c[..3]);
    }
    if let Some(clear) = image.transparent_index() {
        out.extend_from_slice(&[0x21, 0xf9, 0x04, 0x01, 0x00, 0x00, clear, 0x00]);
    }
    out.push(0x2c);
    out.extend_from_slice(&[0, 0, 0, 0]);
    out.extend_from_slice(&w.to_le_bytes());
    out.extend_from_slice(&h.to_le_bytes());
    out.push(if interlaced { 0x40 } else { 0x00 });
    let rows: Vec<usize> = if interlaced {
        let h = usize::from(h);
        [(0, 8), (4, 8), (2, 4), (1, 2)]
            .into_iter()
            .flat_map(|(start, step)| (start..h).step_by(step))
            .collect()
    } else {
        (0..usize::from(h)).collect()
    };
    let width = usize::from(w);
    let pixels: Vec<u8> = rows
        .iter()
        .flat_map(|&y| image.indices[y * width..(y + 1) * width].iter().copied())
        .collect();
    let min_code = bits.max(2);
    out.push(min_code);
    let data = lzw(&pixels, min_code);
    for block in data.chunks(255) {
        out.push(block.len() as u8);
        out.extend_from_slice(block);
    }
    out.push(0);
    out.push(0x3b);
    out
}

/// Variable-width LZW, codes packed least-significant bit first.
fn lzw(pixels: &[u8], min_code: u8) -> Vec<u8> {
    let clear = 1u16 << min_code;
    let end = clear + 1;
    let mut out = Vec::new();
    let mut buffer = 0u32;
    let mut filled = 0u32;
    let mut width = u32::from(min_code) + 1;
    let mut emit = |code: u16, width: u32, out: &mut Vec<u8>| {
        buffer |= u32::from(code) << filled;
        filled += width;
        while filled >= 8 {
            out.push(buffer as u8);
            buffer >>= 8;
            filled -= 8;
        }
    };
    let mut table: HashMap<(u16, u8), u16> = HashMap::new();
    let mut next = end + 1;
    emit(clear, width, &mut out);
    let Some((&first, rest)) = pixels.split_first() else {
        emit(end, width, &mut out);
        if filled > 0 {
            out.push(buffer as u8);
        }
        return out;
    };
    let mut prefix = u16::from(first);
    for &pixel in rest {
        if let Some(&code) = table.get(&(prefix, pixel)) {
            prefix = code;
            continue;
        }
        emit(prefix, width, &mut out);
        if next < 4096 {
            table.insert((prefix, pixel), next);
            // The decoder widens once the code it will add next needs it.
            if u32::from(next) == 1 << width && width < 12 {
                width += 1;
            }
            next += 1;
        } else {
            emit(clear, width, &mut out);
            table.clear();
            next = end + 1;
            width = u32::from(min_code) + 1;
        }
        prefix = u16::from(pixel);
    }
    emit(prefix, width, &mut out);
    emit(end, width, &mut out);
    if filled > 0 {
        out.push(buffer as u8);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal LZW decoder, the inverse of `lzw`, for the round trip.
    fn unlzw(data: &[u8], min_code: u8) -> Vec<u8> {
        let clear = 1usize << min_code;
        let end = clear + 1;
        let mut width = u32::from(min_code) + 1;
        let mut dict: Vec<Vec<u8>> = (0..clear).map(|i| vec![i as u8]).collect();
        dict.push(Vec::new());
        dict.push(Vec::new());
        let (mut bits, mut have, mut pos) = (0u32, 0u32, 0usize);
        let mut prev: Option<Vec<u8>> = None;
        let mut out = Vec::new();
        loop {
            while have < width {
                bits |= u32::from(data[pos]) << have;
                pos += 1;
                have += 8;
            }
            let code = (bits & ((1 << width) - 1)) as usize;
            bits >>= width;
            have -= width;
            if code == clear {
                dict.truncate(end + 1);
                width = u32::from(min_code) + 1;
                prev = None;
                continue;
            }
            if code == end {
                return out;
            }
            let entry = match (dict.get(code), &prev) {
                (Some(e), _) => e.clone(),
                (None, Some(p)) => {
                    let mut e = p.clone();
                    e.push(p[0]);
                    e
                }
                (None, None) => panic!("bad code"),
            };
            out.extend_from_slice(&entry);
            if let Some(p) = prev {
                if dict.len() < 4096 {
                    let mut e = p;
                    e.push(entry[0]);
                    dict.push(e);
                    if dict.len() == 1 << width && width < 12 {
                        width += 1;
                    }
                }
            }
            prev = Some(entry);
        }
    }

    #[test]
    fn lzw_round_trips_long_and_repetitive_runs() {
        let pixels: Vec<u8> = (0..20_000u32).map(|i| ((i / 7) % 13) as u8).collect();
        assert_eq!(unlzw(&lzw(&pixels, 4), 4), pixels);
        let noisy: Vec<u8> = (0..9_000u32)
            .map(|i| (i.wrapping_mul(2_654_435_761) >> 24) as u8)
            .collect();
        assert_eq!(unlzw(&lzw(&noisy, 8), 8), noisy);
    }

    #[test]
    fn a_gif_has_its_header_table_and_transparency() {
        let image = Indexed {
            width: 3,
            height: 2,
            palette: vec![[255, 0, 0, 255], [0, 0, 255, 255], [9, 9, 9, 0]],
            indices: vec![0, 1, 2, 2, 1, 0],
        };
        let gif = encode_gif(&image, false);
        assert_eq!(&gif[..6], b"GIF89a");
        assert_eq!(&gif[6..10], &[3, 0, 2, 0]);
        assert_eq!(gif[10], 0x80 | (1 << 4) | 1, "a 4-entry global table");
        assert_eq!(&gif[13..16], &[255, 0, 0]);
        let gce = gif
            .windows(4)
            .position(|w| w == [0x21, 0xf9, 0x04, 0x01])
            .unwrap();
        assert_eq!(gif[gce + 6], 2, "the transparent index");
        assert_eq!(*gif.last().unwrap(), 0x3b);
        let interlaced = encode_gif(&image, true);
        assert_ne!(gif, interlaced);
    }
}
