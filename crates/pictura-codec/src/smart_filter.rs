//! Edit a smart filter's `Fltr` options in place.
//!
//! The preserved `SoLd`/`SoLE` block stays the source of truth: this module
//! parses it to the descriptor DOM, replaces one key, and re-serializes the
//! whole block so every unmodeled key survives the round trip.

use pictura_core::Layer;

use crate::common::Reader;
use crate::descriptor::{self, get_object_item_mut, set_object_item, DescValue};
use crate::error::PsdError;

/// Replace (or insert) one key in the `Fltr` options of the camera raw filter at
/// `filter_index` on `layer`, rewriting the preserved `SoLd`/`SoLE` block in place.
pub fn set_camera_raw_option(
    layer: &mut Layer,
    filter_index: usize,
    key: &[u8],
    value: DescValue,
) -> Result<(), PsdError> {
    let block_idx = layer
        .extra_blocks
        .iter()
        .position(|b| &b.key == b"SoLd")
        .or_else(|| layer.extra_blocks.iter().position(|b| &b.key == b"SoLE"))
        .ok_or_else(|| malformed("layer has no SoLd/SoLE smart object block"))?;

    let data = &layer.extra_blocks[block_idx].data;
    let mut r = Reader::new(data);
    let signature = r.take(4)?.to_vec();
    if signature != b"soLD" && signature != b"soLE" {
        return Err(PsdError::Unsupported(format!(
            "smart object layer data signature {:?}",
            String::from_utf8_lossy(&signature)
        )));
    }
    let _outer_version = r.u32()?;
    let mut desc = descriptor::read_descriptor(&mut r)?;

    let options = {
        let DescValue::Object {
            items: desc_items, ..
        } = &mut desc
        else {
            return Err(malformed("descriptor is not an object"));
        };
        let DescValue::Object {
            items: filter_fx, ..
        } = get_object_item_mut(desc_items, b"filterFX")
            .ok_or_else(|| malformed("missing filterFX"))?
        else {
            return Err(malformed("filterFX is not an object"));
        };
        let DescValue::List(filters) = get_object_item_mut(filter_fx, b"filterFXList")
            .ok_or_else(|| malformed("missing filterFXList"))?
        else {
            return Err(malformed("filterFXList is not a list"));
        };
        let DescValue::Object { items: item, .. } = filters
            .get_mut(filter_index)
            .ok_or_else(|| malformed("filter index out of range"))?
        else {
            return Err(malformed("filter entry is not an object"));
        };
        let fltr = get_object_item_mut(item, b"Fltr").ok_or_else(|| malformed("missing Fltr"))?;
        let DescValue::Object {
            items: fltr_items, ..
        } = fltr
        else {
            return Err(malformed("Fltr is not an object"));
        };
        set_object_item(fltr_items, key, value);
        descriptor::write_descriptor(fltr)
    };

    let mut new_data = data[..8].to_vec();
    new_data.extend_from_slice(&descriptor::write_descriptor(&desc));
    // psd-tools reads layer tagged blocks without consuming a trailing pad, so the
    // declared length must be even. Photoshop 4-aligns the SoLd block data after
    // the descriptor; match that so a same-size edit keeps the original length.
    // ponytail: zero pad to 4; revisit if a reader rejects the trailing bytes.
    while !new_data.len().is_multiple_of(4) {
        new_data.push(0);
    }
    layer.extra_blocks[block_idx].data = new_data.clone();

    if let Some(so) = layer.smart_object.as_mut() {
        so.config_descriptor = new_data;
        if let Some(filter) = so.smart_filters.get_mut(filter_index) {
            filter.options = options;
        }
    }
    Ok(())
}

