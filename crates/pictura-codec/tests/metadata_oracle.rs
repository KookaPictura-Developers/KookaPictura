//! Metadata oracle: the fixture's EXIF and IPTC must decode to the same values
//! the independent `exiftool` decoder reads from the same file. Self-skips when
//! `exiftool` is unavailable.

use std::path::PathBuf;
use std::process::Command;

use pictura_codec::{decode_image_resources, read_metadata, read_psd, ExifValue, XMP_METADATA};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn exiftool_available() -> bool {
    static AVAILABLE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *AVAILABLE.get_or_init(|| {
        Command::new("exiftool")
            .arg("-ver")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    })
}

/// The tags requested from exiftool, in the order its `-s3` output prints them.
const TAGS: &[&str] = &[
    "-IFD0:Make",
    "-IFD0:Model",
    "-IFD0:Software",
    "-IFD0:ModifyDate",
    "-ExifIFD:ExposureTime",
    "-ExifIFD:FNumber",
    "-ExifIFD:ISO",
    "-ExifIFD:DateTimeOriginal",
    "-ExifIFD:FocalLength",
    "-IPTC:ObjectName",
    "-IPTC:By-line",
    "-IPTC:CopyrightNotice",
    "-IPTC:Caption-Abstract",
];

fn exiftool_values() -> Option<Vec<String>> {
    let out = Command::new("exiftool")
        .arg("-s3")
        .args(TAGS)
        .arg(fixture("metadata.psd"))
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<String> = text.lines().map(str::to_string).collect();
    (lines.len() == TAGS.len()).then_some(lines)
}

fn number(text: &str) -> f64 {
    let text = text.trim();
    if let Some((num, den)) = text.split_once('/') {
        return num.trim().parse::<f64>().unwrap() / den.trim().parse::<f64>().unwrap();
    }
    text.split_whitespace()
        .next()
        .unwrap()
        .parse()
        .unwrap_or_else(|_| panic!("not a number: {text:?}"))
}

#[test]
fn metadata_decodes_and_agrees_with_exiftool() {
    let bytes = std::fs::read(fixture("metadata.psd")).expect("read fixture");
    let doc = read_psd(&bytes).expect("fixture parses");
    let metadata = read_metadata(&doc);

    // Engine-side expectations, independent of exiftool.
    assert_eq!(
        metadata.exif.get(0x010f),
        Some(&ExifValue::Ascii("ACME Cameras".into())),
        "Make"
    );
    assert_eq!(
        metadata.exif.get(0x0110),
        Some(&ExifValue::Ascii("ACME One".into())),
        "Model"
    );
    assert_eq!(
        metadata.exif.get(0x0131),
        Some(&ExifValue::Ascii("Kooka Pictura Fixtures".into())),
        "Software"
    );
    assert_eq!(
        metadata.exif.get(0x0132),
        Some(&ExifValue::Ascii("2026:09:22 12:00:00".into())),
        "ModifyDate"
    );
    assert_eq!(
        metadata.exif.get(0x829d),
        Some(&ExifValue::Rational(28, 10)),
        "FNumber"
    );
    assert_eq!(
        metadata.exif.get(0x8827),
        Some(&ExifValue::Short(200)),
        "ISO"
    );
    assert_eq!(metadata.iptc.text(2, 5).as_deref(), Some("Fixture Title"));
    let xmp_resource = decode_image_resources(&doc)
        .into_iter()
        .find(|r| r.id == XMP_METADATA)
        .expect("fixture carries an XMP resource");
    assert_eq!(metadata.xmp.as_bytes(), xmp_resource.data.as_slice());

    if !exiftool_available() {
        eprintln!("skipping exiftool comparison: exiftool not available");
        return;
    }
    let values = exiftool_values().expect("exiftool is available but produced no values");
    let expect = |index: usize| values[index].as_str();

    assert_eq!(
        metadata.exif.get(0x010f),
        Some(&ExifValue::Ascii(expect(0).into())),
        "Make"
    );
    assert_eq!(
        metadata.exif.get(0x0110),
        Some(&ExifValue::Ascii(expect(1).into())),
        "Model"
    );
    assert_eq!(
        metadata.exif.get(0x0131),
        Some(&ExifValue::Ascii(expect(2).into())),
        "Software"
    );
    assert_eq!(
        metadata.exif.get(0x0132),
        Some(&ExifValue::Ascii(expect(3).into())),
        "ModifyDate"
    );
    assert_eq!(
        metadata.exif.get(0x829a).and_then(ExifValue::as_f64),
        Some(number(expect(4))),
        "ExposureTime"
    );
    assert_eq!(
        metadata.exif.get(0x829d).and_then(ExifValue::as_f64),
        Some(number(expect(5))),
        "FNumber"
    );
    assert_eq!(
        metadata.exif.get(0x8827).and_then(ExifValue::as_f64),
        Some(number(expect(6))),
        "ISO"
    );
    assert_eq!(
        metadata.exif.get(0x9003),
        Some(&ExifValue::Ascii(expect(7).into())),
        "DateTimeOriginal"
    );
    assert_eq!(
        metadata.exif.get(0x920a).and_then(ExifValue::as_f64),
        Some(number(expect(8))),
        "FocalLength"
    );
    assert_eq!(metadata.iptc.text(2, 5).as_deref(), Some(expect(9)));
    assert_eq!(metadata.iptc.text(2, 80).as_deref(), Some(expect(10)));
    assert_eq!(metadata.iptc.text(2, 116).as_deref(), Some(expect(11)));
    assert_eq!(metadata.iptc.text(2, 120).as_deref(), Some(expect(12)));
}
