use pictura_core::*;

use crate::common::*;
use crate::error::PsdError;

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

    // Image data section: 2-byte compression method, then one plane per header
    // channel (color channels first, then alpha/spot/selection channels).
    let compression = r.u16()?;
    let header_channels = channels as usize;
    let width = width as usize;
    let height = height as usize;
    let data = match compression {
        0 => r
            .take(planar_len(header_channels, width, height)?)?
            .to_vec(),
        1 => read_rle(&mut r, header_channels, width, height, is_psb)?,
        c => return Err(PsdError::Unsupported(format!("compression {c}"))),
    };
    let (composite, channels) = split_planes(data, mode, width, height, header_channels)?;

    Ok(Document {
        width: width as u32,
        height: height as u32,
        mode,
        depth: BitDepth::Eight,
        composite,
        layers,
        channels,
    })
}

/// Split the planar image-data section into the mode's color planes (the
/// composite) and the trailing extra channels (saved selections / alpha).
fn split_planes(
    mut data: Vec<u8>,
    mode: ColorMode,
    width: usize,
    height: usize,
    header_channels: usize,
) -> Result<(PixelBuffer, Vec<Channel>), PsdError> {
    let color_channels = mode.color_channels() as usize;
    if header_channels < color_channels {
        return Err(PsdError::Invalid(format!(
            "header has {header_channels} channels for a {color_channels}-channel mode"
        )));
    }
    let plane = width * height;
    let extra = data.split_off(color_channels * plane);
    let channels = extra
        .chunks_exact(plane)
        .enumerate()
        .map(|(i, plane)| Channel {
            id: i as i16,
            data: plane.to_vec(),
        })
        .collect();
    Ok((
        PixelBuffer {
            width: width as u32,
            height: height as u32,
            channels: color_channels as u8,
            data,
        },
        channels,
    ))
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
    let mut layers = build_tree(raws);
    derive_background(&mut layers);
    Ok(layers)
}

/// PSD has no background bit: the bottom top-level, non-group layer named
/// `"Background"` is the Background (design D5, marked inferred).
fn derive_background(layers: &mut [Layer]) {
    if let Some(bottom) = layers.first_mut() {
        if !bottom.is_group && bottom.name == "Background" {
            bottom.background = true;
        }
    }
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

    // M36 attribute defaults; the tagged blocks below override them.
    let mut fill = 255u8;
    let mut lock = LockFlags::default();
    let mut color = ColorLabel::None;

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
            b"lspf" if data.len() >= 4 => {
                let value = u32::from_be_bytes(data[0..4].try_into().unwrap());
                lock = lock_from_bits(value as u8 & 0x0F);
            }
            b"lclr" if data.len() >= 2 => {
                let value = u16::from_be_bytes(data[0..2].try_into().unwrap());
                color = ColorLabel::from_byte(value as u8);
            }
            b"iOpa" if !data.is_empty() => {
                fill = data[0];
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

    // Photoshop ≤5-era transparency-protected bit shares the transparency lock.
    if flags & 0x01 != 0 {
        lock = lock.with(LockFlags::TRANSPARENCY, true);
    }

    Ok(RawLayer {
        layer: Layer {
            name,
            rect,
            blend,
            opacity,
            fill,
            lock,
            color,
            clipping,
            visible: flags & 0x02 == 0,
            mask,
            adjustment,
            channels: Vec::new(),
            children: Vec::new(),
            is_group: false,
            background: false,
        },
        channel_ids,
        channel_lens,
        section,
    })
}

/// Build [`LockFlags`] from the four `lspf`/record `flags` low bits. The
/// newtype has no public bit constructor, so set each lock explicitly.
fn lock_from_bits(bits: u8) -> LockFlags {
    LockFlags::default()
        .with(LockFlags::TRANSPARENCY, bits & LockFlags::TRANSPARENCY != 0)
        .with(LockFlags::PIXELS, bits & LockFlags::PIXELS != 0)
        .with(LockFlags::POSITION, bits & LockFlags::POSITION != 0)
        .with(LockFlags::NESTING, bits & LockFlags::NESTING != 0)
}

/// Decode a `'luni'` tagged block: a `u32` UTF-16 code-unit count followed by
/// that many big-endian `u16` units (a trailing null may follow).
fn parse_luni(data: &[u8]) -> Option<String> {
    if data.len() < 4 {
        return None;
    }
    let count = u32::from_be_bytes(data[0..4].try_into().ok()?) as usize;
    if count == 0 {
        return None;
    }
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
