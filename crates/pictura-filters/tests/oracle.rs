//! ImageMagick differential oracle for `pictura_filters::apply` (task M6-E).
//!
//! ImageMagick implements a handful of the same Blur / Sharpen / Noise filters.
//! This is a *sanity* oracle, not a parity oracle: Adobe's exact integer math
//! and convolution kernels are closed, and the ImageMagick operators only
//! approximate several Photoshop paths. Each filter with a faithful operator is
//! diffed against the ImageMagick result with `pictura_testkit::compare`; the
//! tolerance and the reason for it are in the table below and in
//! `tests/README.md`.
//!
//! Only filters with a faithful ImageMagick operator are run differentially
//! (`GaussianBlur`, `BoxBlur`, `Median`, `UnsharpMask`). The rest are covered by
//! ImageMagick-independent property/known-value tests here and in the module
//! unit tests; the divergences that ruled out a differential test are recorded
//! in the table below and in `tests/README.md`.
//!
//! Regenerate/inspect a result manually with `scripts/filter_oracle.py`; see
//! `tests/README.md`.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use pictura_core::PixelBuffer;
use pictura_filters::{apply, Filter, NoiseDistribution, Quality, RadialMethod};
use pictura_testkit::{compare, Diff};

/// Side length of the raw planar RGB8 test image.
const SIZE: u32 = 16;
const CHANNELS: u8 = 3;

/// One row of the `Filter` -> ImageMagick mapping table. `im` is the equivalent
/// operator (documentation; the actual invocation lives with each test) or
/// `None` when there is no faithful equivalent.
struct Mapping {
    filter: &'static str,
    im: Option<&'static str>,
    tolerance: u8,
    note: &'static str,
}

const MAPPING: &[Mapping] = &[
    Mapping {
        filter: "GaussianBlur",
        im: Some("-gaussian-blur 0x{sigma}  (sigma = radius / 3)"),
        tolerance: 0,
        note: "same 3-sigma separable Gaussian, clamp-to-edge; measured max delta 0",
    },
    Mapping {
        filter: "BoxBlur",
        im: Some("-statistic mean NxN  (N = 2*radius + 1)"),
        tolerance: 0,
        note: "same separable moving average, clamp-to-edge; measured max delta 0",
    },
    Mapping {
        filter: "Median",
        im: Some("-median {radius}"),
        tolerance: 0,
        note: "same (2r+1)^2 per-channel rank filter, clamp-to-edge; measured max delta 0",
    },
    Mapping {
        filter: "UnsharpMask",
        im: Some("-unsharp 0x{sigma}+{amount/100}+{threshold/255}"),
        tolerance: 6,
        note: "same blur-difference gain; IM amount is a fraction (100% = 1.0). IM's internal \
               blur differs slightly from its standalone -gaussian-blur: measured max delta 5 at \
               radius 3 / amount 150 / threshold 0",
    },
    Mapping {
        filter: "MotionBlur",
        im: None,
        tolerance: 0,
        note: "IM -motion-blur builds a one-sided Gaussian line kernel; Pictura averages \
               symmetric uniform taps. Observed max delta 86 (vs -motion-blur 0x5+0)",
    },
    Mapping {
        filter: "RadialBlur",
        im: None,
        tolerance: 0,
        note: "ImageMagick 7 removed -radial-blur; -rotational-blur weights a different angle \
               profile. Observed max delta 58 (Spin 20 vs -rotational-blur 20)",
    },
    Mapping {
        filter: "Average",
        im: None,
        tolerance: 0,
        note: "trivial global region mean; no IM operator shares the window/border semantics. \
               Observed max delta 92 (vs -statistic mean 16x16)",
    },
    Mapping {
        filter: "Blur",
        im: None,
        tolerance: 0,
        note: "PS fixed [1 2 1] separable kernel; IM -blur is a Gaussian. Observed max delta 23 \
               (vs -blur 0x1)",
    },
    Mapping {
        filter: "BlurMore",
        im: None,
        tolerance: 0,
        note: "three passes of the PS [1 2 1] kernel; observed max delta 14 (vs -blur 0x1)",
    },
    Mapping {
        filter: "SurfaceBlur",
        im: None,
        tolerance: 0,
        note: "bilateral range/space weights are closed; IM has no bilateral operator. Observed \
               max delta 32 (vs -gaussian-blur 0x1)",
    },
    Mapping {
        filter: "Sharpen",
        im: None,
        tolerance: 0,
        note: "fixed 3x3 high-pass kernel; IM -sharpen is a Gaussian unsharp. Observed max delta \
               17 (vs -sharpen 0x1)",
    },
    Mapping {
        filter: "SharpenMore",
        im: None,
        tolerance: 0,
        note: "fixed stronger 3x3 high-pass kernel; observed max delta 17 (vs -sharpen 0x1)",
    },
    Mapping {
        filter: "SharpenEdges",
        im: None,
        tolerance: 0,
        note: "edge-gated high-pass with a fixed gate; observed max delta 17 (vs -sharpen 0x1)",
    },
    Mapping {
        filter: "AddNoise",
        im: None,
        tolerance: 0,
        note: "RNG streams differ; same-seed determinism is the contract. Observed max delta 79 \
               (vs -attenuate 0.1 +noise Gaussian)",
    },
    Mapping {
        filter: "Despeckle",
        im: None,
        tolerance: 0,
        note: "IM -despeckle uses a different rank detector; observed max delta 13 (vs \
               -despeckle)",
    },
];

