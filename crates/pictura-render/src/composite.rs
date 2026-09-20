use pictura_adjust::{
    Adjustment, BlackWhiteParams, BrightnessContrastParams, ExposureParams, GradientMapParams,
    GradientStop, HueSaturationParams, LevelsParams, PhotoFilterParams, VibranceParams,
};
use pictura_codec::DescValue;
use pictura_core::{
    AdjustmentData, BlendMode, ColorMode, Document, Layer, PixelBuffer, PsdRect, SmartObject,
    SmartObjectKind,
};

pub(crate) use crate::blend::blend;

/// Composite the document's layer stack.
///
/// Returns a 4-channel (R,G,B,A) planar, straight-alpha, 8-bit buffer at
/// document resolution.
pub fn composite_rgba(doc: &Document) -> PixelBuffer {
    let mut canvas = Canvas::new(doc.width as usize, doc.height as usize);
    for layer in &doc.layers {
        composite_layer(&mut canvas, layer, doc);
    }
    canvas.into_pixel_buffer()
}

/// Straight-alpha RGBA accumulator in normalized `f32`.
///
/// ponytail: one full-document f32 buffer; streaming/tiled compositing is the
/// ceiling to raise when PSB-size docs no longer fit in memory.
#[derive(Clone, Copy, Default)]
struct Px {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

pub(crate) struct Canvas {
    pub(crate) w: usize,
    pub(crate) h: usize,
    px: Vec<Px>,
}

impl Canvas {
    pub(crate) fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            px: vec![Px::default(); w * h],
        }
    }

    pub(crate) fn into_pixel_buffer(self) -> PixelBuffer {
        let plane = self.w * self.h;
        let mut out = PixelBuffer::new(self.w as u32, self.h as u32, 4);
        for (i, p) in self.px.iter().enumerate() {
            out.data[i] = to_u8(p.r);
            out.data[plane + i] = to_u8(p.g);
            out.data[2 * plane + i] = to_u8(p.b);
            out.data[3 * plane + i] = to_u8(p.a);
        }
        out
    }
}

