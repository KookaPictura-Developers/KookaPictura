//! The Type tools' engine: place point type, commit it as a type layer, or
//! render its coverage for the Type Mask tools.
//!
//! Point type is laid out from the click (`TypeSpec::origin`). Horizontal type
//! sets lines `1.2 × size` apart, the click on the first baseline; vertical
//! type stacks each line's characters top to bottom in a column, columns
//! running right to left, the click on the first column's centre axis. A
//! committed layer carries both its rendered pixels and an authored `TySh`, so
//! it composites like any pixel layer and reopens as type. Glyphs come from
//! the face registered for the spec's family (`fonts`), else the bundled face;
//! `TySh` names the face's PostScript name. Ported
//! from photorust's Type tool (`CanvasView` typing / commit).
//!
//! ponytail: vertical type sets every character upright on a fixed
//! `ascent + descent` cell — no `vert` alternates, rotated Latin, or tate-chu-yoko.

use pictura_core::{
    BitDepth, Channel, ColorMode, Document, LayerBlock, LayoutLine, LockFlags, PixelBuffer,
    PlacedGlyph, PsdRect, RasterRequest, TextLayout, TextStyle, TypeSpec, TypeTool,
};

use crate::text_render::{blit_glyph, layout_type, render_text_buffer, BundledText};

/// The lines of a `TySh` string: `\r` separates them (`\n` and `\r\n` are
/// accepted too).
pub(crate) fn type_lines(text: &str) -> Vec<String> {
    text.replace("\r\n", "\r")
        .split(['\r', '\n'])
        .map(str::to_owned)
        .collect()
}

/// 0 left / top, 1 right / bottom, 2 centre, as a fraction of the slack.
pub(crate) fn align_factor(justification: u8) -> f32 {
    match justification {
        1 => 1.0,
        2 => 0.5,
        _ => 0.0,
    }
}

/// Lay out vertical type inside a `width × height` box: column `j` is centred
/// `leading × (j + ½)` from the right edge, and its characters fall one cell
/// apart, the column aligned top / bottom / centre by `justification`.
pub(crate) fn vertical_layout(
    bundled: &BundledText,
    lines: &[String],
    font_size: f32,
    justification: u8,
    width: f32,
    height: f32,
) -> TextLayout {
    let leading = font_size * 1.2;
    let cell = bundled.ascent(font_size) + bundled.descent(font_size);
    let factor = align_factor(justification);
    let columns = lines
        .iter()
        .enumerate()
        .map(|(j, line)| {
            let centre = width - leading * (j as f32 + 0.5);
            let count = line.chars().count() as f32;
            let top = (height - count * cell) * factor;
            let glyphs = line
                .chars()
                .enumerate()
                .flat_map(|(k, c)| {
                    let shaped = bundled.shape_line(&c.to_string(), font_size, 0.0);
                    let advance: f32 = shaped.iter().map(|g| g.advance).sum();
                    let mut x = centre - advance / 2.0;
                    let y = top + k as f32 * cell;
                    shaped.into_iter().map(move |g| {
                        let placed = PlacedGlyph {
                            id: g.id,
                            x: x + g.x_offset,
                            y: y - g.y_offset,
                        };
                        x += g.advance;
                        placed
                    })
                })
                .collect();
            LayoutLine {
                glyphs,
                advance: count * cell,
                baseline: centre,
            }
        })
        .collect();
    TextLayout {
        lines: columns,
        width,
        height,
    }
}

/// Where point type lands: its layer rect in document pixels, and its box
/// relative to the origin (left, top, right, bottom) for the `TySh` bounds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypePlacement {
    pub rect: PsdRect,
    pub bounds: [f64; 4],
}

/// `spec`'s untransformed text box in document pixels: left, top, width,
/// height. `None` for empty text, a non-positive size, or a missing face.
fn text_box(spec: &TypeSpec) -> Option<(f32, f32, f32, f32)> {
    if type_lines(&spec.text)
        .iter()
        .all(|line| line.trim().is_empty())
    {
        return None;
    }
    text_box_any(spec)
}

