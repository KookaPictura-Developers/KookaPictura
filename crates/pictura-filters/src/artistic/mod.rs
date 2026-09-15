//! Artistic family (`m22-artistic-filters`).
//!
//! Split so the shared quantizers/edges ([`reduce`]) and the seeded noise field
//! ([`noise`]) stay separate from the filter implementations.

pub mod noise;
pub mod reduce;
pub mod texture;

mod filters;

pub use filters::{
    colored_pencil, cutout, dry_brush, film_grain, fresco, neon_glow, paint_daubs, palette_knife,
    plastic_wrap, poster_edges, rough_pastels, smudge_stick, sponge, underpainting, watercolor,
};
