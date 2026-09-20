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

use pictura_codec::{read_descriptor, read_psd, write_psb, write_psd, DescValue};
use pictura_core::{
    BitDepth, BlendMode, Channel, ColorLabel, ColorMode, Document, Layer, LayerBlock, LockFlags,
    PsdRect, SmartObject, SmartObjectKind,
};

const FIXTURES: &[(&str, u32, u32, ColorMode)] = &[
    ("two_layers.psd", 8, 8, ColorMode::Rgb),
    ("group.psd", 8, 8, ColorMode::Rgb),
    ("masked.psd", 8, 8, ColorMode::Rgb),
    ("gray.psd", 8, 8, ColorMode::Grayscale),
    ("adjustment.psd", 8, 8, ColorMode::Rgb),
    ("gradient_map.psd", 8, 8, ColorMode::Rgb),
    ("solid_fill.psd", 8, 8, ColorMode::Rgb),
    ("gradient_fill.psd", 8, 8, ColorMode::Rgb),
    ("pattern_fill.psd", 8, 8, ColorMode::Rgb),
    ("pattern_fill_16bit.psd", 8, 8, ColorMode::Rgb),
    ("drop_shadow.psd", 8, 8, ColorMode::Rgb),
    ("outer_glow.psd", 8, 8, ColorMode::Rgb),
    ("inner_shadow.psd", 8, 8, ColorMode::Rgb),
    ("inner_glow.psd", 8, 8, ColorMode::Rgb),
    ("stroke.psd", 8, 8, ColorMode::Rgb),
    ("color_overlay.psd", 8, 8, ColorMode::Rgb),
    ("gradient_overlay.psd", 8, 8, ColorMode::Rgb),
    ("pattern_overlay.psd", 8, 8, ColorMode::Rgb),
    ("satin.psd", 8, 8, ColorMode::Rgb),
    ("bevel.psd", 8, 8, ColorMode::Rgb),
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
            "PhotoFilter",
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
    let phfl = find("PhotoFilter").adjustment.as_ref().unwrap();
    assert_eq!(phfl.key, *b"phfl");
    assert_eq!(
        phfl.data,
        [0, 2, 0, 0, 0, 255, 0, 180, 0, 80, 0, 0, 0, 0, 0, 25, 1, 0, 0, 0,],
        "version 2, sRGB (255,180,80), density 25, luminosity"
    );

    // Round-trip through pictura-codec: whole document, including adjustments.
    let back = read_psd(&write_psd(&doc).unwrap()).unwrap();
    assert_eq!(back, doc);
}

/// The Gradient Map fixture: the psd-tools-authored `grdm` block survives read
/// and whole-document round-trip unchanged.
#[test]
fn gradient_map_layer_preserves_key_and_bytes() {
    let doc = load("gradient_map.psd");
    let names: Vec<&str> = doc.layers.iter().map(|l| l.name.as_str()).collect();
    assert_eq!(names, ["Base", "Gradient Map"]);

    let gm = doc
        .layers
        .iter()
        .find(|l| l.name == "Gradient Map")
        .and_then(|l| l.adjustment.as_ref())
        .expect("gradient map adjustment block");
    assert_eq!(gm.key, *b"grdm");
    assert_eq!(gm.data.len(), 140, "psd-tools version-1 grdm payload");
    assert_eq!(&gm.data[0..2], &[0, 1], "version 1");
    assert_eq!(gm.data[2], 0, "not reversed");
    assert_eq!(gm.data[3], 0, "not dithered");

    let back = read_psd(&write_psd(&doc).unwrap()).unwrap();
    assert_eq!(back, doc);
}

