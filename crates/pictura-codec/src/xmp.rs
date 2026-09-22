//! XMP (RDF/XML) property reader and span patcher.
//!
//! A bounded, entity-free scanner locates the managed properties inside the
//! `rdf:Description` element(s) and records their byte spans in the original
//! packet; [`patch_xmp`] replaces only those spans, so unknown namespaces,
//! unknown properties, comments, and the packet wrapper round-trip verbatim.
//! Only the five predefined XML entities are decoded and no external resource
//! is ever fetched, so an XXE payload has no effect.
//!
//! ponytail: only the fixed property set is modelled; arbitrary RDF and IPTC
//! Extension are preserved but not editable. A packet without a recognisable
//! `rdf:Description`, or a managed property using a construct the patcher
//! cannot safely rewrite, leaves the resource untouched.

use std::collections::HashMap;

const RDF_URI: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
const DC_URI: &str = "http://purl.org/dc/elements/1.1/";
const PHOTOSHOP_URI: &str = "http://ns.adobe.com/photoshop/1.0/";
const XMPRIGHTS_URI: &str = "http://ns.adobe.com/xap/1.0/rights/";

/// Caps so a hostile packet cannot spin the scanner.
const MAX_TOKENS: usize = 8192;
const MAX_ATTRS: usize = 8192;
const MAX_DEPTH: usize = 64;
const MAX_PACKET_BYTES: usize = 8 << 20;
const MAX_VALUE_BYTES: usize = 1 << 20;
const MAX_ENTITY_RUN: usize = 32;

/// A managed XMP property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum XmpField {
    Title,
    Creator,
    Description,
    Subject,
    Rights,
    Credit,
    Source,
    Headline,
    Marked,
}

/// The managed property set of a document's XMP packet.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct XmpProperties {
    pub title: Option<String>,
    pub creator: Vec<String>,
    pub description: Option<String>,
    pub subject: Vec<String>,
    pub rights: Option<String>,
    pub credit: Option<String>,
    pub source: Option<String>,
    pub headline: Option<String>,
    pub marked: Option<bool>,
}

impl XmpProperties {
    /// Whether no managed property is present.
    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.creator.is_empty()
            && self.description.is_none()
            && self.subject.is_empty()
            && self.rights.is_none()
            && self.credit.is_none()
            && self.source.is_none()
            && self.headline.is_none()
            && self.marked.is_none()
    }
}

/// Parse the fixed property set out of an XMP packet. A malformed or truncated
/// packet yields whatever decoded so far and never panics.
pub fn parse_xmp(packet: &str) -> XmpProperties {
    scan_packet(packet.as_bytes()).properties()
}

