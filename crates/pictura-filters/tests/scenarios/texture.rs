//! Spec scenarios from the replaced `texture.rs` unit tests, run through
//! `apply` against the ported photorust engine.

use super::shims::*;
use pictura_core::PixelBuffer;
use pictura_filters::*;

use std::collections::BTreeSet;

fn gradient(w: u32, h: u32, channels: u8) -> PixelBuffer {
    let n = (w * h) as usize;
    let mut data = vec![0u8; n * channels as usize];
    let denom = (w.max(2) - 1) as f64;
    for y in 0..h as usize {
        for x in 0..w as usize {
            let i = y * w as usize + x;
            let step = if x < (w / 2) as usize { 0.0 } else { 100.0 };
            let v = (255.0 * x as f64 / denom + step).min(255.0).round() as u8;
            for c in 0..(channels as usize).min(3) {
                data[c * n + i] = v;
            }
            if channels == 4 {
                data[3 * n + i] = 137;
            }
        }
    }
    PixelBuffer {
        width: w,
        height: h,
        channels,
        data: data.into(),
    }
}

fn alpha_plane(buf: &PixelBuffer) -> Vec<u8> {
    let n = buf.pixel_count();
    buf.data[3 * n..4 * n].to_vec()
}

fn mean_luma(buf: &PixelBuffer) -> f64 {
    let n = buf.pixel_count();
    (0..n)
        .map(|i| {
            luma(
                buf.data[i] as f64,
                buf.data[n + i] as f64,
                buf.data[2 * n + i] as f64,
            )
        })
        .sum::<f64>()
        / n as f64
}

#[test]
fn each_texture_filter_changes_the_colour_planes() {
    let base = gradient(32, 8, 3);

    let mut b = base.clone();
    craquelure(&mut b, 20, 6, 9).unwrap();
    assert_ne!(b.data, base.data, "craquelure must change the image");

    let mut b = base.clone();
    grain(&mut b, 60, 50, GrainType::Regular, [255, 255, 255], 7).unwrap();
    assert_ne!(b.data, base.data, "grain must change the image");

    let mut b = base.clone();
    mosaic_tiles(&mut b, 8, 3, 5, 7).unwrap();
    assert_ne!(b.data, base.data, "mosaic tiles must change the image");

    let mut b = base.clone();
    patchwork(&mut b, 4, 10, 7).unwrap();
    assert_ne!(b.data, base.data, "patchwork must change the image");

    let mut b = base.clone();
    stained_glass(&mut b, 8, 3, 5, [0, 0, 0], 7).unwrap();
    assert_ne!(b.data, base.data, "stained glass must change the image");

    let mut b = base.clone();
    texturizer(&mut b, TextureOptions::default()).unwrap();
    assert_ne!(b.data, base.data, "texturizer must change the image");
}

#[test]
fn every_texture_filter_preserves_alpha_through_apply() {
    let base = gradient(32, 8, 4);
    let n = base.pixel_count();
    let alpha = alpha_plane(&base);
    let filters = [
        Filter::Craquelure {
            crack_spacing: 20,
            crack_depth: 6,
            crack_brightness: 9,
        },
        Filter::Grain {
            intensity: 60,
            contrast: 50,
            grain_type: GrainType::Clumped,
            background: [10, 20, 30],
            seed: 7,
        },
        Filter::MosaicTiles {
            tile_size: 8,
            grout_width: 3,
            lighten_grout: 5,
            seed: 7,
        },
        Filter::Patchwork {
            square_size: 4,
            relief: 10,
            seed: 7,
        },
        Filter::StainedGlass {
            cell_size: 8,
            border_thickness: 3,
            light_intensity: 5,
            foreground: [0, 0, 0],
            seed: 7,
        },
        Filter::Texturizer {
            texture: TextureOptions::default(),
        },
    ];
    for filter in &filters {
        let mut b = base.clone();
        apply(filter, &mut b).unwrap();
        assert_ne!(
            b.data[..3 * n],
            base.data[..3 * n],
            "{filter:?} must change colours"
        );
        assert_eq!(alpha_plane(&b), alpha, "{filter:?} must preserve alpha");
    }
}