/// The solid-color fill fixture: the psd-tools-authored `SoCo` descriptor
/// survives read and whole-document round-trip, with the expected `Clr `
/// `RGBC` doubles.
#[test]
fn solid_fill_layer_preserves_descriptor() {
    let doc = load("solid_fill.psd");
    let names: Vec<&str> = doc.layers.iter().map(|l| l.name.as_str()).collect();
    assert_eq!(names, ["Base", "Solid Fill"]);

    let adj = doc
        .layers
        .iter()
        .find(|l| l.name == "Solid Fill")
        .and_then(|l| l.adjustment.as_ref())
        .expect("solid fill adjustment block");
    assert_eq!(adj.key, *b"SoCo");

    let desc = read_descriptor(&adj.data).expect("version-16 descriptor");
    let DescValue::Object { items, .. } = desc else {
        panic!("top level is an object");
    };
    let clr = items
        .iter()
        .find(|(k, _)| k.as_slice() == b"Clr ")
        .map(|(_, v)| v)
        .expect("Clr item");
    let DescValue::Object {
        class_id, items, ..
    } = clr
    else {
        panic!("Clr is an object");
    };
    assert_eq!(class_id, b"RGBC");
    let get = |key: &[u8]| {
        items
            .iter()
            .find(|(k, _)| k.as_slice() == key)
            .map(|(_, v)| v)
            .expect("component")
    };
    assert_eq!(get(b"Rd  "), &DescValue::Double(10.0));
    assert_eq!(get(b"Grn "), &DescValue::Double(20.0));
    assert_eq!(get(b"Bl  "), &DescValue::Double(30.0));

    let back = read_psd(&write_psd(&doc).unwrap()).unwrap();
    assert_eq!(back, doc);
}

/// The gradient fill fixture: the psd-tools-authored `GdFl` descriptor survives
/// read and whole-document round-trip, with the expected kind, angle, and stops.
#[test]
fn gradient_fill_layer_preserves_descriptor() {
    fn get<'a>(obj: &'a DescValue, key: &[u8]) -> Option<&'a DescValue> {
        let DescValue::Object { items, .. } = obj else {
            return None;
        };
        items
            .iter()
            .find(|(k, _)| k.as_slice() == key)
            .map(|(_, v)| v)
    }

    let doc = load("gradient_fill.psd");
    let names: Vec<&str> = doc.layers.iter().map(|l| l.name.as_str()).collect();
    assert_eq!(names, ["Base", "Gradient Fill"]);

    let adj = doc
        .layers
        .iter()
        .find(|l| l.name == "Gradient Fill")
        .and_then(|l| l.adjustment.as_ref())
        .expect("gradient fill adjustment block");
    assert_eq!(adj.key, *b"GdFl");

    let desc = read_descriptor(&adj.data).expect("version-16 descriptor");
    let kind = get(&desc, b"Type").expect("Type enum");
    assert_eq!(
        kind,
        &DescValue::Enum {
            kind: b"GrdT".to_vec(),
            value: b"Lnr ".to_vec(),
        }
    );
    assert_eq!(get(&desc, b"Angl"), Some(&DescValue::Double(0.0)));

    let grad = get(&desc, b"Grad").expect("Grad object");
    let DescValue::List(clrs) = get(grad, b"Clrs").expect("Clrs list") else {
        panic!("Clrs is a list");
    };
    assert_eq!(clrs.len(), 2, "two stops");
    let components: Vec<(u16, u8, u8, u8)> = clrs
        .iter()
        .map(|stop| {
            let clr = get(stop, b"Clr ").expect("Clr object");
            let c = |key: &[u8]| match get(clr, key) {
                Some(DescValue::Double(v)) => v.round() as u8,
                other => panic!("{key:?} not a double: {other:?}"),
            };
            let location = match get(stop, b"Lctn") {
                Some(DescValue::Double(v)) => v.round() as u16,
                other => panic!("Lctn not a double: {other:?}"),
            };
            (location, c(b"Rd  "), c(b"Grn "), c(b"Bl  "))
        })
        .collect();
    assert_eq!(components, [(0, 0, 0, 0), (4096, 255, 255, 255)]);

    let back = read_psd(&write_psd(&doc).unwrap()).unwrap();
    assert_eq!(back, doc);
}

