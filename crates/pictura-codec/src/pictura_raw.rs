//! Pictura Raw: typed read/write of the on-disk camera-raw smart filter.
//!
//! The on-disk form is the reference's standard camera-raw smart filter
//! (`filterID 2683`, name `"Camera Raw Filter"`) inside `SoLd.filterFX`, so
//! the reference can render it; "Pictura Raw" is only our user-facing name.
//! `PicturaRawSettings` is a thin alias of [`pictura_core::CrsSettings`].
//!
//! ponytail: behavioural parity only. The 11 Basic `Fltr` short keys and their
//! value types are grounded on the synthetic fixture
//! `assets/test_with_smart_object02.psd`; document-level `FXid`/`FEid`/`FMsk`
//! render caches are not authored (the baked proxy carries the pixels).

use pictura_core::{Layer, PicturaRawSettings, SmartFilter, SmartObjectKind};

use crate::common::Reader;
use crate::descriptor::{self, get_object_item, get_object_item_mut, set_object_item, DescValue};
use crate::error::PsdError;

// The two constants below, the `Fltr` class ID `Adobe Camera Raw Filter`, the
// `Nm  ` value, and `filterID 2683` are the reference's on-disk identifiers for the
// camera-raw smart filter. They are kept byte-for-byte for the reference
// compatibility and used nominatively; see NOTICE.md.
/// The PSD smart-filter id of the camera-raw filter.
pub const CAMERA_RAW_FILTER_ID: i32 = 2683;
/// The filter's display name as stored in `filterFXList[].Nm  `.
pub const CAMERA_RAW_FILTER_NAME: &str = "Camera Raw Filter";

/// Decode the 11 PV2012 Basic controls from a `Fltr` options buffer.
///
/// Tolerant: a missing key, a value that is neither `Long` nor `Double`, or a
/// non-finite value leaves the field `None`. An unparseable buffer yields
/// all-`None`.
pub fn decode_pictura_raw_settings(fltr_options: &[u8]) -> PicturaRawSettings {
    let Ok(DescValue::Object { items, .. }) = crate::camera_raw_options(fltr_options) else {
        return PicturaRawSettings::default();
    };
    PicturaRawSettings {
        temperature: number(&items, b"Temp"),
        tint: number(&items, b"Tint"),
        exposure: number(&items, b"Ex12"),
        contrast: number(&items, b"Cr12"),
        highlights: number(&items, b"Hi12"),
        shadows: number(&items, b"Sh12"),
        whites: number(&items, b"Wh12"),
        blacks: number(&items, b"Bk12"),
        clarity: number(&items, b"Cl12"),
        vibrance: number(&items, b"Vibr"),
        saturation: number(&items, b"Strt"),
    }
}

fn number(items: &[(Vec<u8>, DescValue)], key: &[u8]) -> Option<f64> {
    match crate::descriptor::get_object_item(items, key)? {
        DescValue::Double(v) if v.is_finite() => Some(*v),
        DescValue::Long(v) => Some(*v as f64),
        _ => None,
    }
}

/// Encode the 11 Basic controls as a `Fltr` descriptor object.
///
/// The value type matches the fixture: `Ex12` is a `Double`, the other ten are
/// `Long`. ponytail: a non-integral value on an integer-typed control is
/// rounded, so `decode(encode(s)) == s` holds exactly for integral slider
/// values (the reference's slider keys are integers anyway).
pub fn encode_pictura_raw_fltr(settings: &PicturaRawSettings) -> DescValue {
    fn push_long(items: &mut Vec<(Vec<u8>, DescValue)>, key: &[u8], value: Option<f64>) {
        if let Some(v) = value.filter(|v| v.is_finite()) {
            items.push((key.to_vec(), DescValue::Long(v.round() as i32)));
        }
    }
    let mut items: Vec<(Vec<u8>, DescValue)> = Vec::new();
    push_long(&mut items, b"Temp", settings.temperature);
    push_long(&mut items, b"Tint", settings.tint);
    if let Some(v) = settings.exposure.filter(|v| v.is_finite()) {
        items.push((b"Ex12".to_vec(), DescValue::Double(v)));
    }
    push_long(&mut items, b"Cr12", settings.contrast);
    push_long(&mut items, b"Hi12", settings.highlights);
    push_long(&mut items, b"Sh12", settings.shadows);
    push_long(&mut items, b"Wh12", settings.whites);
    push_long(&mut items, b"Bk12", settings.blacks);
    push_long(&mut items, b"Cl12", settings.clarity);
    push_long(&mut items, b"Vibr", settings.vibrance);
    push_long(&mut items, b"Strt", settings.saturation);
    let borrowed = items
        .iter()
        .map(|(k, v)| (k.as_slice(), v.clone()))
        .collect();
    object(b"Adobe Camera Raw Filter", borrowed)
}