fn to_u8(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn composite_layer(canvas: &mut Canvas, layer: &Layer, doc: &Document) {
    if !layer.visible {
        return;
    }
    // The below-content effects (drop shadow, outer glow) render behind the
    // layer's own content.
    crate::layer_effects::composite_layer_effects(canvas, layer, doc);
    if layer.is_group {
        // True pass-through: recurse children straight onto the running canvas
        // so their blend modes see content outside the group. Only exact for
        // opacity 255 / no mask; otherwise fall back to isolated compositing
        // (see the crate-level note).
        if matches!(layer.blend, BlendMode::PassThrough)
            && layer.opacity == 255
            && layer.mask.is_none()
        {
            for child in &layer.children {
                composite_layer(canvas, child, doc);
            }
            return;
        }
        let mut inner = Canvas::new(canvas.w, canvas.h);
        for child in &layer.children {
            composite_layer(&mut inner, child, doc);
        }
        composite_canvas(canvas, layer, &inner);
    } else if let Some(data) = &layer.adjustment {
        // An adjustment layer owns no pixels: it transforms the backdrop it is
        // composited over. Unknown/undecodable keys are a no-op (preserved on
        // save, not applied), never an error.
        if let Some(adjustment) = decode_adjustment(data) {
            composite_adjustment(canvas, layer, doc, &adjustment);
        }
    } else {
        if !composite_smart_source(canvas, layer, layer.smart_object.as_ref()) {
            composite_pixels(canvas, layer, doc);
        }
    }
    // The above-content effect (inner shadow) renders over the layer's content.
    crate::layer_effects::composite_layer_effects_above(canvas, layer, doc);
}

fn composite_pixels(canvas: &mut Canvas, layer: &Layer, doc: &Document) {
    // A smart-object layer with no raster proxy has no channel to draw from. The
    // embedded-source branch above handles it; if that source is unusable, leave
    // the backdrop unchanged instead of painting the channel-less rect black.
    if layer.smart_object.is_some() && channel(layer, 0).is_none() {
        return;
    }
    let lw = layer.rect.width();
    let lh = layer.rect.height();
    if lw <= 0 || lh <= 0 {
        return;
    }
    let lw = lw as usize;
    let x0 = layer.rect.left.max(0);
    let y0 = layer.rect.top.max(0);
    let x1 = layer.rect.right.min(canvas.w as i32);
    let y1 = layer.rect.bottom.min(canvas.h as i32);
    if x1 <= x0 || y1 <= y0 {
        return;
    }

    let gray = matches!(
        doc.mode,
        ColorMode::Grayscale | ColorMode::Bitmap | ColorMode::Duotone
    );
    let ch0 = channel(layer, 0);
    let ch1 = channel(layer, 1).or(ch0);
    let ch2 = channel(layer, 2).or(ch0);
    let alpha = channel(layer, -1);

    for y in y0..y1 {
        for x in x0..x1 {
            let li = (y - layer.rect.top) as usize * lw + (x - layer.rect.left) as usize;
            let (r, g, b) = if gray {
                let v = sample(ch0, li).unwrap_or(0);
                (v, v, v)
            } else {
                (
                    sample(ch0, li).unwrap_or(0),
                    sample(ch1, li).unwrap_or(0),
                    sample(ch2, li).unwrap_or(0),
                )
            };
            let a = sample(alpha, li).unwrap_or(255);
            blend_into(
                canvas,
                layer,
                x as usize,
                y as usize,
                [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0],
                a as f32 / 255.0,
            );
        }
    }
}

/// Render an embedded smart object's source over `region` (document
/// coordinates, within `rect`), scaling with the renderer's integer ratio.
///
/// The returned straight-alpha RGBA buffer has `region`'s dimensions, so a
/// caller that only needs the canvas-clipped part allocates only that much.
/// Source sampling still maps through the full `rect`: a pixel's source is
/// chosen from its offset within `rect`, not within `region`.
///
/// `None` when the object is not embedded, its payload is empty or cannot be
/// decoded, the decoded source is empty, or either rectangle has no positive
/// area. Shared by compositing and by rasterizing a channel-less smart-object
/// layer.
///
/// ponytail: an embedded source that itself holds a no-proxy smart object
/// recurses without a depth guard; a crafted cyclic payload could loop. Add a
/// depth pass-through when untrusted embeddings appear.
pub(crate) fn render_smart_source(
    so: &SmartObject,
    rect: PsdRect,
    region: PsdRect,
) -> Option<PixelBuffer> {
    if so.kind != SmartObjectKind::Embedded {
        return None;
    }
    let payload = so.payload.as_deref().filter(|p| !p.is_empty())?;
    let embedded = pictura_codec::read_psd(payload).ok()?;
    let comp = &embedded.composite;
    let usable = comp.width > 0
        && comp.height > 0
        && comp.channels >= 1
        && comp.data.len() == comp.width as usize * comp.height as usize * comp.channels as usize;
    let fallback;
    let src: &PixelBuffer = if embedded.merged_composite_present && usable {
        comp
    } else {
        fallback = composite_rgba(&embedded);
        &fallback
    };

    let src_w = src.width as usize;
    let src_h = src.height as usize;
    let ch = src.channels as usize;
    if src_w == 0 || src_h == 0 || ch == 0 {
        return None;
    }
    let plane = src_w * src_h;

    let lw = rect.width();
    let lh = rect.height();
    if lw <= 0 || lh <= 0 {
        return None;
    }
    let lw = lw as usize;
    let lh = lh as usize;

    let rw = region.width();
    let rh = region.height();
    if rw <= 0 || rh <= 0 {
        return None;
    }
    let rw = rw as usize;
    let rh = rh as usize;
    let out_plane = rw * rh;
    let mut out = PixelBuffer::new(rw as u32, rh as u32, 4);
    for y in region.top..region.bottom {
        let ly = (y - rect.top) as u64;
        for x in region.left..region.right {
            let lx = (x - rect.left) as u64;
            let sx = lx * src_w as u64 / lw as u64;
            let sy = ly * src_h as u64 / lh as u64;
            let si = sy as usize * src_w + sx as usize;
            if si >= plane {
                continue;
            }
            let (r, g, b) = if ch < 3 {
                let v = src.data[si];
                (v, v, v)
            } else {
                (src.data[si], src.data[plane + si], src.data[2 * plane + si])
            };
            let a = if ch >= 4 {
                src.data[3 * plane + si]
            } else {
                255
            };
            let i = (y - region.top) as usize * rw + (x - region.left) as usize;
            out.data[i] = r;
            out.data[out_plane + i] = g;
            out.data[2 * out_plane + i] = b;
            out.data[3 * out_plane + i] = a;
        }
    }
    Some(out)
}

/// Render a layer from its embedded smart-object source when it has no raster
/// proxy, scaling the decoded source into the layer rect. Returns `true` when
/// it produced pixels, so the caller skips the (empty) channel path.
///
/// `External`/`Alias`/`Unresolved`, an empty payload, or a decode failure
/// returns `false`; the caller then tries the normal path, which draws nothing
/// for a channel-less layer.
fn composite_smart_source(canvas: &mut Canvas, layer: &Layer, so: Option<&SmartObject>) -> bool {
    let Some(so) = so else {
        return false;
    };
    if channel(layer, 0).is_some() {
        return false;
    }
    let region = PsdRect {
        top: layer.rect.top.max(0),
        left: layer.rect.left.max(0),
        bottom: layer.rect.bottom.min(canvas.h as i32),
        right: layer.rect.right.min(canvas.w as i32),
    };
    if region.width() <= 0 || region.height() <= 0 {
        return false;
    }
    let Some(src) = render_smart_source(so, layer.rect, region) else {
        return false;
    };

    let rw = region.width() as usize;
    let rh = region.height() as usize;
    let plane = rw * rh;
    for by in 0..rh {
        for bx in 0..rw {
            let i = by * rw + bx;
            blend_into(
                canvas,
                layer,
                region.left as usize + bx,
                region.top as usize + by,
                [
                    src.data[i] as f32 / 255.0,
                    src.data[plane + i] as f32 / 255.0,
                    src.data[2 * plane + i] as f32 / 255.0,
                ],
                src.data[3 * plane + i] as f32 / 255.0,
            );
        }
    }
    true
}

fn composite_canvas(canvas: &mut Canvas, layer: &Layer, inner: &Canvas) {
    for y in 0..canvas.h {
        for x in 0..canvas.w {
            let p = inner.px[y * inner.w + x];
            if p.a > 0.0 {
                blend_into(canvas, layer, x, y, [p.r, p.g, p.b], p.a);
            }
        }
    }
}

/// Decode a raw PSD adjustment block into a destructive [`Adjustment`] for the
/// encodings this crate supports. `None` means "not understood": the caller
/// leaves the backdrop unchanged (no-op), never errors.
///
/// Supported keys: `nvrt`/`invr` (Invert, no payload), `post` (Posterize),
/// `thrs` (Threshold), `brit` (Brightness/Contrast), `levl` (Levels, composite
/// record), `hue2`/`hue ` (Hue/Saturation), `expA` (Exposure), `vibA`
/// (Vibrance), `blwh` (Black & White), `phfl` (Photo Filter, version 2),
/// `grdm` (Gradient Map, versions 1/3), `blnc` (Color Balance), `mixr`
/// (Channel Mixer), `SoCo`
/// (solid-color fill content, either the 4-byte in-house RGBA tuple or the
/// standard Photoshop descriptor), `GdFl` (gradient fill content), and `PtFl`
/// (pattern fill content), both fill descriptors. Descriptor/custom
/// payloads (`curv`, `selc`, `clrL`, and a version-3 `phfl`) are
/// preserved on disk but not decoded here.
pub fn decode_adjustment(data: &AdjustmentData) -> Option<Adjustment> {
    match &data.key {
        b"nvrt" | b"invr" => Some(Adjustment::Invert),
        b"post" => be_u16(&data.data, 0)
            .filter(|v| (2..=255).contains(v))
            .map(|v| Adjustment::Posterize(v as u8)),
        b"thrs" => be_u16(&data.data, 0)
            .filter(|v| (1..=255).contains(v))
            .map(|v| Adjustment::Threshold(v as u8)),
        b"brit" => decode_brightness_contrast(&data.data),
        b"levl" => decode_levels(&data.data),
        b"hue2" | b"hue " => decode_hue_saturation(&data.data),
        b"expA" => decode_exposure(&data.data),
        b"phfl" => decode_photo_filter(&data.data),
        b"vibA" => decode_vibrance(&data.data),
        b"blwh" => decode_black_white(&data.data),
        b"gdrm" | b"grdm" => decode_gradient_map(&data.data),
        b"blnc" => crate::color_balance::decode_color_balance(&data.data),
        b"mixr" => crate::channel_mixer::decode_channel_mixer(&data.data),
        b"SoCo" => match data.data.as_slice() {
            [r, g, b, a] => Some(Adjustment::SolidFill([*r, *g, *b, *a])),
            _ => decode_solid_fill(&data.data),
        },
        b"GdFl" => crate::fill::decode_gradient_fill(&data.data),
        b"PtFl" => crate::fill::decode_pattern_fill(&data.data),
        _ => None,
    }
}

/// `SoCo`: the standard Photoshop solid-color fill descriptor. A version-16
/// `DescriptorBlock` whose `Clr ` object carries `Rd  `/`Grn `/`Bl  ` `doub`
/// values on the `0..=255` scale; each is rounded and clamped, and alpha is
/// forced to 255 (the descriptor has none). Missing key, wrong type,
/// non-finite value, or a parse error is `None`, never a panic.
fn decode_solid_fill(d: &[u8]) -> Option<Adjustment> {
    let obj = pictura_codec::read_descriptor(d).ok()?;
    let clr = desc_item(&obj, b"Clr ")?;
    if !matches!(clr, DescValue::Object { .. }) {
        return None;
    }
    let component = |key: &[u8]| -> Option<u8> {
        match desc_item(clr, key) {
            Some(DescValue::Double(c)) if c.is_finite() => Some(c.round().clamp(0.0, 255.0) as u8),
            _ => None,
        }
    };
    Some(Adjustment::SolidFill([
        component(b"Rd  ")?,
        component(b"Grn ")?,
        component(b"Bl  ")?,
        255,
    ]))
}

pub(crate) fn be_u16(d: &[u8], at: usize) -> Option<u16> {
    let s = d.get(at..at + 2)?;
    Some(u16::from_be_bytes([s[0], s[1]]))
}

pub(crate) fn be_i16(d: &[u8], at: usize) -> Option<i16> {
    Some(be_u16(d, at)? as i16)
}

fn be_f32(d: &[u8], at: usize) -> Option<f32> {
    let s = d.get(at..at + 4)?;
    Some(f32::from_be_bytes([s[0], s[1], s[2], s[3]]))
}

fn be_u32(d: &[u8], at: usize) -> Option<u32> {
    let s = d.get(at..at + 4)?;
    Some(u32::from_be_bytes([s[0], s[1], s[2], s[3]]))
}

/// `brit`: brightness (i16), contrast (i16), mean (i16), lab_only (u8), pad.
///
/// Photoshop's `brit` block carries no explicit "Use Legacy" flag; CS6 defaults
/// new adjustment layers to the modern curve, so decode with `use_legacy:false`.
/// ponytail: legacy-vs-modern detection is not encoded here; revisit if a CS6
/// baseline for legacy `brit` files appears.
fn decode_brightness_contrast(d: &[u8]) -> Option<Adjustment> {
    let brightness = be_i16(d, 0)?;
    let contrast = be_i16(d, 2)?;
    if !(-150..=150).contains(&brightness) || !(-50..=100).contains(&contrast) {
        return None;
    }
    Some(Adjustment::BrightnessContrast(BrightnessContrastParams {
        brightness,
        contrast,
        use_legacy: false,
    }))
}

/// `levl`: `u16 version` (2) then 29 five-`u16` records. Record 0 is the
/// composite channel; this decoder applies it uniformly to R/G/B.
fn decode_levels(d: &[u8]) -> Option<Adjustment> {
    if be_u16(d, 0)? != 2 {
        return None;
    }
    let input_black = be_u16(d, 2)?;
    let input_white = be_u16(d, 4)?;
    let output_black = be_u16(d, 6)?;
    let output_white = be_u16(d, 8)?;
    let gamma = be_u16(d, 10)?;
    if input_black >= input_white || gamma == 0 {
        return None;
    }
    if [input_black, input_white, output_black, output_white]
        .into_iter()
        .any(|v| v > 255)
    {
        return None;
    }
    Some(Adjustment::Levels(LevelsParams {
        input_black: input_black as u8,
        input_white: input_white as u8,
        gamma: gamma as f64 / 100.0,
        output_black: output_black as u8,
        output_white: output_white as u8,
    }))
}

/// `hue2` (and legacy `hue `): version `u16`, enable `u8`, pad, colorization
/// (3×i16), then the master Hue/Saturation/Lightness triplet (3×i16).
fn decode_hue_saturation(d: &[u8]) -> Option<Adjustment> {
    if be_u16(d, 0)? != 2 {
        return None;
    }
    let hue = be_i16(d, 10)?;
    let saturation = be_i16(d, 12)?;
    let lightness = be_i16(d, 14)?;
    if !(-180..=180).contains(&hue)
        || !(-100..=100).contains(&saturation)
        || !(-100..=100).contains(&lightness)
    {
        return None;
    }
    Some(Adjustment::HueSaturation(HueSaturationParams {
        hue,
        saturation,
        lightness,
    }))
}

/// `expA`: `u16` version (= 1), then big-endian `f32` exposure, offset, and
/// gamma.
fn decode_exposure(d: &[u8]) -> Option<Adjustment> {
    if be_u16(d, 0)? != 1 {
        return None;
    }
    let exposure = be_f32(d, 2)? as f64;
    let offset = be_f32(d, 6)? as f64;
    let gamma = be_f32(d, 10)? as f64;
    if !exposure.is_finite() || !offset.is_finite() || !gamma.is_finite() || gamma <= 0.0 {
        return None;
    }
    Some(Adjustment::Exposure(ExposureParams {
        exposure,
        offset,
        gamma,
    }))
}

/// `phfl` (version 2): `u16` version, `u16` colour space (ignored), four `u16`
/// colour components (first three = R, G, B; fourth ignored), `u32` density,
/// `u8` luminosity. A colour space other than RGB is decoded as RGB.
///
/// ponytail: the block length is flexible — only the 17 field bytes are read, so
/// an unpadded 17–19 byte payload still decodes while 20-byte writers round-trip.
///
/// ponytail: version 3 stores three `u32` CIE XYZ values and needs an XYZ→sRGB
/// transform that is not groundable here; it stays a no-op until a real CS6
/// `phfl` baseline exists.
fn decode_photo_filter(d: &[u8]) -> Option<Adjustment> {
    if be_u16(d, 0)? != 2 {
        return None;
    }
    let component = |at: usize| -> Option<u8> {
        let v = be_u16(d, at)?;
        (v <= 255).then_some(v as u8)
    };
    let color = [component(4)?, component(6)?, component(8)?];
    let density = be_u32(d, 12)?;
    if density > 100 {
        return None;
    }
    let preserve_luminosity = *d.get(16)? != 0;
    Some(Adjustment::PhotoFilter(PhotoFilterParams {
        color,
        density: density as f64,
        preserve_luminosity,
    }))
}

/// `vibA`: descriptor block with `vibrance` and the legacy `Strt` (saturation)
/// integer keys. Missing keys are 0; wrong types and out-of-range values are
/// rejected so a corrupt file cannot silently render a different adjustment.
fn decode_vibrance(d: &[u8]) -> Option<Adjustment> {
    let obj = pictura_codec::read_descriptor(d).ok()?;
    let vibrance = desc_long_or(&obj, b"vibrance", 0.0)?;
    let saturation = desc_long_or(&obj, b"Strt", 0.0)?;
    if !(-100.0..=100.0).contains(&vibrance) || !(-100.0..=100.0).contains(&saturation) {
        return None;
    }
    Some(Adjustment::Vibrance(VibranceParams {
        vibrance: vibrance as i16,
        saturation: saturation as i16,
    }))
}

/// `blwh`: descriptor block with the six channel-percentage longs, a `useTint`
/// bool, and a nested `Clr ` `tintColor` object whose `Rd  `/`Grn `/`Bl  `
/// doubles are 0..1. Missing numeric keys default to Photoshop's Black & White
/// defaults; absent `tintColor` is black.
fn decode_black_white(d: &[u8]) -> Option<Adjustment> {
    let obj = pictura_codec::read_descriptor(d).ok()?;
    let red = desc_long_or(&obj, b"Rd  ", 40.0)?;
    let yellow = desc_long_or(&obj, b"Yllw", 60.0)?;
    let green = desc_long_or(&obj, b"Grn ", 40.0)?;
    let cyan = desc_long_or(&obj, b"Cyn ", 60.0)?;
    let blue = desc_long_or(&obj, b"Bl  ", 20.0)?;
    let magenta = desc_long_or(&obj, b"Mgnt", 80.0)?;
    if [red, yellow, green, cyan, blue, magenta]
        .into_iter()
        .any(|v| !(-200.0..=300.0).contains(&v))
    {
        return None;
    }
    let tint = match desc_item(&obj, b"useTint") {
        Some(DescValue::Bool(b)) => *b,
        Some(_) => return None,
        None => false,
    };
    let tint_color = match desc_item(&obj, b"tintColor") {
        Some(value) => decode_tint_color(value)?,
        None => [0, 0, 0],
    };
    Some(Adjustment::BlackWhite(BlackWhiteParams {
        red,
        yellow,
        green,
        cyan,
        blue,
        magenta,
        tint,
        tint_color,
    }))
}

fn decode_tint_color(value: &DescValue) -> Option<[u8; 3]> {
    if !matches!(value, DescValue::Object { .. }) {
        return None;
    }
    let component = |key: &[u8]| -> Option<u8> {
        match desc_item(value, key) {
            None => Some(0),
            Some(DescValue::Double(c)) => Some((c * 255.0).round().clamp(0.0, 255.0) as u8),
            Some(_) => None,
        }
    };
    Some([
        component(b"Rd  ")?,
        component(b"Grn ")?,
        component(b"Bl  ")?,
    ])
}

pub(crate) fn desc_item<'a>(obj: &'a DescValue, key: &[u8]) -> Option<&'a DescValue> {
    let DescValue::Object { items, .. } = obj else {
        return None;
    };
    items
        .iter()
        .find(|(k, _)| k.as_slice() == key)
        .map(|(_, v)| v)
}

