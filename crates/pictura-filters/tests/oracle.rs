//! ImageMagick differential oracle for `pictura_filters::apply` (tasks M6-E,
//! M7-C, M8-B, M9-B, M11-B).
//!
//! ImageMagick implements a handful of the same Blur / Sharpen / Noise / Other
//! / Stylize / Pixelate filters. This is a *sanity* oracle, not a parity
//! oracle: Adobe's exact integer math and convolution kernels are closed, and
//! the ImageMagick operators only approximate several Photoshop paths. Each
//! filter with a faithful operator is diffed against the ImageMagick result
//! with `pictura_testkit::compare`; the tolerance and the reason for it are in
//! the table below and in `tests/README.md`.
//!
//! Filters with a faithful ImageMagick operator are run differentially
//! (`GaussianBlur`, `BoxBlur`, `Median`, `UnsharpMask`, `Maximum`, `Minimum`,
//! `Offset` with `wrap = true`, `Custom`, `Solarize`, `Mosaic`). The rest are
//! covered by ImageMagick-independent property/known-value tests here and in
//! the module unit tests; the divergences that ruled out a differential test
//! are recorded in the table below and in `tests/README.md`. The M9 Distort
//! filters (`Twirl`, `Pinch`, `Spherize`, `Ripple`, `Wave`) were measured
//! against the closest ImageMagick operator (`-swirl` / `-implode` / `-wave`)
//! and classified no-equivalent; see the table. The M11 Distort filters
//! (`PolarCoordinates`, `Shear`, `ZigZag`, `OceanRipple`) were measured against
//! `-distort Polar`/`DePolar`, `-shear`, `-swirl` and `-wave` and are likewise
//! no-equivalent; see the table.
//!
//! Regenerate/inspect a result manually with `scripts/filter_oracle.py`; see
//! `tests/README.md`.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use pictura_core::PixelBuffer;
use pictura_filters::{
    apply, Filter, MezzotintType, NoiseDistribution, PolarKind, Quality, RadialMethod, RippleSize,
    ShearFill, SpherizeMode, WaveType, ZigZagStyle,
};
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
    Mapping {
        filter: "Maximum",
        im: Some("-morphology Dilate Square:{radius}"),
        tolerance: 0,
        note: "same (2r+1)^2 square grayscale dilate, clamp-to-edge; measured max delta 0 \
               (radius 2). NB: IM `Square:N` takes a *radius* (kernel diameter 2N+1), so the \
               faithful flag is `Square:{radius}`; the M7 plan's `N = 2*radius+1` would dilate \
               over a (4r+3)^2 footprint",
    },
    Mapping {
        filter: "Minimum",
        im: Some("-morphology Erode Square:{radius}"),
        tolerance: 0,
        note: "same (2r+1)^2 square grayscale erode, clamp-to-edge; measured max delta 0 \
               (radius 2). Same `Square:N` radius-vs-diameter note as Maximum",
    },
    Mapping {
        filter: "Offset",
        im: Some("-roll {+horizontal}{+vertical}  (wrap = true)"),
        tolerance: 0,
        note: "wrap = true is an exact integer roll; measured max delta 0 at (3,2) and (2,3). \
               wrap = false fills the exposed area with `background`, which -roll cannot do (it \
               always wraps); observed max delta 240 vs -roll",
    },
    Mapping {
        filter: "HighPass",
        im: None,
        tolerance: 0,
        note: "no single IM operator. A hand-built `\\( +clone -gaussian-blur 0x{sigma} \\) \
               -compose Mathematics -define compose:args=0,-1,1,0.5 -composite` re-implements \
               Pictura's formula and is within measured max delta 1 (radius 3.0, sigma 1.0), but \
               it is not an independent operator; guarded by the flat-field mid-gray test",
    },
    Mapping {
        filter: "Custom",
        im: Some("-convolve {kernel}, -define convolve:scale={sum(kernel)/scale}, -evaluate add {offset/255}%"),
        tolerance: 0,
        note: "same f64 5x5 convolution, clamp-to-edge. IM -convolve normalizes by the kernel \
               sum, so the matching scale is sum(kernel)/scale; measured max delta 0 (edge \
               kernel, scale 4 offset 8 and scale 9 offset -10)",
    },
    Mapping {
        filter: "Emboss",
        im: None,
        tolerance: 0,
        note: "IM -emboss is per-channel with a fixed diagonal kernel and no angle/height/amount; \
               Pictura is an angle-directed second difference on luma with an achromatic output. \
               No angle/radius/sigma matches: observed max delta 210 at angle 135 vs -emboss 0x1 \
               (best case 186 on the axis-aligned angles)",
    },
    Mapping {
        filter: "FindEdges",
        im: None,
        tolerance: 0,
        note: "IM -edge is a different detector and renders edges bright on dark; Pictura's Sobel \
               magnitude is inverted (dark on light). Observed max delta 255 vs -edge 1",
    },
    Mapping {
        filter: "Solarize",
        im: Some("-solarize 50%"),
        tolerance: 0,
        note: "same fixed 50% inversion curve (v >= 128 -> 255 - v); measured max delta 0",
    },
    Mapping {
        filter: "Mosaic",
        im: Some("-filter box -resize {W/n}x{H/n}! -filter point -resize WxH!  (n = cell_size)"),
        tolerance: 0,
        note: "exact top-left block mean when the cell divides both dimensions; measured max delta 0 \
               at cell 4 on 16x16. For non-divisor cells IM's resize window is offset from Pictura's \
               blocks (observed max delta 80..136 at cell 3/5/6/7), so the differential uses cell 4",
    },
    Mapping {
        filter: "Crystallize",
        im: None,
        tolerance: 0,
        note: "seeded Voronoi cells have no ImageMagick operator; closest approximations -kuwahara 4 \
               (observed max delta 118) and -paint 4 (157). Guarded by seed determinism and the \
               flat-field identity property",
    },
    Mapping {
        filter: "Facet",
        im: None,
        tolerance: 0,
        note: "similar-neighbor banded 3x3 mean; IM -statistic mean 3x3 is the closest (observed max \
               delta 76; -kuwahara 1 gives 144). Guarded by flat-field identity and gradient flattening",
    },
    Mapping {
        filter: "Fragment",
        im: None,
        tolerance: 0,
        note: "clamp-anchored sliding 2x2 mean; -statistic mean 2x2 uses a different window/edge rule \
               (observed max delta 85; -kuwahara 1 gives 59). Guarded by the 4-tap known-value test",
    },
    Mapping {
        filter: "Mezzotint",
        im: None,
        tolerance: 0,
        note: "seeded procedural dot/line pattern; IM -threshold 50% and -ordered-dither are different \
               screens (observed max delta 255). Guarded by seed determinism, binarity and achromatic \
               output",
    },
    Mapping {
        filter: "Pointillize",
        im: None,
        tolerance: 0,
        note: "seeded local-color dots over background; IM -spread 2 displaces pixels instead of \
               drawing dots (observed max delta 240; -kuwahara 2 gives 232). Guarded by seed \
               determinism and the source/background color set",
    },
    Mapping {
        filter: "ColorHalftone",
        im: None,
        tolerance: 0,
        note: "per-channel rotated screen has no ImageMagick operator; -ordered-dither o8x8 / h4x4a are \
               fixed orthogonal screens (observed max delta 255). Guarded by determinism and binarity",
    },
    Mapping {
        filter: "Twirl",
        im: None,
        tolerance: 0,
        note: "IM -swirl uses a smooth falloff and radius = min(w,h)/2; Pictura twirls with a linear \
               falloff over max(cx,cy). Best measured max delta 124 (Twirl +45 vs -swirl 45, same \
               sign); -90 vs -swirl 90 gives 170. Guarded by the zero no-op and rotation tests",
    },
    Mapping {
        filter: "Pinch",
        im: None,
        tolerance: 0,
        note: "IM -implode amount is a fraction (not a percent); positive implodes, negative explodes. \
               Pictura's linear radial remap is not IM's implode falloff. Best measured max delta 61 \
               (Pinch 50 vs -implode 0.5); Pinch -50 vs -implode -0.5 gives 84",
    },
    Mapping {
        filter: "Spherize",
        im: None,
        tolerance: 0,
        note: "closest is -implode {amount/100}; Pictura's arc-length sphere map (Normal / \
               HorizontalOnly / VerticalOnly) is not IM's implode falloff. Best measured max delta 159 \
               (Spherize 50 Normal vs -implode 0.5); -50 gives 128",
    },
    Mapping {
        filter: "Ripple",
        im: None,
        tolerance: 0,
        note: "IM -wave displaces one axis with a sine in x and pads the canvas (background fill), \
               while Pictura ripple displaces both axes (dx ~ sin y, dy ~ sin x) with clamp-to-edge \
               and a fixed period per size. Measured max delta 255 / mean 104 (Ripple 100 Medium vs \
               -wave 10x16 cropped to 16x16)",
    },
    Mapping {
        filter: "Wave",
        im: None,
        tolerance: 0,
        note: "IM -wave is a single unseeded sine along one axis; Pictura sums seeded generators with \
               random phase/period/amplitude and an axis-wise scale. Measured max delta 255 / mean 101 \
               (1 generator sine, amp 20, wavelength 10, seed 42 vs -wave 20x10 cropped)",
    },
    Mapping {
        filter: "PolarCoordinates",
        im: None,
        tolerance: 0,
        note: "IM -distort Polar/DePolar use a different angle origin (180 deg off) and \
               pixel-center/radius anchor plus IM's own resampling. Best measured: RectangularToPolar \
               vs `-distort Polar 0` max delta 189 / mean 58.3; PolarToRectangular vs \
               `-distort DePolar 0` max 194 / mean 58.9 (the crossed directions give max 240/241). Guarded \
               by the remap/no-op property tests",
    },
    Mapping {
        filter: "Shear",
        im: None,
        tolerance: 0,
        note: "IM `-shear 0x{angle}` is a whole-canvas y-shear that background-fills the expanded \
               canvas (default black), while Pictura shifts columns by a piecewise-linear curve with \
               clamp/wrap and expands nothing. Straight curve [(-1,-0.5),(1,0.5)] = atan(0.5) = \
               26.565 deg; best measured `-shear 0x26.565 -crop 16x16+0+4 +repage` vs \
               RepeatEdgePixels max 255 / mean 18.4 (WrapAround mean 23.4). Guarded by the zero-curve \
               no-op and fill property tests",
    },
    Mapping {
        filter: "ZigZag",
        im: None,
        tolerance: 0,
        note: "IM `-swirl` uses a smooth falloff about min(w,h)/2; Pictura's cosine radial profile is \
               pinned to zero at the edge with `ridges` reversals. Best measured `-swirl 50` vs amount \
               80 / ridges 5 / AroundCenter max 170 / mean 14.3 (`-swirl 80` mean 14.9, `-swirl -80` \
               mean 18.8, `-implode 0.8` mean 18.6). Guarded by the zero no-op and style tests",
    },
    Mapping {
        filter: "OceanRipple",
        im: None,
        tolerance: 0,
        note: "IM `-wave` is an unseeded single-axis sine that pads the canvas; Pictura sums 8 seeded \
               direction sinusoids with clamp-to-edge. Best measured `-wave 2x8 -crop 16x16+0+2 \
               +repage` vs size 9 / magnitude 20 / seed 42 max 227 / mean 53.7. Guarded by seed \
               determinism and the zero-magnitude no-op",
    },
];

