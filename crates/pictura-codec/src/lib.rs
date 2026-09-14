//! Image codecs. M0 scope: minimal PSD/PSB read + write of a **single composite
//! image**. M1-B adds the **Layer and Mask Information** section: layer records,
//! channel image data (raw + PackBits RLE), group/section markers, Unicode
//! names, and raster layer masks, on top of the unchanged composite path.
//!
//! - Read: file header, color mode data (skip), image resources (skip), the
//!   layer/mask section (parsed when present), then the image data section.
//!   Supported: 8-bit, RGB or Grayscale, composite compression 0 (raw) or 1
//!   (RLE/PackBits); layer channel compression 0 or 1.
//! - Write: emit a valid PSD whose layer section round-trips through
//!   [`read_psd`], using raw channel data and `'luni'`/`'lsct'` tagged blocks.
//! - Anything outside the supported subset returns [`PsdError::Unsupported`],
//!   never a panic.

use pictura_core::{
    AdjustmentData, BitDepth, BlendMode, Channel, ColorMode, Document, Layer, LayerMask,
    PixelBuffer, PsdRect,
};

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

const SECTION_DIVIDER: u32 = 3;
const SECTION_OPEN_FOLDER: u32 = 1;
const SECTION_CLOSED_FOLDER: u32 = 2;

const COMPRESSION_RAW: u16 = 0;
const COMPRESSION_RLE: u16 = 1;

const DIVIDER_NAME: &str = "</Layer group>";

/// Additional-layer-info keys that carry an adjustment.
///
/// The brief's list plus the spellings Photoshop actually writes: Invert is
/// `nvrt` (not `invr`) and the legacy Hue/Saturation key is `hue ` alongside
/// `hue2`. Both spellings are accepted on read.
const ADJUSTMENT_KEYS: [[u8; 4]; 17] = [
    *b"levl", *b"curv", *b"brit", *b"expA", *b"vibA", *b"hue2", *b"hue ", *b"blwh", *b"phfl",
    *b"mixr", *b"gdrm", *b"invr", *b"nvrt", *b"post", *b"thrs", *b"selc", *b"clrL",
];

