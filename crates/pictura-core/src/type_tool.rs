//! A derived view of a layer's `TySh` type-tool block.
//!
//! The raw `TySh` tagged block stays in [`crate::Layer::extra_blocks`] and is
//! re-emitted verbatim on an unmodified save; this view exposes the affine
//! transform, the `Txt ` string, and the bounds. `text_desc`/`warp_desc` keep
//! the full version-16 descriptor bytes from the input so a re-encode is
//! framing-only.

/// Decoded type-tool framing. `transform` is `xx, xy, yx, yy, tx, ty`;
/// `bounds` is `left, top, right, bottom`.
#[derive(Debug, Clone)]
pub struct TypeTool {
    pub transform: [f64; 6],
    pub text: String,
    pub bounds: [i32; 4],
    pub text_desc: Vec<u8>,
    pub warp_desc: Vec<u8>,
    pub fonts: Vec<String>,
    pub style: Option<TextStyle>,
}

/// Effective text style of a type layer's first style run, with the
/// paragraph/style defaults already applied.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TextStyle {
    pub font: Option<String>,
    pub font_size: f64,
    pub fill_color: [f64; 4],
    pub tracking: f64,
    pub justification: u8,
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
    }
}

impl Eq for TypeTool {}
