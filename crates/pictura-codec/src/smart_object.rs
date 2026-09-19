//! Smart-object resolution: link a layer's config descriptor to its
//! document-level linked-layer record by `uuid`.
//!
//! The preserved bytes (`Layer.extra_blocks`, `Document.layer_section_extra`)
//! remain the source of truth for re-emission; everything here derives a typed
//! view and degrades to [`SmartObjectKind::Unresolved`] rather than failing a
//! file whose smart-object data is malformed but preserved.

use pictura_core::{Layer, LayerBlock, SmartObject, SmartObjectKind};

use crate::common::Reader;
use crate::error::PsdError;

/// One record from a document-level `lnkD`/`lnk2`/`lnk3`/`lnkE` list.
#[derive(Debug, Clone, PartialEq, Eq)]
struct LinkedRecord {
    kind: [u8; 4],
    version: u32,
    uuid: String,
    filename: String,
    filetype: [u8; 4],
    creator: [u8; 4],
    payload: Option<Vec<u8>>,
}

/// Resolve smart objects for `layers` (and their children) from the preserved
/// document-level linked-layer bytes. Never fails: an unparseable list or
/// descriptor leaves the layer unresolved.
pub(crate) fn resolve_smart_objects(layers: &mut [Layer], layer_section_extra: &[u8]) {
    let records = collect_linked_records(layer_section_extra);
    for layer in layers.iter_mut() {
        resolve_layer(layer, &records);
    }
}

fn resolve_layer(layer: &mut Layer, records: &[LinkedRecord]) {
    layer.smart_object = build_smart_object(layer, records);
    for child in layer.children.iter_mut() {
        resolve_layer(child, records);
    }
}

fn build_smart_object(layer: &Layer, records: &[LinkedRecord]) -> Option<SmartObject> {
    let block = find_config_block(&layer.extra_blocks)?;
    let raw = block.data.clone();
    let uuid = config_uuid(block).ok()?;
    let uuid = uuid.trim_end_matches('\0').to_string();

    let mut so = SmartObject {
        uuid: uuid.clone(),
        config_descriptor: raw,
        ..Default::default()
    };
    let Some(record) = records.iter().find(|r| r.uuid == uuid) else {
        return Some(so);
    };

    so.filename = record.filename.trim_end_matches('\0').to_string();
    so.filetype = record.filetype;
    so.creator = record.creator;
    match &record.kind {
        b"liFD" => {
            so.kind = SmartObjectKind::Embedded;
            so.crs_xmp = record.payload.as_deref().and_then(extract_crs);
            so.payload = record.payload.clone();
        }
        b"liFE" => so.kind = SmartObjectKind::External,
        b"liFA" => so.kind = SmartObjectKind::Alias,
        _ => so.kind = SmartObjectKind::Unresolved,
    }
    Some(so)
}

/// The config tagged block, preferring the modern descriptors over the legacy
/// placed-layer one (matching psd-tools' lookup order).
fn find_config_block(blocks: &[LayerBlock]) -> Option<&LayerBlock> {
    [b"SoLd", b"SoLE", b"plLd", b"PlLd"]
        .into_iter()
        .find_map(|key| blocks.iter().find(|b| &b.key == key))
}

fn config_uuid(block: &LayerBlock) -> Result<String, PsdError> {
    match &block.key {
        b"SoLd" | b"SoLE" => read_layer_data_uuid(&block.data),
        b"plLd" | b"PlLd" => read_placed_layer_uuid(&block.data),
        _ => Err(PsdError::Unsupported("smart-object config key".into())),
    }
}

// ---------------------------------------------------------------------------
// Document-level linked-layer list
// ---------------------------------------------------------------------------

/// Parse every linked record in the preserved top-level tagged blocks. On any
/// malformed block, keep what parsed so far instead of failing the file.
fn collect_linked_records(layer_section_extra: &[u8]) -> Vec<LinkedRecord> {
    let mut records = Vec::new();
    let mut r = Reader::new(layer_section_extra);
    while r.remaining() >= 12 {
        let Ok(sig) = r.take(4) else { break };
        if sig != b"8BIM" {
            break;
        }
        let Ok(key) = r.take(4) else { break };
        let Ok(len) = r.u32() else { break };
        let Ok(data) = r.take(len as usize) else {
            break;
        };
        if len % 2 == 1 {
            let _ = r.skip(1);
        }
        if matches!(key, b"lnkD" | b"lnk2" | b"lnk3" | b"lnkE") {
            if let Ok(mut parsed) = parse_linked_layers(data) {
                records.append(&mut parsed);
            }
        }
    }
    records
}

