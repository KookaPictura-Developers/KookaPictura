use pictura_adjust::{
    Adjustment, BlackWhiteParams, BrightnessContrastParams, ExposureParams, GradientMapParams,
    GradientStop, HueSaturationParams, LevelsParams, PhotoFilterParams, VibranceParams,
};
use pictura_codec::DescValue;
use pictura_core::{
    AdjustmentData, BlendIf, BlendMode, ColorMode, Document, Knockout, Layer, PixelBuffer, PsdRect,
    SmartObject, SmartObjectKind,
};

pub(crate) use crate::blend::blend;

/// Composite the document's layer stack.
///
/// Returns a 4-channel (R,G,B,A) planar, straight-alpha, 8-bit buffer at
/// document resolution.
pub fn composite_rgba(doc: &Document) -> PixelBuffer {
    let mut canvas = Canvas::new(doc.width as usize, doc.height as usize);
    composite_layers(&mut canvas, doc);
    canvas.into_pixel_buffer()
}

/// Composite `doc.layers` (bottom-first) onto `canvas`, applying each non-bottom
/// layer's knockout against the document background.
fn composite_layers(canvas: &mut Canvas, doc: &Document) {
    let base = knockout_base(canvas, doc);
    for (i, layer) in doc.layers.iter().enumerate() {
        let base = if i == 0 { None } else { base.as_ref() };
        composite_layer(canvas, layer, doc, base);
    }
}

/// The document background (the bottom layer composited alone), built only when
/// a non-bottom layer actually knocks out.
///
/// ponytail: the model has no Background flag, so the bottom layer is assumed to
/// be the background; a non-background bottom resolves to its content rather
/// than transparency. A knockout that is the bottom layer composites as a plain
/// layer (no base).
fn knockout_base(region: &Canvas, doc: &Document) -> Option<Canvas> {
    let present = doc
        .layers
        .iter()
        .skip(1)
        .any(|l| l.knockout != Knockout::None);
    present.then(|| {
        let mut base = Canvas::new_region(region.ox, region.oy, region.w, region.h);
        if let Some(background) = doc.layers.first() {
            composite_layer_inner(&mut base, background, doc);
        }
        base
    })
}

/// Composite only the document-space region `[x0, x0+rw) × [y0, y0+rh)`.
///
/// Byte-identical to the same slice of [`composite_rgba`] for every stack whose
/// only neighborhood operations are layer effects; such a stack falls back to
/// the full composite plus a slice.
///
/// ponytail: object-based layer effects (blur/offset/shape neighborhoods) still
/// full-composite and slice. Per-pixel layers (pixels, fills, adjustments,
/// smart sources, masks, groups) composite the region alone, which is the brush
/// path; add a region-expanded effect kernel only if an effect-bearing paint
/// layer shows up in a profile.
pub(crate) fn composite_rgba_region(
    doc: &Document,
    x0: u32,
    y0: u32,
    rw: u32,
    rh: u32,
) -> PixelBuffer {
    if rw == 0 || rh == 0 {
        return PixelBuffer::new(0, 0, 4);
    }
    if doc.layers.iter().any(has_effect_block) {
        return slice_region(&composite_rgba(doc), doc.width, x0, y0, rw, rh);
    }
    let mut canvas = Canvas::new_region(x0 as i32, y0 as i32, rw as usize, rh as usize);
    composite_layers(&mut canvas, doc);
    canvas.into_pixel_buffer()
}

/// Whether the layer (or a descendant) carries a layer-effects block, which the
/// region compositor cannot restrict to a rectangle. Conservative: a disabled
/// effect still forces the full-composite fallback.
fn has_effect_block(layer: &Layer) -> bool {
    layer.extra_block(b"lfx2").is_some()
        || layer.extra_block(b"lrFX").is_some()
        || layer.children.iter().any(has_effect_block)
}

