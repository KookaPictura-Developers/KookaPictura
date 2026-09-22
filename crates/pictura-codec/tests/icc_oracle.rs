//! ICC conversion oracle: the embedded Adobe-RGB profile fixture must convert
//! its layer pixels to sRGB, matching an independent lcms2 (PIL) conversion, and
//! the profile resource must be dropped on save.

use std::path::PathBuf;
use std::process::Command;

use pictura_codec::{decode_image_resources, read_psd, write_psd, ICC_PROFILE};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn pil_available() -> bool {
    static AVAILABLE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *AVAILABLE.get_or_init(|| {
        Command::new("python3")
            .args(["-c", "import PIL.ImageCms"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    })
}

/// Convert the source color `(200, 100, 50)` from the fixture profile to sRGB
/// with PIL/lcms2; the fixture's Base layer holds that source color.
fn independent_convert() -> Option<[u8; 3]> {
    let script = r#"
import sys
from PIL import Image, ImageCms
src = Image.new("RGB", (1, 1), (200, 100, 50))
prof = ImageCms.getOpenProfile(sys.argv[1])
srgb = ImageCms.createProfile("sRGB")
out = ImageCms.profileToProfile(src, prof, srgb, renderingIntent=1, outputMode="RGB")
print(*out.getpixel((0, 0)))
"#;
    let out = Command::new("python3")
        .arg("-c")
        .arg(script)
        .arg(fixture("psd_icc_rgb.icc"))
        .output()
        .expect("run python3");
    if !out.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let values: Vec<u8> = stdout
        .split_whitespace()
        .map(|t| t.parse().expect("channel value"))
        .collect();
    (values.len() == 3).then(|| [values[0], values[1], values[2]])
}

#[test]
fn embedded_icc_profile_is_applied_and_dropped() {
    let profile = std::fs::read(fixture("psd_icc_rgb.icc")).expect("read profile");
    let bytes = std::fs::read(fixture("icc_profile.psd")).expect("read fixture");
    let doc = read_psd(&bytes).expect("fixture parses");

    assert_eq!(
        doc.source_icc.as_deref(),
        Some(profile.as_slice()),
        "the source profile is recorded"
    );
    assert!(
        doc.document_icc.is_none(),
        "read-normalisation must not set the working profile"
    );
    assert!(
        decode_image_resources(&doc)
            .iter()
            .all(|r| r.id != ICC_PROFILE),
        "the stale profile resource is dropped on read"
    );

    let base = doc
        .layers
        .iter()
        .find(|l| l.name == "Base")
        .expect("Base layer");
    let channel = |id: i16| -> u8 {
        base.channels
            .iter()
            .find(|c| c.id == id)
            .unwrap_or_else(|| panic!("layer has channel {id}"))
            .data[0]
    };
    let pixel = [channel(0), channel(1), channel(2)];

    if !pil_available() {
        eprintln!("skipping lcms2 comparison: PIL not available");
        return;
    }
    let expected = independent_convert().expect("PIL conversion");
    for (got, want) in pixel.iter().zip(expected.iter()) {
        assert!(
            (*got as i32 - *want as i32).abs() <= 1,
            "converted pixel {pixel:?} differs from lcms2 {expected:?}"
        );
    }
}

#[test]
fn icc_normalized_save_has_no_profile_resource() {
    let bytes = std::fs::read(fixture("icc_profile.psd")).expect("read fixture");
    let doc = read_psd(&bytes).expect("fixture parses");
    assert!(
        doc.document_icc.is_none(),
        "the read-normalised document has no working profile"
    );
    let out = write_psd(&doc).expect("write");
    let back = read_psd(&out).expect("re-read");
    assert!(
        decode_image_resources(&back)
            .iter()
            .all(|r| r.id != ICC_PROFILE),
        "the saved file carries no ICC profile resource"
    );
    assert!(
        back.source_icc.is_none(),
        "the re-read file is not ICC-normalized"
    );
    assert!(
        back.document_icc.is_none(),
        "a plain re-read has no working profile"
    );
}

#[test]
fn plain_read_has_no_working_profile() {
    let bytes = std::fs::read(fixture("two_layers.psd")).expect("read fixture");
    let doc = read_psd(&bytes).expect("fixture parses");
    assert!(
        doc.document_icc.is_none(),
        "a document without an assigned profile stays in the sRGB working space"
    );
}
