use super::*;

use pictura_codec::EngineValue;

/// The committed `engine_data.bin` is the raw EngineData blob of a reference
/// 2021 text layer. Parsing and deriving the style here proves the Rust parser
/// agrees with the psd-tools 1.19 EngineData reader on the same bytes.
#[test]
fn engine_data_fixture_decodes_to_expected_style() {
    let path = fixture_dir().join("engine_data.bin");
    let bytes = std::fs::read(&path).expect("read engine_data.bin");
    let root = pictura_codec::parse_engine_data(&bytes).expect("parses");

    let fonts = pictura_codec::extract_fonts(&root);
    assert_eq!(fonts, ["AdobeInvisFont", "MyriadPro-Regular"]);
    let style = pictura_codec::extract_style(&root, &fonts);
    assert_eq!(style.font.as_deref(), Some("MyriadPro-Regular"));
    assert_eq!(style.font_size, 150.0);
    assert_eq!(style.fill_color, [1.0, 1.0, 1.0, 1.0]);
    assert_eq!(style.tracking, 0.0);
    assert_eq!(style.justification, 0);
    assert_eq!(
        get(&root, "EngineDict")
            .and_then(|v| get(v, "Editor"))
            .and_then(|v| get(v, "Text")),
        Some(&EngineValue::String("hello world\r".into()))
    );

    if !psd_tools_available() {
        eprintln!("skipping psd-tools cross-check: python3 + psd-tools not available");
        return;
    }
    assert_eq!(ours(&fonts, &style, &root), psd_tools_values(&path));
}

fn get<'a>(value: &'a EngineValue, key: &str) -> Option<&'a EngineValue> {
    match value {
        EngineValue::Dict(items) => items.iter().find(|(k, _)| k == key).map(|(_, v)| v),
        _ => None,
    }
}

/// Same fields the Python script prints, in the same order.
fn ours(fonts: &[String], style: &pictura_core::TextStyle, root: &EngineValue) -> String {
    let text = match get(root, "EngineDict")
        .and_then(|v| get(v, "Editor"))
        .and_then(|v| get(v, "Text"))
    {
        Some(EngineValue::String(s)) => s.as_str(),
        _ => "",
    };
    let text_hex: String = text.as_bytes().iter().map(|b| format!("{b:02x}")).collect();
    let font = style.font.as_deref().unwrap_or("-");
    let fill = style
        .fill_color
        .iter()
        .map(|v| format!("{v:?}"))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "text_hex={text_hex}\nfonts={}\nfont={font}\nsize={:?}\nfill={fill}\ntracking={:?}\njust={}",
        fonts.join(","),
        style.font_size,
        style.tracking,
        style.justification,
    )
}

fn psd_tools_values(path: &std::path::Path) -> String {
    let script = r#"
import sys
from psd_tools.psd.engine_data import EngineData

d = EngineData.frombytes(open(sys.argv[1], "rb").read())
def g(o, *ks):
    for k in ks:
        o = o[k]
    return o
def s(v):
    return v.value if hasattr(v, "value") else v

text = g(d, "EngineDict", "Editor", "Text")
fonts = [s(e["Name"]) for e in g(d, "ResourceDict", "FontSet")]
run = g(d, "EngineDict", "StyleRun", "RunArray", 0, "StyleSheet", "StyleSheetData")
default = g(d, "ResourceDict", "StyleSheetSet", 0, "StyleSheetData")
idx = int(run.get("Font", default.get("Font", 0)))
font = fonts[idx] if 0 <= idx < len(fonts) else "-"
size = float(run.get("FontSize", default.get("FontSize", 0.0)))
fillc = run.get("FillColor", default.get("FillColor"))
vals = [float(x) for x in fillc["Values"]] if fillc is not None and "Values" in fillc else [0.0, 0.0, 0.0, 1.0]
tracking = float(run.get("Tracking", default.get("Tracking", 0.0)))
just = int(g(d, "EngineDict", "ParagraphRun", "RunArray", 0, "ParagraphSheet", "Properties", "Justification"))

print("text_hex=" + s(text).encode("utf-8").hex())
print("fonts=" + ",".join(fonts))
print("font=" + font)
print("size=" + repr(size))
print("fill=" + ",".join(repr(v) for v in vals))
print("tracking=" + repr(tracking))
print("just=" + str(just))
"#;
    let out = Command::new("python3")
        .arg("-c")
        .arg(script)
        .arg(path)
        .output()
        .expect("run python3");
    assert!(
        out.status.success(),
        "psd-tools failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}