/// [`text_box`] for any text, blank included (the caret still needs a box).
pub(crate) fn text_box_any(spec: &TypeSpec) -> Option<(f32, f32, f32, f32)> {
    let bundled = crate::fonts::face_for(&spec.font)?;
    let size = spec.size as f32;
    let lines = type_lines(&spec.text);
    if size.is_nan() || size <= 0.0 {
        return None;
    }
    let ascent = bundled.ascent(size);
    let descent = bundled.descent(size);
    let leading = size * 1.2;
    let factor = align_factor(spec.justification);
    let (x, y) = (spec.origin.0 as f32, spec.origin.1 as f32);
    let (left, top, width, height) = if spec.vertical {
        let longest = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        let width = lines.len() as f32 * leading;
        let height = longest as f32 * (ascent + descent);
        (
            x - (width - leading / 2.0),
            y - height * factor,
            width,
            height,
        )
    } else {
        let width = lines
            .iter()
            .map(|line| {
                bundled
                    .shape_line(line, size, 0.0)
                    .iter()
                    .map(|g| g.advance)
                    .sum::<f32>()
            })
            .fold(0.0f32, f32::max);
        let height = (lines.len() - 1) as f32 * leading + ascent + descent;
        (x - width * factor, y - ascent, width, height)
    };
    Some((left, top, width, height))
}

/// The untransformed layer rect of a text box: one spare column on the right
/// keeps a glyph's overhang past its advance.
pub(crate) fn flat_rect(left: f32, top: f32, width: f32, height: f32) -> PsdRect {
    PsdRect {
        top: top.floor() as i32,
        left: left.floor() as i32,
        bottom: (top + height).ceil() as i32,
        right: (left + width).ceil() as i32 + 1,
    }
}

/// `(u, v)` relative to the origin, mapped by `spec.matrix` and moved to it.
pub(crate) fn map(spec: &TypeSpec, u: f64, v: f64) -> (f64, f64) {
    let [xx, xy, yx, yy] = spec.matrix;
    (
        spec.origin.0 + xx * u + yx * v,
        spec.origin.1 + xy * u + yy * v,
    )
}

/// Place `spec`: the layer rect bounds the text box mapped by `spec.matrix`
/// about the origin. `None` for empty text, a non-positive size, or a missing
/// face.
pub fn type_placement(spec: &TypeSpec) -> Option<TypePlacement> {
    let (left, top, width, height) = text_box(spec)?;
    let (x, y) = (spec.origin.0 as f32, spec.origin.1 as f32);
    let bounds = [
        f64::from(left - x),
        f64::from(top - y),
        f64::from(left + width - x),
        f64::from(top + height - y),
    ];
    if spec.matrix != TypeSpec::IDENTITY {
        let corners = [
            map(spec, bounds[0], bounds[1]),
            map(spec, bounds[2], bounds[1]),
            map(spec, bounds[2], bounds[3]),
            map(spec, bounds[0], bounds[3]),
        ];
        let lo = |f: fn(&(f64, f64)) -> f64| corners.iter().map(f).fold(f64::MAX, f64::min);
        let hi = |f: fn(&(f64, f64)) -> f64| corners.iter().map(f).fold(f64::MIN, f64::max);
        let rect = PsdRect {
            top: lo(|c| c.1).floor() as i32,
            left: lo(|c| c.0).floor() as i32,
            bottom: hi(|c| c.1).ceil() as i32 + 1,
            right: hi(|c| c.0).ceil() as i32 + 1,
        };
        return Some(TypePlacement { rect, bounds });
    }
    Some(TypePlacement {
        rect: flat_rect(left, top, width, height),
        bounds,
    })
}

/// The unauthored view the renderer needs: `spec`'s style without descriptors.
fn render_view(spec: &TypeSpec) -> TypeTool {
    let [r, g, b, a] = spec.color.map(|c| f64::from(c) / 255.0);
    TypeTool {
        transform: [1.0, 0.0, 0.0, 1.0, spec.origin.0, spec.origin.1],
        text: spec.text.clone(),
        bounds: [0; 4],
        text_desc: Vec::new(),
        warp_desc: Vec::new(),
        fonts: Vec::new(),
        style: Some(TextStyle {
            font: Some(spec.font.clone()),
            font_size: spec.size,
            fill_color: [a, r, g, b],
            tracking: 0.0,
            justification: spec.justification,
        }),
        vertical: spec.vertical,
    }
}