/// The `filterFX` descriptor for a smart object's filter list, or `None` when
/// the list is empty. Used by the writer to author an `SoLd` for a converted
/// smart object and by [`attach_smart_filter`] to insert one.
pub(crate) fn author_filter_fx(
    filters: &[SmartFilter],
    mask_enabled: bool,
    mask_linked: bool,
    extend_with_white: bool,
) -> Option<DescValue> {
    let items: Vec<DescValue> = filters.iter().map(filter_item).collect();
    if items.is_empty() {
        return None;
    }
    Some(object(
        b"filterFXStyle",
        vec![
            (b"enab", DescValue::Bool(true)),
            (b"validAtPosition", DescValue::Bool(true)),
            (b"filterMaskEnable", DescValue::Bool(mask_enabled)),
            (b"filterMaskLinked", DescValue::Bool(mask_linked)),
            (
                b"filterMaskExtendWithWhite",
                DescValue::Bool(extend_with_white),
            ),
            (b"filterFXList", DescValue::List(items)),
        ],
    ))
}

fn filter_item(filter: &SmartFilter) -> DescValue {
    let fltr = {
        let mut r = Reader::new(&filter.options);
        descriptor::read_descriptor(&mut r).unwrap_or_else(|_| object(b"null", Vec::new()))
    };
    object(
        b"filterFX",
        vec![
            (b"Nm  ", DescValue::Text(format!("{}\0", filter.name))),
            (
                b"blendOptions",
                object(
                    b"blendOptions",
                    vec![
                        (
                            b"Opct",
                            DescValue::UnitFloat {
                                unit: *b"#Prc",
                                value: 100.0,
                            },
                        ),
                        (
                            b"Md  ",
                            DescValue::Enum {
                                kind: b"BlnM".to_vec(),
                                value: b"Nrml".to_vec(),
                            },
                        ),
                    ],
                ),
            ),
            (b"enab", DescValue::Bool(filter.enabled)),
            (b"hasoptions", DescValue::Bool(true)),
            (
                b"FrgC",
                object(
                    b"HSBC",
                    vec![
                        (b"H   ", DescValue::Double(0.0)),
                        (b"Strt", DescValue::Double(0.0)),
                        (b"Brgh", DescValue::Double(100.0)),
                    ],
                ),
            ),
            (
                b"BckC",
                object(
                    b"RGBC",
                    vec![
                        (b"Rd  ", DescValue::Double(255.0)),
                        (b"Grn ", DescValue::Double(255.0)),
                        (b"Bl  ", DescValue::Double(255.0)),
                    ],
                ),
            ),
            (b"Fltr", fltr),
            (b"filterID", DescValue::Long(filter.filter_id)),
        ],
    )
}

