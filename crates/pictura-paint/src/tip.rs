//! Brush tip falloff.

use crate::StrokeConfig;

fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Stroke-constant tip state: everything [`TipParams::coverage`] needs that
/// does not change between two dabs of one stroke, so a dab's pixel loop does
/// no trigonometry and no re-derivation of the profile.
#[derive(Clone, Copy)]
pub struct TipParams {
    sin: f32,
    cos: f32,
    flip_x: bool,
    flip_y: bool,
    radius: f32,
    round_radius: f32,
    aliased: bool,
    square: bool,
    core: f32,
    denom: f32,
}

impl TipParams {
    pub(crate) fn new(cfg: &StrokeConfig) -> Self {
        let theta = -(cfg.angle_deg as f32).to_radians();
        let (sin, cos) = theta.sin_cos();
        let radius = cfg.diameter as f32 * 0.5;
        let round = if cfg.roundness == 0 {
            1.0
        } else {
            cfg.roundness as f32 / 100.0
        };
        let hardness = cfg.hardness as f32 / 100.0;
        let core = hardness * (1.0 - (1.0 / radius).min(0.5));
        Self {
            sin,
            cos,
            flip_x: cfg.flip_x,
            flip_y: cfg.flip_y,
            radius,
            round_radius: radius * round,
            aliased: cfg.aliased,
            square: cfg.square,
            core,
            denom: (1.0 - core).max(f32::EPSILON),
        }
    }

    /// Tip alpha in 0..=1 at layer-local offset (dx, dy) from a dab center.
    pub fn coverage(&self, dx: f32, dy: f32) -> f32 {
        // A square tip (the Eraser's Block mode) is axis-aligned and ignores
        // rotation and hardness.
        if self.square {
            let half = self.radius;
            return if dx.abs() <= half && dy.abs() <= half {
                1.0
            } else {
                0.0
            };
        }
        let rx = dx * self.cos - dy * self.sin;
        let ry = dx * self.sin + dy * self.cos;

        let fx = if self.flip_x { -rx } else { rx };
        let fy = if self.flip_y { -ry } else { ry };

        let u = fx.abs() / self.radius;
        let v = fy.abs() / self.round_radius;
        let r = (u * u + v * v).sqrt();

        if self.aliased {
            return if r <= 1.0 { 1.0 } else { 0.0 };
        }

        if r <= self.core {
            1.0
        } else if r < 1.0 {
            smoothstep((1.0 - r) / self.denom)
        } else {
            0.0
        }
    }
}

/// Tip alpha in 0..=1 at layer-local offset (dx, dy) from a dab center.
///
/// Callers that stamp many pixels of one stroke should build a [`TipParams`]
/// once instead, so the profile is not re-derived per pixel.
pub fn tip_coverage(cfg: &StrokeConfig, dx: f32, dy: f32) -> f32 {
    TipParams::new(cfg).coverage(dx, dy)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The per-pixel profile re-derived from `cfg` every call: the reference
    /// the hoisted [`TipParams`] must stay bit-identical to.
    fn reference_coverage(cfg: &StrokeConfig, dx: f32, dy: f32) -> f32 {
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
            smoothstep((1.0 - r) / (1.0 - core).max(f32::EPSILON))
        } else {
            0.0
        }
    }

    #[test]
    fn hoisted_params_match_the_per_pixel_profile() {
        for cfg in [
            StrokeConfig::default(),
            StrokeConfig {
                hardness: 100,
                ..StrokeConfig::default()
            },
            StrokeConfig {
                hardness: 40,
                diameter: 500,
                roundness: 50,
                angle_deg: 37,
                ..StrokeConfig::default()
            },
            StrokeConfig {
                aliased: true,
                flip_x: true,
                flip_y: true,
                angle_deg: -90,
                ..StrokeConfig::default()
            },
            StrokeConfig {
                roundness: 0,
                diameter: 1,
                ..StrokeConfig::default()
            },
        ] {
            let params = TipParams::new(&cfg);
            let r = cfg.diameter as f32 * 0.5;
            for i in -12..=12 {
                for j in -12..=12 {
                    let dx = r * i as f32 / 8.0;
                    let dy = r * j as f32 / 8.0;
                    let expected = reference_coverage(&cfg, dx, dy);
                    assert_eq!(
                        params.coverage(dx, dy),
                        expected,
                        "diameter={} roundness={} angle={} at ({dx}, {dy})",
                        cfg.diameter,
                        cfg.roundness,
                        cfg.angle_deg
                    );
                }
            }
        }
    }

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
    fn square_tip_fills_its_corners() {
        let cfg = StrokeConfig {
            square: true,
            diameter: 10,
            ..StrokeConfig::default()
        };
        assert_eq!(tip_coverage(&cfg, 4.5, 4.5), 1.0);
        assert_eq!(tip_coverage(&cfg, 5.5, 0.0), 0.0);
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
