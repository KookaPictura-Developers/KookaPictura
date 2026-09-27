//! Healing engine checks: the fills, the mode/type matrix, determinism, the
//! refusal contracts, and the layer-level wrapper. Adapted from photorust's
//! `healing.rs` tests onto this crate's `RgbaImage`/`Document` API.

use super::*;
use pictura_core::{
    BitDepth, BlendMode, Channel, ColorLabel, ColorMode, Document, Layer, LockFlags,
};

const BASE: [u8; 4] = [100, 120, 140, 255];

fn flat(w: i32, h: i32, px: [u8; 4]) -> RgbaImage {
    RgbaImage {
        width: w,
        height: h,
        data: vec![px; (w * h) as usize],
    }
}

fn rect(w: i32, h: i32) -> PsdRect {
    PsdRect {
        top: 0,
        left: 0,
        bottom: h,
        right: w,
    }
}

/// Coverage of a disc centred in a `w`×`h` region.
fn disc(region: PsdRect, radius: f32) -> Vec<f32> {
    let (cx, cy) = (
        (region.left + region.right) as f32 / 2.0,
        (region.top + region.bottom) as f32 / 2.0,
    );
    (region.top..region.bottom)
        .flat_map(|y| (region.left..region.right).map(move |x| (x, y)))
        .map(|(x, y)| {
            let d = ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt();
            (radius - d + 0.5).clamp(0.0, 1.0)
        })
        .collect()
}

fn all_ones(region: PsdRect) -> Vec<f32> {
    vec![1.0; (region.width() * region.height()) as usize]
}

fn px(img: &RgbaImage, x: i32, y: i32) -> [u8; 4] {
    img.data[(y * img.width + x) as usize]
}

#[test]
fn a_blemish_on_a_flat_field_disappears() {
    let mut img = flat(32, 32, BASE);
    for y in 12..20 {
        for x in 12..20 {
            img.set(x, y, [255, 255, 255, 255]);
        }
    }
    let region = PsdRect {
        top: 12,
        left: 12,
        bottom: 20,
        right: 20,
    };
    let done = heal_region(
        &mut img,
        region,
        &all_ones(region),
        HealMode::ProximityMatch,
    );
    assert_eq!(done, Some(region));
    // Every healed pixel returns to the flat field; the blemish is gone.
    for y in 12..20 {
        for x in 12..20 {
            assert_eq!(px(&img, x, y), BASE, "blemish survived at ({x}, {y})");
        }
    }
}

#[test]
fn proximity_match_continues_a_gradient() {
    let (w, h) = (48, 16);
    let mut img = flat(w, h, BASE);
    for y in 0..h {
        for x in 0..w {
            let v = (x * 255 / (w - 1)) as u8;
            img.set(x, y, [v, v, v, 255]);
        }
    }
    let original = img.clone();
    let region = PsdRect {
        top: 4,
        left: 20,
        bottom: 12,
        right: 28,
    };
    heal_region(
        &mut img,
        region,
        &all_ones(region),
        HealMode::ProximityMatch,
    );
    // The fill follows the horizontal ramp: monotone in x, close to the original.
    for y in 4..12 {
        for x in 21..28 {
            let here = px(&img, x, y)[0] as i32;
            let left = px(&img, x - 1, y)[0] as i32;
            assert!(here >= left, "gradient dipped at ({x}, {y})");
            let want = px(&original, x, y)[0] as i32;
            assert!((here - want).abs() <= 2, "gradient off at ({x}, {y})");
        }
    }
}

#[test]
fn content_aware_carries_an_edge_across_the_hole() {
    // A vertical edge at x=24; the hole straddles it, so the fill must keep a
    // brightness step rather than flatten it.
    let (w, h) = (48, 32);
    let mut img = flat(w, h, [20, 20, 20, 255]);
    for y in 0..h {
        for x in 24..w {
            img.set(x, y, [220, 220, 220, 255]);
        }
    }
    let region = PsdRect {
        top: 12,
        left: 20,
        bottom: 20,
        right: 28,
    };
    heal_region(&mut img, region, &all_ones(region), HealMode::ContentAware);
    // Left of the edge stays dark, right stays light: the step is preserved.
    let dark = px(&img, 22, 16)[0] as i32;
    let light = px(&img, 26, 16)[0] as i32;
    assert!(
        light - dark > 100,
        "edge collapsed: dark={dark} light={light}"
    );
}