/// Attach `filter` to `layer`'s smart object, replacing an existing filter with
/// the same `filter_id` or appending a new one.
///
/// Three cases, in order:
/// 1. A preserved `SoLd`/`SoLE` block is parsed, its `filterFXList` rewritten,
///    and the layer's typed view kept consistent. Every other descriptor key
///    survives.
/// 2. A smart object with no preserved block records the filter on
///    `SmartObject.smart_filters`; the writer authors the block on save.
/// 3. Anything else is refused without mutating: a layer with no smart object,
///    a non-embedded object, or a legacy `plLd`/`PlLd` placed layer (no `SoLd`
///    to carry a filter and the writer does not author one beside it).
pub fn attach_smart_filter(layer: &mut Layer, filter: SmartFilter) -> Result<(), PsdError> {
    if let Some(block_idx) = config_block_index(layer) {
        return insert_into_config(layer, block_idx, &filter);
    }
    if layer
        .extra_blocks
        .iter()
        .any(|b| matches!(&b.key, b"plLd" | b"PlLd"))
    {
        return Err(PsdError::Unsupported(
            "legacy placed-layer smart object".into(),
        ));
    }
    let Some(so) = layer.smart_object.as_mut() else {
        return Err(malformed("layer has no smart object"));
    };
    if so.kind != SmartObjectKind::Embedded || so.payload.is_none() {
        return Err(malformed("smart object is not an embedded payload"));
    }
    match so
        .smart_filters
        .iter_mut()
        .find(|f| f.filter_id == filter.filter_id)
    {
        Some(existing) => *existing = filter,
        None => so.smart_filters.push(filter),
    }
    Ok(())
}

/// Attach the camera-raw smart filter carrying `settings` to `layer`.
///
/// A thin wrapper over [`attach_smart_filter`] with the camera-raw filter id.
pub fn attach_pictura_raw_filter(
    layer: &mut Layer,
    settings: &PicturaRawSettings,
) -> Result<(), PsdError> {
    attach_smart_filter(layer, new_filter(settings))
}

fn config_block_index(layer: &Layer) -> Option<usize> {
    layer
        .extra_blocks
        .iter()
        .position(|b| &b.key == b"SoLd")
        .or_else(|| layer.extra_blocks.iter().position(|b| &b.key == b"SoLE"))
}

fn new_filter(settings: &PicturaRawSettings) -> SmartFilter {
    SmartFilter {
        filter_id: CAMERA_RAW_FILTER_ID,
        name: CAMERA_RAW_FILTER_NAME.to_string(),
        enabled: true,
        options: descriptor::write_descriptor(&encode_pictura_raw_fltr(settings)),
    }
}

/// The group flags the writer authors for a fresh `filterFX` on `layer`.
fn group_flags(layer: &Layer) -> (bool, bool, bool) {
    layer
        .smart_object
        .as_ref()
        .map(|so| {
            (
                so.filter_mask_enabled,
                so.filter_mask_linked,
                so.filter_mask_extend_with_white,
            )
        })
        .unwrap_or((true, false, true))
}

/// The `filterID` of a `filterFXList` entry, or `None` when absent.
fn entry_filter_id(entry: &DescValue) -> Option<i32> {
    let DescValue::Object { items, .. } = entry else {
        return None;
    };
    match get_object_item(items, b"filterID") {
        Some(DescValue::Long(id)) => Some(*id),
        _ => None,
    }
}

/// Insert or replace the entry in a preserved `SoLd`/`SoLE` descriptor,
/// rewriting the block and the layer's typed view consistently.
fn insert_into_config(
    layer: &mut Layer,
    block_idx: usize,
    filter: &SmartFilter,
) -> Result<(), PsdError> {
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
    let DescValue::Object { items, .. } = &mut desc else {
        return Err(malformed("descriptor is not an object"));
    };
    match get_object_item_mut(items, b"filterFX") {
        Some(DescValue::Object {
            items: filter_fx, ..
        }) => match get_object_item_mut(filter_fx, b"filterFXList") {
            Some(DescValue::List(list)) => {
                match list
                    .iter_mut()
                    .find(|entry| entry_filter_id(entry) == Some(filter.filter_id))
                {
                    Some(entry) => *entry = filter_item(filter),
                    None => list.push(filter_item(filter)),
                }
            }
            _ => return Err(malformed("filterFXList is not a list")),
        },
        Some(_) => return Err(malformed("filterFX is not an object")),
        None => {
            let (enabled, linked, extend) = group_flags(layer);
            set_object_item(
                items,
                b"filterFX",
                author_filter_fx(std::slice::from_ref(filter), enabled, linked, extend)
                    .ok_or_else(|| malformed("no filter to author"))?,
            );
        }
    }

    let mut new_data = data[..8].to_vec();
    new_data.extend_from_slice(&descriptor::write_descriptor(&desc));
    // psd-tools reads layer tagged blocks without consuming a trailing pad, so
    // the declared length must be even; the reference 4-aligns the SoLd descriptor.
    while !new_data.len().is_multiple_of(4) {
        new_data.push(0);
    }
    layer.extra_blocks[block_idx].data = new_data.clone();
    if let Some(so) = layer.smart_object.as_mut() {
        so.config_descriptor = new_data;
        match so
            .smart_filters
            .iter_mut()
            .find(|f| f.filter_id == filter.filter_id)
        {
            Some(existing) => *existing = filter.clone(),
            None => so.smart_filters.push(filter.clone()),
        }
    }
    Ok(())
}