/// Filters the table marks as having no faithful ImageMagick equivalent.
const NO_EQUIVALENT: [&str; 11] = [
    "MotionBlur",
    "RadialBlur",
    "Average",
    "Blur",
    "BlurMore",
    "SurfaceBlur",
    "Sharpen",
    "SharpenMore",
    "SharpenEdges",
    "AddNoise",
    "Despeckle",
];

fn script() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/filter_oracle.py")
}

fn magick_available() -> bool {
    Command::new("magick")
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Unique scratch directory per call so tests can run in parallel.
fn scratch_dir(tag: &str) -> PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "pictura-filters-oracle-{}-{tag}-{n}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Deterministic `SIZE`x`SIZE` RGB8 test image: two ramps plus a hard block
/// checker so every filter sees both gradients and edges.
fn test_image_interleaved() -> Vec<u8> {
    let mut data = Vec::with_capacity((SIZE * SIZE * 3) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let r = (x * 16) as u8;
            let g = (y * 16) as u8;
            let b = if (x / 4 + y / 4) % 2 == 0 { 40 } else { 210 };
            data.extend_from_slice(&[r, g, b]);
        }
    }
    data
}

/// Interleaved RGB8 -> planar RGB (the `PixelBuffer` layout `apply` consumes).
fn planarize(interleaved: &[u8]) -> Vec<u8> {
    let pixels = (SIZE * SIZE) as usize;
    let mut planar = vec![0u8; interleaved.len()];
    for i in 0..pixels {
        for c in 0..CHANNELS as usize {
            planar[c * pixels + i] = interleaved[i * CHANNELS as usize + c];
        }
    }
    planar
}

fn test_image_planar() -> Vec<u8> {
    planarize(&test_image_interleaved())
}

/// The `SIZE`x`SIZE` test image as a `PixelBuffer`.
fn test_image_buffer() -> PixelBuffer {
    PixelBuffer {
        width: SIZE,
        height: SIZE,
        channels: CHANNELS,
        data: test_image_planar(),
    }
}