/// Filters the table marks as having no faithful ImageMagick equivalent.
const NO_EQUIVALENT: [&str; 29] = [
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
    "HighPass",
    "Emboss",
    "FindEdges",
    "Crystallize",
    "Facet",
    "Fragment",
    "Mezzotint",
    "Pointillize",
    "ColorHalftone",
    "Twirl",
    "Pinch",
    "Spherize",
    "Ripple",
    "Wave",
    "PolarCoordinates",
    "Shear",
    "ZigZag",
    "OceanRipple",
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
    assert_eq!(MAPPING.len(), 39, "one mapping row per Filter variant");
    // Exactly one row per `Filter` variant, no duplicates, full enum coverage.
    let all: [&str; 39] = [
        "GaussianBlur",
        "BoxBlur",
        "MotionBlur",
        "RadialBlur",
        "Average",
        "Blur",
        "BlurMore",
        "SurfaceBlur",
        "Sharpen",
        "SharpenMore",
        "SharpenEdges",
        "UnsharpMask",
        "AddNoise",
        "Median",
        "Despeckle",
        "Maximum",
        "Minimum",
        "Offset",
        "HighPass",
        "Custom",
        "Emboss",
        "FindEdges",
        "Solarize",
        "Mosaic",
        "Crystallize",
        "Facet",
        "Fragment",
        "Mezzotint",
        "Pointillize",
        "ColorHalftone",
        "Twirl",
        "Pinch",
        "Spherize",
        "Ripple",
        "Wave",
        "PolarCoordinates",
        "Shear",
        "ZigZag",
        "OceanRipple",
    ];
    let mut mapped: Vec<&str> = MAPPING.iter().map(|m| m.filter).collect();
    mapped.sort_unstable();
    let mut expected_all = all.to_vec();
    expected_all.sort_unstable();
    assert_eq!(mapped, expected_all, "every Filter variant needs one row");

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

/// `Filter::Custom` kernel used by the differential test: a 5x5 sharpening /
/// edge kernel with a non-trivial sum, so both the divisor and the bias are
/// exercised. Sum = 29.
const CUSTOM_TEST_KERNEL: [[f64; 5]; 5] = [
    [0.0, 0.0, -1.0, 0.0, 0.0],
    [0.0, -1.0, 4.0, -1.0, 0.0],
    [-1.0, 4.0, 20.0, 4.0, -1.0],
    [0.0, -1.0, 4.0, -1.0, 0.0],
    [0.0, 0.0, -1.0, 0.0, 0.0],
];

#[test]
fn maximum_matches_imagemagick() {
    differential(
        &Filter::Maximum { radius: 2 },
        &["--op", "maximum", "--radius", "2"],
        0,
        "Maximum radius 2",
    );
}

#[test]
fn minimum_matches_imagemagick() {
    differential(
        &Filter::Minimum { radius: 2 },
        &["--op", "minimum", "--radius", "2"],
        0,
        "Minimum radius 2",
    );
}

#[test]
fn offset_wrap_matches_imagemagick() {
    differential(
        &Filter::Offset {
            horizontal: 3,
            vertical: 2,
            wrap: true,
            background: [0, 0, 0],
        },
        &["--op", "roll", "--horizontal", "3", "--vertical", "2"],
        0,
        "Offset wrap (3,2)",
    );
}

#[test]
fn custom_matches_imagemagick() {
    differential(
        &Filter::Custom {
            kernel: CUSTOM_TEST_KERNEL,
            scale: 4.0,
            offset: 8.0,
        },
        &[
            "--op",
            "convolve",
            "--kernel",
            "0,0,-1,0,0,0,-1,4,-1,0,-1,4,20,4,-1,0,-1,4,-1,0,0,0,-1,0,0",
            "--kernel-scale",
            "4",
            "--kernel-offset",
            "8",
        ],
        0,
        "Custom edge kernel scale 4 offset 8",
    );
}

#[test]
fn solarize_matches_imagemagick() {
    differential(
        &Filter::Solarize,
        &["--op", "solarize", "--threshold-percent", "50"],
        0,
        "Solarize 50%",
    );
}

/// M8: `Mosaic` with a cell that divides the image is an exact top-left block
/// mean, which `-filter box -resize` down + `-filter point -resize` up matches
/// bit-exactly (measured max delta 0).
#[test]
fn mosaic_matches_imagemagick() {
    differential(
        &Filter::Mosaic { cell_size: 4 },
        &["--op", "mosaic", "--cell", "4"],
        0,
        "Mosaic cell 4",
    );
}

/// M7 no-equivalent rows: guard the contracts directly because no ImageMagick
/// operator is faithful. See the table above for the observed deltas.
#[test]
fn m7_no_equivalent_filters_properties() {
    // A flat color field for the spatial filters.
    let mut flat = PixelBuffer::new(6, 5, 3);
    for v in flat.data.iter_mut() {
        *v = 90;
    }

    // High Pass: a flat field collapses to mid-gray (128).
    let mut hp = flat.clone();
    apply(&Filter::HighPass { radius: 2.0 }, &mut hp).expect("apply");
    assert!(
        hp.data.iter().all(|&v| (v as i32 - 128).abs() <= 1),
        "HighPass flat field must be mid-gray"
    );

    // Find Edges: a flat field is white (Sobel magnitude 0, inverted), and a
    // step edge is dark.
    let mut fe = flat.clone();
    apply(&Filter::FindEdges, &mut fe).expect("apply");
    assert!(
        fe.data.iter().all(|&v| v == 255),
        "FindEdges flat field must stay light"
    );
    let mut step = pixel_row(&[0, 0, 0, 0, 255, 255, 255, 255]);
    apply(&Filter::FindEdges, &mut step).expect("apply");
    assert!(
        step.data[3] < 128 && step.data[4] < 128,
        "FindEdges must darken a step edge"
    );

    // Emboss: flat field is neutral gray and the output is achromatic, even on
    // colored input.
    let mut emb = flat.clone();
    apply(
        &Filter::Emboss {
            angle: 135.0,
            height: 3.0,
            amount: 100.0,
        },
        &mut emb,
    )
    .expect("apply");
    assert!(
        emb.data.iter().all(|&v| (v as i32 - 128).abs() <= 1),
        "Emboss flat field must be neutral gray"
    );
    let colored = test_image_buffer();
    let mut colored_emb = colored.clone();
    apply(
        &Filter::Emboss {
            angle: 135.0,
            height: 3.0,
            amount: 100.0,
        },
        &mut colored_emb,
    )
    .expect("apply");
    let n = colored.pixel_count();
    for i in 0..n {
        assert_eq!(
            colored_emb.data[i],
            colored_emb.data[n + i],
            "Emboss R != G at {i}"
        );
        assert_eq!(
            colored_emb.data[i],
            colored_emb.data[2 * n + i],
            "Emboss R != B at {i}"
        );
    }

    // Offset wrap = false: the exposed area takes `background`, the shifted
    // area copies the source.
    let base = test_image_buffer();
    let mut shifted = base.clone();
    apply(
        &Filter::Offset {
            horizontal: 3,
            vertical: 2,
            wrap: false,
            background: [10, 20, 30],
        },
        &mut shifted,
    )
    .expect("apply");
    let at = |b: &PixelBuffer, x: usize, y: usize, c: usize| {
        b.data[c * b.pixel_count() + y * b.width as usize + x]
    };
    assert_eq!(
        [
            at(&shifted, 0, 0, 0),
            at(&shifted, 0, 0, 1),
            at(&shifted, 0, 0, 2)
        ],
        [10, 20, 30],
        "exposed pixel must take the background"
    );
    for c in 0..3 {
        assert_eq!(
            at(&base, 5, 5, c),
            at(&shifted, 8, 7, c),
            "shifted pixel must copy the source (channel {c})"
        );
    }
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

/// M8 no-equivalent rows: ImageMagick has no faithful operator, so guard the
/// contracts directly. The measured deltas against the closest operators are in
/// the table above. See also the module unit tests for the same filters.
#[test]
fn m8_no_equivalent_filters_properties() {
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
            let sum: u32 = [(0usize, 0usize), (1, 0), (0, 1), (1, 1)]
                .iter()
                .map(|&(ox, oy)| {
                    let sx = (x + ox).min(w - 1);
                    let sy = (y + oy).min(w - 1);
                    src[sy * w + sx] as u32
                })
                .sum();
            assert_eq!(
                ramp.data[y * w + x],
                ((sum + 2) / 4) as u8,
                "Fragment at ({x},{y})"
            );
        }
    }

    // Mezzotint: seed-deterministic, binary, and achromatic.
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
        "Mezzotint must be black/white"
    );
    for p in 0..n {
        assert_eq!(mz_a.data[p], mz_a.data[n + p], "Mezzotint R != G");
        assert_eq!(mz_a.data[p], mz_a.data[2 * n + p], "Mezzotint R != B");
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

/// The differential harness must actually catch a wrong filter output. Apply a
/// faithful filter, check it matches, then perturb a scratch copy and check the
/// comparison fails; revert and confirm it matches again.
#[test]
fn differential_harness_detects_perturbation() {
    if !magick_available() {
        eprintln!("skipping: `magick` not on PATH");
        return;
    }
    let original = test_image_planar();
    let reference = oracle(
        &["--op", "solarize", "--threshold-percent", "50"],
        &original,
    );
    let mut buf = PixelBuffer {
        width: SIZE,
        height: SIZE,
        channels: CHANNELS,
        data: original,
    };
    apply(&Filter::Solarize, &mut buf).expect("apply");
    assert!(
        compare(&buf.data, &reference, 0)
            .expect("lengths")
            .is_empty(),
        "sanity: Solarize must match the oracle before perturbation"
    );

    let mut perturbed = buf.clone();
    perturbed.data[0] = perturbed.data[0].wrapping_add(37);
    assert!(
        !compare(&perturbed.data, &reference, 0)
            .expect("lengths")
            .is_empty(),
        "a perturbed filter output must not compare equal"
    );

    perturbed.data[0] = perturbed.data[0].wrapping_sub(37);
    assert!(
        compare(&perturbed.data, &reference, 0)
            .expect("lengths")
            .is_empty(),
        "reverting the scratch copy must restore the match"
    );
}

/// M9 no-equivalent rows: ImageMagick has no faithful Distort operator (the
/// closest measured operators are in the table above), so guard the contracts
/// directly at the `Filter::apply` level. The module unit tests cover the same
/// filters in more detail.
#[test]
fn m9_no_equivalent_filters_properties() {
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
/// diverge structurally (see the table above); guard the contracts directly at
/// the `Filter::apply` level. The module unit tests cover the same filters in
/// more detail.
#[test]
fn m11_no_equivalent_filters_properties() {
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

    // Shear: a flat (zero) curve is a bit-exact no-op; a sloped curve shifts
    // columns, and the two fill modes differ.
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