/// Read a `long` item as `f64`; `None` when the key is present but not a long.
fn desc_long_or(obj: &DescValue, key: &[u8], default: f64) -> Option<f64> {
    match desc_item(obj, key) {
        None => Some(default),
        Some(DescValue::Long(n)) => Some(*n as f64),
        Some(_) => None,
    }
}

/// `grdm`: the legacy Gradient Map struct (psd-tools `GradientMap`). Layout:
/// `u16` version (1 or 3), `u8` reverse, `u8` dither, a `4`-byte method when
/// version 3, a unicode name (`u32` UTF-16 char count + data), a `u16` colour
/// stop count, then each stop: `u32` location, `u32` midpoint, `u16` mode, four
/// `u16` colour components, `2` pad bytes. Everything after the colour stops
/// (transparency stops and the trailing gradient fields) is ignored.
///
/// ponytail: the first three components are the only colour read, reduced with
/// `>> 8` so `65535` maps to `255`; midpoint bias, dither, opacity stops, and
/// non-RGB colour models are not modelled.
fn decode_gradient_map(d: &[u8]) -> Option<Adjustment> {
    let version = be_u16(d, 0)?;
    if version != 1 && version != 3 {
        return None;
    }
    let reverse = *d.get(2)? != 0;
    let _dither = *d.get(3)?;
    let mut at = if version == 3 { 8 } else { 4 };
    let name_chars = be_u32(d, at)? as usize;
    at += 4 + name_chars * 2;
    let count = be_u16(d, at)?;
    at += 2;
    if count < 2 {
        return None;
    }
    let mut stops = Vec::with_capacity(count as usize);
    let mut previous: Option<u16> = None;
    for _ in 0..count {
        let location = be_u32(d, at)?;
        if location > 4096 {
            return None;
        }
        let location = location as u16;
        if previous.is_some_and(|p| location <= p) {
            return None;
        }
        previous = Some(location);
        let color = [
            (be_u16(d, at + 10)? >> 8) as u8,
            (be_u16(d, at + 12)? >> 8) as u8,
            (be_u16(d, at + 14)? >> 8) as u8,
        ];
        stops.push(GradientStop { location, color });
        at += 20;
    }
    Some(Adjustment::GradientMap(GradientMapParams {
        stops,
        reverse,
    }))
}

