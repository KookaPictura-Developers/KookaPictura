//! A Gaussian blur approximated by three box passes: its cost is independent
//! of the radius, which a neighbourhood operator with a radius in the hundreds
//! (HDR Toning's Local Adaptation, Shadows/Highlights) needs to stay
//! interactive.
//!
//! ponytail: three equal boxes, not the exact Gaussian; the profile differs
//! from a true kernel by a few percent at the tails.

/// Blur the `width` × `height` row-major `plane` in place to approximate a
/// Gaussian of standard deviation `sigma`, edges clamped.
pub fn approx_gaussian(plane: &mut [f64], width: usize, height: usize, sigma: f64) {
    if width == 0 || height == 0 || plane.len() != width * height || sigma.is_nan() || sigma <= 0.0
    {
        return;
    }
    // Three boxes of width w have variance 3 (w² - 1) / 12 = sigma².
    let box_width = (4.0 * sigma * sigma + 1.0).sqrt();
    let half = ((box_width - 1.0) / 2.0).round().max(1.0) as usize;
    let mut scratch = Vec::new();
    let mut column = vec![0.0; height];
    for _ in 0..3 {
        for row in plane.chunks_exact_mut(width) {
            box_blur(row, half, &mut scratch);
        }
        for x in 0..width {
            for (y, v) in column.iter_mut().enumerate() {
                *v = plane[y * width + x];
            }
            box_blur(&mut column, half, &mut scratch);
            for (y, v) in column.iter().enumerate() {
                plane[y * width + x] = *v;
            }
        }
    }
}

/// A running-sum box blur of `values` over `2 * half + 1` samples, edges
/// clamped.
fn box_blur(values: &mut [f64], half: usize, scratch: &mut Vec<f64>) {
    let n = values.len();
    if n < 2 {
        return;
    }
    scratch.clear();
    scratch.extend_from_slice(values);
    let at = |i: isize| scratch[i.clamp(0, n as isize - 1) as usize];
    let width = (2 * half + 1) as f64;
    let half = half as isize;
    let mut sum: f64 = (-half..=half).map(at).sum();
    for (i, v) in values.iter_mut().enumerate() {
        *v = sum / width;
        let i = i as isize;
        sum += at(i + half + 1) - at(i - half);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_flat_plane_stays_flat_and_a_spike_spreads_like_a_gaussian() {
        let mut flat = vec![0.4; 64];
        approx_gaussian(&mut flat, 8, 8, 5.0);
        assert!(flat.iter().all(|v| (v - 0.4).abs() < 1e-12));

        let (w, sigma) = (201usize, 10.0);
        let mut spike = vec![0.0; w * w];
        let centre = 100 * w + 100;
        spike[centre] = 1.0;
        approx_gaussian(&mut spike, w, w, sigma);
        let total: f64 = spike.iter().sum();
        assert!((total - 1.0).abs() < 1e-9, "mass is kept: {total}");
        // The variance along a row through the centre matches sigma².
        let row: Vec<f64> = (0..w).map(|x| spike[100 * w + x]).collect();
        let mass: f64 = row.iter().sum();
        let variance: f64 = row
            .iter()
            .enumerate()
            .map(|(x, v)| v * (x as f64 - 100.0).powi(2))
            .sum::<f64>()
            / mass;
        assert!(
            (variance.sqrt() - sigma).abs() < 0.6,
            "sigma {}",
            variance.sqrt()
        );
    }
}
