//! Author a new type layer's `TySh` block from a [`TypeSpec`].
//!
//! The read side ([`crate::type_tool`]) keeps an opened file's descriptors
//! verbatim; a layer the Type tools create has none, so this writes them. The
//! text descriptor carries the string and the layer-level choices (orientation,
//! anti-aliasing, bounds); its `EngineData` is the text engine's own dump. That
//! dump has to be complete enough for Photoshop to build a live text object —
//! the run lengths must add up to the string, the run must name a font in the
//! font set — or it falls back to the pixels. Not written: warping (the warp
//! descriptor says none), kinsoku / mojikumi sets (empty), and the `Rendered`
//! cache Photoshop rebuilds. Ported from photorust's `core/src/psd/text_write.rs`.

use pictura_core::{TypeSpec, TypeTool};

use crate::descriptor::{write_descriptor, DescValue};
use crate::type_tool::decode_type_tool;

/// The `TySh` view for `spec`, decoded back from the bytes it encodes to so the
/// fonts and style match what a reopened file reports. `bounds` is the text box
/// relative to the origin: left, top, right, bottom.
pub fn author_type_tool(spec: &TypeSpec, bounds: [f64; 4]) -> TypeTool {
    let mut tool = TypeTool {
        transform: [
            spec.matrix[0],
            spec.matrix[1],
            spec.matrix[2],
            spec.matrix[3],
            spec.origin.0,
            spec.origin.1,
        ],
        text: spec.text.clone(),
        // ponytail: the trailing four integers are written as the rounded text
        // box; photorust writes zeros. Settle against a CS6-authored corpus.
        bounds: bounds.map(|v| v.round() as i32),
        text_desc: text_descriptor(spec, bounds),
        warp_desc: warp_descriptor(),
        fonts: Vec::new(),
        style: None,
        vertical: spec.vertical,
    };
    if let Some(decoded) = decode_type_tool(&crate::encode_type_tool(&tool)) {
        tool.fonts = decoded.fonts;
        tool.style = decoded.style;
    }
    tool
}

fn key(name: &str) -> Vec<u8> {
    name.as_bytes().to_vec()
}

fn enumerated(kind: &str, value: &str) -> DescValue {
    DescValue::Enum {
        kind: key(kind),
        value: key(value),
    }
}

fn object(class: &str, items: Vec<(Vec<u8>, DescValue)>) -> DescValue {
    DescValue::Object {
        name: String::new(),
        class_id: key(class),
        items,
    }
}

fn bounds_object(class: &str, bounds: [f64; 4]) -> DescValue {
    let items = ["Left", "Top ", "Rght", "Btom"]
        .iter()
        .zip(bounds)
        .map(|(name, value)| {
            (
                key(name),
                DescValue::UnitFloat {
                    unit: *b"#Pnt",
                    value,
                },
            )
        })
        .collect();
    object(class, items)
}

fn text_descriptor(spec: &TypeSpec, bounds: [f64; 4]) -> Vec<u8> {
    let engine = engine_data(spec);
    let mut raw = b"tdta".to_vec();
    raw.extend_from_slice(&(engine.len() as u32).to_be_bytes());
    raw.extend_from_slice(&engine);
    write_descriptor(&object(
        "TxLr",
        vec![
            (
                key("Txt "),
                DescValue::Text(format!("{}\0", engine_text(&spec.text))),
            ),
            (key("textGridding"), enumerated("textGridding", "None")),
            (
                key("Ornt"),
                enumerated("Ornt", if spec.vertical { "Vrtc" } else { "Hrzn" }),
            ),
            (
                key("AntA"),
                enumerated(
                    "Annt",
                    if spec.antialias {
                        "antiAliasSharp"
                    } else {
                        "antiAliasNone"
                    },
                ),
            ),
            // Point text without a warp occupies exactly its own box.
            (key("bounds"), bounds_object("bounds", bounds)),
            (key("boundingBox"), bounds_object("boundingBox", bounds)),
            (key("TextIndex"), DescValue::Long(0)),
            (key("EngineData"), DescValue::Raw(raw)),
        ],
    ))
}

fn warp_descriptor() -> Vec<u8> {
    write_descriptor(&object(
        "warp",
        vec![
            (key("warpStyle"), enumerated("warpStyle", "warpNone")),
            (key("warpValue"), DescValue::Double(0.0)),
            (key("warpPerspective"), DescValue::Double(0.0)),
            (key("warpPerspectiveOther"), DescValue::Double(0.0)),
            (key("warpRotate"), enumerated("Ornt", "Hrzn")),
        ],
    ))
}