// --- Encoders for the same subset ------------------------------------------
//
// These build the raw `AdjustmentData` the decoder above reads, so the app can
// create adjustment layers in memory. Byte layouts mirror psd-tools' adjustment
// structs (the independent oracle the codec fixtures come from); each is the
// minimal block for the key, not a full re-implementation of Photoshop's writer.

/// `nvrt`: Invert carries no payload.
pub fn encode_invert() -> AdjustmentData {
    AdjustmentData {
        key: *b"nvrt",
        data: Vec::new(),
    }
}

/// `post`: a `u16` levels value (2..=255) plus 2 pad bytes (psd-tools
/// `ShortIntegerElement`, `H2x`). Out-of-range input is clamped.
pub fn encode_posterize(levels: u8) -> AdjustmentData {
    encode_short(*b"post", levels.clamp(2, 255) as u16)
}

/// `thrs`: a `u16` level (1..=255) plus 2 pad bytes. Out-of-range input is clamped.
pub fn encode_threshold(level: u8) -> AdjustmentData {
    encode_short(*b"thrs", level.clamp(1, 255) as u16)
}

/// `brit`: brightness (i16), contrast (i16), mean (i16), lab_only (u8), pad
/// (psd-tools `3HBx`). Inputs are clamped to the decoder's accepted ranges
/// (brightness -150..=150, contrast -50..=100); `use_legacy` is not representable
/// and the decoder always uses the modern curve.
pub fn encode_brightness_contrast(brightness: i16, contrast: i16) -> AdjustmentData {
    let b = brightness.clamp(-150, 150);
    let c = contrast.clamp(-50, 100);
    let mut data = Vec::with_capacity(8);
    data.extend_from_slice(&b.to_be_bytes());
    data.extend_from_slice(&c.to_be_bytes());
    data.extend_from_slice(&0i16.to_be_bytes());
    data.push(0);
    data.push(0);
    AdjustmentData {
        key: *b"brit",
        data,
    }
}