/// Copy the region `[x0, x0+rw) × [y0, y0+rh)` out of a full-document buffer.
fn slice_region(
    full: &PixelBuffer,
    full_width: u32,
    x0: u32,
    y0: u32,
    rw: u32,
    rh: u32,
) -> PixelBuffer {
    let fw = full_width as usize;
    let fplane = full.pixel_count();
    let rw = rw as usize;
    let rh = rh as usize;
    let rplane = rw * rh;
    let mut out = PixelBuffer::new(rw as u32, rh as u32, 4);
    for ry in 0..rh {
        let frow = (y0 as usize + ry) * fw + x0 as usize;
        let rrow = ry * rw;
        for rx in 0..rw {
            let f = frow + rx;
            let r = rrow + rx;
            for c in 0..4 {
                out.data[c * rplane + r] = full.data[c * fplane + f];
            }
        }
    }
    out
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
    /// Document x of canvas column 0.
    pub(crate) ox: i32,
    /// Document y of canvas row 0.
    pub(crate) oy: i32,
    px: Vec<Px>,
    /// Per-pixel "the layer contributed here" flag, set only for the temporary
    /// canvas a knockout layer composites into. `None` on the output canvas so
    /// the normal path stays allocation-free.
    cover: Option<Vec<bool>>,
}

impl Canvas {
    pub(crate) fn new(w: usize, h: usize) -> Self {
        Self::new_region(0, 0, w, h)
    }