fn is_adjustment_key(key: &[u8; 4]) -> bool {
    ADJUSTMENT_KEYS.contains(key)
}

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

    fn u8(&mut self) -> Result<u8, PsdError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, PsdError> {
        let s = self.take(2)?;
        Ok(u16::from_be_bytes(
            s.try_into().map_err(|_| PsdError::Truncated)?,
        ))
    }

    fn i16(&mut self) -> Result<i16, PsdError> {
        Ok(self.u16()? as i16)
    }

    fn u32(&mut self) -> Result<u32, PsdError> {
        let s = self.take(4)?;
        Ok(u32::from_be_bytes(
            s.try_into().map_err(|_| PsdError::Truncated)?,
        ))
    }

    fn i32(&mut self) -> Result<i32, PsdError> {
        Ok(self.u32()? as i32)
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

/// Parse a PSD (or PSB) file into a [`Document`] holding the composite image and
/// the layer tree (bottom-first, matching PSD on-disk z-order).
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
    // Layer and mask information section: 4-byte length (8 in PSB).
    let layers = read_layer_section(&mut r, is_psb)?;

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
        layers,
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
        counts.push(read_rle_count(r, is_psb)?);
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

// ---------------------------------------------------------------------------
// Layer and mask information section — reading
// ---------------------------------------------------------------------------

/// One layer record before the tree is assembled. Channel info (id + declared
/// data length) is kept separate so channel image data can be read in a second
/// pass, after all records.
struct RawLayer {
    layer: Layer,
    channel_ids: Vec<i16>,
    channel_lens: Vec<usize>,
    section: Option<u32>,
}

fn read_layer_section(r: &mut Reader, is_psb: bool) -> Result<Vec<Layer>, PsdError> {
    let section_len = if is_psb {
        r.u64()? as usize
    } else {
        r.u32()? as usize
    };
    if section_len == 0 {
        return Ok(Vec::new());
    }
    let section_end = r
        .pos
        .checked_add(section_len)
        .ok_or_else(|| PsdError::Invalid("layer section length overflow".into()))?;
    if section_end > r.data.len() {
        return Err(PsdError::Truncated);
    }

    let info_len = if is_psb {
        r.u64()? as usize
    } else {
        r.u32()? as usize
    };
    let mut layers = Vec::new();
    if info_len != 0 {
        let info_end = r
            .pos
            .checked_add(info_len)
            .ok_or_else(|| PsdError::Invalid("layer info length overflow".into()))?;
        if info_end > section_end {
            return Err(PsdError::Invalid("layer info exceeds layer section".into()));
        }
        layers = read_layer_info(r, is_psb, info_end)?;
        r.pos = info_end;
    }

    // Global layer mask info: 4-byte length + opaque bytes (skipped).
    let global_len = r.u32()? as usize;
    r.skip(global_len)?;
    if r.pos > section_end {
        return Err(PsdError::Invalid(
            "global layer mask exceeds section".into(),
        ));
    }
    // Remaining bytes are additional layer information; not needed in M1.
    r.pos = section_end;
    Ok(layers)
}

fn read_layer_info(r: &mut Reader, is_psb: bool, info_end: usize) -> Result<Vec<Layer>, PsdError> {
    let count = r.i16()?;
    let n = count.unsigned_abs() as usize;
    let mut raws: Vec<RawLayer> = Vec::new();
    for _ in 0..n {
        raws.push(read_layer_record(r, is_psb)?);
    }

    for raw in raws.iter_mut() {
        let layer_w = raw.layer.rect.width().max(0) as usize;
        let layer_h = raw.layer.rect.height().max(0) as usize;
        let mask_dims = raw.layer.mask.as_ref().map(|m| {
            (
                m.rect.width().max(0) as usize,
                m.rect.height().max(0) as usize,
            )
        });

        let mut channels = Vec::new();
        let mut mask_data = None;
        for (&id, &len) in raw.channel_ids.iter().zip(raw.channel_lens.iter()) {
            // The user layer mask channel (-2) is sized by the mask rect, which
            // may differ from the layer rect.
            let (w, h) = if id == -2 {
                mask_dims.unwrap_or((layer_w, layer_h))
            } else {
                (layer_w, layer_h)
            };
            let data = read_channel_data(r, len, w, h, is_psb)?;
            match id {
                -2 => mask_data = Some(data),
                // Real user mask (-3) belongs to the vector/real mask path, out
                // of M1 scope; its bytes are consumed but not modelled.
                -3 => {}
                _ => channels.push(Channel { id, data }),
            }
        }
        raw.layer.channels = channels;
        if let Some(data) = mask_data {
            match raw.layer.mask.as_mut() {
                Some(mask) => mask.data = Some(data),
                None => {
                    raw.layer.mask = Some(LayerMask {
                        rect: raw.layer.rect,
                        default_color: 0,
                        disabled: false,
                        flags: 0,
                        data: Some(data),
                    });
                }
            }
        }
    }

    if r.pos > info_end {
        return Err(PsdError::Invalid("layer records exceed layer info".into()));
    }
    r.pos = info_end;
    Ok(build_tree(raws))
}

fn read_layer_record(r: &mut Reader, is_psb: bool) -> Result<RawLayer, PsdError> {
    let rect = PsdRect {
        top: r.i32()?,
        left: r.i32()?,
        bottom: r.i32()?,
        right: r.i32()?,
    };
    let nch = r.u16()?;
    if nch > MAX_CHANNELS {
        return Err(PsdError::Invalid(format!("layer channel count {nch}")));
    }
    let mut channel_ids = Vec::with_capacity(nch as usize);
    let mut channel_lens = Vec::with_capacity(nch as usize);
    for _ in 0..nch {
        channel_ids.push(r.i16()?);
        channel_lens.push(if is_psb {
            r.u64()? as usize
        } else {
            r.u32()? as usize
        });
    }

    let mut signature = [0u8; 4];
    signature.copy_from_slice(r.take(4)?);
    if &signature != b"8BIM" {
        return Err(PsdError::Invalid("bad layer blend signature".into()));
    }
    let mut key = [0u8; 4];
    key.copy_from_slice(r.take(4)?);
    let mut blend = BlendMode::from_psd_key(key)
        .ok_or_else(|| PsdError::Unsupported(format!("blend mode {:?}", key)))?;

    let opacity = r.u8()?;
    let clipping = r.u8()? != 0;
    let flags = r.u8()?;
    let _filler = r.u8()?;

    let extra_len = r.u32()? as usize;
    let extra = r.take(extra_len)?;
    let mut er = Reader::new(extra);

    // Layer mask / adjustment layer data block.
    let mask_len = er.u32()? as usize;
    let mut mask = None;
    if mask_len > 0 {
        let bytes = er.take(mask_len)?;
        let mut mr = Reader::new(bytes);
        let mask_rect = PsdRect {
            top: mr.i32()?,
            left: mr.i32()?,
            bottom: mr.i32()?,
            right: mr.i32()?,
        };
        let default_color = mr.u8()?;
        let mask_flags = mr.u8()?;
        mask = Some(LayerMask {
            rect: mask_rect,
            default_color,
            disabled: mask_flags & 0x02 != 0,
            flags: mask_flags,
            data: None,
        });
    }

    // Layer blending ranges (opaque).
    let ranges_len = er.u32()? as usize;
    er.skip(ranges_len)?;

    // Legacy Pascal name, padded so (length byte + chars) is a multiple of 4.
    let name_len = er.u8()? as usize;
    let name_bytes = er.take(name_len)?;
    let mut name = String::from_utf8_lossy(name_bytes).into_owned();
    let pad = (4 - ((name_len + 1) % 4)) % 4;
    er.skip(pad)?;

    // Additional layer information: 'luni' (Unicode name), 'lsct' (group marker),
    // and the adjustment block for adjustment layers (stored verbatim).
    let mut section = None;
    let mut adjustment = None;
    while er.remaining() >= 12 {
        let mut tag_sig = [0u8; 4];
        tag_sig.copy_from_slice(er.take(4)?);
        if &tag_sig != b"8BIM" {
            return Err(PsdError::Invalid("bad tagged block signature".into()));
        }
        let mut tag_key = [0u8; 4];
        tag_key.copy_from_slice(er.take(4)?);
        let tag_len = er.u32()? as usize;
        let data = er.take(tag_len)?;
        if tag_len % 2 == 1 {
            // Tagged block data is padded to an even length.
            let _ = er.skip(1);
        }
        match &tag_key {
            b"luni" => {
                if let Some(unicode) = parse_luni(data) {
                    name = unicode;
                }
            }
            b"lsct" if data.len() >= 4 => {
                section = Some(u32::from_be_bytes(data[0..4].try_into().unwrap()));
                // Photoshop/psd-tools store a group's blend key inside 'lsct'
                // (after the '8BIM' signature); the folder record's own key is
                // usually 'norm'. Prefer the 'lsct' key so pass-through groups
                // load as PassThrough.
                if data.len() >= 12 && &data[4..8] == b"8BIM" {
                    let lsct_key: [u8; 4] = data[8..12].try_into().unwrap();
                    if let Some(m) = BlendMode::from_psd_key(lsct_key) {
                        blend = m;
                    }
                }
            }
            b"lsct" => {}
            k if is_adjustment_key(k) && adjustment.is_none() => {
                adjustment = Some(AdjustmentData {
                    key: tag_key,
                    data: data.to_vec(),
                });
            }
            _ => {}
        }
    }

    Ok(RawLayer {
        layer: Layer {
            name,
            rect,
            blend,
            opacity,
            clipping,
            visible: flags & 0x02 == 0,
            mask,
            adjustment,
            channels: Vec::new(),
            children: Vec::new(),
            is_group: false,
        },
        channel_ids,
        channel_lens,
        section,
    })
}

/// Decode a `'luni'` tagged block: a `u32` UTF-16 code-unit count followed by
/// that many big-endian `u16` units (a trailing null may follow).
fn parse_luni(data: &[u8]) -> Option<String> {
    if data.len() < 4 {
        return None;
    }
    let count = u32::from_be_bytes(data[0..4].try_into().ok()?) as usize;
    let bytes = count.checked_mul(2)?;
    let end = (4 + bytes).min(data.len());
    let (pairs, _) = data[4..end].as_chunks::<2>();
    let units: Vec<u16> = pairs
        .iter()
        .map(|c| u16::from_be_bytes([c[0], c[1]]))
        .collect();
    Some(
        String::from_utf16_lossy(&units)
            .trim_end_matches('\0')
            .to_string(),
    )
}

fn read_rle_count(r: &mut Reader, is_psb: bool) -> Result<usize, PsdError> {
    Ok(if is_psb {
        r.u32()? as usize
    } else {
        r.u16()? as usize
    })
}

/// Read one layer channel's image data. `declared_len` comes from the channel
/// info and **includes** the 2-byte compression header.
fn read_channel_data(
    r: &mut Reader,
    declared_len: usize,
    width: usize,
    height: usize,
    is_psb: bool,
) -> Result<Vec<u8>, PsdError> {
    let pixels = width
        .checked_mul(height)
        .ok_or_else(|| PsdError::Invalid("layer channel size overflow".into()))?;
    if declared_len == 0 {
        return Ok(Vec::new());
    }
    if declared_len < 2 {
        return Err(PsdError::Invalid(format!(
            "layer channel length {declared_len}"
        )));
    }
    let compression = r.u16()?;
    let payload = r.take(declared_len - 2)?;
    match compression {
        COMPRESSION_RAW => {
            if payload.len() < pixels {
                return Err(PsdError::Invalid("raw layer channel too short".into()));
            }
            Ok(payload[..pixels].to_vec())
        }
        COMPRESSION_RLE => decode_rle_channel(payload, width, height, is_psb),
        2 | 3 => Err(PsdError::Unsupported(
            "ZIP layer channel compression".into(),
        )),
        c => Err(PsdError::Unsupported(format!(
            "layer channel compression {c}"
        ))),
    }
}

fn decode_rle_channel(
    payload: &[u8],
    width: usize,
    height: usize,
    is_psb: bool,
) -> Result<Vec<u8>, PsdError> {
    let mut pr = Reader::new(payload);
    let mut counts = Vec::with_capacity(height);
    for _ in 0..height {
        counts.push(read_rle_count(&mut pr, is_psb)?);
    }
    let mut out = vec![0u8; width * height];
    for (row, &count) in counts.iter().enumerate() {
        let packed = pr.take(count)?;
        decode_packbits(packed, &mut out[row * width..(row + 1) * width])?;
    }
    Ok(out)
}

/// Assemble the bottom-first layer tree from flat records. Section dividers
/// (`'lsct'` type 3) open a group; folder records (types 1/2) close it.
fn build_tree(raws: Vec<RawLayer>) -> Vec<Layer> {
    let mut stack: Vec<Vec<Layer>> = vec![Vec::new()];
    for raw in raws {
        match raw.section {
            Some(SECTION_DIVIDER) => stack.push(Vec::new()),
            Some(SECTION_OPEN_FOLDER) | Some(SECTION_CLOSED_FOLDER) => {
                let mut group = raw.layer;
                group.is_group = true;
                group.channels = Vec::new();
                group.mask = None;
                group.children = stack.pop().unwrap_or_default();
                stack.last_mut().unwrap().push(group);
            }
            _ => stack.last_mut().unwrap().push(raw.layer),
        }
    }
    // Unbalanced section dividers (bounding sections with no folder) are
    // flattened into their parent rather than dropped.
    while stack.len() > 1 {
        let contents = stack.pop().unwrap();
        stack.last_mut().unwrap().extend(contents);
    }
    stack.pop().unwrap()
}

// ---------------------------------------------------------------------------
// Layer and mask information section — writing
// ---------------------------------------------------------------------------

/// A flat record in on-disk order, borrowing the model layer it came from.
struct OutRecord<'a> {
    layer: Option<&'a Layer>,
    section: u32,
    name: &'a str,
}

