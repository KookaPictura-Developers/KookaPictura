use cxx_qt_lib::{QString, QStringList};
use pictura_core::{
    AdjustmentData, BlendMode, ColorLabel, Document, Layer, LayerMask, LockFlags, PsdRect,
};
use pictura_paint::{PaintMode, Rgba};
use pictura_select::CombineMode;
/// Build an adjustment layer for `kind`, or `None` for an unknown kind.
///
/// Defaults are chosen so a freshly added layer visibly changes the composite;
/// editing parameters is out of scope for M4-C. `mask` confines the effect to a
/// selection when one is active.
pub(super) fn adjustment_layer(kind: &str, mask: Option<LayerMask>) -> Option<Layer> {
    use pictura_render::{
        encode_brightness_contrast, encode_gradient_map, encode_hue_saturation, encode_invert,
        encode_photo_filter, encode_posterize, encode_threshold, GradientStop,
    };

    let (name, data): (&str, AdjustmentData) = match kind {
        "invert" => ("Invert", encode_invert()),
        "posterize" => ("Posterize", encode_posterize(4)),
        "threshold" => ("Threshold", encode_threshold(128)),
        "brightness-contrast" => ("Brightness/Contrast", encode_brightness_contrast(20, 0)),
        "hue-saturation" => ("Hue/Saturation", encode_hue_saturation(30, 0, 0)),
        "photo-filter" => (
            "Photo Filter",
            encode_photo_filter([255, 180, 80], 25.0, true),
        ),
        "gradient-map" => (
            "Gradient Map",
            encode_gradient_map(
                &[
                    GradientStop {
                        location: 0,
                        color: [0, 0, 0],
                    },
                    GradientStop {
                        location: 4096,
                        color: [255, 255, 255],
                    },
                ],
                false,
            ),
        ),
        _ => return None,
    };

    Some(Layer {
        name: name.into(),
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
        mask,
        adjustment: Some(data),
        channels: Vec::new(),
        children: Vec::new(),
        is_group: false,
        background: false,
        ..Default::default()
    })
}
/// Delegates to the shared M36 Background heuristic in `pictura-render`, so the
/// panel core and the bridge agree on one definition.
pub(super) fn is_background_layer(doc: &Document, i: i32) -> bool {
    pictura_render::is_background(doc, &i.to_string())
}
/// Map a lock-strip flag name to its [`LockFlags`] bit. `"all"` is the derived
/// four-bit set; anything else is `None`.
pub(super) fn lock_bit(flag: &str) -> Option<u8> {
    match flag {
        "transparency" => Some(LockFlags::TRANSPARENCY),
        "pixels" => Some(LockFlags::PIXELS),
        "position" => Some(LockFlags::POSITION),
        "nesting" => Some(LockFlags::NESTING),
        "all" => Some(LockFlags::all().bits()),
        _ => None,
    }
}
/// The shared row-kind label, so [`layer_kind`] and the tree projection agree.
pub(super) fn layer_kind_str(doc: &Document, path: &str, layer: &Layer) -> QString {
    if layer.is_group {
        QString::from("group")
    } else if layer.adjustment.is_some() {
        QString::from("adjustment")
    } else if pictura_render::is_background(doc, path) {
        QString::from("background")
    } else {
        QString::from("pixel")
    }
}
/// Owned copy of a `QStringList`'s entries.
pub(super) fn list_of_strings(paths: &QStringList) -> Vec<String> {
    paths.iter().map(|path| path.to_string()).collect()
}
/// Borrow a `Vec<String>` as `&[&str]` for the path core.
pub(super) fn as_str_slice(owned: &[String]) -> Vec<&str> {
    owned.iter().map(String::as_str).collect()
}
/// Map a filter `kind` to its [`pictura_filters::Filter`], or `None` unknown.
///
/// Defaults are chosen so a fresh apply visibly changes a non-trivial image;
/// filter dialogs are out of scope for M6-C.
pub(super) fn filter_from_kind(kind: &str) -> Option<pictura_filters::Filter> {
    use pictura_filters::{
        BrushType, Filter, GrainType, HalftoneType, LensType, LightDirection, MezzotintType,
        NoiseDistribution, PolarKind, RippleSize, ShearFill, SpherizeMode, StrokeDirection,
        TextureOptions, WaveType, ZigZagStyle,
    };

    Some(match kind {
        "gaussian-blur" => Filter::GaussianBlur { radius: 5.0 },
        "box-blur" => Filter::BoxBlur { radius: 3 },
        "surface-blur" => Filter::SurfaceBlur {
            radius: 10,
            threshold: 20,
        },
        "motion-blur" => Filter::MotionBlur {
            angle: 0.0,
            distance: 15,
        },
        "median" => Filter::Median { radius: 2 },
        "despeckle" => Filter::Despeckle,
        "sharpen" => Filter::Sharpen,
        "sharpen-more" => Filter::SharpenMore,
        "unsharp-mask" => Filter::UnsharpMask {
            amount: 150.0,
            radius: 1.0,
            threshold: 0,
        },
        "add-noise" => Filter::AddNoise {
            amount: 25.0,
            distribution: NoiseDistribution::Uniform,
            monochromatic: false,
            seed: 1,
        },
        "maximum" => Filter::Maximum { radius: 2 },
        "minimum" => Filter::Minimum { radius: 2 },
        "offset" => Filter::Offset {
            horizontal: 4,
            vertical: 4,
            wrap: true,
            background: [0, 0, 0],
        },
        "high-pass" => Filter::HighPass { radius: 4.0 },
        "emboss" => Filter::Emboss {
            angle: 135.0,
            height: 2.0,
            amount: 100.0,
        },
        "find-edges" => Filter::FindEdges,
        "solarize" => Filter::Solarize,
        "mosaic" => Filter::Mosaic { cell_size: 10 },
        "crystallize" => Filter::Crystallize {
            cell_size: 10,
            seed: 1,
        },
        "facet" => Filter::Facet,
        "fragment" => Filter::Fragment,
        "mezzotint" => Filter::Mezzotint {
            kind: MezzotintType::FineDots,
            seed: 1,
        },
        "pointillize" => Filter::Pointillize {
            cell_size: 5,
            background: [0, 0, 0],
            seed: 1,
        },
        "color-halftone" => Filter::ColorHalftone {
            max_radius: 5,
            angles: [108.0, 162.0, 90.0, 45.0],
        },
        "twirl" => Filter::Twirl { angle: 90.0 },
        "pinch" => Filter::Pinch { amount: 50.0 },
        "spherize" => Filter::Spherize {
            amount: 100.0,
            mode: SpherizeMode::Normal,
        },
        "ripple" => Filter::Ripple {
            amount: 100.0,
            size: RippleSize::Medium,
        },
        "wave" => Filter::Wave {
            generators: 5,
            wavelength: (10.0, 120.0),
            amplitude: (5.0, 35.0),
            kind: WaveType::Sine,
            scale: (100.0, 100.0),
            seed: 1,
            repeat_edge: true,
        },
        "polar-coordinates" => Filter::PolarCoordinates {
            kind: PolarKind::RectangularToPolar,
        },
        "shear" => Filter::Shear {
            curve: vec![(-1.0, -0.5), (0.0, 0.0), (1.0, 0.5)],
            fill: ShearFill::RepeatEdgePixels,
        },
        "zigzag" => Filter::ZigZag {
            amount: 50.0,
            ridges: 5,
            style: ZigZagStyle::AroundCenter,
        },
        "ocean-ripple" => Filter::OceanRipple {
            size: 9,
            magnitude: 5,
            seed: 1,
        },
        "clouds" => Filter::Clouds {
            color_a: [0, 0, 0],
            color_b: [255, 255, 255],
            starker: false,
            seed: 1,
        },
        "difference-clouds" => Filter::DifferenceClouds {
            color_a: [0, 0, 0],
            color_b: [255, 255, 255],
            starker: false,
            seed: 1,
        },
        "fibers" => Filter::Fibers {
            variance: 16.0,
            strength: 4.0,
            color_a: [0, 0, 0],
            color_b: [255, 255, 255],
            seed: 1,
        },
        "lens-flare" => Filter::LensFlare {
            brightness: 100.0,
            center: (0.5, 0.5),
            lens: LensType::Zoom,
        },
        "colored-pencil" => Filter::ColoredPencil {
            pencil_width: 6,
            stroke_pressure: 8,
            paper_brightness: 20,
            foreground: [0, 0, 0],
            background: [255, 255, 255],
            seed: 1,
        },
        "cutout" => Filter::Cutout {
            levels: 4,
            edge_simplicity: 0,
            edge_fidelity: 1,
        },
        "dry-brush" => Filter::DryBrush {
            brush_size: 8,
            brush_detail: 6,
            texture: 2,
            seed: 1,
        },
        "film-grain" => Filter::FilmGrain {
            grain: 10,
            highlight_area: 5,
            intensity: 5,
            seed: 1,
        },
        "fresco" => Filter::Fresco {
            brush_size: 8,
            brush_detail: 6,
            texture: 2,
            seed: 1,
        },
        "neon-glow" => Filter::NeonGlow {
            glow_size: 8,
            glow_brightness: 40,
            glow_color: [0, 255, 255],
        },
        "paint-daubs" => Filter::PaintDaubs {
            brush_size: 8,
            sharpness: 20,
            brush_type: BrushType::Simple,
            seed: 1,
        },
        "palette-knife" => Filter::PaletteKnife {
            stroke_size: 12,
            stroke_detail: 2,
            softness: 8,
            seed: 1,
        },
        "plastic-wrap" => Filter::PlasticWrap {
            highlight_strength: 0,
            detail: 6,
            smoothness: 3,
        },
        "poster-edges" => Filter::PosterEdges {
            edge_thickness: 3,
            edge_intensity: 10,
            posterization: 4,
        },
        "rough-pastels" => Filter::RoughPastels {
            stroke_length: 8,
            stroke_detail: 6,
            texture: TextureOptions::default(),
            foreground: [0, 0, 0],
            background: [255, 255, 255],
            seed: 1,
        },
        "smudge-stick" => Filter::SmudgeStick {
            stroke_length: 4,
            highlight_area: 8,
            intensity: 6,
            seed: 1,
        },
        "sponge" => Filter::Sponge {
            brush_size: 6,
            definition: 18,
            smoothness: 4,
            seed: 1,
        },
        "underpainting" => Filter::Underpainting {
            brush_size: 10,
            texture_coverage: 24,
            texture: TextureOptions::default(),
            seed: 1,
        },
        "watercolor" => Filter::Watercolor {
            brush_detail: 8,
            shadow_intensity: 6,
            texture: 2,
            foreground: [0, 0, 0],
            background: [255, 255, 255],
            seed: 1,
        },
        "accented-edges" => Filter::AccentedEdges {
            edge_width: 2,
            edge_brightness: 38,
            smoothness: 5,
        },
        "angled-strokes" => Filter::AngledStrokes {
            direction_balance: 50,
            stroke_length: 15,
            sharpness: 3,
        },
        "crosshatch" => Filter::Crosshatch {
            stroke_length: 9,
            sharpness: 6,
            strength: 1,
        },
        "dark-strokes" => Filter::DarkStrokes {
            balance: 5,
            black_intensity: 6,
            white_intensity: 5,
        },
        "ink-outlines" => Filter::InkOutlines {
            stroke_length: 10,
            dark_intensity: 25,
            light_intensity: 25,
        },
        "spatter" => Filter::Spatter {
            spray_radius: 10,
            smoothness: 5,
            seed: 1,
        },
        "sprayed-strokes" => Filter::SprayedStrokes {
            stroke_length: 12,
            spray_radius: 7,
            direction: StrokeDirection::RightDiagonal,
            seed: 1,
        },
        "sumi-e" => Filter::SumiE {
            stroke_width: 8,
            stroke_pressure: 5,
            contrast: 20,
        },
        "bas-relief" => Filter::BasRelief {
            detail: 6,
            smoothness: 3,
            light_direction: LightDirection::Bottom,
            foreground: [0, 0, 0],
            background: [255, 255, 255],
        },
        "chalk-charcoal" => Filter::ChalkCharcoal {
            charcoal_area: 6,
            chalk_area: 6,
            stroke_pressure: 1,
            foreground: [0, 0, 0],
            background: [255, 255, 255],
            seed: 1,
        },
        "charcoal" => Filter::Charcoal {
            thickness: 1,
            detail: 3,
            light_dark_balance: 50,
            foreground: [0, 0, 0],
            background: [255, 255, 255],
            seed: 1,
        },
        "chrome" => Filter::Chrome {
            detail: 4,
            smoothness: 7,
        },
        "conte-crayon" => Filter::ConteCrayon {
            foreground_level: 8,
            background_level: 7,
            texture: TextureOptions::default(),
            foreground: [0, 0, 0],
            background: [255, 255, 255],
            seed: 1,
        },
        "graphic-pen" => Filter::GraphicPen {
            stroke_length: 6,
            light_dark_balance: 50,
            direction: StrokeDirection::RightDiagonal,
            foreground: [0, 0, 0],
            background: [255, 255, 255],
        },
        "halftone-pattern" => Filter::HalftonePattern {
            size: 5,
            contrast: 5,
            pattern: HalftoneType::Dot,
        },
        "note-paper" => Filter::NotePaper {
            image_balance: 25,
            graininess: 10,
            relief: 11,
            seed: 1,
        },
        "photocopy" => Filter::Photocopy {
            detail: 5,
            darkness: 20,
        },
        "plaster" => Filter::Plaster {
            image_balance: 25,
            smoothness: 2,
            light_direction: LightDirection::Bottom,
            foreground: [0, 0, 0],
            background: [255, 255, 255],
        },
        "reticulation" => Filter::Reticulation {
            density: 13,
            black_level: 10,
            white_level: 40,
            foreground: [0, 0, 0],
            background: [255, 255, 255],
            seed: 1,
        },
        "stamp" => Filter::Stamp {
            light_dark_balance: 25,
            smoothness: 5,
            foreground: [0, 0, 0],
            background: [255, 255, 255],
        },
        "torn-edges" => Filter::TornEdges {
            image_balance: 25,
            smoothness: 1,
            contrast: 8,
            foreground: [0, 0, 0],
            background: [255, 255, 255],
        },
        "water-paper" => Filter::WaterPaper {
            fiber_length: 15,
            brightness: 45,
            contrast: 60,
            seed: 1,
        },
        "craquelure" => Filter::Craquelure {
            crack_spacing: 10,
            crack_depth: 6,
            crack_brightness: 9,
        },
        "grain" => Filter::Grain {
            intensity: 40,
            contrast: 50,
            grain_type: GrainType::Regular,
            background: [255, 255, 255],
            seed: 1,
        },
        "mosaic-tiles" => Filter::MosaicTiles {
            tile_size: 12,
            grout_width: 3,
            lighten_grout: 1,
            seed: 1,
        },
        "patchwork" => Filter::Patchwork {
            square_size: 5,
            relief: 8,
            seed: 1,
        },
        "stained-glass" => Filter::StainedGlass {
            cell_size: 10,
            border_thickness: 4,
            light_intensity: 5,
            foreground: [0, 0, 0],
            seed: 1,
        },
        "texturizer" => Filter::Texturizer {
            texture: TextureOptions::default(),
        },
        "oil-paint" => Filter::OilPaint {
            stylization: 3.5,
            cleanliness: 4.5,
            scale: 0.75,
            bristle_detail: 3.0,
            angular_direction: 85.0,
            shine: 0.55,
        },
        // `Custom` requires a caller-supplied 5x5 kernel, so no meaningful
        // default exists; it stays out of the dock and is left unmapped.
        _ => return None,
    })
}
/// Map an image-size resample `kind` to [`pictura_render::Resample`], or
/// `None` unknown.
pub(super) fn parse_resample(kind: &str) -> Option<pictura_render::Resample> {
    use pictura_render::Resample;

    Some(match kind {
        "nearest" => Resample::Nearest,
        "bilinear" => Resample::Bilinear,
        "bicubic" => Resample::Bicubic,
        _ => return None,
    })
}
/// Map a canvas-size `anchor` to [`pictura_render::Anchor`], or `None` unknown.
pub(super) fn parse_anchor(anchor: &str) -> Option<pictura_render::Anchor> {
    use pictura_render::Anchor;

    Some(match anchor {
        "top-left" => Anchor::TopLeft,
        "top-center" => Anchor::TopCenter,
        "top-right" => Anchor::TopRight,
        "center-left" => Anchor::MiddleLeft,
        "center" => Anchor::Center,
        "center-right" => Anchor::MiddleRight,
        "bottom-left" => Anchor::BottomLeft,
        "bottom-center" => Anchor::BottomCenter,
        "bottom-right" => Anchor::BottomRight,
        _ => return None,
    })
}
/// Map a selection `mode` string to [`CombineMode`]; unknown means `New`.
pub(super) fn combine_mode_from(mode: &str) -> CombineMode {
    match mode {
        "add" => CombineMode::Add,
        "subtract" => CombineMode::Subtract,
        "intersect" => CombineMode::Intersect,
        _ => CombineMode::New,
    }
}
/// Unpack a `0xAARRGGBB` colour.
pub(super) fn rgba_from_argb(argb: u32) -> Rgba {
    Rgba {
        r: (argb >> 16) as u8,
        g: (argb >> 8) as u8,
        b: argb as u8,
        a: (argb >> 24) as u8,
    }
}
/// Map a paint-mode string to [`PaintMode`]; unknown means `Normal`.
pub(super) fn paint_mode_from(mode: &str) -> PaintMode {
    match mode {
        "dissolve" => PaintMode::Dissolve,
        "behind" => PaintMode::Behind,
        "clear" => PaintMode::Clear,
        _ => PaintMode::Normal,
    }
}
/// Index of the topmost pixel layer: the last layer (bottom-first order) that is
/// neither a group nor an adjustment.
pub(super) fn topmost_pixel_layer_index(doc: &Document) -> Option<usize> {
    doc.layers
        .iter()
        .rposition(|l| l.adjustment.is_none() && !l.is_group)
}
/// The topmost pixel layer: the last layer (bottom-first order) that is
/// neither a group nor an adjustment.
pub(super) fn topmost_pixel_layer(doc: &mut Document) -> Option<&mut Layer> {
    let index = topmost_pixel_layer_index(doc)?;
    doc.layers.get_mut(index)
}
/// The topmost pixel layer's document-space rect.
pub(super) fn topmost_pixel_layer_rect(doc: &Document) -> Option<PsdRect> {
    topmost_pixel_layer_index(doc).map(|index| doc.layers[index].rect)
}
/// A rectangle that provably bounds a visibility toggle's effect, or `None` when
/// the toggle can change a pixel outside any such rectangle.
///
/// A pixel layer's channels cover its `rect`, so hiding it changes the composite
/// only there. An adjustment layer transforms the whole backdrop, but a mask
/// that is enabled, carries data, and has a zero `default_color` confines it to
/// its mask `rect` (outside, `mask_alpha` returns the default 0). Everything
/// else — groups, unmasked/disabled/data-less/non-zero-default adjustments — is
/// unbounded and returns `None`. `refresh_region` clamps the returned rect.
pub(super) fn layer_visibility_region(layer: &Layer) -> Option<PsdRect> {
    if layer.is_group {
        return None;
    }
    if layer.adjustment.is_some() {
        let mask = layer.mask.as_ref()?;
        if mask.disabled || mask.data.is_none() || mask.default_color != 0 {
            return None;
        }
        return Some(mask.rect);
    }
    if !layer
        .channels
        .iter()
        .any(|c| c.id >= 0 && !c.data.is_empty())
    {
        return None;
    }
    match &layer.mask {
        None => Some(layer.rect),
        Some(mask) if mask.disabled || mask.default_color == 0 => Some(layer.rect),
        Some(_) => None,
    }
}
/// The bounding box of two document-space rects.
pub(super) fn union_rect(a: PsdRect, b: PsdRect) -> PsdRect {
    PsdRect {
        top: a.top.min(b.top),
        left: a.left.min(b.left),
        bottom: a.bottom.max(b.bottom),
        right: a.right.max(b.right),
    }
}
/// Clamp `rect` to `width`×`height`.
///
/// Returns `(x0, y0, w, h)` in document pixels, or `None` when the intersection
/// is empty (zero or negative area).
pub(super) fn clamp_region(rect: PsdRect, width: u32, height: u32) -> Option<(i32, i32, u32, u32)> {
    let x0 = rect.left.max(0);
    let y0 = rect.top.max(0);
    let x1 = rect.right.min(width as i32);
    let y1 = rect.bottom.min(height as i32);
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    Some((x0, y0, (x1 - x0) as u32, (y1 - y0) as u32))
}
/// The region to patch for the move-preview base, or `None` to fall back to a
/// full composite.
///
/// The authoritative composite must match the document (`cached_ok`); an empty
/// clamped rect also falls back. There is no area budget: the region is blitted
/// in C++ regardless of size.
pub(super) fn move_preview_region(
    rect: PsdRect,
    width: u32,
    height: u32,
    cached_ok: bool,
) -> Option<(i32, i32, u32, u32)> {
    if !cached_ok {
        return None;
    }
    clamp_region(rect, width, height)
}

