//! Pixelate family (`FILT-084`): block/cell/pattern filters.
//!
//! Split into two modules so the six complex filters can be worked in parallel:
//! [`texture`] (Mosaic, Facet, Fragment, Mezzotint) and [`cells`] (Crystallize,
//! Pointillize, Color Halftone).

mod cells;
mod texture;

pub use cells::{color_halftone, crystallize, pointillize};
pub use texture::{facet, fragment, mezzotint, mosaic};