fn flatten(layers: &[Layer]) -> Vec<OutRecord<'_>> {
    enum Frame<'a> {
        Visit(&'a Layer),
        Group(&'a Layer),
    }
    let mut out = Vec::new();
    let mut stack: Vec<Frame> = layers.iter().rev().map(Frame::Visit).collect();
    while let Some(frame) = stack.pop() {
        match frame {
            Frame::Visit(layer) if layer.is_group => {
                out.push(OutRecord {
                    layer: None,
                    section: SECTION_DIVIDER,
                    name: DIVIDER_NAME,
                });
                stack.push(Frame::Group(layer));
                for child in layer.children.iter().rev() {
                    stack.push(Frame::Visit(child));
                }
            }
            Frame::Visit(layer) => out.push(OutRecord {
                layer: Some(layer),
                section: 0,
                name: &layer.name,
            }),
            Frame::Group(layer) => out.push(OutRecord {
                layer: Some(layer),
                section: SECTION_OPEN_FOLDER,
                name: &layer.name,
            }),
        }
    }
    out
}

fn write_layer_info(doc: &Document) -> Result<Vec<u8>, PsdError> {
    let records = flatten(&doc.layers);
    if records.len() > i16::MAX as usize {
        return Err(PsdError::Unsupported("too many layer records".into()));
    }

    let mut info = Vec::new();
    info.extend_from_slice(&(records.len() as i16).to_be_bytes());

    let mut channel_data: Vec<Vec<(i16, Vec<u8>)>> = Vec::with_capacity(records.len());
    for record in &records {
        let mut channels: Vec<(i16, Vec<u8>)> = Vec::new();
        if let Some(layer) = record.layer {
            for channel in &layer.channels {
                channels.push((channel.id, channel.data.clone()));
            }
            if let Some(mask) = &layer.mask {
                let width = mask.rect.width().max(0) as usize;
                let height = mask.rect.height().max(0) as usize;
                let pixels = width
                    .checked_mul(height)
                    .ok_or_else(|| PsdError::Invalid("layer mask size overflow".into()))?;
                let data = match &mask.data {
                    Some(data) => {
                        if data.len() != pixels {
                            return Err(PsdError::Invalid(
                                "layer mask data length mismatch".into(),
                            ));
                        }
                        data.clone()
                    }
                    None => vec![mask.default_color; pixels],
                };
                channels.push((-2, data));
            }
        }
        write_record(&mut info, record, &channels);
        channel_data.push(channels);
    }

    for channels in &channel_data {
        for (_, data) in channels {
            info.extend_from_slice(&COMPRESSION_RAW.to_be_bytes());
            info.extend_from_slice(data);
        }
    }

    while info.len() % 4 != 0 {
        info.push(0);
    }
    Ok(info)
}