/// Parse a linked-layer list: a sequence of `u64`-length-prefixed records,
/// each padded to a 4-byte boundary.
fn parse_linked_layers(data: &[u8]) -> Result<Vec<LinkedRecord>, PsdError> {
    let mut r = Reader::new(data);
    let mut out = Vec::new();
    while r.remaining() >= 8 {
        let len = r.u64()? as usize;
        let block = r.take(len)?;
        let pad = (4 - len % 4) % 4;
        r.skip(pad)?;
        out.push(parse_linked_layer(block)?);
    }
    Ok(out)
}

fn parse_linked_layer(data: &[u8]) -> Result<LinkedRecord, PsdError> {
    let mut r = Reader::new(data);
    let kind = arr4(r.take(4)?);
    let version = r.u32()?;
    if !(1..=8).contains(&version) {
        return Err(PsdError::Unsupported(format!(
            "linked layer version {version}"
        )));
    }
    let uuid = read_pascal_string(&mut r)?;
    let filename = read_unicode_string(&mut r)?;
    let filetype = arr4(r.take(4)?);
    let creator = arr4(r.take(4)?);
    let datasize = r.u64()? as usize;
    let open_file = r.u8()? != 0;
    if open_file {
        skip_descriptor_block(&mut r)?;
    }

    let mut payload = None;
    match &kind {
        b"liFD" => payload = Some(r.take(datasize)?.to_vec()),
        b"liFE" => {
            skip_descriptor_block(&mut r)?; // linked_file descriptor
            if version > 3 {
                r.skip(16)?; // timestamp: u32 + 4 bytes + f64
            }
            let _filesize = r.u64()?;
            if version > 2 {
                let _ = r.take(datasize)?;
            }
        }
        b"liFA" => r.skip(8)?,
        _ => {}
    }
    if version >= 5 {
        let _child_id = read_unicode_string(&mut r)?;
    }
    if version >= 6 {
        r.skip(8)?; // mod_time
    }
    if version >= 7 {
        r.skip(1)?; // lock_state
    }
    if &kind == b"liFE" && version == 2 {
        let _ = r.take(datasize)?;
    }
    Ok(LinkedRecord {
        kind,
        version,
        uuid,
        filename,
        filetype,
        creator,
        payload,
    })
}

// ---------------------------------------------------------------------------
// Layer config descriptors
// ---------------------------------------------------------------------------

/// `SoLd`/`SoLE`: `soLD` + version + a `DescriptorBlock`; the uuid is the
/// `Idnt` string item.
fn read_layer_data_uuid(data: &[u8]) -> Result<String, PsdError> {
    let mut r = Reader::new(data);
    let _kind = r.take(4)?;
    let _version = r.u32()?;
    let version = r.u32()?;
    if version != 16 {
        return Err(PsdError::Unsupported(format!(
            "descriptor block version {version}"
        )));
    }
    read_unicode_string(&mut r)?; // name
    read_length_and_key(&mut r)?; // classID
    let count = r.u32()?;
    for _ in 0..count {
        let key = read_length_and_key(&mut r)?;
        let ostype = arr4(r.take(4)?);
        if key == b"Idnt" && &ostype == b"TEXT" {
            return read_unicode_string(&mut r);
        }
        skip_descriptor_value(&mut r, ostype)?;
    }
    Err(PsdError::Invalid("SoLd has no Idnt".into()))
}

/// Legacy `plLd`/`PlLd`: `plcL` + version + a Pascal (MacRoman) uuid.
fn read_placed_layer_uuid(data: &[u8]) -> Result<String, PsdError> {
    let mut r = Reader::new(data);
    let _kind = r.take(4)?;
    let _version = r.u32()?;
    read_pascal_string(&mut r)
}

// ---------------------------------------------------------------------------
// Descriptor primitives
// ---------------------------------------------------------------------------

/// Skip a `DescriptorBlock`: a version-16 header then a descriptor body.
fn skip_descriptor_block(r: &mut Reader) -> Result<(), PsdError> {
    let version = r.u32()?;
    if version != 16 {
        return Err(PsdError::Unsupported(format!(
            "descriptor block version {version}"
        )));
    }
    skip_descriptor_body(r)
}