/// `hue2`: version (2), enable (1), pad, colorization (3×i16), then the master
/// Hue/Saturation/Lightness triplet (3×i16), followed by the six per-band range
/// records (6 × 7 i16) that Photoshop stores. The decoder only reads the version
/// and the master triplet; the trailing records are zeroed so the block matches
/// the real 100-byte layout.
pub fn encode_hue_saturation(hue: i16, saturation: i16, lightness: i16) -> AdjustmentData {
    let hue = hue.clamp(-180, 180);
    let saturation = saturation.clamp(-100, 100);
    let lightness = lightness.clamp(-100, 100);
    let mut data = Vec::with_capacity(100);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.push(1);
    data.push(0);
    data.extend_from_slice(&[0u8; 6]);
    data.extend_from_slice(&hue.to_be_bytes());
    data.extend_from_slice(&saturation.to_be_bytes());
    data.extend_from_slice(&lightness.to_be_bytes());
    data.extend_from_slice(&[0u8; 84]);
    AdjustmentData {
        key: *b"hue2",
        data,
    }
}

/// `phfl`: the version-2 fixed struct the decoder reads. `density` clamps to
/// `0..=100`; the colour space and fourth component are zero, and three pad
/// bytes round the block to 20.
pub fn encode_photo_filter(
    color: [u8; 3],
    density: f64,
    preserve_luminosity: bool,
) -> AdjustmentData {
    let density = density.clamp(0.0, 100.0).round() as u32;
    let mut data = Vec::with_capacity(20);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(&0u16.to_be_bytes());
    for c in color {
        data.extend_from_slice(&(c as u16).to_be_bytes());
    }
    data.extend_from_slice(&0u16.to_be_bytes());
    data.extend_from_slice(&density.to_be_bytes());
    data.push(u8::from(preserve_luminosity));
    data.extend_from_slice(&[0u8; 3]);
    AdjustmentData {
        key: *b"phfl",
        data,
    }
}

