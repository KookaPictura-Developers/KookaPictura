//! GPU vs CPU parity for the separable blend modes.
//!
//! The CPU compositor is the oracle. Each scene drives one blend mode with a
//! gradient backdrop and a partially transparent gradient source, and the GPU
//! output must match `composite_rgba` within ±1 LSB per channel.
//!
//! Every test skips with a printed note when no Vulkan adapter is usable, so a
//! GPU-less CI stays green. On the target machine (RTX 3090, Vulkan) the
//! parity path actually runs.
//!
//! M31 adds region compositing: a region-limited GPU composite must be
//! byte-identical to the same slice of the full `composite_active` result,
//! because it is the same per-pixel kernel over a region-sized canvas.

mod blend;
mod common;
mod fresh_white;
mod large;
mod layers;
mod region;