#[test]
fn boundaries_are_accepted_and_out_of_range_rejected() {
    let mut b = gradient(8, 8, 3);

    assert!(craquelure(&mut b, 2, 0, 0).is_ok());
    assert!(craquelure(&mut b, 100, 10, 10).is_ok());
    assert!(craquelure(&mut b, 1, 5, 5).is_err());
    assert!(craquelure(&mut b, 50, 11, 5).is_err());
    assert!(craquelure(&mut b, 50, 5, 11).is_err());

    assert!(grain(&mut b, 0, 0, GrainType::Regular, [0, 0, 0], 1).is_ok());
    assert!(grain(&mut b, 100, 100, GrainType::Speckle, [255; 3], 1).is_ok());
    assert!(grain(&mut b, 101, 50, GrainType::Regular, [0, 0, 0], 1).is_err());
    assert!(grain(&mut b, 50, 101, GrainType::Regular, [0, 0, 0], 1).is_err());

    assert!(mosaic_tiles(&mut b, 2, 1, 0, 1).is_ok());
    assert!(mosaic_tiles(&mut b, 100, 15, 10, 1).is_ok());
    assert!(mosaic_tiles(&mut b, 1, 3, 5, 1).is_err());
    assert!(mosaic_tiles(&mut b, 50, 0, 5, 1).is_err());
    assert!(mosaic_tiles(&mut b, 50, 16, 5, 1).is_err());
    assert!(mosaic_tiles(&mut b, 50, 5, 11, 1).is_err());

    assert!(patchwork(&mut b, 0, 0, 1).is_ok());
    assert!(patchwork(&mut b, 10, 25, 1).is_ok());
    assert!(patchwork(&mut b, 11, 10, 1).is_err());
    assert!(patchwork(&mut b, 5, 26, 1).is_err());

    assert!(stained_glass(&mut b, 2, 1, 0, [0, 0, 0], 1).is_ok());
    assert!(stained_glass(&mut b, 50, 20, 10, [255; 3], 1).is_ok());
    assert!(stained_glass(&mut b, 1, 3, 5, [0, 0, 0], 1).is_err());
    assert!(stained_glass(&mut b, 50, 0, 5, [0, 0, 0], 1).is_err());
    assert!(stained_glass(&mut b, 50, 21, 5, [0, 0, 0], 1).is_err());
    assert!(stained_glass(&mut b, 50, 5, 11, [0, 0, 0], 1).is_err());

    assert!(texturizer(&mut b, TextureOptions::default()).is_ok());
    let bad_scaling = TextureOptions {
        scaling: 49,
        ..TextureOptions::default()
    };
    assert!(texturizer(&mut b, bad_scaling).is_err());
    let bad_relief = TextureOptions {
        relief: 51,
        ..TextureOptions::default()
    };
    assert!(texturizer(&mut b, bad_relief).is_err());
    let bad_light = TextureOptions {
        light_direction: 8,
        ..TextureOptions::default()
    };
    assert!(texturizer(&mut b, bad_light).is_err());
}

#[test]
fn stochastic_filters_are_seed_deterministic() {
    let base = gradient(24, 24, 3);

    let (mut a, mut c) = (base.clone(), base.clone());
    grain(&mut a, 60, 50, GrainType::Regular, [0, 0, 0], 77).unwrap();
    grain(&mut c, 60, 50, GrainType::Regular, [0, 0, 0], 78).unwrap();
    let mut b = base.clone();
    grain(&mut b, 60, 50, GrainType::Regular, [0, 0, 0], 77).unwrap();
    assert_eq!(a.data, b.data, "grain same seed must be bit-identical");
    assert_ne!(a.data, c.data, "grain different seed must differ");

    let (mut a, mut c) = (base.clone(), base.clone());
    mosaic_tiles(&mut a, 8, 3, 5, 5).unwrap();
    mosaic_tiles(&mut c, 8, 3, 5, 6).unwrap();
    let mut b = base.clone();
    mosaic_tiles(&mut b, 8, 3, 5, 5).unwrap();
    assert_eq!(a.data, b.data, "mosaic same seed must be bit-identical");
    assert_ne!(a.data, c.data, "mosaic different seed must differ");

    let (mut a, mut c) = (base.clone(), base.clone());
    patchwork(&mut a, 4, 10, 5).unwrap();
    patchwork(&mut c, 4, 10, 6).unwrap();
    let mut b = base.clone();
    patchwork(&mut b, 4, 10, 5).unwrap();
    assert_eq!(a.data, b.data, "patchwork same seed must be bit-identical");
    assert_ne!(a.data, c.data, "patchwork different seed must differ");

    let (mut a, mut c) = (base.clone(), base.clone());
    stained_glass(&mut a, 6, 2, 5, [0, 0, 0], 5).unwrap();
    stained_glass(&mut c, 6, 2, 5, [0, 0, 0], 6).unwrap();
    let mut b = base.clone();
    stained_glass(&mut b, 6, 2, 5, [0, 0, 0], 5).unwrap();
    assert_eq!(a.data, b.data, "stained same seed must be bit-identical");
    assert_ne!(a.data, c.data, "stained different seed must differ");
}

