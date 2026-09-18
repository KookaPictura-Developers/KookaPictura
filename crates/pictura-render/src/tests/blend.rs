use super::*;

// --- one test per blend mode, Cb = 0.25, Cs = 0.75 ---------------------

#[test]
fn mode_normal() {
    assert_blend(BlendMode::Normal, [0.25; 3], [0.75; 3], [0.75; 3]);
}

#[test]
fn mode_dissolve_passes_source_color() {
    assert_blend(BlendMode::Dissolve, [0.25; 3], [0.75; 3], [0.75; 3]);
}

#[test]
fn mode_darken() {
    assert_blend(BlendMode::Darken, [0.25; 3], [0.75; 3], [0.25; 3]);
}

#[test]
fn mode_multiply() {
    assert_blend(BlendMode::Multiply, [0.25; 3], [0.75; 3], [0.1875; 3]);
}

#[test]
fn mode_color_burn() {
    assert_blend(BlendMode::ColorBurn, [0.25; 3], [0.75; 3], [0.0; 3]);
}

#[test]
fn mode_linear_burn() {
    assert_blend(BlendMode::LinearBurn, [0.25; 3], [0.75; 3], [0.0; 3]);
}

#[test]
fn mode_darker_color() {
    assert_blend(BlendMode::DarkerColor, [0.25; 3], [0.75; 3], [0.25; 3]);
}

#[test]
fn mode_lighten() {
    assert_blend(BlendMode::Lighten, [0.25; 3], [0.75; 3], [0.75; 3]);
}

#[test]
fn mode_screen() {
    assert_blend(BlendMode::Screen, [0.25; 3], [0.75; 3], [0.8125; 3]);
}

#[test]
fn mode_color_dodge() {
    assert_blend(BlendMode::ColorDodge, [0.25; 3], [0.75; 3], [1.0; 3]);
}

#[test]
fn mode_linear_dodge() {
    assert_blend(BlendMode::LinearDodge, [0.25; 3], [0.75; 3], [1.0; 3]);
}

#[test]
fn mode_lighter_color() {
    assert_blend(BlendMode::LighterColor, [0.25; 3], [0.75; 3], [0.75; 3]);
}

#[test]
fn mode_overlay() {
    assert_blend(BlendMode::Overlay, [0.25; 3], [0.75; 3], [0.375; 3]);
}

#[test]
fn mode_soft_light() {
    assert_blend(BlendMode::SoftLight, [0.25; 3], [0.75; 3], [0.375; 3]);
}

#[test]
fn mode_hard_light() {
    assert_blend(BlendMode::HardLight, [0.25; 3], [0.75; 3], [0.625; 3]);
}

#[test]
fn mode_vivid_light() {
    assert_blend(BlendMode::VividLight, [0.25; 3], [0.75; 3], [0.5; 3]);
}

#[test]
fn mode_linear_light() {
    assert_blend(BlendMode::LinearLight, [0.25; 3], [0.75; 3], [0.75; 3]);
}

#[test]
fn mode_pin_light() {
    assert_blend(BlendMode::PinLight, [0.25; 3], [0.75; 3], [0.5; 3]);
}

#[test]
fn mode_hard_mix() {
    assert_blend(BlendMode::HardMix, [0.25; 3], [0.75; 3], [1.0; 3]);
}

#[test]
fn mode_difference() {
    assert_blend(BlendMode::Difference, [0.25; 3], [0.75; 3], [0.5; 3]);
}

#[test]
fn mode_exclusion() {
    assert_blend(BlendMode::Exclusion, [0.25; 3], [0.75; 3], [0.625; 3]);
}

#[test]
fn mode_subtract() {
    assert_blend(BlendMode::Subtract, [0.25; 3], [0.75; 3], [0.0; 3]);
}

#[test]
fn mode_divide() {
    assert_blend(BlendMode::Divide, [0.25; 3], [0.75; 3], [1.0 / 3.0; 3]);
}

#[test]
fn mode_hue() {
    assert_blend(BlendMode::Hue, [0.25; 3], [0.75; 3], [0.25; 3]);
}

#[test]
fn mode_saturation() {
    assert_blend(BlendMode::Saturation, [0.25; 3], [0.75; 3], [0.25; 3]);
}

#[test]
fn mode_color() {
    assert_blend(BlendMode::Color, [0.25; 3], [0.75; 3], [0.25; 3]);
}

#[test]
fn mode_luminosity() {
    assert_blend(BlendMode::Luminosity, [0.25; 3], [0.75; 3], [0.75; 3]);
}

// --- chromatic non-separable check (hand-computed, W3C helpers) ---------

#[test]
fn nonseparable_modes_chromatic_w3c_values() {
    let cb = [0.1, 0.5, 0.3];
    let cs = [0.8, 0.4, 0.0];
    let cases = [
        (BlendMode::Hue, [0.52, 0.32, 0.12]),
        (BlendMode::Saturation, [0.0, 0.555_038_8, 0.277_519_4]),
        (BlendMode::Color, [0.601_680_7, 0.300_840_4, 0.0]),
        (BlendMode::Luminosity, [0.218, 0.618, 0.418]),
    ];
    for (mode, expected) in cases {
        let got = blend(mode, cb, cs);
        for i in 0..3 {
            assert!(
                (got[i] - expected[i]).abs() < 1e-4,
                "{mode:?}[{i}]: got {} want {}",
                got[i],
                expected[i]
            );
        }
    }
}