/// Render `spec` over its placement rect: planar R, G, B, A. Anti-aliasing off
/// thresholds the coverage at half, so every edge pixel is in or out.
pub fn render_type(spec: &TypeSpec) -> Option<(PsdRect, PixelBuffer)> {
    let placement = type_placement(spec)?;
    let rect = placement.rect;
    let mut buffer = if spec.matrix == TypeSpec::IDENTITY {
        render_text_buffer(&render_view(spec), rect.width(), rect.height())?
    } else {
        render_mapped(spec, rect)?
    };
    if !spec.antialias {
        let plane = buffer.pixel_count();
        for alpha in &mut buffer.data[3 * plane..] {
            *alpha = if *alpha >= 128 { 255 } else { 0 };
        }
    }
    Some((rect, buffer))
}

/// Transformed type: lay the text out in its untransformed box, then map each
/// glyph's pen through `spec.matrix` and rasterize its outline mapped the same
/// way, so scaled or rotated type stays as sharp as unscaled type.
fn render_mapped(spec: &TypeSpec, rect: PsdRect) -> Option<PixelBuffer> {
    let bundled = crate::fonts::face_for(&spec.font)?;
    let flat = TypeSpec {
        matrix: TypeSpec::IDENTITY,
        ..spec.clone()
    };
    let unmapped = type_placement(&flat)?.rect;
    let view = render_view(&flat);
    let layout = layout_type(bundled, &view, unmapped.width(), unmapped.height())?;
    let size = spec.size as f32;
    let ascent = bundled.ascent(size);
    let color = spec.color.map(|c| f64::from(c) / 255.0);
    let m = spec.matrix.map(|v| v as f32);
    let rasterizer = bundled.rasterizer();
    let mut out = PixelBuffer::new(rect.width() as u32, rect.height() as u32, 4);
    let mut painted = false;
    for glyph in layout.lines.iter().flat_map(|line| &line.glyphs) {
        let u = f64::from(unmapped.left) + f64::from(glyph.x) - spec.origin.0;
        let v = f64::from(unmapped.top) + f64::from(glyph.y + ascent) - spec.origin.1;
        let (dx, dy) = map(spec, u, v);
        let (fx, fy) = (
            (dx - f64::from(rect.left)) as f32,
            (dy - f64::from(rect.top)) as f32,
        );
        let (px, py) = (fx.floor(), fy.floor());
        let request = RasterRequest {
            glyph: glyph.id,
            px_size: size,
            subpixel_x: fx - px,
            subpixel_y: py - fy,
        };
        if let Some(mask) = rasterizer.rasterize_mapped(&request, m) {
            painted |= blit_glyph(&mut out, &mask, px as i32, py as i32, color);
        }
    }
    painted.then_some(out)
}

/// The Type Mask tools' selection: `spec`'s coverage on a `width × height`
/// document plane, clipped to the canvas. `None` when nothing renders.
pub fn type_mask(width: u32, height: u32, spec: &TypeSpec) -> Option<Vec<u8>> {
    let (rect, buffer) = render_type(spec)?;
    let (dw, dh) = (width as i32, height as i32);
    let plane = buffer.pixel_count();
    let mut mask = vec![0u8; width as usize * height as usize];
    for ly in 0..rect.height() {
        let y = rect.top + ly;
        if !(0..dh).contains(&y) {
            continue;
        }
        for lx in 0..rect.width() {
            let x = rect.left + lx;
            if (0..dw).contains(&x) {
                let i = (ly * rect.width() + lx) as usize;
                mask[(y * dw + x) as usize] = buffer.data[3 * plane + i];
            }
        }
    }
    Some(mask)
}

/// A type layer's name: its first line, as CS6 names one, cut at 30 characters.
fn layer_name(text: &str) -> String {
    let first = type_lines(text).into_iter().find(|l| !l.trim().is_empty());
    first.unwrap_or_default().trim().chars().take(30).collect()
}

