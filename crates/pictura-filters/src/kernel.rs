//! Shared 1-D kernel helpers for the filter family.
//!
//! The UI radius maps to a Gaussian with the 3σ support convention from
//! `FILT-010`; every spatial filter builds its weights here.

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
}
