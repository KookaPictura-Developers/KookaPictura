//! Encode an [`EngineValue`] tree back to EngineData bytes.
//!
//! The counterpart to [`crate::engine_data::parse_engine_data`]: the output
//! follows the same grammar (psd-tools 1.19 `engine_data.py`) so
//! `parse(encode(tree)) == tree` for the value kinds the type engine uses —
//! dicts (key order preserved), lists, `Int`, `Double`, `Bool`, and `String`.
//!
//! Lossy edges, acceptable because only re-set blocks are re-encoded while an
//! unmodified open→save keeps its raw bytes:
//!
//! - The parser stores a `String` with no encoding tag, so the encoder cannot
//!   reproduce the original bytes. Every string is written as UTF-16BE with a
//!   BOM — the form Photoshop and psd-tools emit and read; equality is semantic
//!   only.
//! - The parser's `from_utf16_lossy` already replaced unpaired surrogates with
//!   U+FFFD, so those are unrecoverable.
//! - There is no MacRoman reverse table: a high-byte name or string decodes to
//!   a `char` and re-encodes as UTF-16BE, never as the original MacRoman byte.
//! - A dict key must be `[A-Za-z0-9_]+` to survive a re-parse, because the
//!   parser's property-name scanner accepts nothing else.
//! - Whitespace and number spelling may differ (`1.0` round-trips as `1.0`; a
//!   `1.0000` becomes `1.0`); both parse to the same typed tree.
//!
//! Programmatic input has two more edges the parser does not: a non-finite
//! `Double` has no EngineData spelling (it is written as `0.0`), and a dict key
//! outside the scanner's charset is not round-trippable. A tree nested past the
//! parser's depth cap is refused rather than overflowing the stack.

use crate::engine_data::EngineValue;
use crate::error::PsdError;

/// The parser's nesting cap (`engine_data::MAX_DEPTH`); an encoded tree deeper
/// than this could not be parsed back.
const MAX_DEPTH: u32 = 64;

/// Serialize `value` as EngineData, or fail when the tree nests past the
/// parser's depth cap.
pub fn encode_engine_data(value: &EngineValue) -> Result<Vec<u8>, PsdError> {
    let mut out = Vec::new();
    encode_value(value, 0, &mut out)?;
    Ok(out)
}

fn encode_value(value: &EngineValue, depth: u32, out: &mut Vec<u8>) -> Result<(), PsdError> {
    if depth > MAX_DEPTH {
        return Err(PsdError::Invalid(
            "EngineData encode: nesting too deep".to_string(),
        ));
    }
    match value {
        EngineValue::Dict(items) => {
            out.extend_from_slice(b"<<");
            for (name, item) in items {
                out.push(b' ');
                out.push(b'/');
                out.extend_from_slice(name.as_bytes());
                out.push(b' ');
                encode_value(item, depth + 1, out)?;
            }
            out.extend_from_slice(b" >>");
        }
        EngineValue::List(items) => {
            out.push(b'[');
            for item in items {
                out.push(b' ');
                encode_value(item, depth + 1, out)?;
            }
            out.extend_from_slice(b" ]");
        }
        EngineValue::Int(value) => out.extend_from_slice(value.to_string().as_bytes()),
        EngineValue::Double(value) => out.extend_from_slice(format_double(*value).as_bytes()),
        EngineValue::Bool(value) => out.extend_from_slice(if *value { b"true" } else { b"false" }),
        EngineValue::String(text) => encode_string(text, out),
    }
    Ok(())
}

/// A `Double` must carry a decimal point or the parser reads it back as `Int`;
/// `f64`'s `Display` never uses exponent notation. Non-finite values have no
/// EngineData spelling and are written as `0.0`.
fn format_double(value: f64) -> String {
    if !value.is_finite() {
        return "0.0".to_string();
    }
    let text = format!("{value}");
    if text.contains('.') {
        text
    } else {
        format!("{text}.0")
    }
}

