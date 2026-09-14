//! Image codecs. M0 scope: minimal PSD/PSB read + write of a **single composite
//! image** (no layers). The full format matrix is specified in
//! `docs/01-architecture/file-formats.md`.
//!
//! ## M0 contract
//!
//! - Read: file header, color mode data (skip), image resources (skip), skip the
//!   layer/mask section if present, then the image data section. Supported:
//!   8-bit, RGB or Grayscale, compression 0 (raw) or 1 (RLE/PackBits).
//! - Write: emit a valid PSD with an empty layer/mask section and raw image
//!   data, such that `read_psd(&write_psd(doc)?)` round-trips.
//! - Anything outside the supported subset returns [`PsdError::Unsupported`],
//!   never a panic.

use pictura_core::{BitDepth, ColorMode, Document, PixelBuffer};

#[derive(Debug, thiserror::Error)]
pub enum PsdError {
    #[error("not a PSD/PSB: bad signature {0:#06x}")]
    BadSignature(u32),
    #[error("unsupported: {0}")]
    Unsupported(String),
    #[error("truncated file")]
    Truncated,
    #[error("invalid data: {0}")]
    Invalid(String),
}

const SIGNATURE: u32 = 0x3842_5053; // "8BPS"
const VERSION_PSD: u16 = 1;
const VERSION_PSB: u16 = 2;

const MODE_GRAYSCALE: u16 = 1;
const MODE_RGB: u16 = 3;

const MAX_CHANNELS: u16 = 56;
const MAX_DIM_PSD: u32 = 30_000;
const MAX_DIM_PSB: u32 = 300_000;

/// Cursor over the file bytes. Every read is bounds-checked, so malformed input
/// yields [`PsdError::Truncated`] instead of an index panic.
struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], PsdError> {
        if self.remaining() < n {
            return Err(PsdError::Truncated);
        }
        let out = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(out)
    }

    fn u16(&mut self) -> Result<u16, PsdError> {
        let s = self.take(2)?;
        Ok(u16::from_be_bytes(
            s.try_into().map_err(|_| PsdError::Truncated)?,
        ))
    }

    fn u32(&mut self) -> Result<u32, PsdError> {
        let s = self.take(4)?;
        Ok(u32::from_be_bytes(
            s.try_into().map_err(|_| PsdError::Truncated)?,
        ))
    }

    fn u64(&mut self) -> Result<u64, PsdError> {
        let s = self.take(8)?;
        Ok(u64::from_be_bytes(
            s.try_into().map_err(|_| PsdError::Truncated)?,
        ))
    }

    fn skip(&mut self, n: usize) -> Result<(), PsdError> {
        self.take(n).map(|_| ())
    }
}

