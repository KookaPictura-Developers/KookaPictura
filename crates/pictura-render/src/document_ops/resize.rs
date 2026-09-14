//! `Image > Image Size` at document scope (`IMG-001`).

// M12-A1: scale every layer rect, resample its channel planes and mask from the
// old rect size to the new, then recompute `doc.composite = composite_rgba(doc)`.
pub fn resize_document(
    doc: &mut pictura_core::Document,
    width: u32,
    height: u32,
    resample: pictura_ops::Resample,
) -> Result<(), pictura_ops::OpsError> {
    let _ = (doc, width, height, resample);
    Err(pictura_ops::OpsError::Unsupported(
        "resize_document not implemented yet".into(),
    ))
}
