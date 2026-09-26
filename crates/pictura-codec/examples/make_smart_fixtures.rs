//! Author the synthetic smart-object PSD fixtures committed under `assets/`.
//!
//! Regenerate with:
//!
//! ```bash
//! cargo run -p pictura-codec --example make_smart_fixtures
//! ```
//!
//! Both fixtures mirror the structure the old reference files carried, but every
//! byte is authored here by `pictura-codec` itself:
//!
//! - `test_with_smart_object01.psd`: layer `Layer 1` with one embedded smart
//!   object and no smart filter.
//! - `test_with_smart_object02.psd`: layer `Layer 1 copy` with the same
//!   embedded smart object plus one Camera Raw smart filter (`filterID 2683`).
//!
//! The embedded payload is a minimal version-2 `8BPS` document (`write_psb`), so
//! the fixtures stay a few KB; the output is deterministic.

use std::path::Path;

use pictura_codec::{encode_pictura_raw_fltr, write_descriptor, write_psb, write_psd};
use pictura_core::{
    BitDepth, Channel, ColorMode, Document, Layer, PicturaRawSettings, PsdRect, SmartFilter,
    SmartObject, SmartObjectKind,
};

const EMBEDDED_FILENAME: &str = "Layer 1.psb";

fn embedded_psb() -> Vec<u8> {
    write_psb(&Document::new(2, 2, ColorMode::Rgb, BitDepth::Eight)).expect("embedded PSB writes")
}

fn smart_layer(name: &str, payload: Vec<u8>, smart_filters: Vec<SmartFilter>) -> Layer {
    Layer {
        name: name.to_string(),
        rect: PsdRect {
            top: 0,
            left: 0,
            bottom: 4,
            right: 4,
        },
        // A placed object normally keeps its rendered proxy; a 4x4 red square.
        channels: vec![
            Channel {
                id: 0,
                data: vec![255; 16],
            },
            Channel {
                id: 1,
                data: vec![0; 16],
            },
            Channel {
                id: 2,
                data: vec![0; 16],
            },
            Channel {
                id: -1,
                data: vec![255; 16],
            },
        ],
        smart_object: Some(SmartObject {
            kind: SmartObjectKind::Embedded,
            payload: Some(payload),
            filename: EMBEDDED_FILENAME.to_string(),
            filetype: *b"8BPB",
            creator: *b"8BIM",
            smart_filters,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// The Camera Raw `Fltr` options the tests assert on: the eleven PV2012 Basic
/// controls plus `Dhze` (a key the engine does not model, so it proves a filter
/// edit preserves unmodeled keys).
fn camera_raw_options() -> Vec<u8> {
    let settings = PicturaRawSettings {
        temperature: Some(-30.0),
        tint: Some(12.0),
        exposure: Some(-1.15),
        contrast: Some(12.0),
        highlights: Some(-15.0),
        shadows: Some(10.0),
        whites: Some(-18.0),
        blacks: Some(9.0),
        clarity: Some(-12.0),
        vibrance: Some(13.0),
        saturation: Some(-12.0),
    };
    let mut fltr = encode_pictura_raw_fltr(&settings);
    if let pictura_codec::DescValue::Object { items, .. } = &mut fltr {
        items.push((b"Dhze".to_vec(), pictura_codec::DescValue::Long(-14)));
    }
    write_descriptor(&fltr)
}

fn camera_raw_filter() -> SmartFilter {
    SmartFilter {
        filter_id: pictura_codec::CAMERA_RAW_FILTER_ID,
        name: pictura_codec::CAMERA_RAW_FILTER_NAME.to_string(),
        enabled: true,
        options: camera_raw_options(),
    }
}

fn document(layer: Layer) -> Document {
    let mut doc = Document::new(4, 4, ColorMode::Rgb, BitDepth::Eight);
    for (i, byte) in doc.composite.data.iter_mut().enumerate() {
        *byte = (i * 7 + 3) as u8;
    }
    doc.layers = vec![layer];
    doc
}

fn main() {
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    let payload = embedded_psb();

    let fixtures = [
        (
            "test_with_smart_object01.psd",
            document(smart_layer("Layer 1", payload.clone(), Vec::new())),
        ),
        (
            "test_with_smart_object02.psd",
            document(smart_layer(
                "Layer 1 copy",
                payload,
                vec![camera_raw_filter()],
            )),
        ),
    ];

    for (name, doc) in fixtures {
        let path = assets.join(name);
        let bytes = write_psd(&doc).expect("fixture writes");
        std::fs::write(&path, &bytes).expect("fixture writes to disk");
        println!("wrote {} ({} bytes)", path.display(), bytes.len());
    }
}
