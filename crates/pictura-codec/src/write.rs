use pictura_core::*;

use crate::common::*;
use crate::error::PsdError;

// ---------------------------------------------------------------------------
// Layer and mask information section — writing
// ---------------------------------------------------------------------------

/// A flat record in on-disk order, borrowing the model layer it came from.
struct OutRecord<'a> {
    layer: Option<&'a Layer>,
    section: u32,
    name: &'a str,
}

/// A layer channel to emit. Both variants are a complete on-disk stream
/// (compression word included): `Encoded` is engine-authored PackBits RLE,
/// `Verbatim` is a preserved `Layer.raw_channels` stream re-emitted unchanged.
enum OutChannel {
    Encoded(Vec<u8>),
    Verbatim(Vec<u8>),
}

impl OutChannel {
    fn bytes(&self) -> &[u8] {
        match self {
            OutChannel::Encoded(data) | OutChannel::Verbatim(data) => data,
        }
    }

    fn declared_len(&self) -> u32 {
        self.bytes().len() as u32
    }

    fn write(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self.bytes());
    }
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
                name: record_name(layer),
            }),
            Frame::Group(layer) => out.push(OutRecord {
                layer: Some(layer),
                section: SECTION_OPEN_FOLDER,
                name: record_name(layer),
            }),
        }
    }
    out
}

/// A flagged layer is written under the PSD `"Background"` name convention
/// (design D5); the flag itself has no PSD bit.
fn record_name(layer: &Layer) -> &str {
    if layer.background {
        "Background"
    } else {
        &layer.name
    }
}

fn write_layer_info(doc: &Document) -> Result<Vec<u8>, PsdError> {
    let records = flatten(&doc.layers);
    if records.len() > i16::MAX as usize {
        return Err(PsdError::Unsupported("too many layer records".into()));
    }

    let mut info = Vec::new();
    info.extend_from_slice(&(records.len() as i16).to_be_bytes());

    let mut channel_data: Vec<Vec<(i16, OutChannel)>> = Vec::with_capacity(records.len());
    for record in &records {
        let mut channels: Vec<(i16, OutChannel)> = Vec::new();
        if let Some(layer) = record.layer {
            let layer_w = layer.rect.width().max(0) as usize;
            let layer_h = layer.rect.height().max(0) as usize;
            for channel in &layer.channels {
                channels.push((
                    channel.id,
                    OutChannel::Encoded(rle_channel(layer_w, layer_h, &channel.data)?),
                ));
            }
            for channel in &layer.raw_channels {
                channels.push((channel.id, OutChannel::Verbatim(channel.data.clone())));
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
                channels.push((-2, OutChannel::Encoded(rle_channel(width, height, &data)?)));
            }
        }
        if channels.len() > MAX_CHANNELS as usize {
            return Err(PsdError::Invalid(format!(
                "layer channel count {}",
                channels.len()
            )));
        }
        write_record(&mut info, record, &channels, doc.width, doc.height);
        channel_data.push(channels);
    }

    for channels in &channel_data {
        for (_, channel) in channels {
            channel.write(&mut info);
        }
    }

    while info.len() % 4 != 0 {
        info.push(0);
    }
    Ok(info)
}