#[test]
fn grain_types_are_distinguishable() {
    let base = gradient(32, 32, 3);
    let kinds = [
        GrainType::Regular,
        GrainType::Soft,
        GrainType::Sprinkles,
        GrainType::Clumped,
        GrainType::Contrasty,
        GrainType::Enlarged,
        GrainType::Stippled,
        GrainType::Horizontal,
        GrainType::Vertical,
        GrainType::Speckle,
    ];
    let mut seen: BTreeSet<Vec<u8>> = BTreeSet::new();
    for kind in kinds {
        let mut b = base.clone();
        grain(&mut b, 70, 60, kind, [200, 150, 80], 21).unwrap();
        seen.insert(b.data.to_vec());
    }
    assert!(
        seen.len() >= 8,
        "the ten grain types must produce distinguishable output, got {}",
        seen.len()
    );
}

#[test]
fn grain_intensity_zero_is_a_noop() {
    let base = gradient(16, 8, 3);
    let mut b = base.clone();
    // Contrast stretches about mid-grey even without grain; 50 is neutral.
    grain(&mut b, 0, 50, GrainType::Regular, [9, 9, 9], 3).unwrap();
    assert_eq!(
        b.data, base.data,
        "intensity 0 at neutral contrast must be a no-op"
    );
}

#[test]
fn craquelure_spacing_and_brightness_matter() {
    let base = gradient(64, 16, 3);
    let (mut fine, mut coarse) = (base.clone(), base.clone());
    craquelure(&mut fine, 2, 6, 9).unwrap();
    craquelure(&mut coarse, 100, 6, 9).unwrap();
    assert_ne!(fine.data, coarse.data, "crack spacing must matter");

    let (mut dark, mut bright) = (base.clone(), base.clone());
    craquelure(&mut dark, 20, 6, 0).unwrap();
    craquelure(&mut bright, 20, 6, 10).unwrap();
    assert_ne!(dark.data, bright.data, "crack brightness must matter");
}

#[test]
fn mosaic_lighten_grout_lifts_luminance() {
    let base = gradient(32, 32, 3);
    let (mut dark, mut light) = (base.clone(), base.clone());
    mosaic_tiles(&mut dark, 8, 3, 0, 5).unwrap();
    mosaic_tiles(&mut light, 8, 3, 10, 5).unwrap();
    assert!(
        mean_luma(&light) > mean_luma(&dark),
        "lighten grout 10 ({}) must lift over 0 ({})",
        mean_luma(&light),
        mean_luma(&dark)
    );
}

#[test]
fn stained_glass_foreground_and_cell_size_matter() {
    let base = gradient(32, 32, 3);
    let (mut black, mut red) = (base.clone(), base.clone());
    stained_glass(&mut black, 8, 3, 5, [0, 0, 0], 7).unwrap();
    stained_glass(&mut red, 8, 3, 5, [255, 0, 0], 7).unwrap();
    assert_ne!(black.data, red.data, "foreground border colour must matter");

    let (mut small, mut large) = (base.clone(), base.clone());
    stained_glass(&mut small, 2, 3, 5, [0, 0, 0], 7).unwrap();
    stained_glass(&mut large, 50, 3, 5, [0, 0, 0], 7).unwrap();
    assert_ne!(small.data, large.data, "cell size must matter");
}

#[test]
fn texturizer_surfaces_and_light_directions_differ() {
    let base = gradient(32, 8, 3);
    let mut default_surface = base.clone();
    texturizer(&mut default_surface, TextureOptions::default()).unwrap();
    let mut brick = base.clone();
    texturizer(
        &mut brick,
        TextureOptions {
            surface: TextureSurface::Brick,
            ..TextureOptions::default()
        },
    )
    .unwrap();
    let mut lit = base.clone();
    texturizer(
        &mut lit,
        TextureOptions {
            light_direction: 4,
            ..TextureOptions::default()
        },
    )
    .unwrap();
    assert_ne!(default_surface.data, brick.data, "surface must matter");
    assert_ne!(
        default_surface.data, lit.data,
        "light direction must matter"
    );
}

#[test]
fn degenerate_sizes_do_not_panic() {
    for &(w, h) in &[(1u32, 1u32), (2, 3)] {
        let base = gradient(w, h, 3);
        let mut b = base.clone();
        craquelure(&mut b, 2, 10, 10).unwrap();
        let mut b = base.clone();
        mosaic_tiles(&mut b, 2, 1, 10, 1).unwrap();
        let mut b = base.clone();
        patchwork(&mut b, 0, 25, 1).unwrap();
        let mut b = base.clone();
        stained_glass(&mut b, 2, 20, 10, [3, 3, 3], 1).unwrap();
    }
}
