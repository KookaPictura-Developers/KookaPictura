//! The bundled-font glyph backend: shape, rasterize, and materialize a type
//! layer into pixels.
//!
//! The engine (`pictura_core`) owns deterministic layout and the `Rasterizer`
//! port; this module is the pure-Rust backend over a bundled Liberation Sans
//! face. No Qt, no C dependency, no GSUB/GPOS shaping (a marked ceiling).

use pictura_core::{
    layout_lines, Channel, Document, GlyphMask, Layer, LayoutParams, PixelBuffer, PsdRect,
    RasterRequest, Rasterizer, ShapedGlyph, TextAlign, TextProvenance, TypeTool,
};

const FONT_BYTES: &[u8] = include_bytes!("../assets/LiberationSans-Regular.ttf");
const RESOLVED_FAMILY: &str = "Liberation Sans";
const BACKEND: &str = "fontdue";
// Literal pin of the `fontdue` version in Cargo.toml; bump with the dependency.
const BACKEND_VERSION: &str = "0.9.4";

/// The bundled face, parsed once.
pub struct BundledText {
    font: fontdue::Font,
}

impl BundledText {
    /// Parse the bundled font; `None` if the embedded bytes fail to parse.
    pub fn new() -> Option<Self> {
        let font = fontdue::Font::from_bytes(FONT_BYTES, fontdue::FontSettings::default()).ok()?;
        Some(Self { font })
    }

    /// A rasterizer borrowing this face.
    pub fn rasterizer(&self) -> BundledRasterizer<'_> {
        BundledRasterizer { font: &self.font }
    }

    /// Shape one line into glyph ids and **device-pixel** advances. Lay the
    /// result out with `LayoutParams.units_per_em == font_size`.
    ///
    /// `_tracking` is ignored here: spacing is the layout's job (design D2).
    pub fn shape_line(&self, text: &str, font_size: f32, _tracking: f32) -> Vec<ShapedGlyph> {
        text.chars()
            .filter(|c| *c != '\n' && *c != '\r')
            .map(|c| {
                let id = self.font.lookup_glyph_index(c);
                let advance = self.font.metrics_indexed(id, font_size).advance_width;
                ShapedGlyph { id, advance }
            })
            .collect()
    }

    /// The ascent at `font_size`, used to place the first baseline inside the
    /// layer rect instead of on its top edge.
    fn ascent(&self, font_size: f32) -> f32 {
        self.font
            .horizontal_line_metrics(font_size)
            .map_or(font_size, |m| m.ascent)
    }

    /// Record the substitution of `requested` by the bundled face.
    pub fn provenance(&self, requested: &str) -> TextProvenance {
        TextProvenance::new(
            requested,
            RESOLVED_FAMILY,
            font_hash(),
            BACKEND,
            BACKEND_VERSION,
        )
    }
}

/// A glyph rasterizer over the bundled face.
pub struct BundledRasterizer<'a> {
    font: &'a fontdue::Font,
}

impl Rasterizer for BundledRasterizer<'_> {
    fn rasterize(&self, request: &RasterRequest) -> Option<GlyphMask> {
        // `rasterize_indexed` indexes an internal vec and panics out of range.
        if request.glyph >= self.font.glyph_count() {
            return None;
        }
        let (metrics, coverage) = self.font.rasterize_indexed(request.glyph, request.px_size);
        if metrics.width == 0 || metrics.height == 0 {
            return None;
        }
        if coverage.len() != metrics.width * metrics.height {
            return None;
        }
        Some(GlyphMask {
            width: metrics.width as u32,
            height: metrics.height as u32,
            left: metrics.xmin,
            top: metrics.ymin,
            coverage,
        })
    }
}

/// A deterministic non-cryptographic FNV-1a digest of the font bytes, padded to
/// a 32-byte identity.
//
// ponytail: not a content/security digest; swap for a real hash if documents
// ever pin a font by digest.
fn font_hash() -> [u8; 32] {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &byte in FONT_BYTES {
        h ^= byte as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    let mut out = [0u8; 32];
    for (i, chunk) in out.chunks_mut(8).enumerate() {
        chunk.copy_from_slice(&h.wrapping_mul(i as u64 + 1).to_le_bytes());
    }
    out
}

/// The bundled face parsed once per process: compositing a proxy-less type
/// layer must not reparse the ~400 KB font every frame.
fn bundled() -> Option<&'static BundledText> {
    use std::sync::OnceLock;
    static CACHE: OnceLock<Option<BundledText>> = OnceLock::new();
    CACHE.get_or_init(BundledText::new).as_ref()
}