/// `spec` as a type layer's content: its rect, rendered `0/1/2/-1` channels,
/// and the authored `TySh` view. `None` when nothing renders.
fn type_content(spec: &TypeSpec) -> Option<(PsdRect, Vec<Channel>, TypeTool)> {
    let placement = type_placement(spec)?;
    let (rect, buffer) = render_type(spec)?;
    let authored = TypeSpec {
        font: crate::fonts::font_postscript_name(&spec.font).unwrap_or_else(|| spec.font.clone()),
        ..spec.clone()
    };
    let tool = pictura_codec::author_type_tool(&authored, placement.bounds);
    let plane = buffer.pixel_count();
    let channels = [0, 1, 2, -1]
        .into_iter()
        .enumerate()
        .map(|(i, id)| Channel {
            id,
            data: buffer.data[i * plane..(i + 1) * plane].to_vec().into(),
        })
        .collect();
    Some((rect, channels, tool))
}

fn rgb8(doc: &Document) -> bool {
    doc.mode == ColorMode::Rgb && doc.depth == BitDepth::Eight
}

/// Commit `spec` as a type layer above `selection_path` (the [`insert_node`]
/// rule): rendered pixels, an authored `TySh`, and CS6's type locks
/// (transparency and pixels). Returns the new path; empty when nothing renders
/// or the document is not 8-bit RGB.
///
/// ponytail: 8-bit RGB only; Grayscale / CMYK / Lab and 16/32-bit documents
/// need the layer's channels in the document's mode and depth.
///
/// [`insert_node`]: crate::document_ops::insert_node
pub fn add_type_layer(doc: &mut Document, selection_path: &str, spec: &TypeSpec) -> String {
    if !rgb8(doc) {
        return String::new();
    }
    let Some((rect, channels, tool)) = type_content(spec) else {
        return String::new();
    };
    let layer = pictura_core::Layer {
        name: layer_name(&spec.text),
        rect,
        lock: LockFlags::default()
            .with(LockFlags::TRANSPARENCY, true)
            .with(LockFlags::PIXELS, true),
        channels,
        extra_blocks: vec![LayerBlock {
            key: *b"TySh",
            data: pictura_codec::encode_type_tool(&tool),
        }],
        type_tool: Some(tool),
        ..Default::default()
    };
    crate::document_ops::insert_node(doc, selection_path, layer)
}

/// The topmost visible type layer whose rect contains `(x, y)`, searching into
/// visible groups; `None` when the click lands on no type.
pub fn type_layer_at(doc: &Document, x: f64, y: f64) -> Option<String> {
    fn walk(layers: &[pictura_core::Layer], prefix: &str, x: f64, y: f64) -> Option<String> {
        layers.iter().enumerate().rev().find_map(|(i, layer)| {
            if !layer.visible {
                return None;
            }
            let path = if prefix.is_empty() {
                i.to_string()
            } else {
                format!("{prefix}/{i}")
            };
            if layer.is_group {
                return walk(&layer.children, &path, x, y);
            }
            let r = layer.rect;
            let inside = x >= f64::from(r.left)
                && x < f64::from(r.right)
                && y >= f64::from(r.top)
                && y < f64::from(r.bottom);
            (inside && layer.type_tool.as_ref().is_some_and(|t| t.style.is_some())).then_some(path)
        })
    }
    walk(&doc.layers, "", x, y)
}

/// The spec a type layer reopens with: its text, font, size, colour,
/// justification, and orientation, the origin following the layer if it moved
/// since its `TySh` transform was written, and the transform's linear part as
/// `matrix`. Anti-aliasing is not in the view and comes back on.
///
/// ponytail: one style for the whole text (the first run's).
pub fn type_layer_spec(layer: &pictura_core::Layer) -> Option<TypeSpec> {
    let tool = layer.type_tool.as_ref()?;
    let style = tool.style.as_ref()?;
    let mut spec = TypeSpec {
        text: type_lines(&tool.text).join("\r"),
        font: style
            .font
            .as_deref()
            .map(|name| crate::fonts::font_family(name).unwrap_or_else(|| name.to_string()))
            .unwrap_or_default(),
        size: style.font_size,
        color: style.rgba(),
        justification: style.justification.min(2),
        vertical: tool.vertical,
        antialias: true,
        origin: (tool.transform[4], tool.transform[5]),
        matrix: [
            tool.transform[0],
            tool.transform[1],
            tool.transform[2],
            tool.transform[3],
        ],
    };
    if let Some(placed) = type_placement(&spec) {
        spec.origin.0 += f64::from(layer.rect.left - placed.rect.left);
        spec.origin.1 += f64::from(layer.rect.top - placed.rect.top);
    }
    Some(spec)
}

