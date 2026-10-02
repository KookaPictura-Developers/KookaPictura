//! A derived view of a layer's `TySh` type-tool block.
//!
//! The raw `TySh` tagged block stays in [`crate::Layer::extra_blocks`] and is
//! re-emitted verbatim on an unmodified save; this view exposes the affine
//! transform, the `Txt ` string, and the bounds. `text_desc`/`warp_desc` keep
//! the full version-16 descriptor bytes from the input so a re-encode is
//! framing-only.

/// Decoded type-tool framing. `transform` is `xx, xy, yx, yy, tx, ty`;
/// `bounds` is `left, top, right, bottom`. `vertical` is the text
/// descriptor's `Ornt` (`Vrtc`): the Vertical Type tool's columns.
#[derive(Debug, Clone)]
pub struct TypeTool {
    pub transform: [f64; 6],
    pub text: String,
    pub bounds: [i32; 4],
    pub text_desc: Vec<u8>,
    pub warp_desc: Vec<u8>,
    pub fonts: Vec<String>,
    pub style: Option<TextStyle>,
    pub vertical: bool,
}

/// Effective text style of a type layer's first style run, with the
/// paragraph/style defaults already applied. `fill_color` keeps EngineData's
/// `Values` order: alpha, red, green, blue.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TextStyle {
    pub font: Option<String>,
    pub font_size: f64,
    pub fill_color: [f64; 4],
    pub tracking: f64,
    pub justification: u8,
}

impl TextStyle {
    /// `fill_color` as 8-bit straight RGBA.
    pub fn rgba(&self) -> [u8; 4] {
        let [a, r, g, b] = self
            .fill_color
            .map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8);
        [r, g, b, a]
    }
}

/// What the Type tools commit: a point-type string and how it is set.
///
/// `text` separates lines with `\r`, as `TySh` does. `origin` is the click in
/// document pixels: the first baseline's start (horizontal) or the first
/// column's top centre (vertical), moved by `justification` — 0 left/top,
/// 1 right/bottom, 2 centre. `font` is a family name; `size` is in pixels.
/// `matrix` is the linear part of the `TySh` transform (`xx, xy, yx, yy`:
/// `x' = xx·x + yx·y`, `y' = xy·x + yy·y`) mapping the laid-out text about the
/// origin — Free Transform's scale and rotation; identity for new type.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeSpec {
    pub text: String,
    pub font: String,
    pub size: f64,
    pub color: [u8; 4],
    pub justification: u8,
    pub vertical: bool,
    pub antialias: bool,
    pub origin: (f64, f64),
    pub matrix: [f64; 4],
}

impl TypeSpec {
    pub const IDENTITY: [f64; 4] = [1.0, 0.0, 0.0, 1.0];
}

impl PartialEq for TypeTool {
    fn eq(&self, other: &Self) -> bool {
        self.transform.map(f64::to_bits) == other.transform.map(f64::to_bits)
            && self.text == other.text
            && self.bounds == other.bounds
            && self.text_desc == other.text_desc
            && self.warp_desc == other.warp_desc
            && self.fonts == other.fonts
            && self.style == other.style
            && self.vertical == other.vertical
    }
}

impl Eq for TypeTool {}