/// Line breaks as the engine wants them: carriage returns only. A `\n` left in
/// the string shows in Photoshop as a missing-glyph box.
fn engine_text(text: &str) -> String {
    text.replace("\r\n", "\r").replace('\n', "\r")
}

/// EngineData's `/Text`, which always ends in a paragraph-closing `\r` that the
/// descriptor's `Txt ` does not carry; the run lengths count it.
fn engine_body(text: &str) -> String {
    let mut body = engine_text(text);
    if !body.ends_with('\r') {
        body.push('\r');
    }
    body
}

/// Assembles the dump as raw bytes: string values are binary UTF-16.
struct Dump {
    out: Vec<u8>,
}

impl Dump {
    fn line(&mut self, depth: usize, text: &str) {
        self.out.extend(std::iter::repeat_n(b'\t', depth));
        self.out.extend_from_slice(text.as_bytes());
        self.out.push(b'\n');
    }

    fn open(&mut self, depth: usize, name: &str) {
        self.line(depth, name);
        self.line(depth, "<<");
    }

    fn close(&mut self, depth: usize) {
        self.line(depth, ">>");
    }

    /// `key (…)` with the value as BOM-led UTF-16BE, `(`, `)` and `\` escaped.
    fn string(&mut self, depth: usize, key: &str, text: &str) {
        self.out.extend(std::iter::repeat_n(b'\t', depth));
        self.out.extend_from_slice(key.as_bytes());
        self.out.extend_from_slice(b" (\xFE\xFF");
        for unit in text.encode_utf16() {
            for byte in unit.to_be_bytes() {
                if matches!(byte, b'(' | b')' | b'\\') {
                    self.out.push(b'\\');
                }
                self.out.push(byte);
            }
        }
        self.out.extend_from_slice(b")\n");
    }
}

fn engine_data(spec: &TypeSpec) -> Vec<u8> {
    let body = engine_body(&spec.text);
    let font = postscript_name(&spec.font);
    let mut d = Dump {
        out: b"\n\n".to_vec(),
    };
    d.line(0, "<<");
    d.open(1, "/EngineDict");
    d.open(2, "/Editor");
    d.string(3, "/Text", &body);
    d.close(2);
    paragraph_run(&mut d, spec, &body);
    style_run(&mut d, spec, &body);
    grid_info(&mut d);
    // 0 is None; 1 is Sharp, the setting the tool's anti-aliasing means.
    d.line(2, &format!("/AntiAlias {}", u8::from(spec.antialias)));
    d.line(2, "/UseFractionalGlyphWidths true");
    d.close(1);
    // `ResourceDict` is what this layer uses, `DocumentResources` what the
    // document offers; identical is right for a lone text layer.
    for name in ["/ResourceDict", "/DocumentResources"] {
        d.open(1, name);
        resources(&mut d, spec, &font);
        d.close(1);
    }
    d.close(0);
    // An even length keeps the `TySh` block aligned without a padding byte.
    if d.out.len() % 2 == 1 {
        d.out.push(b'\n');
    }
    d.out
}

fn paragraph_run(d: &mut Dump, spec: &TypeSpec, body: &str) {
    d.open(2, "/ParagraphRun");
    d.open(3, "/DefaultRunData");
    paragraph_sheet(d, 4, spec.justification);
    d.close(3);
    let lengths = paragraph_lengths(body);
    d.line(3, "/RunArray [");
    for _ in &lengths {
        d.line(4, "<<");
        paragraph_sheet(d, 5, spec.justification);
        d.line(4, ">>");
    }
    d.line(3, "]");
    d.line(3, &format!("/RunLengthArray [ {} ]", join(&lengths)));
    d.line(3, "/IsJoinable 1");
    d.close(2);
}

