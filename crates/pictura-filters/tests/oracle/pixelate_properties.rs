//! M8 no-equivalent rows: ImageMagick has no faithful operator, so guard the
//! contracts directly.

use pictura_core::PixelBuffer;
use pictura_filters::{apply, Filter, MezzotintType};

use crate::common::test_image_buffer;

/// The measured deltas against the closest operators are in the mapping table.
/// See also the module unit tests for the same filters.
#[test]
fn no_equivalent_filters_properties() {
    let original = test_image_buffer();

    // Crystallize: seed-deterministic and piecewise constant; a flat field is
    // unchanged (the Voronoi mean of a constant is that constant).
    let mut a = original.clone();
    let mut b = original.clone();
    let mut c = original.clone();
    apply(
        &Filter::Crystallize {
            cell_size: 4,
            seed: 7,
        },
        &mut a,
    )
    .expect("apply");
    apply(
        &Filter::Crystallize {
            cell_size: 4,
            seed: 7,
        },
        &mut b,
    )
    .expect("apply");
    apply(
        &Filter::Crystallize {
            cell_size: 4,
            seed: 8,
        },
        &mut c,
    )
    .expect("apply");
    assert_eq!(a.data, b.data, "Crystallize must be seed-deterministic");
    assert_ne!(a.data, c.data, "Crystallize must vary with the seed");
    let mut flat = PixelBuffer::new(12, 12, 3);
    for v in flat.data.iter_mut() {
        *v = 90;
    }
    let before = flat.clone();
    apply(
        &Filter::Crystallize {
            cell_size: 4,
            seed: 1,
        },
        &mut flat,
    )
    .expect("apply");
    assert_eq!(
        flat.data, before.data,
        "Crystallize of a flat field is identity"
    );

    // Facet: a flat field is a bit-exact no-op; a gradient loses distinct values.
    let mut facet_flat = before.clone();
    apply(&Filter::Facet, &mut facet_flat).expect("apply");
    assert_eq!(
        facet_flat.data, before.data,
        "Facet flat field must be a no-op"
    );
    let n = original.pixel_count();
    let mut grad = PixelBuffer::new(64, 1, 3);
    for x in 0..64usize {
        let v = (255.0 * x as f64 / 63.0).round() as u8;
        grad.data[x] = v;
    }
    let distinct_before = grad.data[..64]
        .iter()
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    apply(&Filter::Facet, &mut grad).expect("apply");
    let distinct_after = grad.data[..64]
        .iter()
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    assert!(
        distinct_after < distinct_before,
        "Facet must flatten a gradient ({distinct_before} -> {distinct_after})"
    );

    // Fragment: a flat field is a bit-exact no-op; the output is the rounded
    // mean of the four source taps (known value on a 4x4 ramp).
    let mut frag_flat = before.clone();
    apply(&Filter::Fragment, &mut frag_flat).expect("apply");
    assert_eq!(
        frag_flat.data, before.data,
        "Fragment flat field must be a no-op"
    );
    let w = 4usize;
    let mut ramp = PixelBuffer::new(w as u32, w as u32, 3);
    for y in 0..w {
        for x in 0..w {
            let v = (x * 10 + y) as u8;
            for c in 0..3 {
                ramp.data[c * 16 + y * w + x] = v;
            }
        }
    }
    let src = ramp.data.clone();
    apply(&Filter::Fragment, &mut ramp).expect("apply");
    for y in 0..w {
        for x in 0..w {
            // CS6: four copies offset to the corners of a square and averaged.
            let sum: u32 = [(-4i32, -4i32), (4, -4), (-4, 4), (4, 4)]
                .iter()
                .map(|&(ox, oy)| {
                    let sx = (x as i32 + ox).clamp(0, w as i32 - 1) as usize;
                    let sy = (y as i32 + oy).clamp(0, w as i32 - 1) as usize;
                    src[sy * w + sx] as u32
                })
                .sum();
            assert_eq!(
                ramp.data[y * w + x],
                (sum / 4) as u8,
                "Fragment at ({x},{y})"
            );
        }
    }

    // Mezzotint: seed-deterministic and binary per channel. CS6 gives black
    // and white on a grayscale picture and fully saturated colours on a
    // colour one, so only a gray source stays achromatic.
    let mut mz_a = original.clone();
    let mut mz_b = original.clone();
    let mut mz_c = original.clone();
    apply(
        &Filter::Mezzotint {
            kind: MezzotintType::FineDots,
            seed: 3,
        },
        &mut mz_a,
    )
    .expect("apply");
    apply(
        &Filter::Mezzotint {
            kind: MezzotintType::FineDots,
            seed: 3,
        },
        &mut mz_b,
    )
    .expect("apply");
    apply(
        &Filter::Mezzotint {
            kind: MezzotintType::CoarseDots,
            seed: 3,
        },
        &mut mz_c,
    )
    .expect("apply");
    assert_eq!(mz_a.data, mz_b.data, "Mezzotint must be seed-deterministic");
    assert_ne!(mz_a.data, mz_c.data, "Mezzotint must vary with the kind");
    assert!(
        mz_a.data.iter().all(|&v| v == 0 || v == 255),
        "Mezzotint must be fully on or off per channel"
    );
    let mut gray = original.clone();
    for p in 0..n {
        let v = gray.data[p];
        gray.data[n + p] = v;
        gray.data[2 * n + p] = v;
    }
    apply(
        &Filter::Mezzotint {
            kind: MezzotintType::FineDots,
            seed: 3,
        },
        &mut gray,
    )
    .expect("apply");
    for p in 0..n {
        assert_eq!(gray.data[p], gray.data[n + p], "gray Mezzotint R != G");
        assert_eq!(gray.data[p], gray.data[2 * n + p], "gray Mezzotint R != B");
    }

    // Pointillize: seed-deterministic; a red source over a blue background
    // leaves only those two colors.
    let red = {
        let mut b = PixelBuffer::new(16, 16, 3);
        let m = b.pixel_count();
        for v in b.data[..m].iter_mut() {
            *v = 255;
        }
        b
    };
    let mut pt_a = red.clone();
    let mut pt_b = red.clone();
    let mut pt_c = red.clone();
    apply(
        &Filter::Pointillize {
            cell_size: 3,
            background: [0, 0, 255],
            seed: 9,
        },
        &mut pt_a,
    )
    .expect("apply");
    apply(
        &Filter::Pointillize {
            cell_size: 3,
            background: [0, 0, 255],
            seed: 9,
        },
        &mut pt_b,
    )
    .expect("apply");
    apply(
        &Filter::Pointillize {
            cell_size: 3,
            background: [0, 0, 255],
            seed: 10,
        },
        &mut pt_c,
    )
    .expect("apply");
    assert_eq!(
        pt_a.data, pt_b.data,
        "Pointillize must be seed-deterministic"
    );
    assert_ne!(pt_a.data, pt_c.data, "Pointillize must vary with the seed");
    let mut saw_source = false;
    let mut saw_background = false;
    for p in 0..pt_a.pixel_count() {
        let px = [pt_a.data[p], pt_a.data[n + p], pt_a.data[2 * n + p]];
        match px {
            [255, 0, 0] => saw_source = true,
            [0, 0, 255] => saw_background = true,
            _ => panic!("Pointillize produced an unexpected color {px:?}"),
        }
    }
    assert!(
        saw_source && saw_background,
        "expected dots over background"
    );

    // Color Halftone: deterministic and black/white.
    let gray = {
        let mut b = PixelBuffer::new(32, 32, 3);
        for v in b.data.iter_mut() {
            *v = 128;
        }
        b
    };
    let mut ch_a = gray.clone();
    let mut ch_b = gray.clone();
    apply(
        &Filter::ColorHalftone {
            max_radius: 5,
            angles: [15.0, 75.0, 0.0, 45.0],
        },
        &mut ch_a,
    )
    .expect("apply");
    apply(
        &Filter::ColorHalftone {
            max_radius: 5,
            angles: [15.0, 75.0, 0.0, 45.0],
        },
        &mut ch_b,
    )
    .expect("apply");
    assert_eq!(ch_a.data, ch_b.data, "ColorHalftone must be deterministic");
    assert!(
        ch_a.data.iter().all(|&v| v == 0 || v == 255),
        "ColorHalftone must be black/white"
    );
}
