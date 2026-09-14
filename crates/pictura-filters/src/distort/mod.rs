//! Distort family (`FILT-040`): geometric inverse-mapping warps.
//!
//! Split into two modules so the radial and undulating filters can be worked in
//! parallel: [`radial`] (Twirl, Pinch, Spherize) and [`undulate`] (Ripple, Wave).

mod coord;
mod radial;
mod ripples;
mod undulate;

pub use coord::{polar_coordinates, shear};
pub use radial::{pinch, spherize, twirl};
pub use ripples::{ocean_ripple, zigzag};
pub use undulate::{ripple, wave};