#[test]
fn create_texture_adds_grain_where_the_surroundings_have_grain() {
    let (w, h) = (40, 40);
    let mut img = flat(w, h, BASE);
    // Noisy surroundings, deterministic noise.
    let mut seed = 1u32;
    for y in 0..h {
        for x in 0..w {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let n = (seed >> 24) as i32 - 128;
            let v = (120 + n / 4).clamp(0, 255) as u8;
            img.set(x, y, [v, v, v, 255]);
        }
    }
    let region = PsdRect {
        top: 16,
        left: 16,
        bottom: 24,
        right: 24,
    };
    heal_region(&mut img, region, &all_ones(region), HealMode::CreateTexture);
    // The fill is not a flat block: it varies like its surroundings.
    let mut min = 255u8;
    let mut max = 0u8;
    for y in 16..24 {
        for x in 16..24 {
            let v = px(&img, x, y)[0];
            min = min.min(v);
            max = max.max(v);
        }
    }
    assert!(max - min >= 4, "texture fill is flat: {}..{}", min, max);
}

#[test]
fn healing_leaves_pixels_outside_the_brush_alone() {
    let mut img = flat(32, 32, BASE);
    img.set(11, 16, [1, 2, 3, 255]);
    img.set(20, 16, [4, 5, 6, 255]);
    let region = PsdRect {
        top: 12,
        left: 12,
        bottom: 20,
        right: 20,
    };
    heal_region(
        &mut img,
        region,
        &all_ones(region),
        HealMode::ProximityMatch,
    );
    assert_eq!(px(&img, 11, 16), [1, 2, 3, 255]);
    assert_eq!(px(&img, 20, 16), [4, 5, 6, 255]);
}

#[test]
fn partial_coverage_blends_rather_than_replacing() {
    let mut img = flat(32, 32, [0, 0, 0, 255]);
    // A white 4×4 patch in the middle of black; half coverage keeps it grey.
    for y in 14..18 {
        for x in 14..18 {
            img.set(x, y, [255, 255, 255, 255]);
        }
    }
    let region = PsdRect {
        top: 14,
        left: 14,
        bottom: 18,
        right: 18,
    };
    heal_region(&mut img, region, &[0.5; 16], HealMode::ProximityMatch);
    let v = px(&img, 16, 16)[0];
    assert!((1..=254).contains(&v), "expected a blend, got {v}");
}

#[test]
fn an_empty_coverage_mask_does_nothing() {
    let mut img = flat(32, 32, BASE);
    let before = img.clone();
    let region = PsdRect {
        top: 12,
        left: 12,
        bottom: 20,
        right: 20,
    };
    let done = heal_region(&mut img, region, &vec![0.0; 64], HealMode::ProximityMatch);
    assert_eq!(done, None);
    assert_eq!(img, before);
}

#[test]
fn a_hole_with_no_surroundings_is_refused() {
    // Coverage is the whole (canvas-sized) region: no boundary to read from.
    let mut img = flat(16, 16, BASE);
    let before = img.clone();
    let region = rect(16, 16);
    let done = heal_region(
        &mut img,
        region,
        &all_ones(region),
        HealMode::ProximityMatch,
    );
    assert_eq!(done, None);
    assert_eq!(img, before);
}

#[test]
fn healing_is_deterministic() {
    let (w, h) = (40, 40);
    let mut a = flat(w, h, BASE);
    for y in 0..h {
        for x in 0..w {
            let v = ((x * 7 + y * 13) % 256) as u8;
            a.set(x, y, [v, v, v, 255]);
        }
    }
    let mut b = a.clone();
    let region = PsdRect {
        top: 12,
        left: 12,
        bottom: 28,
        right: 28,
    };
    let cov = disc(region, 6.0);
    // Content-Aware is seeded: the same stroke must heal the same way twice.
    heal_region(&mut a, region, &cov, HealMode::ContentAware);
    heal_region(&mut b, region, &cov, HealMode::ContentAware);
    assert_eq!(a, b);
}

#[test]
fn a_zero_offset_clone_is_a_no_op() {
    let mut img = flat(32, 32, BASE);
    let before = img.clone();
    let region = PsdRect {
        top: 12,
        left: 12,
        bottom: 20,
        right: 20,
    };
    let done = clone_region(&mut img, region, &all_ones(region), (0, 0), Transfer::Full);
    assert_eq!(done, None);
    assert_eq!(img, before);
}

