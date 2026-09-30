//! M12-B structural oracle for `pictura_render::document_ops`.
//!
//! These tests check *structure*, not pixels: after each document operation the
//! layer stack must still serialize through `write_psd` → `read_psd` with the
//! same dimensions, layer count, names, and rects, and the cached
//! `doc.composite` must equal `composite_rgba(&doc)`. The exact orientation and
//! canvas remaps are checked for identity, and a `psd-tools` check (skipped when
//! the tool is absent) confirms an independent PSD reader sees the same
//! dimensions and layer count. See `tests/README.md` for the mapping and
//! tolerances.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use pictura_codec::{read_psd, write_psd};
use pictura_core::{
    BitDepth, BlendMode, Channel, ColorLabel, ColorMode, Document, Layer, LayerMask, LockFlags,
    PsdRect,
};
use pictura_ops::{Anchor, Resample};
use pictura_render::{
    composite_rgba, flip_document, resize_canvas_document, resize_document, rotate_document,
};

// --- document builders (same layer-builder style as the unit tests) ---------

fn rect(top: i32, left: i32, bottom: i32, right: i32) -> PsdRect {
    PsdRect {
        top,
        left,
        bottom,
        right,
    }
}

fn plane(base: u8, n: usize) -> Vec<u8> {
    (0..n).map(|i| base.wrapping_add(i as u8)).collect()
}

fn pixel_layer(name: &str, r: PsdRect, mask: Option<LayerMask>) -> Layer {
    let n = (r.width().max(0) * r.height().max(0)) as usize;
    Layer {
        name: name.into(),
        rect: r,
        blend: BlendMode::Normal,
        opacity: 255,
        fill: 255,
        lock: LockFlags::default(),
        color: ColorLabel::None,
        clipping: false,
        visible: true,
        mask,
        adjustment: None,
        channels: vec![
            Channel {
                id: 0,
                data: plane(0, n).into(),
            },
            Channel {
                id: 1,
                data: plane(60, n).into(),
            },
            Channel {
                id: 2,
                data: plane(120, n).into(),
            },
            Channel {
                id: -1,
                data: plane(200, n).into(),
            },
        ],
        children: Vec::new(),
        is_group: false,
        background: false,
        ..Default::default()
    }
}

fn group(name: &str, children: Vec<Layer>) -> Layer {
    Layer {
        name: name.into(),
        rect: rect(0, 0, 0, 0),
        blend: BlendMode::PassThrough,
        opacity: 255,
        fill: 255,
        lock: LockFlags::default(),
        color: ColorLabel::None,
        clipping: false,
        visible: true,
        mask: None,
        adjustment: None,
        channels: Vec::new(),
        children,
        is_group: true,
        background: false,
        ..Default::default()
    }
}

/// Non-square 6×4 document: a full layer, a masked layer, and a group with one
/// child (exercises the group recursion), plus a document-level channel. The
/// composite is canonical so full-document equality is a valid identity check.
fn sample_doc() -> Document {
    let mut doc = Document::new(6, 4, ColorMode::Rgb, BitDepth::Eight);
    let mask = LayerMask {
        rect: rect(1, 1, 3, 5),
        default_color: 255,
        disabled: false,
        flags: 0,
        data: Some(plane(10, 8).into()),
        ..Default::default()
    };
    doc.layers = vec![
        pixel_layer("base", rect(0, 0, 4, 6), None),
        pixel_layer("masked", rect(1, 1, 3, 5), Some(mask)),
        group("grp", vec![pixel_layer("inner", rect(0, 0, 2, 3), None)]),
    ];
    doc.channels = vec![Channel {
        id: -1,
        data: plane(0, 24).into(),
    }];
    doc.composite = composite_rgba(&doc);
    doc
}

/// Plain two-layer document for the external `psd-tools` check.
fn plain_doc() -> Document {
    let mut doc = Document::new(8, 6, ColorMode::Rgb, BitDepth::Eight);
    doc.layers = vec![
        pixel_layer("bottom", rect(0, 0, 6, 8), None),
        pixel_layer("top", rect(0, 0, 3, 4), None),
    ];
    doc.composite = composite_rgba(&doc);
    doc
}