fn paragraph_sheet(d: &mut Dump, depth: usize, justification: u8) {
    d.open(depth, "/ParagraphSheet");
    d.line(depth + 1, "/DefaultStyleSheet 0");
    d.open(depth + 1, "/Properties");
    for line in [
        format!("/Justification {justification}"),
        "/FirstLineIndent 0.0".into(),
        "/StartIndent 0.0".into(),
        "/EndIndent 0.0".into(),
        "/SpaceBefore 0.0".into(),
        "/SpaceAfter 0.0".into(),
        "/AutoHyphenate true".into(),
        "/HyphenatedWordSize 6".into(),
        "/PreHyphen 2".into(),
        "/PostHyphen 3".into(),
        "/ConsecutiveHyphens 8".into(),
        "/Zone 36.0".into(),
        "/WordSpacing [ .8 1.0 1.33 ]".into(),
        "/LetterSpacing [ 0.0 0.0 0.0 ]".into(),
        "/GlyphSpacing [ 1.0 1.0 1.0 ]".into(),
        "/AutoLeading 1.2".into(),
        "/LeadingType 0".into(),
        "/Hanging false".into(),
        "/Burasagari false".into(),
        "/KinsokuOrder 0".into(),
        "/EveryLineComposer false".into(),
    ] {
        d.line(depth + 2, &line);
    }
    d.close(depth + 1);
    d.close(depth);
    d.open(depth, "/Adjustments");
    d.line(depth + 1, "/Axis [ 1.0 0.0 1.0 ]");
    d.line(depth + 1, "/XY [ 0.0 0.0 ]");
    d.close(depth);
}

fn style_run(d: &mut Dump, spec: &TypeSpec, body: &str) {
    d.open(2, "/StyleRun");
    d.open(3, "/DefaultRunData");
    style_sheet(d, 4, spec);
    d.close(3);
    d.line(3, "/RunArray [");
    d.line(4, "<<");
    style_sheet(d, 5, spec);
    d.line(4, ">>");
    d.line(3, "]");
    d.line(
        3,
        &format!("/RunLengthArray [ {} ]", body.encode_utf16().count()),
    );
    d.line(3, "/IsJoinable 2");
    d.close(2);
}

/// One style sheet; the font set holds the spec's single font, so `/Font` is 0.
fn style_sheet(d: &mut Dump, depth: usize, spec: &TypeSpec) {
    d.open(depth, "/StyleSheet");
    d.open(depth + 1, "/StyleSheetData");
    for line in [
        "/Font 0".to_string(),
        format!("/FontSize {}", number(spec.size)),
        "/AutoLeading true".into(),
        format!("/Leading {}", number(spec.size * 1.2)),
        "/HorizontalScale 1.0".into(),
        "/VerticalScale 1.0".into(),
        "/Tracking 0".into(),
        "/BaselineShift 0.0".into(),
        "/AutoKerning true".into(),
        "/Kerning 0".into(),
        "/FontCaps 0".into(),
        "/FontBaseline 0".into(),
        "/Underline false".into(),
        "/Strikethrough false".into(),
        "/Ligatures true".into(),
        "/StyleRunAlignment 2".into(),
        "/NoBreak false".into(),
    ] {
        d.line(depth + 2, &line);
    }
    d.open(depth + 2, "/FillColor");
    // Type 1 is RGB; the values run alpha first, as fractions.
    d.line(depth + 3, "/Type 1");
    let [r, g, b, a] = spec.color.map(|v| number(f64::from(v) / 255.0));
    d.line(depth + 3, &format!("/Values [ {a} {r} {g} {b} ]"));
    d.close(depth + 2);
    d.line(depth + 2, "/FillFlag true");
    d.line(depth + 2, "/StrokeFlag false");
    d.close(depth + 1);
    d.close(depth);
}

fn grid_info(d: &mut Dump) {
    d.open(2, "/GridInfo");
    d.line(3, "/GridIsOn false");
    d.line(3, "/ShowGrid false");
    d.line(3, "/GridSize 18.0");
    d.line(3, "/GridLeading 22.0");
    for name in ["/GridColor", "/GridLeadingFillColor"] {
        d.open(3, name);
        d.line(4, "/Type 1");
        d.line(4, "/Values [ 0.0 0.0 0.0 1.0 ]");
        d.close(3);
    }
    d.line(3, "/AlignLineHeightToGridFlags false");
    d.close(2);
}

