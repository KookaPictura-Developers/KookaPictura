//! `Image > Canvas Size` (`IMG-002`).

use pictura_core::PixelBuffer;

use crate::{validate, OpsError};

/// 3×3 anchor grid, read row-major top-to-bottom, left-to-right.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    Center,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

/// Grow or crop the canvas to `width × height`, placing the source per `anchor`.
///
/// The output is planar like the input: channel `c` occupies
/// `data[c * W * H .. (c + 1) * W * H]`, row-major. Added pixels are a flat
/// `background` fill (no blending); cropped pixels are discarded, not resampled.
pub fn resize_canvas(
    buf: &PixelBuffer,
    width: u32,
    height: u32,
    anchor: Anchor,
    background: [u8; 4],
) -> Result<PixelBuffer, OpsError> {
    validate(buf)?;
    if width == 0 || height == 0 {
        return Err(OpsError::InvalidParams(
            "width and height must be >= 1".into(),
        ));
    }
    if width == buf.width && height == buf.height {
        return Ok(buf.clone());
    }

    let new_w = width as i64;
    let new_h = height as i64;
    let src_w = buf.width as i64;
    let src_h = buf.height as i64;
    let dx = match anchor {
        Anchor::TopLeft | Anchor::MiddleLeft | Anchor::BottomLeft => 0,
        Anchor::TopCenter | Anchor::Center | Anchor::BottomCenter => (new_w - src_w) / 2,
        Anchor::TopRight | Anchor::MiddleRight | Anchor::BottomRight => new_w - src_w,
    };
    let dy = match anchor {
        Anchor::TopLeft | Anchor::TopCenter | Anchor::TopRight => 0,
        Anchor::MiddleLeft | Anchor::Center | Anchor::MiddleRight => (new_h - src_h) / 2,
        Anchor::BottomLeft | Anchor::BottomCenter | Anchor::BottomRight => new_h - src_h,
    };

    // Overlap of the blitted source with the new canvas, in destination space.
    let x0 = dx.max(0);
    let x1 = (src_w + dx).min(new_w);
    let y0 = dy.max(0);
    let y1 = (src_h + dy).min(new_h);

    let nd = width as usize * height as usize;
    let ns = buf.pixel_count();
    let mut out = PixelBuffer::new(width, height, buf.channels);
    for (c, (dst, src)) in out
        .data
        .chunks_exact_mut(nd)
        .zip(buf.data.chunks_exact(ns))
        .enumerate()
    {
        dst.fill(background[c]);
        for y in y0..y1 {
            let sy = (y - dy) as usize;
            let dst_row = y as usize * width as usize;
            let src_row = sy * buf.width as usize;
            for x in x0..x1 {
                let sx = (x - dx) as usize;
                dst[dst_row + x as usize] = src[src_row + sx];
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buf3(w: u32, h: u32) -> PixelBuffer {
        let mut b = PixelBuffer::new(w, h, 3);
        let n = b.pixel_count();
        for y in 0..h as usize {
            for x in 0..w as usize {
                let i = y * w as usize + x;
                b.data[i] = x as u8;
                b.data[n + i] = y as u8;
                b.data[2 * n + i] = 10;
            }
        }
        b
    }

    fn buf4(w: u32, h: u32) -> PixelBuffer {
        let mut b = PixelBuffer::new(w, h, 4);
        let n = b.pixel_count();
        for y in 0..h as usize {
            for x in 0..w as usize {
                let i = y * w as usize + x;
                b.data[i] = x as u8;
                b.data[n + i] = y as u8;
                b.data[2 * n + i] = 30;
                b.data[3 * n + i] = 200;
            }
        }
        b
    }

    /// Sample channel `c` at `(x, y)` from a planar buffer.
    fn ch(b: &PixelBuffer, x: u32, y: u32, c: usize) -> u8 {
        let n = b.pixel_count();
        b.data[c * n + y as usize * b.width as usize + x as usize]
    }

    #[test]
    fn grow_topleft_keeps_source_and_fills_right() {
        let src = buf3(2, 2);
        let original = src.clone();
        let out = resize_canvas(&src, 3, 2, Anchor::TopLeft, [77, 88, 99, 255]).unwrap();

        assert_eq!((out.width, out.height, out.channels), (3, 2, 3));
        assert_eq!(ch(&out, 0, 0, 0), 0);
        assert_eq!(ch(&out, 1, 1, 0), 1);
        for y in 0..2 {
            assert_eq!(ch(&out, 2, y, 0), 77);
            assert_eq!(ch(&out, 2, y, 1), 88);
            assert_eq!(ch(&out, 2, y, 2), 99);
        }
        assert_eq!(src, original, "input must be untouched");
    }

    #[test]
    fn grow_center_centers_source() {
        let out = resize_canvas(&buf3(2, 2), 4, 4, Anchor::Center, [1, 2, 3, 0]).unwrap();

        assert_eq!(ch(&out, 1, 1, 0), 0);
        assert_eq!(ch(&out, 2, 2, 0), 1);
        assert_eq!(ch(&out, 1, 1, 1), 0);
        assert_eq!(ch(&out, 0, 0, 2), 3, "corner is background");
        assert_eq!(ch(&out, 3, 3, 1), 2, "bottom-right is background");
    }

    #[test]
    fn shrink_topleft_crops_top_left_sub_rectangle() {
        let out = resize_canvas(&buf3(4, 4), 2, 2, Anchor::TopLeft, [0; 4]).unwrap();

        assert_eq!((out.width, out.height), (2, 2));
        assert_eq!(ch(&out, 0, 0, 0), 0);
        assert_eq!(ch(&out, 1, 1, 0), 1);
        assert_eq!(ch(&out, 1, 1, 1), 1);
    }

    #[test]
    fn bottom_right_anchor_positions_grow_and_shrink() {
        let grown = resize_canvas(&buf3(2, 2), 4, 4, Anchor::BottomRight, [9, 9, 9, 0]).unwrap();
        assert_eq!(ch(&grown, 2, 2, 0), 0);
        assert_eq!(ch(&grown, 3, 3, 0), 1);
        assert_eq!(ch(&grown, 0, 0, 0), 9);

        let cropped = resize_canvas(&buf3(4, 4), 2, 2, Anchor::BottomRight, [0; 4]).unwrap();
        assert_eq!(ch(&cropped, 0, 0, 0), 2);
        assert_eq!(ch(&cropped, 1, 1, 0), 3);
    }

    #[test]
    fn four_channel_grow_fills_alpha_from_background() {
        let out = resize_canvas(&buf4(2, 2), 3, 2, Anchor::TopLeft, [10, 20, 30, 7]).unwrap();

        assert_eq!(out.channels, 4);
        assert_eq!(ch(&out, 2, 0, 0), 10);
        assert_eq!(ch(&out, 2, 0, 3), 7, "added alpha comes from background[3]");
        assert_eq!(ch(&out, 1, 0, 3), 200, "source alpha is preserved");
    }

    #[test]
    fn invalid_dimensions_error() {
        let src = buf3(2, 2);
        assert!(matches!(
            resize_canvas(&src, 0, 2, Anchor::Center, [0; 4]),
            Err(OpsError::InvalidParams(_))
        ));
        assert!(matches!(
            resize_canvas(&src, 2, 0, Anchor::Center, [0; 4]),
            Err(OpsError::InvalidParams(_))
        ));
    }

    #[test]
    fn one_by_one_does_not_panic() {
        assert!(resize_canvas(&buf3(1, 1), 2, 2, Anchor::Center, [0; 4]).is_ok());
        assert!(resize_canvas(&buf3(1, 1), 1, 1, Anchor::BottomRight, [0; 4]).is_ok());
    }

    #[test]
    fn identity_size_returns_clone() {
        let src = buf4(3, 2);
        let out = resize_canvas(&src, 3, 2, Anchor::BottomRight, [9, 9, 9, 9]).unwrap();
        assert_eq!(out, src);
    }
}
