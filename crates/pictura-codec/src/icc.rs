//! Embedded-ICC normalization on read.
//!
//! The engine works in sRGB; a file whose pixels are in another ICC space must
//! be converted on load. This pass runs after the mode/depth `normalize`, finds
//! the profile resource, and converts the composite and every layer's color
//! channels with lcms2 (`pictura-color`). It then removes the stale profile from
//! the preserved resources so a save is not mis-tagged.
//!
//! ponytail: "is sRGB" is the profile's `Description` containing "srgb" (lcms2
//! has no profile compare); a differently-named sRGB profile is converted within
//! a rounding LSB. Rendering intent is fixed at relative-colorimetric.

use pictura_color::{convert, Intent, Profile};
use pictura_core::{ColorMode, Document, Layer, PixelBuffer};

use crate::image_resources::{
    decode_image_resources_with_len, encode_image_resources, frame_image_resource, ImageResource,
    ICC_PROFILE,
};

/// The `Description` of an ICC profile, or `None` when it does not parse.
pub fn profile_description(icc: &[u8]) -> Option<String> {
    Profile::from_icc(icc).ok()?.description()
}

/// Convert a non-sRGB RGB document to the sRGB working space and drop its
/// profile. A non-RGB document, or a profile the engine cannot transform, is
/// left untouched so the pixels and the resource stay consistent.
pub(crate) fn apply_icc(doc: Document) -> Document {
    if doc.mode != ColorMode::Rgb || doc.image_resources.is_empty() {
        return doc;
    }
    let (resources, consumed) = decode_image_resources_with_len(&doc);
    let Some(profile) = resources.iter().find(|r| r.id == ICC_PROFILE) else {
        return doc;
    };
    let Ok(src) = Profile::from_icc(&profile.data) else {
        return doc;
    };
    if src.is_srgb() {
        return doc;
    }
    let dst = Profile::srgb();
    // The composite transform decides: if it cannot run (for example a CMYK
    // profile on an RGB document), leave the whole document untouched rather
    // than strip a profile whose pixels were not converted.
    let Some(composite) = convert_buffer(&doc.composite, &src, &dst) else {
        return doc;
    };
    let profile_bytes = profile.data.clone();
    let mut doc = doc;
    doc.composite = composite;
    convert_layers(&mut doc.layers, &src, &dst);
    doc.source_icc = Some(profile_bytes);
    let section = rebuild_resources(&doc, resources, consumed, None);
    doc.image_resources = section;
    doc
}

/// Convert a buffer's color planes (the first 3 for an RGB/RGBA one, leaving any
/// alpha plane untouched), or `None` when there is no convertible color plane or
/// the transform cannot run; the buffer is never partially converted.
fn convert_buffer(buf: &PixelBuffer, src: &Profile, dst: &Profile) -> Option<PixelBuffer> {
    let color = match buf.channels {
        3 | 4 => 3,
        _ => return None,
    };
    let n = buf.pixel_count();
    if n == 0 {
        return None;
    }
    let planes: Vec<Vec<u8>> = (0..color)
        .map(|ch| buf.data[ch * n..(ch + 1) * n].to_vec())
        .collect();
    let converted = convert_planes(&planes, src, dst)?;
    let mut out = buf.clone();
    for (ch, plane) in converted.into_iter().enumerate() {
        out.data[ch * n..(ch + 1) * n].copy_from_slice(&plane);
    }
    Some(out)
}

/// Convert a layer's RGB color channels in place, selected by channel id `0..3`
/// rather than by storage order. A layer missing a color channel is unchanged.
fn convert_layer(layer: &mut Layer, src: &Profile, dst: &Profile) {
    let mut slots = Vec::with_capacity(3);
    let mut planes = Vec::with_capacity(3);
    for id in 0..3i16 {
        let Some((slot, channel)) = layer.channels.iter().enumerate().find(|(_, c)| c.id == id)
        else {
            return;
        };
        slots.push(slot);
        planes.push(channel.data.clone());
    }
    if let Some(converted) = convert_planes(&planes, src, dst) {
        for (slot, plane) in slots.into_iter().zip(converted) {
            layer.channels[slot].data = plane;
        }
    }
}