fn resources(d: &mut Dump, spec: &TypeSpec, font: &str) {
    // Empty rather than absent: East Asian line breaking is not offered, but
    // the keys are part of the shape Photoshop reads.
    d.line(2, "/KinsokuSet [");
    d.line(2, "]");
    d.line(2, "/MojiKumiSet [");
    d.line(2, "]");
    d.line(2, "/TheNormalStyleSheet 0");
    d.line(2, "/TheNormalParagraphSheet 0");
    d.line(2, "/ParagraphSheetSet [");
    d.line(3, "<<");
    d.string(4, "/Name", "Normal RGB");
    d.line(4, "/DefaultStyleSheet 0");
    paragraph_sheet(d, 4, spec.justification);
    d.line(3, ">>");
    d.line(2, "]");
    d.line(2, "/StyleSheetSet [");
    d.line(3, "<<");
    d.string(4, "/Name", "Normal RGB");
    style_sheet(d, 4, spec);
    d.line(3, ">>");
    d.line(2, "]");
    d.line(2, "/FontSet [");
    d.line(3, "<<");
    d.string(4, "/Name", font);
    d.line(4, "/Script 0");
    d.line(4, "/FontType 1");
    d.line(4, "/Synthetic 0");
    d.line(3, ">>");
    d.line(2, "]");
    d.line(2, "/SuperscriptSize .583");
    d.line(2, "/SuperscriptPosition .333");
    d.line(2, "/SubscriptSize .583");
    d.line(2, "/SubscriptPosition .333");
    d.line(2, "/SmallCapSize .7");
}

/// A family as one PostScript name: whitespace dropped, so `Times New Roman`
/// is `TimesNewRoman`. ponytail: a heuristic — a family whose real PostScript
/// name differs (`TimesNewRomanPSMT`) is substituted on reopening in Photoshop.
fn postscript_name(family: &str) -> String {
    let name: String = family.chars().filter(|c| !c.is_whitespace()).collect();
    if name.is_empty() {
        "Helvetica".to_string()
    } else {
        name
    }
}

/// Paragraph lengths in UTF-16 units, each including its closing `\r`, so they
/// add up to the whole body — the property Photoshop checks.
fn paragraph_lengths(body: &str) -> Vec<usize> {
    let mut lengths = Vec::new();
    let mut current = 0;
    for c in body.chars() {
        current += c.len_utf16();
        if c == '\r' {
            lengths.push(current);
            current = 0;
        }
    }
    if current > 0 || lengths.is_empty() {
        lengths.push(current);
    }
    lengths
}

fn join(values: &[usize]) -> String {
    values
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(" ")
}

/// A real with its decimal point: an integer without one is another type to
/// the engine.
fn number(value: f64) -> String {
    let text = format!("{value:.2}");
    let trimmed = text.trim_end_matches('0');
    if trimmed.ends_with('.') {
        format!("{trimmed}0")
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine_data::parse_engine_data;

    fn spec(text: &str, vertical: bool) -> TypeSpec {
        TypeSpec {
            text: text.into(),
            font: "Liberation Sans".into(),
            size: 36.0,
            color: [255, 0, 0, 255],
            justification: 2,
            vertical,
            antialias: true,
            origin: (40.0, 60.0),
            matrix: TypeSpec::IDENTITY,
        }
    }

    #[test]
    fn authored_type_tool_reads_back_as_written() {
        let tool = author_type_tool(&spec("Hi (there)\rtwo", false), [-50.0, -30.0, 50.0, 40.0]);
        let back = decode_type_tool(&crate::encode_type_tool(&tool)).expect("decodes");
        assert_eq!(back, tool);
        assert_eq!(back.text, "Hi (there)\rtwo");
        assert_eq!(back.transform, [1.0, 0.0, 0.0, 1.0, 40.0, 60.0]);
        assert_eq!(back.bounds, [-50, -30, 50, 40]);
        assert!(!back.vertical);
        assert_eq!(back.fonts, ["LiberationSans"]);
        let style = back.style.expect("style");
        assert_eq!(style.font.as_deref(), Some("LiberationSans"));
        assert_eq!(style.font_size, 36.0);
        assert_eq!(style.rgba(), [255, 0, 0, 255]);
        assert_eq!(style.justification, 2);
    }

    #[test]
    fn vertical_orientation_round_trips() {
        let tool = author_type_tool(&spec("V", true), [0.0, 0.0, 10.0, 10.0]);
        assert!(tool.vertical);
        let back = decode_type_tool(&crate::encode_type_tool(&tool)).expect("decodes");
        assert!(back.vertical);
    }

    #[test]
    fn run_lengths_cover_the_engine_text() {
        let engine = engine_data(&spec("ab\rcd", false));
        assert_eq!(engine.len() % 2, 0);
        let root = parse_engine_data(&engine).expect("parses");
        let text = format!("{root:?}");
        assert!(text.contains("String(\"ab\\rcd\\r\")"), "{text}");
        assert!(text.contains("(\"RunLengthArray\", List([Int(3), Int(3)]))"));
        assert!(text.contains("(\"RunLengthArray\", List([Int(6)]))"));
    }
}
