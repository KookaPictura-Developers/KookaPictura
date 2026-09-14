//! Rec.601 luma weights shared by the filters.

pub const LUMA: [f64; 3] = [0.299, 0.587, 0.114];

pub fn luma(r: f64, g: f64, b: f64) -> f64 {
    LUMA[0] * r + LUMA[1] * g + LUMA[2] * b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weights_sum_to_one_and_mix_channels() {
        let sum: f64 = LUMA.iter().sum();
        assert!((sum - 1.0).abs() < 1e-12);
        assert!((luma(255.0, 255.0, 255.0) - 255.0).abs() < 1e-9);
        assert_eq!(luma(0.0, 0.0, 0.0), 0.0);
        assert!((luma(255.0, 0.0, 0.0) - 0.299 * 255.0).abs() < 1e-9);
    }
}
