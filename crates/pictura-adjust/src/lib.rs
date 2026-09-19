//! Adjustment operations for Kooka Pictura.
//!
//! M4 scope: the destructive `Image > Adjustments` math as pure functions over a
//! planar 8-bit [`PixelBuffer`] (channels 3 or 4; alpha is never modified).
//! Specs live in `docs/04-image-ops/adjustments/`. Adjustment *layers* (model,
//! PSD serialization, compositor integration) are a later task.
//!
//! Adobe's closed kernels are approximated where the specs say so; each
//! approximation is marked inline. Everything is deterministic and returns
//! [`AdjustError`] instead of panicking on bad input.

mod apply;
mod auto;
mod color;
mod common;
mod tonal;
mod types;

#[cfg(test)]
mod tests;

pub use apply::apply;
pub use types::{
    AdjustError, Adjustment, AutoKind, BlackWhiteParams, BrightnessContrastParams,
    ChannelMixerParams, ColorBalanceParams, CurvesParams, ExposureParams, GradientFillParams,
    GradientKind, GradientMapParams, GradientStop, HueSaturationParams, LevelsParams,
    PatternFillParams, PhotoFilterParams, VibranceParams,
};
