//! Image codecs. M0 scope: minimal PSD/PSB read + write of a **single composite
//! image**. M1-B adds the **Layer and Mask Information** section: layer records,
//! channel image data (raw + PackBits RLE), group/section markers, Unicode
//! names, and raster layer masks, on top of the unchanged composite path.
//!
//! - Read: file header, color mode data and image resources (kept verbatim),
//!   the layer/mask section (parsed when present), then the image data section.
//!   Supported: 8-bit, RGB or Grayscale, composite and layer channel
//!   compression 0 (raw), 1 (RLE/PackBits), 2 (ZIP), or 3 (ZIP-with-prediction).
//!   An unrecognized layer blend key degrades to Normal but is kept verbatim for
//!   re-save, unmodeled layer blocks and channels are retained, and a layered
//!   file with no merged composite ("Maximize Compatibility" off) yields a
//!   zeroed composite instead of a truncation error.
//! - Write: emit a valid PSD whose layer section round-trips through
//!   [`read_psd`], using PackBits RLE channel data and `'luni'`/`'lsct'` tagged
//!   blocks, and re-emit every preserved verbatim block so an open→save loses
//!   nothing.
//! - Anything outside the supported subset returns [`PsdError::Unsupported`],
//!   never a panic.

mod common;
mod descriptor;
mod error;
mod read;
mod smart_filter;
mod smart_object;
mod smart_writer;
mod write;

#[cfg(test)]
mod smart_writer_tests;
#[cfg(test)]
mod tests;

pub use descriptor::{write_descriptor, DescValue};
pub use error::PsdError;
pub use read::read_psd;
pub use smart_filter::set_camera_raw_option;
pub use write::write_psd;

/// Read a Camera Raw Filter's `Fltr` options from a
/// [`pictura_core::SmartFilter::options`] byte buffer.
pub fn camera_raw_options(options: &[u8]) -> Result<DescValue, PsdError> {
    let mut reader = common::Reader::new(options);
    descriptor::read_descriptor(&mut reader)
}

/// Read a bare version-16 `DescriptorBlock` (for example an adjustment-layer
/// payload) into its [`DescValue::Object`].
pub fn read_descriptor(bytes: &[u8]) -> Result<DescValue, PsdError> {
    let mut reader = common::Reader::new(bytes);
    descriptor::read_descriptor(&mut reader)
}
