//! `Image > Image Rotation` and flips (`IMG-003`).
//!
//! The quarter/half turns and flips are exact index remaps; `rotate_arbitrary`
//! grows the canvas to the axis-aligned bounding box and bilinearly resamples.

use pictura_core::PixelBuffer;

use crate::{validate, OpsError};

/// Generic index remap: for each destination pixel `(x, y)`, `src_of` yields the
/// source pixel to copy from. Channels are copied verbatim.
fn remap(
    buf: &PixelBuffer,
    out_w: u32,
    out_h: u32,
    src_of: impl Fn(u32, u32) -> (u32, u32),
) -> PixelBuffer {
    let ch = buf.channels as usize;
    let in_pc = buf.pixel_count();
    let out_pc = out_w as usize * out_h as usize;
    let mut out = PixelBuffer::new(out_w, out_h, buf.channels);
    for y in 0..out_h {
        for x in 0..out_w {
            let (sx, sy) = src_of(x, y);
            let si = sy as usize * buf.width as usize + sx as usize;
            let di = y as usize * out_w as usize + x as usize;
            for c in 0..ch {
                out.data[c * out_pc + di] = buf.data[c * in_pc + si];
            }
        }
    }
    out
}

/// 90° clockwise: `(x, y) -> (H-1-y, x)`; dimensions swap.
pub fn rotate90_cw(buf: &PixelBuffer) -> PixelBuffer {
    remap(buf, buf.height, buf.width, |x, y| (y, buf.height - 1 - x))
}

/// 90° counter-clockwise: `(x, y) -> (y, W-1-x)`; dimensions swap.
pub fn rotate90_ccw(buf: &PixelBuffer) -> PixelBuffer {
    remap(buf, buf.height, buf.width, |x, y| (buf.width - 1 - y, x))
}

/// Half-turn: `(x, y) -> (W-1-x, H-1-y)`.
pub fn rotate180(buf: &PixelBuffer) -> PixelBuffer {
    remap(buf, buf.width, buf.height, |x, y| {
        (buf.width - 1 - x, buf.height - 1 - y)
    })
}

/// Mirror along the vertical axis: `(x, y) -> (W-1-x, y)`.
pub fn flip_horizontal(buf: &PixelBuffer) -> PixelBuffer {
    remap(buf, buf.width, buf.height, |x, y| (buf.width - 1 - x, y))
}

/// Mirror along the horizontal axis: `(x, y) -> (x, H-1-y)`.
pub fn flip_vertical(buf: &PixelBuffer) -> PixelBuffer {
    remap(buf, buf.width, buf.height, |x, y| (x, buf.height - 1 - y))
}

