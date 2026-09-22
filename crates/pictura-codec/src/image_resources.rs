//! Image-resource section parser.
//!
//! The section is preserved verbatim by the reader and re-emitted byte-for-byte
//! by the writer; this module derives a typed view (id, name, data) so callers
//! can reach the embedded ICC profile, XMP packet, EXIF, and IPTC without
//! re-parsing the block framing. The raw bytes stay the source of truth.
//!
//! ponytail: an unrecognized signature ends parsing (the accepted set is the
//! psd-tools one); add a signature when a real file needs it.

use pictura_core::Document;

use crate::common::Reader;

/// The ICC profile resource id.
pub const ICC_PROFILE: u16 = 1039;
/// The XMP metadata resource id.
pub const XMP_METADATA: u16 = 1060;
/// The first EXIF resource id.
pub const EXIF_DATA_1: u16 = 1058;
/// The third EXIF resource id.
pub const EXIF_DATA_3: u16 = 1059;
/// The IPTC-NAA resource id.
pub const IPTC_NAA: u16 = 1028;

/// One image-resource block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageResource {
    pub id: u16,
    pub name: String,
    pub data: Vec<u8>,
    /// The exact on-disk bytes of the whole block (signature through padded
    /// data), so the section can be re-emitted or rewritten byte-for-byte.
    pub raw: Vec<u8>,
}

/// Signatures psd-tools accepts for an image-resource block; each uses the same
/// `signature id name size data` framing.
const SIGNATURES: [[u8; 4]; 6] = [*b"8BIM", *b"8B64", *b"MeSa", *b"AgHg", *b"PHUT", *b"DCSR"];

/// Parse the document's preserved image-resource section into typed records in
/// stored order. A truncated block or an unrecognized signature ends parsing and
/// the records decoded so far are returned; never panics.
pub fn decode_image_resources(document: &Document) -> Vec<ImageResource> {
    let mut reader = Reader::new(&document.image_resources);
    let mut out = Vec::new();
    while reader.remaining() >= 4 {
        let start = reader.pos;
        let Ok(signature) = reader.take(4) else {
            break;
        };
        if !SIGNATURES.iter().any(|s| s.as_slice() == signature) {
            break;
        }
        let Ok(id) = reader.u16() else {
            break;
        };
        let Ok(name_len) = reader.u8() else {
            break;
        };
        let name_len = name_len as usize;
        let Ok(raw_name) = reader.take(name_len) else {
            break;
        };
        if !(name_len + 1).is_multiple_of(2) && reader.take(1).is_err() {
            break;
        }
        let name = String::from_utf8_lossy(raw_name)
            .trim_end_matches('\0')
            .to_string();
        let Ok(size) = reader.u32() else {
            break;
        };
        let size = size as usize;
        let Ok(data) = reader.take(size) else {
            break;
        };
        if !size.is_multiple_of(2) && reader.take(1).is_err() {
            break;
        }
        out.push(ImageResource {
            id,
            name,
            data: data.to_vec(),
            raw: reader.data[start..reader.pos].to_vec(),
        });
    }
    out
}

/// Re-serialize typed records into an image-resource section by concatenating
/// each record's raw block bytes. `encode_image_resources(&decode_image_resources(doc))`
/// reproduces `doc.image_resources`.
pub fn encode_image_resources(resources: &[ImageResource]) -> Vec<u8> {
    let mut out = Vec::new();
    for resource in resources {
        out.extend_from_slice(&resource.raw);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use pictura_core::{BitDepth, ColorMode};

    fn resource(signature: &[u8; 4], id: u16, name: &str, data: &[u8]) -> Vec<u8> {
        let mut out = signature.to_vec();
        out.extend_from_slice(&id.to_be_bytes());
        out.push(name.len() as u8);
        out.extend_from_slice(name.as_bytes());
        if !(name.len() + 1).is_multiple_of(2) {
            out.push(0);
        }
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        out.extend_from_slice(data);
        if !data.len().is_multiple_of(2) {
            out.push(0);
        }
        out
    }

    fn document_with(section: Vec<u8>) -> Document {
        let mut doc = Document::new(1, 1, ColorMode::Rgb, BitDepth::Eight);
        doc.image_resources = section;
        doc
    }

    #[test]
    fn empty_section_is_empty() {
        assert_eq!(
            decode_image_resources(&document_with(Vec::new())),
            Vec::new()
        );
    }

    #[test]
    fn decodes_resources_in_order_with_odd_padding() {
        let first = resource(b"8BIM", ICC_PROFILE, "ICC", b"profile-bytes");
        let mut section = first.clone();
        section.extend(resource(b"8BIM", XMP_METADATA, "xmp", b"<x/>"));
        let decoded = decode_image_resources(&document_with(section.clone()));
        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded[0].id, ICC_PROFILE);
        assert_eq!(decoded[0].name, "ICC");
        assert_eq!(decoded[0].data, b"profile-bytes");
        assert_eq!(decoded[0].raw, first, "the raw block is retained");
        assert_eq!(decoded[1].id, XMP_METADATA);
        assert_eq!(decoded[1].data, b"<x/>");
        assert_eq!(
            encode_image_resources(&decoded),
            section,
            "decode then encode reproduces the section"
        );
    }

    #[test]
    fn accepts_alternate_signatures() {
        let section = resource(b"MeSa", IPTC_NAA, "iptc", b"data");
        let decoded = decode_image_resources(&document_with(section));
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].id, IPTC_NAA);
    }

    #[test]
    fn truncation_returns_prior_records_and_never_panics() {
        let mut section = resource(b"8BIM", EXIF_DATA_1, "exif", b"abc");
        section.extend_from_slice(b"8BIM"); // header cut off mid-block
        let decoded = decode_image_resources(&document_with(section));
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].id, EXIF_DATA_1);

        // Every prefix of a valid section must be panic-free.
        let full = resource(b"8BIM", ICC_PROFILE, "ICC", b"profile");
        for cut in 0..full.len() {
            let _ = decode_image_resources(&document_with(full[..cut].to_vec()));
        }
    }

    #[test]
    fn unrecognized_signature_stops() {
        let mut section = resource(b"8BIM", ICC_PROFILE, "ICC", b"a");
        section.extend_from_slice(b"nope");
        let decoded = decode_image_resources(&document_with(section));
        assert_eq!(decoded.len(), 1);
    }
}
