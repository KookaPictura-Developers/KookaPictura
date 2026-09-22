//! ICC color management for Kooka Pictura.
//!
//! M3 scope: working-space profiles (sRGB / Adobe RGB / ProPhoto), profile
//! assignment vs conversion, rendering intents, and black point compensation.
//! Spec: `docs/01-architecture/color-management.md`.
//!
//! The engine is Little CMS 2 through the `lcms2` crate. Built-in wide-gamut
//! profiles are synthesised from their primaries at runtime rather than bundled,
//! so no Adobe profile files are distributed.

use lcms2::{
    CIExyY, CIExyYTRIPLE, Flags, InfoType, Intent as LcmsIntent, Locale, PixelFormat,
    Profile as LcmsProfile, ToneCurve, Transform,
};

/// Rendering intent (ICC).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent {
    Perceptual,
    RelativeColorimetric,
    Saturation,
    AbsoluteColorimetric,
}

impl Intent {
    fn to_lcms(self) -> LcmsIntent {
        match self {
            Intent::Perceptual => LcmsIntent::Perceptual,
            Intent::RelativeColorimetric => LcmsIntent::RelativeColorimetric,
            Intent::Saturation => LcmsIntent::Saturation,
            Intent::AbsoluteColorimetric => LcmsIntent::AbsoluteColorimetric,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ColorError {
    #[error("invalid ICC profile")]
    InvalidProfile,
    #[error("unsupported: {0}")]
    Unsupported(String),
}

/// An ICC profile handle.
#[derive(Debug)]
pub struct Profile(LcmsProfile);

impl Profile {
    /// sRGB IEC61966-2.1 (the default working space).
    pub fn srgb() -> Profile {
        Profile(LcmsProfile::new_srgb())
    }

    /// Adobe RGB (1998): D65, Adobe primaries, gamma 563/256.
    pub fn adobe_rgb() -> Profile {
        rgb_profile(
            CIExyY {
                x: 0.3127,
                y: 0.3290,
                Y: 1.0,
            },
            CIExyYTRIPLE {
                Red: CIExyY {
                    x: 0.6400,
                    y: 0.3300,
                    Y: 1.0,
                },
                Green: CIExyY {
                    x: 0.2100,
                    y: 0.7100,
                    Y: 1.0,
                },
                Blue: CIExyY {
                    x: 0.1500,
                    y: 0.0600,
                    Y: 1.0,
                },
            },
            ToneCurve::new(563.0 / 256.0),
        )
    }

    /// ProPhoto RGB (ROMM RGB): D50, ROMM primaries, ROMM tone curve.
    pub fn pro_photo() -> Profile {
        rgb_profile(
            CIExyY {
                x: 0.34567,
                y: 0.35850,
                Y: 1.0,
            },
            CIExyYTRIPLE {
                Red: CIExyY {
                    x: 0.7347,
                    y: 0.2653,
                    Y: 1.0,
                },
                Green: CIExyY {
                    x: 0.1596,
                    y: 0.8404,
                    Y: 1.0,
                },
                Blue: CIExyY {
                    x: 0.0366,
                    y: 0.0001,
                    Y: 1.0,
                },
            },
            romm_tone_curve(),
        )
    }

    /// Parse an ICC profile from raw bytes. Malformed input yields an error.
    pub fn from_icc(bytes: &[u8]) -> Result<Profile, ColorError> {
        LcmsProfile::new_icc(bytes)
            .map(Profile)
            .map_err(|_| ColorError::InvalidProfile)
    }

    /// Serialize the profile back to ICC bytes.
    pub fn to_icc(&self) -> Vec<u8> {
        // A Profile only exists after lcms2 accepted it, so serialization
        // succeeds in practice; the empty fallback keeps this panic-free.
        self.0.icc().unwrap_or_default()
    }

    /// The profile's `Description` tag (`cmsInfoDescription`), when present.
    pub fn description(&self) -> Option<String> {
        self.0.info(InfoType::Description, Locale::default())
    }

    /// Whether this profile is the sRGB working space, by its description.
    ///
    /// ponytail: lcms2 has no profile-equality check, so this matches "srgb"
    /// case-insensitively in the `Description` tag; a differently-named sRGB
    /// profile is treated as non-sRGB (converted, within a rounding LSB).
    pub fn is_srgb(&self) -> bool {
        self.description()
            .is_some_and(|d| d.to_ascii_lowercase().contains("srgb"))
    }
}

fn rgb_profile(white: CIExyY, primaries: CIExyYTRIPLE, curve: ToneCurve) -> Profile {
    let curves = [&curve, &curve, &curve];
    Profile(
        LcmsProfile::new_rgb(&white, &primaries, &curves)
            .expect("built-in RGB primaries are valid"),
    )
}

/// ROMM RGB's tone curve: linear toe below 1/32, then a gamma-1.8 power.
/// lcms2 has no parametric type for it, so tabulate the encoded→linear mapping.
fn romm_tone_curve() -> ToneCurve {
    const N: usize = 4096;
    let mut table = [0u16; N];
    for (i, slot) in table.iter_mut().enumerate() {
        let encoded = i as f64 / (N - 1) as f64;
        let linear = if encoded < 0.03125 {
            encoded / 16.0
        } else {
            encoded.powf(1.8)
        };
        *slot = (linear * 65535.0).round() as u16;
    }
    ToneCurve::new_tabulated(&table)
}

/// Convert interleaved 8- or 16-bit samples between two profiles.
///
/// `channels` is 1 (gray), 3 (RGB) or 4 (RGBA); RGBA alpha is copied through
/// unchanged. Output keeps the input's channel count and bit depth. 16-bit
/// samples are native-endian.
#[allow(clippy::too_many_arguments)]
pub fn convert(
    src: &Profile,
    dst: &Profile,
    data: &[u8],
    width: u32,
    height: u32,
    channels: u8,
    bits: u8,
    intent: Intent,
    black_point_compensation: bool,
) -> Result<Vec<u8>, ColorError> {
    let (in_format, out_format) = formats(channels, bits)?;
    let bytes_per_pixel = channels as usize * (bits / 8) as usize;
    let expected = (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(bytes_per_pixel))
        .ok_or_else(|| ColorError::Unsupported("image dimensions overflow".into()))?;
    if data.len() != expected {
        return Err(ColorError::Unsupported(format!(
            "expected {expected} bytes for {width}x{height}x{channels} @ {bits}-bit, got {}",
            data.len()
        )));
    }
    if expected == 0 {
        return Ok(Vec::new());
    }

    let mut flags = Flags::default();
    if black_point_compensation {
        flags = flags | Flags::BLACKPOINT_COMPENSATION;
    }
    if channels == 4 {
        flags = flags | Flags::COPY_ALPHA;
    }

    // `Transform<u8, u8>` lets lcms2 read/write the byte buffer directly using
    // the declared pixel format; no endianness or alignment juggling needed.
    let transform: Transform<u8, u8> = Transform::new_flags(
        &src.0,
        in_format,
        &dst.0,
        out_format,
        intent.to_lcms(),
        flags,
    )
    .map_err(|e| ColorError::Unsupported(e.to_string()))?;

    let mut out = vec![0u8; expected];
    transform.transform_pixels(data, &mut out);
    Ok(out)
}

/// Retag: returns the same pixels with a new profile attached (no transform).
pub fn assign(data: &[u8], profile: Profile) -> (Vec<u8>, Profile) {
    (data.to_vec(), profile)
}

fn formats(channels: u8, bits: u8) -> Result<(PixelFormat, PixelFormat), ColorError> {
    let format = match (channels, bits) {
        (1, 8) => PixelFormat::GRAY_8,
        (1, 16) => PixelFormat::GRAY_16,
        (3, 8) => PixelFormat::RGB_8,
        (3, 16) => PixelFormat::RGB_16,
        (4, 8) => PixelFormat::RGBA_8,
        (4, 16) => PixelFormat::RGBA_16,
        (c, b) => return Err(ColorError::Unsupported(format!("{c} channels at {b} bits"))),
    };
    Ok((format, format))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn description_and_srgb_detection() {
        assert!(Profile::srgb().is_srgb(), "the working space is sRGB");
        let adobe = Profile::adobe_rgb();
        assert!(!adobe.is_srgb(), "Adobe RGB is not sRGB");
        assert!(
            adobe.description().is_some(),
            "a synthesized profile has a description"
        );
        let reloaded = Profile::from_icc(&adobe.to_icc()).expect("round-trips");
        assert!(!reloaded.is_srgb(), "detection survives an ICC round trip");
    }

    fn rgb8(pixels: &[[u8; 3]]) -> Vec<u8> {
        pixels.iter().flatten().copied().collect()
    }

    fn rgb16(pixels: &[[u16; 3]]) -> Vec<u8> {
        let mut out = Vec::new();
        for pixel in pixels {
            for channel in pixel {
                out.extend_from_slice(&channel.to_ne_bytes());
            }
        }
        out
    }

    fn rgb16_from_bytes(bytes: &[u8]) -> Vec<u16> {
        bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| u16::from_ne_bytes(*c))
            .collect()
    }

    #[test]
    fn identity_rgb8() {
        let srgb = Profile::srgb();
        let img = rgb8(&[[0, 0, 0], [255, 255, 255], [10, 128, 240], [200, 50, 100]]);
        let out = convert(
            &srgb,
            &srgb,
            &img,
            4,
            1,
            3,
            8,
            Intent::RelativeColorimetric,
            false,
        )
        .unwrap();
        for (a, b) in img.iter().zip(&out) {
            assert!((*a as i16 - *b as i16).abs() <= 1, "{a} vs {b}");
        }
    }

    #[test]
    fn identity_rgb16() {
        let srgb = Profile::srgb();
        let pixels = [
            [0u16, 0, 0],
            [65535, 65535, 65535],
            [1000, 30000, 60000],
            [40000, 20000, 500],
        ];
        let img = rgb16(&pixels);
        let out = convert(
            &srgb,
            &srgb,
            &img,
            4,
            1,
            3,
            16,
            Intent::RelativeColorimetric,
            false,
        )
        .unwrap();
        let got = rgb16_from_bytes(&out);
        let want: Vec<u16> = pixels.iter().flatten().copied().collect();
        for (a, b) in want.iter().zip(&got) {
            assert!((*a as i32 - *b as i32).abs() <= 1, "{a} vs {b}");
        }
    }

    #[test]
    fn round_trip_srgb_adobe_rgb() {
        let srgb = Profile::srgb();
        let adobe = Profile::adobe_rgb();
        let img = rgb8(&[
            [0, 0, 0],
            [255, 255, 255],
            [128, 128, 128],
            [255, 0, 0],
            [0, 255, 0],
            [0, 0, 255],
            [64, 160, 220],
        ]);
        let mid = convert(
            &srgb,
            &adobe,
            &img,
            7,
            1,
            3,
            8,
            Intent::RelativeColorimetric,
            false,
        )
        .unwrap();
        let back = convert(
            &adobe,
            &srgb,
            &mid,
            7,
            1,
            3,
            8,
            Intent::RelativeColorimetric,
            false,
        )
        .unwrap();
        for (a, b) in img.iter().zip(&back) {
            assert!((*a as i16 - *b as i16).abs() <= 3, "{a} vs {b}");
        }
    }

    #[test]
    fn every_intent_produces_output() {
        let srgb = Profile::srgb();
        let adobe = Profile::adobe_rgb();
        let mut img = Vec::new();
        for i in 0..64u8 {
            img.extend_from_slice(&[i * 4, 255 - i * 4, i * 3]);
        }
        for intent in [
            Intent::Perceptual,
            Intent::RelativeColorimetric,
            Intent::Saturation,
            Intent::AbsoluteColorimetric,
        ] {
            let out = convert(&srgb, &adobe, &img, 64, 1, 3, 8, intent, true).unwrap();
            assert_eq!(out.len(), img.len());
            let (triples, _) = out.as_chunks::<3>();
            let distinct: HashSet<&[u8; 3]> = triples.iter().collect();
            assert!(distinct.len() > 8, "{intent:?} produced degenerate output");
        }
    }

    #[test]
    fn gray_identity() {
        // No gray built-in, so build one with lcms2 and load it through from_icc.
        let gray = {
            let white = lcms2::white_point_from_temp(6504.0).unwrap();
            let curve = ToneCurve::new(2.2);
            let profile = LcmsProfile::new_gray(&white, &curve).unwrap();
            Profile::from_icc(&profile.icc().unwrap()).unwrap()
        };
        let img = [0u8, 1, 64, 128, 200, 254, 255];
        let out = convert(
            &gray,
            &gray,
            &img,
            7,
            1,
            1,
            8,
            Intent::RelativeColorimetric,
            false,
        )
        .unwrap();
        for (a, b) in img.iter().zip(&out) {
            assert!((*a as i16 - *b as i16).abs() <= 1, "{a} vs {b}");
        }
    }

    #[test]
    fn rgba_alpha_is_preserved() {
        let srgb = Profile::srgb();
        let adobe = Profile::adobe_rgb();
        let pixels: [[u8; 4]; 4] = [
            [0, 0, 0, 0],
            [255, 255, 255, 1],
            [10, 128, 240, 127],
            [200, 50, 100, 255],
        ];
        let img: Vec<u8> = pixels.iter().flatten().copied().collect();
        let out = convert(
            &srgb,
            &adobe,
            &img,
            4,
            1,
            4,
            8,
            Intent::RelativeColorimetric,
            false,
        )
        .unwrap();
        for (i, pixel) in pixels.iter().enumerate() {
            assert_eq!(out[i * 4 + 3], pixel[3], "alpha at pixel {i}");
        }
    }

    #[test]
    fn malformed_icc_is_error_not_panic() {
        assert!(Profile::from_icc(&[]).is_err());
        assert!(Profile::from_icc(&[0u8; 128]).is_err());
        assert!(Profile::from_icc(b"definitely not an ICC profile").is_err());
    }

    #[test]
    fn icc_round_trips() {
        for profile in [Profile::srgb(), Profile::adobe_rgb(), Profile::pro_photo()] {
            let bytes = profile.to_icc();
            assert!(!bytes.is_empty());
            let reloaded = Profile::from_icc(&bytes).unwrap();
            assert_eq!(reloaded.0.color_space(), profile.0.color_space());
        }
    }

    #[test]
    fn assign_retags_without_changing_pixels() {
        let img = rgb8(&[[1, 2, 3], [4, 5, 6]]);
        let (out, profile) = assign(&img, Profile::adobe_rgb());
        assert_eq!(out, img);
        assert_eq!(profile.0.color_space(), lcms2::ColorSpaceSignature::RgbData);
    }
}
