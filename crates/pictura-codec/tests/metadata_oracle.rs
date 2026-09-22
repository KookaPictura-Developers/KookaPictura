//! Metadata oracle: the fixture's EXIF and IPTC must decode to the same values
//! the independent `exiftool` decoder reads from the same file. Self-skips when
//! `exiftool` is unavailable.

use std::path::{Path, PathBuf};
use std::process::Command;

use pictura_codec::{
    apply_template, decode_image_resources, read_metadata, read_psd, set_file_info_fields,
    write_psd, xmp_properties, ExifValue, MergeMode, XmpProperties, XMP_METADATA,
};

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

fn exiftool_fields(path: &Path, tags: &[&str]) -> Option<Vec<String>> {
    let out = Command::new("exiftool")
        .arg("-s3")
        .args(tags)
        .arg(path)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<String> = text.lines().map(str::to_string).collect();
    (lines.len() == tags.len()).then_some(lines)
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

const XMP_TAGS: &[&str] = &[
    "-XMP-dc:Title",
    "-XMP-dc:Creator",
    "-XMP-dc:Description",
    "-XMP-dc:Rights",
    "-XMP-photoshop:Credit",
    "-XMP-photoshop:Source",
];

#[test]
fn parsed_xmp_agrees_with_exiftool() {
    let bytes = std::fs::read(fixture("metadata.psd")).expect("read fixture");
    let doc = read_psd(&bytes).expect("fixture parses");
    let props = xmp_properties(&doc);

    // Engine-side expectations, independent of exiftool.
    assert_eq!(props.title.as_deref(), Some("Fixture Title"));
    assert_eq!(props.creator, vec!["Ada Lovelace"]);
    assert_eq!(props.description.as_deref(), Some("A fixture caption."));
    assert_eq!(props.rights.as_deref(), Some("(c) 2026 Kooka Pictura"));
    assert_eq!(props.credit.as_deref(), Some("Kooka Pictura"));
    assert_eq!(props.source.as_deref(), Some("Test Suite"));

    if !exiftool_available() {
        eprintln!("skipping exiftool XMP comparison: exiftool not available");
        return;
    }
    let values = exiftool_fields(&fixture("metadata.psd"), XMP_TAGS)
        .expect("exiftool is available but produced no values");
    assert_eq!(props.title.as_deref(), Some(values[0].as_str()));
    assert_eq!(props.creator, vec![values[1].clone()]);
    assert_eq!(props.description.as_deref(), Some(values[2].as_str()));
    assert_eq!(props.rights.as_deref(), Some(values[3].as_str()));
    assert_eq!(props.credit.as_deref(), Some(values[4].as_str()));
    assert_eq!(props.source.as_deref(), Some(values[5].as_str()));
}

#[test]
fn edited_xmp_survives_save_and_exiftool() {
    let bytes = std::fs::read(fixture("metadata.psd")).expect("read fixture");
    let mut doc = read_psd(&bytes).expect("fixture parses");
    assert!(
        set_file_info_fields(
            &mut doc,
            &[
                (2, 5, "Edited Title".into()),
                (2, 110, "Edited Credit".into())
            ]
        ),
        "the edit changed the document"
    );

    let out = write_psd(&doc).expect("write");
    let back = read_psd(&out).expect("re-read");
    let props = xmp_properties(&back);
    assert_eq!(props.title.as_deref(), Some("Edited Title"));
    assert_eq!(props.credit.as_deref(), Some("Edited Credit"));
    assert_eq!(
        read_metadata(&back).iptc.text(2, 5).as_deref(),
        Some("Edited Title"),
        "IIM is synced"
    );
    let raw = read_metadata(&back).xmp;
    assert!(
        raw.contains("acme:Marker=\"keep-me\""),
        "the unknown namespace attribute survives"
    );
    assert!(
        raw.contains("<acme:Note>keep-me-too</acme:Note>"),
        "the unknown-namespace property survives"
    );

    if !exiftool_available() {
        eprintln!("skipping exiftool XMP round-trip: exiftool not available");
        return;
    }
    let dir = std::env::temp_dir().join(format!("pictura-xmp-write-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("edited.psd");
    std::fs::write(&path, &out).unwrap();
    let values = exiftool_fields(&path, &["-XMP-dc:Title", "-XMP-photoshop:Credit"])
        .expect("exiftool is available but produced no values");
    assert_eq!(values[0], "Edited Title");
    assert_eq!(values[1], "Edited Credit");
}

#[test]
fn applied_template_survives_save_and_exiftool() {
    let template = XmpProperties {
        title: Some("Template Title".into()),
        credit: Some("Template Credit".into()),
        ..Default::default()
    };
    for mode in [
        MergeMode::Append,
        MergeMode::Replace,
        MergeMode::KeepOriginalReplaceMatching,
    ] {
        let bytes = std::fs::read(fixture("metadata.psd")).expect("read fixture");
        let mut doc = read_psd(&bytes).expect("fixture parses");
        if mode == MergeMode::Append {
            // Empty the fields first so Append has something to fill.
            set_file_info_fields(&mut doc, &[(2, 5, String::new()), (2, 110, String::new())]);
        }
        let exif_before = read_metadata(&doc).exif.clone();
        assert!(apply_template(&mut doc, &template, mode), "{mode:?}");

        let out = write_psd(&doc).expect("write");
        let back = read_psd(&out).expect("re-read");
        assert_eq!(
            xmp_properties(&back).title.as_deref(),
            Some("Template Title"),
            "{mode:?}: XMP title"
        );
        assert_eq!(
            xmp_properties(&back).credit.as_deref(),
            Some("Template Credit"),
            "{mode:?}: XMP credit"
        );
        assert_eq!(
            read_metadata(&back).iptc.text(2, 5).as_deref(),
            Some("Template Title"),
            "{mode:?}: IIM is synced"
        );
        assert_eq!(
            read_metadata(&back).iptc.text(2, 110).as_deref(),
            Some("Template Credit"),
            "{mode:?}: IIM credit is synced"
        );
        assert_eq!(
            read_metadata(&back).exif,
            exif_before,
            "{mode:?}: EXIF survives"
        );
        let raw = read_metadata(&back).xmp;
        assert!(
            raw.contains("acme:Marker=\"keep-me\""),
            "{mode:?}: unknown namespace attribute survives"
        );
        assert!(
            raw.contains("<acme:Note>keep-me-too</acme:Note>"),
            "{mode:?}: unknown-namespace property survives"
        );

        if !exiftool_available() {
            eprintln!("skipping exiftool template comparison: exiftool not available");
            continue;
        }
        let dir =
            std::env::temp_dir().join(format!("pictura-template-{}-{mode:?}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("template.psd");
        std::fs::write(&path, &out).unwrap();
        let values = exiftool_fields(&path, &["-XMP-dc:Title", "-XMP-photoshop:Credit"])
            .expect("exiftool is available but produced no values");
        assert_eq!(values[0], "Template Title", "{mode:?}: exiftool title");
        assert_eq!(values[1], "Template Credit", "{mode:?}: exiftool credit");
        // Negative control: a wrong value would not compare equal.
        assert_ne!(values[0], "Not The Title", "{mode:?}");
        assert_ne!(values[1], "Not The Credit", "{mode:?}");
    }
}