// --- structural helpers -----------------------------------------------------

fn round_trip(doc: &Document) -> Document {
    read_psd(&write_psd(doc).expect("write_psd")).expect("read_psd")
}

/// Separate from the assertion so the mismatch path can be tested directly
/// (`dimension_check_bites`) instead of only being exercised when correct.
fn check_dims(back: &Document, expected_w: u32, expected_h: u32) -> Result<(), String> {
    if back.width != expected_w || back.height != expected_h {
        return Err(format!(
            "expected {expected_w}x{expected_h}, read {}x{}",
            back.width, back.height
        ));
    }
    Ok(())
}

/// Depth-first `(name, rect)` list, groups before their children.
fn flattened(layers: &[Layer]) -> Vec<(String, PsdRect)> {
    fn walk(layers: &[Layer], out: &mut Vec<(String, PsdRect)>) {
        for layer in layers {
            out.push((layer.name.clone(), layer.rect));
            walk(&layer.children, out);
        }
    }
    let mut out = Vec::new();
    walk(layers, &mut out);
    out
}

/// Composite consistency plus structural survival through `write_psd`/`read_psd`.
fn assert_structural_oracle(doc: &Document, label: &str) {
    assert_eq!(
        doc.composite,
        composite_rgba(doc),
        "{label}: cached composite diverged from composite_rgba"
    );

    let back = round_trip(doc);
    check_dims(&back, doc.width, doc.height).unwrap_or_else(|e| panic!("{label}: {e}"));
    assert_eq!(
        back.layers.len(),
        doc.layers.len(),
        "{label}: top-level layer count"
    );
    assert_eq!(
        flattened(&back.layers),
        flattened(&doc.layers),
        "{label}: layer names/rects changed across write_psd/read_psd"
    );
}

// --- structural + composite tests, one per operation ------------------------

#[test]
fn resize_document_preserves_structure_and_composite() {
    let mut doc = sample_doc();
    resize_document(&mut doc, 12, 8, Resample::Nearest).unwrap();
    assert_eq!((doc.width, doc.height), (12, 8));
    assert_structural_oracle(&doc, "resize_document");
}

#[test]
fn resize_canvas_document_preserves_structure_and_composite() {
    let mut doc = sample_doc();
    resize_canvas_document(&mut doc, 8, 6, Anchor::Center).unwrap();
    assert_eq!((doc.width, doc.height), (8, 6));
    assert_structural_oracle(&doc, "resize_canvas_document");
}

#[test]
fn rotate_document_preserves_structure_and_composite() {
    let mut doc = sample_doc();
    rotate_document(&mut doc, 1).unwrap();
    assert_eq!((doc.width, doc.height), (4, 6));
    assert_structural_oracle(&doc, "rotate_document");
}

#[test]
fn flip_document_preserves_structure_and_composite() {
    let mut doc = sample_doc();
    flip_document(&mut doc, true);
    assert_structural_oracle(&doc, "flip_document");
}

// --- exactness --------------------------------------------------------------

#[test]
fn four_quarter_turns_are_identity() {
    let before = sample_doc();
    let mut doc = before.clone();
    for _ in 0..4 {
        rotate_document(&mut doc, 1).unwrap();
    }
    assert_eq!(doc, before, "4 × 90° must restore the document");
}

#[test]
fn flip_twice_is_identity() {
    for horizontal in [true, false] {
        let before = sample_doc();
        let mut doc = before.clone();
        flip_document(&mut doc, horizontal);
        flip_document(&mut doc, horizontal);
        assert_eq!(doc, before, "horizontal={horizontal}");
    }
}

