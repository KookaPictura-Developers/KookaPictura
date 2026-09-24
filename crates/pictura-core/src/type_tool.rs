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
}

impl PartialEq for TypeTool {
    fn eq(&self, other: &Self) -> bool {
        self.transform.map(f64::to_bits) == other.transform.map(f64::to_bits)
            && self.text == other.text
            && self.bounds == other.bounds
            && self.text_desc == other.text_desc
            && self.warp_desc == other.warp_desc
    }
}

impl Eq for TypeTool {}
