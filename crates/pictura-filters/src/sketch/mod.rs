//! Sketch filter family stubs (`m25-*`).
//!
//! Split into [`relief`] (edge/emboss-style renders) and [`paper`]
//! (paper-and-ink simulations).

mod paper;
mod relief;

pub use paper::{note_paper, photocopy, plaster, reticulation, stamp, torn_edges, water_paper};
pub use relief::{
    bas_relief, chalk_charcoal, charcoal, chrome, conte_crayon, graphic_pen, halftone_pattern,
};
