//! Artistic filters (`m22-artistic-filters`): Cutout, Film Grain, Neon Glow and
//! Poster Edges. Behavioural models; Adobe's closed algorithms are approximated
//! and each shortcut is marked with a `ponytail:` note.

mod brush;
mod common;
mod effects;

pub use brush::{
    colored_pencil, dry_brush, fresco, paint_daubs, palette_knife, plastic_wrap, rough_pastels,
    smudge_stick, sponge, underpainting, watercolor,
};
pub use effects::{cutout, film_grain, neon_glow, poster_edges};

#[cfg(test)]
mod tests;
