//! Brush tip falloff.

use crate::StrokeConfig;

fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Tip alpha in 0..=1 at layer-local offset (dx, dy) from a dab center.
pub fn tip_coverage(cfg: &StrokeConfig, dx: f32, dy: f32) -> f32 {
    let theta = -(cfg.angle_deg as f32).to_radians();
    let (s, c) = theta.sin_cos();
    let rx = dx * c - dy * s;
    let ry = dx * s + dy * c;

    let fx = if cfg.flip_x { -rx } else { rx };
    let fy = if cfg.flip_y { -ry } else { ry };

    let radius = cfg.diameter as f32 * 0.5;
    let round = if cfg.roundness == 0 {
        1.0
    } else {
        cfg.roundness as f32 / 100.0
    };

    let u = fx.abs() / radius;
    let v = fy.abs() / (radius * round);
    let r = (u * u + v * v).sqrt();

    if cfg.aliased {
        return if r <= 1.0 { 1.0 } else { 0.0 };
    }

    let hardness = cfg.hardness as f32 / 100.0;
    let core = hardness * (1.0 - (1.0 / radius).min(0.5));
    if r <= core {
        1.0
    } else if r < 1.0 {
        let denom = (1.0 - core).max(f32::EPSILON);
        smoothstep((1.0 - r) / denom)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_is_fully_covered() {
        let cfg = StrokeConfig::default();
        assert_eq!(tip_coverage(&cfg, 0.0, 0.0), 1.0);
    }

    #[test]
    fn outside_radius_is_zero() {
        let cfg = StrokeConfig::default();
        let r = cfg.diameter as f32 * 0.5;
        assert_eq!(tip_coverage(&cfg, r * 2.0, 0.0), 0.0);
        assert_eq!(tip_coverage(&cfg, 0.0, r * 2.0), 0.0);
    }

    #[test]
    fn pencil_is_binary() {
        let cfg = StrokeConfig {
            aliased: true,
            ..StrokeConfig::default()
        };
        let r = cfg.diameter as f32 * 0.5;
        for i in 0..=20 {
            let d = r * (i as f32 / 10.0);
            let cov = tip_coverage(&cfg, d, 0.0);
            assert!(cov == 0.0 || cov == 1.0, "pencil produced {cov} at {d}");
        }
    }

    #[test]
    fn brush_has_soft_ramp() {
        let cfg = StrokeConfig {
            hardness: 100,
            ..StrokeConfig::default()
        };
        let r = cfg.diameter as f32 * 0.5;
        let cov = tip_coverage(&cfg, r * 0.9, 0.0);
        assert!(
            cov > 0.0 && cov < 1.0,
            "expected partial coverage, got {cov}"
        );
    }

    #[test]
    fn roundness_squashes_vertical_axis() {
        let cfg = StrokeConfig {
            roundness: 50,
            ..StrokeConfig::default()
        };
        let r = cfg.diameter as f32 * 0.5;
        let horizontal = tip_coverage(&cfg, r * 0.9, 0.0);
        let vertical = tip_coverage(&cfg, 0.0, r * 0.9);
        assert!(
            horizontal > vertical,
            "horizontal={horizontal} vertical={vertical}"
        );
    }
}