fn encode_string(text: &str, out: &mut Vec<u8>) {
    // Always UTF-16BE with a BOM: psd-tools' EngineData reader (and Photoshop)
    // decode `(`-strings as UTF-16, and only recognize the BOM form; a bare
    // MacRoman run is an unknown token there. The parser accepts both, so a
    // parse→encode→parse round-trip is unaffected.
    out.push(b'(');
    out.extend_from_slice(&[0xFE, 0xFF]);
    for unit in text.encode_utf16() {
        for byte in unit.to_be_bytes() {
            push_escaped(byte, out);
        }
    }
    out.push(b')');
}

fn push_escaped(byte: u8, out: &mut Vec<u8>) {
    if matches!(byte, b'(' | b')' | b'\\') {
        out.push(b'\\');
    }
    out.push(byte);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine_data::parse_engine_data;

    #[test]
    fn nested_dict_and_list_round_trip() {
        let source = b"<< /EngineDict << /Editor << /Text (\xFE\xFF\x00h\x00i) >> \
            /StyleRun << /RunArray [ << /StyleSheet << /StyleSheetData << \
            /Font 1 /FontSize 150.0 /Tracking 0.25 /FillColor << /Values [ 1 1 1 1 ] >> \
            >> >> >> ] >> /AntiAlias 1 >> /ResourceDict << \
            /FontSet [ << /Name (Invis) >> << /Name (Myriad) >> ] \
            /StyleSheetSet [ << /StyleSheetData << /Font 1 /Tracking .25 >> >> ] >> >>";
        let tree = parse_engine_data(source).expect("parses");
        let encoded = encode_engine_data(&tree).expect("encodes");
        let back = parse_engine_data(&encoded).expect("re-parses");
        assert_eq!(back, tree);
    }

    #[test]
    fn every_value_kind_round_trips() {
        let cases = [
            EngineValue::Int(-7),
            EngineValue::Int(i64::MAX),
            EngineValue::Double(1.5),
            EngineValue::Double(2.0),
            EngineValue::Double(0.1),
            EngineValue::Double(-0.0),
            EngineValue::Bool(true),
            EngineValue::Bool(false),
            EngineValue::String("hi".into()),
            EngineValue::String("héllo → 漢".into()),
            EngineValue::String("esc (paren) \\ slash".into()),
            EngineValue::String(String::new()),
            EngineValue::List(vec![EngineValue::Int(1), EngineValue::String("x".into())]),
            EngineValue::Dict(vec![("a".into(), EngineValue::Double(0.5))]),
        ];
        for value in cases {
            let encoded = encode_engine_data(&value).expect("encodes");
            let back = parse_engine_data(&encoded).expect("re-parses");
            assert_eq!(back, value, "{value:?} -> {encoded:?}");
        }
    }

    #[test]
    fn double_keeps_its_fractional_kind() {
        let encoded = encode_engine_data(&EngineValue::Double(2.0)).expect("encodes");
        assert_eq!(encoded, b"2.0");
        assert_eq!(
            parse_engine_data(&encoded).unwrap(),
            EngineValue::Double(2.0)
        );
    }

    /// ASCII strings must still carry the UTF-16BE BOM: psd-tools' EngineData
    /// reader recognizes only the BOM form, so a bare MacRoman run is a parse
    /// error for the independent oracle.
    #[test]
    fn ascii_string_is_utf16_with_a_bom() {
        let encoded = encode_engine_data(&EngineValue::String("hi".into())).expect("encodes");
        assert_eq!(encoded, b"(\xfe\xff\x00h\x00i)");
    }

    #[test]
    fn deeply_nested_tree_is_refused_instead_of_overflowing() {
        assert!(encode_engine_data(&tree_of(super::MAX_DEPTH + 1)).is_err());
        assert!(encode_engine_data(&tree_of(super::MAX_DEPTH)).is_ok());
    }

    fn tree_of(depth: u32) -> EngineValue {
        let mut value = EngineValue::Int(0);
        for _ in 0..depth {
            value = EngineValue::List(vec![value]);
        }
        value
    }
}