/// `SoCo`: the standard Photoshop solid-color fill descriptor. `Clr ` is an
/// `RGBC` object carrying the three components as `doub` values on the
/// `0..=255` scale. The descriptor has no alpha, so authored fills are opaque.
pub fn encode_solid_color_fill(color: [u8; 3]) -> AdjustmentData {
    let clr = DescValue::Object {
        name: String::new(),
        class_id: b"RGBC".to_vec(),
        items: [b"Rd  ", b"Grn ", b"Bl  "]
            .into_iter()
            .zip(color)
            .map(|(key, c)| (key.to_vec(), DescValue::Double(c as f64)))
            .collect(),
    };
    let desc = DescValue::Object {
        name: String::new(),
        class_id: b"SoCo".to_vec(),
        items: vec![(b"Clr ".to_vec(), clr)],
    };
    AdjustmentData {
        key: *b"SoCo",
        data: pictura_codec::write_descriptor(&desc),
    }
}

/// `grdm`: the version-1 Gradient Map block. Writes the reverse flag, dither 0,
/// an empty unicode name, the supplied stops (8-bit colours scaled to the
/// 16-bit storage scale), zero transparency stops, and psd-tools' trailing
/// defaults, padded to a 4-byte boundary.
pub fn encode_gradient_map(stops: &[GradientStop], reverse: bool) -> AdjustmentData {
    let mut data = Vec::new();
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(u8::from(reverse));
    data.push(0);
    data.extend_from_slice(&0u32.to_be_bytes()); // empty unicode name
    data.extend_from_slice(&(stops.len() as u16).to_be_bytes());
    for stop in stops {
        data.extend_from_slice(&(stop.location as u32).to_be_bytes());
        data.extend_from_slice(&50u32.to_be_bytes()); // midpoint
        data.extend_from_slice(&0u16.to_be_bytes()); // mode
        for c in stop.color {
            let v = (c as u16) * 257; // inverse of the decoder's `>> 8`
            data.extend_from_slice(&v.to_be_bytes());
        }
        data.extend_from_slice(&0u16.to_be_bytes()); // alpha
        data.extend_from_slice(&[0, 0]); // stop pad
    }
    data.extend_from_slice(&0u16.to_be_bytes()); // transparency stop count
    data.extend_from_slice(&2u16.to_be_bytes()); // expansion
    data.extend_from_slice(&0u16.to_be_bytes()); // interpolation
    data.extend_from_slice(&32u16.to_be_bytes()); // length
    data.extend_from_slice(&0u16.to_be_bytes()); // mode
    data.extend_from_slice(&0u32.to_be_bytes()); // random seed
    data.extend_from_slice(&0u16.to_be_bytes()); // show transparency
    data.extend_from_slice(&0u16.to_be_bytes()); // use vector color
    data.extend_from_slice(&0u32.to_be_bytes()); // roughness
    data.extend_from_slice(&0u16.to_be_bytes()); // color model
    data.extend_from_slice(&[0u8; 16]); // min/max colour (4H each)
    data.extend_from_slice(&[0, 0]); // dummy
    while data.len() % 4 != 0 {
        data.push(0);
    }
    AdjustmentData {
        key: *b"grdm",
        data,
    }
}

