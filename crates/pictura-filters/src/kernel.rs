//! Shared 1-D kernel helpers for the filter family.
//!
//! The UI radius maps to a Gaussian with the 3σ support convention from
//! `FILT-010`; every spatial filter builds its weights here.

use pictura_core::PixelBuffer;

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
            data: vec![50, 50, 50, 100, 100, 100, 150, 150, 150, 10, 20, 30],
        };
        gaussian_blur_planes(&mut buf, 1.0);
        assert_eq!(&buf.data[0..3], &[50, 50, 50]);
        assert_eq!(&buf.data[3..6], &[100, 100, 100]);
        assert_eq!(&buf.data[6..9], &[150, 150, 150]);
        assert_eq!(&buf.data[9..12], &[10, 20, 30], "alpha untouched");
    }
}
