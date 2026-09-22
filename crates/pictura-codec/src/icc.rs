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
    decode_image_resources, encode_image_resources, ImageResource, ICC_PROFILE,
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
    let resources = decode_image_resources(&doc);
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
    for layer in &mut doc.layers {
        convert_layer(layer, &src, &dst);
    }
    doc.source_icc = Some(profile_bytes);
    let kept: Vec<ImageResource> = resources
        .into_iter()
        .filter(|r| r.id != ICC_PROFILE)
        .collect();
    doc.image_resources = encode_image_resources(&kept);
    doc
}

/// Convert a 3-channel RGB buffer's planes, or `None` when it is not RGB or the
/// transform cannot run; the buffer is never partially converted.
fn convert_buffer(buf: &PixelBuffer, src: &Profile, dst: &Profile) -> Option<PixelBuffer> {
    let channels = buf.channels as usize;
    if channels != 3 {
        return None;
    }
    let n = buf.pixel_count();
    if n == 0 {
        return None;
    }
    let planes: Vec<Vec<u8>> = (0..channels)
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

#[cfg(test)]
mod tests {
    use super::*;
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
}
