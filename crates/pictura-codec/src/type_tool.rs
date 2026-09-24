//! Decode a layer's preserved `TySh` block into the derived
//! [`pictura_core::TypeTool`] view.
//!
//! The raw block stays in `Layer.extra_blocks` and is the source of truth for
//! re-emission; this module only parses the framing. The text and warp
//! descriptors are kept as their full version-16 bytes so an edited view can
//! re-encode framing-only (EngineData stays opaque inside those bytes). A
//! malformed block leaves the view unset rather than failing the document.

use pictura_core::{Layer, TypeTool};

use crate::common::Reader;
use crate::descriptor::{self, get_object_item, DescValue};

/// Resolve `TySh` for `layers` (and their children). Never fails: an absent or
/// unparseable block leaves `type_tool` unset.
pub(crate) fn resolve_type_tools(layers: &mut [Layer]) {
    for layer in layers.iter_mut() {
        layer.type_tool = layer
            .extra_block(b"TySh")
            .and_then(|block| decode_type_tool(&block.data));
        resolve_type_tools(&mut layer.children);
    }
}

/// Parse the `TySh` payload: version-1 framing, transform, text and warp
/// descriptors, then four bounds. `ponytail:` bounds are `i32` per psd-tools;
/// Adobe's table says "4 * 8" — switch if a real CS6 file disagrees.
pub(crate) fn decode_type_tool(data: &[u8]) -> Option<TypeTool> {
    let mut r = Reader::new(data);
    if r.u16().ok()? != 1 {
        return None;
    }
    let mut transform = [0f64; 6];
    for slot in &mut transform {
        *slot = f64::from_bits(r.u64().ok()?);
    }
    if r.u16().ok()? != 50 {
        return None;
    }
    let text_start = r.pos;
    let text_value = descriptor::read_descriptor(&mut r).ok()?;
    let text_desc = data.get(text_start..r.pos)?.to_vec();
    if r.u16().ok()? != 1 {
        return None;
    }
    let warp_start = r.pos;
    descriptor::read_descriptor(&mut r).ok()?;
    let warp_desc = data.get(warp_start..r.pos)?.to_vec();
    let bounds = [r.i32().ok()?, r.i32().ok()?, r.i32().ok()?, r.i32().ok()?];
    Some(TypeTool {
        transform,
        text: extract_text(&text_value),
        bounds,
        text_desc,
        warp_desc,
    })
}

/// Rebuild the `TySh` payload from a view: framing plus the preserved
/// descriptor bytes. Block padding is the tagged-block writer's job.
pub fn encode_type_tool(tool: &TypeTool) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&1u16.to_be_bytes());
    for value in tool.transform {
        out.extend_from_slice(&value.to_bits().to_be_bytes());
    }
    out.extend_from_slice(&50u16.to_be_bytes());
    out.extend_from_slice(&tool.text_desc);
    out.extend_from_slice(&1u16.to_be_bytes());
    out.extend_from_slice(&tool.warp_desc);
    for value in tool.bounds {
        out.extend_from_slice(&value.to_be_bytes());
    }
    out
}