/// The pattern fill fixture: the psd-tools-authored `PtFl` descriptor and the
/// `Patt` tagged block's pixels survive read and whole-document round-trip.
#[test]
fn pattern_fill_layer_preserves_descriptor_and_pattern() {
    fn get<'a>(obj: &'a DescValue, key: &[u8]) -> Option<&'a DescValue> {
        let DescValue::Object { items, .. } = obj else {
            return None;
        };
        items
            .iter()
            .find(|(k, _)| k.as_slice() == key)
            .map(|(_, v)| v)
    }

    let doc = load("pattern_fill.psd");
    let names: Vec<&str> = doc.layers.iter().map(|l| l.name.as_str()).collect();
    assert_eq!(names, ["Base", "Pattern Fill"]);

    let adj = doc
        .layers
        .iter()
        .find(|l| l.name == "Pattern Fill")
        .and_then(|l| l.adjustment.as_ref())
        .expect("pattern fill adjustment block");
    assert_eq!(
        adj.key, *b"PtFl",
        "the PtFl block is routed to layer.adjustment"
    );

    let desc = read_descriptor(&adj.data).expect("version-16 descriptor");
    let ptrn = get(&desc, b"Ptrn").expect("Ptrn object");
    assert_eq!(
        get(ptrn, b"Idnt"),
        Some(&DescValue::Text("pictura-pattern\0".into())),
        "the descriptor references the pattern by id"
    );

    let patterns = pictura_codec::decode_patterns(&doc);
    assert_eq!(patterns.len(), 1, "one Patt pattern");
    assert_eq!(patterns[0].pattern_id, "pictura-pattern");
    assert_eq!((patterns[0].width, patterns[0].height), (2, 2));
    assert_eq!(
        patterns[0].rgba,
        vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,],
        "row-major red, green / blue, white"
    );

    let back = read_psd(&write_psd(&doc).unwrap()).unwrap();
    assert_eq!(back, doc, "the whole document round-trips");
    assert_eq!(
        pictura_codec::decode_patterns(&back),
        patterns,
        "the Patt pixels survive a round-trip"
    );
}

/// The 16-bit pattern fixture: the `PtFl` block is still recognised, but the
/// 16-bit pattern planes are skipped (not misdecoded as 8-bit pixels), and the
/// document still round-trips.
#[test]
fn pattern_fill_16bit_pattern_is_skipped() {
    let doc = load("pattern_fill_16bit.psd");
    let adj = doc
        .layers
        .iter()
        .find(|l| l.name == "Pattern Fill 16")
        .and_then(|l| l.adjustment.as_ref())
        .expect("pattern fill adjustment block");
    assert_eq!(adj.key, *b"PtFl");
    assert!(
        pictura_codec::decode_patterns(&doc).is_empty(),
        "16-bit pattern planes are skipped"
    );

    let back = read_psd(&write_psd(&doc).unwrap()).unwrap();
    assert_eq!(back, doc, "the whole document round-trips");
}

