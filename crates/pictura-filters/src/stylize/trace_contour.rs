//! Stylize ▸ Trace Contour: draw a line where each channel crosses a
//! brightness.
//!
//! Ported from photorust's `core/src/filters/stylize.rs` (GPL-3.0-or-later;
//! see the change proposal).

use pictura_core::PixelBuffer;

use crate::kernel::clamp_index;
use crate::{validate, ContourEdge, FilterError};

pub fn trace_contour(
    buf: &mut PixelBuffer,
    level: u8,
    edge: ContourEdge,
) -> Result<(), FilterError> {
    validate(buf)?;
    let w = buf.width as usize;
    let h = buf.height as usize;
    let n = w * h;
    let planes = (buf.channels as usize).min(3);
    let src = buf.data.clone();
    let level = level as i32;

    // Each channel is traced on its own, so a boundary only one channel
    // crosses leaves that channel black and the others white.
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            for c in 0..planes {
                let at = |dx: isize, dy: isize| -> i32 {
                    let sx = clamp_index(x as isize + dx, w);
                    let sy = clamp_index(y as isize + dy, h);
                    src[c * n + sy * w + sx] as i32
                };
                let here = src[c * n + i] as i32;
                let around = [at(-1, 0), at(1, 0), at(0, -1), at(0, 1)];
                let marked = match edge {
                    ContourEdge::Upper => here >= level && around.iter().any(|&v| v < level),
                    ContourEdge::Lower => here < level && around.iter().any(|&v| v >= level),
                };
                buf.data[c * n + i] = if marked { 0 } else { 255 };
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(values: &[u8], channels: u8, alpha: &[u8]) -> PixelBuffer {
        let n = values.len();
        let mut data = Vec::new();
        for _ in 0..3 {
            data.extend_from_slice(values);
        }
        if channels == 4 {
            data.extend_from_slice(alpha);
        }
        PixelBuffer {
            width: n as u32,
            height: 1,
            channels,
            data: data.into(),
        }
    }

    fn plane(buf: &PixelBuffer, c: usize) -> Vec<u8> {
        let n = buf.pixel_count();
        buf.data[c * n..c * n + n].to_vec()
    }

    #[test]
    fn trace_contour_level_zero_and_max_differ() {
        let values: Vec<u8> = (0..=255).collect();
        let base = row(&values, 3, &[]);
        let mut low = base.clone();
        trace_contour(&mut low, 0, ContourEdge::Upper).unwrap();
        let mut high = base.clone();
        trace_contour(&mut high, 255, ContourEdge::Upper).unwrap();
        assert!(
            plane(&low, 0).iter().all(|&v| v == 255),
            "level 0 crosses nothing"
        );
        assert!(plane(&high, 0).contains(&0), "level 255 inks the peak");
        assert_ne!(low.data, high.data);
    }

    #[test]
    fn trace_contour_edge_picks_the_side_of_the_step() {
        let base = row(&[0, 0, 0, 255, 255, 255], 3, &[]);
        let mut upper = base.clone();
        trace_contour(&mut upper, 128, ContourEdge::Upper).unwrap();
        let mut lower = base.clone();
        trace_contour(&mut lower, 128, ContourEdge::Lower).unwrap();
        assert_eq!(plane(&upper, 0)[3], 0, "Upper inks the bright side");
        assert_eq!(plane(&lower, 0)[2], 0, "Lower inks the dark side");
        assert!(plane(&upper, 0)[2] == 255 && plane(&lower, 0)[3] == 255);
    }

    #[test]
    fn trace_contour_leaves_a_flat_field_blank_and_alpha_untouched() {
        let n = 12;
        let values = vec![100u8; n];
        let alpha: Vec<u8> = (0..n).map(|i| (i * 5) as u8).collect();
        let base = row(&values, 4, &alpha);
        let mut out = base.clone();
        trace_contour(&mut out, 100, ContourEdge::Upper).unwrap();
        for c in 0..3 {
            assert!(
                plane(&out, c).iter().all(|&v| v == 255),
                "flat field must stay white"
            );
        }
        assert_eq!(plane(&out, 3), alpha);
    }
}