/// Convert every layer in `layers` and, recursively, every descendant of a
/// group, so a nested pixel layer is not left in the old profile.
fn convert_layers(layers: &mut [Layer], src: &Profile, dst: &Profile) {
    for layer in layers {
        convert_layer(layer, src, dst);
        convert_layers(&mut layer.children, src, dst);
    }
}

/// Rebuild an image-resource section: every parsed block except 1039 (with an
/// optional freshly framed 1039 appended), followed by the unparsed tail the
/// decoder did not consume, byte-for-byte.
fn rebuild_resources(
    doc: &Document,
    resources: Vec<ImageResource>,
    consumed: usize,
    new_icc: Option<&[u8]>,
) -> Vec<u8> {
    let mut kept: Vec<ImageResource> = resources
        .into_iter()
        .filter(|r| r.id != ICC_PROFILE)
        .collect();
    if let Some(bytes) = new_icc {
        kept.push(frame_image_resource(ICC_PROFILE, "", bytes));
    }
    let mut section = encode_image_resources(&kept);
    section.extend_from_slice(&doc.image_resources[consumed..]);
    section
}

/// Interleave 1 or 3 equal-length planes, convert, and split back. The planes
/// are treated as a `N x 1` image; the lcms2 transform is per-pixel.
fn convert_planes(planes: &[Vec<u8>], src: &Profile, dst: &Profile) -> Option<Vec<Vec<u8>>> {
    let channels = planes.len();
    if channels != 1 && channels != 3 {
        return None;
    }
    let n = planes[0].len();
    if n == 0 || planes.iter().any(|p| p.len() != n) {
        return None;
    }
    let mut packed = vec![0u8; n * channels];
    for (ch, plane) in planes.iter().enumerate() {
        for (i, value) in plane.iter().enumerate() {
            packed[i * channels + ch] = *value;
        }
    }
    let out = convert(
        src,
        dst,
        &packed,
        n as u32,
        1,
        channels as u8,
        8,
        Intent::RelativeColorimetric,
        false,
    )
    .ok()?;
    let mut converted = vec![vec![0u8; n]; channels];
    for (ch, plane) in converted.iter_mut().enumerate() {
        for (i, slot) in plane.iter_mut().enumerate() {
            *slot = out[i * channels + ch];
        }
    }
    Some(converted)
}

/// Assign Profile: retag the document with `target` without touching pixels.
///
/// Rewrites `image_resources` so it carries a framed resource 1039 with
/// `target`'s ICC bytes, or omits 1039 for the sRGB working space, preserving
/// every other resource block (and any unparsed tail) byte-for-byte. Sets
/// `document_icc` to match. Only an RGB document is retagged.
pub fn assign_document_profile(doc: &mut Document, target: &Profile) {
    if doc.mode != ColorMode::Rgb {
        return;
    }
    let icc = if target.is_srgb() {
        None
    } else {
        Some(target.to_icc())
    };
    let (resources, consumed) = decode_image_resources_with_len(doc);
    let section = rebuild_resources(doc, resources, consumed, icc.as_deref());
    doc.image_resources = section;
    doc.document_icc = icc;
}

/// Convert to Profile: transform the composite and every layer's color channels
/// from the document's working profile (`sRGB` when `document_icc` is `None`) to
/// `dst`, then assign `dst`.
///
/// A channel the transform cannot convert (a partial layer) is left untouched,
/// not treated as an error. Returns `false` without mutating for a non-RGB
/// document or when the recorded working profile cannot be parsed.
pub fn convert_document(doc: &mut Document, dst: &Profile) -> bool {
    if doc.mode != ColorMode::Rgb {
        return false;
    }
    let src = match doc.document_icc.as_deref() {
        None => Profile::srgb(),
        Some(icc) => match Profile::from_icc(icc) {
            Ok(profile) => profile,
            Err(_) => return false,
        },
    };
    if let Some(composite) = convert_buffer(&doc.composite, &src, dst) {
        doc.composite = composite;
    }
    convert_layers(&mut doc.layers, &src, dst);
    assign_document_profile(doc, dst);
    true
}

