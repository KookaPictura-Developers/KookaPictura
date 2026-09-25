use super::*;
use crate::composite::{Canvas, Px};
use pictura_adjust::{Adjustment, AutoKind};
use pictura_core::Samples;

fn levels_layer(output_white: u16, opacity: u8) -> Layer {
    let mut data = 2u16.to_be_bytes().to_vec();
    data.extend_from_slice(&0u16.to_be_bytes());
    data.extend_from_slice(&255u16.to_be_bytes());
    data.extend_from_slice(&0u16.to_be_bytes());
    data.extend_from_slice(&output_white.to_be_bytes());
    data.extend_from_slice(&100u16.to_be_bytes());
    adjustment_layer("levels", *b"levl", data, opacity, None)
}

fn base_layer() -> Layer {
    solid(
        "base",
        full(4, 4),
        (100, 140, 200),
        255,
        BlendMode::Normal,
        255,
    )
}

/// A depth-16 document with a covering pixel layer plus the Levels adjustment
/// layer at `opacity`.
fn levels_opacity_doc(output_white: u16, opacity: u8) -> Document {
    let mut d = doc(
        4,
        4,
        vec![base_layer(), levels_layer(output_white, opacity)],
    );
    d.source_depth = Some(BitDepth::Sixteen);
    d
}

fn native_u16(d: &Document) -> Vec<u16> {
    match composite_native(d) {
        Some(Samples::U16(v)) => v,
        other => panic!("expected a u16 native composite, got {other:?}"),
    }
}

#[test]
fn depth16_native_composite_keeps_sub_8bit_adjustment_samples() {
    let d = levels_opacity_doc(200, 255);

    let plane = (d.width * d.height) as usize;
    let v = native_u16(&d);
    assert_eq!(v.len(), plane * 4);

    let rgba = composite_rgba(&d);
    let differs = (0..3).any(|c| {
        let eight = &rgba.data[c * plane..(c + 1) * plane];
        (0..plane).any(|i| v[c * plane + i] != eight[i] as u16 * 257)
    });
    assert!(
        differs,
        "native adjustment must not collapse to the 8-bit widen"
    );
}

#[test]
fn adjustment_opacity_is_honored_at_native_depth() {
    let mut base_doc = doc(4, 4, vec![base_layer()]);
    base_doc.source_depth = Some(BitDepth::Sixteen);
    let base = native_u16(&base_doc);
    let full = native_u16(&levels_opacity_doc(200, 255));
    let mid = native_u16(&levels_opacity_doc(200, 128));

    let plane = (base_doc.width * base_doc.height) as usize;
    for c in 0..3 {
        for i in 0..plane {
            let at = c * plane + i;
            let (b, f, m) = (base[at], full[at], mid[at]);
            assert_ne!(
                m, f,
                "opacity 128 must differ from opacity 255 (channel {c}, px {i})"
            );
            assert_ne!(m, b, "opacity 128 must differ from the unadjusted backdrop");
            assert!(
                b.min(f) <= m && m <= b.max(f),
                "opacity 128 must land between backdrop {b} and full {f} (got {m})"
            );
        }
    }
}

#[test]
fn native_opacity_255_matches_the_8bit_composite_within_one_lsb() {
    let d = levels_opacity_doc(200, 255);
    let narrowed = composite_native(&d)
        .expect("depth-16 composite")
        .narrow_to_u8();
    let rgba = composite_rgba(&d);
    assert_eq!(narrowed.len(), rgba.data.len());
    for (i, (native, eight)) in narrowed.iter().zip(rgba.data.iter()).enumerate() {
        assert!(
            (*native as i32 - *eight as i32).abs() <= 1,
            "sample {i}: native {native} vs 8-bit {eight}"
        );
    }
}

#[test]
fn composite_native_is_none_without_depth_and_f32_at_32() {
    let d = doc(
        2,
        2,
        vec![solid(
            "base",
            full(2, 2),
            (10, 20, 30),
            255,
            BlendMode::Normal,
            255,
        )],
    );
    assert!(composite_native(&d).is_none());

    let mut d32 = d.clone();
    d32.source_depth = Some(BitDepth::ThirtyTwo);
    match composite_native(&d32) {
        Some(Samples::F32(v)) => assert_eq!(v.len(), 2 * 2 * 4),
        other => panic!("depth-32 document must yield f32 samples, got {other:?}"),
    }
}

#[test]
fn auto_adjustment_on_depth16_falls_back_to_the_8bit_apply() {
    let mut d = doc(4, 4, Vec::new());
    d.source_depth = Some(BitDepth::Sixteen);

    let mut canvas = Canvas::new(4, 4);
    for (i, p) in canvas.px.iter_mut().enumerate() {
        let v = (80.0 + i as f32 * 6.0) / 255.0;
        *p = Px {
            r: v,
            g: v,
            b: v,
            a: 1.0,
        };
    }
    let before: Vec<f32> = canvas.px.iter().map(|p| p.r).collect();

    let layer = adjustment_layer("auto", *b"auto", Vec::new(), 255, None);
    crate::composite_native::composite_adjustment(
        &mut canvas,
        &layer,
        &d,
        &Adjustment::Auto(AutoKind::Color),
    );

    assert!(
        canvas
            .px
            .iter()
            .zip(&before)
            .any(|(p, b)| (p.r - b).abs() > 1.0 / 255.0),
        "auto is outside the native set, so the 8-bit apply must still run"
    );
}
