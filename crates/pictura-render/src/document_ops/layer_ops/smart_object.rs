//! Convert a raster pixel layer into an embedded smart object.
//!
//! The operation authors a self-contained source document from the layer's
//! raster, serializes it with `write_psd`, and attaches the bytes as an
//! embedded [`SmartObject`]. The layer keeps its pixel channels (the raster
//! proxy), so rendering is unchanged; a layer with a proxy is drawn from the
//! proxy and the embedded source is only decoded for a channel-less layer.

use pictura_core::{
    BitDepth, BlendMode, Channel, Document, Layer, PsdRect, SmartObject, SmartObjectKind,
};

use super::paths::{format_segments, resolve_path, resolve_path_mut};

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

/// Whether `path` resolves to a replaceable smart-object layer: not a group, no
/// adjustment data, and an `Embedded` object with a non-empty payload.
pub fn can_replace_smart_object_contents(doc: &Document, path: &str) -> bool {
    resolve_path(doc, path).is_some_and(|layer| {
        !layer.is_group
            && layer.adjustment.is_none()
            && layer.smart_object.as_ref().is_some_and(|so| {
                so.kind == SmartObjectKind::Embedded
                    && so.payload.as_ref().is_some_and(|p| !p.is_empty())
            })
    })
}

/// Replace the embedded source of the smart-object layer at `path`.
///
/// Returns `false` without mutating the document when the target is ineligible
/// or `bytes` do not parse as a PSD/PSB document. On success only the typed
/// source and the pixel channels change: the channels are cleared so the new
/// source renders through the embedded-source path scaled into the existing
/// `rect`, and the preserved config block and matching linked record are dropped
/// so a re-save authors a fresh source rather than re-emitting the old one.
pub fn replace_smart_object_contents(
    doc: &mut Document,
    path: &str,
    filename: &str,
    bytes: &[u8],
) -> bool {
    if !can_replace_smart_object_contents(doc, path) || pictura_codec::read_psd(bytes).is_err() {
        return false;
    }
    let old_uuid = resolve_path(doc, path)
        .and_then(|layer| layer.smart_object.as_ref())
        .map(|so| so.uuid.clone())
        .unwrap_or_default();
    {
        let layer = resolve_path_mut(doc, path).expect("can_replace_smart_object_contents checked");
        layer.channels.clear();
        layer
            .extra_blocks
            .retain(|block| !matches!(&block.key, b"SoLd" | b"SoLE" | b"plLd" | b"PlLd"));
        let so = layer.smart_object.as_mut().expect("checked");
        so.payload = Some(bytes.to_vec());
        so.filename = filename.to_string();
        so.filetype = *b"8BPB";
        so.creator = *b"8BIM";
        so.uuid.clear();
    }
    if !old_uuid.is_empty() {
        if let Some(cleaned) =
            pictura_codec::remove_linked_source(&doc.layer_section_extra, &old_uuid)
        {
            doc.layer_section_extra = cleaned;
        }
    }
    true
}

/// Place a PSD/PSB file as a new topmost embedded smart-object layer.
///
/// Decodes `bytes` with [`pictura_codec::read_psd`]; returns `None` without
/// mutating the document when they do not parse. On success appends a
/// channel-less layer named `filename`, sized to the decoded document at
/// `(0, 0)`, carrying an embedded [`SmartObject`] whose payload is `bytes`, and
/// returns the new layer's path.
pub fn place_smart_object(doc: &mut Document, filename: &str, bytes: &[u8]) -> Option<String> {
    let decoded = pictura_codec::read_psd(bytes).ok()?;
    let index = doc.layers.len();
    doc.layers.push(Layer {
        name: filename.to_string(),
        rect: PsdRect {
            top: 0,
            left: 0,
            bottom: decoded.height as i32,
            right: decoded.width as i32,
        },
        blend: BlendMode::Normal,
        opacity: 255,
        fill: 255,
        visible: true,
        channels: Vec::new(),
        smart_object: Some(SmartObject {
            kind: SmartObjectKind::Embedded,
            payload: Some(bytes.to_vec()),
            filename: filename.to_string(),
            filetype: *b"8BPB",
            creator: *b"8BIM",
            ..Default::default()
        }),
        ..Default::default()
    });
    Some(format_segments(&[index]))
}

/// Whether `path` resolves to a rasterizable smart-object layer: not a group,
/// no adjustment data, and a typed smart object to consume.
pub fn can_rasterize_smart_object(doc: &Document, path: &str) -> bool {
    resolve_path(doc, path).is_some_and(|layer| {
        !layer.is_group && layer.adjustment.is_none() && layer.smart_object.is_some()
    })
}

/// Rasterize the smart-object layer at `path` back into a plain pixel layer.
///
/// A layer with a color channel keeps its raster proxy byte-for-byte; a
/// channel-less layer is materialized from the decoded embedded source into
/// channels `0..mode.color_channels()` plus a `-1` alpha. On success the typed
/// object, the preserved config block, and the matching document-level linked
/// record are dropped so a re-save authors no smart object. Returns `false`
/// without mutating for an ineligible target or an undecodable payload.
pub fn rasterize_smart_object(doc: &mut Document, path: &str) -> bool {
    if !can_rasterize_smart_object(doc, path) {
        return false;
    }
    let rendered = {
        let layer = resolve_path(doc, path).expect("can_rasterize_smart_object checked");
        if layer.channels.iter().any(|c| c.id == 0) {
            None
        } else {
            let so = layer.smart_object.as_ref().expect("checked");
            let Some(src) = crate::render_smart_source(so, layer.rect, layer.rect) else {
                return false;
            };
            let plane = src.width as usize * src.height as usize;
            let mut channels: Vec<Channel> = (0..doc.mode.color_channels() as usize)
                .map(|c| Channel {
                    id: c as i16,
                    data: src.data[c * plane..(c + 1) * plane].to_vec(),
                })
                .collect();
            channels.push(Channel {
                id: -1,
                data: src.data[3 * plane..4 * plane].to_vec(),
            });
            Some(channels)
        }
    };
    let uuid = resolve_path(doc, path)
        .and_then(|layer| layer.smart_object.as_ref())
        .map(|so| so.uuid.clone())
        .unwrap_or_default();
    {
        let layer = resolve_path_mut(doc, path).expect("checked");
        if let Some(channels) = rendered {
            layer.channels = channels;
        }
        layer
            .extra_blocks
            .retain(|block| !matches!(&block.key, b"SoLd" | b"SoLE" | b"plLd" | b"PlLd"));
        layer.smart_object = None;
    }
    if !uuid.is_empty() {
        if let Some(cleaned) = pictura_codec::remove_linked_source(&doc.layer_section_extra, &uuid)
        {
            doc.layer_section_extra = cleaned;
        }
    }
    true
}
