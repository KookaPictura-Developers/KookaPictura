//! A read-only metadata view over a document's image resources.
//!
//! `read_metadata` gathers the decoded EXIF (1058/1059), IPTC-IIM (1028), and
//! the raw XMP packet (1060) so the app can show them. Nothing here mutates the
//! document; the raw resource section still round-trips byte-for-byte.
//!
//! ponytail: XMP is exposed as raw text, not parsed into fields, so the model is
//! read-only and no XML parser is needed. Add field extraction when editing is.

use pictura_core::Document;

use crate::exif::{parse_exif, Exif};
use crate::image_resources::{
    decode_image_resources, EXIF_DATA_1, EXIF_DATA_3, IPTC_NAA, XMP_METADATA,
};
use crate::iptc::{parse_iptc, Iptc};

/// A document's decoded metadata. An absent resource yields the empty value.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DocumentMetadata {
    pub exif: Exif,
    pub iptc: Iptc,
    /// The raw XMP packet as UTF-8 (lossy) text, empty when absent.
    pub xmp: String,
}

impl DocumentMetadata {
    pub fn is_empty(&self) -> bool {
        self.exif.is_empty() && self.iptc.is_empty() && self.xmp.is_empty()
    }
}

/// Gather the decoded metadata from a document's preserved image resources.
pub fn read_metadata(document: &Document) -> DocumentMetadata {
    let mut metadata = DocumentMetadata::default();
    for resource in decode_image_resources(document) {
        match resource.id {
            EXIF_DATA_1 | EXIF_DATA_3 if metadata.exif.is_empty() => {
                metadata.exif = parse_exif(&resource.data);
            }
            IPTC_NAA if metadata.iptc.is_empty() => {
                metadata.iptc = parse_iptc(&resource.data);
            }
            XMP_METADATA if metadata.xmp.is_empty() => {
                metadata.xmp = String::from_utf8_lossy(&resource.data).into_owned();
            }
            _ => {}
        }
    }
    metadata
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image_resources::encode_image_resources;
    use pictura_core::{BitDepth, ColorMode};

    fn resource(id: u16, name: &str, data: &[u8]) -> crate::image_resources::ImageResource {
        let mut raw = b"8BIM".to_vec();
        raw.extend_from_slice(&id.to_be_bytes());
        raw.push(name.len() as u8);
        raw.extend_from_slice(name.as_bytes());
        if !(name.len() + 1).is_multiple_of(2) {
            raw.push(0);
        }
        raw.extend_from_slice(&(data.len() as u32).to_be_bytes());
        raw.extend_from_slice(data);
        if !data.len().is_multiple_of(2) {
            raw.push(0);
        }
        crate::image_resources::ImageResource {
            id,
            name: name.to_string(),
            data: data.to_vec(),
            raw,
        }
    }

    fn document_with(resources: &[crate::image_resources::ImageResource]) -> Document {
        let mut doc = Document::new(1, 1, ColorMode::Rgb, BitDepth::Eight);
        doc.image_resources = encode_image_resources(resources);
        doc
    }

    #[test]
    fn gathers_exif_iptc_and_xmp() {
        let mut tiff = b"II".to_vec();
        tiff.extend_from_slice(&42u16.to_le_bytes());
        tiff.extend_from_slice(&8u32.to_le_bytes());
        tiff.extend_from_slice(&0u16.to_le_bytes()); // one IFD, zero entries
        tiff.extend_from_slice(&0u32.to_le_bytes()); // next IFD
        let mut iim = vec![0x1c, 2, 5];
        iim.extend_from_slice(&2u16.to_be_bytes());
        iim.extend_from_slice(b"Hi");

        let doc = document_with(&[
            resource(XMP_METADATA, "xmp", b"<x:xmpmeta/>"),
            resource(EXIF_DATA_1, "exif", &tiff),
            resource(IPTC_NAA, "iptc", &iim),
        ]);
        let metadata = read_metadata(&doc);
        assert_eq!(metadata.xmp, "<x:xmpmeta/>");
        assert_eq!(metadata.iptc.text(2, 5).as_deref(), Some("Hi"));
        // The zero-entry TIFF parses to an empty Exif, which is still "present".
        assert!(metadata.exif.is_empty());
    }

    #[test]
    fn no_metadata_resources_is_empty() {
        let doc = document_with(&[resource(crate::image_resources::ICC_PROFILE, "ICC", b"x")]);
        let metadata = read_metadata(&doc);
        assert!(metadata.is_empty());
    }

    fn tiff_with_make() -> Vec<u8> {
        let make = b"ACME\x00";
        let mut out = b"II".to_vec();
        out.extend_from_slice(&42u16.to_le_bytes());
        out.extend_from_slice(&8u32.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&0x010fu16.to_le_bytes());
        out.extend_from_slice(&2u16.to_le_bytes());
        out.extend_from_slice(&(make.len() as u32).to_le_bytes());
        out.extend_from_slice(&26u32.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(make);
        out
    }

    #[test]
    fn decodes_exif_from_the_third_resource() {
        let doc = document_with(&[resource(EXIF_DATA_3, "exif", &tiff_with_make())]);
        let metadata = read_metadata(&doc);
        assert_eq!(
            metadata.exif.get(0x010f),
            Some(&crate::exif::ExifValue::Ascii("ACME".into()))
        );
    }
}
