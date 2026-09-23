//! Bitmap write-back: decide whether a document read from a depth-1 Bitmap PSD
//! can be re-emitted byte-exactly as header mode Bitmap (0). Split from
//! `write.rs` to stay under the file-size cap.

use pictura_core::{BitDepth, ColorMode, Document};

use crate::write::composite_retained;

/// True when `doc` can be written back as a flat, unchanged Bitmap: a depth-1
/// Bitmap source with no layers or extra channels whose retained packed plane
/// still expands to the current working RGB composite. A Bitmap read expands the
/// packed bits to RGB, so an edit makes this false and the save falls back to the
/// working mode; no RGB-to-1-bit threshold is invented, and layered or
/// extra-channel Bitmap write-back is out of scope.
pub(crate) fn writes_bitmap(doc: &Document) -> bool {
    if doc.source_mode != Some(ColorMode::Bitmap)
        || doc.composite.channels != 3
        || !doc.merged_composite_present
        || !doc.layers.is_empty()
        || !doc.channels.is_empty()
        || doc.source_planes.as_ref().map(|s| s.depth) != Some(BitDepth::One)
    {
        return false;
    }
    let Some(retained) = composite_retained(doc, 1, 0) else {
        return false;
    };
    // `get`-style length safety: a short `composite.data` on a public document
    // must fall back to RGB, not compare or slice out of bounds.
    crate::color_mode::bitmap_rows_to_rgb(retained, doc.width as usize, doc.height as usize)
        == doc.composite.data
}
