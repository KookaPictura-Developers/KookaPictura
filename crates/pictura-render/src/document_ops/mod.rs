//! Document-scope wrappers around the M10 `pictura-ops` image operations
//! (`IMG-001`, `IMG-002`, `IMG-003`). See `docs/dev/m12-document-ops.md`.

mod canvas;
mod orient;
mod resize;

pub use canvas::resize_canvas_document;
pub use orient::{flip_document, rotate_document};
pub use resize::resize_document;
