//! `Image > Image Rotation` and flips at document scope (`IMG-003`).

// M12-A3: exact index remaps of every layer rect and channel plane (90/270 swap
// the document dimensions), then recompute `doc.composite = composite_rgba(doc)`.
pub fn rotate_document(
    doc: &mut pictura_core::Document,
    quarter_turns: u8,
) -> Result<(), pictura_ops::OpsError> {
    let _ = (doc, quarter_turns);
    Err(pictura_ops::OpsError::Unsupported(
        "rotate_document not implemented yet".into(),
    ))
}

pub fn flip_document(doc: &mut pictura_core::Document, horizontal: bool) {
    let _ = (doc, horizontal);
}