fn skip_descriptor_body(r: &mut Reader) -> Result<(), PsdError> {
    read_unicode_string(r)?; // name
    read_length_and_key(r)?; // classID
    let count = r.u32()?;
    for _ in 0..count {
        read_length_and_key(r)?; // item key
        let ostype = arr4(r.take(4)?);
        skip_descriptor_value(r, ostype)?;
    }
    Ok(())
}

fn skip_descriptor_value(r: &mut Reader, ostype: [u8; 4]) -> Result<(), PsdError> {
    match ostype.as_slice() {
        b"Objc" | b"GlbO" => skip_descriptor_body(r),
        b"obj " => skip_list(r),
        b"VlLs" => skip_list(r),
        b"doub" => r.skip(8),
        b"UntF" => r.skip(12),
        b"UnFl" => {
            r.skip(4)?;
            let n = r.u32()? as usize;
            r.skip(n.checked_mul(8).ok_or_else(overflow)?)
        }
        b"TEXT" => {
            read_unicode_string(r)?;
            Ok(())
        }
        b"enum" => {
            read_length_and_key(r)?;
            read_length_and_key(r)?;
            Ok(())
        }
        b"long" | b"indx" | b"Idnt" => r.skip(4),
        b"comp" => r.skip(8),
        b"bool" => r.skip(1),
        b"type" | b"GlbC" | b"Clss" => {
            read_unicode_string(r)?;
            read_length_and_key(r)?;
            Ok(())
        }
        b"alis" | b"tdta" | b"Pth " => {
            let n = r.u32()? as usize;
            r.skip(n)
        }
        b"ObAr" => {
            r.skip(4)?;
            skip_descriptor_body(r)
        }
        b"prop" => {
            read_unicode_string(r)?;
            read_length_and_key(r)?;
            read_length_and_key(r)?;
            Ok(())
        }
        b"Enmr" => {
            read_unicode_string(r)?;
            read_length_and_key(r)?;
            read_length_and_key(r)?;
            read_length_and_key(r)?;
            Ok(())
        }
        b"rele" => {
            read_unicode_string(r)?;
            read_length_and_key(r)?;
            r.skip(4)
        }
        b"name" => {
            read_unicode_string(r)?;
            read_length_and_key(r)?;
            read_unicode_string(r)?;
            Ok(())
        }
        _ => Err(PsdError::Unsupported(format!(
            "descriptor ostype {:?}",
            String::from_utf8_lossy(&ostype)
        ))),
    }
}

/// A reference/list value: a count, then repeated (ostype, value) pairs.
fn skip_list(r: &mut Reader) -> Result<(), PsdError> {
    let count = r.u32()?;
    for _ in 0..count {
        let ostype = arr4(r.take(4)?);
        skip_descriptor_value(r, ostype)?;
    }
    Ok(())
}

/// Descriptor key: a `u32` length; a zero length means the following 4 bytes
/// are an interned term key.
fn read_length_and_key(r: &mut Reader) -> Result<Vec<u8>, PsdError> {
    let len = r.u32()? as usize;
    let n = if len == 0 { 4 } else { len };
    Ok(r.take(n)?.to_vec())
}

/// UTF-16BE string with a `u32` code-unit count.
fn read_unicode_string(r: &mut Reader) -> Result<String, PsdError> {
    let count = r.u32()? as usize;
    let bytes = count.checked_mul(2).ok_or_else(overflow)?;
    let data = r.take(bytes)?;
    let units: Vec<u16> = data
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_be_bytes([c[0], c[1]]))
        .collect();
    Ok(String::from_utf16_lossy(&units))
}

/// Pascal string (length byte + bytes). MacRoman high bytes are approximated as
/// lossy UTF-8, which is exact for the ASCII uuids and filenames here.
fn read_pascal_string(r: &mut Reader) -> Result<String, PsdError> {
    let len = r.u8()? as usize;
    let data = r.take(len)?;
    Ok(String::from_utf8_lossy(data).into_owned())
}

fn overflow() -> PsdError {
    PsdError::Invalid("smart-object size overflow".into())
}

fn arr4(s: &[u8]) -> [u8; 4] {
    let mut a = [0u8; 4];
    a.copy_from_slice(&s[..4]);
    a
}