fn object(class_id: &[u8], items: Vec<(&[u8], DescValue)>) -> DescValue {
    DescValue::Object {
        name: String::new(),
        class_id: class_id.to_vec(),
        items: items.into_iter().map(|(k, v)| (k.to_vec(), v)).collect(),
    }
}

fn malformed(what: &str) -> PsdError {
    PsdError::Invalid(format!("Pictura Raw filter descriptor: {what}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pictura_core::{Document, LayerBlock, SmartObject};
    use std::path::PathBuf;

    fn embedded_layer() -> Layer {
        Layer {
            name: "L".into(),
            rect: pictura_core::PsdRect {
                top: 0,
                left: 0,
                bottom: 2,
                right: 2,
            },
            smart_object: Some(SmartObject {
                kind: SmartObjectKind::Embedded,
                payload: Some(b"8BPS\x00\x01not-a-real-psd".to_vec()),
                filename: "src.psd".into(),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn full_settings() -> PicturaRawSettings {
        PicturaRawSettings {
            temperature: Some(20.0),
            tint: Some(-5.0),
            exposure: Some(0.75),
            contrast: Some(-10.0),
            highlights: Some(15.0),
            shadows: Some(-8.0),
            whites: Some(12.0),
            blacks: Some(-3.0),
            clarity: Some(6.0),
            vibrance: Some(-9.0),
            saturation: Some(4.0),
        }
    }

    #[test]
    fn encode_decode_round_trips_the_set_fields() {
        let settings = full_settings();
        let fltr = encode_pictura_raw_fltr(&settings);
        let bytes = descriptor::write_descriptor(&fltr);
        assert_eq!(decode_pictura_raw_settings(&bytes), settings);
    }

    #[test]
    fn decode_is_tolerant_of_missing_and_malformed() {
        assert_eq!(
            decode_pictura_raw_settings(&[]),
            PicturaRawSettings::default()
        );
        assert_eq!(
            decode_pictura_raw_settings(b"not a descriptor"),
            PicturaRawSettings::default()
        );
        let fltr = object(
            b"Adobe Camera Raw Filter",
            vec![
                (b"Temp", DescValue::Text("warm".into())),
                (b"Ex12", DescValue::Double(f64::NAN)),
                (b"Vibr", DescValue::Long(7)),
            ],
        );
        let decoded = decode_pictura_raw_settings(&descriptor::write_descriptor(&fltr));
        assert_eq!(decoded.temperature, None, "a non-numeric key is ignored");
        assert_eq!(decoded.exposure, None, "a non-finite value is ignored");
        assert_eq!(decoded.vibrance, Some(7.0));
    }

    #[test]
    fn attach_authors_then_write_read_round_trips() {
        let settings = full_settings();
        let mut layer = embedded_layer();
        attach_pictura_raw_filter(&mut layer, &settings).expect("attach authors filterFX");
        let filter = &layer.smart_object.as_ref().unwrap().smart_filters[0];
        assert_eq!(filter.filter_id, CAMERA_RAW_FILTER_ID);
        assert_eq!(decode_pictura_raw_settings(&filter.options), settings);

        let mut doc = Document::new(
            2,
            2,
            pictura_core::ColorMode::Rgb,
            pictura_core::BitDepth::Eight,
        );
        doc.layers = vec![layer];
        let bytes = crate::write_psd(&doc).expect("writes");
        let back = crate::read_psd(&bytes).expect("re-reads");
        let filter = &back.layers[0]
            .smart_object
            .as_ref()
            .expect("embedded object resolves")
            .smart_filters[0];
        assert_eq!(filter.filter_id, CAMERA_RAW_FILTER_ID);
        assert_eq!(filter.name, CAMERA_RAW_FILTER_NAME);
        assert_eq!(decode_pictura_raw_settings(&filter.options), settings);
    }

    #[test]
    fn reattach_without_a_preserved_block_updates_the_typed_filter() {
        let mut layer = embedded_layer();
        attach_pictura_raw_filter(&mut layer, &full_settings()).expect("first attach authors");
        let second = PicturaRawSettings {
            temperature: Some(-30.0),
            exposure: Some(-1.0),
            ..Default::default()
        };
        attach_pictura_raw_filter(&mut layer, &second).expect("re-attach updates in memory");
        let filter = &layer.smart_object.as_ref().unwrap().smart_filters[0];
        assert_eq!(filter.filter_id, CAMERA_RAW_FILTER_ID);
        assert_eq!(decode_pictura_raw_settings(&filter.options), second);
    }

    #[test]
    fn attach_inserts_into_a_preserved_sold_and_preserves_other_keys() {
        let so = SmartObject {
            uuid: "12345678-1234-1234-1234-123456789abc".into(),
            kind: SmartObjectKind::Embedded,
            payload: Some(b"8BPS\x00\x01x".to_vec()),
            ..Default::default()
        };
        let layer = embedded_layer();
        let data = crate::smart_writer::author_sold_block(&so, &layer, 2, 2);
        let mut layer = layer;
        layer.smart_object.as_mut().unwrap().uuid = so.uuid.clone();
        layer.extra_blocks.push(LayerBlock {
            key: *b"SoLd",
            data,
        });

        let settings = PicturaRawSettings {
            exposure: Some(1.5),
            ..Default::default()
        };
        attach_pictura_raw_filter(&mut layer, &settings).expect("inserts filterFX");
        assert_eq!(
            decode_pictura_raw_settings(
                &layer.smart_object.as_ref().unwrap().smart_filters[0].options
            ),
            settings
        );

        let bytes = crate::write_psd(&{
            let mut doc = Document::new(
                2,
                2,
                pictura_core::ColorMode::Rgb,
                pictura_core::BitDepth::Eight,
            );
            doc.layers = vec![layer];
            doc
        })
        .expect("writes");
        let back = crate::read_psd(&bytes).expect("re-reads");
        let filter = &back.layers[0].smart_object.as_ref().unwrap().smart_filters[0];
        assert_eq!(filter.filter_id, CAMERA_RAW_FILTER_ID);
        assert_eq!(decode_pictura_raw_settings(&filter.options), settings);
    }

    #[test]
    fn attach_errors_without_a_smart_object() {
        let mut layer = Layer::default();
        assert!(attach_pictura_raw_filter(&mut layer, &full_settings()).is_err());
        assert!(layer.smart_object.is_none());
    }

    #[test]
    fn fixture_all_eleven_basic_keys_decode() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/test_with_smart_object02.psd");
        let Ok(bytes) = std::fs::read(&path) else {
            eprintln!("skipping: {} not found", path.display());
            return;
        };
        let doc = crate::read_psd(&bytes).expect("fixture parses");
        let filter = &doc
            .layers
            .iter()
            .find(|l| l.name == "Layer 1 copy")
            .and_then(|l| l.smart_object.as_ref())
            .expect("smart object resolved")
            .smart_filters[0];
        assert_eq!(filter.filter_id, CAMERA_RAW_FILTER_ID);
        assert_eq!(
            decode_pictura_raw_settings(&filter.options),
            PicturaRawSettings {
                temperature: Some(-30.0),
                tint: Some(12.0),
                exposure: Some(-1.15),
                contrast: Some(12.0),
                highlights: Some(-15.0),
                shadows: Some(10.0),
                whites: Some(-18.0),
                blacks: Some(9.0),
                clarity: Some(-12.0),
                vibrance: Some(13.0),
                saturation: Some(-12.0),
            }
        );
    }
}