fn write_record(out: &mut Vec<u8>, record: &OutRecord, channels: &[(i16, Vec<u8>)]) {
    match record.layer {
        Some(layer) => {
            out.extend_from_slice(&layer.rect.top.to_be_bytes());
            out.extend_from_slice(&layer.rect.left.to_be_bytes());
            out.extend_from_slice(&layer.rect.bottom.to_be_bytes());
            out.extend_from_slice(&layer.rect.right.to_be_bytes());
            out.extend_from_slice(&(channels.len() as u16).to_be_bytes());
            for (id, data) in channels {
                out.extend_from_slice(&id.to_be_bytes());
                out.extend_from_slice(&((2 + data.len()) as u32).to_be_bytes());
            }
            out.extend_from_slice(b"8BIM");
            out.extend_from_slice(&layer.blend.to_psd_key());
            out.push(layer.opacity);
            out.push(u8::from(layer.clipping));
            out.push(if layer.visible { 0 } else { 0x02 });
            out.push(0); // filler

            let mut extra = Vec::new();
            write_extra(&mut extra, layer, record.name, record.section);
            out.extend_from_slice(&(extra.len() as u32).to_be_bytes());
            out.extend_from_slice(&extra);
        }
        None => {
            out.extend_from_slice(&[0u8; 16]); // empty rect
            out.extend_from_slice(&0u16.to_be_bytes()); // no channels
            out.extend_from_slice(b"8BIM");
            out.extend_from_slice(&BlendMode::Normal.to_psd_key());
            out.push(255);
            out.push(0);
            out.push(0);
            out.push(0);

            let mut extra = Vec::new();
            write_extra(
                &mut extra,
                &empty_layer(record.name),
                record.name,
                record.section,
            );
            out.extend_from_slice(&(extra.len() as u32).to_be_bytes());
            out.extend_from_slice(&extra);
        }
    }
}