/// Replace the layer at `path`'s `0/1/2/-1` channels from a packed straight-alpha
/// RGBA buffer of the layer-rect size, drop `TySh`, and clear the type tool.
/// Returns false without mutating for a missing layer, a non-positive rect, or a
/// buffer that is not `width * height * 4` bytes.
pub fn materialize_text_rgba(doc: &mut Document, path: &str, rgba: &[u8]) -> bool {
    let Some(layer) = crate::resolve_path(doc, path) else {
        return false;
    };
    let width = layer.rect.width();
    let height = layer.rect.height();
    if width <= 0 || height <= 0 {
        return false;
    }
    let Some(expected) = (width as usize * height as usize).checked_mul(4) else {
        return false;
    };
    if rgba.len() != expected {
        return false;
    }
    let plane = expected / 4;
    let Some(layer) = crate::resolve_path_mut(doc, path) else {
        return false;
    };
    let mut r = Vec::with_capacity(plane);
    let mut g = Vec::with_capacity(plane);
    let mut b = Vec::with_capacity(plane);
    let mut a = Vec::with_capacity(plane);
    for px in rgba.as_chunks::<4>().0 {
        r.push(px[0]);
        g.push(px[1]);
        b.push(px[2]);
        a.push(px[3]);
    }
    layer.channels = vec![
        Channel { id: 0, data: r },
        Channel { id: 1, data: g },
        Channel { id: 2, data: b },
        Channel { id: -1, data: a },
    ];
    layer.extra_blocks.retain(|block| &block.key != b"TySh");
    layer.type_tool = None;
    true
}

/// Shape, lay out, and paint a type layer over its rect, then drop the type
/// tool: the layer becomes raster. Returns false without mutating when there is
/// no type tool or style, the rect has no area, or no glyph rendered.
pub fn render_text_layer(doc: &mut Document, path: &str) -> bool {
    let Some(layer) = crate::resolve_path(doc, path) else {
        return false;
    };
    let Some(type_tool) = layer.type_tool.as_ref() else {
        return false;
    };
    let Some(buffer) = render_text_buffer(type_tool, layer.rect.width(), layer.rect.height())
    else {
        return false;
    };
    let plane = buffer.pixel_count();
    let mut rgba = vec![0u8; plane * 4];
    for i in 0..plane {
        rgba[4 * i] = buffer.data[i];
        rgba[4 * i + 1] = buffer.data[plane + i];
        rgba[4 * i + 2] = buffer.data[2 * plane + i];
        rgba[4 * i + 3] = buffer.data[3 * plane + i];
    }
    materialize_text_rgba(doc, path, &rgba)
}

/// Shape, lay out, and paint a type tool into a straight-alpha RGBA buffer of
/// `width × height` (layer-rect-local). `None` when the style is absent, the
/// size is non-positive, or no glyph painted a pixel.
pub(crate) fn render_text_buffer(
    type_tool: &TypeTool,
    width: i32,
    height: i32,
) -> Option<PixelBuffer> {
    let style = type_tool.style.as_ref()?;
    if width <= 0 || height <= 0 {
        return None;
    }
    let font_size = style.font_size as f32;
    let tracking = style.tracking as f32;
    let color = style.fill_color;
    let align = TextAlign::from_justification(style.justification);

    let bundled = bundled()?;
    let lines: Vec<Vec<ShapedGlyph>> = type_tool
        .text
        .lines()
        .map(|line| bundled.shape_line(line, font_size, tracking))
        .collect();
    let layout = layout_lines(
        &lines,
        &LayoutParams {
            font_size,
            units_per_em: font_size,
            tracking,
            leading: font_size * 1.2,
            align,
            wrap_width: Some(width as f32),
        },
    );
    if layout.lines.iter().all(|line| line.glyphs.is_empty()) {
        return None;
    }

    let n = width as usize * height as usize;
    let mut out = PixelBuffer::new(width as u32, height as u32, 4);

    let rasterizer = bundled.rasterizer();
    let baseline_shift = bundled.ascent(font_size);
    let mut painted = false;
    for line in &layout.lines {
        let baseline = (line.baseline + baseline_shift).round() as i32;
        for glyph in &line.glyphs {
            let Some(mask) = rasterizer.rasterize(&RasterRequest {
                glyph: glyph.id,
                px_size: font_size,
                subpixel_x: 0.0,
                subpixel_y: 0.0,
            }) else {
                continue;
            };
            let pen_x = glyph.x.round() as i32 + mask.left;
            let top = baseline - (mask.height as i32 + mask.top);
            for row in 0..mask.height {
                let y = top + row as i32;
                if y < 0 || y >= height {
                    continue;
                }
                for col in 0..mask.width {
                    let x = pen_x + col as i32;
                    if x < 0 || x >= width {
                        continue;
                    }
                    let coverage = mask.coverage[(row * mask.width + col) as usize];
                    if coverage == 0 {
                        continue;
                    }
                    let index = y as usize * width as usize + x as usize;
                    out.data[index] = tint(coverage, color[0]);
                    out.data[n + index] = tint(coverage, color[1]);
                    out.data[2 * n + index] = tint(coverage, color[2]);
                    out.data[3 * n + index] = out.data[3 * n + index].max(coverage);
                    painted = true;
                }
            }
        }
    }
    if !painted {
        return None;
    }
    Some(out)
}