/// Re-set the type layer at `path` from `spec` in place: new rect, pixels, and
/// authored `TySh`. A name that was the old text's default
/// follows the new text; a renamed layer keeps its name. False (unchanged) for
/// a missing or non-type layer, blank text, or a document that is not 8-bit RGB.
///
/// ponytail: an opened layer's own `TySh` (several runs, warp, paragraph type)
/// is replaced by the single-run authored one.
pub fn replace_type_layer(doc: &mut Document, path: &str, spec: &TypeSpec) -> bool {
    if !rgb8(doc) {
        return false;
    }
    let Some((rect, channels, tool)) = type_content(spec) else {
        return false;
    };
    let Some(layer) = crate::resolve_path_mut(doc, path) else {
        return false;
    };
    let Some(old) = layer.type_tool.as_ref() else {
        return false;
    };
    if layer.name == layer_name(&old.text) {
        layer.name = layer_name(&spec.text);
    }
    layer.rect = rect;
    layer.channels = channels;
    layer.raw_channels.clear();
    layer.source_channels = None;
    let block = LayerBlock {
        key: *b"TySh",
        data: pictura_codec::encode_type_tool(&tool),
    };
    match layer.extra_blocks.iter_mut().find(|b| &b.key == b"TySh") {
        Some(existing) => *existing = block,
        None => layer.extra_blocks.push(block),
    }
    layer.type_tool = Some(tool);
    true
}

