//! Differential tests against an independent oracle.
//!
//! The fixtures in `tests/fixtures/` are authored by the Python `psd-tools`
//! library (`scripts/generate-fixtures.py`), not by this crate. Anything the
//! Rust codec reads here proves it agrees with a separate implementation of
//! the PSD format rather than only with its own writer.
//!
//! Regenerate the fixtures with `python3 scripts/generate-fixtures.py`.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use pictura_codec::{read_psd, write_psd};
use pictura_core::{
    BitDepth, BlendMode, Channel, ColorLabel, ColorMode, Document, Layer, LockFlags, PsdRect,
};

const FIXTURES: &[(&str, u32, u32, ColorMode)] = &[
    ("two_layers.psd", 8, 8, ColorMode::Rgb),
    ("group.psd", 8, 8, ColorMode::Rgb),
    ("masked.psd", 8, 8, ColorMode::Rgb),
    ("gray.psd", 8, 8, ColorMode::Grayscale),
    ("adjustment.psd", 8, 8, ColorMode::Rgb),
];

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn load(name: &str) -> pictura_core::Document {
    let path = fixture_dir().join(name);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "cannot read {} ({e}); run `python3 scripts/generate-fixtures.py`",
            path.display()
        )
    });
    read_psd(&bytes).unwrap_or_else(|e| panic!("{} failed to parse: {e}", path.display()))
}

/// Independent of layer support: every oracle fixture must parse as a valid
/// document with the dimensions and color mode the generator chose.
#[test]
fn all_fixtures_parse_with_expected_dimensions() {
    for (name, width, height, mode) in FIXTURES {
        let doc = load(name);
        assert_eq!(doc.width, *width, "{name}: width");
        assert_eq!(doc.height, *height, "{name}: height");
        assert_eq!(doc.mode, *mode, "{name}: color mode");
    }
}

// The tests below assert the layer tree read from the psd-tools-authored
// fixtures: the codec must agree with an independent PSD implementation.

#[test]
fn two_layers_tree() {
    let doc = load("two_layers.psd");
    assert_eq!(doc.layers.len(), 2, "two named pixel layers");
    // Bottom layer on disk: "Red", bounds (top=0, left=0, bottom=4, right=4).
    let bottom = &doc.layers[0];
    assert_eq!(bottom.name, "Red");
    assert!(!bottom.is_group());
    assert_eq!(
        (
            bottom.rect.top,
            bottom.rect.left,
            bottom.rect.bottom,
            bottom.rect.right
        ),
        (0, 0, 4, 4)
    );
    // Top layer on disk: "Blue", bounds (top=4, left=4, bottom=8, right=8).
    let top = &doc.layers[1];
    assert_eq!(top.name, "Blue");
    assert_eq!(
        (top.rect.top, top.rect.left, top.rect.bottom, top.rect.right),
        (4, 4, 8, 8)
    );
    assert_eq!(bottom.blend, BlendMode::Normal);
    assert_eq!(bottom.opacity, 255);
}

#[test]
fn group_tree() {
    let doc = load("group.psd");
    assert_eq!(doc.layers.len(), 1);
    let group = &doc.layers[0];
    assert!(group.is_group());
    assert_eq!(group.name, "Group A");
    // psd-tools authors groups with the default Pass Through blend, stored in
    // the 'lsct' block (the folder record's own key is 'norm').
    assert_eq!(group.blend, BlendMode::PassThrough);
    assert_eq!(group.children.len(), 2);
    // Children (bottom-first): "Inner Green", "Inner Yellow".
    assert_eq!(group.children[0].name, "Inner Green");
    assert_eq!(group.children[1].name, "Inner Yellow");
}

#[test]
fn masked_layer_has_mask() {
    let doc = load("masked.psd");
    assert_eq!(doc.layers.len(), 1);
    let layer = &doc.layers[0];
    assert_eq!(layer.name, "Masked");
    let mask = layer
        .mask
        .as_ref()
        .expect("masked.psd has a raster layer mask");
    assert_eq!(
        (
            mask.rect.top,
            mask.rect.left,
            mask.rect.bottom,
            mask.rect.right
        ),
        (0, 0, 8, 8)
    );
    assert_eq!(mask.data.as_ref().map(|d| d.len()), Some(64));
}

#[test]
fn gray_layer_tree() {
    let doc = load("gray.psd");
    assert_eq!(doc.layers.len(), 1);
    assert_eq!(doc.layers[0].name, "Gray");
}

/// Adjustment layers authored by psd-tools: keys and payload bytes must survive
/// read, and the codec must write the same key+bytes back unchanged.
#[test]
fn adjustment_layers_preserve_key_and_bytes() {
    let doc = load("adjustment.psd");
    let names: Vec<&str> = doc.layers.iter().map(|l| l.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "Base",
            "Invert",
            "Posterize",
            "Threshold",
            "BrightnessContrast",
            "Levels",
        ]
    );

    let find = |n: &str| doc.layers.iter().find(|l| l.name == n).unwrap();
    let invert = find("Invert").adjustment.as_ref().expect("invert block");
    assert_eq!(invert.key, *b"nvrt");
    assert!(invert.data.is_empty(), "Invert has no payload");
    assert_eq!(
        find("Posterize").adjustment.as_ref().unwrap().data,
        [0, 4, 0, 0]
    );
    assert_eq!(
        find("Threshold").adjustment.as_ref().unwrap().data,
        [0, 128, 0, 0]
    );
    let brit = find("BrightnessContrast").adjustment.as_ref().unwrap();
    assert_eq!(brit.key, *b"brit");
    assert_eq!(brit.data, [0, 10, 0, 20, 0, 0, 0, 0]);
    let levl = find("Levels").adjustment.as_ref().unwrap();
    assert_eq!(levl.key, *b"levl");
    assert_eq!(levl.data.len(), 292);

    // Round-trip through pictura-codec: whole document, including adjustments.
    let back = read_psd(&write_psd(&doc).unwrap()).unwrap();
    assert_eq!(back, doc);
}