/// Composite a proxy-less type layer live: render its text buffer and blend it
/// over the canvas-clipped region with the layer's opacity/mask/blend. `false`
/// when there is no type tool, a colour channel is present, or nothing painted,
/// so the caller's normal path runs.
///
/// ponytail: axis-aligned placement at the layer rect, first-run style, no
/// kerning — inherited from the bundled renderer; add transform/rotation when
/// the type model carries one.
pub(crate) fn composite_type_source(canvas: &mut crate::composite::Canvas, layer: &Layer) -> bool {
    let Some(type_tool) = layer.type_tool.as_ref() else {
        return false;
    };
    if crate::channel(layer, 0).is_some() {
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
    let Some(buf) = render_text_buffer(type_tool, layer.rect.width(), layer.rect.height()) else {
        return false;
    };
    let lw = layer.rect.width() as usize;
    let plane = buf.pixel_count();
    let rw = region.width() as usize;
    let rh = region.height() as usize;
    for by in 0..rh {
        for bx in 0..rw {
            let lx = (region.left - layer.rect.left) as usize + bx;
            let ly = (region.top - layer.rect.top) as usize + by;
            let i = ly * lw + lx;
            crate::composite::blend_into(
                canvas,
                layer,
                region.left as usize + bx,
                region.top as usize + by,
                [
                    buf.data[i] as f32 / 255.0,
                    buf.data[plane + i] as f32 / 255.0,
                    buf.data[2 * plane + i] as f32 / 255.0,
                ],
                buf.data[3 * plane + i] as f32 / 255.0,
            );
        }
    }
    true
}

/// `coverage/255 * channel` as an 8-bit straight colour sample.
fn tint(coverage: u8, channel: f64) -> u8 {
    (coverage as f64 / 255.0 * channel * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use pictura_core::{BitDepth, ColorMode, Layer, LayerBlock, PsdRect, TextStyle, TypeTool};

    fn bundled() -> BundledText {
        BundledText::new().expect("bundled font parses")
    }

    fn type_layer(style: Option<TextStyle>) -> Layer {
        Layer {
            name: "text".into(),
            rect: PsdRect {
                top: 0,
                left: 0,
                bottom: 80,
                right: 200,
            },
            extra_blocks: vec![LayerBlock {
                key: *b"TySh",
                data: vec![1, 2, 3],
            }],
            type_tool: Some(TypeTool {
                transform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                text: "Hi".into(),
                bounds: [0, 0, 80, 200],
                text_desc: Vec::new(),
                warp_desc: Vec::new(),
                fonts: vec!["Arial".into()],
                style,
            }),
            ..Default::default()
        }
    }

    fn style() -> TextStyle {
        TextStyle {
            font: Some("Arial".into()),
            font_size: 48.0,
            fill_color: [0.0, 0.0, 0.0, 1.0],
            tracking: 0.0,
            justification: 0,
        }
    }

    fn doc_with(layer: Layer) -> Document {
        let mut doc = Document::new(200, 80, ColorMode::Rgb, BitDepth::Eight);
        doc.layers = vec![layer];
        doc
    }

    #[test]
    fn rasterizer_returns_coverage_of_width_times_height() {
        let bundled = bundled();
        let index = bundled.font.lookup_glyph_index('A');
        assert_ne!(index, 0, "'A' is in Liberation Sans");
        let mask = bundled
            .rasterizer()
            .rasterize(&RasterRequest {
                glyph: index,
                px_size: 48.0,
                subpixel_x: 0.0,
                subpixel_y: 0.0,
            })
            .expect("'A' rasterizes");
        assert_seeded(&mask);
    }

    fn assert_seeded(mask: &GlyphMask) {
        assert_eq!(mask.coverage.len(), (mask.width * mask.height) as usize);
        assert!(
            mask.coverage.iter().any(|&c| c > 0),
            "coverage is non-empty"
        );
    }

    #[test]
    fn rasterizer_refuses_an_absent_index_without_panicking() {
        let bundled = bundled();
        let mask = bundled.rasterizer().rasterize(&RasterRequest {
            glyph: u16::MAX,
            px_size: 48.0,
            subpixel_x: 0.0,
            subpixel_y: 0.0,
        });
        assert!(mask.is_none());
    }

    #[test]
    fn shape_line_maps_two_glyphs_with_nonzero_advances() {
        let glyphs = bundled().shape_line("AB", 48.0, 0.0);
        assert_eq!(glyphs.len(), 2);
        assert!(glyphs.iter().all(|g| g.advance > 0.0));
    }

    #[test]
    fn render_text_layer_paints_and_clears_the_type_tool() {
        let mut doc = doc_with(type_layer(Some(style())));
        assert!(render_text_layer(&mut doc, "0"));
        let layer = crate::resolve_path(&doc, "0").unwrap();
        assert!(layer.type_tool.is_none(), "type tool is cleared");
        assert!(
            layer.extra_blocks.iter().all(|b| &b.key != b"TySh"),
            "TySh block is gone"
        );
        let alpha = crate::channel(layer, -1).expect("alpha channel");
        assert!(alpha.iter().any(|&a| a > 0), "non-empty alpha coverage");
        assert_eq!(alpha.len(), 200 * 80);
    }

    #[test]
    fn render_text_layer_refuses_a_missing_style_without_mutating() {
        let mut doc = doc_with(type_layer(None));
        let before = doc.clone();
        assert!(!render_text_layer(&mut doc, "0"));
        assert_eq!(doc, before, "refusal leaves the document unchanged");
    }

    #[test]
    fn materialize_text_rgba_refuses_a_wrong_size_buffer_without_mutating() {
        let mut doc = doc_with(type_layer(Some(style())));
        let before = doc.clone();
        assert!(!materialize_text_rgba(&mut doc, "0", &[0u8; 10]));
        assert_eq!(doc, before, "refusal leaves the document unchanged");
    }

    #[test]
    fn materialize_text_rgba_refuses_a_missing_layer() {
        let mut doc = doc_with(type_layer(Some(style())));
        assert!(!materialize_text_rgba(&mut doc, "9", &[]));
    }

    #[test]
    fn materialize_text_rgba_sets_channels_and_clears_the_type_tool() {
        let mut doc = doc_with(type_layer(Some(style())));
        let n = 200 * 80;
        let mut rgba = vec![0u8; n * 4];
        rgba[0] = 10;
        rgba[1] = 20;
        rgba[2] = 30;
        rgba[3] = 40;
        assert!(materialize_text_rgba(&mut doc, "0", &rgba));
        let layer = crate::resolve_path(&doc, "0").unwrap();
        assert!(layer.type_tool.is_none(), "type tool is cleared");
        assert!(
            layer.extra_blocks.iter().all(|b| &b.key != b"TySh"),
            "TySh block is gone"
        );
        assert_eq!(crate::channel(layer, 0).unwrap()[0], 10);
        assert_eq!(crate::channel(layer, 1).unwrap()[0], 20);
        assert_eq!(crate::channel(layer, 2).unwrap()[0], 30);
        assert_eq!(crate::channel(layer, -1).unwrap()[0], 40);
    }

    #[test]
    fn provenance_records_the_bundled_substitution() {
        let provenance = bundled().provenance("Arial");
        assert_eq!(provenance.requested_family, "Arial");
        assert_eq!(provenance.resolved_family, "Liberation Sans");
        assert_eq!(provenance.backend, "fontdue");
        assert_ne!(provenance.font_hash, [0u8; 32]);
    }
}
