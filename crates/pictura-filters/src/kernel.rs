//! Shared 1-D kernel helpers for the filter family.
//!
//! The UI radius maps to a Gaussian with the 3σ support convention from
//! `FILT-010`; every spatial filter builds its weights here.

use pictura_core::PixelBuffer;
use rand_chacha::{rand_core::RngCore, ChaCha8Rng};

use crate::FilterError;

/// One uniform `f64` in `[0, 1)` from 53 random bits.
pub(crate) fn unit_f64(rng: &mut ChaCha8Rng) -> f64 {
    (rng.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
}

/// UI radius is the 3σ support per `FILT-010`; floor σ at 0.1.
pub fn sigma_from_radius(radius: f64) -> f64 {
    (radius / 3.0).max(0.1)
}

/// Normalized 1-D FIR kernel, support `⌈3σ⌉` each side.
pub fn gaussian_kernel(sigma: f64) -> Vec<f64> {
    if sigma <= 0.0 {
        return vec![1.0];
    }
    let support = (3.0 * sigma).ceil() as isize;
    let size = (2 * support + 1) as usize;
    let two_sigma_sq = 2.0 * sigma * sigma;
    let mut weights = Vec::with_capacity(size);
    let mut sum = 0.0;
    for i in -support..=support {
        let x = i as f64;
        let w = (-(x * x) / two_sigma_sq).exp();
        sum += w;
        weights.push(w);
    }
    for w in &mut weights {
        *w /= sum;
    }
    weights
}

/// Clamp-to-edge index access (border policy per `FILT-010`).
pub fn clamp_index(i: isize, n: usize) -> usize {
    if n == 0 {
        return 0;
    }
    i.clamp(0, n as isize - 1) as usize
}

/// Edge policy for a source coordinate that falls outside the image.
#[derive(Clone, Copy)]
pub(crate) enum Edge {
    Clamp,
    Wrap,
}

/// Bilinear sample at `(x, y)` under `edge`.
pub(crate) fn bilinear(plane: &[u8], w: usize, h: usize, x: f64, y: f64, edge: Edge) -> f64 {
    let x0 = x.floor();
    let y0 = y.floor();
    let (fx, fy) = (x - x0, y - y0);
    let (i0, j0) = (x0 as isize, y0 as isize);
    let (xi, xi1) = (idx(i0, w, edge), idx(i0 + 1, w, edge));
    let (yi, yi1) = (idx(j0, h, edge), idx(j0 + 1, h, edge));
    let p00 = plane[yi * w + xi] as f64;
    let p10 = plane[yi * w + xi1] as f64;
    let p01 = plane[yi1 * w + xi] as f64;
    let p11 = plane[yi1 * w + xi1] as f64;
    (p00 * (1.0 - fx) + p10 * fx) * (1.0 - fy) + (p01 * (1.0 - fx) + p11 * fx) * fy
}

fn idx(i: isize, n: usize, edge: Edge) -> usize {
    match edge {
        Edge::Clamp => clamp_index(i, n),
        Edge::Wrap => i.rem_euclid(n as isize) as usize,
    }
}

pub(crate) fn to_u8(v: f64) -> u8 {
    v.round().clamp(0.0, 255.0) as u8
}

pub(crate) fn invalid(msg: String) -> FilterError {
    FilterError::InvalidParams(msg)
}

/// Separable box mean with clamp-to-edge, for smoothness merges.
pub(crate) fn box_mean(src: &[f64], w: usize, h: usize, radius: usize) -> Vec<f64> {
    if radius == 0 {
        return src.to_vec();
    }
    let mut tmp = vec![0.0f64; src.len()];
    for y in 0..h {
        for x in 0..w {
            let mut s = 0.0;
            for d in 0..=2 * radius {
                let sx = clamp_index(x as isize + d as isize - radius as isize, w);
                s += src[y * w + sx];
            }
            tmp[y * w + x] = s;
        }
    }
    let area = ((2 * radius + 1) * (2 * radius + 1)) as f64;
    let mut out = vec![0.0f64; src.len()];
    for y in 0..h {
        for x in 0..w {
            let mut s = 0.0;
            for d in 0..=2 * radius {
                let sy = clamp_index(y as isize + d as isize - radius as isize, h);
                s += tmp[sy * w + x];
            }
            out[y * w + x] = s / area;
        }
    }
    out
}

/// 3x3 clamp-to-edge convolution of the color planes (alpha untouched).
pub(crate) fn convolve3x3_planes(buf: &mut PixelBuffer, kernel: &[[f64; 3]; 3], norm: f64) {
    let w = buf.width as usize;
    let h = buf.height as usize;
    let n = w * h;
    let planes = (buf.channels as usize).min(3);
    for c in 0..planes {
        let base = c * n;
        let src = buf.data[base..base + n].to_vec();
        for y in 0..h {
            for x in 0..w {
                let mut acc = 0f64;
                for (ky, row) in kernel.iter().enumerate() {
                    let sy = clamp_index(y as isize + ky as isize - 1, h);
                    for (kx, &kv) in row.iter().enumerate() {
                        let sx = clamp_index(x as isize + kx as isize - 1, w);
                        acc += kv * src[sy * w + sx] as f64;
                    }
                }
                buf.data[base + y * w + x] = (acc / norm).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
}

/// Separable Gaussian blur of the color planes of a 3/4-channel planar buffer
/// (alpha untouched), clamp-to-edge. Shared by Gaussian Blur and Unsharp Mask.
pub fn gaussian_blur_planes(buf: &mut PixelBuffer, sigma: f64) {
    let w = buf.width as usize;
    let h = buf.height as usize;
    let n = w * h;
    if sigma <= 0.0 || n == 0 {
        return;
    }
    let kernel = gaussian_kernel(sigma);
    let support = (kernel.len() / 2) as isize;
    let planes = (buf.channels as usize).min(3);
    for c in 0..planes {
        let base = c * n;
        let src = buf.data[base..base + n].to_vec();
        let mut tmp = vec![0f32; n];
        for y in 0..h {
            for x in 0..w {
                let mut acc = 0f32;
                for (ki, &kw) in kernel.iter().enumerate() {
                    let sx = clamp_index(x as isize + ki as isize - support, w);
                    acc += kw as f32 * src[y * w + sx] as f32;
                }
                tmp[y * w + x] = acc;
            }
        }
        for y in 0..h {
            for x in 0..w {
                let mut acc = 0f32;
                for (ki, &kw) in kernel.iter().enumerate() {
                    let sy = clamp_index(y as isize + ki as isize - support, h);
                    acc += kw as f32 * tmp[sy * w + x];
                }
                buf.data[base + y * w + x] = acc.round().clamp(0.0, 255.0) as u8;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sigma_floor() {
        assert_eq!(sigma_from_radius(0.0), 0.1);
        assert_eq!(sigma_from_radius(0.3), 0.1);
        assert!((sigma_from_radius(3.0) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn kernel_is_normalized_and_symmetric() {
        let k = gaussian_kernel(1.0);
        assert_eq!(k.len(), 7);
        let sum: f64 = k.iter().sum();
        assert!((sum - 1.0).abs() < 1e-12);
        assert!((k[0] - k[6]).abs() < 1e-12);
    }

    #[test]
    fn non_positive_sigma_is_single_sample() {
        assert_eq!(gaussian_kernel(0.0), vec![1.0]);
    }

    #[test]
    fn clamp_index_clamps_to_edges() {
        assert_eq!(clamp_index(-5, 4), 0);
        assert_eq!(clamp_index(2, 4), 2);
        assert_eq!(clamp_index(99, 4), 3);
        assert_eq!(clamp_index(0, 0), 0);
    }

    #[test]
    fn gaussian_blur_planes_is_noop_on_uniform_and_preserves_alpha() {
        let mut buf = PixelBuffer {
            width: 3,
            height: 1,
            channels: 4,
            data: vec![50, 50, 50, 100, 100, 100, 150, 150, 150, 10, 20, 30].into(),
        };
        gaussian_blur_planes(&mut buf, 1.0);
        assert_eq!(&buf.data[0..3], &[50, 50, 50]);
        assert_eq!(&buf.data[3..6], &[100, 100, 100]);
        assert_eq!(&buf.data[6..9], &[150, 150, 150]);
        assert_eq!(&buf.data[9..12], &[10, 20, 30], "alpha untouched");
    }
}
