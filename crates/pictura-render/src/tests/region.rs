//! The brush path's region composite: exact against the full composite, and
//! fast beside shape layers and layer effects.

use super::*;
use crate::composite::composite_rgba_region;
use pictura_core::shape::{outline_in_box, ShapeKind, ShapeOptions};

/// A white document with an ellipse shape layer at (70, 20)-(110, 60) given a
/// 4 px outside stroke, and a half-covered pixel layer.
fn shapes_doc() -> Document {
    let mut d = doc(
        120,
        80,
        vec![
            solid(
                "Background",
                full(120, 80),
                (255, 255, 255),
                255,
                BlendMode::Normal,
                255,
            ),
            solid(
                "Half",
                rect(0, 0, 40, 60),
                (0, 0, 200),
                128,
                BlendMode::Multiply,
                255,
            ),
        ],
    );
    let ellipse = outline_in_box(
        ShapeOptions::new(ShapeKind::Ellipse, 0.0, 3),
        (70.0, 20.0, 40.0, 40.0),
    )
    .unwrap();
    let path = crate::add_shape_layer(&mut d, "", [255, 0, 0, 255], "Ellipse", &ellipse, None);
    let layer = crate::resolve_path_mut(&mut d, &path).unwrap();
    crate::set_shape_stroke(
        layer,
        Some(&crate::ShapeStroke {
            color: [0, 160, 0],
            width: 4,
            position: crate::StrokePosition::Outside,
        }),
    );
    d
}

fn assert_region_matches_full(d: &Document, x0: u32, y0: u32, w: u32, h: u32) {
    let full = composite_rgba(d);
    let region = composite_rgba_region(d, x0, y0, w, h);
    for y in 0..h {
        for x in 0..w {
            assert_eq!(
                px(&region, x, y),
                px(&full, x0 + x, y0 + y),
                "pixel ({}, {})",
                x0 + x,
                y0 + y
            );
        }
    }
}

#[test]
fn a_region_no_effect_reaches_composites_alone_and_exactly() {
    let d = shapes_doc();
    // Left of the stroked ellipse (its reach starts at x = 64): the fast path.
    assert_region_matches_full(&d, 5, 5, 50, 50);
}

#[test]
fn a_region_an_effect_reaches_still_matches_the_full_composite() {
    let d = shapes_doc();
    // Across the stroke band: the full-composite fallback.
    assert_region_matches_full(&d, 50, 10, 40, 40);
    let full = composite_rgba(&d);
    assert_eq!(
        rgb(&full, 68, 40),
        [0, 160, 0],
        "the outside stroke is drawn"
    );
}

#[test]
#[ignore = "profiling: prints the brush-dab region cost beside shape layers"]
fn shape_layer_region_profile() {
    let (w, h) = (2000u32, 1500u32);
    let mut d = Document::from_rgba("Background", w, h, &vec![255; (w * h * 4) as usize]);
    let time = |d: &Document, label: &str| {
        let t = std::time::Instant::now();
        for i in 0..20 {
            let _ = composite_rgba_region(d, 100 + i * 10, 100, 64, 64);
        }
        eprintln!("{label}: {:?} per 64x64 region", t.elapsed() / 20);
    };
    time(&d, "background only");
    for (k, kind) in [
        ShapeKind::Rectangle,
        ShapeKind::Ellipse,
        ShapeKind::CustomShape,
        ShapeKind::Polygon,
    ]
    .into_iter()
    .enumerate()
    {
        let mut o = ShapeOptions::new(kind, 0.0, 5);
        o.custom = 4;
        let s = outline_in_box(o, (100.0 + k as f64 * 300.0, 100.0, 400.0, 300.0)).unwrap();
        crate::add_shape_layer(&mut d, "", [255, 0, 0, 255], "S", &s, None);
    }
    time(&d, "4 shape layers");
    let t = std::time::Instant::now();
    let _ = composite_rgba(&d);
    eprintln!("full composite with 4 shapes: {:?}", t.elapsed());
    let path = format!("{}", d.layers.len() - 1);
    let layer = crate::resolve_path_mut(&mut d, &path).unwrap();
    crate::set_shape_stroke(
        layer,
        Some(&crate::ShapeStroke {
            color: [0, 0, 0],
            width: 3,
            position: crate::StrokePosition::Center,
        }),
    );
    time(&d, "4 shapes, one stroked away from the region");
}
