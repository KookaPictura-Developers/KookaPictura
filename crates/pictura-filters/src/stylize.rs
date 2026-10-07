//! Stylize family (`FILT-050`): Solarize, the one Stylize filter Kooka still
//! runs itself (its ImageMagick oracle is exact). The rest of the family is the
//! photorust engine's (`crate::photorust::stylize`).

use pictura_core::PixelBuffer;

use crate::{validate, FilterError};

/// CS6's four Diffuse modes, in the order its dialog lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiffuseMode {
    #[default]
    Normal,
    DarkenOnly,
    LightenOnly,
    Anisotropic,
}

pub fn solarize(buf: &mut PixelBuffer) -> Result<(), FilterError> {
    validate(buf)?;
    let n = buf.pixel_count();
    let planes = (buf.channels as usize).min(3);
    for c in 0..planes {
        let base = c * n;
        for v in &mut buf.data[base..base + n] {
            if *v >= 128 {
                *v = 255 - *v;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn planar(width: u32, height: u32, channels: u8, planes: &[Vec<u8>]) -> PixelBuffer {
        let mut data = Vec::new();
        for p in planes {
            data.extend_from_slice(p);
        }
        PixelBuffer {
            width,
            height,
            channels,
            data: data.into(),
        }
    }

    fn gray_row(values: &[u8]) -> PixelBuffer {
        planar(
            values.len() as u32,
            1,
            3,
            &[values.to_vec(), values.to_vec(), values.to_vec()],
        )
    }

    #[test]
    fn solarize_known_values() {
        // FILT-050 fixed 50% curve: 255 >= 128 -> 255 - 255 = 0.
        let values = [0u8, 100, 127, 128, 200, 254, 255];
        let expected = [0u8, 100, 127, 127, 55, 1, 0];
        let base = gray_row(&values);
        let mut out = base.clone();
        solarize(&mut out).unwrap();
        let n = values.len();
        for (i, &e) in expected.iter().enumerate() {
            assert_eq!(out.data[i], e, "solarize({})", values[i]);
            assert_eq!(out.data[n + i], e, "solarize({})", values[i]);
            assert_eq!(out.data[2 * n + i], e, "solarize({})", values[i]);
        }
    }

    #[test]
    fn solarize_is_idempotent_not_involutive() {
        let values: Vec<u8> = (0..=255).collect();
        let base = gray_row(&values);
        let mut once = base.clone();
        solarize(&mut once).unwrap();
        let mut twice = once.clone();
        solarize(&mut twice).unwrap();
        assert_ne!(once.data, base.data, "solarize must change the ramp");
        assert_eq!(twice.data, once.data, "solarize is idempotent");
        assert_eq!(once.data[200], 55);
        assert_eq!(twice.data[200], 55);
    }

    #[test]
    fn solarize_preserves_alpha() {
        let expected: Vec<u8> = (0..20u32).map(|i| (i * 11) as u8).collect();
        let n = expected.len();
        let r: Vec<u8> = (0..n).map(|i| (i * 13) as u8).collect();
        let g: Vec<u8> = (0..n).map(|i| (i * 17) as u8).collect();
        let b: Vec<u8> = (0..n).map(|i| (i * 19) as u8).collect();
        let mut s = planar(5, 4, 4, &[r, g, b, expected.clone()]);
        solarize(&mut s).unwrap();
        assert_eq!(s.data[3 * n..].to_vec(), expected);
    }
}