/// Set managed properties in an XMP packet by replacing only the bytes of each
/// matched property and copying every other byte verbatim. Setting a property
/// that already has the requested value returns the input unchanged. Returns
/// `None` when the packet has no recognisable `rdf:Description`, or a managed
/// property that must be rewritten uses a construct that cannot be safely
/// reserialised (the caller then leaves the resource untouched).
pub fn patch_xmp(packet: &str, updates: &[(XmpField, String)]) -> Option<String> {
    if updates.is_empty() {
        return Some(packet.to_string());
    }
    let bytes = packet.as_bytes();
    let scan = scan_packet(bytes);
    let description = scan.description.as_ref()?;
    if scan.malformed || scan.over_limit {
        return None;
    }

    let mut requested: Vec<(XmpField, String)> = Vec::new();
    for (field, value) in updates {
        match requested.iter_mut().find(|(f, _)| f == field) {
            Some(slot) => slot.1 = value.clone(),
            None => requested.push((*field, value.clone())),
        }
    }

    let mut edits: Vec<(usize, usize, String)> = Vec::new();
    let mut insertions = String::new();
    for (field, value) in &requested {
        if scan.spans.iter().filter(|s| s.field == *field).count() > 1 {
            return None;
        }
        match scan.spans.iter().find(|s| s.field == *field) {
            Some(span) => {
                if span.unsafe_ || !span.complete {
                    return None;
                }
                if value.is_empty() {
                    // Clearing removes the property bytes entirely; an absent
                    // property already satisfies an empty request.
                    edits.push((span.start, span.end, String::new()));
                    continue;
                }
                if *field == XmpField::Marked && parse_xmp_bool(value).is_none() {
                    return None;
                }
                if already_matches(span, *field, value) {
                    continue;
                }
                edits.push((span.start, span.end, replacement(span, *field, value)?));
            }
            None => {
                if value.is_empty() {
                    continue;
                }
                if *field == XmpField::Marked && parse_xmp_bool(value).is_none() {
                    return None;
                }
                insertions.push_str(&insertion(*field, value, &description.scope)?);
            }
        }
    }
    if edits.is_empty() && insertions.is_empty() {
        return Some(packet.to_string());
    }

    if !insertions.is_empty() {
        // ponytail: a self-closing rdf:Description that needs both an attribute
        // edit and an insertion fails closed, because the insertion rewrites the
        // whole start tag and would overlap the attribute's span.
        if description.self_closing {
            let raw = &bytes[description.start_tag.0..description.start_tag.1];
            let head = raw.strip_suffix(b"/>")?;
            let head = std::str::from_utf8(head).ok()?;
            let qname = str_at(bytes, description.qname);
            edits.push((
                description.start_tag.0,
                description.start_tag.1,
                format!("{head}>{insertions}</{qname}>"),
            ));
        } else {
            let at = description.content_end;
            edits.push((at, at, insertions));
        }
    }

    edits.sort_by_key(|e| (e.0, e.1));
    for pair in edits.windows(2) {
        if pair[1].0 < pair[0].1 {
            return None;
        }
    }
    let mut out = String::with_capacity(packet.len() + 64);
    let mut cursor = 0;
    for (start, end, fragment) in &edits {
        if *start < cursor {
            return None;
        }
        out.push_str(&packet[cursor..*start]);
        out.push_str(fragment);
        cursor = *end;
    }
    out.push_str(&packet[cursor..]);
    Some(out)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Form {
    Attribute,
    Element,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContainerKind {
    Alt,
    Seq,
    Bag,
}

#[derive(Debug, Clone)]
struct LiVal {
    lang: Option<String>,
    value: String,
}

#[derive(Debug, Clone)]
struct SpanInfo {
    field: XmpField,
    form: Form,
    start: usize,
    end: usize,
    name: String,
    values: Vec<LiVal>,
    container: Option<(ContainerKind, String)>,
    unsafe_: bool,
    complete: bool,
}

impl SpanInfo {
    fn scalar_value(&self) -> Option<String> {
        self.values
            .iter()
            .find(|v| v.lang.as_deref() == Some("x-default"))
            .or_else(|| self.values.first())
            .map(|v| v.value.clone())
    }
}

#[derive(Debug, Clone)]
struct Description {
    qname: (usize, usize),
    start_tag: (usize, usize),
    self_closing: bool,
    content_end: usize,
    /// The namespace declarations in scope at the Description start tag, so an
    /// insertion only reuses a prefix actually bound there.
    scope: HashMap<String, String>,
}

#[derive(Debug, Default)]
struct Scan {
    spans: Vec<SpanInfo>,
    description: Option<Description>,
    malformed: bool,
    over_limit: bool,
}

impl Scan {
    fn properties(&self) -> XmpProperties {
        let mut props = XmpProperties::default();
        for span in &self.spans {
            match span.field {
                XmpField::Title if props.title.is_none() => props.title = span.scalar_value(),
                XmpField::Creator if props.creator.is_empty() => {
                    props.creator = span.values.iter().map(|v| v.value.clone()).collect();
                }
                XmpField::Description if props.description.is_none() => {
                    props.description = span.scalar_value();
                }
                XmpField::Subject if props.subject.is_empty() => {
                    props.subject = span.values.iter().map(|v| v.value.clone()).collect();
                }
                XmpField::Rights if props.rights.is_none() => props.rights = span.scalar_value(),
                XmpField::Credit if props.credit.is_none() => props.credit = span.scalar_value(),
                XmpField::Source if props.source.is_none() => props.source = span.scalar_value(),
                XmpField::Headline if props.headline.is_none() => {
                    props.headline = span.scalar_value();
                }
                XmpField::Marked if props.marked.is_none() => {
                    props.marked = span.scalar_value().and_then(|v| parse_xmp_bool(&v));
                }
                _ => {}
            }
        }
        props
    }
}

/// The lexicographically smallest declared prefix bound to `uri` in `scope`.
fn prefix_for(scope: &HashMap<String, String>, uri: &str) -> Option<String> {
    scope
        .iter()
        .filter(|(prefix, declared)| !prefix.is_empty() && declared.as_str() == uri)
        .map(|(prefix, _)| prefix.as_str())
        .min()
        .map(str::to_string)
}

#[derive(Debug, Clone)]
struct RawAttr {
    name: (usize, usize),
    value: (usize, usize),
}

#[derive(Debug)]
enum RawKind {
    Start {
        qname: (usize, usize),
        self_closing: bool,
        attrs: Vec<RawAttr>,
    },
    End {
        qname: (usize, usize),
    },
    Text,
    Ignored,
}

#[derive(Debug)]
struct RawToken {
    start: usize,
    end: usize,
    kind: RawKind,
}

#[derive(Debug)]
struct Frame {
    qname: (usize, usize),
    is_description: bool,
    field: Option<XmpField>,
    start: usize,
    values: Vec<LiVal>,
    container: Option<(ContainerKind, String)>,
    unsafe_: bool,
    text_buf: Option<Vec<u8>>,
    is_li: bool,
    li_lang: Option<String>,
    li_owner: Option<usize>,
    /// Declarations this element added, restored when its scope closes.
    ns_undo: Vec<(String, Option<String>)>,
}

fn managed_field(uri: &str, local: &str) -> Option<XmpField> {
    Some(match (uri, local) {
        (DC_URI, "title") => XmpField::Title,
        (DC_URI, "creator") => XmpField::Creator,
        (DC_URI, "description") => XmpField::Description,
        (DC_URI, "subject") => XmpField::Subject,
        (DC_URI, "rights") => XmpField::Rights,
        (PHOTOSHOP_URI, "Credit") => XmpField::Credit,
        (PHOTOSHOP_URI, "Source") => XmpField::Source,
        (PHOTOSHOP_URI, "Headline") => XmpField::Headline,
        (XMPRIGHTS_URI, "Marked") => XmpField::Marked,
        _ => return None,
    })
}

fn canonical(field: XmpField) -> (&'static str, &'static str) {
    match field {
        XmpField::Title => (DC_URI, "title"),
        XmpField::Creator => (DC_URI, "creator"),
        XmpField::Description => (DC_URI, "description"),
        XmpField::Subject => (DC_URI, "subject"),
        XmpField::Rights => (DC_URI, "rights"),
        XmpField::Credit => (PHOTOSHOP_URI, "Credit"),
        XmpField::Source => (PHOTOSHOP_URI, "Source"),
        XmpField::Headline => (PHOTOSHOP_URI, "Headline"),
        XmpField::Marked => (XMPRIGHTS_URI, "Marked"),
    }
}

/// Whether the property already holds the requested value. The comparison is
/// on the property's scalar value — the `x-default` item for an `rdf:Alt`, the
/// first item for `rdf:Seq`/`rdf:Bag`, the text for a simple property — so it
/// agrees with the reader and with the IPTC-Core sync gate.
fn already_matches(span: &SpanInfo, field: XmpField, value: &str) -> bool {
    if field == XmpField::Marked {
        return span.values.first().and_then(|v| parse_xmp_bool(&v.value)) == parse_xmp_bool(value);
    }
    span.scalar_value().as_deref() == Some(value)
}

fn replacement(span: &SpanInfo, field: XmpField, value: &str) -> Option<String> {
    let escaped = escape_xml(value);
    if span.form == Form::Attribute {
        return Some(format!("{}=\"{}\"", span.name, escaped));
    }
    let name = span.name.as_str();
    let rdf = span.container.as_ref().map(|(_, prefix)| prefix.as_str());
    // ponytail: a rewritten Alt/Seq/Bag collapses to a single item; the public
    // update API carries one value per field.
    match field {
        XmpField::Title | XmpField::Description | XmpField::Rights if rdf.is_some() => {
            let rdf = rdf?;
            if rdf.is_empty() {
                return None;
            }
            Some(format!(
                "<{name}><{rdf}:Alt><{rdf}:li xml:lang=\"x-default\">{escaped}</{rdf}:li></{rdf}:Alt></{name}>"
            ))
        }
        XmpField::Creator if rdf.is_some() => {
            let rdf = rdf?;
            if rdf.is_empty() {
                return None;
            }
            Some(format!(
                "<{name}><{rdf}:Seq><{rdf}:li>{escaped}</{rdf}:li></{rdf}:Seq></{name}>"
            ))
        }
        XmpField::Subject if rdf.is_some() => {
            let rdf = rdf?;
            if rdf.is_empty() {
                return None;
            }
            Some(format!(
                "<{name}><{rdf}:Bag><{rdf}:li>{escaped}</{rdf}:li></{rdf}:Bag></{name}>"
            ))
        }
        XmpField::Marked => {
            let text = if parse_xmp_bool(value)? {
                "True"
            } else {
                "False"
            };
            Some(format!("<{name}>{text}</{name}>"))
        }
        _ => Some(format!("<{name}>{escaped}</{name}>")),
    }
}

fn insertion(field: XmpField, value: &str, scope: &HashMap<String, String>) -> Option<String> {
    let escaped = escape_xml(value);
    let (uri, local) = canonical(field);
    // ponytail: an inserted property reuses a prefix the receiving
    // rdf:Description already has in scope; an unbound prefix fails closed
    // rather than editing the namespace declarations.
    let prefix = prefix_for(scope, uri)?;
    match field {
        XmpField::Title | XmpField::Description | XmpField::Rights => {
            let rdf = prefix_for(scope, RDF_URI)?;
            Some(format!(
                "<{prefix}:{local}><{rdf}:Alt><{rdf}:li xml:lang=\"x-default\">{escaped}</{rdf}:li></{rdf}:Alt></{prefix}:{local}>"
            ))
        }
        XmpField::Creator => {
            let rdf = prefix_for(scope, RDF_URI)?;
            Some(format!(
                "<{prefix}:{local}><{rdf}:Seq><{rdf}:li>{escaped}</{rdf}:li></{rdf}:Seq></{prefix}:{local}>"
            ))
        }
        XmpField::Subject => {
            let rdf = prefix_for(scope, RDF_URI)?;
            Some(format!(
                "<{prefix}:{local}><{rdf}:Bag><{rdf}:li>{escaped}</{rdf}:li></{rdf}:Bag></{prefix}:{local}>"
            ))
        }
        XmpField::Marked => {
            let text = if parse_xmp_bool(value)? {
                "True"
            } else {
                "False"
            };
            Some(format!("<{prefix}:{local}>{text}</{prefix}:{local}>"))
        }
        _ => Some(format!("<{prefix}:{local}>{escaped}</{prefix}:{local}>")),
    }
}

fn parse_xmp_bool(value: &str) -> Option<bool> {
    if value.eq_ignore_ascii_case("true") || value == "1" {
        Some(true)
    } else if value.eq_ignore_ascii_case("false") || value == "0" {
        Some(false)
    } else {
        None
    }
}

fn escape_xml(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

fn str_at(bytes: &[u8], span: (usize, usize)) -> &str {
    std::str::from_utf8(&bytes[span.0..span.1]).unwrap_or("")
}

fn split_qname(name: &str) -> (&str, &str) {
    match name.split_once(':') {
        Some((prefix, local)) => (prefix, local),
        None => ("", name),
    }
}

fn xmlns_prefix(name: &str) -> Option<&str> {
    if name == "xmlns" {
        Some("")
    } else {
        name.strip_prefix("xmlns:")
    }
}

fn push_declarations(
    current: &mut HashMap<String, String>,
    bytes: &[u8],
    attrs: &[RawAttr],
) -> Vec<(String, Option<String>)> {
    let mut declared = Vec::new();
    for attr in attrs {
        let name = str_at(bytes, attr.name);
        if let Some(prefix) = xmlns_prefix(name) {
            let uri = decode_text(&bytes[attr.value.0..attr.value.1]).text;
            let old = current.insert(prefix.to_string(), uri);
            declared.push((prefix.to_string(), old));
        }
    }
    declared
}

fn restore_declarations(
    current: &mut HashMap<String, String>,
    declared: Vec<(String, Option<String>)>,
) {
    for (prefix, old) in declared.into_iter().rev() {
        match old {
            Some(uri) => {
                current.insert(prefix, uri);
            }
            None => {
                current.remove(&prefix);
            }
        }
    }
}

fn nearest_field(stack: &[Frame]) -> Option<usize> {
    stack.iter().rposition(|frame| frame.field.is_some())
}

struct Decoded {
    text: String,
    unsafe_: bool,
}

fn decode_text(raw: &[u8]) -> Decoded {
    let source = String::from_utf8_lossy(raw);
    let mut out = String::with_capacity(source.len());
    let mut unsafe_ = false;
    let mut rest = source.as_ref();
    while let Some(index) = rest.find('&') {
        out.push_str(&rest[..index]);
        let after = &rest[index + 1..];
        let mut semi = None;
        for (offset, ch) in after.char_indices() {
            if offset > MAX_ENTITY_RUN {
                break;
            }
            if ch == ';' {
                semi = Some(offset);
                break;
            }
        }
        match semi {
            Some(offset) => {
                match &after[..offset] {
                    "amp" => out.push('&'),
                    "lt" => out.push('<'),
                    "gt" => out.push('>'),
                    "quot" => out.push('"'),
                    "apos" => out.push('\''),
                    other => {
                        unsafe_ = true;
                        out.push('&');
                        out.push_str(other);
                        out.push(';');
                    }
                }
                rest = &after[offset + 1..];
            }
            None => {
                out.push('&');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    Decoded { text: out, unsafe_ }
}

fn read_name_end(bytes: &[u8], start: usize, limit: usize) -> Option<usize> {
    if start >= limit {
        return None;
    }
    let mut i = start;
    while i < limit {
        match bytes[i] {
            b' ' | b'\t' | b'\r' | b'\n' | b'/' | b'>' | b'=' => break,
            _ => i += 1,
        }
    }
    Some(i)
}

fn skip_ws(bytes: &[u8], mut i: usize, limit: usize) -> usize {
    while i < limit && matches!(bytes[i], b' ' | b'\t' | b'\r' | b'\n') {
        i += 1;
    }
    i
}

fn find_from(bytes: &[u8], start: usize, needle: &[u8], limit: usize) -> Option<usize> {
    if start > limit || needle.is_empty() {
        return None;
    }
    let mut i = start;
    while i + needle.len() <= limit {
        if &bytes[i..i + needle.len()] == needle {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn tokenize(bytes: &[u8]) -> (Vec<RawToken>, bool) {
    let mut tokens: Vec<RawToken> = Vec::new();
    let limit = bytes.len().min(MAX_PACKET_BYTES);
    let mut pos = 0;
    while pos < limit {
        if tokens.len() >= MAX_TOKENS {
            return (tokens, false);
        }
        if bytes[pos] != b'<' {
            let start = pos;
            while pos < limit && bytes[pos] != b'<' {
                pos += 1;
            }
            tokens.push(RawToken {
                start,
                end: pos,
                kind: RawKind::Text,
            });
            continue;
        }
        if bytes[pos..].starts_with(b"<!--") {
            let Some(close) = find_from(bytes, pos + 4, b"-->", limit) else {
                return (tokens, false);
            };
            tokens.push(RawToken {
                start: pos,
                end: close + 3,
                kind: RawKind::Ignored,
            });
            pos = close + 3;
            continue;
        }
        if bytes[pos..].starts_with(b"<![CDATA[") {
            let Some(close) = find_from(bytes, pos + 9, b"]]>", limit) else {
                return (tokens, false);
            };
            tokens.push(RawToken {
                start: pos + 9,
                end: close,
                kind: RawKind::Text,
            });
            pos = close + 3;
            continue;
        }
        if bytes[pos..].starts_with(b"</") {
            let name_start = pos + 2;
            let Some(name_end) = read_name_end(bytes, name_start, limit) else {
                return (tokens, false);
            };
            let Some(gt) = find_from(bytes, name_end, b">", limit) else {
                return (tokens, false);
            };
            tokens.push(RawToken {
                start: pos,
                end: gt + 1,
                kind: RawKind::End {
                    qname: (name_start, name_end),
                },
            });
            pos = gt + 1;
            continue;
        }
        if bytes[pos..].starts_with(b"<?") {
            let Some(close) = find_from(bytes, pos + 2, b"?>", limit) else {
                return (tokens, false);
            };
            tokens.push(RawToken {
                start: pos,
                end: close + 2,
                kind: RawKind::Ignored,
            });
            pos = close + 2;
            continue;
        }
        if bytes[pos..].starts_with(b"<!") {
            let mut i = pos + 2;
            let mut depth = 0i32;
            let mut end = None;
            while i < limit {
                match bytes[i] {
                    b'[' => depth += 1,
                    b']' => depth -= 1,
                    b'>' if depth <= 0 => {
                        end = Some(i);
                        break;
                    }
                    _ => {}
                }
                i += 1;
            }
            let Some(close) = end else {
                return (tokens, false);
            };
            tokens.push(RawToken {
                start: pos,
                end: close + 1,
                kind: RawKind::Ignored,
            });
            pos = close + 1;
            continue;
        }
        let name_start = pos + 1;
        let Some(name_end) = read_name_end(bytes, name_start, limit) else {
            return (tokens, false);
        };
        if name_end == name_start {
            return (tokens, false);
        }
        let mut attrs: Vec<RawAttr> = Vec::new();
        let mut self_closing = false;
        let mut i = name_end;
        loop {
            i = skip_ws(bytes, i, limit);
            if i >= limit {
                return (tokens, false);
            }
            match bytes[i] {
                b'>' => {
                    i += 1;
                    break;
                }
                b'/' => {
                    if i + 1 < limit && bytes[i + 1] == b'>' {
                        self_closing = true;
                        i += 2;
                        break;
                    }
                    return (tokens, false);
                }
                _ => {}
            }
            let Some(attr_end) = read_name_end(bytes, i, limit) else {
                return (tokens, false);
            };
            if attr_end == i {
                return (tokens, false);
            }
            let name = (i, attr_end);
            i = skip_ws(bytes, attr_end, limit);
            if i >= limit || bytes[i] != b'=' {
                return (tokens, false);
            }
            i += 1;
            i = skip_ws(bytes, i, limit);
            if i >= limit || (bytes[i] != b'"' && bytes[i] != b'\'') {
                return (tokens, false);
            }
            let quote = bytes[i];
            i += 1;
            let value_start = i;
            while i < limit && bytes[i] != quote {
                i += 1;
            }
            if i >= limit {
                return (tokens, false);
            }
            if attrs.len() >= MAX_ATTRS {
                return (tokens, false);
            }
            attrs.push(RawAttr {
                name,
                value: (value_start, i),
            });
            i += 1;
        }
        tokens.push(RawToken {
            start: pos,
            end: i,
            kind: RawKind::Start {
                qname: (name_start, name_end),
                self_closing,
                attrs,
            },
        });
        pos = i;
    }
    (tokens, true)
}

fn scan_packet(bytes: &[u8]) -> Scan {
    let (tokens, complete) = tokenize(bytes);
    let mut scan = Scan {
        malformed: !complete,
        // A packet past the byte cap may have been tokenised as "complete" on a
        // token boundary, so refuse to patch it.
        over_limit: bytes.len() > MAX_PACKET_BYTES,
        ..Scan::default()
    };
    let mut stack: Vec<Frame> = Vec::new();
    let mut current: HashMap<String, String> = HashMap::new();

    for tok in &tokens {
        if stack.len() > MAX_DEPTH {
            scan.over_limit = true;
            break;
        }
        match &tok.kind {
            RawKind::Text => {
                let sink = stack.iter().rposition(|frame| frame.text_buf.is_some());
                if let Some(index) = sink {
                    let frame = &mut stack[index];
                    let len = tok.end - tok.start;
                    match &mut frame.text_buf {
                        Some(buf) if buf.len() + len <= MAX_VALUE_BYTES => {
                            buf.extend_from_slice(&bytes[tok.start..tok.end]);
                        }
                        _ => frame.unsafe_ = true,
                    }
                }
            }
            RawKind::Ignored => {
                // A comment/PI/declaration inside a managed property would be
                // dropped by the rewrite, so fail closed instead.
                if let Some(owner) = nearest_field(&stack) {
                    stack[owner].unsafe_ = true;
                }
            }
            RawKind::Start {
                qname,
                self_closing,
                attrs,
            } => {
                let declared = push_declarations(&mut current, bytes, attrs);
                let name = str_at(bytes, *qname);
                let (prefix, local) = split_qname(name);
                let uri = current.get(prefix).cloned();
                let field = uri.as_deref().and_then(|u| managed_field(u, local));
                let is_rdf = uri.as_deref() == Some(RDF_URI);
                let is_description = is_rdf && local == "Description";
                let parent_is_description = stack.last().is_some_and(|frame| frame.is_description);

                let mut pushed: Option<Frame> = None;
                if is_description {
                    for attr in attrs {
                        let attr_name = str_at(bytes, attr.name);
                        let (attr_prefix, attr_local) = split_qname(attr_name);
                        let Some(attr_uri) = current.get(attr_prefix) else {
                            continue;
                        };
                        let Some(managed) = managed_field(attr_uri, attr_local) else {
                            continue;
                        };
                        let decoded = decode_text(&bytes[attr.value.0..attr.value.1]);
                        scan.spans.push(SpanInfo {
                            field: managed,
                            form: Form::Attribute,
                            start: attr.name.0,
                            end: attr.value.1 + 1,
                            name: attr_name.to_string(),
                            values: vec![LiVal {
                                lang: None,
                                value: decoded.text,
                            }],
                            container: None,
                            unsafe_: decoded.unsafe_,
                            complete: true,
                        });
                    }
                    if scan.description.is_none() {
                        scan.description = Some(Description {
                            qname: *qname,
                            start_tag: (tok.start, tok.end),
                            self_closing: *self_closing,
                            content_end: tok.end,
                            scope: current.clone(),
                        });
                    }
                    if !*self_closing {
                        let mut description = frame(*qname, tok.start);
                        description.is_description = true;
                        pushed = Some(description);
                    }
                } else if let Some(managed) = field.filter(|_| parent_is_description) {
                    // The writer emits no attributes on a managed property
                    // element, so any attribute on it would be dropped by the
                    // rewrite.
                    let unsafe_ = !attrs.is_empty();
                    if *self_closing {
                        scan.spans.push(SpanInfo {
                            field: managed,
                            form: Form::Element,
                            start: tok.start,
                            end: tok.end,
                            name: name.to_string(),
                            values: Vec::new(),
                            container: None,
                            unsafe_,
                            complete: true,
                        });
                    } else {
                        let mut managed_frame = frame(*qname, tok.start);
                        managed_frame.field = Some(managed);
                        managed_frame.unsafe_ = unsafe_;
                        managed_frame.text_buf = Some(Vec::new());
                        pushed = Some(managed_frame);
                    }
                } else if is_rdf && matches!(local, "Alt" | "Seq" | "Bag") {
                    if let Some(owner) = nearest_field(&stack) {
                        let kind = match local {
                            "Alt" => ContainerKind::Alt,
                            "Seq" => ContainerKind::Seq,
                            _ => ContainerKind::Bag,
                        };
                        stack[owner].container = Some((kind, prefix.to_string()));
                        stack[owner].text_buf = None;
                        if !attrs.is_empty() {
                            stack[owner].unsafe_ = true;
                        }
                    }
                    if !*self_closing {
                        pushed = Some(frame(*qname, tok.start));
                    }
                } else if is_rdf && local == "li" {
                    let owner = stack
                        .iter()
                        .rposition(|frame| frame.field.is_some() && frame.container.is_some());
                    let lang = attrs.iter().find_map(|attr| {
                        (str_at(bytes, attr.name) == "xml:lang")
                            .then(|| decode_text(&bytes[attr.value.0..attr.value.1]).text)
                    });
                    if let Some(owner) = owner {
                        if attrs
                            .iter()
                            .any(|attr| str_at(bytes, attr.name) != "xml:lang")
                        {
                            stack[owner].unsafe_ = true;
                        }
                    }
                    if *self_closing {
                        if let Some(owner) = owner {
                            stack[owner].values.push(LiVal {
                                lang,
                                value: String::new(),
                            });
                        }
                    } else {
                        let mut li_frame = frame(*qname, tok.start);
                        li_frame.is_li = true;
                        li_frame.li_lang = lang;
                        li_frame.li_owner = owner;
                        li_frame.text_buf = Some(Vec::new());
                        pushed = Some(li_frame);
                    }
                } else {
                    if let Some(owner) = nearest_field(&stack) {
                        stack[owner].unsafe_ = true;
                    }
                    if !*self_closing {
                        pushed = Some(frame(*qname, tok.start));
                    }
                }
                match pushed {
                    Some(mut pending) => {
                        pending.ns_undo = declared;
                        stack.push(pending);
                    }
                    None => restore_declarations(&mut current, declared),
                }
            }
            RawKind::End { qname } => {
                let Some(top) = stack.last() else {
                    scan.malformed = true;
                    break;
                };
                if str_at(bytes, top.qname) != str_at(bytes, *qname) {
                    scan.malformed = true;
                    break;
                }
                let mut frame = stack.pop().expect("checked non-empty");
                restore_declarations(&mut current, std::mem::take(&mut frame.ns_undo));
                if frame.is_description {
                    if let Some(description) = scan.description.as_mut() {
                        description.content_end = tok.start;
                    }
                    continue;
                }
                if frame.is_li {
                    if let Some(owner) = frame.li_owner {
                        let decoded = decode_text(frame.text_buf.as_deref().unwrap_or(&[]));
                        if let Some(target) = stack.get_mut(owner) {
                            target.unsafe_ |= decoded.unsafe_;
                            target.values.push(LiVal {
                                lang: frame.li_lang,
                                value: decoded.text,
                            });
                        }
                    }
                    continue;
                }
                if let Some(managed) = frame.field {
                    let (values, unsafe_) = if frame.container.is_some() {
                        (frame.values, frame.unsafe_)
                    } else {
                        let decoded = decode_text(frame.text_buf.as_deref().unwrap_or(&[]));
                        (
                            vec![LiVal {
                                lang: None,
                                value: decoded.text,
                            }],
                            frame.unsafe_ || decoded.unsafe_,
                        )
                    };
                    scan.spans.push(SpanInfo {
                        field: managed,
                        form: Form::Element,
                        start: frame.start,
                        end: tok.end,
                        name: str_at(bytes, frame.qname).to_string(),
                        values,
                        container: frame.container,
                        unsafe_,
                        complete: true,
                    });
                }
            }
        }
    }

    let unclosed = !stack.is_empty();
    while let Some(mut frame) = stack.pop() {
        restore_declarations(&mut current, std::mem::take(&mut frame.ns_undo));
        if frame.is_li {
            if let Some(owner) = frame.li_owner {
                let decoded = decode_text(frame.text_buf.as_deref().unwrap_or(&[]));
                if let Some(target) = stack.get_mut(owner) {
                    target.values.push(LiVal {
                        lang: frame.li_lang,
                        value: decoded.text,
                    });
                }
            }
            continue;
        }
        if let Some(managed) = frame.field {
            let (values, unsafe_) = if frame.container.is_some() {
                (frame.values, frame.unsafe_)
            } else {
                let decoded = decode_text(frame.text_buf.as_deref().unwrap_or(&[]));
                (
                    vec![LiVal {
                        lang: None,
                        value: decoded.text,
                    }],
                    frame.unsafe_ || decoded.unsafe_,
                )
            };
            scan.spans.push(SpanInfo {
                field: managed,
                form: Form::Element,
                start: frame.start,
                end: bytes.len(),
                name: str_at(bytes, frame.qname).to_string(),
                values,
                container: frame.container,
                unsafe_,
                complete: false,
            });
        }
    }
    scan.malformed |= unclosed;
    scan
}

fn frame(qname: (usize, usize), start: usize) -> Frame {
    Frame {
        qname,
        is_description: false,
        field: None,
        start,
        values: Vec::new(),
        container: None,
        unsafe_: false,
        text_buf: None,
        is_li: false,
        li_lang: None,
        li_owner: None,
        ns_undo: Vec::new(),
    }
}

#[cfg(test)]
mod tests;