#[test]
fn cloning_carries_the_source_texture_over() {
    // Left half dark, right half light. Clone a light patch (offset +8 in x)
    // into a hole on the dark side: the solve keeps the destination's low
    // level and takes the source's texture.
    let (w, h) = (48, 48);
    let mut img = flat(w, h, [30, 30, 30, 255]);
    for y in 0..h {
        for x in 24..w {
            let v = 180 + ((x - 24) as u8 % 20);
            img.set(x, y, [v, v, v, 255]);
        }
    }
    let region = PsdRect {
        top: 20,
        left: 8,
        bottom: 28,
        right: 16,
    };
    clone_region(&mut img, region, &all_ones(region), (16, 0), Transfer::Full);
    // The transplanted patch is near the destination's level (dark), not the
    // source's absolute level, because the boundary is dark.
    let v = px(&img, 12, 24)[0] as i32;
    assert!(v < 120, "clone ignored the destination's level: {v}");
}

#[test]
fn texture_only_clone_keeps_destination_colour() {
    let (w, h) = (48, 48);
    let mut img = flat(w, h, [40, 60, 80, 255]);
    // Source region (offset +16): a saturated red patch.
    for y in 0..h {
        for x in 32..40 {
            img.set(x, y, [255, 0, 0, 255]);
        }
    }
    let region = PsdRect {
        top: 20,
        left: 16,
        bottom: 28,
        right: 24,
    };
    clone_region(
        &mut img,
        region,
        &all_ones(region),
        (16, 0),
        Transfer::TextureOnly,
    );
    // Hue stays with the destination: red does not dominate.
    let p = px(&img, 20, 24);
    assert!(
        p[0] as i32 - (p[2] as i32) < 60,
        "TextureOnly leaked the source colour: {p:?}"
    );
}

#[test]
fn mode_round_trips_through_its_integer() {
    assert_eq!(HealMode::from_i32(0), Some(HealMode::ProximityMatch));
    assert_eq!(HealMode::from_i32(1), Some(HealMode::CreateTexture));
    assert_eq!(HealMode::from_i32(2), Some(HealMode::ContentAware));
    assert_eq!(HealMode::from_i32(3), None);
    assert_eq!(HealMode::default(), HealMode::ProximityMatch);
}

fn layer_doc(w: u32, h: u32, rgba: [u8; 4]) -> Document {
    let mut doc = Document::new(w, h, ColorMode::Rgb, BitDepth::Eight);
    let n = (w * h) as usize;
    doc.layers.push(Layer {
        name: "px".into(),
        rect: rect(w as i32, h as i32),
        blend: BlendMode::Normal,
        opacity: 255,
        fill: 255,
        lock: LockFlags::default(),
        color: ColorLabel::None,
        clipping: false,
        visible: true,
        channels: vec![
            Channel {
                id: 0,
                data: vec![rgba[0]; n],
            },
            Channel {
                id: 1,
                data: vec![rgba[1]; n],
            },
            Channel {
                id: 2,
                data: vec![rgba[2]; n],
            },
            Channel {
                id: -1,
                data: vec![rgba[3]; n],
            },
        ],
        ..Layer::default()
    });
    doc
}

fn channel(doc: &Document, layer: usize, id: i16, x: i32, y: i32) -> u8 {
    let l = &doc.layers[layer];
    let i = (y * l.rect.width() + x) as usize;
    l.channels.iter().find(|c| c.id == id).unwrap().data[i]
}

#[test]
fn heal_layer_writes_the_pixels_and_returns_a_document_rect() {
    let mut doc = layer_doc(32, 32, BASE);
    // A blemish on a flat field.
    {
        let l = &mut doc.layers[0];
        for y in 12..20 {
            for x in 12..20 {
                let i = (y * 32 + x) as usize;
                for c in &mut l.channels[..3] {
                    c.data[i] = 255;
                }
            }
        }
    }
    let region = PsdRect {
        top: 12,
        left: 12,
        bottom: 20,
        right: 20,
    };
    let done = heal_layer(
        &mut doc,
        "0",
        region,
        &all_ones(region),
        |img, local, cov| heal_region(img, local, cov, HealMode::ProximityMatch),
    )
    .unwrap();
    assert_eq!(done, Some(region));
    assert_eq!(channel(&doc, 0, 0, 16, 16), BASE[0]);
    assert_eq!(channel(&doc, 0, 1, 16, 16), BASE[1]);
    assert_eq!(channel(&doc, 0, 2, 16, 16), BASE[2]);
}

