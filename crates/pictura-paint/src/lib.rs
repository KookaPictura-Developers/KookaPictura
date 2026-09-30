//! Brush engine primitives: stroke configuration, dab spacing, and tip coverage.

pub mod art_history;
pub mod eraser;
pub mod healing;
pub mod mixer;
pub mod pattern;
pub mod replace;
pub mod spacing;
pub mod stamp;
pub mod stroke;
pub mod tip;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintMode {
    Normal,
    Dissolve,
    Behind,
    Clear,
}

#[derive(Clone, Copy, Debug)]
pub struct StrokeSample {
    pub x: f32,
    pub y: f32,
    pub pressure: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct StrokeConfig {
    pub color: Rgba,
    pub background: Rgba,
    pub diameter: u32,
    pub hardness: u8,
    pub roundness: u8,
    pub angle_deg: i32,
    pub spacing: spacing::SpacingMode,
    pub opacity: u8,
    pub flow: u8,
    pub mode: PaintMode,
    pub aliased: bool,
    /// A hard square tip `diameter` wide (the Eraser's Block mode); roundness,
    /// angle, hardness, and aliasing do not apply.
    pub square: bool,
    pub flip_x: bool,
    pub flip_y: bool,
    pub auto_erase: bool,
}

impl StrokeConfig {
    /// Clamp every field: diameter 1..=5000, hardness/opacity/flow/roundness 0..=100, angle -180..=180.
    pub fn sanitized(&self) -> StrokeConfig {
        StrokeConfig {
            diameter: self.diameter.clamp(1, 5000),
            hardness: self.hardness.min(100),
            roundness: self.roundness.min(100),
            angle_deg: self.angle_deg.clamp(-180, 180),
            opacity: self.opacity.min(100),
            flow: self.flow.min(100),
            ..*self
        }
    }
}

impl Default for StrokeConfig {
    fn default() -> Self {
        Self {
            color: Rgba {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            },
            background: Rgba {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
            diameter: 12,
            hardness: 100,
            roundness: 100,
            angle_deg: 0,
            spacing: spacing::SpacingMode::Fixed(25),
            opacity: 100,
            flow: 100,
            mode: PaintMode::Normal,
            aliased: false,
            square: false,
            flip_x: false,
            flip_y: false,
            auto_erase: false,
        }
    }
}

pub use healing::HealStroke;
pub use stroke::{paint_stroke, PaintError, Stroke, StrokeKind, StrokeOutcome};
pub use tip::tip_coverage;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitized_clamps_out_of_range_fields() {
        let cfg = StrokeConfig {
            diameter: 99999,
            hardness: 200,
            roundness: 200,
            angle_deg: 400,
            opacity: 250,
            flow: 250,
            ..StrokeConfig::default()
        }
        .sanitized();
        assert_eq!(cfg.diameter, 5000);
        assert_eq!(cfg.hardness, 100);
        assert_eq!(cfg.roundness, 100);
        assert_eq!(cfg.opacity, 100);
        assert_eq!(cfg.flow, 100);
        assert_eq!(cfg.angle_deg, 180);
    }

    #[test]
    fn sanitized_clamps_low_and_negative() {
        let cfg = StrokeConfig {
            diameter: 0,
            angle_deg: -400,
            ..StrokeConfig::default()
        }
        .sanitized();
        assert_eq!(cfg.diameter, 1);
        assert_eq!(cfg.angle_deg, -180);
    }

    #[test]
    fn default_matches_cs6_basics() {
        let cfg = StrokeConfig::default();
        assert_eq!(cfg.diameter, 12);
        assert_eq!(cfg.hardness, 100);
        assert_eq!(cfg.roundness, 100);
        assert_eq!(cfg.spacing, spacing::SpacingMode::Fixed(25));
        assert_eq!(
            cfg.color,
            Rgba {
                r: 0,
                g: 0,
                b: 0,
                a: 255
            }
        );
        assert_eq!(
            cfg.background,
            Rgba {
                r: 255,
                g: 255,
                b: 255,
                a: 255
            }
        );
    }
}
