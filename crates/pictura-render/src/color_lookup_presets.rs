//! The built-in Color Lookup presets.
//!
//! Ported from photorust's `Adjustment::color_lookup_tables`
//! (`core/src/filters/adjust.rs`), which models each look as three per-channel
//! curves rather than an Adobe `.look` file (those are Adobe's). The engine
//! samples 3-D `.CUBE` files, so each diagonal curve is baked into a
//! `LUT_3D_SIZE 16` grid here; `"None"` is the identity.

/// The named looks, in menu order. Index 0 is the identity.
pub const COLOR_LOOKUP_PRESETS: [&str; 7] = [
    "None",
    "Warm Contrast",
    "Cool Shadows",
    "Faded Film",
    "Bleach Bypass",
    "Crisp Warm",
    "Moonlight",
];

fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

/// One output channel's curve, taking an input level in `0..=1`.
type Curve = fn(f32) -> f32;

/// The three per-channel curves for a named look, or `None` for a name that is
/// not one. Each maps an input level in `0..=1`.
fn preset_curves(name: &str) -> Option<[Curve; 3]> {
    Some(match name {
        "None" => [|v| v, |v| v, |v| v],
        "Warm Contrast" => [
            |v| clamp01((v - 0.5) * 1.25 + 0.5).powf(0.92),
            |v| clamp01((v - 0.5) * 1.2 + 0.5),
            |v| clamp01((v - 0.5) * 1.2 + 0.5).powf(1.1),
        ],
        "Cool Shadows" => [
            |v| v.powf(1.12),
            |v| v.powf(1.02),
            |v| 0.06 + v.powf(0.9) * 0.94,
        ],
        "Faded Film" => [
            |v| 0.09 + clamp01((v - 0.5) * 0.82 + 0.5) * 0.88,
            |v| 0.08 + clamp01((v - 0.5) * 0.82 + 0.5) * 0.88,
            |v| 0.12 + clamp01((v - 0.5) * 0.8 + 0.5) * 0.84,
        ],
        "Bleach Bypass" => [
            |v| clamp01((v - 0.45) * 1.5 + 0.5),
            |v| clamp01((v - 0.45) * 1.45 + 0.5),
            |v| clamp01((v - 0.45) * 1.4 + 0.5),
        ],
        "Crisp Warm" => [
            |v| clamp01((v - 0.5) * 1.15 + 0.5).powf(0.85),
            |v| clamp01((v - 0.5) * 1.15 + 0.5).powf(0.98),
            |v| clamp01((v - 0.5) * 1.15 + 0.5).powf(1.18),
        ],
        "Moonlight" => [
            |v| v.powf(1.45) * 0.85,
            |v| v.powf(1.25) * 0.92,
            |v| 0.05 + v.powf(0.95) * 0.95,
        ],
        _ => return None,
    })
}

/// A `LUT_3D_SIZE 16` `.CUBE` for a named look, or `None` for an unknown name.
/// Each output channel depends only on its own input (a diagonal lookup), so a
/// `"None"` cube trisamples back to its input exactly.
pub fn preset_cube(name: &str) -> Option<Vec<u8>> {
    let curves = preset_curves(name)?;
    const SIZE: usize = 16;
    let mut s = format!("TITLE \"{name}\"\nLUT_3D_SIZE {SIZE}\n");
    let max = (SIZE - 1) as f32;
    for b in 0..SIZE {
        for g in 0..SIZE {
            for r in 0..SIZE {
                let out = [
                    curves[0](r as f32 / max),
                    curves[1](g as f32 / max),
                    curves[2](b as f32 / max),
                ];
                s.push_str(&format!("{} {} {}\n", out[0], out[1], out[2]));
            }
        }
    }
    Some(s.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pictura_adjust::{parse_cube, Adjustment, ColorLookupKind, ColorLookupParams};
    use pictura_core::PixelBuffer;

    #[test]
    fn every_preset_builds_a_valid_16_grid_cube() {
        for name in COLOR_LOOKUP_PRESETS {
            let cube = preset_cube(name).unwrap_or_else(|| panic!("{name} has a cube"));
            let lut = parse_cube(&cube).unwrap_or_else(|| panic!("{name} parses"));
            assert_eq!(lut.size, 16, "{name}");
            assert_eq!(lut.points.len(), 16 * 16 * 16, "{name}");
            assert!(lut.points.iter().flatten().all(|v| v.is_finite()), "{name}");
        }
        assert!(preset_cube("Nope").is_none());
    }

    #[test]
    fn none_is_the_identity() {
        let params = ColorLookupParams {
            kind: ColorLookupKind::ThreeDLut,
            lookup: parse_cube(&preset_cube("None").unwrap()),
        };
        let mut buf = PixelBuffer::new(3, 1, 3);
        buf.data = vec![10, 128, 250, 20, 130, 200, 30, 100, 60].into();
        let before = buf.data.to_vec();
        pictura_adjust::apply(&Adjustment::ColorLookup(params), &mut buf).unwrap();
        assert_eq!(buf.data.to_vec(), before);
    }
}
