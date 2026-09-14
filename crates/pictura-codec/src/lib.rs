//! Image codecs. M0 scope: minimal PSD/PSB read + write of a **single composite
//! image** (no layers). The full format matrix is specified in
//! `docs/01-architecture/file-formats.md`.
//!
//! ## M0 contract
//!
//! - Read: file header, color mode data (skip), image resources (skip), skip the
//!   layer/mask section if present, then the image data section. Supported:
//!   8-bit, RGB or Grayscale, compression 0 (raw) or 1 (RLE/PackBits).
//! - Write: emit a valid PSD with an empty layer/mask section and raw image
//!   data, such that `read_psd(&write_psd(doc)?)` round-trips.
//! - Anything outside the supported subset returns [`PsdError::Unsupported`],
//!   never a panic.

use pictura_core::Document;

#[derive(Debug, thiserror::Error)]
pub enum PsdError {
    #[error("not a PSD/PSB: bad signature {0:#06x}")]
    BadSignature(u32),
    #[error("unsupported: {0}")]
    Unsupported(String),
    #[error("truncated file")]
    Truncated,
    #[error("invalid data: {0}")]
    Invalid(String),
}

/// Parse a PSD (or PSB) file into a [`Document`] holding the composite image.
pub fn read_psd(bytes: &[u8]) -> Result<Document, PsdError> {
    let _ = bytes;
    unimplemented!("M0-B: implement minimal PSD read")
}

/// Serialize a [`Document`]'s composite image into a valid PSD file.
pub fn write_psd(doc: &Document) -> Result<Vec<u8>, PsdError> {
    let _ = doc;
    unimplemented!("M0-B: implement minimal PSD write")
}