/// Convert a document-space buffer to the sRGB working space for display.
///
/// Returns the input unchanged when the document has no working profile (the
/// sRGB default) or records one that cannot be parsed, so an unprofiled document
/// displays exactly as before.
pub fn buffer_to_srgb(doc: &Document, buf: &PixelBuffer) -> PixelBuffer {
    let Some(icc) = doc.document_icc.as_deref() else {
        return buf.clone();
    };
    let Ok(src) = Profile::from_icc(icc) else {
        return buf.clone();
    };
    if src.is_srgb() {
        return buf.clone();
    }
    // ponytail: re-parses the ICC and rebuilds the transform on every display
    // refresh; cache the transform if a profiled document's refresh ever shows
    // up in a profile.
    convert_buffer(buf, &src, &Profile::srgb()).unwrap_or_else(|| buf.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image_resources::decode_image_resources;
    use pictura_core::{BitDepth, Channel, ColorMode, PsdRect};

    /// One `8BIM` resource block with an empty Pascal name.
    fn block(id: u16, data: &[u8]) -> Vec<u8> {
        let mut out = b"8BIM".to_vec();
        out.extend_from_slice(&id.to_be_bytes());
        out.push(0); // empty Pascal name
        out.push(0); // name pad (1-byte name is odd)
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        out.extend_from_slice(data);
        if !data.len().is_multiple_of(2) {
            out.push(0);
        }
        out
    }

    fn icc_resource(profile: &[u8]) -> Vec<u8> {
        block(ICC_PROFILE, profile)
    }

    fn xmp_resource(data: &[u8]) -> Vec<u8> {
        block(crate::image_resources::XMP_METADATA, data)
    }

    fn rgb_document() -> Document {
        let mut doc = Document::new(2, 1, ColorMode::Rgb, BitDepth::Eight);
        doc.composite.data = vec![200, 10, 100, 20, 50, 30];
        doc
    }

    #[test]
    fn non_srgb_profile_converts_and_drops_the_resource() {
        let profile = Profile::adobe_rgb().to_icc();
        let mut doc = rgb_document();
        doc.image_resources = icc_resource(&profile);
        let original = doc.composite.data.clone();

        let out = apply_icc(doc);
        assert_eq!(out.source_icc.as_deref(), Some(profile.as_slice()));
        assert_ne!(out.composite.data, original, "pixels were converted");
        assert!(
            decode_image_resources(&out)
                .iter()
                .all(|r| r.id != ICC_PROFILE),
            "the stale profile was removed"
        );
        assert!(out.image_resources.is_empty(), "no other resources remain");
    }

    #[test]
    fn srgb_profile_is_left_untouched() {
        let profile = Profile::srgb().to_icc();
        let mut doc = rgb_document();
        let section = icc_resource(&profile);
        doc.image_resources = section.clone();
        let original = doc.composite.data.clone();

        let out = apply_icc(doc);
        assert!(out.source_icc.is_none());
        assert_eq!(out.composite.data, original);
        assert_eq!(out.image_resources, section);
    }

    #[test]
    fn undecodable_profile_is_ignored() {
        let mut doc = rgb_document();
        doc.image_resources = icc_resource(b"not an ICC profile");
        let original = doc.composite.data.clone();
        let out = apply_icc(doc);
        assert!(out.source_icc.is_none());
        assert_eq!(out.composite.data, original);
    }

    #[test]
    fn a_layer_with_incomplete_color_channels_is_skipped() {
        let profile = Profile::adobe_rgb().to_icc();
        let mut doc = rgb_document();
        doc.image_resources = icc_resource(&profile);
        doc.layers.push(Layer {
            name: "partial".into(),
            rect: PsdRect {
                top: 0,
                left: 0,
                bottom: 1,
                right: 2,
            },
            channels: vec![Channel {
                id: 0,
                data: vec![1, 2],
            }],
            ..Default::default()
        });
        let original = doc.layers[0].channels[0].data.clone();
        let out = apply_icc(doc);
        assert_eq!(out.layers[0].channels[0].data, original);
    }

    #[test]
    fn grayscale_document_is_left_untouched() {
        let profile = Profile::adobe_rgb().to_icc();
        let mut doc = Document::new(2, 1, ColorMode::Grayscale, BitDepth::Eight);
        doc.composite.data = vec![10, 200];
        let section = icc_resource(&profile);
        doc.image_resources = section.clone();

        let out = apply_icc(doc);
        assert!(out.source_icc.is_none(), "grayscale is not converted");
        assert_eq!(out.composite.data, vec![10, 200]);
        assert_eq!(out.image_resources, section, "the profile is kept");
    }

    #[test]
    fn other_resources_survive_conversion() {
        let profile = Profile::adobe_rgb().to_icc();
        let mut doc = rgb_document();
        let mut section = icc_resource(&profile);
        section.extend(xmp_resource(b"<x/>"));
        doc.image_resources = section;

        let out = apply_icc(doc);
        let resources = decode_image_resources(&out);
        assert!(resources.iter().all(|r| r.id != ICC_PROFILE));
        let xmp = resources
            .iter()
            .find(|r| r.id == crate::image_resources::XMP_METADATA)
            .expect("XMP survives");
        assert_eq!(xmp.data, b"<x/>");
    }

    #[test]
    fn malformed_section_is_a_noop() {
        let mut doc = rgb_document();
        doc.image_resources = b"not a resource section".to_vec();
        let original = doc.composite.data.clone();
        let out = apply_icc(doc);
        assert!(out.source_icc.is_none());
        assert_eq!(out.composite.data, original);
    }

    #[test]
    fn layer_channels_are_selected_by_id_not_storage_order() {
        let profile = Profile::adobe_rgb().to_icc();
        let mut doc = rgb_document();
        doc.image_resources = icc_resource(&profile);
        doc.layers.push(Layer {
            name: "bgr".into(),
            rect: PsdRect {
                top: 0,
                left: 0,
                bottom: 1,
                right: 1,
            },
            // Stored G, B, R; id-based selection must convert [R, G, B].
            channels: vec![
                Channel {
                    id: 1,
                    data: vec![10],
                },
                Channel {
                    id: 2,
                    data: vec![20],
                },
                Channel {
                    id: 0,
                    data: vec![30],
                },
            ],
            ..Default::default()
        });

        let out = apply_icc(doc);
        let id = |id: i16| {
            out.layers[0]
                .channels
                .iter()
                .find(|c| c.id == id)
                .expect("channel")
                .data[0]
        };
        let src = Profile::from_icc(&profile).expect("profile");
        let expected = convert_planes(&[vec![30], vec![10], vec![20]], &src, &Profile::srgb())
            .expect("converts");
        assert_eq!(
            [id(0), id(1), id(2)],
            [expected[0][0], expected[1][0], expected[2][0]]
        );
    }

    #[test]
    fn assign_leaves_composite_and_layers_byte_identical() {
        let mut doc = Document::from_rgba("px", 2, 1, &[10, 20, 30, 255, 40, 50, 60, 128]);
        let before = doc.clone();
        assign_document_profile(&mut doc, &Profile::adobe_rgb());
        assert_eq!(doc.composite, before.composite);
        assert_eq!(doc.layers[0].channels, before.layers[0].channels);
        assert!(doc.document_icc.is_some());
    }

    #[test]
    fn assign_frames_a_1039_block_for_a_non_srgb_profile() {
        let profile = Profile::adobe_rgb().to_icc();
        let mut doc = Document::new(1, 1, ColorMode::Rgb, BitDepth::Eight);
        assign_document_profile(&mut doc, &Profile::adobe_rgb());
        let resources = decode_image_resources(&doc);
        let icc = resources
            .iter()
            .find(|r| r.id == ICC_PROFILE)
            .expect("a 1039 block is framed");
        assert_eq!(icc.data, profile);
        assert_eq!(doc.document_icc.as_deref(), Some(profile.as_slice()));
    }

    #[test]
    fn assigning_srgb_removes_1039_and_clears_document_icc() {
        let profile = Profile::adobe_rgb().to_icc();
        let mut doc = Document::new(1, 1, ColorMode::Rgb, BitDepth::Eight);
        doc.image_resources = icc_resource(&profile);
        doc.document_icc = Some(profile);
        assign_document_profile(&mut doc, &Profile::srgb());
        assert!(doc.document_icc.is_none());
        assert!(decode_image_resources(&doc)
            .iter()
            .all(|r| r.id != ICC_PROFILE));
    }

    #[test]
    fn assign_preserves_other_resources() {
        let mut doc = Document::new(1, 1, ColorMode::Rgb, BitDepth::Eight);
        doc.image_resources = xmp_resource(b"<x/>");
        assign_document_profile(&mut doc, &Profile::adobe_rgb());
        let resources = decode_image_resources(&doc);
        let xmp = resources
            .iter()
            .find(|r| r.id == crate::image_resources::XMP_METADATA)
            .expect("the XMP block survives");
        assert_eq!(xmp.data, b"<x/>");
        assert!(resources.iter().any(|r| r.id == ICC_PROFILE));
    }

    #[test]
    fn convert_changes_pixels_and_tags_destination() {
        let mut doc = rgb_document();
        let before = doc.composite.data.clone();
        assert!(convert_document(&mut doc, &Profile::adobe_rgb()));
        assert_ne!(doc.composite.data, before, "the composite was converted");
        assert!(doc.document_icc.is_some());
        assert!(decode_image_resources(&doc)
            .iter()
            .any(|r| r.id == ICC_PROFILE));
    }

    #[test]
    fn convert_with_incomplete_layer_channels_is_not_an_error() {
        let mut doc = rgb_document();
        doc.layers.push(Layer {
            name: "partial".into(),
            rect: PsdRect {
                top: 0,
                left: 0,
                bottom: 1,
                right: 2,
            },
            channels: vec![Channel {
                id: 0,
                data: vec![1, 2],
            }],
            ..Default::default()
        });
        let before = doc.layers[0].channels[0].data.clone();
        assert!(convert_document(&mut doc, &Profile::adobe_rgb()));
        assert_eq!(doc.layers[0].channels[0].data, before);
    }

    #[test]
    fn buffer_to_srgb_is_identity_without_a_document_profile() {
        let doc = rgb_document();
        assert_eq!(buffer_to_srgb(&doc, &doc.composite), doc.composite);
    }

    #[test]
    fn buffer_to_srgb_converts_a_profiled_document() {
        let mut doc = rgb_document();
        let before = doc.composite.data.clone();
        doc.document_icc = Some(Profile::adobe_rgb().to_icc());
        let out = buffer_to_srgb(&doc, &doc.composite);
        assert_ne!(out.data, before);
        assert_eq!(out.channels, doc.composite.channels);
    }

    fn rgb_layer(name: &str) -> Layer {
        Layer {
            name: name.into(),
            rect: PsdRect {
                top: 0,
                left: 0,
                bottom: 1,
                right: 2,
            },
            channels: vec![
                Channel {
                    id: 0,
                    data: vec![200, 10],
                },
                Channel {
                    id: 1,
                    data: vec![100, 20],
                },
                Channel {
                    id: 2,
                    data: vec![50, 30],
                },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn convert_reaches_pixel_layers_nested_in_groups() {
        let mut doc = rgb_document();
        let child = rgb_layer("child");
        let before = child.channels[0].data.clone();
        doc.layers.push(Layer {
            name: "group".into(),
            is_group: true,
            children: vec![child],
            ..Default::default()
        });
        assert!(convert_document(&mut doc, &Profile::adobe_rgb()));
        assert_ne!(
            doc.layers[0].children[0].channels[0].data, before,
            "the nested layer was converted"
        );
    }

    #[test]
    fn read_conversion_reaches_pixel_layers_nested_in_groups() {
        let profile = Profile::adobe_rgb().to_icc();
        let mut doc = rgb_document();
        doc.image_resources = icc_resource(&profile);
        let child = rgb_layer("child");
        let before = child.channels[0].data.clone();
        doc.layers.push(Layer {
            name: "group".into(),
            is_group: true,
            children: vec![child],
            ..Default::default()
        });
        let out = apply_icc(doc);
        assert_ne!(out.layers[0].children[0].channels[0].data, before);
    }

    #[test]
    fn assign_preserves_an_unparsed_resource_tail() {
        let profile = Profile::adobe_rgb().to_icc();
        let mut doc = Document::new(1, 1, ColorMode::Rgb, BitDepth::Eight);
        let mut section = icc_resource(&profile);
        let tail = b"nope\x00\x01\x02\x03tail".to_vec();
        section.extend_from_slice(&tail);
        doc.image_resources = section;
        assign_document_profile(&mut doc, &Profile::adobe_rgb());
        assert!(
            doc.image_resources.ends_with(&tail),
            "the unknown-signature tail survives"
        );
        assert!(decode_image_resources(&doc)
            .iter()
            .any(|r| r.id == ICC_PROFILE));
        assert_eq!(doc.document_icc.as_deref(), Some(profile.as_slice()));
    }

    #[test]
    fn read_conversion_preserves_an_unparsed_resource_tail() {
        let profile = Profile::adobe_rgb().to_icc();
        let mut doc = rgb_document();
        let mut section = icc_resource(&profile);
        let tail = b"nope\x00\x01\x02\x03tail".to_vec();
        section.extend_from_slice(&tail);
        doc.image_resources = section;
        let out = apply_icc(doc);
        assert_eq!(
            out.image_resources, tail,
            "the unparsed tail survives read-normalisation"
        );
        assert!(decode_image_resources(&out)
            .iter()
            .all(|r| r.id != ICC_PROFILE));
        assert!(out.document_icc.is_none());
    }

    #[test]
    fn convert_document_is_a_noop_for_a_grayscale_document() {
        let mut doc = Document::new(2, 1, ColorMode::Grayscale, BitDepth::Eight);
        doc.composite.data = vec![10, 200];
        let before = doc.clone();
        assert!(!convert_document(&mut doc, &Profile::adobe_rgb()));
        assert_eq!(doc, before);
    }

    #[test]
    fn assign_document_profile_is_a_noop_for_a_grayscale_document() {
        let mut doc = Document::new(2, 1, ColorMode::Grayscale, BitDepth::Eight);
        assign_document_profile(&mut doc, &Profile::adobe_rgb());
        assert!(doc.document_icc.is_none());
        assert!(doc.image_resources.is_empty());
    }

    #[test]
    fn assign_then_convert_back_to_srgb_leaves_it_untagged() {
        let mut doc = Document::from_rgba("px", 2, 1, &[10, 20, 30, 255, 40, 50, 60, 255]);
        assign_document_profile(&mut doc, &Profile::adobe_rgb());
        assert!(doc.document_icc.is_some());
        assert!(convert_document(&mut doc, &Profile::srgb()));
        assert!(doc.document_icc.is_none());
        assert!(decode_image_resources(&doc)
            .iter()
            .all(|r| r.id != ICC_PROFILE));
    }
}
