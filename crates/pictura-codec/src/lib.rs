//! Image codecs. M0 scope: minimal PSD/PSB read + write of a **single composite
//! image**. M1-B adds the **Layer and Mask Information** section: layer records,
//! channel image data (raw + PackBits RLE), group/section markers, Unicode
//! names, and raster layer masks, on top of the unchanged composite path.
//!
//! - Read: file header, color mode data (skip), image resources (skip), the
//!   layer/mask section (parsed when present), then the image data section.
//!   Supported: 8-bit, RGB or Grayscale, composite and layer channel
//!   compression 0 (raw), 1 (RLE/PackBits), 2 (ZIP), or 3 (ZIP-with-prediction).
//!   An unrecognized layer blend key degrades to Normal, and a layered file
//!   with no merged composite ("Maximize Compatibility" off) yields a zeroed
//!   composite instead of a truncation error.
//! - Write: emit a valid PSD whose layer section round-trips through
//!   [`read_psd`], using raw channel data and `'luni'`/`'lsct'` tagged blocks.
//! - Anything outside the supported subset returns [`PsdError::Unsupported`],
//!   never a panic.

mod common;
mod error;
mod read;
mod write;

#[cfg(test)]
mod tests;

pub use error::PsdError;
pub use read::read_psd;
pub use write::write_psd;