/// Return the XMP packet from an embedded payload when it carries `crs:`
/// settings, else `None`.
fn extract_crs(payload: &[u8]) -> Option<Vec<u8>> {
    let start = find_subslice(payload, b"<x:xmpmeta")?;
    let rest = &payload[start..];
    let len = find_subslice(rest, b"</x:xmpmeta>")
        .map(|i| i + b"</x:xmpmeta>".len())
        .or_else(|| find_subslice(rest, b"/>").map(|i| i + 2))?;
    let packet = &rest[..len];
    find_subslice(packet, b"crs:").map(|_| packet.to_vec())
}

/// Naive substring search; payloads are ~1 MiB and this runs once per embedded
/// object, so a two-pointer scan is plenty.
fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn pascal(s: &str) -> Vec<u8> {
        let b = s.as_bytes();
        let mut v = vec![b.len() as u8];
        v.extend_from_slice(b);
        v
    }

    fn unicode(s: &str) -> Vec<u8> {
        let units: Vec<u16> = s.encode_utf16().collect();
        let mut v = (units.len() as u32).to_be_bytes().to_vec();
        for unit in units {
            v.extend_from_slice(&unit.to_be_bytes());
        }
        v
    }

    fn empty_descriptor() -> Vec<u8> {
        let mut d = Vec::new();
        d.extend_from_slice(&16u32.to_be_bytes());
        d.extend_from_slice(&0u32.to_be_bytes()); // name
        d.extend_from_slice(&0u32.to_be_bytes()); // classID key length
        d.extend_from_slice(b"null");
        d.extend_from_slice(&0u32.to_be_bytes()); // item count
        d
    }

    fn plld(uuid: &str) -> Vec<u8> {
        let mut v = b"plcL".to_vec();
        v.extend_from_slice(&3u32.to_be_bytes());
        v.extend_from_slice(&pascal(uuid));
        v
    }

    fn linked_layer(
        kind: &[u8; 4],
        version: u32,
        uuid: &str,
        filename: &str,
        data: &[u8],
    ) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(kind);
        v.extend_from_slice(&version.to_be_bytes());
        v.extend_from_slice(&pascal(uuid));
        v.extend_from_slice(&unicode(filename));
        v.extend_from_slice(b"8BPB");
        v.extend_from_slice(b"8BIM");
        v.extend_from_slice(&(data.len() as u64).to_be_bytes());
        v.push(0); // no open_file descriptor
        match kind {
            b"liFD" => v.extend_from_slice(data),
            b"liFE" => {
                v.extend_from_slice(&empty_descriptor());
                if version > 3 {
                    v.extend_from_slice(&[0u8; 16]);
                }
                v.extend_from_slice(&0u64.to_be_bytes());
            }
            b"liFA" => v.extend_from_slice(&[0u8; 8]),
            _ => {}
        }
        if version >= 5 {
            v.extend_from_slice(&unicode("\0"));
        }
        if version >= 6 {
            v.extend_from_slice(&0f64.to_be_bytes());
        }
        if version >= 7 {
            v.push(0);
        }
        v
    }

    fn tagged(key: &[u8; 4], data: &[u8]) -> Vec<u8> {
        let mut v = b"8BIM".to_vec();
        v.extend_from_slice(key);
        v.extend_from_slice(&(data.len() as u32).to_be_bytes());
        v.extend_from_slice(data);
        if data.len() % 2 == 1 {
            v.push(0);
        }
        v
    }

    fn linked_list(blocks: &[Vec<u8>]) -> Vec<u8> {
        let mut v = Vec::new();
        for b in blocks {
            v.extend_from_slice(&(b.len() as u64).to_be_bytes());
            v.extend_from_slice(b);
            v.extend_from_slice(&vec![0u8; (4 - b.len() % 4) % 4]);
        }
        v
    }

    fn config_layer(uuid: &str) -> Layer {
        Layer {
            name: "L".into(),
            extra_blocks: vec![LayerBlock {
                key: *b"PlLd",
                data: plld(uuid),
            }],
            ..Default::default()
        }
    }

    fn fnv1a(bytes: &[u8]) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for &b in bytes {
            h ^= b as u64;
            h = h.wrapping_mul(0x100_0000_01b3);
        }
        h
    }

    #[test]
    fn external_record_is_reported_without_embedded_payload() {
        let uuid = "11111111-2222-3333-4444-555555555555";
        let record = linked_layer(b"liFE", 3, uuid, "linked.psd", &[]);
        let section = tagged(b"lnk2", &linked_list(&[record]));

        let mut layers = vec![config_layer(uuid)];
        resolve_smart_objects(&mut layers, &section);
        let so = layers[0].smart_object.as_ref().expect("resolved");
        assert_eq!(so.kind, SmartObjectKind::External);
        assert!(so.payload.is_none(), "external payload is not exposed");
        assert_eq!(so.filename, "linked.psd");
    }

    #[test]
    fn config_without_matching_record_is_unresolved() {
        let mut layers = vec![config_layer("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee")];
        resolve_smart_objects(&mut layers, &tagged(b"lnk2", &linked_list(&[])));
        let so = layers[0].smart_object.as_ref().expect("resolved");
        assert_eq!(so.kind, SmartObjectKind::Unresolved);
        assert_eq!(so.uuid, "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee");
        assert!(so.payload.is_none());
    }

    #[test]
    fn embedded_record_exposes_payload() {
        let uuid = "99999999-8888-7777-6666-555555555555";
        let record = linked_layer(b"liFD", 7, uuid, "Layer 1.psb", b"8BPS\x00\x02raw");
        let section = tagged(b"lnk2", &linked_list(&[record]));

        let mut layers = vec![config_layer(uuid)];
        resolve_smart_objects(&mut layers, &section);
        let so = layers[0].smart_object.as_ref().expect("resolved");
        assert_eq!(so.kind, SmartObjectKind::Embedded);
        assert_eq!(so.filename, "Layer 1.psb");
        assert_eq!(so.filetype, *b"8BPB");
        assert_eq!(so.creator, *b"8BIM");
        assert_eq!(so.payload.as_deref(), Some(&b"8BPS\x00\x02raw"[..]));
    }

    #[test]
    fn malformed_record_degrades_without_panic() {
        assert!(parse_linked_layers(&[0, 0, 0, 0, 0, 0, 0, 5, 1, 2, 3]).is_err());
        let section = tagged(b"lnk2", &[0, 0, 0, 0, 0, 0, 0, 5, 1, 2, 3]);
        assert!(collect_linked_records(&section).is_empty());
    }

    #[test]
    fn fixture_smart_object_resolves_embedded_payload() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/test_with_smart_object01.psd");
        let Ok(bytes) = std::fs::read(&path) else {
            eprintln!("skipping: {} not found", path.display());
            return;
        };
        let doc = crate::read_psd(&bytes).expect("fixture parses");
        let layer = doc
            .layers
            .iter()
            .find(|l| l.name == "Layer 1")
            .expect("Layer 1 present");
        let so = layer.smart_object.as_ref().expect("smart object resolved");
        assert_eq!(so.kind, SmartObjectKind::Embedded);
        assert_eq!(so.uuid, "7b11c3bf-7ecf-e74e-855c-f0751f15a1a4");
        assert_eq!(so.filename, "Layer 1.psb");
        assert_eq!(&so.filetype, b"8BPB");
        assert_eq!(&so.creator, b"8BIM");
        assert!(!so.config_descriptor.is_empty());

        let payload = so.payload.as_ref().expect("embedded payload");
        assert_eq!(payload.len(), 1_048_576);
        assert_eq!(&payload[..4], b"8BPS");
        assert_eq!(payload[4], 0);
        assert_eq!(payload[5], 2);
        assert_eq!(fnv1a(payload), 0xac0d_7ee0_9994_eda1);
        assert!(so.crs_xmp.is_none(), "fixture XMP carries no crs: property");

        // The preserved blocks must still re-emit and re-resolve: writing the
        // read document keeps the object intact byte-for-byte.
        let written = crate::write_psd(&doc).expect("fixture writes");
        let back = crate::read_psd(&written).expect("written fixture re-reads");
        let so2 = back
            .layers
            .iter()
            .find(|l| l.name == "Layer 1")
            .and_then(|l| l.smart_object.as_ref())
            .expect("smart object survives a write");
        assert_eq!(so2.uuid, so.uuid);
        assert_eq!(so2.kind, SmartObjectKind::Embedded);
        assert_eq!(fnv1a(so2.payload.as_ref().unwrap()), 0xac0d_7ee0_9994_eda1);
    }
}
