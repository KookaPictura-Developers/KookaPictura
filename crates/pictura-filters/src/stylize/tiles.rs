//! Stylize ▸ Tiles: cut the picture into squares and nudge each one off where
//! it was, leaving gaps in the chosen fill.
//!
//! Ported from photorust's `core/src/filters/stylize.rs` (GPL-3.0-or-later;
//! see the change proposal).

use pictura_core::PixelBuffer;

use crate::{validate, FilterError, TileFill};

pub fn tiles(
    buf: &mut PixelBuffer,
    count: u32,
    offset: u32,
    fill: TileFill,
    foreground: [u8; 3],
    background: [u8; 3],
) -> Result<(), FilterError> {
    validate(buf)?;
    if count == 0 || count > 99 {
        return Err(FilterError::InvalidParams(format!(
            "tiles count {count} is outside 1..=99"
        )));
    }
    if offset == 0 || offset > 99 {
        return Err(FilterError::InvalidParams(format!(
            "tiles offset {offset} is outside 1..=99"
        )));
    }
    let w = buf.width as usize;
    let h = buf.height as usize;
    let n = w * h;
    let planes = (buf.channels as usize).min(3);
    // Square tiles counted across the shorter side; at least one pixel.
    let size = (w.min(h) / count as usize).max(1) as i32;
    // Minimum Offset is a percentage of the tile, at least one pixel once the
    // filter is asked for at all.
    let reach = (((size as f32 * offset as f32) / 100.0).round() as i32).max(1);

    let source = buf.clone();

    // ponytail: the reference fills/inverts RGBA; here the ground touches the
    // three color planes only, so alpha is never modified.
    match fill {
        TileFill::BackgroundColor => fill_planes(buf, n, planes, background),
        TileFill::ForegroundColor => fill_planes(buf, n, planes, foreground),
        TileFill::UnalteredImage => {}
        TileFill::InverseImage => {
            for p in 0..planes {
                let base = p * n;
                for v in &mut buf.data[base..base + n] {
                    *v = 255 - *v;
                }
            }
        }
    }

    let columns = (w as i32 + size - 1) / size;
    let rows = (h as i32 + size - 1) / size;
    let span = (reach * 2 + 1) as u32;
    for row in 0..rows {
        for column in 0..columns {
            let hash = hash(column as u32, row as u32);
            let dx = (hash % span) as i32 - reach;
            let dy = ((hash >> 16) % span) as i32 - reach;
            let (left, top) = (column * size, row * size);
            for y in top..(top + size).min(h as i32) {
                for x in left..(left + size).min(w as i32) {
                    let (tx, ty) = (x + dx, y + dy);
                    if tx >= 0 && ty >= 0 && tx < w as i32 && ty < h as i32 {
                        let si = y as usize * w + x as usize;
                        let di = ty as usize * w + tx as usize;
                        for p in 0..planes {
                            buf.data[p * n + di] = source.data[p * n + si];
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn fill_planes(buf: &mut PixelBuffer, n: usize, planes: usize, colour: [u8; 3]) {
    for (p, &v) in colour.iter().enumerate().take(planes) {
        let base = p * n;
        buf.data[base..base + n].fill(v);
    }
}

fn hash(x: u32, y: u32) -> u32 {
    let mut h = x.wrapping_mul(0x27d4_eb2d) ^ y.wrapping_mul(0x1656_67b1);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2545_f491);
    h ^= h >> 13;
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uniform(
        w: u32,
        h: u32,
        channels: u8,
        rgb: [u8; 3],
        alpha: impl Fn(usize) -> u8,
    ) -> PixelBuffer {
        let n = (w * h) as usize;
        let mut data = vec![0u8; n * channels as usize];
        for i in 0..n {
            data[i] = rgb[0];
            data[n + i] = rgb[1];
            data[2 * n + i] = rgb[2];
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

    fn has_pixel(buf: &PixelBuffer, rgb: [u8; 3]) -> bool {
        let n = buf.pixel_count();
        (0..n).any(|i| {
            buf.data[i] == rgb[0] && buf.data[n + i] == rgb[1] && buf.data[2 * n + i] == rgb[2]
        })
    }

    #[test]
    fn tiles_fill_gaps_with_the_background_colour() {
        let base = uniform(8, 8, 3, [10, 20, 30], |_| 0);
        let mut out = base.clone();
        tiles(
            &mut out,
            2,
            50,
            TileFill::BackgroundColor,
            [1, 2, 3],
            [255, 0, 0],
        )
        .unwrap();
        assert!(
            has_pixel(&out, [255, 0, 0]),
            "background fill must show in the gaps"
        );
        assert!(
            has_pixel(&out, [10, 20, 30]),
            "the tiles themselves must survive"
        );
    }

    #[test]
    fn tiles_fill_gaps_with_the_foreground_colour() {
        let base = uniform(8, 8, 3, [10, 20, 30], |_| 0);
        let mut out = base.clone();
        tiles(
            &mut out,
            2,
            50,
            TileFill::ForegroundColor,
            [255, 0, 0],
            [0, 0, 0],
        )
        .unwrap();
        assert!(
            has_pixel(&out, [255, 0, 0]),
            "foreground fill must show in the gaps"
        );
        assert!(
            has_pixel(&out, [10, 20, 30]),
            "the tiles themselves must survive"
        );
    }

    #[test]
    fn tiles_high_count_on_small_image_does_not_panic() {
        let n = 16usize;
        let mut data = vec![0u8; n * 3];
        for i in 0..n {
            data[i] = (i * 15) as u8;
            data[n + i] = (i * 7) as u8;
            data[2 * n + i] = (i * 3) as u8;
        }
        let base = PixelBuffer {
            width: 4,
            height: 4,
            channels: 3,
            data: data.into(),
        };
        let mut out = base.clone();
        tiles(
            &mut out,
            99,
            50,
            TileFill::BackgroundColor,
            [0, 0, 0],
            [255, 255, 255],
        )
        .unwrap();
        let mut again = base.clone();
        tiles(
            &mut again,
            99,
            50,
            TileFill::BackgroundColor,
            [0, 0, 0],
            [255, 255, 255],
        )
        .unwrap();
        assert_eq!(
            out.data, again.data,
            "a count above the tile size still produces deterministic output"
        );
    }

    #[test]
    fn tiles_background_invariant_on_uniform_source() {
        let src = [10, 20, 30];
        let bg = [200, 210, 220];
        let base = uniform(8, 8, 3, src, |_| 0);
        let mut out = base.clone();
        tiles(&mut out, 3, 40, TileFill::BackgroundColor, [0, 0, 0], bg).unwrap();
        let n = out.pixel_count();
        for i in 0..n {
            let p = [out.data[i], out.data[n + i], out.data[2 * n + i]];
            assert!(
                p == src || p == bg,
                "pixel {i} is neither source nor background: {p:?}"
            );
        }
    }

    #[test]
    fn tiles_inverse_fill_negates_the_ground() {
        let base = uniform(8, 8, 3, [10, 20, 30], |_| 0);
        let mut out = base.clone();
        tiles(&mut out, 3, 1, TileFill::InverseImage, [0, 0, 0], [0, 0, 0]).unwrap();
        assert!(
            has_pixel(&out, [245, 235, 225]),
            "ground is the inverted picture"
        );
    }

    #[test]
    fn tiles_unaltered_fill_leaves_a_flat_picture_alone() {
        let base = uniform(8, 8, 3, [77, 88, 99], |_| 0);
        let mut out = base.clone();
        tiles(
            &mut out,
            3,
            20,
            TileFill::UnalteredImage,
            [0, 0, 0],
            [0, 0, 0],
        )
        .unwrap();
        assert_eq!(out, base, "every tile is a copy of the same flat colour");
    }

    #[test]
    fn tiles_are_deterministic_and_preserve_alpha() {
        let base = uniform(11, 7, 4, [40, 90, 160], |i| (i * 13) as u8);
        let n = base.pixel_count();
        let alpha = base.data[3 * n..].to_vec();
        let mut a = base.clone();
        tiles(
            &mut a,
            4,
            40,
            TileFill::BackgroundColor,
            [0, 0, 0],
            [200, 200, 200],
        )
        .unwrap();
        let mut b = base.clone();
        tiles(
            &mut b,
            4,
            40,
            TileFill::BackgroundColor,
            [0, 0, 0],
            [200, 200, 200],
        )
        .unwrap();
        assert_eq!(a.data, b.data, "offsets come from the grid, not a seed");
        assert_eq!(&a.data[3 * n..], &alpha[..]);
    }

    #[test]
    fn tiles_reject_bad_count_and_offset() {
        let base = uniform(6, 6, 3, [50, 60, 70], |_| 0);
        let mut out = base.clone();
        assert!(tiles(
            &mut out,
            0,
            10,
            TileFill::BackgroundColor,
            [0, 0, 0],
            [0, 0, 0]
        )
        .is_err());
        assert!(tiles(
            &mut out,
            100,
            10,
            TileFill::BackgroundColor,
            [0, 0, 0],
            [0, 0, 0]
        )
        .is_err());
        assert!(tiles(
            &mut out,
            3,
            0,
            TileFill::BackgroundColor,
            [0, 0, 0],
            [0, 0, 0]
        )
        .is_err());
        assert!(tiles(
            &mut out,
            3,
            100,
            TileFill::BackgroundColor,
            [0, 0, 0],
            [0, 0, 0]
        )
        .is_err());
        assert_eq!(out, base, "rejected parameters must not modify the buffer");
    }
}