    /// A canvas covering document-space `[ox, ox+w) × [oy, oy+h)`.
    pub(crate) fn new_region(ox: i32, oy: i32, w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            ox,
            oy,
            px: vec![Px::default(); w * h],
            cover: None,
        }
    }

    /// A copy of `base`'s pixels with a fresh all-false coverage mask, so a
    /// knockout composite can record which pixels it actually covered.
    fn with_cover_from(base: &Canvas) -> Canvas {
        Canvas {
            w: base.w,
            h: base.h,
            ox: base.ox,
            oy: base.oy,
            px: base.px.clone(),
            cover: Some(vec![false; base.px.len()]),
        }
    }

    /// Document bounds of the canvas.
    pub(crate) fn x0(&self) -> i32 {
        self.ox
    }
    pub(crate) fn y0(&self) -> i32 {
        self.oy
    }
    pub(crate) fn x1(&self) -> i32 {
        self.ox + self.w as i32
    }
    pub(crate) fn y1(&self) -> i32 {
        self.oy + self.h as i32
    }

    /// The accumulator index of document pixel `(x, y)`.
    fn idx(&self, x: usize, y: usize) -> usize {
        debug_assert!((x as i32) >= self.ox && (y as i32) >= self.oy);
        (y as i32 - self.oy) as usize * self.w + (x as i32 - self.ox) as usize
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

/// Composite `layer` onto the running `canvas`. A non-`None` knockout with a
/// `base` (the document background) routes through [`composite_knockout`];
/// everything else is the plain inner dispatch.
fn composite_layer(canvas: &mut Canvas, layer: &Layer, doc: &Document, base: Option<&Canvas>) {
    match base {
        Some(base) if layer.knockout != Knockout::None => {
            composite_knockout(canvas, layer, doc, base);
        }
        _ => composite_layer_inner(canvas, layer, doc),
    }
}

/// Composite `layer` against the document background `base`, then replace the
/// running backdrop only where the layer contributed, punching the layers
/// between it and the background through at those pixels.
///
/// ponytail: the per-pixel mechanism is inferred from the documented
/// shape-composited-against-the-stopping-point rule
/// (`docs/05-layers/layers-overview.md:181`); there is no Photoshop oracle. A
/// shallow stopping point inside a nested group and the clipping-mask base are
/// not resolved (a knockout inside a group is inert), `Transparency Shapes
/// Layers` (restricting coverage to the content's opaque pixels) is not applied,
/// and the bottom layer is assumed to be the background.
fn composite_knockout(canvas: &mut Canvas, layer: &Layer, doc: &Document, base: &Canvas) {
    let mut tmp = Canvas::with_cover_from(base);
    composite_layer_inner(&mut tmp, layer, doc);
    let cover = tmp.cover.take().unwrap_or_default();
    for (i, covered) in cover.iter().enumerate() {
        if *covered {
            canvas.px[i] = tmp.px[i];
        }
    }
}

fn composite_layer_inner(canvas: &mut Canvas, layer: &Layer, doc: &Document) {
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
                composite_layer(canvas, child, doc, None);
            }
            return;
        }
        let mut inner = Canvas::new_region(canvas.ox, canvas.oy, canvas.w, canvas.h);
        for child in &layer.children {
            composite_layer(&mut inner, child, doc, None);
        }
        composite_canvas(canvas, layer, &inner);
    } else if let Some(adjustment) = crate::fill::decode_layer_fill(layer) {
        composite_adjustment(canvas, layer, doc, &adjustment);
    } else if layer.adjustment.is_none()
        && !composite_smart_source(canvas, layer, layer.smart_object.as_ref())
        && !crate::text_render::composite_type_source(canvas, layer)
    {
        // A layer with no decoded fill but an adjustment block is a no-op here
        // (the guard above). With no adjustment block a channel-less layer (for
        // example a shape whose `vscg` did not decode) falls through to
        // `composite_pixels`, which paints an opaque black rect over the layer
        // rect: the "vscg does not change the backdrop" guarantee is relative to
        // the bare channel-less layer. ponytail: pre-existing channel-less
        // behavior, left as is; gate it when a fill-less shape layer needs to be
        // truly transparent.
        composite_pixels(canvas, layer, doc);
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
    let x0 = layer.rect.left.max(canvas.x0());
    let y0 = layer.rect.top.max(canvas.y0());
    let x1 = layer.rect.right.min(canvas.x1());
    let y1 = layer.rect.bottom.min(canvas.y1());
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
        top: layer.rect.top.max(canvas.y0()),
        left: layer.rect.left.max(canvas.x0()),
        bottom: layer.rect.bottom.min(canvas.y1()),
        right: layer.rect.right.min(canvas.x1()),
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
    for y in canvas.y0()..canvas.y1() {
        for x in canvas.x0()..canvas.x1() {
            let p = inner.px[inner.idx(x as usize, y as usize)];
            if p.a > 0.0 {
                blend_into(canvas, layer, x as usize, y as usize, [p.r, p.g, p.b], p.a);
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
/// (Vibrance), `blwh` (Black & White), `phfl` (Photo Filter, versions 2/3),
/// `grdm` (Gradient Map, versions 1/3), `blnc` (Color Balance), `mixr`
/// (Channel Mixer), `curv` (Curves), `selc` (Selective Color), `SoCo`
/// (solid-color fill content, either the 4-byte in-house RGBA tuple or the
/// standard Photoshop descriptor), `GdFl` (gradient fill content), `PtFl`
/// (pattern fill content), both fill descriptors, and `clrL` (Color Lookup,
/// whose embedded `.CUBE` is parsed when the kind is a 3-D LUT).
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
        b"curv" => crate::curves::decode_curves(&data.data),
        b"selc" => crate::selective_color::decode_selective_color(&data.data),
        b"clrL" => crate::color_lookup::decode_color_lookup(&data.data),
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
pub(crate) fn decode_solid_fill(d: &[u8]) -> Option<Adjustment> {
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

/// `phfl` version 2: `u16` version, `u16` colour space (ignored), four `u16`
/// colour components (first three = R, G, B; fourth ignored), `u32` density,
/// `u8` luminosity. A colour space other than RGB is decoded as RGB.
/// Version 3: three `be_u32` CIE XYZ values (offsets 2/6/10), `u32` density
/// (14), `u8` luminosity (18).
///
/// ponytail: the version-2 block length is flexible — only the 17 field bytes
/// are read, so an unpadded 17–19 byte payload still decodes while 20-byte
/// writers round-trip.
fn decode_photo_filter(d: &[u8]) -> Option<Adjustment> {
    let (color, density, preserve_luminosity) = match be_u16(d, 0)? {
        2 => {
            let component = |at: usize| -> Option<u8> {
                let v = be_u16(d, at)?;
                (v <= 255).then_some(v as u8)
            };
            (
                [component(4)?, component(6)?, component(8)?],
                be_u32(d, 12)?,
                *d.get(16)? != 0,
            )
        }
        3 => (
            xyz_d50_to_srgb_u8([be_u32(d, 2)?, be_u32(d, 6)?, be_u32(d, 10)?]),
            be_u32(d, 14)?,
            *d.get(18)? != 0,
        ),
        _ => return None,
    };
    if density > 100 {
        return None;
    }
    Some(Adjustment::PhotoFilter(PhotoFilterParams {
        color,
        density: density as f64,
        preserve_luminosity,
    }))
}

// ponytail: 16.16 scale and D50 white are unproven without a CS6 v3 fixture;
// layout grounded on psd-tools. If a real file disagrees, only the scale
// constant / white adaptation changes, not the field layout.
fn xyz_d50_to_srgb_u8(xyz: [u32; 3]) -> [u8; 3] {
    let scale = |v: u32| v as f64 / 65536.0;
    pictura_codec::xyz_d50_to_srgb_u8([scale(xyz[0]), scale(xyz[1]), scale(xyz[2])])
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
    for y in canvas.y0()..canvas.y1() {
        for x in canvas.x0()..canvas.x1() {
            let i = canvas.idx(x as usize, y as usize);
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
            blend_into(canvas, layer, x as usize, y as usize, cs, backdrop_alpha);
        }
    }
}

/// The Blend If weight for a source colour over a running backdrop: `1.0` when
/// the layer carries no non-default ranges, else the product of the active
/// composite and per-channel gates (each `0` or `1`).
///
/// ponytail: Rec.601 composite gray and a hard 0/1 gate; Photoshop's weighting
/// and split-slider feather are unpublished and the model stores no feather.
/// ponytail: channel groups are assumed R,G,B in order; the model does not label
/// them, and groups beyond RGB are ignored.
pub(crate) fn blend_if_factor(view: Option<&BlendIf>, cs: [f32; 3], backdrop: [f32; 3]) -> f32 {
    let Some(view) = view else {
        return 1.0;
    };
    if view.is_default() {
        return 1.0;
    }
    let gate = |(black, white): (u16, u16), value: f32| {
        if (black, white) == (0, 65535) || black > white {
            return 1.0;
        }
        let lo = black as f32 / 65535.0;
        let hi = white as f32 / 65535.0;
        if value <= lo || value >= hi {
            0.0
        } else {
            1.0
        }
    };
    let gray = |c: [f32; 3]| 0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2];
    let mut factor =
        gate(view.composite_source, gray(cs)) * gate(view.composite_dest, gray(backdrop));
    for (i, (source, dest)) in view.channel_ranges.iter().enumerate().take(3) {
        factor *= gate(*source, cs[i]) * gate(*dest, backdrop[i]);
    }
    factor
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
    let gate = match layer.blend_if.as_ref() {
        Some(view) if !view.is_default() => {
            let backdrop = canvas.px[canvas.idx(x, y)];
            blend_if_factor(Some(view), cs, [backdrop.r, backdrop.g, backdrop.b])
        }
        _ => 1.0,
    };
    blend_parts(
        canvas,
        x,
        y,
        cs,
        src_a * opacity * fill * mask * gate,
        layer.blend,
    );
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
    let i = canvas.idx(x, y);
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
    if let Some(cover) = canvas.cover.as_mut() {
        cover[i] = true;
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
///
/// The raster and vector coverages multiply, so either can suppress the layer;
/// an absent or disabled part contributes `255`.
pub(crate) fn mask_alpha(layer: &Layer, x: i32, y: i32) -> u8 {
    let raster = raster_mask_alpha(layer, x, y);
    let vector = crate::vector_mask::coverage(layer.vector_mask.as_ref(), x, y);
    ((raster as u16 * vector as u16 + 127) / 255) as u8
}

/// The raster layer-mask sample at a canvas pixel; `255` when absent/disabled.
fn raster_mask_alpha(layer: &Layer, x: i32, y: i32) -> u8 {
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
