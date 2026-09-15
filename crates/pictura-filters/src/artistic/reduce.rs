//! Shared reductions for the Artistic family (`m22-artistic-filters`):
//! posterization, Sobel edge magnitude, and float-to-byte clamping.

/// Round and clamp an arbitrary float to a valid 8-bit sample.
pub fn clamp_u8(v: f64) -> u8 {
    if !v.is_finite() {
        return 0;
    }
    v.round().clamp(0.0, 255.0) as u8
}

/// Quantize `val` to `levels` evenly spaced bands across 0..=255.
pub fn posterize(val: u8, levels: u8) -> u8 {
    if levels <= 1 {
        return val;
    }
    let denom = (levels - 1) as f64;
    let q = (val as f64 * denom / 255.0).round();
    clamp_u8(q * 255.0 / denom)
}

/// Un-normalized Sobel magnitude at `(x, y)` from a clamped luma accessor.
///
/// `luma_at(ax, ay)` receives absolute coordinates and is expected to apply the
/// buffer's clamp-to-edge policy.
pub fn edge_magnitude(luma_at: impl Fn(isize, isize) -> f64, x: usize, y: usize) -> f64 {
    let xi = x as isize;
    let yi = y as isize;
    let at = |dx: isize, dy: isize| luma_at(xi + dx, yi + dy);
    let gx = (at(1, -1) + 2.0 * at(1, 0) + at(1, 1)) - (at(-1, -1) + 2.0 * at(-1, 0) + at(-1, 1));
    let gy = (at(-1, 1) + 2.0 * at(0, 1) + at(1, 1)) - (at(-1, -1) + 2.0 * at(0, -1) + at(1, -1));
    (gx * gx + gy * gy).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn posterize_spans_the_ramp_and_is_idempotent() {
        assert_eq!(posterize(0, 2), 0);
        assert_eq!(posterize(255, 2), 255);
        assert_eq!(posterize(128, 2), 255);
        let bands8: Vec<u8> = (0..=255u8).map(|v| posterize(v, 8)).collect();
        let distinct: std::collections::BTreeSet<u8> = bands8.iter().copied().collect();
        assert_eq!(distinct.len(), 8);
        for v in bands8 {
            assert_eq!(posterize(v, 8), v);
        }
        // A single-level request is a pass-through so callers can disable it.
        assert_eq!(posterize(37, 1), 37);
    }

    #[test]
    fn clamp_u8_handles_non_finite() {
        assert_eq!(clamp_u8(-10.0), 0);
        assert_eq!(clamp_u8(999.0), 255);
        assert_eq!(clamp_u8(f64::NAN), 0);
    }

    #[test]
    fn edge_magnitude_is_zero_on_a_flat_field_and_large_on_a_step() {
        let flat = |_: isize, _: isize| 100.0;
        assert_eq!(edge_magnitude(flat, 3, 3), 0.0);
        let step = |ax: isize, _: isize| if ax < 4 { 0.0 } else { 255.0 };
        let mag = edge_magnitude(step, 4, 4);
        assert!(mag > 500.0, "step magnitude {mag}");
    }
}