/// Placeholder used to reuse [`write_extra`] for the (layer-less) divider record.
fn empty_layer(name: &str) -> Layer {
    Layer {
        name: name.to_string(),
        rect: PsdRect {
            top: 0,
            left: 0,
            bottom: 0,
            right: 0,
        },
        blend: BlendMode::Normal,
        opacity: 255,
        clipping: false,
        visible: true,
        mask: None,
        adjustment: None,
        channels: Vec::new(),
        children: Vec::new(),
        is_group: false,
    }
}

fn write_extra(out: &mut Vec<u8>, layer: &Layer, name: &str, section: u32) {
    match &layer.mask {
        Some(mask) => {
            // Mask block: rect (4×i32) + default colour + flags.
            out.extend_from_slice(&18u32.to_be_bytes());
            out.extend_from_slice(&mask.rect.top.to_be_bytes());
            out.extend_from_slice(&mask.rect.left.to_be_bytes());
            out.extend_from_slice(&mask.rect.bottom.to_be_bytes());
            out.extend_from_slice(&mask.rect.right.to_be_bytes());
            out.push(mask.default_color);
            let flags = (mask.flags & !0x02) | if mask.disabled { 0x02 } else { 0 };
            out.push(flags);
        }
        None => out.extend_from_slice(&0u32.to_be_bytes()),
    }
    // Blending ranges: empty.
    out.extend_from_slice(&0u32.to_be_bytes());
    write_pascal(out, name);
    write_tag(out, b"luni", &luni_data(name));
    if let Some(adjustment) = &layer.adjustment {
        // Adjustment payload is opaque here; write the key and bytes back as read.
        write_tag(out, &adjustment.key, &adjustment.data);
    }
    if section != 0 {
        // Section-divider setting: kind + '8BIM' + blend key. Photoshop and
        // psd-tools read a group's blend mode from here, so `pass` must ride
        // along with the section marker.
        let mut lsct = Vec::with_capacity(12);
        lsct.extend_from_slice(&section.to_be_bytes());
        lsct.extend_from_slice(b"8BIM");
        lsct.extend_from_slice(&layer.blend.to_psd_key());
        write_tag(out, b"lsct", &lsct);
    }
    if out.len() % 2 == 1 {
        out.push(0);
    }
}

