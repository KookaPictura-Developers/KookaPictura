//! End-to-end native-depth editing: a covered tonal adjustment applied to a
//! depth-16 document's retained samples, saved and re-read, keeps the adjusted
//! native samples instead of the 8-bit widen.

use std::path::PathBuf;

use pictura_adjust::{apply_native, Adjustment, LevelsParams};
use pictura_codec::{read_psd, write_psd};
use pictura_core::{BitDepth, Samples};

#[test]
fn native_depth_edit_survives_a_psd_round_trip() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../pictura-codec/tests/fixtures/rgb16.psd");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let mut doc = read_psd(&bytes).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()));

    let width = doc.width as usize;
    let height = doc.height as usize;
    let channels = doc.composite.channels;
    let source = doc
        .source_planes
        .as_mut()
        .expect("rgb16 retains source planes");
    assert_eq!(source.depth, BitDepth::Sixteen);

    let adjustment = Adjustment::Levels(LevelsParams {
        input_black: 0,
        input_white: 255,
        gamma: 1.0,
        output_black: 0,
        output_white: 200,
    });
    apply_native(&adjustment, &mut source.samples, width, height, channels)
        .expect("native apply on retained samples");
    let adjusted = source.samples.clone();

    let plane = width * height * channels as usize;
    doc.composite.data = adjusted.narrow_to_u8()[..plane].to_vec();

    let out = write_psd(&doc).expect("save edited document");
    let back = read_psd(&out).expect("re-read saved document");
    assert_eq!(back.source_planes.unwrap().samples, adjusted);

    let Samples::U16(v) = &adjusted else {
        panic!("rgb16 retains u16 samples")
    };
    assert!(
        v.iter().any(|s| (s >> 8) != (s & 0xff)),
        "expected an adjusted sample that is not the 8-bit widen high*257"
    );
}
