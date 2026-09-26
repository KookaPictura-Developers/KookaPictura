//! Pictura Raw document op: bake the filter's Basic controls into a
//! smart-object layer's raster proxy.
//!
//! The layer's embedded source is rendered into its `rect`, filtered, and
//! written back into the layer color channels (the proxy), matching how
//! the reference stores a smart-filtered object. The filter settings are attached
//! so they round-trip on save. ponytail: group/nested smart objects are not
//! recursed into.

use pictura_core::{Channel, Document, Layer, PicturaRawSettings, PixelBuffer, SmartObjectKind};

use super::layer_ops::{resolve_path, resolve_path_mut};

/// Apply the Pictura Raw Basic controls to the smart-object layer at `path`.
///
/// Returns `false` without mutating for a missing path, a non-smart-object, a
/// group, an adjustment layer, a non-embedded object, an empty/undecodable
/// payload, or an unrenderable rect. On success the layer's color channels hold
/// the filtered source at the layer rect, alpha is preserved, and the settings
/// are attached to the camera-raw smart filter.
pub fn apply_pictura_raw(doc: &mut Document, path: &str, settings: &PicturaRawSettings) -> bool {
    let Some(layer) = resolve_path(doc, path) else {
        return false;
    };
    if layer.is_group || layer.adjustment.is_some() {
        return false;
    }
    let Some(so) = layer.smart_object.as_ref() else {
        return false;
    };
    if so.kind != SmartObjectKind::Embedded {
        return false;
    }
    let rect = layer.rect;
    if rect.width() <= 0 || rect.height() <= 0 {
        return false;
    }
    let Some(src) = crate::composite::render_smart_source(so, rect, rect) else {
        return false;
    };
    let Ok(filtered) = pictura_adjust::render_pictura_raw(&src, settings) else {
        return false;
    };
    let color_channels = doc.mode.color_channels() as usize;

    let layer = resolve_path_mut(doc, path).expect("resolved above");
    if pictura_codec::attach_pictura_raw_filter(layer, settings).is_err() {
        return false;
    }
    write_proxy(layer, &filtered, color_channels);
    true
}

/// Replace the layer's color channels from `src`, keeping an existing alpha
/// channel or materializing the rendered one.
fn write_proxy(layer: &mut Layer, src: &PixelBuffer, color_channels: usize) {
    let plane = src.width as usize * src.height as usize;
    let existing_alpha = layer
        .channels
        .iter()
        .find(|c| c.id == -1)
        .map(|c| c.data.clone());
    let mut channels: Vec<Channel> = (0..color_channels)
        .map(|c| Channel {
            id: c as i16,
            data: src.data[c * plane..(c + 1) * plane].to_vec(),
        })
        .collect();
    let alpha = existing_alpha.unwrap_or_else(|| {
        if src.channels >= 4 {
            src.data[3 * plane..4 * plane].to_vec()
        } else {
            vec![255; plane]
        }
    });
    channels.push(Channel {
        id: -1,
        data: alpha,
    });
    layer.channels = channels;
}