/// Free Transform / Skew of a type layer as type: fold the document-space
/// affine `a` (`xx, xy, yx, yy, tx, ty`, the `TySh` order) into its matrix and
/// origin and re-set it, so the text stays sharp and editable. `None` when the
/// layer is not type this can re-set (no style, a layer mask, or not 8-bit RGB),
/// leaving the caller to resample its pixels; `Some(false)` for a
/// position-locked layer.
///
/// ponytail: a type layer with a layer mask is resampled as pixels.
pub fn transform_type_layer(doc: &mut Document, path: &str, a: [f64; 6]) -> Option<bool> {
    let layer = crate::resolve_path(doc, path)?;
    if layer.mask.is_some() || !rgb8(doc) {
        return None;
    }
    if layer.lock.contains(LockFlags::POSITION) {
        return Some(false);
    }
    let mut spec = type_layer_spec(layer)?;
    let [xx, xy, yx, yy] = spec.matrix;
    spec.matrix = [
        a[0] * xx + a[2] * xy,
        a[1] * xx + a[3] * xy,
        a[0] * yx + a[2] * yy,
        a[1] * yx + a[3] * yy,
    ];
    let (ox, oy) = spec.origin;
    spec.origin = (a[0] * ox + a[2] * oy + a[4], a[1] * ox + a[3] * oy + a[5]);
    Some(replace_type_layer(doc, path, &spec))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(text: &str, vertical: bool, justification: u8) -> TypeSpec {
        TypeSpec {
            text: text.into(),
            font: "Liberation Sans".into(),
            size: 32.0,
            color: [0, 0, 255, 255],
            justification,
            vertical,
            antialias: true,
            origin: (100.0, 80.0),
            matrix: TypeSpec::IDENTITY,
        }
    }

    fn alpha_bounds(width: u32, mask: &[u8]) -> (u32, u32, u32, u32) {
        let mut b = (u32::MAX, u32::MAX, 0, 0);
        for (i, _) in mask.iter().enumerate().filter(|(_, &a)| a > 0) {
            let (x, y) = (i as u32 % width, i as u32 / width);
            b = (b.0.min(x), b.1.min(y), b.2.max(x), b.3.max(y));
        }
        b
    }

    #[test]
    fn horizontal_type_sits_on_its_baseline_and_follows_alignment() {
        let left = type_placement(&spec("Type", false, 0)).unwrap();
        assert_eq!(left.rect.left, 100);
        assert!(left.rect.top < 80 && left.rect.bottom > 80, "{left:?}");
        let right = type_placement(&spec("Type", false, 1)).unwrap();
        assert!(right.rect.right <= 102 && right.rect.left < 100);
        let centre = type_placement(&spec("Type", false, 2)).unwrap();
        assert!(centre.rect.left < 100 && centre.rect.right > 100);
        let two = type_placement(&spec("Type\rTwo", false, 0)).unwrap();
        assert!(two.rect.height() >= left.rect.height() + 38);
    }

    #[test]
    fn vertical_type_is_taller_than_wide_and_columns_run_leftward() {
        let one = type_placement(&spec("ABCD", true, 0)).unwrap();
        assert!(one.rect.height() > 3 * one.rect.width(), "{one:?}");
        assert!(one.rect.left < 100 && one.rect.right > 100);
        assert_eq!(one.rect.top, 80);
        let two = type_placement(&spec("AB\rCD", true, 0)).unwrap();
        assert_eq!(two.rect.right, one.rect.right);
        assert!(two.rect.left < one.rect.left);
        let mask = type_mask(200, 400, &spec("ABCD", true, 0)).unwrap();
        let (x0, y0, x1, y1) = alpha_bounds(200, &mask);
        assert!(y1 - y0 > 3 * (x1 - x0), "glyphs stack downward");
    }

    #[test]
    fn type_mask_is_coverage_clipped_to_the_canvas() {
        let mask = type_mask(120, 100, &spec("Wide text", false, 0)).unwrap();
        assert_eq!(mask.len(), 120 * 100);
        let (x0, _, x1, y1) = alpha_bounds(120, &mask);
        assert!(x0 >= 100 && x1 == 119 && y1 < 100);
        let mut hard = spec("O", false, 0);
        hard.antialias = false;
        let mask = type_mask(200, 200, &hard).unwrap();
        assert!(mask.iter().all(|&a| a == 0 || a == 255));
        assert!(mask.contains(&255));
        assert!(type_mask(200, 200, &spec(" \r ", false, 0)).is_none());
    }

    #[test]
    fn add_type_layer_inserts_a_locked_pixel_backed_type_layer() {
        let mut doc = Document::new(300, 200, ColorMode::Rgb, BitDepth::Eight);
        let path = add_type_layer(&mut doc, "", &spec("Hello\rworld", false, 0));
        assert_eq!(path, "0");
        let layer = &doc.layers[0];
        assert_eq!(layer.name, "Hello");
        assert!(layer.is_type());
        assert_eq!(layer.lock.bits(), 0x03);
        let tool = layer.type_tool.as_ref().unwrap();
        assert_eq!(tool.text, "Hello\rworld");
        assert_eq!(tool.style.as_ref().unwrap().rgba(), [0, 0, 255, 255]);
        let alpha = crate::channel(layer, -1).unwrap();
        let blue = crate::channel(layer, 2).unwrap();
        let painted = alpha.iter().position(|&a| a == 255).expect("opaque ink");
        assert_eq!(blue[painted], 255);
        assert_eq!(crate::channel(layer, 0).unwrap()[painted], 0);

        let mut gray = Document::new(300, 200, ColorMode::Grayscale, BitDepth::Eight);
        assert!(add_type_layer(&mut gray, "", &spec("x", false, 0)).is_empty());
    }

    #[test]
    fn a_type_layer_reopens_where_it_is_and_edits_in_place() {
        let mut doc = Document::new(300, 200, ColorMode::Rgb, BitDepth::Eight);
        let original = spec("Hello", false, 2);
        let path = add_type_layer(&mut doc, "", &original);
        let r = doc.layers[0].rect;
        assert_eq!(
            type_layer_at(&doc, f64::from(r.left + 2), 75.0).as_deref(),
            Some("0")
        );
        assert_eq!(type_layer_at(&doc, 5.0, 5.0), None);

        // Moved after it was set: the reopened origin follows the pixels.
        doc.layers[0].rect = PsdRect {
            top: r.top + 10,
            left: r.left + 20,
            bottom: r.bottom + 10,
            right: r.right + 20,
        };
        let reopened = type_layer_spec(&doc.layers[0]).unwrap();
        assert_eq!(reopened.text, "Hello");
        assert_eq!(reopened.size, 32.0);
        assert_eq!(reopened.color, [0, 0, 255, 255]);
        assert_eq!(reopened.justification, 2);
        assert_eq!(reopened.origin, (120.0, 90.0));

        let mut edited = reopened.clone();
        edited.text = "Hello there".into();
        doc.layers[0].visible = false;
        assert!(replace_type_layer(&mut doc, &path, &edited));
        let layer = &doc.layers[0];
        assert_eq!(doc.layers.len(), 1);
        assert!(!layer.visible, "visibility is the caller's");
        assert_eq!(layer.name, "Hello there");
        assert_eq!(layer.type_tool.as_ref().unwrap().text, "Hello there");
        assert_eq!(
            layer
                .extra_blocks
                .iter()
                .filter(|b| &b.key == b"TySh")
                .count(),
            1
        );
        assert!(layer.rect.width() > r.width());
        assert!(!replace_type_layer(&mut doc, "5", &edited));
    }

    /// Soft edge pixels per solid pixel: resampling blurs it, re-setting does not.
    fn softness(layer: &pictura_core::Layer) -> f64 {
        let alpha = crate::channel(layer, -1).unwrap();
        let soft = alpha.iter().filter(|&&a| a > 0 && a < 255).count();
        let solid = alpha.iter().filter(|&&a| a == 255).count();
        soft as f64 / solid.max(1) as f64
    }

    #[test]
    fn free_transform_re_sets_type_sharp_and_editable() {
        use crate::document_ops::{transform_layer, transform_layer_quad, LayerTransform};
        let mut doc = Document::new(600, 600, ColorMode::Rgb, BitDepth::Eight);
        let path = add_type_layer(&mut doc, "", &spec("Sharp", false, 0));
        let before = doc.layers[0].clone();
        let scale = LayerTransform {
            scale_x: 3.0,
            scale_y: 3.0,
            angle_radians: 0.0,
            dx: 0.0,
            dy: 0.0,
        };
        assert!(transform_layer(&mut doc, &path, scale));
        let after = &doc.layers[0];
        assert!(after.is_type());
        let tool = after.type_tool.as_ref().unwrap();
        assert_eq!(tool.transform[..4], [3.0, 0.0, 0.0, 3.0]);
        assert!((after.rect.width() - 3 * before.rect.width()).abs() <= 4);
        assert!(
            softness(after) < softness(&before) * 0.6,
            "{} vs {}",
            softness(after),
            softness(&before)
        );

        // Editing keeps the transform; a quarter turn stands the text up.
        let mut edited = type_layer_spec(after).unwrap();
        assert_eq!(edited.matrix, [3.0, 0.0, 0.0, 3.0]);
        edited.text = "Sharper".into();
        assert!(replace_type_layer(&mut doc, &path, &edited));
        assert_eq!(doc.layers[0].type_tool.as_ref().unwrap().transform[0], 3.0);
        let turn = LayerTransform {
            scale_x: 1.0,
            scale_y: 1.0,
            angle_radians: std::f64::consts::FRAC_PI_2,
            dx: 0.0,
            dy: 0.0,
        };
        let wide = doc.layers[0].rect;
        assert!(transform_layer(&mut doc, &path, turn));
        let tall = doc.layers[0].rect;
        assert!(tall.height() > tall.width() && (tall.height() - wide.width()).abs() <= 4);

        // Skew is affine and stays type; a position lock refuses.
        let r = doc.layers[0].rect;
        let (l, t, rr, b) = (
            f64::from(r.left),
            f64::from(r.top),
            f64::from(r.right),
            f64::from(r.bottom),
        );
        let skew = [(l + 20.0, t), (rr + 20.0, t), (rr, b), (l, b)];
        assert!(transform_layer_quad(&mut doc, &path, skew));
        let tool = doc.layers[0].type_tool.as_ref().unwrap();
        assert!(doc.layers[0].is_type() && tool.transform[2].abs() > 0.01);
        doc.layers[0].lock = doc.layers[0].lock.with(LockFlags::POSITION, true);
        assert!(!transform_layer(&mut doc, &path, scale));
    }
}
