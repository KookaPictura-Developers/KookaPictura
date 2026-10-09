//! M9 and M11 no-equivalent Distort rows: ImageMagick's closest Distort
//! operators diverge structurally (see the mapping table), so guard the
//! contracts directly at the `Filter::apply` level.

use pictura_filters::{
    apply, Filter, PolarKind, RippleSize, ShearFill, SpherizeMode, WaveType, ZigZagStyle,
};

use crate::common::test_image_buffer;

/// M9 no-equivalent rows: ImageMagick has no faithful Distort operator (the
/// closest measured operators are in the mapping table), so guard the contracts
/// directly at the `Filter::apply` level. The module unit tests cover the same
/// filters in more detail.
#[test]
fn warp_no_equivalent_filters_properties() {
    let original = test_image_buffer();

    // Zero amount is a bit-exact no-op for every radial / ripple warp.
    for filter in [
        Filter::Twirl { angle: 0.0 },
        Filter::Pinch { amount: 0.0 },
        Filter::Spherize {
            amount: 0.0,
            mode: SpherizeMode::Normal,
        },
        Filter::Spherize {
            amount: 0.0,
            mode: SpherizeMode::HorizontalOnly,
        },
        Filter::Spherize {
            amount: 0.0,
            mode: SpherizeMode::VerticalOnly,
        },
        Filter::Ripple {
            amount: 0.0,
            size: RippleSize::Medium,
        },
    ] {
        let mut out = original.clone();
        apply(&filter, &mut out).expect("apply");
        assert_eq!(out.data, original.data, "{filter:?} must be a no-op at 0");
    }

    // Non-zero warps move pixels.
    for filter in [
        Filter::Twirl { angle: 45.0 },
        Filter::Pinch { amount: 50.0 },
        Filter::Spherize {
            amount: 50.0,
            mode: SpherizeMode::Normal,
        },
        Filter::Ripple {
            amount: 100.0,
            size: RippleSize::Medium,
        },
    ] {
        let mut out = original.clone();
        apply(&filter, &mut out).expect("apply");
        assert_ne!(out.data, original.data, "{filter:?} must move pixels");
    }

    // Wave is seed-deterministic and seed-sensitive.
    let wave = |seed: u64| Filter::Wave {
        generators: 3,
        wavelength: (10.0, 40.0),
        amplitude: (5.0, 15.0),
        kind: WaveType::Sine,
        scale: (100.0, 50.0),
        seed,
        repeat_edge: true,
    };
    let run = |filter: &Filter| {
        let mut out = original.clone();
        apply(filter, &mut out).expect("apply");
        out.data
    };
    assert_eq!(
        run(&wave(7)),
        run(&wave(7)),
        "Wave must be seed-deterministic"
    );
    assert_ne!(run(&wave(7)), run(&wave(8)), "Wave must vary with the seed");
}

/// M11 no-equivalent rows: ImageMagick's closest Distort operators
/// (`-distort Polar`/`DePolar`, `-shear`, `-swirl`, `-wave`) were measured and
/// diverge structurally (see the mapping table); guard the contracts directly
/// at the `Filter::apply` level. The module unit tests cover the same filters
/// in more detail.
#[test]
fn polar_no_equivalent_filters_properties() {
    let original = test_image_buffer();
    let run = |filter: &Filter| {
        let mut out = original.clone();
        apply(filter, &mut out).expect("apply");
        out.data
    };

    // Polar Coordinates: both directions are non-trivial and differ from each
    // other; the two are not inverses bit-for-bit (resampling loses detail).
    let r2p = run(&Filter::PolarCoordinates {
        kind: PolarKind::RectangularToPolar,
    });
    let p2r = run(&Filter::PolarCoordinates {
        kind: PolarKind::PolarToRectangular,
    });
    assert_ne!(r2p, original.data, "RectangularToPolar must remap");
    assert_ne!(p2r, original.data, "PolarToRectangular must remap");
    assert_ne!(r2p, p2r, "the two polar directions must differ");

    // Shear: a flat (zero) curve is a bit-exact no-op; a curve pinned to zero
    // at the top and bending right lower down leaves the top row alone and
    // pushes the bottom rows right, and the two fill modes differ.
    let noop = run(&Filter::Shear {
        curve: vec![(-1.0, 0.0), (1.0, 0.0)],
        fill: ShearFill::RepeatEdgePixels,
    });
    assert_eq!(noop, original.data, "a zero shear curve must be a no-op");
    let edge = run(&Filter::Shear {
        curve: vec![(-1.0, -0.5), (1.0, 0.5)],
        fill: ShearFill::RepeatEdgePixels,
    });
    let wrap = run(&Filter::Shear {
        curve: vec![(-1.0, -0.5), (1.0, 0.5)],
        fill: ShearFill::WrapAround,
    });
    assert_ne!(edge, original.data, "a sloped shear must move pixels");
    assert_ne!(edge, wrap, "the shear fill modes must differ");

    let mut marker = pictura_core::PixelBuffer::new(16, 16, 3);
    marker.data.fill(255);
    for y in 0..16usize {
        for c in 0..3 {
            marker.data[c * 256 + y * 16 + 4] = 0;
        }
    }
    let mut shifted = marker.clone();
    apply(
        &Filter::Shear {
            curve: vec![(-1.0, 0.0), (1.0, 0.5)],
            fill: ShearFill::RepeatEdgePixels,
        },
        &mut shifted,
    )
    .expect("apply");
    let dark_at = |buf: &pictura_core::PixelBuffer, y: usize| -> Option<usize> {
        (0..16).find(|&x| buf.data[y * 16 + x] < 128)
    };
    assert_eq!(dark_at(&shifted, 0), Some(4), "the top row must not move");
    assert!(
        dark_at(&shifted, 15).is_some_and(|x| x > 4),
        "the bottom row must move right"
    );

    // ZigZag: amount 0 is a bit-exact no-op; the three styles differ.
    let zz_noop = run(&Filter::ZigZag {
        amount: 0.0,
        ridges: 5,
        style: ZigZagStyle::AroundCenter,
    });
    assert_eq!(zz_noop, original.data, "ZigZag amount 0 must be a no-op");
    let styles = [
        ZigZagStyle::AroundCenter,
        ZigZagStyle::OutFromCenter,
        ZigZagStyle::PondRipples,
    ]
    .map(|style| {
        run(&Filter::ZigZag {
            amount: 80.0,
            ridges: 5,
            style,
        })
    });
    assert_ne!(styles[0], original.data, "ZigZag must displace pixels");
    assert_ne!(styles[0], styles[1], "ZigZag styles must differ (a/b)");
    assert_ne!(styles[0], styles[2], "ZigZag styles must differ (a/c)");
    assert_ne!(styles[1], styles[2], "ZigZag styles must differ (b/c)");

    // Ocean Ripple: magnitude 0 is a no-op; identical seeds match and different
    // seeds differ.
    let ocean = |seed: u64| {
        run(&Filter::OceanRipple {
            size: 9,
            magnitude: 20,
            seed,
        })
    };
    let ocean_noop = run(&Filter::OceanRipple {
        size: 9,
        magnitude: 0,
        seed: 1,
    });
    assert_eq!(
        ocean_noop, original.data,
        "OceanRipple magnitude 0 must be a no-op"
    );
    assert_eq!(
        ocean(42),
        ocean(42),
        "OceanRipple must be seed-deterministic"
    );
    assert_ne!(ocean(42), ocean(43), "OceanRipple must vary with the seed");
    assert_ne!(ocean(42), original.data, "OceanRipple must displace pixels");
}