/// The drop-shadow fixture: the psd-tools-authored `lfx2` object-based effects
/// block survives read and whole-document round-trip, and psd-tools reads the
/// effect back as a `DropShadow` with the authored values.
#[test]
fn drop_shadow_layer_preserves_effect_block() {
    let doc = load("drop_shadow.psd");
    let layer = doc
        .layers
        .iter()
        .find(|l| l.name == "Shadowed")
        .expect("Shadowed layer");
    let block = layer
        .extra_block(b"lfx2")
        .expect("lfx2 is preserved in extra_blocks");
    assert!(block.data.len() > 8, "DescriptorBlock2 payload");
    assert_eq!(&block.data[0..4], &[0, 0, 0, 1], "version 1");
    assert_eq!(&block.data[4..8], &[0, 0, 0, 16], "data version 16");

    let back = read_psd(&write_psd(&doc).unwrap()).unwrap();
    assert_eq!(back, doc, "the whole document round-trips");
    assert!(
        back.layers
            .iter()
            .find(|l| l.name == "Shadowed")
            .and_then(|l| l.extra_block(b"lfx2"))
            .is_some(),
        "lfx2 survives a round-trip"
    );

    if !psd_tools_available() {
        eprintln!("skipping: python3 + psd-tools not available");
        return;
    }
    let dir = scratch_dir("psd-drop-shadow");
    let path = dir.join("drop_shadow.psd");
    std::fs::write(
        &path,
        std::fs::read(fixture_dir().join("drop_shadow.psd")).unwrap(),
    )
    .unwrap();

    let script = r#"
import sys
from psd_tools import PSDImage
psd = PSDImage.open(sys.argv[1], lazy=False)
layer = [l for l in psd if l.name == "Shadowed"][0]
effect = layer.effects.items[0]
print(type(effect).__name__)
print(1 if effect.enabled else 0)
print(1 if effect.present else 0)
print(effect.opacity)
print(effect.blend_mode.decode())
print(int(round(effect.distance)))
print(int(round(effect.size)))
print(1 if effect.use_global_light else 0)
print(1 if effect.layer_knocks_out else 0)
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
    let got: Vec<&str> = stdout.lines().map(str::trim).collect();
    assert_eq!(
        got,
        ["DropShadow", "1", "1", "75.0", "Mltp", "5", "5", "0", "0",],
        "psd-tools reads the authored DrSh effect; stdout={stdout}"
    );
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

/// The independent oracle decodes both the RLE composite section and
/// engine-encoded layer channels: psd-tools must recover the exact pixels.
#[test]
fn psd_tools_reads_rle_composite_and_layer() {
    if !psd_tools_available() {
        eprintln!("skipping: python3 + psd-tools not available");
        return;
    }

    let mut doc = Document::new(4, 2, ColorMode::Rgb, BitDepth::Eight);
    let composite: Vec<u8> = (0..24).map(|i| (i * 7 + 1) as u8).collect();
    doc.composite.data = composite.clone();
    let planes: Vec<Vec<u8>> = (0..4u8)
        .map(|c| (0..8).map(|i| c * 20 + i).collect())
        .collect();
    doc.layers = vec![Layer {
        name: "Rle".into(),
        rect: PsdRect {
            top: 0,
            left: 0,
            bottom: 2,
            right: 4,
        },
        blend: BlendMode::Normal,
        opacity: 255,
        fill: 255,
        lock: LockFlags::default(),
        color: ColorLabel::None,
        clipping: false,
        visible: true,
        mask: None,
        adjustment: None,
        channels: vec![
            Channel {
                id: 0,
                data: planes[0].clone(),
            },
            Channel {
                id: 1,
                data: planes[1].clone(),
            },
            Channel {
                id: 2,
                data: planes[2].clone(),
            },
            Channel {
                id: -1,
                data: planes[3].clone(),
            },
        ],
        children: Vec::new(),
        is_group: false,
        background: false,
        ..Default::default()
    }];

    let dir = scratch_dir("psd-rle-read");
    let path = dir.join("rle.psd");
    std::fs::write(&path, write_psd(&doc).unwrap()).unwrap();

    let script = r#"
import sys
import numpy as np
from psd_tools import PSDImage
psd = PSDImage.open(sys.argv[1], lazy=False)
arr = np.array(psd.composite().convert("RGB"))
print(arr[:, :, 0].tobytes().hex())
print(arr[:, :, 1].tobytes().hex())
print(arr[:, :, 2].tobytes().hex())
print(np.round(psd[0].numpy()[:, :, 0] * 255).astype(np.uint8).tobytes().hex())
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
    let got: Vec<&str> = stdout.lines().map(str::trim).collect();
    let hex = |bytes: &[u8]| bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
    assert_eq!(got[0], hex(&composite[0..8]), "composite R plane");
    assert_eq!(got[1], hex(&composite[8..16]), "composite G plane");
    assert_eq!(got[2], hex(&composite[16..24]), "composite B plane");
    assert_eq!(got[3], hex(&planes[0]), "layer R channel");
}

/// The independent oracle opens a codec-written version-2 PSB and decodes the
/// same composite pixels.
#[test]
fn psd_tools_reads_written_psb_composite() {
    if !psd_tools_available() {
        eprintln!("skipping: python3 + psd-tools not available");
        return;
    }

    let mut doc = Document::new(4, 2, ColorMode::Rgb, BitDepth::Eight);
    let composite: Vec<u8> = (0..24).map(|i| (i * 7 + 1) as u8).collect();
    doc.composite.data = composite.clone();

    let dir = scratch_dir("psb-write-read");
    let path = dir.join("written.psb");
    let bytes = write_psb(&doc).unwrap();
    assert_eq!(
        u16::from_be_bytes([bytes[4], bytes[5]]),
        2,
        "PSB version word"
    );
    std::fs::write(&path, &bytes).unwrap();

    let script = r#"
import sys
import numpy as np
from psd_tools import PSDImage
psd = PSDImage.open(sys.argv[1], lazy=False)
arr = np.array(psd.composite().convert("RGB"))
print(arr[:, :, 0].tobytes().hex())
print(arr[:, :, 1].tobytes().hex())
print(arr[:, :, 2].tobytes().hex())
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
    let got: Vec<&str> = stdout.lines().map(str::trim).collect();
    let hex = |bytes: &[u8]| bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
    assert_eq!(got[0], hex(&composite[0..8]), "PSB composite R plane");
    assert_eq!(got[1], hex(&composite[8..16]), "PSB composite G plane");
    assert_eq!(got[2], hex(&composite[16..24]), "PSB composite B plane");
}

/// A 4x4 RGB document with one layer holding an embedded smart object, which
/// the writer authors as a `SoLd` block plus a document-level `lnk2` record.
fn embedded_smart_doc() -> Document {
    let mut doc = Document::new(4, 4, ColorMode::Rgb, BitDepth::Eight);
    for (i, b) in doc.composite.data.iter_mut().enumerate() {
        *b = (i * 7 + 1) as u8;
    }
    doc.layers = vec![Layer {
        name: "Smart".into(),
        rect: PsdRect {
            top: 0,
            left: 0,
            bottom: 4,
            right: 4,
        },
        channels: vec![
            Channel {
                id: 0,
                data: vec![10; 16],
            },
            Channel {
                id: 1,
                data: vec![20; 16],
            },
            Channel {
                id: 2,
                data: vec![30; 16],
            },
            Channel {
                id: -1,
                data: vec![255; 16],
            },
        ],
        smart_object: Some(SmartObject {
            filename: "source.psb".into(),
            kind: SmartObjectKind::Embedded,
            payload: Some((0..4096).map(|i| (i % 251) as u8).collect()),
            ..Default::default()
        }),
        ..Default::default()
    }];
    doc
}

/// Run psd-tools over `bytes`, asserting `PSD.read` and `PSDImage.open` both
/// succeed, and return its stdout.
fn psd_tools_reads(tag: &str, bytes: &[u8]) -> String {
    let dir = scratch_dir(tag);
    let path = dir.join("doc.psb");
    std::fs::write(&path, bytes).unwrap();

    let script = r#"
import sys
from psd_tools import PSDImage
from psd_tools.psd import PSD
with open(sys.argv[1], "rb") as f:
    PSD.read(f)
image = PSDImage.open(sys.argv[1], lazy=False)
so = image[0].smart_object
print(so.kind)
print(so.filename)
print(len(so.data))
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
        "psd-tools failed to parse:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Assert `psd_tools_reads` stdout reports the authored embedded smart object.
fn assert_embedded_smart_object(stdout: &str) {
    let mut lines = stdout.lines();
    assert_eq!(
        lines.next().map(str::trim),
        Some("data"),
        "psd-tools sees the embedded smart object; stdout={stdout}"
    );
    assert_eq!(
        lines.next().map(str::trim),
        Some("source.psb"),
        "psd-tools reports the authored filename; stdout={stdout}"
    );
    let len: usize = lines
        .next()
        .expect("embedded data length line")
        .trim()
        .parse()
        .unwrap();
    assert_eq!(len, 4096, "embedded payload length matches");
}

/// CRITICAL repro: a PSB carrying an authored smart object frames its `lnk2`
/// big-key block with a `u64` length, so psd-tools parses the whole document
/// without a framing error (previously a MemoryError).
#[test]
fn psd_tools_parses_authored_smart_object_in_psb() {
    if !psd_tools_available() {
        eprintln!("skipping: python3 + psd-tools not available");
        return;
    }
    let bytes = write_psb(&embedded_smart_doc()).unwrap();
    let stdout = psd_tools_reads("psb-authored-smart-object", &bytes);
    assert_embedded_smart_object(&stdout);
}

/// CRITICAL-1 regression: an odd-length tagged block is declared even with its
/// pad inside, so psd-tools still finds the block that follows it.
#[test]
fn psd_tools_parses_block_after_odd_length_block_in_psb() {
    if !psd_tools_available() {
        eprintln!("skipping: python3 + psd-tools not available");
        return;
    }
    let mut doc = embedded_smart_doc();
    // A 5-byte (odd) and a 6-byte (non-4-multiple) per-layer block precede the
    // authored `SoLd`. With the old external pad the next `8BIM` is misread and
    // the smart object is dropped.
    doc.layers[0].extra_blocks = vec![
        LayerBlock {
            key: *b"zzzz",
            data: vec![1, 2, 3, 4, 5],
        },
        LayerBlock {
            key: *b"yyyy",
            data: vec![1, 2, 3, 4, 5, 6],
        },
    ];
    let bytes = write_psb(&doc).unwrap();
    let stdout = psd_tools_reads("psb-odd-block-then-smart", &bytes);
    assert_embedded_smart_object(&stdout);
}

/// CRITICAL-2 regression: a PSD-sourced document whose preserved `lnk2` is
/// u32-framed is re-framed to u64 when written as a PSB, so psd-tools reads it.
#[test]
fn psd_tools_parses_reframed_psd_big_key_in_psb() {
    if !psd_tools_available() {
        eprintln!("skipping: python3 + psd-tools not available");
        return;
    }
    let psd_bytes = write_psd(&embedded_smart_doc()).unwrap();
    let src = read_psd(&psd_bytes).unwrap();
    assert!(!src.is_psb, "the intermediate is a PSD");

    let bytes = write_psb(&src).unwrap();
    assert_eq!(
        u16::from_be_bytes([bytes[4], bytes[5]]),
        2,
        "the re-save is a PSB"
    );
    let stdout = psd_tools_reads("psb-reframed-psd-big-key", &bytes);
    assert_embedded_smart_object(&stdout);
}

/// Fix A regression: a preserved document-level block whose length is not a
/// multiple of 4 declares its exact length and is padded externally to 4, so
/// psd-tools still reads the authored `lnk2` that follows it.
#[test]
fn psd_tools_parses_doc_level_block_not_multiple_of_four_in_psb() {
    if !psd_tools_available() {
        eprintln!("skipping: python3 + psd-tools not available");
        return;
    }
    let mut doc = embedded_smart_doc();
    // A 6-byte unknown global block: exact length 6 + 2 external pad bytes, which
    // the old per-layer even-inside rule would have mis-framed.
    let mut block = b"8BIMzzzz".to_vec();
    block.extend_from_slice(&6u32.to_be_bytes());
    block.extend_from_slice(&[1, 2, 3, 4, 5, 6]);
    block.extend_from_slice(&[0, 0]);
    doc.layer_section_extra = block;

    let bytes = write_psb(&doc).unwrap();
    let stdout = psd_tools_reads("psb-doc-block-not-mult-of-four", &bytes);
    assert_embedded_smart_object(&stdout);
}

/// ImageMagick opens a written RLE document and reports its dimensions.
#[test]
fn imagemagick_reads_written_rle() {
    if Command::new("magick").arg("-version").output().is_err() {
        eprintln!("skipping: `magick` not on PATH");
        return;
    }

    let mut doc = Document::new(4, 2, ColorMode::Rgb, BitDepth::Eight);
    for (i, b) in doc.composite.data.iter_mut().enumerate() {
        *b = (i * 7 + 1) as u8;
    }

    let dir = scratch_dir("psd-rle-magick");
    let path = dir.join("rle.psd");
    std::fs::write(&path, write_psd(&doc).unwrap()).unwrap();

    let out = Command::new("magick")
        .arg("identify")
        .arg("-format")
        .arg("%wx%h")
        .arg(&path)
        .output()
        .expect("run magick");
    let _ = std::fs::remove_dir_all(&dir);

    assert!(
        out.status.success(),
        "magick failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "4x2");
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
        background: false,
        ..Default::default()
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

/// The codec writes a flagged layer under the `"Background"` name even when the
/// model layer was named differently, and psd-tools reads that name back.
#[test]
fn psd_tools_sees_background_name_for_flagged_layer() {
    if !psd_tools_available() {
        eprintln!("skipping: python3 + psd-tools not available");
        return;
    }

    let mut doc = Document::new(2, 2, ColorMode::Rgb, BitDepth::Eight);
    let layer = Layer {
        name: "Base".into(),
        rect: PsdRect {
            top: 0,
            left: 0,
            bottom: 2,
            right: 2,
        },
        blend: BlendMode::Normal,
        opacity: 255,
        fill: 255,
        lock: LockFlags::default(),
        color: ColorLabel::None,
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
        background: true,
        ..Default::default()
    };
    doc.layers = vec![layer];

    let dir = scratch_dir("psd-background");
    let path = dir.join("background.psd");
    std::fs::write(&path, write_psd(&doc).unwrap()).unwrap();

    let script = r#"
import sys
from psd_tools import PSDImage
psd = PSDImage.open(sys.argv[1])
print(psd[0].name)
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
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        "Background",
        "a flagged layer is written under the Background name"
    );
}

// -- Hand-built ZIP/ZIP-prediction files, psd-tools as the decoder oracle --
//
// psd-tools' writer only emits raw/RLE, so these files are assembled by hand
// here and psd-tools is used as an independent decoder of the same bytes.

fn psd_header(channels: u16, width: u32, height: u32) -> Vec<u8> {
    let mut v = Vec::new();
    v.extend_from_slice(b"8BPS");
    v.extend_from_slice(&1u16.to_be_bytes());
    v.extend_from_slice(&[0u8; 6]);
    v.extend_from_slice(&channels.to_be_bytes());
    v.extend_from_slice(&height.to_be_bytes());
    v.extend_from_slice(&width.to_be_bytes());
    v.extend_from_slice(&8u16.to_be_bytes());
    v.extend_from_slice(&3u16.to_be_bytes()); // RGB
    v
}

fn zlib_compress(data: &[u8]) -> Vec<u8> {
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::Write;

    let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
    enc.write_all(data).unwrap();
    enc.finish().unwrap()
}

/// Forward byte-wise delta per `row_len` scanline: inverse of the reader's
/// `undo_prediction`.
fn predict_forward(data: &[u8], row_len: usize) -> Vec<u8> {
    let mut out = data.to_vec();
    for row_start in (0..out.len()).step_by(row_len) {
        let row_end = (row_start + row_len).min(out.len());
        for i in (row_start + 1)..row_end {
            out[i] = data[i].wrapping_sub(data[i - 1]);
        }
    }
    out
}

/// A 2x2 RGB PSD with one pixel layer carrying only channel 0, whose data is
/// `compression` followed by `encoded`; the merged composite is raw zeros.
fn one_layer_zip_psd(compression: u16, encoded: &[u8]) -> Vec<u8> {
    let channel_len = 2 + encoded.len();

    let mut extra = Vec::new();
    extra.extend_from_slice(&0u32.to_be_bytes()); // no mask
    extra.extend_from_slice(&0u32.to_be_bytes()); // blending ranges
    extra.extend_from_slice(&[1, b'L', 0, 0]); // pascal name "L"

    let mut rec = Vec::new();
    for v in [0i32, 0, 2, 2] {
        rec.extend_from_slice(&v.to_be_bytes());
    }
    rec.extend_from_slice(&1u16.to_be_bytes()); // one channel
    rec.extend_from_slice(&0i16.to_be_bytes()); // id 0
    rec.extend_from_slice(&(channel_len as u32).to_be_bytes());
    rec.extend_from_slice(b"8BIM");
    rec.extend_from_slice(b"norm");
    rec.push(255);
    rec.push(0);
    rec.push(0);
    rec.push(0);
    rec.extend_from_slice(&(extra.len() as u32).to_be_bytes());
    rec.extend_from_slice(&extra);

    let mut info = Vec::new();
    info.extend_from_slice(&1i16.to_be_bytes());
    info.extend_from_slice(&rec);
    info.extend_from_slice(&compression.to_be_bytes());
    info.extend_from_slice(encoded);
    while info.len() % 4 != 0 {
        info.push(0);
    }

    let mut out = psd_header(3, 2, 2);
    out.extend_from_slice(&0u32.to_be_bytes()); // color mode data
    out.extend_from_slice(&0u32.to_be_bytes()); // image resources
    let section_len = 4 + info.len() + 4;
    out.extend_from_slice(&(section_len as u32).to_be_bytes());
    out.extend_from_slice(&(info.len() as u32).to_be_bytes());
    out.extend_from_slice(&info);
    out.extend_from_slice(&0u32.to_be_bytes()); // global layer mask
    out.extend_from_slice(&0u16.to_be_bytes()); // raw composite
    out.extend_from_slice(&[0u8; 12]); // 2x2 RGB planes
    out
}

/// A ZIP-with-prediction layer channel built by hand must decode, in
/// psd-tools, to the same bytes our reader recovers. psd-tools 1.19 returns
/// `numpy()` scaled to 0..1 floats, so a byte is `round(v * 255)`.
#[test]
fn zip_prediction_layer_channel_matches_psd_tools() {
    if !psd_tools_available() {
        eprintln!("skipping: python3 + psd-tools not available");
        return;
    }

    let expected = [10u8, 20, 30, 40];
    let encoded = zlib_compress(&predict_forward(&expected, 2));
    let psd = one_layer_zip_psd(3, &encoded);

    let dir = scratch_dir("psd-zip-layer");
    let path = dir.join("zip_layer.psd");
    std::fs::write(&path, &psd).unwrap();

    let script = r#"
import sys
from psd_tools import PSDImage
psd = PSDImage.open(sys.argv[1])
layer = psd[0]
arr = layer.numpy()
print(",".join(str(int(round(v * 255))) for v in arr[..., 0].flatten()))
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
        "psd-tools failed on the hand-built file:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let got: Vec<u8> = String::from_utf8_lossy(&out.stdout)
        .trim()
        .split(',')
        .map(|s| s.parse().unwrap())
        .collect();
    let ours = read_psd(&psd).unwrap();
    assert_eq!(ours.layers[0].channels[0].data, expected);
    assert_eq!(
        got, expected,
        "psd-tools decodes the ZIP-prediction channel"
    );
}

/// A bare ZIP-with-prediction composite built by hand must decode, in
/// psd-tools, to the same RGB planes our reader recovers.
#[test]
fn zip_prediction_composite_matches_psd_tools() {
    if !psd_tools_available() {
        eprintln!("skipping: python3 + psd-tools not available");
        return;
    }

    let planes: Vec<u8> = (1..=12).collect();
    let encoded = zlib_compress(&predict_forward(&planes, 2));

    let mut psd = psd_header(3, 2, 2);
    psd.extend_from_slice(&[0u8; 12]); // color mode, resources, empty layer section
    psd.extend_from_slice(&3u16.to_be_bytes());
    psd.extend_from_slice(&encoded);

    let dir = scratch_dir("psd-zip-composite");
    let path = dir.join("zip_composite.psd");
    std::fs::write(&path, &psd).unwrap();

    let script = r#"
import sys
from psd_tools import PSDImage
psd = PSDImage.open(sys.argv[1])
arr = psd.numpy()
vals = []
for c in range(arr.shape[2]):
    vals.extend(int(round(v * 255)) for v in arr[..., c].flatten())
print(",".join(str(v) for v in vals))
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
        "psd-tools failed on the hand-built file:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let got: Vec<u8> = String::from_utf8_lossy(&out.stdout)
        .trim()
        .split(',')
        .map(|s| s.parse().unwrap())
        .collect();
    let ours = read_psd(&psd).unwrap();
    assert_eq!(ours.composite.data, planes);
    assert_eq!(
        got, planes,
        "psd-tools decodes the ZIP-prediction composite"
    );
}

/// Opaque sections survive an open→save round-trip authoritatively: the exact
/// image-resource, color-mode-data, global-mask and trailing layer-section bytes
/// are re-emitted, and the result is still a valid PSD that psd-tools opens and
/// reads the original layer tree from.
#[test]
fn opaque_resources_survive_write_and_psd_tools_still_opens() {
    if !psd_tools_available() {
        eprintln!("skipping: python3 + psd-tools not available");
        return;
    }

    let doc = load("two_layers.psd");
    assert!(
        !doc.image_resources.is_empty(),
        "two_layers.psd carries a non-empty image-resource section"
    );

    let out = write_psd(&doc).unwrap();
    let back = read_psd(&out).unwrap();
    assert_eq!(back.image_resources, doc.image_resources, "image resources");
    assert_eq!(back.color_mode_data, doc.color_mode_data, "color-mode data");
    assert_eq!(back.global_layer_mask, doc.global_layer_mask, "global mask");
    assert_eq!(
        back.layer_section_extra, doc.layer_section_extra,
        "layer-section trailing bytes"
    );

    let dir = scratch_dir("opaque-resources");
    let path = dir.join("opaque.psd");
    std::fs::write(&path, &out).unwrap();

    let script = r#"
import sys
from psd_tools import PSDImage
psd = PSDImage.open(sys.argv[1], lazy=False)
print(len(list(psd)))
for layer in psd:
    print(layer.name)
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
        "psd-tools failed on the re-emitted file:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut lines = stdout.lines();
    let count: usize = lines
        .next()
        .expect("layer count line")
        .trim()
        .parse()
        .unwrap();
    let names: Vec<&str> = lines.collect();
    assert_eq!(count, 2, "psd-tools sees two layers; stdout={stdout}");
    assert_eq!(names, ["Red", "Blue"], "psd-tools layer names");
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

#[path = "oracle/outer_glow.rs"]
mod outer_glow;

#[path = "oracle/inner_shadow.rs"]
mod inner_shadow;

#[path = "oracle/inner_glow.rs"]
mod inner_glow;

#[path = "oracle/stroke.rs"]
mod stroke;

#[path = "oracle/color_overlay.rs"]
mod color_overlay;

#[path = "oracle/gradient_overlay.rs"]
mod gradient_overlay;

#[path = "oracle/pattern_overlay.rs"]
mod pattern_overlay;

#[path = "oracle/satin.rs"]
mod satin;

#[path = "oracle/bevel.rs"]
mod bevel;
