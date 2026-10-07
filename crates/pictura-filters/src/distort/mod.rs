//! Distort family (`FILT-040`): the geometric warps Kooka still runs itself,
//! Shear and Ocean Ripple. The rest of the family is the photorust engine's
//! (`crate::photorust::distort`).

mod coord;
mod ripples;

pub use coord::shear;
pub use ripples::ocean_ripple;