/// Rotate about the canvas center by `angle_deg` (positive = clockwise),
/// growing the canvas to the rotated image's axis-aligned bounding box.
/// Inverse-mapped bilinear sampling with clamp-to-edge; points outside the
/// original rectangle become `background` (channel 3 is alpha for 4-channel
/// buffers).
pub fn rotate_arbitrary(
    buf: &PixelBuffer,
    angle_deg: f64,
    background: [u8; 4],
) -> Result<PixelBuffer, OpsError> {
    validate(buf)?;
    if !angle_deg.is_finite() || !(-359.99..=359.99).contains(&angle_deg) {
        return Err(OpsError::InvalidParams(
            "angle must be finite and within -359.99..=359.99".into(),
        ));
    }
    if angle_deg == 0.0 {
        return Ok(buf.clone());
    }

    let w = buf.width as f64;
    let h = buf.height as f64;
    let (sin, cos) = angle_deg.to_radians().sin_cos();
    // The epsilon keeps an exact quarter turn, where cos(90°) is ~6e-17 rather
    // than 0, from rounding an exact integer size up to the next pixel.
    // ponytail: 1e-9 absolute slack, safe below the 300k PSB limit.
    let out_w = ((w * cos.abs() + h * sin.abs()) - 1e-9).ceil().max(1.0) as u32;
    let out_h = ((w * sin.abs() + h * cos.abs()) - 1e-9).ceil().max(1.0) as u32;

    let ch = buf.channels as usize;
    let in_pc = buf.pixel_count();
    let out_pc = out_w as usize * out_h as usize;
    let in_w = buf.width as usize;
    let mut out = PixelBuffer::new(out_w, out_h, buf.channels);

    let src_cx = w / 2.0;
    let src_cy = h / 2.0;
    let dst_cx = out_w as f64 / 2.0;
    let dst_cy = out_h as f64 / 2.0;
    let max_x = w - 1.0;
    let max_y = h - 1.0;

    for y in 0..out_h {
        for x in 0..out_w {
            let dx = (x as f64 + 0.5) - dst_cx;
            let dy = (y as f64 + 0.5) - dst_cy;
            let sx = src_cx + cos * dx + sin * dy;
            let sy = src_cy - sin * dx + cos * dy;
            let di = y as usize * out_w as usize + x as usize;

            if sx < 0.0 || sy < 0.0 || sx > w || sy > h {
                for (c, &b) in background.iter().take(ch).enumerate() {
                    out.data[c * out_pc + di] = b;
                }
                continue;
            }

            let u = sx - 0.5;
            let v = sy - 0.5;
            let x0f = u.floor();
            let y0f = v.floor();
            let fx = u - x0f;
            let fy = v - y0f;
            let x0 = x0f.clamp(0.0, max_x) as usize;
            let y0 = y0f.clamp(0.0, max_y) as usize;
            let x1 = (x0f + 1.0).clamp(0.0, max_x) as usize;
            let y1 = (y0f + 1.0).clamp(0.0, max_y) as usize;

            for c in 0..ch {
                let base = c * in_pc;
                let p00 = buf.data[base + y0 * in_w + x0] as f64;
                let p10 = buf.data[base + y0 * in_w + x1] as f64;
                let p01 = buf.data[base + y1 * in_w + x0] as f64;
                let p11 = buf.data[base + y1 * in_w + x1] as f64;
                let top = p00 + (p10 - p00) * fx;
                let bot = p01 + (p11 - p01) * fx;
                let val = top + (bot - top) * fy;
                out.data[c * out_pc + di] = val.round().clamp(0.0, 255.0) as u8;
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
        let pc = b.pixel_count();
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) as usize;
                b.data[i] = (x * 10 + y) as u8;
                b.data[pc + i] = (x + y * 10) as u8;
                b.data[2 * pc + i] = (x * 3 + y * 7) as u8;
            }
        }
        b
    }

    fn buf4(w: u32, h: u32) -> PixelBuffer {
        let mut b = PixelBuffer::new(w, h, 4);
        let pc = b.pixel_count();
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) as usize;
                b.data[i] = (x * 10 + y) as u8;
                b.data[pc + i] = (x + y * 10) as u8;
                b.data[2 * pc + i] = (x * 3 + y * 7) as u8;
                b.data[3 * pc + i] = 200u32.saturating_sub(x + y) as u8;
            }
        }
        b
    }

    fn px(buf: &PixelBuffer, x: u32, y: u32, c: usize) -> u8 {
        buf.data[c * buf.pixel_count() + (y * buf.width + x) as usize]
    }

    #[test]
    fn rotate90_cw_then_ccw_round_trips() {
        let src = buf3(3, 2);
        assert_eq!(rotate90_ccw(&rotate90_cw(&src)), src);
        assert_eq!(rotate90_cw(&rotate90_ccw(&src)), src);
    }

    #[test]
    fn rotate180_twice_is_identity() {
        let src = buf3(3, 2);
        assert_eq!(rotate180(&rotate180(&src)), src);
    }

    #[test]
    fn flip_horizontal_twice_is_identity() {
        let src = buf3(3, 2);
        assert_eq!(flip_horizontal(&flip_horizontal(&src)), src);
    }

    #[test]
    fn flip_h_then_flip_v_equals_rotate180() {
        let src = buf3(3, 2);
        assert_eq!(flip_vertical(&flip_horizontal(&src)), rotate180(&src));
    }

    #[test]
    fn quarter_turns_swap_dimensions_and_remap_exactly() {
        let src = buf3(3, 2);
        let cw = rotate90_cw(&src);
        assert_eq!((cw.width, cw.height), (2, 3));
        let ccw = rotate90_ccw(&src);
        assert_eq!((ccw.width, ccw.height), (2, 3));

        for y in 0..2u32 {
            for x in 0..3u32 {
                for c in 0..3 {
                    // 90° CW: (x, y) -> (H-1-y, x)
                    assert_eq!(px(&cw, 1 - y, x, c), px(&src, x, y, c), "cw");
                    // 90° CCW: (x, y) -> (y, W-1-x)
                    assert_eq!(px(&ccw, y, 3 - 1 - x, c), px(&src, x, y, c), "ccw");
                }
            }
        }
    }

    #[test]
    fn arbitrary_90_matches_rotate90_cw() {
        let src = buf3(5, 3);
        let cw = rotate90_cw(&src);
        let arb = rotate_arbitrary(&src, 90.0, [0; 4]).unwrap();
        assert_eq!((arb.width, arb.height), (cw.width, cw.height));
        for i in 0..arb.data.len() {
            let d = (arb.data[i] as i32 - cw.data[i] as i32).abs();
            assert!(d <= 1, "sample {i} differs by {d}");
        }
    }

    #[test]
    fn arbitrary_preserves_center_and_fills_corners() {
        let src = buf4(5, 5);
        let bg = [9u8, 8, 7, 6];
        let rot = rotate_arbitrary(&src, 30.0, bg).unwrap();

        let (cx, cy) = (rot.width / 2, rot.height / 2);
        for c in 0..4 {
            assert_eq!(px(&rot, cx, cy, c), px(&src, 2, 2, c), "center channel {c}");
        }

        let (rx, by) = (rot.width - 1, rot.height - 1);
        for (c, &b) in bg.iter().enumerate() {
            assert_eq!(px(&rot, 0, 0, c), b, "corner tl {c}");
            assert_eq!(px(&rot, rx, 0, c), b, "corner tr {c}");
            assert_eq!(px(&rot, 0, by, c), b, "corner bl {c}");
            assert_eq!(px(&rot, rx, by, c), b, "corner br {c}");
        }
    }

    #[test]
    fn arbitrary_rejects_bad_angles() {
        let src = buf3(2, 2);
        for a in [360.0, -360.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(
                matches!(
                    rotate_arbitrary(&src, a, [0; 4]),
                    Err(OpsError::InvalidParams(_))
                ),
                "angle {a} must be rejected"
            );
        }
        assert!(rotate_arbitrary(&src, 359.99, [0; 4]).is_ok());
        assert!(rotate_arbitrary(&src, -359.99, [0; 4]).is_ok());
    }

    #[test]
    fn arbitrary_preserves_alpha() {
        let mut src = buf4(4, 4);
        let pc = src.pixel_count();
        for a in &mut src.data[3 * pc..4 * pc] {
            *a = 77;
        }
        let bg = [0u8, 0, 0, 0];
        let rot = rotate_arbitrary(&src, 45.0, bg).unwrap();
        let rot_pc = rot.pixel_count();
        for i in 0..rot_pc {
            let alpha = rot.data[3 * rot_pc + i];
            assert!(alpha == 77 || alpha == bg[3], "alpha {alpha} unexpected");
        }
    }

    #[test]
    fn one_by_one_does_not_panic() {
        let src = buf3(1, 1);
        assert_eq!(rotate90_cw(&src), src);
        assert_eq!(rotate90_ccw(&src), src);
        assert_eq!(rotate180(&src), src);
        assert_eq!(flip_horizontal(&src), src);
        assert_eq!(flip_vertical(&src), src);
        for a in [0.0, 45.0, 90.0, -90.0, 180.0, 359.99] {
            let _ = rotate_arbitrary(&src, a, [0; 4]).unwrap();
        }
    }
}