/// Parse a PSD (or PSB) file into a [`Document`] holding the composite image.
pub fn read_psd(bytes: &[u8]) -> Result<Document, PsdError> {
    let mut r = Reader::new(bytes);
    let sig = r.u32()?;
    if sig != SIGNATURE {
        return Err(PsdError::BadSignature(sig));
    }

    let version = r.u16()?;
    let is_psb = match version {
        VERSION_PSD => false,
        VERSION_PSB => true,
        v => return Err(PsdError::Unsupported(format!("PSD version {v}"))),
    };

    r.skip(6)?; // reserved
    let channels = r.u16()?;
    let height = r.u32()?;
    let width = r.u32()?;
    let depth = r.u16()?;
    let mode_code = r.u16()?;

    let max_dim = if is_psb { MAX_DIM_PSB } else { MAX_DIM_PSD };
    if channels == 0 || channels > MAX_CHANNELS {
        return Err(PsdError::Invalid(format!("channel count {channels}")));
    }
    if width == 0 || height == 0 {
        return Err(PsdError::Invalid("zero image dimension".into()));
    }
    if width > max_dim || height > max_dim {
        return Err(PsdError::Unsupported(format!(
            "dimension {width}x{height} exceeds {max_dim}"
        )));
    }
    if depth != 8 {
        return Err(PsdError::Unsupported(format!("bit depth {depth}")));
    }
    let mode = match mode_code {
        MODE_GRAYSCALE => ColorMode::Grayscale,
        MODE_RGB => ColorMode::Rgb,
        c => return Err(PsdError::Unsupported(format!("color mode {c}"))),
    };

    // Color mode data section: 4-byte length + opaque bytes (skipped).
    let color_mode_len = r.u32()? as usize;
    r.skip(color_mode_len)?;
    // Image resources section: 4-byte length + opaque bytes (skipped).
    let resources_len = r.u32()? as usize;
    r.skip(resources_len)?;
    // Layer and mask information section: 4-byte length (8 in PSB) + bytes.
    let layer_mask_len = if is_psb {
        r.u64()? as usize
    } else {
        r.u32()? as usize
    };
    r.skip(layer_mask_len)?;

    // Image data section: 2-byte compression method, then channel data.
    let compression = r.u16()?;
    let channels = channels as usize;
    let width = width as usize;
    let height = height as usize;
    let data = match compression {
        0 => r.take(planar_len(channels, width, height)?)?.to_vec(),
        1 => read_rle(&mut r, channels, width, height, is_psb)?,
        c => return Err(PsdError::Unsupported(format!("compression {c}"))),
    };

    Ok(Document {
        width: width as u32,
        height: height as u32,
        mode,
        depth: BitDepth::Eight,
        composite: PixelBuffer {
            width: width as u32,
            height: height as u32,
            channels: channels as u8,
            data,
        },
    })
}

fn planar_len(channels: usize, width: usize, height: usize) -> Result<usize, PsdError> {
    channels
        .checked_mul(width)
        .and_then(|n| n.checked_mul(height))
        .ok_or_else(|| PsdError::Invalid("image dimensions overflow".into()))
}

fn read_rle(
    r: &mut Reader,
    channels: usize,
    width: usize,
    height: usize,
    is_psb: bool,
) -> Result<Vec<u8>, PsdError> {
    let rows = channels
        .checked_mul(height)
        .ok_or_else(|| PsdError::Invalid("image dimensions overflow".into()))?;
    // Scanline byte-count table: 2-byte entries in PSD, 4-byte in PSB.
    let mut counts = Vec::with_capacity(rows);
    for _ in 0..rows {
        counts.push(if is_psb {
            r.u32()? as usize
        } else {
            r.u16()? as usize
        });
    }

    let mut out = vec![0u8; planar_len(channels, width, height)?];
    let plane = width * height;
    for (i, &count) in counts.iter().enumerate() {
        let packed = r.take(count)?;
        let channel = i / height;
        let row = i % height;
        let start = channel * plane + row * width;
        decode_packbits(packed, &mut out[start..start + width])?;
    }
    Ok(out)
}

/// Decode one PackBits scanline into `dst` (exactly `dst.len()` bytes).
fn decode_packbits(src: &[u8], dst: &mut [u8]) -> Result<(), PsdError> {
    let mut si = 0usize;
    let mut di = 0usize;
    while di < dst.len() {
        let control = *src
            .get(si)
            .ok_or_else(|| PsdError::Invalid("RLE underrun".into()))? as i8;
        si += 1;
        if control >= 0 {
            let count = control as usize + 1;
            if si + count > src.len() || di + count > dst.len() {
                return Err(PsdError::Invalid("RLE literal overrun".into()));
            }
            dst[di..di + count].copy_from_slice(&src[si..si + count]);
            si += count;
            di += count;
        } else if control != -128 {
            let count = (1 - control as i32) as usize; // -1 => 2, -127 => 128
            let value = *src
                .get(si)
                .ok_or_else(|| PsdError::Invalid("RLE underrun".into()))?;
            si += 1;
            if di + count > dst.len() {
                return Err(PsdError::Invalid("RLE repeat overrun".into()));
            }
            dst[di..di + count].fill(value);
            di += count;
        }
        // control == -128 is a no-op.
    }
    Ok(())
}