/// `H2x` payload: a big-endian `u16` value plus two zero pad bytes.
fn encode_short(key: [u8; 4], value: u16) -> AdjustmentData {
    let mut data = value.to_be_bytes().to_vec();
    data.extend_from_slice(&[0, 0]);
    AdjustmentData { key, data }
}

/// Apply a decoded adjustment to the running backdrop, then gate the result by
/// the layer's mask/opacity/blend (Photoshop applies the adjustment to the
/// backdrop and blends the adjusted result back).
fn composite_adjustment(
    canvas: &mut Canvas,
    layer: &Layer,
    doc: &Document,
    adjustment: &Adjustment,
) {
    // Fill content is generative: it adds color inside the layer's rect instead
    // of transforming the backdrop, so it takes the normal-content path.
    if let Adjustment::SolidFill(rgba) = adjustment {
        crate::fill::composite_solid_fill(canvas, layer, *rgba);
        return;
    }
    if let Adjustment::GradientFill(params) = adjustment {
        crate::fill::composite_gradient_fill(canvas, layer, params);
        return;
    }
    if let Adjustment::PatternFill(params) = adjustment {
        crate::fill::composite_pattern_fill(canvas, layer, doc, params);
        return;
    }
    let n = canvas.w * canvas.h;
    if n == 0 {
        return;
    }
    let mut buf = PixelBuffer::new(canvas.w as u32, canvas.h as u32, 3);
    for (i, p) in canvas.px.iter().enumerate() {
        buf.data[i] = to_u8(p.r);
        buf.data[n + i] = to_u8(p.g);
        buf.data[2 * n + i] = to_u8(p.b);
    }
    if pictura_adjust::apply(adjustment, &mut buf).is_err() {
        return; // invalid/unsupported parameters: no-op, never an error
    }
    for y in 0..canvas.h {
        for x in 0..canvas.w {
            let i = y * canvas.w + x;
            // Source coverage is the backdrop's own alpha: an adjustment adds no
            // content where the backdrop is transparent.
            let backdrop_alpha = canvas.px[i].a;
            if backdrop_alpha <= 0.0 {
                continue;
            }
            let cs = [
                buf.data[i] as f32 / 255.0,
                buf.data[n + i] as f32 / 255.0,
                buf.data[2 * n + i] as f32 / 255.0,
            ];
            blend_into(canvas, layer, x, y, cs, backdrop_alpha);
        }
    }
}

