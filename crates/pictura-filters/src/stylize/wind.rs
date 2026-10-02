//! Stylize ▸ Wind: blow the picture sideways off its edges.
//!
//! Ported from photorust's `core/src/filters/stylize.rs` (GPL-3.0-or-later;
//! see the change proposal).

use pictura_core::PixelBuffer;

use crate::luma::luma;
use crate::{validate, FilterError, WindMethod};

/// How different two neighbouring pixels have to be before the wind catches
/// the edge between them.
const WIND_EDGE: f64 = 10.0;

pub fn wind(
    buf: &mut PixelBuffer,
    method: WindMethod,
    from_right: bool,
) -> Result<(), FilterError> {
    validate(buf)?;
    let w = buf.width as usize;
    let h = buf.height as usize;
    let n = w * h;
    let planes = (buf.channels as usize).min(3);
    let src = buf.data.clone();
    // Away from where the wind comes from.
    let step: isize = if from_right { -1 } else { 1 };
    let (longest, density, tail) = shape(method);

    let get = |data: &[u8], x: usize, y: usize| -> [f64; 3] {
        let i = y * w + x;
        [data[i] as f64, data[n + i] as f64, data[2 * n + i] as f64]
    };

    for y in 0..h {
        for x in 0..w {
            let here = get(&src, x, y);
            let behind = (x as isize - step).clamp(0, w as isize - 1) as usize;
            let upwind = get(&src, behind, y);
            // Only where the upwind pixel is the brighter one, which keeps the
            // streaks to one side of a shape.
            if luma(upwind[0], upwind[1], upwind[2]) - luma(here[0], here[1], here[2]) < WIND_EDGE {
                continue;
            }
            let hsh = hash(x as u32, y as u32);
            if hsh % 100 >= density {
                continue;
            }
            let length = 1 + (hsh >> 8) % longest;

            for i in 0..length as isize {
                let tx = x as isize + step * i;
                if tx < 0 || tx >= w as isize {
                    break;
                }
                let sy = match method {
                    WindMethod::Stagger => {
                        let drift = (hash(x as u32, (y as isize + i) as u32) % 3) as isize - 1;
                        (y as isize + drift).clamp(0, h as isize - 1) as usize
                    }
                    _ => y,
                };
                let colour = get(&src, behind, sy);
                let t = i as f64 / length as f64;
                let weight = 1.0 - t * (1.0 - tail);
                let di = y * w + tx as usize;
                for (c, &value) in colour.iter().enumerate().take(planes) {
                    let idx = c * n + di;
                    let was = buf.data[idx] as f64;
                    buf.data[idx] = (was + (value - was) * weight).round().clamp(0.0, 255.0) as u8;
                }
            }
        }
    }
    Ok(())
}

/// Streak length, how many edges get one (out of 100), and how much strength
/// survives at the far end.
fn shape(method: WindMethod) -> (u32, u32, f64) {
    match method {
        WindMethod::Wind => (12, 55, 0.0),
        WindMethod::Blast => (32, 85, 0.55),
        WindMethod::Stagger => (16, 70, 0.25),
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

    const W: u32 = 24;
    const H: u32 = 16;

    fn block(channels: u8, alpha: impl Fn(usize) -> u8) -> PixelBuffer {
        let n = (W * H) as usize;
        let mut data = vec![0u8; n * channels as usize];
        for y in 0..H {
            for x in 0..W {
                let i = (y * W + x) as usize;
                let v: u8 = if (8..16).contains(&x) && (4..12).contains(&y) {
                    220
                } else {
                    0
                };
                data[i] = v;
                data[n + i] = v;
                data[2 * n + i] = v;
                if channels == 4 {
                    data[3 * n + i] = alpha(i);
                }
            }
        }
        PixelBuffer {
            width: W,
            height: H,
            channels,
            data: data.into(),
        }
    }

    fn changed_before(base: &PixelBuffer, out: &PixelBuffer, limit: u32) -> bool {
        let w = W as usize;
        (0..limit as usize)
            .any(|x| (0..H as usize).any(|y| base.data[y * w + x] != out.data[y * w + x]))
    }

    fn changed_after(base: &PixelBuffer, out: &PixelBuffer, start: u32) -> bool {
        let w = W as usize;
        (start as usize..W as usize)
            .any(|x| (0..H as usize).any(|y| base.data[y * w + x] != out.data[y * w + x]))
    }

    #[test]
    fn wind_streaks_downwind_and_leaves_the_upwind_side() {
        let base = block(3, |_| 0);
        let mut right = base.clone();
        wind(&mut right, WindMethod::Blast, false).unwrap();
        assert!(
            changed_after(&base, &right, 16),
            "blows streaks to the right"
        );
        assert!(
            !changed_before(&base, &right, 8),
            "the upwind side stays clean"
        );

        let mut left = base.clone();
        wind(&mut left, WindMethod::Blast, true).unwrap();
        assert!(changed_before(&base, &left, 8), "from the right blows left");
        assert!(!changed_after(&base, &left, 16), "the far side stays clean");
    }

    #[test]
    fn stagger_drifts_rows_away_from_plain_wind() {
        let base = block(3, |_| 0);
        let mut plain = base.clone();
        wind(&mut plain, WindMethod::Wind, false).unwrap();
        let mut stagger = base.clone();
        wind(&mut stagger, WindMethod::Stagger, false).unwrap();
        assert_ne!(stagger.data, base.data, "stagger still blows streaks");
        assert_ne!(stagger.data, plain.data, "stagger drifts the sampled rows");
    }

    #[test]
    fn wind_flat_ground_is_untouched() {
        let n = (W * H) as usize;
        let mut flat = PixelBuffer::new(W, H, 3);
        for v in flat.data[..n].iter_mut() {
            *v = 90;
        }
        for v in flat.data[n..2 * n].iter_mut() {
            *v = 90;
        }
        for v in flat.data[2 * n..].iter_mut() {
            *v = 90;
        }
        let base = flat.clone();
        for method in [WindMethod::Wind, WindMethod::Blast, WindMethod::Stagger] {
            let mut out = base.clone();
            wind(&mut out, method, false).unwrap();
            assert_eq!(out, base, "{method:?} must not touch flat ground");
        }
    }

    #[test]
    fn wind_is_deterministic_and_preserves_alpha() {
        let base = block(4, |i| (i * 7) as u8);
        let n = base.pixel_count();
        let alpha = base.data[3 * n..].to_vec();
        let mut a = base.clone();
        wind(&mut a, WindMethod::Stagger, false).unwrap();
        let mut b = base.clone();
        wind(&mut b, WindMethod::Stagger, false).unwrap();
        assert_eq!(a.data, b.data, "streaks come from the coordinate hash");
        assert_eq!(&a.data[3 * n..], &alpha[..]);
    }

    #[test]
    fn wind_rejects_an_empty_buffer() {
        let mut empty = PixelBuffer::new(0, 0, 3);
        assert!(wind(&mut empty, WindMethod::Wind, false).is_err());
    }
}
