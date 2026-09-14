//! `Image > Canvas Size` at document scope (`IMG-002`).

// M12-A2: translate every layer rect by the anchor offset, re-extend the
// document channels with fill 0, then recompute `doc.composite = composite_rgba(doc)`.
pub fn resize_canvas_document(
    doc: &mut pictura_core::Document,
    width: u32,
    height: u32,
    anchor: pictura_ops::Anchor,
    background: [u8; 4],
) -> Result<(), pictura_ops::OpsError> {
    let _ = (doc, width, height, anchor, background);
    Err(pictura_ops::OpsError::Unsupported(
        "resize_canvas_document not implemented yet".into(),
    ))
}