/// Composite one source sample over the running backdrop, applying the layer's
/// opacity, fill (ignored for groups) and mask to the source alpha.
pub(crate) fn blend_into(
    canvas: &mut Canvas,
    layer: &Layer,
    x: usize,
    y: usize,
    cs: [f32; 3],
    src_a: f32,
) {
    let opacity = layer.opacity as f32 / 255.0;
    let fill = if layer.is_group {
        1.0
    } else {
        layer.fill as f32 / 255.0
    };
    let mask = mask_alpha(layer, x as i32, y as i32) as f32 / 255.0;
    blend_parts(canvas, x, y, cs, src_a * opacity * fill * mask, layer.blend);
}

/// Composite one already-opacity-weighted source sample over the running
/// backdrop with the given blend mode. Shared by content and layer effects.
pub(crate) fn blend_parts(
    canvas: &mut Canvas,
    x: usize,
    y: usize,
    cs: [f32; 3],
    alpha: f32,
    mode: BlendMode,
) {
    let mut as_ = alpha;
    if as_ <= 0.0 {
        return;
    }
    let i = y * canvas.w + x;
    let cb = canvas.px[i];

    // Dissolve is stochastic: the effective alpha is a threshold on a fixed
    // per-pixel noise field, and passing pixels go fully opaque.
    // ponytail: deterministic splitmix hash, not Adobe's unpublished noise tile;
    // swap when a CS6 dither baseline exists.
    if matches!(mode, BlendMode::Dissolve) {
        if dissolve_noise(x, y) >= as_ {
            return;
        }
        as_ = 1.0;
    }

    let ab = cb.a;
    let b = blend(mode, [cb.r, cb.g, cb.b], cs);
    let ao = as_ + ab * (1.0 - as_);
    if ao <= 0.0 {
        canvas.px[i] = Px::default();
        return;
    }
    let co = |idx: usize, c: f32| {
        ((1.0 - ab) * as_ * cs[idx] + as_ * ab * b[idx] + (1.0 - as_) * ab * c) / ao
    };
    canvas.px[i] = Px {
        r: co(0, cb.r),
        g: co(1, cb.g),
        b: co(2, cb.b),
        a: ao,
    };
}

pub(crate) fn channel(layer: &Layer, id: i16) -> Option<&[u8]> {
    layer
        .channels
        .iter()
        .find(|c| c.id == id)
        .map(|c| c.data.as_slice())
}

pub(crate) fn sample(data: Option<&[u8]>, idx: usize) -> Option<u8> {
    data.and_then(|d| d.get(idx).copied())
}

/// Mask value at a canvas pixel; `255` when there is no usable mask.
pub(crate) fn mask_alpha(layer: &Layer, x: i32, y: i32) -> u8 {
    let Some(mask) = &layer.mask else {
        return 255;
    };
    if mask.disabled {
        return 255;
    }
    let Some(data) = &mask.data else {
        return 255;
    };
    let mw = mask.rect.width();
    let mh = mask.rect.height();
    if mw <= 0 || mh <= 0 {
        return mask.default_color;
    }
    let mx = x - mask.rect.left;
    let my = y - mask.rect.top;
    if mx < 0 || my < 0 || mx >= mw || my >= mh {
        return mask.default_color;
    }
    let idx = my as usize * mw as usize + mx as usize;
    data.get(idx).copied().unwrap_or(mask.default_color)
}

fn dissolve_noise(x: usize, y: usize) -> f32 {
    let mut z = (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (y as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9)
        ^ 0xD1B5_4A32_D192_ED03;
    z ^= z >> 30;
    z = z.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z ^= z >> 27;
    z = z.wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    ((z >> 56) as f32) / 256.0
}
