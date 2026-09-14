//! Differential tests against an independent oracle.
//!
//! The fixtures in `tests/fixtures/` are authored by the Python `psd-tools`
//! library (`scripts/generate-fixtures.py`), not by this crate. Anything the
//! Rust codec reads here proves it agrees with a separate implementation of
//! the PSD format rather than only with its own writer.
//!
//! Regenerate the fixtures with `python3 scripts/generate-fixtures.py`.

use std::path::PathBuf;

use pictura_codec::read_psd;
use pictura_core::{BlendMode, ColorMode};

const FIXTURES: &[(&str, u32, u32, ColorMode)] = &[
    ("two_layers.psd", 8, 8, ColorMode::Rgb),
    ("group.psd", 8, 8, ColorMode::Rgb),
    ("masked.psd", 8, 8, ColorMode::Rgb),
    ("gray.psd", 8, 8, ColorMode::Grayscale),
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