/// Run `scripts/filter_oracle.py apply` over the planar test image and return
/// the raw planar result. `extra` holds the operator arguments.
fn oracle(extra: &[&str], input_planar: &[u8]) -> Vec<u8> {
    let dir = scratch_dir("oracle");
    let input_path = dir.join("in.rgb");
    let output_path = dir.join("out.rgb");
    std::fs::write(&input_path, input_planar).unwrap();
    let result = Command::new("python3")
        .arg(script())
        .arg("apply")
        .args(["--size", "16x16", "--planar"])
        .args(extra)
        .arg(&input_path)
        .arg(&output_path)
        .output()
        .expect("run filter_oracle.py apply");
    assert!(
        result.status.success(),
        "oracle failed:\n{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let bytes = std::fs::read(&output_path).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    bytes
}

/// Apply `filter` and diff it against the ImageMagick oracle. `None` when
/// `magick` is not on PATH.
fn oracle_diff(filter: &Filter, extra: &[&str], tolerance: u8) -> Option<Diff> {
    if !magick_available() {
        return None;
    }
    let original = test_image_planar();
    let reference = oracle(extra, &original);
    let mut buf = PixelBuffer {
        width: SIZE,
        height: SIZE,
        channels: CHANNELS,
        data: original,
    };
    apply(filter, &mut buf).expect("pictura_filters::apply");
    Some(compare(&buf.data, &reference, tolerance).expect("buffer lengths agree"))
}

/// Diff `apply` against the ImageMagick oracle. Skips when `magick` is absent.
fn differential(filter: &Filter, extra: &[&str], tolerance: u8, label: &str) {
    let Some(diff) = oracle_diff(filter, extra, tolerance) else {
        eprintln!("skipping {label}: `magick` not on PATH");
        return;
    };
    assert!(
        diff.is_empty(),
        "{label}: {} of {} samples over tolerance {tolerance} (max delta {})",
        diff.differing,
        diff.samples,
        diff.max_delta
    );
}

/// One horizontal planar RGB row from interleaved greys.
fn pixel_row(px: &[u8]) -> PixelBuffer {
    let n = px.len();
    let mut data = vec![0u8; n * 3];
    for (i, &v) in px.iter().enumerate() {
        data[i] = v;
        data[n + i] = v;
        data[2 * n + i] = v;
    }
    PixelBuffer {
        width: n as u32,
        height: 1,
        channels: 3,
        data,
    }
}

fn plane_range(buf: &PixelBuffer) -> u8 {
    let plane = &buf.data[..buf.pixel_count()];
    *plane.iter().max().unwrap() - *plane.iter().min().unwrap()
}

#[test]
fn reference_script_is_present() {
    let path = script();
    assert!(
        path.is_file(),
        "missing oracle script at {}",
        path.display()
    );
    let help = Command::new("python3")
        .arg(&path)
        .arg("apply")
        .arg("--help")
        .output()
        .expect("run filter_oracle.py apply --help");
    assert!(
        help.status.success(),
        "`apply --help` failed:\n{}",
        String::from_utf8_lossy(&help.stderr)
    );
}

#[test]
fn imagemagick_runs() {
    if !magick_available() {
        eprintln!("skipping: `magick` not on PATH");
        return;
    }
    let out = Command::new("python3")
        .arg(script())
        .arg("version")
        .output()
        .expect("run filter_oracle.py version");
    assert!(
        out.status.success(),
        "`version` failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains("ImageMagick"),
        "unexpected version output:\n{text}"
    );
}

/// The oracle itself, independent of `apply`: `-negate` must produce `M - v`
/// for every planar sample (this also proves the planar round-trip).
#[test]
fn oracle_negate_matches_expected_bytes() {
    if !magick_available() {
        eprintln!("skipping: `magick` not on PATH");
        return;
    }
    let input = test_image_planar();
    let output = oracle(&["--im-args=-negate"], &input);
    assert_eq!(output.len(), input.len());
    let expected: Vec<u8> = input.iter().map(|v| 255 - v).collect();
    assert_eq!(output, expected);
}

#[test]
fn mapping_marks_no_equivalent_operators() {
    assert_eq!(MAPPING.len(), 15, "one mapping row per Filter variant");
    let none: Vec<&str> = MAPPING
        .iter()
        .filter(|m| m.im.is_none())
        .map(|m| m.filter)
        .collect();
    let mut expected = NO_EQUIVALENT.to_vec();
    expected.sort_unstable();
    let mut none = none;
    none.sort_unstable();
    assert_eq!(none, expected);
    for m in MAPPING {
        assert!(!m.note.is_empty(), "{}: empty note", m.filter);
        if m.im.is_none() {
            assert_eq!(
                m.tolerance, 0,
                "{}: no equivalent must not set tolerance",
                m.filter
            );
        } else {
            assert!(m.tolerance <= 8, "{}: tolerance out of range", m.filter);
        }
    }
}

#[test]
fn gaussian_blur_matches_imagemagick() {
    differential(
        &Filter::GaussianBlur { radius: 3.0 },
        &["--op", "gaussian", "--sigma", "1.0"],
        0,
        "GaussianBlur radius 3.0",
    );
}

#[test]
fn box_blur_matches_imagemagick() {
    differential(
        &Filter::BoxBlur { radius: 3 },
        &["--op", "box", "--radius", "3"],
        0,
        "BoxBlur radius 3",
    );
}

#[test]
fn median_matches_imagemagick() {
    differential(
        &Filter::Median { radius: 1 },
        &["--op", "median", "--radius", "1"],
        0,
        "Median radius 1",
    );
}

#[test]
fn unsharp_mask_matches_imagemagick() {
    differential(
        &Filter::UnsharpMask {
            amount: 150.0,
            radius: 3.0,
            threshold: 0,
        },
        &[
            "--op",
            "unsharp",
            "--sigma",
            "1.0",
            "--amount",
            "150",
            "--threshold",
            "0",
        ],
        6,
        "UnsharpMask radius 3.0 amount 150 threshold 0",
    );
}

/// ImageMagick has no faithful MotionBlur equivalent: `-motion-blur` builds a
/// one-sided Gaussian line kernel while Pictura averages symmetric uniform taps
/// (observed max delta 86). Guard the tap geometry directly instead.
#[test]
fn motion_blur_known_values() {
    let mut b = PixelBuffer::new(9, 9, 3);
    b.data[4 * 9 + 4] = 255;
    apply(
        &Filter::MotionBlur {
            angle: 0.0,
            distance: 5,
        },
        &mut b,
    )
    .expect("apply");
    let px = |x: usize, y: usize| b.data[y * 9 + x];
    assert_eq!(px(4, 4), 51);
    assert_eq!(px(2, 4), 51);
    assert_eq!(px(6, 4), 51);
    assert_eq!(px(4, 3), 0);
    assert_eq!(px(4, 5), 0);

    // distance <= 1 is a no-op.
    let mut base = PixelBuffer::new(4, 4, 3);
    base.data[5] = 200;
    let mut same = base.clone();
    apply(
        &Filter::MotionBlur {
            angle: 30.0,
            distance: 1,
        },
        &mut same,
    )
    .expect("apply");
    assert_eq!(same.data, base.data);
}

/// ImageMagick has no faithful Sharpen-family equivalent (fixed 3x3 kernels;
/// IM `-sharpen` is a Gaussian unsharp). Guard the PS contract directly.
#[test]
fn sharpen_family_known_values() {
    let flat = pixel_row(&[77, 77, 77, 77, 77, 77]);
    for filter in [Filter::Sharpen, Filter::SharpenMore, Filter::SharpenEdges] {
        let mut out = flat.clone();
        apply(&filter, &mut out).expect("apply");
        assert_eq!(
            out.data, flat.data,
            "{filter:?} must not change a flat field"
        );
    }

    let base = pixel_row(&[100, 100, 100, 100, 140, 140, 140, 140]);
    for filter in [Filter::Sharpen, Filter::SharpenMore, Filter::SharpenEdges] {
        let mut out = base.clone();
        apply(&filter, &mut out).expect("apply");
        assert!(
            plane_range(&out) > plane_range(&base),
            "{filter:?} must increase edge contrast"
        );
    }
}

/// ImageMagick has no faithful Average / Radial / Surface / Blur equivalent.
/// Guard their contracts (identity at zero, or a trivial known value).
#[test]
fn no_equivalent_filters_properties() {
    let original = test_image_buffer();

    // Average is the exact global mean, replicated across every pixel.
    let mut averaged = original.clone();
    apply(&Filter::Average, &mut averaged).expect("apply");
    let n = original.pixel_count();
    for c in 0..3usize {
        let plane = &original.data[c * n..c * n + n];
        let mean = (plane.iter().map(|&v| v as u64).sum::<u64>() as f64 / n as f64).round() as u8;
        assert!(
            averaged.data[c * n..c * n + n].iter().all(|&v| v == mean),
            "Average plane {c} is not the constant region mean {mean}"
        );
    }

    // Radial spin/zoom are identity at zero amount (Surface rejects radius 0).
    for filter in [
        Filter::RadialBlur {
            method: RadialMethod::Spin,
            amount: 0.0,
            quality: Quality::Best,
        },
        Filter::RadialBlur {
            method: RadialMethod::Zoom,
            amount: 0.0,
            quality: Quality::Best,
        },
    ] {
        let mut out = original.clone();
        apply(&filter, &mut out).expect("apply");
        assert_eq!(out.data, original.data, "{filter:?} must be identity");
    }

    // Blur / Blur More must be near-identity on a solid field.
    for filter in [Filter::Blur, Filter::BlurMore] {
        let mut solid = PixelBuffer::new(6, 5, 3);
        for v in solid.data.iter_mut() {
            *v = 90;
        }
        let before = solid.clone();
        apply(&filter, &mut solid).expect("apply");
        for (a, b) in solid.data.iter().zip(before.data.iter()) {
            assert!(
                (*a as i32 - *b as i32).abs() <= 1,
                "{filter:?} drifted on solid"
            );
        }
    }
}

/// Add Noise is randomized, so it is not diffed against ImageMagick. The
/// contract is same-seed determinism (and that different seeds differ).
#[test]
fn add_noise_same_seed_is_deterministic() {
    for distribution in [NoiseDistribution::Uniform, NoiseDistribution::Gaussian] {
        for monochromatic in [false, true] {
            let base = test_image_buffer();
            let mut a = base.clone();
            let mut b = base.clone();
            let mut c = base.clone();
            let filter = Filter::AddNoise {
                amount: 10.0,
                distribution,
                monochromatic,
                seed: 42,
            };
            apply(&filter, &mut a).expect("apply");
            apply(&filter, &mut b).expect("apply");
            apply(
                &Filter::AddNoise {
                    amount: 10.0,
                    distribution,
                    monochromatic,
                    seed: 43,
                },
                &mut c,
            )
            .expect("apply");
            assert_eq!(a.data, b.data, "{filter:?} must be seed-deterministic");
            assert_ne!(a.data, c.data, "{filter:?} must vary with the seed");
        }
    }
}