fn write_record(
    out: &mut Vec<u8>,
    record: &OutRecord,
    channels: &[(i16, OutChannel)],
    doc_width: u32,
    doc_height: u32,
) {
    match record.layer {
        Some(layer) => {
            out.extend_from_slice(&layer.rect.top.to_be_bytes());
            out.extend_from_slice(&layer.rect.left.to_be_bytes());
            out.extend_from_slice(&layer.rect.bottom.to_be_bytes());
            out.extend_from_slice(&layer.rect.right.to_be_bytes());
            out.extend_from_slice(&(channels.len() as u16).to_be_bytes());
            for (id, channel) in channels {
                out.extend_from_slice(&id.to_be_bytes());
                out.extend_from_slice(&channel.declared_len().to_be_bytes());
            }
            out.extend_from_slice(b"8BIM");
            // Preserve an unrecognized blend key; a recognized key rides along
            // only while it still matches the mode, so a changed mode wins.
            let blend_key = layer
                .blend_key
                .filter(|k| match BlendMode::from_psd_key(*k) {
                    // An unknown key is the only representation of its mode, so
                    // it survives while the mode is still the Normal fallback.
                    None => layer.blend == BlendMode::Normal,
                    Some(mode) => mode == layer.blend,
                })
                .unwrap_or_else(|| layer.blend.to_psd_key());
            out.extend_from_slice(&blend_key);
            out.push(layer.opacity);
            out.push(u8::from(layer.clipping));
            let mut record_flags = if layer.visible { 0 } else { 0x02 };
            if layer.lock.contains(LockFlags::TRANSPARENCY) {
                record_flags |= 0x01;
            }
            out.push(record_flags);
            out.push(0); // filler

            let mut extra = Vec::new();
            write_extra(
                &mut extra,
                layer,
                record.name,
                record.section,
                doc_width,
                doc_height,
            );
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
                doc_width,
                doc_height,
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
        fill: 255,
        lock: LockFlags::default(),
        color: ColorLabel::None,
        clipping: false,
        visible: true,
        mask: None,
        adjustment: None,
        channels: Vec::new(),
        children: Vec::new(),
        is_group: false,
        background: false,
        ..Default::default()
    }
}

fn write_extra(
    out: &mut Vec<u8>,
    layer: &Layer,
    name: &str,
    section: u32,
    doc_width: u32,
    doc_height: u32,
) {
    match &layer.mask {
        Some(mask) => {
            // Mask block: rect (4×i32) + default colour + flags + preserved tail.
            out.extend_from_slice(&(18u32 + mask.extra.len() as u32).to_be_bytes());
            out.extend_from_slice(&mask.rect.top.to_be_bytes());
            out.extend_from_slice(&mask.rect.left.to_be_bytes());
            out.extend_from_slice(&mask.rect.bottom.to_be_bytes());
            out.extend_from_slice(&mask.rect.right.to_be_bytes());
            out.push(mask.default_color);
            let flags = (mask.flags & !0x02) | if mask.disabled { 0x02 } else { 0 };
            out.push(flags);
            out.extend_from_slice(&mask.extra);
        }
        None => out.extend_from_slice(&0u32.to_be_bytes()),
    }
    // Layer blending ranges, re-emitted verbatim (empty for engine documents).
    out.extend_from_slice(&(layer.blending_ranges.len() as u32).to_be_bytes());
    out.extend_from_slice(&layer.blending_ranges);
    write_pascal(out, name);
    write_tag(out, b"luni", &luni_data(name));
    // M36 layer attributes, each omitted at its default so default documents
    // serialize byte-identically to before.
    if layer.lock.bits() != 0 {
        write_tag(out, b"lspf", &(u32::from(layer.lock.bits())).to_be_bytes());
    }
    if layer.color != ColorLabel::None {
        let mut lclr = [0u8; 8];
        lclr[0..2].copy_from_slice(&u16::from(layer.color.to_byte()).to_be_bytes());
        write_tag(out, b"lclr", &lclr);
    }
    if layer.fill != 255 {
        write_tag(out, b"iOpa", &[layer.fill]);
    }
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
    // Unmodeled tagged blocks, re-emitted in encounter order.
    for block in &layer.extra_blocks {
        write_tag(out, &block.key, &block.data);
    }
    // An embedded smart object with no preserved config block is authored here.
    if let Some(so) = crate::smart_writer::should_author(layer) {
        let data = crate::smart_writer::author_sold_block(so, layer, doc_width, doc_height);
        write_tag(out, b"SoLd", &data);
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

pub(crate) fn write_tag(out: &mut Vec<u8>, key: &[u8; 4], data: &[u8]) {
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

// ---------------------------------------------------------------------------
// PackBits RLE encoding
// ---------------------------------------------------------------------------

/// Append one PackBits-encoded scanline to `out`. Runs of three or more equal
/// bytes become a repeat packet (control `1 - n`, then the byte); all other
/// bytes become literal packets of up to 128. The split is fixed, so repeated
/// writes are byte-identical.
fn encode_packbits_row(row: &[u8], out: &mut Vec<u8>) {
    let mut i = 0;
    while i < row.len() {
        let mut run = 1;
        while i + run < row.len() && row[i + run] == row[i] && run < 128 {
            run += 1;
        }
        if run >= 3 {
            out.push((1 - run as i32) as i8 as u8);
            out.push(row[i]);
            i += run;
            continue;
        }
        let start = i;
        let mut end = i;
        while end < row.len() && end - start < 128 {
            if end + 2 < row.len() && row[end] == row[end + 1] && row[end] == row[end + 2] {
                break;
            }
            end += 1;
        }
        out.push((end - start - 1) as u8);
        out.extend_from_slice(&row[start..end]);
        i = end;
    }
}

/// Encode `planes` (each `width * height`, plane-major) into one RLE payload:
/// all 2-byte scanline byte counts first (plane-major, then row-major), then the
/// packed rows in the same order. The compression word is not included.
pub(crate) fn encode_scanlines(
    planes: &[&[u8]],
    width: usize,
    height: usize,
) -> Result<Vec<u8>, PsdError> {
    let plane_len = width
        .checked_mul(height)
        .ok_or_else(|| PsdError::Invalid("RLE plane size overflow".into()))?;
    let mut counts = Vec::with_capacity(planes.len() * height * 2);
    let mut rows = Vec::new();
    for plane in planes {
        if plane.len() != plane_len {
            return Err(PsdError::Invalid("RLE plane length mismatch".into()));
        }
        for row in 0..height {
            let start = row * width;
            let mut packed = Vec::new();
            encode_packbits_row(&plane[start..start + width], &mut packed);
            // PSB's 4-byte counts are where a wider limit is needed; the PSD
            // maximum width (30 000) cannot reach the u16 limit. ponytail: PSD only.
            if packed.len() > u16::MAX as usize {
                return Err(PsdError::Invalid(
                    "RLE scanline exceeds u16 byte count".into(),
                ));
            }
            counts.extend_from_slice(&(packed.len() as u16).to_be_bytes());
            rows.extend_from_slice(&packed);
        }
    }
    counts.extend_from_slice(&rows);
    Ok(counts)
}

/// The complete on-disk layer-channel stream for one engine-encoded plane: the
/// compression word, the count table, then the packed rows.
fn rle_channel(width: usize, height: usize, plane: &[u8]) -> Result<Vec<u8>, PsdError> {
    let mut out = Vec::with_capacity(2 + plane.len());
    out.extend_from_slice(&COMPRESSION_RLE.to_be_bytes());
    out.extend_from_slice(&encode_scanlines(&[plane], width, height)?);
    Ok(out)
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
    let color_channels = doc.composite.channels as usize;
    let channels = color_channels + doc.channels.len();
    if channels == 0 || channels > MAX_CHANNELS as usize {
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
    let plane = doc.width as usize * doc.height as usize;
    if doc.composite.data.len() != color_channels * plane {
        return Err(PsdError::Invalid("composite data length mismatch".into()));
    }
    for channel in &doc.channels {
        if channel.data.len() != plane {
            return Err(PsdError::Invalid(format!(
                "document channel {} data length mismatch",
                channel.id
            )));
        }
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
    out.extend_from_slice(&(doc.color_mode_data.len() as u32).to_be_bytes());
    out.extend_from_slice(&doc.color_mode_data);
    out.extend_from_slice(&(doc.image_resources.len() as u32).to_be_bytes());
    out.extend_from_slice(&doc.image_resources);

    // Authored smart objects append a document-level linked-record block; the
    // preserved trailing bytes stay untouched, so an existing file is unchanged.
    let mut extra = doc.layer_section_extra.clone();
    let authoring = crate::smart_writer::collect_authoring(&doc.layers);
    if !authoring.is_empty() {
        extra.extend_from_slice(&crate::smart_writer::author_lnk2_bytes(&authoring));
    }

    if doc.layers.is_empty() && doc.global_layer_mask.is_empty() && extra.is_empty() {
        out.extend_from_slice(&0u32.to_be_bytes()); // zero-length layer/mask section
    } else {
        let info = write_layer_info(doc)?;
        let section_len = 4 + info.len() + 4 + doc.global_layer_mask.len() + extra.len();
        out.extend_from_slice(&(section_len as u32).to_be_bytes());
        out.extend_from_slice(&(info.len() as u32).to_be_bytes());
        out.extend_from_slice(&info);
        out.extend_from_slice(&(doc.global_layer_mask.len() as u32).to_be_bytes());
        out.extend_from_slice(&doc.global_layer_mask);
        out.extend_from_slice(&extra);
    }

    out.extend_from_slice(&COMPRESSION_RLE.to_be_bytes());
    let mut planes: Vec<&[u8]> = Vec::with_capacity(channels);
    for c in 0..color_channels {
        planes.push(&doc.composite.data[c * plane..(c + 1) * plane]);
    }
    for channel in &doc.channels {
        planes.push(&channel.data);
    }
    out.extend_from_slice(&encode_scanlines(
        &planes,
        doc.width as usize,
        doc.height as usize,
    )?);
    Ok(out)
}