#[test]
fn heal_layer_refuses_a_group_or_bad_path() {
    let mut doc = layer_doc(16, 16, BASE);
    doc.layers.push(Layer {
        name: "group".into(),
        is_group: true,
        ..Layer::default()
    });
    let region = rect(4, 4);
    let op = |img: &mut RgbaImage, local, cov: &[f32]| {
        heal_region(img, local, cov, HealMode::ProximityMatch)
    };
    assert_eq!(
        heal_layer(&mut doc, "9", region, &all_ones(region), op),
        Err(HealError::NoRasterLayer)
    );
    assert_eq!(
        heal_layer(&mut doc, "nope", region, &all_ones(region), op),
        Err(HealError::NoRasterLayer)
    );
    assert_eq!(
        heal_layer(&mut doc, "1", region, &all_ones(region), op),
        Err(HealError::NoRasterLayer)
    );
}

#[test]
fn heal_layer_refuses_a_pixel_locked_layer() {
    let mut doc = layer_doc(16, 16, BASE);
    doc.layers[0].lock = doc.layers[0].lock.with(LockFlags::PIXELS, true);
    let region = rect(4, 4);
    assert_eq!(
        heal_layer(
            &mut doc,
            "0",
            region,
            &all_ones(region),
            |img, local, cov| { heal_region(img, local, cov, HealMode::ProximityMatch) }
        ),
        Err(HealError::Locked)
    );
}

#[test]
fn a_transparency_lock_keeps_alpha() {
    let mut doc = layer_doc(16, 16, [BASE[0], BASE[1], BASE[2], 40]);
    doc.layers[0].lock = doc.layers[0].lock.with(LockFlags::TRANSPARENCY, true);
    let region = rect(4, 4);
    let done = heal_layer(
        &mut doc,
        "0",
        region,
        &all_ones(region),
        |img, local, cov| {
            // Pretend the heal changed alpha; the wrapper must restore it.
            let r = heal_region(img, local, cov, HealMode::ProximityMatch);
            img.data.iter_mut().for_each(|p| p[3] = 0);
            r
        },
    )
    .unwrap();
    assert!(done.is_some());
    assert_eq!(channel(&doc, 0, -1, 8, 8), 40, "alpha was not preserved");
}

#[test]
fn a_spot_gesture_heals_under_its_dab() {
    let mut doc = layer_doc(32, 32, BASE);
    // A white blemish the brush will cover.
    for y in 14..18 {
        for x in 14..18 {
            let i = (y * 32 + x) as usize;
            for c in &mut doc.layers[0].channels[..3] {
                c.data[i] = 255;
            }
        }
    }
    let mut stroke = HealStroke::begin(&doc, "0", 8, 100).unwrap();
    assert_eq!(stroke.layer_rect(), rect(32, 32));
    assert!(stroke.dab(16.0, 16.0).is_some());
    assert!(stroke.is_painted());
    let outcome = stroke
        .commit(&mut doc, HealMode::ProximityMatch, None, Transfer::Full)
        .unwrap()
        .expect("a populated mask commits");
    assert_eq!(
        outcome.document.layers[0].channels[0].data[(16 * 32 + 16) as usize],
        BASE[0]
    );
}

#[test]
fn an_empty_gesture_commits_nothing() {
    let mut doc = layer_doc(16, 16, BASE);
    let stroke = HealStroke::begin(&doc, "0", 8, 100).unwrap();
    assert!(!stroke.is_painted());
    assert!(stroke
        .commit(&mut doc, HealMode::ProximityMatch, None, Transfer::Full)
        .unwrap()
        .is_none());
}

#[test]
fn a_clone_gesture_uses_its_source_offset() {
    let (w, h) = (48, 48);
    let mut doc = layer_doc(w, h, [30, 30, 30, 255]);
    // Light band on the right for the source offset to sample.
    for y in 0..h {
        for x in 24..w {
            let i = (y * w + x) as usize;
            for c in &mut doc.layers[0].channels[..3] {
                c.data[i] = 180;
            }
        }
    }
    let mut stroke = HealStroke::begin(&doc, "0", 8, 100).unwrap();
    stroke.dab(12.0, 24.0);
    let outcome = stroke
        .commit(
            &mut doc,
            HealMode::ProximityMatch,
            Some((16, 0)),
            Transfer::TextureOnly,
        )
        .unwrap()
        .expect("a populated mask commits");
    // TextureOnly keeps the destination's dark level, not the source's light.
    let v = outcome.document.layers[0].channels[0].data[(24 * w + 12) as usize];
    assert!(v < 120, "clone ignored the destination's level: {v}");
}