fn write_pascal(out: &mut Vec<u8>, name: &str) {
    let bytes = name.as_bytes();
    let len = bytes.len().min(255);
    out.push(len as u8);
    out.extend_from_slice(&bytes[..len]);
    let pad = (4 - ((len + 1) % 4)) % 4;
    for _ in 0..pad {
        out.push(0);
    }
}

fn write_tag(out: &mut Vec<u8>, key: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(b"8BIM");
    out.extend_from_slice(key);
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(data);
    if data.len() % 2 == 1 {
        out.push(0);
    }
}

fn luni_data(name: &str) -> Vec<u8> {
    let units: Vec<u16> = name.encode_utf16().collect();
    let mut data = Vec::with_capacity(4 + units.len() * 2);
    data.extend_from_slice(&(units.len() as u32).to_be_bytes());
    for unit in units {
        data.extend_from_slice(&unit.to_be_bytes());
    }
    data
}

/// Serialize a [`Document`]'s composite image and layer tree into a valid PSD.
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

    if doc.layers.is_empty() {
        out.extend_from_slice(&0u32.to_be_bytes()); // zero-length layer/mask section
    } else {
        let info = write_layer_info(doc)?;
        let section_len = 4 + info.len() + 4; // layer info length + global mask length
        out.extend_from_slice(&(section_len as u32).to_be_bytes());
        out.extend_from_slice(&(info.len() as u32).to_be_bytes());
        out.extend_from_slice(&info);
        out.extend_from_slice(&0u32.to_be_bytes()); // empty global layer mask
    }

    out.extend_from_slice(&COMPRESSION_RAW.to_be_bytes());
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
        assert!(doc.layers.is_empty());
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

    // -- M1-B: layers ------------------------------------------------------

    fn rect(top: i32, left: i32, bottom: i32, right: i32) -> PsdRect {
        PsdRect {
            top,
            left,
            bottom,
            right,
        }
    }

    fn pixel(name: &str, r: PsdRect, color_channels: u8, blend: BlendMode, opacity: u8) -> Layer {
        let width = r.width().max(0) as usize;
        let height = r.height().max(0) as usize;
        let channels = (0..color_channels)
            .map(|c| Channel {
                id: c as i16,
                data: vec![c * 40 + 17; width * height],
            })
            .chain(std::iter::once(Channel {
                id: -1,
                data: vec![255; width * height],
            }))
            .collect();
        Layer {
            name: name.to_string(),
            rect: r,
            blend,
            opacity,
            clipping: false,
            visible: true,
            mask: None,
            adjustment: None,
            channels,
            children: Vec::new(),
            is_group: false,
        }
    }

    #[test]
    fn round_trip_layers_group_and_mask() {
        let mut doc = Document::new(16, 16, ColorMode::Rgb, BitDepth::Eight);
        for (i, b) in doc.composite.data.iter_mut().enumerate() {
            *b = (i % 251) as u8;
        }

        let red = pixel("Red", rect(0, 0, 4, 4), 3, BlendMode::Multiply, 200);
        let green = pixel("Green", rect(4, 4, 8, 8), 3, BlendMode::Screen, 255);
        let blue = pixel("Blue", rect(8, 8, 12, 12), 3, BlendMode::Normal, 128);
        let group = Layer {
            name: "Group A".to_string(),
            rect: rect(0, 0, 0, 0),
            blend: BlendMode::Normal,
            opacity: 255,
            clipping: false,
            visible: true,
            mask: None,
            adjustment: None,
            channels: Vec::new(),
            children: vec![green, blue],
            is_group: true,
        };

        let mut masked = pixel("Masked", rect(2, 2, 6, 6), 3, BlendMode::Overlay, 255);
        masked.mask = Some(LayerMask {
            rect: rect(2, 2, 6, 6),
            default_color: 0,
            disabled: true,
            flags: 0x02,
            data: Some(vec![7u8; 16]),
        });

        doc.layers = vec![red, group, masked];

        let bytes = write_psd(&doc).unwrap();
        let back = read_psd(&bytes).unwrap();

        assert_eq!(back.layers.len(), 3);
        assert_eq!(back.layers[0].name, "Red");
        assert_eq!(back.layers[0].blend, BlendMode::Multiply);
        assert_eq!(back.layers[0].opacity, 200);
        assert_eq!(back.layers[0].rect, rect(0, 0, 4, 4));
        assert!(back.layers[1].is_group());
        assert_eq!(back.layers[1].name, "Group A");
        assert_eq!(back.layers[1].children.len(), 2);
        assert_eq!(back.layers[1].children[0].name, "Green");
        assert_eq!(back.layers[1].children[1].name, "Blue");
        assert_eq!(back.layers[1].children[0].rect, rect(4, 4, 8, 8));
        let mask = back.layers[2].mask.as_ref().expect("mask round-trips");
        assert_eq!(mask.rect, rect(2, 2, 6, 6));
        assert!(mask.disabled);
        assert_eq!(mask.data.as_deref(), Some(&[7u8; 16][..]));

        // The strongest check: the whole document is equal.
        assert_eq!(back, doc);
    }

    #[test]
    fn pass_through_group_round_trips() {
        let mut doc = Document::new(8, 8, ColorMode::Rgb, BitDepth::Eight);
        let child = pixel("Child", rect(0, 0, 4, 4), 3, BlendMode::Multiply, 255);
        doc.layers = vec![Layer {
            name: "Pass Group".to_string(),
            rect: rect(0, 0, 0, 0),
            blend: BlendMode::PassThrough,
            opacity: 255,
            clipping: false,
            visible: true,
            mask: None,
            adjustment: None,
            channels: Vec::new(),
            children: vec![child],
            is_group: true,
        }];

        let bytes = write_psd(&doc).unwrap();
        let back = read_psd(&bytes).unwrap();
        assert!(back.layers[0].is_group());
        assert_eq!(back.layers[0].blend, BlendMode::PassThrough);
        assert_eq!(back, doc);
    }

    #[test]
    fn gray_layer_round_trips() {
        let mut doc = Document::new(8, 8, ColorMode::Grayscale, BitDepth::Eight);
        doc.layers = vec![pixel("Gray", rect(0, 0, 8, 8), 1, BlendMode::Normal, 255)];
        let bytes = write_psd(&doc).unwrap();
        assert_eq!(read_psd(&bytes).unwrap(), doc);
    }

    #[test]
    fn adjustment_layers_round_trip_key_and_bytes() {
        let mut doc = Document::new(8, 8, ColorMode::Rgb, BitDepth::Eight);
        for (i, b) in doc.composite.data.iter_mut().enumerate() {
            *b = (i % 251) as u8;
        }
        let base = pixel("Base", rect(0, 0, 8, 8), 3, BlendMode::Normal, 255);

        // Odd (`curv`) and even payloads, empty (Invert) and descriptor-ish.
        let cases: &[([u8; 4], Vec<u8>)] = &[
            (*b"nvrt", Vec::new()),
            (*b"post", vec![0, 4, 0, 0]),
            (*b"thrs", vec![0, 128, 0, 0]),
            (*b"brit", vec![0, 10, 0, 20, 0, 0, 0, 0]),
            (*b"hue2", vec![0; 16]),
            (*b"curv", vec![1, 2, 3]),
        ];

        let mut layers = vec![base];
        for (i, (key, data)) in cases.iter().enumerate() {
            layers.push(Layer {
                name: format!("adj{i}"),
                rect: rect(0, 0, 0, 0),
                blend: BlendMode::Normal,
                opacity: 255,
                clipping: false,
                visible: true,
                mask: None,
                adjustment: Some(AdjustmentData {
                    key: *key,
                    data: data.clone(),
                }),
                channels: Vec::new(),
                children: Vec::new(),
                is_group: false,
            });
        }
        doc.layers = layers;

        let back = read_psd(&write_psd(&doc).unwrap()).unwrap();
        assert_eq!(back, doc);
        for (i, (key, data)) in cases.iter().enumerate() {
            let adj = back.layers[i + 1]
                .adjustment
                .as_ref()
                .expect("adjustment round-trips");
            assert_eq!(&adj.key, key);
            assert_eq!(&adj.data, data);
        }
    }

    #[test]
    fn malformed_layer_section_is_error_not_panic() {
        let mut doc = Document::new(8, 8, ColorMode::Rgb, BitDepth::Eight);
        doc.layers = vec![pixel("Only", rect(0, 0, 8, 8), 3, BlendMode::Normal, 255)];
        let good = write_psd(&doc).unwrap();

        // Truncated inside the layer/mask section.
        assert!(read_psd(&good[..60]).is_err());

        // Bogus layer count: claims 100 records but the section holds one.
        let mut bogus = good.clone();
        bogus[42..44].copy_from_slice(&100i16.to_be_bytes());
        assert!(read_psd(&bogus).is_err());

        // Bogus channel data length: first channel info length at offset 64
        // (header 26 + section lengths 12 + count 2 + rect 16 + nch 2 + id 2).
        let mut bad_len = good.clone();
        bad_len[64..68].copy_from_slice(&u32::MAX.to_be_bytes());
        assert!(read_psd(&bad_len).is_err());
    }

    #[test]
    fn zip_layer_compression_is_unsupported() {
        let mut doc = Document::new(4, 4, ColorMode::Rgb, BitDepth::Eight);
        doc.layers = vec![pixel("Only", rect(0, 0, 4, 4), 3, BlendMode::Normal, 255)];
        let mut bytes = write_psd(&doc).unwrap();
        // Layer channel data follows the first (and only) record's extra data.
        let extra_len = u32::from_be_bytes(bytes[98..102].try_into().unwrap()) as usize;
        let channel_data = 102 + extra_len;
        bytes[channel_data + 1] = 2; // compression 2 = ZIP
        assert!(matches!(read_psd(&bytes), Err(PsdError::Unsupported(_))));
    }
}
