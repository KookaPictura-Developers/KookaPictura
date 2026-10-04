use super::*;

#[test]
fn every_kind_defaults_from_empty_and_rejects_wrong_arity() {
    for (kind, n) in FILTER_ARITIES {
        assert!(
            filter_from_kind_params(kind, &[]).is_some(),
            "{kind} must default from an empty slot list"
        );
        let slots = vec![0.0f64; *n];
        assert!(
            filter_from_kind_params(kind, &slots).is_some(),
            "{kind} must accept exactly {n} slots"
        );
        let mut too_many = slots.clone();
        too_many.push(0.0);
        assert!(
            filter_from_kind_params(kind, &too_many).is_none(),
            "{kind} must refuse {} slots",
            n + 1
        );
        if *n > 1 {
            let mut too_few = slots.clone();
            too_few.pop();
            assert!(
                filter_from_kind_params(kind, &too_few).is_none(),
                "{kind} must refuse {} slots",
                n - 1
            );
        }
    }
    assert!(filter_from_kind_params("bogus", &[]).is_none());
}

#[test]
fn parameter_values_override_defaults() {
    assert_eq!(
        filter_from_kind_params("gaussian-blur", &[12.5]),
        Some(Filter::GaussianBlur { radius: 12.5 })
    );
    assert_eq!(
        filter_from_kind_params("add-noise", &[50.0, 1.0, 1.0, 7.0]),
        Some(Filter::AddNoise {
            amount: 50.0,
            distribution: NoiseDistribution::Gaussian,
            monochromatic: true,
            seed: 7,
        })
    );
    assert_eq!(
        filter_from_kind_params("radial-blur", &[1.0, 20.0, 2.0]),
        Some(Filter::RadialBlur {
            method: RadialMethod::Zoom,
            amount: 20.0,
            quality: Quality::Best,
        })
    );
    // Wind direction is a Choice index: 0 = "From the Right", 1 = "From the Left".
    assert_eq!(
        filter_from_kind_params("wind", &[2.0, 0.0]),
        Some(Filter::Wind {
            method: WindMethod::Stagger,
            from_right: true,
        })
    );
    assert_eq!(
        filter_from_kind_params("wind", &[2.0, 1.0]),
        Some(Filter::Wind {
            method: WindMethod::Stagger,
            from_right: false,
        })
    );
}

#[test]
fn multi_slot_offsets_decode_correctly() {
    // rgb group at base 1.
    assert_eq!(
        filter_from_kind_params("pointillize", &[3.0, 10.0, 20.0, 30.0, 9.0]),
        Some(Filter::Pointillize {
            cell_size: 3,
            background: [10, 20, 30],
            seed: 9,
        })
    );
    // A 5-slot TextureOptions block.
    assert_eq!(
        filter_from_kind_params("texturizer", &[1.0, 55.0, 7.0, 3.0, 1.0]),
        Some(Filter::Texturizer {
            texture: TextureOptions {
                surface: TextureSurface::Burlap,
                scaling: 55,
                relief: 7,
                light_direction: 3,
                invert: true,
            },
        })
    );
    // The 7-slot shear curve and fill.
    assert_eq!(
        filter_from_kind_params("shear", &[0.25, 0.5, 0.75, 0.85, 1.5, 2.5, 0.0]),
        Some(Filter::Shear {
            curve: vec![(0.25, 0.5), (0.75, 0.85), (1.5, 2.5)],
            fill: ShearFill::WrapAround,
        })
    );
    // Wave wavelength/amplitude/scale pairs.
    assert_eq!(
        filter_from_kind_params("wave", &[4.0, 6.0, 8.0, 2.0, 3.0, 1.0, 50.0, 75.0, 11.0]),
        Some(Filter::Wave {
            generators: 4,
            wavelength: (6.0, 8.0),
            amplitude: (2.0, 3.0),
            kind: WaveType::Triangle,
            scale: (50.0, 75.0),
            seed: 11,
            repeat_edge: true,
        })
    );
    // Smart Sharpen shadow (6..9) and highlight (9..12) TonalFade blocks.
    assert_eq!(
        filter_from_kind_params(
            "smart-sharpen",
            &[50.0, 2.0, 30.0, 1.0, 45.0, 1.0, 10.0, 20.0, 3.0, 40.0, 50.0, 4.0],
        ),
        Some(Filter::SmartSharpen {
            amount: 50.0,
            radius: 2.0,
            reduce_noise: 30.0,
            remove: SharpenRemove::LensBlur,
            angle: 45.0,
            more_accurate: true,
            shadow: TonalFade {
                amount: 10,
                width: 20,
                radius: 3,
            },
            highlight: TonalFade {
                amount: 40,
                width: 50,
                radius: 4,
            },
        })
    );
}
