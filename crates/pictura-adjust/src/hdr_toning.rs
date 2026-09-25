//! HDR toning (32-bit → display). `docs/04-image-ops/adjustments/hdr-toning.md`.
//!
//! Only CS6's **Exposure & Gamma** method is implemented: it is the one method
//! whose formula the spec fixes (`clip(exposure·2^EV)^(1/gamma)`, Exposure 0 /
//! Gamma 1.0 = identity). Local Adaptation, Equalize Histogram, and Highlight
//! Compression have closed kernels and are deliberately absent.
//!
//! The operator runs in linear light on `f32` samples (a 32-bit PSD composite),
//! so values above `1.0` survive instead of being clipped by the 8-bit path.
//!
//! ponytail: no HDR Toning dialog and no 32→8/16 conversion command yet; this is
//! the operator the dialog will call.

use crate::AdjustError;

/// CS6 HDR Conversion "Exposure and Gamma" parameters. `Default` is the identity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExposureGamma {
    /// Exposure in stops (EV); `0.0` is neutral.
    pub exposure_ev: f64,
    /// Contrast gamma; `1.0` is neutral. Must be finite and `> 0`.
    pub gamma: f64,
}

impl Default for ExposureGamma {
    fn default() -> Self {
        Self {
            exposure_ev: 0.0,
            gamma: 1.0,
        }
    }
}

/// Apply Exposure & Gamma to linear-light `f32` samples.
///
/// `gain = 2^exposure_ev`; each output sample is
/// `sign(x·gain)·|x·gain|^(1/gamma)`. This is the documented
/// `(x·2^EV)^(1/gamma)` on the non-negative domain, keeps values above `1.0`
/// unclamped, maps `0.0` to `0.0`, and preserves the sign of a negative sample.
///
/// Returns [`AdjustError::InvalidParams`] when `exposure_ev` is not finite, or
/// `gamma` is not finite / not strictly positive.
pub fn exposure_gamma(samples: &[f32], params: ExposureGamma) -> Result<Vec<f32>, AdjustError> {
    if !params.exposure_ev.is_finite() {
        return Err(AdjustError::InvalidParams("exposure must be finite".into()));
    }
    if !params.gamma.is_finite() || params.gamma <= 0.0 {
        return Err(AdjustError::InvalidParams(
            "gamma must be finite and > 0".into(),
        ));
    }
    let gain = 2f64.powf(params.exposure_ev);
    if !gain.is_finite() {
        return Err(AdjustError::InvalidParams(
            "exposure overflows the gain".into(),
        ));
    }
    let inv_gamma = 1.0 / params.gamma;
    Ok(samples
        .iter()
        .map(|&x| {
            let g = f64::from(x) * gain;
            (g.signum() * g.abs().powf(inv_gamma)) as f32
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity() -> ExposureGamma {
        ExposureGamma {
            exposure_ev: 0.0,
            gamma: 1.0,
        }
    }

    #[test]
    fn identity_is_neutral() {
        let src = [0.0f32, 0.25, 0.5, 1.0, 2.0];
        let out = exposure_gamma(&src, identity()).unwrap();
        for (a, b) in src.iter().zip(&out) {
            assert!((a - b).abs() < 1e-6, "{a} -> {b}");
        }
    }

    #[test]
    fn documented_formula_above_one_is_unclamped() {
        // (2.0 * 2^1)^(1/2) = 2.0
        let out = exposure_gamma(
            &[2.0],
            ExposureGamma {
                exposure_ev: 1.0,
                gamma: 2.0,
            },
        )
        .unwrap();
        assert!((out[0] - 2.0).abs() < 1e-5, "got {}", out[0]);
    }

    #[test]
    fn zero_and_negative_are_defined() {
        let out = exposure_gamma(
            &[0.0, -0.5, -2.0],
            ExposureGamma {
                exposure_ev: 0.0,
                gamma: 2.0,
            },
        )
        .unwrap();
        assert_eq!(out[0], 0.0);
        assert!(out[1] < 0.0 && out[1].is_finite(), "{:?}", out);
        assert!(out[2] < 0.0 && out[2].is_finite(), "{:?}", out);
    }

    #[test]
    fn bad_parameters_are_rejected() {
        for gamma in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(matches!(
                exposure_gamma(
                    &[1.0],
                    ExposureGamma {
                        exposure_ev: 0.0,
                        gamma
                    }
                ),
                Err(AdjustError::InvalidParams(_))
            ));
        }
        for exposure_ev in [f64::NAN, f64::INFINITY] {
            assert!(matches!(
                exposure_gamma(
                    &[1.0],
                    ExposureGamma {
                        exposure_ev,
                        gamma: 1.0
                    }
                ),
                Err(AdjustError::InvalidParams(_))
            ));
        }
    }
}