fn extract_text(value: &DescValue) -> String {
    let DescValue::Object { items, .. } = value else {
        return String::new();
    };
    match get_object_item(items, b"Txt ") {
        Some(DescValue::Text(text)) => text.clone(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::descriptor::write_descriptor;

    const TEXT: &str = "Hi\rThere";

    fn text_desc() -> Vec<u8> {
        write_descriptor(&DescValue::Object {
            name: String::new(),
            class_id: b"TxtX".to_vec(),
            items: vec![(b"Txt ".to_vec(), DescValue::Text(TEXT.into()))],
        })
    }

    fn empty_desc() -> Vec<u8> {
        write_descriptor(&DescValue::Object {
            name: String::new(),
            class_id: b"null".to_vec(),
            items: Vec::new(),
        })
    }

    fn synthetic_tysh() -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&1u16.to_be_bytes());
        for value in [1.0f64, 0.0, 0.0, 1.0, 10.0, 20.0] {
            data.extend_from_slice(&value.to_bits().to_be_bytes());
        }
        data.extend_from_slice(&50u16.to_be_bytes());
        data.extend_from_slice(&text_desc());
        data.extend_from_slice(&1u16.to_be_bytes());
        data.extend_from_slice(&empty_desc());
        for value in [0i32, 0, 100, 50] {
            data.extend_from_slice(&value.to_be_bytes());
        }
        data
    }

    #[test]
    fn type_tool_decodes_transform_text_bounds() {
        let tool = decode_type_tool(&synthetic_tysh()).expect("decodes");
        assert_eq!(tool.transform, [1.0, 0.0, 0.0, 1.0, 10.0, 20.0]);
        assert_eq!(tool.text, TEXT);
        assert_eq!(tool.bounds, [0, 0, 100, 50]);
        assert_eq!(tool.text_desc, text_desc());
        assert_eq!(tool.warp_desc, empty_desc());
    }

    #[test]
    fn type_tool_encode_decode_roundtrip() {
        let original = decode_type_tool(&synthetic_tysh()).expect("decodes");
        let reencoded = encode_type_tool(&original);
        assert_eq!(&reencoded[..2], &1u16.to_be_bytes(), "version 1");
        let text_version = u16::from_be_bytes(reencoded[50..52].try_into().unwrap());
        assert_eq!(text_version, 50, "u16 after the transform is 50");
        let back = decode_type_tool(&reencoded).expect("re-decodes");
        assert_eq!(back.transform, original.transform);
        assert_eq!(back.text, original.text);
        assert_eq!(back.bounds, original.bounds);
        assert_eq!(back.text_desc, original.text_desc);
        assert_eq!(back.warp_desc, original.warp_desc);
    }

    #[test]
    fn type_tool_missing_txt_yields_empty_text() {
        let mut data = Vec::new();
        data.extend_from_slice(&1u16.to_be_bytes());
        for value in [1.0f64, 0.0, 0.0, 1.0, 0.0, 0.0] {
            data.extend_from_slice(&value.to_bits().to_be_bytes());
        }
        data.extend_from_slice(&50u16.to_be_bytes());
        data.extend_from_slice(&empty_desc());
        data.extend_from_slice(&1u16.to_be_bytes());
        data.extend_from_slice(&empty_desc());
        for value in [1i32, 2, 3, 4] {
            data.extend_from_slice(&value.to_be_bytes());
        }
        let tool = decode_type_tool(&data).expect("decodes");
        assert_eq!(tool.text, "");
        assert_eq!(tool.bounds, [1, 2, 3, 4]);
    }

    #[test]
    fn type_tool_malformed_is_none() {
        assert!(decode_type_tool(&[]).is_none(), "empty");
        assert!(
            decode_type_tool(&synthetic_tysh()[..20]).is_none(),
            "truncated"
        );

        let mut bad_version = synthetic_tysh();
        bad_version[..2].copy_from_slice(&2u16.to_be_bytes());
        assert!(decode_type_tool(&bad_version).is_none(), "version != 1");

        let mut bytes = synthetic_tysh();
        let text_version_at = 2 + 6 * 8;
        bytes[text_version_at..text_version_at + 2].copy_from_slice(&49u16.to_be_bytes());
        assert!(decode_type_tool(&bytes).is_none(), "text version != 50");

        let mut layers = [Layer {
            extra_blocks: vec![pictura_core::LayerBlock {
                key: *b"TySh",
                data: bad_version,
            }],
            ..Default::default()
        }];
        resolve_type_tools(&mut layers);
        assert!(layers[0].type_tool.is_none(), "view stays unset");
        assert!(layers[0].is_type(), "kind detection stays presence-only");
    }
}