/// Extra/alpha channels: a PSD written by `pictura-codec` with one extra plane
/// is read by `psd-tools`, which must report the bumped header channel count and
/// expose our plane as the composite alpha. Independent of our own reader, this
/// proves the alternate-channel layout matches a second PSD implementation.
#[test]
fn psd_tools_sees_written_extra_channel() {
    if !psd_tools_available() {
        eprintln!("skipping: python3 + psd-tools not available");
        return;
    }

    let mut doc = Document::new(4, 2, ColorMode::Rgb, BitDepth::Eight);
    for (i, b) in doc.composite.data.iter_mut().enumerate() {
        *b = (i * 9 + 1) as u8;
    }
    let alpha: Vec<u8> = (0..8).map(|i| 200 + i as u8).collect();
    doc.channels = vec![Channel {
        id: 0,
        data: alpha.clone(),
    }];

    let dir = scratch_dir("psd-alpha");
    let path = dir.join("extra_channel.psd");
    std::fs::write(&path, write_psd(&doc).unwrap()).unwrap();

    let script = r#"
import sys
from psd_tools import PSDImage
psd = PSDImage.open(sys.argv[1])
print(psd.channels)
print(psd.composite().getchannel("A").tobytes().hex())
"#;
    let out = Command::new("python3")
        .arg("-c")
        .arg(script)
        .arg(&path)
        .output()
        .expect("run python3");
    let _ = std::fs::remove_dir_all(&dir);

    assert!(
        out.status.success(),
        "psd-tools failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut lines = stdout.lines();
    let channels: u32 = lines
        .next()
        .expect("channel count line")
        .trim()
        .parse()
        .unwrap();
    let alpha_hex = lines.next().expect("alpha hex line").trim();
    let expected: String = alpha.iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(channels, 4, "psd-tools sees color + extra channels");
    assert_eq!(
        alpha_hex, expected,
        "psd-tools alpha equals the extra plane"
    );
}

/// M36: the independent oracle reads back the `lspf`/`lclr`/`iOpa` tags with
/// the values the model set.
#[test]
fn psd_tools_sees_layer_attributes() {
    if !psd_tools_available() {
        eprintln!("skipping: python3 + psd-tools not available");
        return;
    }

    let mut doc = Document::new(2, 2, ColorMode::Rgb, BitDepth::Eight);
    for (i, b) in doc.composite.data.iter_mut().enumerate() {
        *b = (i * 3 + 1) as u8;
    }
    let layer = Layer {
        name: "Attrs".into(),
        rect: PsdRect {
            top: 0,
            left: 0,
            bottom: 2,
            right: 2,
        },
        blend: BlendMode::Normal,
        opacity: 255,
        fill: 128,
        lock: LockFlags::default()
            .with(LockFlags::TRANSPARENCY, true)
            .with(LockFlags::POSITION, true),
        color: ColorLabel::Violet,
        clipping: false,
        visible: true,
        mask: None,
        adjustment: None,
        channels: vec![
            Channel {
                id: 0,
                data: vec![7; 4],
            },
            Channel {
                id: 1,
                data: vec![7; 4],
            },
            Channel {
                id: 2,
                data: vec![7; 4],
            },
        ],
        children: Vec::new(),
        is_group: false,
    };
    doc.layers = vec![layer];

    let dir = scratch_dir("psd-attrs");
    let path = dir.join("attrs.psd");
    std::fs::write(&path, write_psd(&doc).unwrap()).unwrap();

    let script = r#"
import sys
from psd_tools import PSDImage
psd = PSDImage.open(sys.argv[1])
layer = psd[0]
print(layer.fill_opacity)
print(1 if layer.locks.transparency else 0)
print(1 if layer.locks.composite else 0)
print(1 if layer.locks.position else 0)
print(layer.sheet_color.value)
"#;
    let out = Command::new("python3")
        .arg("-c")
        .arg(script)
        .arg(&path)
        .output()
        .expect("run python3");
    let _ = std::fs::remove_dir_all(&dir);

    assert!(
        out.status.success(),
        "psd-tools failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let got: Vec<i32> = stdout.lines().map(|l| l.trim().parse().unwrap()).collect();
    assert_eq!(
        got,
        vec![128, 1, 0, 1, 6],
        "psd-tools must report fill=128, transparency+position locks, violet"
    );
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

/// Unique scratch directory per call so tests can run in parallel.
fn scratch_dir(tag: &str) -> PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "pictura-codec-oracle-{}-{tag}-{n}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