fn malformed(what: &str) -> PsdError {
    PsdError::Invalid(format!("camera raw filter descriptor: {what}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::descriptor::get_object_item;
    use pictura_core::LayerBlock;
    use std::path::PathBuf;

    fn object(items: Vec<(&[u8], DescValue)>) -> DescValue {
        DescValue::Object {
            name: String::new(),
            class_id: b"null".to_vec(),
            items: items.into_iter().map(|(k, v)| (k.to_vec(), v)).collect(),
        }
    }

    fn double(value: DescValue) -> f64 {
        match value {
            DescValue::Double(v) => v,
            other => panic!("expected Double, got {other:?}"),
        }
    }

    #[test]
    fn fixture_camera_raw_option_edit_round_trips() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/test_with_smart_object02.psd");
        let Ok(bytes) = std::fs::read(&path) else {
            eprintln!("skipping: {} not found", path.display());
            return;
        };
        let mut doc = crate::read_psd(&bytes).expect("fixture parses");
        let layer = doc
            .layers
            .iter_mut()
            .find(|l| l.name == "Layer 1 copy")
            .expect("Layer 1 copy present");

        let filter = &layer.smart_object.as_ref().unwrap().smart_filters[0];
        assert_eq!(filter.filter_id, 2683);
        let DescValue::Object { items, .. } =
            crate::camera_raw_options(&filter.options).expect("Fltr parses")
        else {
            panic!("Fltr is an object");
        };
        assert!((double(get_object_item(&items, b"Ex12").unwrap().clone()) + 1.15).abs() < 1e-6);

        set_camera_raw_option(layer, 0, b"Ex12", DescValue::Double(2.5)).expect("edit succeeds");

        let written = crate::write_psd(&doc).expect("fixture writes");
        let back = crate::read_psd(&written).expect("written fixture re-reads");
        let so = back
            .layers
            .iter()
            .find(|l| l.name == "Layer 1 copy")
            .and_then(|l| l.smart_object.as_ref())
            .expect("smart object survives a write");
        let filter = &so.smart_filters[0];
        assert_eq!(filter.filter_id, 2683);
        let DescValue::Object { items, .. } =
            crate::camera_raw_options(&filter.options).expect("edited Fltr parses")
        else {
            panic!("Fltr is an object");
        };
        assert_eq!(
            get_object_item(&items, b"Ex12"),
            Some(&DescValue::Double(2.5))
        );
        assert_eq!(
            get_object_item(&items, b"Dhze"),
            Some(&DescValue::Long(-14))
        );
        assert_eq!(
            get_object_item(&items, b"Temp"),
            Some(&DescValue::Long(-30))
        );
        assert_eq!(get_object_item(&items, b"Cr12"), Some(&DescValue::Long(12)));
        assert_eq!(get_object_item(&items, b"Vibr"), Some(&DescValue::Long(13)));
    }

    #[test]
    fn noop_edit_preserves_config_descriptor_bytes() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/test_with_smart_object02.psd");
        let Ok(bytes) = std::fs::read(&path) else {
            eprintln!("skipping: {} not found", path.display());
            return;
        };
        let mut doc = crate::read_psd(&bytes).expect("fixture parses");
        let layer = doc
            .layers
            .iter_mut()
            .find(|l| l.name == "Layer 1 copy")
            .expect("Layer 1 copy present");
        let original = layer
            .smart_object
            .as_ref()
            .unwrap()
            .config_descriptor
            .clone();

        set_camera_raw_option(layer, 0, b"Ex12", DescValue::Double(-1.15)).expect("no-op edit");

        let edited = &layer.smart_object.as_ref().unwrap().config_descriptor;
        assert_eq!(
            edited.len(),
            original.len(),
            "a same-size edit keeps the descriptor length"
        );
        for class_id in [
            &b"warp"[..],
            &b"classFloatRect"[..],
            &b"Pnt "[..],
            &b"filterFXStyle"[..],
        ] {
            assert!(
                edited.windows(class_id.len()).any(|w| w == class_id),
                "classID {:?} survives the edit",
                String::from_utf8_lossy(class_id)
            );
        }
    }

    #[test]
    fn set_camera_raw_option_errors_without_sold() {
        let mut layer = Layer::default();
        assert!(
            set_camera_raw_option(&mut layer, 0, b"Ex12", DescValue::Double(1.0)).is_err(),
            "a layer without a smart object block must error, not panic"
        );
    }

    #[test]
    fn edit_accepts_sole_signature_and_preserves_it() {
        let entry = object(vec![
            (b"filterID", DescValue::Long(2683)),
            (b"Fltr", object(vec![(b"Ex12", DescValue::Double(-1.15))])),
        ]);
        let desc = object(vec![(
            b"filterFX",
            object(vec![(b"filterFXList", DescValue::List(vec![entry]))]),
        )]);
        let mut data = b"soLE".to_vec();
        data.extend_from_slice(&4u32.to_be_bytes());
        data.extend_from_slice(&descriptor::write_descriptor(&desc));

        let mut layer = Layer::default();
        layer.extra_blocks.push(LayerBlock {
            key: *b"SoLE",
            data,
        });
        set_camera_raw_option(&mut layer, 0, b"Ex12", DescValue::Double(2.5)).expect("SoLE edit");
        assert_eq!(&layer.extra_blocks[0].data[..4], b"soLE");

        let mut r = Reader::new(&layer.extra_blocks[0].data);
        let _ = r.take(4).unwrap();
        let _ = r.u32().unwrap();
        let DescValue::Object { items, .. } = descriptor::read_descriptor(&mut r).unwrap() else {
            panic!("descriptor object");
        };
        let DescValue::Object {
            items: filter_fx, ..
        } = get_object_item(&items, b"filterFX").unwrap()
        else {
            panic!("filterFX object");
        };
        let DescValue::List(filters) = get_object_item(filter_fx, b"filterFXList").unwrap() else {
            panic!("filterFXList list");
        };
        let DescValue::Object { items: item, .. } = &filters[0] else {
            panic!("filter entry object");
        };
        let DescValue::Object {
            items: fltr_items, ..
        } = get_object_item(item, b"Fltr").unwrap()
        else {
            panic!("Fltr object");
        };
        assert_eq!(
            get_object_item(fltr_items, b"Ex12"),
            Some(&DescValue::Double(2.5))
        );
    }

    #[test]
    fn unknown_signature_is_unsupported() {
        let mut layer = Layer::default();
        layer.extra_blocks.push(LayerBlock {
            key: *b"SoLd",
            data: b"junk\x00\x00\x00\x00".to_vec(),
        });
        let err =
            set_camera_raw_option(&mut layer, 0, b"Ex12", DescValue::Double(1.0)).unwrap_err();
        assert!(matches!(err, PsdError::Unsupported(_)), "got {err:?}");
    }

    #[test]
    fn descriptor_dom_round_trips_after_edit() {
        let fltr = DescValue::Object {
            name: String::new(),
            class_id: b"null".to_vec(),
            items: vec![
                (b"Keep".to_vec(), DescValue::Long(7)),
                (b"Ex12".to_vec(), DescValue::Double(-1.15)),
                (b"Dhze".to_vec(), DescValue::Long(-14)),
            ],
        };
        let mut parsed =
            crate::camera_raw_options(&descriptor::write_descriptor(&fltr)).expect("parses");
        let DescValue::Object { items, .. } = &mut parsed else {
            panic!("Fltr is an object");
        };
        set_object_item(items, b"Ex12", DescValue::Double(2.5));

        let out = descriptor::write_descriptor(&parsed);
        let DescValue::Object { items, .. } = crate::camera_raw_options(&out).expect("re-parses")
        else {
            panic!("Fltr is an object");
        };
        assert_eq!(
            get_object_item(&items, b"Ex12"),
            Some(&DescValue::Double(2.5))
        );
        assert_eq!(get_object_item(&items, b"Keep"), Some(&DescValue::Long(7)));
        assert_eq!(
            get_object_item(&items, b"Dhze"),
            Some(&DescValue::Long(-14))
        );
    }
}
