//! Seeded noise fields for the Artistic family (`m22-artistic-filters`).

use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaCha8Rng,
};

/// White-noise field in `0..1`, one value per pixel, deterministic from `seed`.
///
/// ponytail: uncorrelated per-pixel noise; swap in an interpolated lattice if
/// grain needs visible clumping at large scales.
pub fn value_noise(width: usize, height: usize, seed: u64) -> Vec<f64> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let scale = 1.0 / (u32::MAX as f64 + 1.0);
    (0..width * height)
        .map(|_| rng.next_u32() as f64 * scale)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_is_seed_deterministic_and_in_range() {
        let a = value_noise(16, 9, 3);
        let b = value_noise(16, 9, 3);
        let c = value_noise(16, 9, 4);
        assert_eq!(a, b, "same seed must be bit-identical");
        assert_ne!(a, c, "different seed must differ");
        assert!(a.iter().all(|v| (0.0..1.0).contains(v)), "out of range");
    }
}
