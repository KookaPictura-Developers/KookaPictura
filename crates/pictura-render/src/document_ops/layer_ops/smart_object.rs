//! Convert a raster pixel layer into an embedded smart object.
//!
//! The operation authors a self-contained source document from the layer's
//! raster, serializes it with `write_psd`, and attaches the bytes as an
//! embedded [`SmartObject`]. The layer keeps its pixel channels (the raster
//! proxy), so rendering is unchanged; a layer with a proxy is drawn from the
//! proxy and the embedded source is only decoded for a channel-less layer.

use pictura_core::{BitDepth, Document, PsdRect, SmartObject, SmartObjectKind};

use super::paths::{resolve_path, resolve_path_mut};

/// Whether `path` resolves to a single convertible raster pixel layer: not a
/// group, no adjustment data, not the Background, no existing smart object, and
/// a `rect` with positive width and height.
///
/// Also requires an 8-bit document: `write_psd` is 8-bit only (roadmap G4), so a
/// deeper document could not serialize its embedded source.
pub fn can_convert_to_smart_object(doc: &Document, path: &str) -> bool {
    doc.depth == BitDepth::Eight
        && resolve_path(doc, path).is_some_and(|layer| {
            !layer.is_group
                && layer.adjustment.is_none()
                && !layer.background
                && layer.smart_object.is_none()
                && layer.rect.width() > 0
                && layer.rect.height() > 0
        })
}

/// Convert the raster pixel layer at `path` into an embedded smart object.
///
/// Returns `false` without mutating the document for any ineligible target or
/// when the embedded source cannot be serialized.
pub fn convert_to_smart_object(doc: &mut Document, path: &str) -> bool {
    if !can_convert_to_smart_object(doc, path) {
        return false;
    }
    let (payload, name) = {
        let layer = resolve_path(doc, path).expect("can_convert_to_smart_object checked");
        let w = layer.rect.width() as u32;
        let h = layer.rect.height() as u32;
        let mut embedded = Document::new(w, h, doc.mode, doc.depth);
        let plane = w as usize * h as usize;
        for c in 0..doc.mode.color_channels() as usize {
            if let Some(channel) = layer.channels.iter().find(|ch| ch.id == c as i16) {
                let n = plane.min(channel.data.len());
                embedded.composite.data[c * plane..c * plane + n]
                    .copy_from_slice(&channel.data[..n]);
            }
        }
        let mut copy = layer.clone();
        copy.rect = PsdRect {
            top: 0,
            left: 0,
            bottom: h as i32,
            right: w as i32,
        };
        embedded.layers = vec![copy];
        let Ok(payload) = pictura_codec::write_psd(&embedded) else {
            return false;
        };
        (payload, layer.name.clone())
    };
    let layer = resolve_path_mut(doc, path).expect("can_convert_to_smart_object checked");
    layer.smart_object = Some(SmartObject {
        kind: SmartObjectKind::Embedded,
        payload: Some(payload),
        filename: format!("{name}.psd"),
        filetype: *b"8BPB",
        creator: *b"8BIM",
        ..Default::default()
    });
    true
}