/// Probe `bytes`, decode them with Qt to packed RGBA8888, and enforce the
/// budget against the actual decoded allocation.
///
/// Returns `(rgba, width, height)`, or `None` when a recognized header is over
/// budget, Qt cannot decode the bytes, or the decoded allocation is over budget.
///
/// Qt's runtime decoders are the authoritative allow-list: the probe is only a
/// fast-path guard that pre-empts a recognized header, so a container the probe
/// does not recognize is still handed to Qt and capped afterwards.
pub(super) fn decode_import(bytes: &[u8]) -> Option<(Vec<u8>, u32, u32)> {
    let budget = pictura_codec::ImageBudget::default();
    match pictura_codec::probe_image(bytes, budget) {
        Ok(_) | Err(pictura_codec::ImportError::UnknownContainer { .. }) => {}
        Err(_) => return None,
    }
    let mut width = 0i32;
    let mut height = 0i32;
    let rgba = super::qobject::decode_image_rgba(bytes, &mut width, &mut height);
    if width <= 0 || height <= 0 {
        return None;
    }
    let (w, h) = (width as u32, height as u32);
    if w > budget.max_dimension || h > budget.max_dimension {
        return None;
    }
    if (w as u64) * (h as u64) * 4 > budget.max_alloc_bytes {
        return None;
    }
    if rgba.len() != (w as usize) * (h as usize) * 4 {
        return None;
    }
    Some((rgba, w, h))
}
