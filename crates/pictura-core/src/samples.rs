//! Native-depth sample storage and the shared narrowing/widening rules
//! (`psd-bit-depth`). Split from `lib.rs` for the file-size cap.

use crate::BitDepth;

/// A planar buffer of document-depth samples: `u8`, `u16`, or `f32` per channel.
#[derive(Debug, Clone, PartialEq)]
pub enum Samples {
    U8(Vec<u8>),
    U16(Vec<u16>),
    F32(Vec<f32>),
}

impl Samples {
    /// The bit depth this store holds.
    pub fn depth(&self) -> BitDepth {
        match self {
            Samples::U8(_) => BitDepth::Eight,
            Samples::U16(_) => BitDepth::Sixteen,
            Samples::F32(_) => BitDepth::ThirtyTwo,
        }
    }

    /// Number of samples (not pixels).
    pub fn len(&self) -> usize {
        match self {
            Samples::U8(v) => v.len(),
            Samples::U16(v) => v.len(),
            Samples::F32(v) => v.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Decode a PSD native byte image into samples: big-endian `u16` at depth 16,
    /// IEEE-754 big-endian `f32` at depth 32; depth 1 and 8 bytes stay bytes.
    pub fn from_bytes(bytes: &[u8], depth: BitDepth) -> Samples {
        match depth {
            BitDepth::Sixteen => Samples::U16(
                bytes
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|b| u16::from_be_bytes(*b))
                    .collect(),
            ),
            BitDepth::ThirtyTwo => Samples::F32(
                bytes
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .map(|b| f32::from_be_bytes(*b))
                    .collect(),
            ),
            _ => Samples::U8(bytes.to_vec()),
        }
    }

    /// Encode samples back to the PSD native byte image; the inverse of
    /// [`Samples::from_bytes`].
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Samples::U8(v) => v.clone(),
            Samples::U16(v) => v.iter().flat_map(|s| s.to_be_bytes()).collect(),
            Samples::F32(v) => v.iter().flat_map(|s| s.to_be_bytes()).collect(),
        }
    }

    /// Narrow to 8-bit: depth 16 keeps the high byte, depth 32 is
    /// `clamp(trunc(v * 256), 0, 255)`, depth 1/8 is the bytes unchanged.
    pub fn narrow_to_u8(&self) -> Vec<u8> {
        match self {
            Samples::U8(v) => v.clone(),
            Samples::U16(v) => v.iter().map(|&s| (s >> 8) as u8).collect(),
            Samples::F32(v) => v
                .iter()
                .map(|&s| (s * 256.0).trunc().clamp(0.0, 255.0) as u8)
                .collect(),
        }
    }

    /// Widen an 8-bit plane to `depth`: `v * 257` at depth 16, `v / 255` scaled
    /// into `[0, 1]` at depth 32, the bytes unchanged at depth 1/8.
    pub fn widen_from_u8(bytes: &[u8], depth: BitDepth) -> Samples {
        match depth {
            BitDepth::Sixteen => Samples::U16(bytes.iter().map(|&v| v as u16 * 257).collect()),
            BitDepth::ThirtyTwo => Samples::F32(bytes.iter().map(|&v| v as f32 / 255.0).collect()),
            _ => Samples::U8(bytes.to_vec()),
        }
    }

    /// A cloned sub-range, or `None` when it is out of bounds. Views one channel
    /// plane of a flat multi-plane store.
    pub fn slice(&self, range: std::ops::Range<usize>) -> Option<Samples> {
        if range.start > range.end || range.end > self.len() {
            return None;
        }
        Some(match self {
            Samples::U8(v) => Samples::U8(v[range].to_vec()),
            Samples::U16(v) => Samples::U16(v[range].to_vec()),
            Samples::F32(v) => Samples::F32(v[range].to_vec()),
        })
    }

    /// The raw bytes when the store is 8-bit, including the packed depth-1
    /// Bitmap plane; `None` for a native `u16`/`f32` store.
    pub fn as_u8(&self) -> Option<&[u8]> {
        match self {
            Samples::U8(v) => Some(v),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn narrow_u16_matches_the_byte_rule() {
        let store = Samples::U16(vec![0, 255, 256, 32768, 65535]);
        assert_eq!(store.narrow_to_u8(), vec![0, 0, 1, 128, 255]);
    }

    #[test]
    fn narrow_f32_matches_the_clipped_rule() {
        let store = Samples::F32(vec![1.5, -0.5, 0.0, 0.5]);
        assert_eq!(store.narrow_to_u8(), vec![255, 0, 0, 128]);
    }

    #[test]
    fn widen_u8_to_u16_is_257_times() {
        assert_eq!(
            Samples::widen_from_u8(&[0, 1, 255], BitDepth::Sixteen),
            Samples::U16(vec![0, 257, 65535])
        );
    }

    #[test]
    fn widen_u8_to_f32_scales_to_unit_range() {
        assert_eq!(
            Samples::widen_from_u8(&[0, 1, 255], BitDepth::ThirtyTwo),
            Samples::F32(vec![0.0, 1.0 / 255.0, 1.0])
        );
    }

    #[test]
    fn len_depth_and_empty_follow_the_variant() {
        assert_eq!(Samples::U8(vec![0; 3]).depth(), BitDepth::Eight);
        assert_eq!(Samples::U16(vec![0; 3]).depth(), BitDepth::Sixteen);
        assert_eq!(Samples::F32(vec![0.0; 3]).depth(), BitDepth::ThirtyTwo);
        assert_eq!(Samples::U16(vec![0; 3]).len(), 3);
        assert!(Samples::U8(Vec::new()).is_empty());
        assert!(!Samples::F32(vec![0.0]).is_empty());
    }

    #[test]
    fn encode_inverts_decode_byte_exactly() {
        let u16_bytes: Vec<u8> = (0..40u16)
            .flat_map(|v| v.wrapping_mul(1031).to_be_bytes())
            .collect();
        assert_eq!(
            Samples::from_bytes(&u16_bytes, BitDepth::Sixteen).to_bytes(),
            u16_bytes
        );
        let f32_bytes: Vec<u8> = [0.0f32, 0.5, 1.0, -0.5, 1.5]
            .iter()
            .flat_map(|v| v.to_be_bytes())
            .collect();
        assert_eq!(
            Samples::from_bytes(&f32_bytes, BitDepth::ThirtyTwo).to_bytes(),
            f32_bytes
        );
    }

    #[test]
    fn slice_is_bounds_checked() {
        let store = Samples::U16(vec![1, 2, 3, 4]);
        assert_eq!(store.slice(1..3), Some(Samples::U16(vec![2, 3])));
        assert_eq!(store.slice(2..5), None);
        assert_eq!(Samples::U8(vec![1, 2]).as_u8(), Some(&[1u8, 2][..]));
        assert_eq!(Samples::F32(vec![0.0]).as_u8(), None);
    }
}
