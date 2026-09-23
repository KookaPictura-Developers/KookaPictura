//! Differential tests for the Bitmap, Indexed, CMYK, and Lab color modes.
//!
//! `read_psd` normalizes each to RGB; the committed fixtures (authored by
//! `scripts/generate-fixtures.py`) are re-read here with the independent
//! `psd-tools` decoder and compared pixel for pixel. The oracle self-skips when
//! `psd-tools` is absent.

use std::path::PathBuf;
use std::process::Command;

use pictura_codec::{read_psd, write_psd};
use pictura_core::{BitDepth, ColorMode, Document};

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn load(name: &str) -> Document {
    let path = fixture_dir().join(name);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "cannot read {} ({e}); run `python3 scripts/generate-fixtures.py`",
            path.display()
        )
    });
    read_psd(&bytes).unwrap_or_else(|e| panic!("{} failed to parse: {e}", path.display()))
}

fn psd_tools_available() -> bool {
    static AVAILABLE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *AVAILABLE.get_or_init(|| {
        Command::new("python3")
            .args(["-c", "import psd_tools"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    })
}

/// Decode `path` with `psd-tools` and return its composite as planar RGB hex.
fn psd_tools_composite_planes(path: &std::path::Path) -> Vec<String> {
    let script = r#"
import sys
import numpy as np
from psd_tools import PSDImage
psd = PSDImage.open(sys.argv[1], lazy=False)
arr = np.array(psd.composite().convert("RGB"))
for c in range(3):
    print(arr[:, :, c].tobytes().hex())
"#;
    let out = Command::new("python3")
        .arg("-c")
        .arg(script)
        .arg(path)
        .output()
        .expect("run python3");
    assert!(
        out.status.success(),
        "psd-tools failed on {}:\n{}",
        path.display(),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|line| line.trim().to_string())
        .collect()
}

/// Decode `path` with `psd-tools` and convert its Lab composite to RGB with
/// lcms2's exact (unoptimized) transform. psd-tools' own `.convert("RGB")` uses
/// an optimized color LUT that can differ by up to ~20 on some in-gamut colors,
/// so the exact transform is the reference for the profile-free Lab formula.
fn lcms2_exact_lab_planes(path: &std::path::Path) -> Vec<String> {
    let script = r#"
import sys
from psd_tools import PSDImage
from PIL import ImageCms
psd = PSDImage.open(sys.argv[1], lazy=False)
lab = psd.composite()
lab_prof = ImageCms.createProfile("LAB")
srgb = ImageCms.createProfile("sRGB")
tr = ImageCms.buildTransform(
    lab_prof, srgb, "LAB", "RGB", flags=ImageCms.Flags.NOOPTIMIZE
)
rgb = tr.apply(lab)
for c in range(3):
    print(rgb.getchannel(c).tobytes().hex())
"#;
    let out = Command::new("python3")
        .arg("-c")
        .arg(script)
        .arg(path)
        .output()
        .expect("run python3");
    assert!(
        out.status.success(),
        "psd-tools/lcms2 failed on {}:\n{}",
        path.display(),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|line| line.trim().to_string())
        .collect()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn assert_planes_within(label: &str, planes: &[&[u8]], reference: &[String], tolerance: u8) {
    assert_eq!(
        reference.len(),
        planes.len(),
        "{label}: reference plane count"
    );
    for (c, (ours, expected_hex)) in planes.iter().zip(reference.iter()).enumerate() {
        let expected: Vec<u8> = (0..expected_hex.len() / 2)
            .map(|i| u8::from_str_radix(&expected_hex[i * 2..i * 2 + 2], 16).unwrap())
            .collect();
        assert_eq!(ours.len(), expected.len(), "{label}: plane {c} length");
        for (i, (ours, theirs)) in ours.iter().zip(expected.iter()).enumerate() {
            let diff = (*ours as i32 - *theirs as i32).abs();
            assert!(
                diff <= tolerance as i32,
                "{label}: plane {c} pixel {i} ours={ours} reference={theirs} diff={diff}"
            );
        }
    }
}

fn ours_matching(name: &str, tolerance: u8, reference: fn(&std::path::Path) -> Vec<String>) {
    let path = fixture_dir().join(name);
    let doc = load(name);
    let data = &doc.composite.data;
    let plane = doc.composite.pixel_count();
    let our_planes: Vec<&[u8]> = (0..3).map(|c| &data[c * plane..(c + 1) * plane]).collect();
    let theirs = reference(&path);
    assert_planes_within(name, &our_planes, &theirs, tolerance);
}

#[test]
fn indexed_fixture_matches_psd_tools() {
    if !psd_tools_available() {
        eprintln!("skipping: python3 + psd-tools not available");
        return;
    }
    // Palette expansion is exact: no tolerance.
    let path = fixture_dir().join("indexed.psd");
    let doc = load("indexed.psd");
    let plane = doc.composite.pixel_count();
    let theirs = psd_tools_composite_planes(&path);
    for (c, expected_hex) in theirs.iter().enumerate() {
        assert_eq!(
            hex(&doc.composite.data[c * plane..(c + 1) * plane]),
            *expected_hex
        );
    }
}

#[test]
fn cmyk_fixture_matches_psd_tools_within_one() {
    if !psd_tools_available() {
        eprintln!("skipping: python3 + psd-tools not available");
        return;
    }
    // psd-tools/Pillow rounds `color * black / 255`; the engine floors it, a
    // <=1 difference the varied fixture planes make visible.
    ours_matching("cmyk.psd", 1, psd_tools_composite_planes);
    let doc = load("cmyk.psd");
    assert_eq!(doc.layers.len(), 1, "cmyk.psd carries one pixel layer");
    assert_eq!(
        doc.layers[0]
            .channels
            .iter()
            .map(|c| c.id)
            .collect::<Vec<_>>(),
        vec![0, 1, 2, -1],
        "the CMYK layer becomes three RGB channels plus transparency"
    );
}

#[test]
fn lab_fixture_matches_lcms2_exact_within_one() {
    if !psd_tools_available() {
        eprintln!("skipping: python3 + psd-tools not available");
        return;
    }
    // The exact lcms2 transform, not psd-tools' optimized-LUT `.convert("RGB")`.
    ours_matching("lab.psd", 1, lcms2_exact_lab_planes);
}

#[test]
fn bitmap_fixture_matches_psd_tools() {
    if !psd_tools_available() {
        eprintln!("skipping: python3 + psd-tools not available");
        return;
    }
    ours_matching("bitmap.psd", 0, psd_tools_composite_planes);
}

/// A non-Lab normalized document still saves as RGB: re-reading the written file
/// yields the same working mode and pixels with no source mode, and `psd-tools`
/// opens it as an RGB document. (Lab saves back as Lab; see the next test.)
#[test]
fn normalized_documents_round_trip_as_rgb() {
    for (name, source) in [
        ("indexed.psd", ColorMode::Indexed),
        ("cmyk.psd", ColorMode::Cmyk),
        ("bitmap.psd", ColorMode::Bitmap),
    ] {
        let doc = load(name);
        assert_eq!(doc.mode, ColorMode::Rgb, "{name}: working mode");
        assert_eq!(
            doc.source_mode,
            Some(source),
            "{name}: recorded source mode"
        );
        assert_eq!(doc.depth, BitDepth::Eight, "{name}: normalized depth");
        if source == ColorMode::Indexed {
            assert!(doc.color_mode_data.is_empty(), "{name}: palette consumed");
        }

        let out = write_psd(&doc).unwrap();
        assert_eq!(
            u16::from_be_bytes(out[24..26].try_into().unwrap()),
            3,
            "{name}: output header color mode is RGB"
        );
        let back = read_psd(&out).unwrap();
        assert_eq!(back.mode, ColorMode::Rgb, "{name}: re-read mode");
        assert_eq!(back.source_mode, None, "{name}: no source mode after save");
        assert_eq!(
            back.composite, doc.composite,
            "{name}: normalized pixels are stable across a save"
        );

        if !psd_tools_available() {
            eprintln!("skipping: python3 + psd-tools not available");
            continue;
        }
        let dir = scratch_dir("color-mode-roundtrip");
        let path = dir.join(name);
        std::fs::write(&path, &out).unwrap();
        let script = r#"
import sys
from psd_tools import PSDImage
psd = PSDImage.open(sys.argv[1])
print(int(psd.color_mode))
"#;
        let result = Command::new("python3")
            .arg("-c")
            .arg(script)
            .arg(&path)
            .output()
            .expect("run python3");
        let _ = std::fs::remove_dir_all(&dir);
        assert!(
            result.status.success(),
            "psd-tools failed on the written {name}:\n{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&result.stdout).trim(),
            "3",
            "{name}: psd-tools opens the output as RGB"
        );
    }
}

/// A Lab document saves back as Lab: the output header color mode is Lab, the
/// planes stay three channels, and an **unedited** document re-emits the retained
/// Lab planes byte-identically, so a re-read reproduces the working RGB exactly.
/// lcms2's exact transform of the written Lab planes is the independent
/// reference; lcms2 and psd-tools are test-only.
#[test]
fn lab_document_saves_as_lab() {
    let doc = load("lab.psd");
    assert_eq!(
        doc.source_mode,
        Some(ColorMode::Lab),
        "recorded source mode"
    );
    let plane = doc.composite.pixel_count();
    let retained = doc.source_planes.clone().expect("Lab planes retained");
    assert_eq!(retained.depth, BitDepth::Eight, "8-bit Lab store");
    assert!(
        !doc.retains_source_depth(),
        "the Lab store is not a native-depth store"
    );
    let source_lab = retained.data[..3 * plane].to_vec();

    let out = write_psd(&doc).unwrap();
    assert_eq!(
        u16::from_be_bytes(out[12..14].try_into().unwrap()),
        3,
        "output keeps three color channels"
    );
    assert_eq!(
        u16::from_be_bytes(out[24..26].try_into().unwrap()),
        9,
        "output header color mode is Lab"
    );

    let back = read_psd(&out).unwrap();
    assert_eq!(back.mode, ColorMode::Rgb, "re-read working mode");
    assert_eq!(
        back.source_mode,
        Some(ColorMode::Lab),
        "re-read source mode"
    );
    let back_retained = back.source_planes.clone().expect("re-read Lab planes");
    assert_eq!(
        back_retained.data[..3 * plane],
        source_lab[..],
        "unedited Lab planes are re-emitted byte-identically"
    );
    assert_eq!(
        back.composite, doc.composite,
        "re-reading the retained planes reproduces the working RGB exactly"
    );
    let working: Vec<&[u8]> = (0..3)
        .map(|c| &doc.composite.data[c * plane..(c + 1) * plane])
        .collect();

    if !psd_tools_available() {
        eprintln!("skipping: python3 + psd-tools not available");
        return;
    }
    let dir = scratch_dir("lab-write-back");
    let path = dir.join("lab.psd");
    std::fs::write(&path, &out).unwrap();
    // lcms2's exact transform of the written Lab planes must reproduce the working
    // RGB the document was editing within tolerance.
    let reference = lcms2_exact_lab_planes(&path);
    assert_planes_within("lab.psd lcms2", &working, &reference, 1);
    let script = r#"
import sys
from psd_tools import PSDImage
psd = PSDImage.open(sys.argv[1])
print(int(psd.color_mode))
"#;
    let result = Command::new("python3")
        .arg("-c")
        .arg(script)
        .arg(&path)
        .output()
        .expect("run python3");
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        result.status.success(),
        "psd-tools failed on the written lab.psd:\n{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&result.stdout).trim(),
        "9",
        "psd-tools opens the output as Lab"
    );
}

fn scratch_dir(tag: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "pictura-codec-color-{}-{tag}-{n}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
