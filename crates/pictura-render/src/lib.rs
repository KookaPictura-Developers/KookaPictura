//! CPU compositing for Kooka Pictura.
//!
//! M2 scope: composite the document layer stack (opacity, raster masks, groups,
//! 27 blend modes) into an image. Spec: `docs/05-layers/blend-modes.md` and the
//! W3C Compositing and Blending Level 1 model. GPU acceleration is M2.5.
//!
//! Contract owned by task M2-A:
//!
//! ```ignore
//! /// 4-channel (R,G,B,A) planar, straight alpha, 8-bit, at document size.
//! pub fn composite_rgba(doc: &pictura_core::Document) -> pictura_core::PixelBuffer;
//! ```

use pictura_core::Document;
use pictura_core::PixelBuffer;

/// Composite the document's layer stack.
///
/// Returns a 4-channel (R,G,B,A) planar, straight-alpha, 8-bit buffer at
/// document resolution.
pub fn composite_rgba(doc: &Document) -> PixelBuffer {
    let _ = doc;
    unimplemented!("M2-A: implement CPU layer-stack compositing")
}