/// Serialize a [`Document`]'s composite image into a valid PSD file.
pub fn write_psd(doc: &Document) -> Result<Vec<u8>, PsdError> {
    if doc.depth != BitDepth::Eight {
        return Err(PsdError::Unsupported("write supports 8-bit only".into()));
    }
    let mode_code = match doc.mode {
        ColorMode::Grayscale => MODE_GRAYSCALE,
        ColorMode::Rgb => MODE_RGB,
        m => return Err(PsdError::Unsupported(format!("write color mode {m:?}"))),
    };
    let channels = doc.composite.channels;
    if channels == 0 || channels > MAX_CHANNELS as u8 {
        return Err(PsdError::Invalid(format!("channel count {channels}")));
    }
    if doc.width == 0 || doc.height == 0 || doc.width > MAX_DIM_PSD || doc.height > MAX_DIM_PSD {
        return Err(PsdError::Unsupported(format!(
            "dimension {}x{}",
            doc.width, doc.height
        )));
    }
    if doc.composite.width != doc.width || doc.composite.height != doc.height {
        return Err(PsdError::Invalid(
            "composite size does not match document".into(),
        ));
    }
    if doc.composite.data.len()
        != planar_len(channels as usize, doc.width as usize, doc.height as usize)?
    {
        return Err(PsdError::Invalid("composite data length mismatch".into()));
    }

    let mut out = Vec::with_capacity(26 + 12 + 2 + doc.composite.data.len());
    out.extend_from_slice(&SIGNATURE.to_be_bytes());
    out.extend_from_slice(&VERSION_PSD.to_be_bytes());
    out.extend_from_slice(&[0u8; 6]); // reserved
    out.extend_from_slice(&(channels as u16).to_be_bytes());
    out.extend_from_slice(&doc.height.to_be_bytes());
    out.extend_from_slice(&doc.width.to_be_bytes());
    out.extend_from_slice(&8u16.to_be_bytes()); // depth
    out.extend_from_slice(&mode_code.to_be_bytes());
    out.extend_from_slice(&0u32.to_be_bytes()); // empty color mode data
    out.extend_from_slice(&0u32.to_be_bytes()); // empty image resources
    out.extend_from_slice(&0u32.to_be_bytes()); // zero-length layer/mask section
    out.extend_from_slice(&0u16.to_be_bytes()); // raw compression
    out.extend_from_slice(&doc.composite.data);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(version: u16, channels: u16, width: u32, height: u32, mode: u16) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(b"8BPS");
        v.extend_from_slice(&version.to_be_bytes());
        v.extend_from_slice(&[0u8; 6]);
        v.extend_from_slice(&channels.to_be_bytes());
        v.extend_from_slice(&height.to_be_bytes());
        v.extend_from_slice(&width.to_be_bytes());
        v.extend_from_slice(&8u16.to_be_bytes());
        v.extend_from_slice(&mode.to_be_bytes());
        v
    }

    fn psd_sections() -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(&0u32.to_be_bytes());
        v.extend_from_slice(&0u32.to_be_bytes());
        v.extend_from_slice(&0u32.to_be_bytes());
        v
    }

    #[test]
    fn parse_hand_constructed_raw_rgb() {
        let mut p = header(1, 3, 2, 2, 3);
        p.extend_from_slice(&psd_sections());
        p.extend_from_slice(&0u16.to_be_bytes());
        p.extend_from_slice(&[1, 2, 3, 4]); // R plane
        p.extend_from_slice(&[5, 6, 7, 8]); // G plane
        p.extend_from_slice(&[9, 10, 11, 12]); // B plane

        let doc = read_psd(&p).unwrap();
        assert_eq!(doc.width, 2);
        assert_eq!(doc.height, 2);
        assert_eq!(doc.mode, ColorMode::Rgb);
        assert_eq!(doc.depth, BitDepth::Eight);
        assert_eq!(doc.composite.channels, 3);
        assert_eq!(
            doc.composite.data,
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]
        );
    }

    #[test]
    fn round_trip_rgb_and_grayscale() {
        for (mode, channels) in [(ColorMode::Rgb, 3u8), (ColorMode::Grayscale, 1u8)] {
            let mut doc = Document::new(3, 2, mode, BitDepth::Eight);
            for (i, b) in doc.composite.data.iter_mut().enumerate() {
                *b = (i * 7 + 3) as u8;
            }
            assert_eq!(doc.composite.channels, channels);
            let bytes = write_psd(&doc).unwrap();
            assert_eq!(read_psd(&bytes).unwrap(), doc);
        }
    }

    #[test]
    fn parse_rle_psd() {
        let mut p = header(1, 1, 4, 2, 1);
        p.extend_from_slice(&psd_sections());
        p.extend_from_slice(&1u16.to_be_bytes()); // RLE
        p.extend_from_slice(&5u16.to_be_bytes()); // row 0 packed length
        p.extend_from_slice(&2u16.to_be_bytes()); // row 1 packed length
        p.push(3);
        p.extend_from_slice(&[0, 1, 2, 3]); // literal run of 4
        p.push(253);
        p.push(5); // repeat 5 four times

        let doc = read_psd(&p).unwrap();
        assert_eq!(doc.composite.data, vec![0, 1, 2, 3, 5, 5, 5, 5]);
    }

    #[test]
    fn parse_rle_psb_uses_four_byte_counts() {
        let mut p = header(2, 1, 4, 1, 1);
        p.extend_from_slice(&0u32.to_be_bytes()); // color mode
        p.extend_from_slice(&0u32.to_be_bytes()); // image resources
        p.extend_from_slice(&0u64.to_be_bytes()); // PSB layer/mask is 8 bytes
        p.extend_from_slice(&1u16.to_be_bytes()); // RLE
        p.extend_from_slice(&5u32.to_be_bytes()); // 4-byte scanline count
        p.push(3);
        p.extend_from_slice(&[9, 8, 7, 6]);

        let doc = read_psd(&p).unwrap();
        assert_eq!(doc.composite.data, vec![9, 8, 7, 6]);
    }

    #[test]
    fn bad_signature_is_error_not_panic() {
        let mut p = header(1, 3, 1, 1, 3);
        p[0] = b'X';
        assert!(matches!(read_psd(&p), Err(PsdError::BadSignature(_))));
    }

    #[test]
    fn truncated_is_error_not_panic() {
        let mut doc = Document::new(2, 2, ColorMode::Rgb, BitDepth::Eight);
        doc.composite.data = (0..12).collect();
        let bytes = write_psd(&doc).unwrap();
        assert!(matches!(
            read_psd(&bytes[..bytes.len() - 1]),
            Err(PsdError::Truncated)
        ));
        assert!(matches!(read_psd(b"8BPS"), Err(PsdError::Truncated)));
    }

    #[test]
    fn lcg_round_trip_property() {
        let mut state: u64 = 0x1234_5678_9abc_def0;
        let mut next = move || {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (state >> 33) as u32
        };

        for _ in 0..40 {
            let mode = if next() % 2 == 0 {
                ColorMode::Rgb
            } else {
                ColorMode::Grayscale
            };
            let channels = if mode == ColorMode::Rgb { 3 } else { 1 };
            let width = 1 + next() % 17;
            let height = 1 + next() % 17;

            let mut doc = Document::new(width, height, mode, BitDepth::Eight);
            for b in doc.composite.data.iter_mut() {
                *b = next() as u8;
            }
            let bytes = write_psd(&doc).unwrap();
            let back = read_psd(&bytes).unwrap();
            assert_eq!(back, doc, "mismatch for {width}x{height} ch={channels}");
        }
    }
}