/// Grow then shrink back to the original size with the opposite anchor: the
/// document dimensions return to the original (content is not expected to — the
/// content-preserving case is the centered identity below).
#[test]
fn canvas_grow_then_shrink_opposite_anchor_restores_dimensions() {
    let mut doc = sample_doc();
    let (w0, h0) = (doc.width, doc.height);
    resize_canvas_document(&mut doc, w0 + 2, h0 + 2, Anchor::TopLeft).unwrap();
    resize_canvas_document(&mut doc, w0, h0, Anchor::BottomRight).unwrap();
    assert_eq!((doc.width, doc.height), (w0, h0));
    assert_structural_oracle(&doc, "canvas grow/shrink");
}

#[test]
fn canvas_center_grow_then_shrink_restores_document() {
    let before = sample_doc();
    let mut doc = before.clone();
    resize_canvas_document(
        &mut doc,
        before.width + 2,
        before.height + 2,
        Anchor::Center,
    )
    .unwrap();
    resize_canvas_document(&mut doc, before.width, before.height, Anchor::Center).unwrap();
    assert_eq!(doc, before, "centered grow then shrink must be identity");
}

// --- proof the dimension check bites ----------------------------------------

#[test]
fn dimension_check_bites() {
    let mut doc = sample_doc();
    resize_document(&mut doc, 12, 8, Resample::Nearest).unwrap();
    let back = round_trip(&doc);

    assert!(check_dims(&back, 12, 8).is_ok(), "correct size must pass");
    assert!(check_dims(&back, 13, 8).is_err(), "wrong width must fail");
    assert!(check_dims(&back, 12, 9).is_err(), "wrong height must fail");
}

// --- independent psd-tools check --------------------------------------------

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

/// Write `doc` to a scratch file and have `scripts/validate_output.py` open it
/// with `psd-tools`, then assert the header reports our dimensions and layer
/// count. Returns early with a message when `python3`/`psd_tools` is missing.
fn assert_psd_tools_sees(doc: &Document, tag: &str) {
    if !psd_tools_available() {
        eprintln!("skipping psd-tools check for {tag}: python3 + psd_tools not available");
        return;
    }

    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/validate_output.py");
    let dir = scratch_dir(tag);
    let path = dir.join(format!("{tag}.psd"));
    std::fs::write(&path, write_psd(doc).unwrap()).unwrap();

    let out = Command::new("python3")
        .arg(&script)
        .arg(&path)
        .output()
        .expect("run scripts/validate_output.py");
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    let _ = std::fs::remove_dir_all(&dir);

    assert!(
        out.status.success(),
        "validate_output.py failed for {tag}:\n{stdout}\n{stderr}"
    );

    // Header: `<path>: <W>x<H> mode=<MODE> depth=<N> layers=<K>`.
    let header = stdout
        .lines()
        .next()
        .expect("validate_output.py header line");
    eprintln!("psd-tools {tag}: {header}");
    let (_, rest) = header
        .split_once(": ")
        .unwrap_or_else(|| panic!("unexpected header: {header}"));
    let mut tokens = rest.split_whitespace();
    let dims = tokens.next().expect("dims token");
    let (pw, ph) = dims.split_once('x').expect("WxH token");
    assert_eq!(
        pw.parse::<u32>().unwrap(),
        doc.width,
        "psd-tools width ({tag})"
    );
    assert_eq!(
        ph.parse::<u32>().unwrap(),
        doc.height,
        "psd-tools height ({tag})"
    );
    let layers = tokens
        .find_map(|t| t.strip_prefix("layers="))
        .expect("layers token")
        .parse::<usize>()
        .unwrap();
    assert_eq!(layers, doc.layers.len(), "psd-tools layer count ({tag})");
}

#[test]
fn psd_tools_sees_resized_document() {
    let mut doc = plain_doc();
    resize_document(&mut doc, 16, 12, Resample::Nearest).unwrap();
    assert_psd_tools_sees(&doc, "resized");
}

#[test]
fn psd_tools_sees_rotated_document() {
    let mut doc = plain_doc();
    rotate_document(&mut doc, 1).unwrap();
    assert_psd_tools_sees(&doc, "rotated");
}

/// Unique scratch directory per call so tests can run in parallel.
fn scratch_dir(tag: &str) -> PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "pictura-render-doc-oracle-{}-{tag}-{n}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
