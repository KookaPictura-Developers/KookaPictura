//! IPTC-IIM decoder.
//!
//! `parse_iptc` turns a PSD IPTC-NAA image resource (1028) into ordered
//! `(record, dataset, value)` triples so the File Info dialog can show core
//! caption/credit fields. The raw resource still round-trips byte-for-byte.
//!
//! ponytail: IIM streams only (no XMP field merge), and a fixed name table.

/// The maximum records walked, so a hostile stream cannot spin the loop.
const MAX_RECORDS: usize = 4096;

/// Ordered decoded IPTC-IIM records.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Iptc {
    records: Vec<(u8, u8, Vec<u8>)>,
}

impl Iptc {
    /// The value bytes for a `record:dataset` pair, or `None`.
    pub fn get(&self, record: u8, dataset: u8) -> Option<&[u8]> {
        self.records
            .iter()
            .find(|(r, d, _)| *r == record && *d == dataset)
            .map(|(_, _, v)| v.as_slice())
    }

    /// The value for a `record:dataset` pair decoded as UTF-8 (lossy).
    pub fn text(&self, record: u8, dataset: u8) -> Option<String> {
        self.get(record, dataset)
            .map(|v| String::from_utf8_lossy(v).into_owned())
    }

    pub fn records(&self) -> &[(u8, u8, Vec<u8>)] {
        &self.records
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

/// A display name for a common IPTC-IIM `record:dataset` pair, or `None`.
pub fn iptc_field_name(record: u8, dataset: u8) -> Option<&'static str> {
    Some(match (record, dataset) {
        (2, 5) => "Object Name",
        (2, 7) => "Edit Status",
        (2, 10) => "Urgency",
        (2, 15) => "Category",
        (2, 20) => "Supplemental Category",
        (2, 25) => "Keywords",
        (2, 40) => "Special Instructions",
        (2, 55) => "Date Created",
        (2, 60) => "Time Created",
        (2, 80) => "By-line",
        (2, 85) => "By-line Title",
        (2, 90) => "City",
        (2, 95) => "State/Province",
        (2, 101) => "Country",
        (2, 105) => "Headline",
        (2, 110) => "Credit",
        (2, 115) => "Source",
        (2, 116) => "Copyright Notice",
        (2, 118) => "Contact",
        (2, 120) => "Caption/Abstract",
        (2, 122) => "Writer/Editor",
        _ => return None,
    })
}

/// Decode a PSD IPTC-NAA image resource into ordered records. Parsing stops at
/// the first malformed or truncated record and returns what it read; never
/// panics.
pub fn parse_iptc(data: &[u8]) -> Iptc {
    let mut records = Vec::new();
    let mut pos = 0;
    while records.len() < MAX_RECORDS {
        let Some(marker) = data.get(pos) else { break };
        if *marker != 0x1c {
            break;
        }
        let Some(header) = data.get(pos..pos + 5) else {
            break;
        };
        let len = u16::from_be_bytes([header[3], header[4]]) as usize;
        let Some(value) = data.get(pos + 5..pos + 5 + len) else {
            break;
        };
        records.push((header[1], header[2], value.to_vec()));
        pos += 5 + len;
    }
    Iptc { records }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(r: u8, d: u8, value: &[u8]) -> Vec<u8> {
        let mut out = vec![0x1c, r, d];
        out.extend_from_slice(&(value.len() as u16).to_be_bytes());
        out.extend_from_slice(value);
        out
    }

    fn sample() -> Vec<u8> {
        let mut out = Vec::new();
        out.extend(record(2, 5, b"Fixture Title"));
        out.extend(record(2, 80, b"Ada Lovelace"));
        out.extend(record(2, 116, b"(c) 2026 Kooka Pictura"));
        out
    }

    #[test]
    fn decodes_records_in_order() {
        let iptc = parse_iptc(&sample());
        assert_eq!(iptc.records().len(), 3);
        assert_eq!(iptc.text(2, 5).as_deref(), Some("Fixture Title"));
        assert_eq!(iptc.text(2, 80).as_deref(), Some("Ada Lovelace"));
        assert_eq!(iptc.get(2, 116), Some(b"(c) 2026 Kooka Pictura".as_slice()));
        assert_eq!(
            iptc_field_name(2, 120),
            Some("Caption/Abstract"),
            "known names resolve"
        );
    }

    #[test]
    fn malformed_input_is_truncated_and_panic_free() {
        assert!(parse_iptc(&[]).is_empty());
        assert!(parse_iptc(b"not iptc").is_empty());
        let mut truncated = record(2, 5, b"Title");
        truncated.push(0x1c); // a record header cut off mid-stream
        let iptc = parse_iptc(&truncated);
        assert_eq!(iptc.records().len(), 1);

        let full = sample();
        for cut in 0..full.len() {
            let _ = parse_iptc(&full[..cut]);
        }
    }
}
