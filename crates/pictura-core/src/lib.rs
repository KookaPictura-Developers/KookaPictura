//! Core document types for Kooka Pictura.
//!
//! M0 scope is deliberately tiny: an 8-bit RGB/gray composite image. The full
//! layer/channel/mask model is specified in `docs/01-architecture/document-model.md`
//! and is **not** implemented here yet.

mod advanced_blending;
mod crs;
mod text_render;
mod type_tool;
mod vector;
pub use advanced_blending::{BlendIf, Knockout};
pub use crs::CrsSettings;
pub use text_render::{
    layout_lines, FontPolicy, GlyphMask, LayoutLine, LayoutParams, PlacedGlyph, RasterRequest,
    Rasterizer, ShapedGlyph, TextAlign, TextLayout, TextProvenance,
};
pub use type_tool::{TextStyle, TypeTool};
pub use vector::{VectorFillRule, VectorMask, VectorSubpath};

/// PSD color modes (`header.color_mode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Bitmap,
    Grayscale,
    Indexed,
    Rgb,
    Cmyk,
    Multichannel,
    Duotone,
    Lab,
}

impl ColorMode {
    /// Number of color channels the composite carries in PSD.
    ///
    /// Extra alpha/spot/selection channels are additional; Multichannel has no
    /// color channels at all (its count is header-authoritative).
    pub fn color_channels(self) -> u8 {
        match self {
            ColorMode::Bitmap | ColorMode::Grayscale | ColorMode::Indexed | ColorMode::Duotone => 1,
            ColorMode::Rgb | ColorMode::Lab => 3,
            ColorMode::Cmyk => 4,
            ColorMode::Multichannel => 0,
        }
    }
}

/// Bits per channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitDepth {
    One,
    Eight,
    Sixteen,
    ThirtyTwo,
}

/// PSD image-data compression method (the `image_data.compression` word).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Compression {
    #[default]
    Rle,
    Raw,
    Zip,
    ZipPrediction,
}

impl Compression {
    /// The PSD compression word (0/1/2/3).
    pub fn to_code(self) -> u16 {
        match self {
            Compression::Raw => 0,
            Compression::Rle => 1,
            Compression::Zip => 2,
            Compression::ZipPrediction => 3,
        }
    }

    /// Parse a PSD compression word; `None` for an unknown code.
    pub fn from_code(code: u16) -> Option<Compression> {
        Some(match code {
            0 => Compression::Raw,
            1 => Compression::Rle,
            2 => Compression::Zip,
            3 => Compression::ZipPrediction,
            _ => return None,
        })
    }
}

/// A planar, row-major, 8-bit-per-channel pixel buffer.
///
/// `data.len() == width * height * channels`. Planar means channel `c` for the
/// whole image comes first, matching the PSD image-data layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PixelBuffer {
    pub width: u32,
    pub height: u32,
    pub channels: u8,
    pub data: Vec<u8>,
}

impl PixelBuffer {
    pub fn new(width: u32, height: u32, channels: u8) -> Self {
        Self {
            width,
            height,
            channels,
            data: vec![0; width as usize * height as usize * channels as usize],
        }
    }

    /// Number of pixels (not samples).
    pub fn pixel_count(&self) -> usize {
        self.width as usize * self.height as usize
    }
}

/// A minimal document: dimensions, mode, depth, one composite image, and a
/// layer tree (bottom-first, matching PSD z-order on disk).
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub width: u32,
    pub height: u32,
    pub mode: ColorMode,
    pub depth: BitDepth,
    /// The header color mode of the file this document was read from, when that
    /// mode was normalized to the working mode on read. `None` for a Grayscale
    /// or RGB file and for a constructed document.
    pub source_mode: Option<ColorMode>,
    /// The header bit depth of the file this document was read from when it was
    /// not 8-bit. `None` for an 8-bit file, a depth-1 Bitmap file (whose mode
    /// `source_mode` records), and a constructed document; `Some(16/32)` for a
    /// 16/32-bit file of any mode, including a converted CMYK/Lab read. A
    /// Grayscale/RGB 16/32-bit read also retains native planes and re-emits this
    /// depth on save; a converted mode records it for the notice but retains no
    /// samples and still saves 8-bit.
    pub source_depth: Option<BitDepth>,
    /// Retained source planes of the composite color channels and the document
    /// extra channels, re-emitted on save when the plane is unchanged. Set for a
    /// 16/32-bit Grayscale or RGB read (native-depth samples) and for an 8-bit
    /// Lab read (the Lab color planes, so an unedited Lab file saves exactly).
    /// `None` for an 8-bit RGB/Grayscale read, a constructed document, and a
    /// 16/32-bit converted mode.
    pub source_planes: Option<SourcePlanes>,
    /// The 768-byte Indexed palette (256 red, then green, then blue) read from an
    /// Indexed file, retained so an unchanged document can re-emit it. `None` for
    /// every other mode, an RGB/Grayscale read, and a constructed document.
    /// Deliberately separate from `color_mode_data`, which stays cleared on an
    /// Indexed read (normalization consumes the palette).
    pub source_palette: Option<[u8; 768]>,
    /// The embedded ICC profile bytes of the file this document was read from,
    /// when it was converted to the sRGB working space on read. `None` for a
    /// file with no decodable non-sRGB profile and for a constructed document.
    /// Deliberately not re-emitted on save: the profile is removed from
    /// `image_resources` so the converted sRGB pixels are not mis-tagged.
    pub source_icc: Option<Vec<u8>>,
    /// The ICC bytes of the working profile the stored composite and layer color
    /// numbers are in. `None` means the sRGB working space. Set by an
    /// assign/convert command, or by a Preserve incoming-policy read that kept the
    /// embedded non-sRGB profile; distinct from `source_icc`, which records a
    /// profile a read converted away.
    pub document_icc: Option<Vec<u8>>,
    pub composite: PixelBuffer,
    /// True when the file carried a merged composite image-data section. False
    /// when the file ended after the layer section (maximize-compatibility off),
    /// in which case `composite` is a zero-filled placeholder, not authoritative.
    pub merged_composite_present: bool,
    /// Compression observed for the composite image-data section on read; RLE for
    /// a constructed document. Re-emitted on save to preserve the source encoding.
    pub composite_compression: Compression,
    /// Compression observed for the first engine-encoded layer channel (layer
    /// color channels and the raster mask) on read; RLE for a constructed
    /// document. Re-emitted on save; a document mixing kinds normalizes to the
    /// first seen.
    pub layer_compression: Compression,
    /// True when the document was read from a version-2 PSB container; false for
    /// a PSD and for new/blank documents. Selects the PSB container and its
    /// widened length fields on re-save.
    pub is_psb: bool,
    pub layers: Vec<Layer>,
    /// Document-level extra channels (saved selections / spot channels), which
    /// live after the color channels in the PSD image-data section.
    pub channels: Vec<Channel>,
    /// Verbatim color-mode-data section bytes, preserved for lossless re-save.
    pub color_mode_data: Vec<u8>,
    /// Verbatim image-resource section bytes, preserved for lossless re-save.
    pub image_resources: Vec<u8>,
    /// Global layer mask payload, preserved verbatim for lossless re-save.
    pub global_layer_mask: Vec<u8>,
    /// Bytes after the global layer mask up to the layer-section end, preserved
    /// verbatim for lossless re-save.
    pub layer_section_extra: Vec<u8>,
}

impl Document {
    pub fn new(width: u32, height: u32, mode: ColorMode, depth: BitDepth) -> Self {
        let channels = mode.color_channels();
        Self {
            width,
            height,
            mode,
            depth,
            source_mode: None,
            source_depth: None,
            source_planes: None,
            source_palette: None,
            source_icc: None,
            document_icc: None,
            composite: PixelBuffer::new(width, height, channels),
            merged_composite_present: true,
            composite_compression: Compression::Rle,
            layer_compression: Compression::Rle,
            is_psb: false,
            layers: Vec::new(),
            channels: Vec::new(),
            color_mode_data: Vec::new(),
            image_resources: Vec::new(),
            global_layer_mask: Vec::new(),
            layer_section_extra: Vec::new(),
        }
    }

    /// Build an RGB/8-bit document from packed RGBA8888 bytes (one pixel per
    /// four bytes, byte order R, G, B, A).
    ///
    /// The composite is a 4-plane RGBA [`PixelBuffer`] and the document holds
    /// exactly one pixel layer named `name`, both seeded from `rgba`. A buffer
    /// shorter than `width * height * 4` leaves the missing pixels transparent
    /// instead of panicking.
    pub fn from_rgba(name: &str, width: u32, height: u32, rgba: &[u8]) -> Document {
        let plane = width as usize * height as usize;
        let mut doc = Document::new(width, height, ColorMode::Rgb, BitDepth::Eight);
        let mut composite = vec![0u8; plane * 4];
        let mut channels = vec![
            Channel {
                id: 0,
                data: vec![0; plane],
            },
            Channel {
                id: 1,
                data: vec![0; plane],
            },
            Channel {
                id: 2,
                data: vec![0; plane],
            },
            Channel {
                id: -1,
                data: vec![0; plane],
            },
        ];
        for i in 0..plane {
            let at = i * 4;
            if at + 4 > rgba.len() {
                break;
            }
            for (c, value) in [
                (0, rgba[at]),
                (1, rgba[at + 1]),
                (2, rgba[at + 2]),
                (3, rgba[at + 3]),
            ] {
                channels[c].data[i] = value;
                composite[c * plane + i] = value;
            }
        }
        doc.composite = PixelBuffer {
            width,
            height,
            channels: 4,
            data: composite,
        };
        doc.layers.push(Layer {
            name: name.to_string(),
            rect: PsdRect {
                top: 0,
                left: 0,
                bottom: height as i32,
                right: width as i32,
            },
            channels,
            ..Default::default()
        });
        doc
    }

    /// True when a read retained native 16/32-bit samples for some plane, so
    /// `write_psd` re-emits the source depth. False for an 8-bit read (including
    /// an 8-bit Lab read, whose retained store is 8-bit) and a constructed
    /// document, which still save 8-bit.
    pub fn retains_source_depth(&self) -> bool {
        self.source_planes
            .as_ref()
            .is_some_and(|store| store.depth != BitDepth::Eight)
            || retains_source_depth(&self.layers)
    }
}

/// Whether any layer in a tree retained native 16/32-bit channel samples.
fn retains_source_depth(layers: &[Layer]) -> bool {
    layers.iter().any(|l| {
        l.source_channels
            .as_ref()
            .is_some_and(|store| store.depth != BitDepth::Eight)
            || retains_source_depth(&l.children)
    })
}

impl Default for Document {
    fn default() -> Self {
        Document::new(0, 0, ColorMode::Rgb, BitDepth::Eight)
    }
}

/// A layer blend mode. The 27 modes Photoshop CS6 exposes for a layer, plus the
/// group-only `'pass'` (Pass Through) option. Pass Through is not a layer mode:
/// it is not part of [`BlendMode::LAYER_MODES`] and must only be used on groups.
/// See `MODEL.md` for the key table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    Normal,
    Dissolve,
    Darken,
    Multiply,
    ColorBurn,
    LinearBurn,
    DarkerColor,
    Lighten,
    Screen,
    ColorDodge,
    LinearDodge,
    LighterColor,
    Overlay,
    SoftLight,
    HardLight,
    VividLight,
    LinearLight,
    PinLight,
    HardMix,
    Difference,
    Exclusion,
    Subtract,
    Divide,
    Hue,
    Saturation,
    Color,
    Luminosity,
    /// Group-only: children blend directly against the parent backdrop.
    PassThrough,
}

impl BlendMode {
    /// The 27 layer modes, in PSD-spec table order (same order as the key list
    /// below). [`BlendMode::PassThrough`] is deliberately excluded.
    pub const LAYER_MODES: [BlendMode; 27] = [
        BlendMode::Normal,
        BlendMode::Dissolve,
        BlendMode::Darken,
        BlendMode::Multiply,
        BlendMode::ColorBurn,
        BlendMode::LinearBurn,
        BlendMode::DarkerColor,
        BlendMode::Lighten,
        BlendMode::Screen,
        BlendMode::ColorDodge,
        BlendMode::LinearDodge,
        BlendMode::LighterColor,
        BlendMode::Overlay,
        BlendMode::SoftLight,
        BlendMode::HardLight,
        BlendMode::VividLight,
        BlendMode::LinearLight,
        BlendMode::PinLight,
        BlendMode::HardMix,
        BlendMode::Difference,
        BlendMode::Exclusion,
        BlendMode::Subtract,
        BlendMode::Divide,
        BlendMode::Hue,
        BlendMode::Saturation,
        BlendMode::Color,
        BlendMode::Luminosity,
    ];

    /// The 4-byte PSD blend-mode key.
    pub fn to_psd_key(&self) -> [u8; 4] {
        match self {
            BlendMode::Normal => *b"norm",
            BlendMode::Dissolve => *b"diss",
            BlendMode::Darken => *b"dark",
            BlendMode::Multiply => *b"mul ",
            BlendMode::ColorBurn => *b"idiv",
            BlendMode::LinearBurn => *b"lbrn",
            BlendMode::DarkerColor => *b"dkCl",
            BlendMode::Lighten => *b"lite",
            BlendMode::Screen => *b"scrn",
            BlendMode::ColorDodge => *b"div ",
            BlendMode::LinearDodge => *b"lddg",
            BlendMode::LighterColor => *b"lgCl",
            BlendMode::Overlay => *b"over",
            BlendMode::SoftLight => *b"sLit",
            BlendMode::HardLight => *b"hLit",
            BlendMode::VividLight => *b"vLit",
            BlendMode::LinearLight => *b"lLit",
            BlendMode::PinLight => *b"pLit",
            BlendMode::HardMix => *b"hMix",
            BlendMode::Difference => *b"diff",
            BlendMode::Exclusion => *b"smud",
            BlendMode::Subtract => *b"fsub",
            BlendMode::Divide => *b"fdiv",
            BlendMode::Hue => *b"hue ",
            BlendMode::Saturation => *b"sat ",
            BlendMode::Color => *b"colr",
            BlendMode::Luminosity => *b"lum ",
            BlendMode::PassThrough => *b"pass",
        }
    }

    /// Parse a 4-byte PSD key; `None` for unknown keys.
    pub fn from_psd_key(key: [u8; 4]) -> Option<BlendMode> {
        if key == *b"pass" {
            return Some(BlendMode::PassThrough);
        }
        Self::LAYER_MODES
            .into_iter()
            .find(|m| m.to_psd_key() == key)
    }
}

/// A signed rectangle in PSD coordinates. Bounds may fall outside the canvas,
/// so all edges are `i32` and `right`/`bottom` may be smaller than `left`/`top`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PsdRect {
    pub top: i32,
    pub left: i32,
    pub bottom: i32,
    pub right: i32,
}

impl PsdRect {
    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }
}

/// One planar channel of layer data.
///
/// `id`: `0,1,2…` color, `-1` transparency mask, `-2` user layer mask,
/// `-3` real user mask.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Channel {
    pub id: i16,
    pub data: Vec<u8>,
}

/// The raw additional-layer-info block of an adjustment layer.
///
/// `key` is the 4-byte PSD adjustment key (e.g. `nvrt`, `brit`, `levl`) and
/// `data` is its payload, stored verbatim. `pictura-core` does not interpret
/// the payload; `pictura-render` decodes the subset it understands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdjustmentData {
    pub key: [u8; 4],
    pub data: Vec<u8>,
}

/// An additional-layer-info tagged block the engine does not model, preserved
/// verbatim for lossless re-save.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LayerBlock {
    pub key: [u8; 4],
    pub data: Vec<u8>,
}

/// A layer channel the engine does not decode, stored as its full on-disk
/// stream including the 2-byte compression header for verbatim re-emit.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RawChannel {
    pub id: i16,
    pub data: Vec<u8>,
}

/// Retained source-depth samples of a document's color planes and extra
/// channels, kept so an unchanged plane can be re-emitted exactly.
///
/// `data` is the planar image-data buffer: the composite color channels
/// followed by the document extra channels, each `row_bytes(width, depth) *
/// height` native bytes, in PSD order. Set for a 16/32-bit Grayscale or RGB
/// read, and for an 8-bit Lab read (whose `depth` is `Eight` and whose bytes
/// are the Lab color planes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePlanes {
    pub depth: BitDepth,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

/// A layer's retained native-depth channel samples, keyed by channel id and
/// carrying the layer rect they were decoded for. A saved layer whose rect
/// still matches can re-emit them; a moved layer falls back to widening.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceChannels {
    pub depth: BitDepth,
    pub rect: PsdRect,
    /// `(channel id, plane bytes)` sorted by channel id. A native-depth store
    /// holds the color channels, `-1` transparency, `-2` mask, and unmodeled
    /// ids; an 8-bit Lab store holds only the color channels (`0..3`).
    pub planes: Vec<(i16, Vec<u8>)>,
}

impl SourceChannels {
    /// Build a store, sorting the planes by channel id so equality does not
    /// depend on whether they were captured in read or write emission order.
    pub fn new(depth: BitDepth, rect: PsdRect, mut planes: Vec<(i16, Vec<u8>)>) -> Self {
        planes.sort_by_key(|(id, _)| *id);
        Self {
            depth,
            rect,
            planes,
        }
    }
}

/// How a smart object's source is linked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SmartObjectKind {
    /// `liFD`: the source bytes are embedded in the document.
    Embedded,
    /// `liFE`: the source is an external path; never read from disk.
    External,
    /// `liFA`: an alias link.
    Alias,
    /// A config descriptor exists but no matching linked record was found (or
    /// its parse failed). The preserved bytes remain the source of truth.
    #[default]
    Unresolved,
}

/// A typed view of one smart filter in a layer's `filterFX` list.
///
/// `options` is the canonical re-serialization of the filter's `Fltr`
/// descriptor. The preserved `SoLd` descriptor bytes remain the source of truth
/// for re-emission, so this view is ignored while they exist.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SmartFilter {
    pub filter_id: i32,
    pub name: String,
    pub enabled: bool,
    pub options: Vec<u8>,
}

/// A typed view of a layer's embedded/linked smart object, derived on read.
///
/// The raw config descriptor and the document-level linked record are preserved
/// separately and remain the source of truth for re-emission; this view only
/// exposes what the engine can resolve.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SmartObject {
    /// The `Idnt`/`PlLd` uuid that links the layer config to its record.
    pub uuid: String,
    /// Original filename, with any trailing NUL stripped.
    pub filename: String,
    /// 4-byte file-type code stored in the record (e.g. `8BPB`).
    pub filetype: [u8; 4],
    /// 4-byte creator code stored in the record (e.g. `8BIM`).
    pub creator: [u8; 4],
    pub kind: SmartObjectKind,
    /// Raw `SoLd`/`SoLE`/`plLd`/`PlLd` block data, kept verbatim.
    pub config_descriptor: Vec<u8>,
    /// Embedded source bytes; `Some` only for [`SmartObjectKind::Embedded`].
    pub payload: Option<Vec<u8>>,
    /// The `crs:` XMP packet found in an embedded payload, if any.
    pub crs_xmp: Option<Vec<u8>>,
    /// Typed view of the fixed `crs:` property set in [`SmartObject::crs_xmp`].
    pub crs: Option<CrsSettings>,
    /// Smart filters derived from the descriptor's `filterFX` list. The raw
    /// `config_descriptor` bytes remain the source of truth for re-emission.
    pub smart_filters: Vec<SmartFilter>,
}

/// A raster layer mask. `data` is `None` until the channel image is decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerMask {
    pub rect: PsdRect,
    pub default_color: u8,
    pub disabled: bool,
    pub flags: u8,
    pub data: Option<Vec<u8>>,
    /// Mask block bytes after the fixed 18-byte header, preserved verbatim for
    /// lossless re-save.
    pub extra: Vec<u8>,
}

impl Default for LayerMask {
    fn default() -> Self {
        Self {
            rect: PsdRect {
                top: 0,
                left: 0,
                bottom: 0,
                right: 0,
            },
            default_color: 0,
            disabled: false,
            flags: 0,
            data: None,
            extra: Vec::new(),
        }
    }
}

/// PSD `lclr` sheet color. Values match psd-tools `SheetColorType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ColorLabel {
    None = 0,
    Red = 1,
    Orange = 2,
    Yellow = 3,
    Green = 4,
    Blue = 5,
    Violet = 6,
    Gray = 7,
}

impl ColorLabel {
    pub fn from_byte(v: u8) -> ColorLabel {
        match v {
            0 => ColorLabel::None,
            1 => ColorLabel::Red,
            2 => ColorLabel::Orange,
            3 => ColorLabel::Yellow,
            4 => ColorLabel::Green,
            5 => ColorLabel::Blue,
            6 => ColorLabel::Violet,
            7 => ColorLabel::Gray,
            _ => ColorLabel::None,
        }
    }

    pub fn to_byte(self) -> u8 {
        self as u8
    }
}

/// The four CS6 layer locks. A `u8` bit set, not an enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LockFlags(u8);

impl LockFlags {
    pub const TRANSPARENCY: u8 = 0x01;
    pub const PIXELS: u8 = 0x02;
    pub const POSITION: u8 = 0x04;
    pub const NESTING: u8 = 0x08;

    pub const fn bits(self) -> u8 {
        self.0
    }

    pub fn contains(self, flag: u8) -> bool {
        self.0 & flag != 0
    }

    pub fn with(self, flag: u8, on: bool) -> LockFlags {
        if on {
            LockFlags(self.0 | flag)
        } else {
            LockFlags(self.0 & !flag)
        }
    }

    /// All four lockable bits (the panel's "Lock All" toggle).
    pub fn all() -> LockFlags {
        LockFlags(0x01 | 0x02 | 0x04 | 0x08)
    }

    pub fn is_all(self) -> bool {
        self.0 & 0x0F == 0x0F
    }
}

/// True when a layer's `POSITION` lock forbids moving it.
///
/// Lives in `pictura-core` (the crate every mutation entry shares) so
/// `pictura-paint` need not depend on `pictura-render`.
pub fn layer_move_locked(layer: &Layer) -> bool {
    layer.lock.contains(LockFlags::POSITION)
}

/// True when a layer's `PIXELS` lock forbids mutating its pixels.
pub fn layer_pixel_locked(layer: &Layer) -> bool {
    layer.lock.contains(LockFlags::PIXELS)
}

/// True when a layer's `TRANSPARENCY` lock forbids an alpha-changing edit.
pub fn layer_transparency_locked(layer: &Layer) -> bool {
    layer.lock.contains(LockFlags::TRANSPARENCY)
}

/// A pixel layer or a group (`is_group`). Groups carry `children`, bottom-first
/// like everything else. `rect` is the layer bounds; for groups it may be empty.
///
/// An adjustment layer carries `adjustment` and, in PSD, no color channels (its
/// mask still uses channel `-2`). `adjustment` is opaque to this crate.
#[derive(Debug, Clone, PartialEq)]
pub struct Layer {
    pub name: String,
    pub rect: PsdRect,
    pub blend: BlendMode,
    pub opacity: u8,
    pub fill: u8,
    pub lock: LockFlags,
    pub color: ColorLabel,
    pub clipping: bool,
    pub visible: bool,
    pub mask: Option<LayerMask>,
    pub adjustment: Option<AdjustmentData>,
    pub channels: Vec<Channel>,
    pub children: Vec<Layer>,
    pub is_group: bool,
    /// First-class Background flag. PSD has no background bit; the codec infers
    /// it from the bottom top-level non-group named `"Background"` and writes
    /// that name back for a flagged layer (design D5).
    pub background: bool,
    /// Raw blend-mode key, stored only when it does not map to a known
    /// [`BlendMode`], preserved for lossless re-save.
    pub blend_key: Option<[u8; 4]>,
    /// Raw blending-ranges block bytes, preserved verbatim for lossless re-save.
    pub blending_ranges: Vec<u8>,
    /// `knko` knockout mode (`None` when the block is absent).
    pub knockout: Knockout,
    /// `clbl` blend clipped elements (CS6 default `true` when absent).
    pub blend_clipping: bool,
    /// `infx` blend interior elements (CS6 default `true` when absent).
    pub blend_interior: bool,
    /// Typed Blend If view of `blending_ranges`; `None` when empty or malformed.
    pub blend_if: Option<BlendIf>,
    /// Additional-layer-info tagged blocks the engine does not model, preserved
    /// verbatim for lossless re-save.
    pub extra_blocks: Vec<LayerBlock>,
    /// Layer channels the engine does not decode, kept as full on-disk streams
    /// for verbatim re-emit.
    pub raw_channels: Vec<RawChannel>,
    /// Derived embedded/linked smart-object view; `None` for ordinary layers.
    pub smart_object: Option<SmartObject>,
    /// Derived `vmsk` vector-mask view; `None` when absent or unparseable. The
    /// raw block remains in `extra_blocks` and is the serialization source.
    pub vector_mask: Option<VectorMask>,
    /// Derived `TySh` type-tool view; `None` when absent or unparseable. The
    /// raw block remains in `extra_blocks` and is the serialization source.
    pub type_tool: Option<TypeTool>,
    /// Retained source channel samples, re-emitted on save when the layer has
    /// not moved: native `16`/`32`-bit samples for a Grayscale/RGB read, or the
    /// 8-bit Lab color planes for a Lab read.
    pub source_channels: Option<SourceChannels>,
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            name: String::new(),
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
            blend_key: None,
            blending_ranges: Vec::new(),
            knockout: Knockout::None,
            blend_clipping: true,
            blend_interior: true,
            blend_if: None,
            extra_blocks: Vec::new(),
            raw_channels: Vec::new(),
            smart_object: None,
            vector_mask: None,
            type_tool: None,
            source_channels: None,
        }
    }
}

impl Layer {
    pub fn is_group(&self) -> bool {
        self.is_group
    }

    /// The preserved additional-layer-information block keyed `key`, if any.
    pub fn extra_block(&self, key: &[u8; 4]) -> Option<&LayerBlock> {
        self.extra_blocks.iter().find(|b| &b.key == key)
    }

    /// A type layer: any preserved `TySh` (type tool) block is present.
    pub fn is_type(&self) -> bool {
        self.extra_block(b"TySh").is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_27_blend_keys_round_trip() {
        assert_eq!(BlendMode::LAYER_MODES.len(), 27);
        let mut keys: Vec<[u8; 4]> = Vec::new();
        for mode in BlendMode::LAYER_MODES {
            let key = mode.to_psd_key();
            assert_eq!(BlendMode::from_psd_key(key), Some(mode), "{mode:?}");
            keys.push(key);
        }
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), 27, "blend keys must be unique");
        assert!(
            !BlendMode::LAYER_MODES.contains(&BlendMode::PassThrough),
            "Pass Through is group-only, not a 28th layer mode"
        );
    }

    #[test]
    fn pass_through_maps_to_and_from_pass() {
        assert_eq!(BlendMode::PassThrough.to_psd_key(), *b"pass");
        assert_eq!(
            BlendMode::from_psd_key(*b"pass"),
            Some(BlendMode::PassThrough)
        );
    }

    #[test]
    fn known_key_examples() {
        assert_eq!(BlendMode::Normal.to_psd_key(), *b"norm");
        assert_eq!(BlendMode::Multiply.to_psd_key(), *b"mul ");
        assert_eq!(BlendMode::Screen.to_psd_key(), *b"scrn");
        assert_eq!(BlendMode::ColorBurn.to_psd_key(), *b"idiv");
        assert_eq!(BlendMode::Luminosity.to_psd_key(), *b"lum ");
        assert_eq!(BlendMode::from_psd_key(*b"mul "), Some(BlendMode::Multiply));
    }

    #[test]
    fn unknown_blend_key_is_none() {
        assert_eq!(BlendMode::from_psd_key(*b"zzzz"), None);
        assert_eq!(BlendMode::from_psd_key(*b"nrml"), None);
        assert_eq!(BlendMode::from_psd_key(*b"pas "), None);
    }

    #[test]
    fn extra_block_finds_present_key_and_none_for_absent() {
        let layer = Layer {
            extra_blocks: vec![LayerBlock {
                key: *b"lfx2",
                data: vec![1, 2, 3, 4],
            }],
            ..Default::default()
        };
        assert_eq!(
            layer.extra_block(b"lfx2").map(|b| b.data.as_slice()),
            Some(&[1, 2, 3, 4][..])
        );
        assert!(layer.extra_block(b"SoLd").is_none());
    }

    #[test]
    fn group_vs_pixel_layer() {
        let rect = PsdRect {
            top: 0,
            left: 0,
            bottom: 4,
            right: 4,
        };
        let pixel = Layer {
            name: "Pixel".into(),
            rect,
            blend: BlendMode::Normal,
            opacity: 255,
            fill: 255,
            lock: LockFlags::default(),
            color: ColorLabel::None,
            clipping: false,
            visible: true,
            mask: None,
            adjustment: None,
            channels: vec![Channel {
                id: 0,
                data: vec![0; 16],
            }],
            children: Vec::new(),
            is_group: false,
            background: false,
            ..Default::default()
        };
        assert!(!pixel.is_group());
        assert_eq!(pixel.channels.len(), 1);

        let group = Layer {
            name: "Group".into(),
            rect,
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
            children: vec![pixel],
            is_group: true,
            background: false,
            ..Default::default()
        };
        assert!(group.is_group());
        assert_eq!(group.children.len(), 1);
    }

    #[test]
    fn mask_present_and_absent() {
        let rect = PsdRect {
            top: 1,
            left: 2,
            bottom: 3,
            right: 4,
        };
        let bare = Layer {
            name: "bare".into(),
            rect,
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
        };
        assert!(bare.mask.is_none());

        let masked = Layer {
            mask: Some(LayerMask {
                rect,
                default_color: 255,
                disabled: false,
                flags: 0,
                data: Some(vec![255; 4]),
                ..Default::default()
            }),
            ..bare
        };
        let mask = masked.mask.expect("mask present");
        assert_eq!(mask.default_color, 255);
        assert_eq!(mask.data.as_deref(), Some(&[255u8, 255, 255, 255][..]));
    }

    #[test]
    fn rect_width_height_signed_and_offset() {
        let outside = PsdRect {
            top: -10,
            left: -20,
            bottom: 30,
            right: 40,
        };
        assert_eq!(outside.width(), 60);
        assert_eq!(outside.height(), 40);

        let offset = PsdRect {
            top: 100,
            left: 50,
            bottom: 150,
            right: 250,
        };
        assert_eq!(offset.width(), 200);
        assert_eq!(offset.height(), 50);
    }

    #[test]
    fn document_new_has_no_layers_and_keeps_composite() {
        let doc = Document::new(3, 2, ColorMode::Rgb, BitDepth::Eight);
        assert!(doc.layers.is_empty());
        assert_eq!(doc.composite.channels, 3);
    }

    #[test]
    fn field_and_method_is_group_agree() {
        let layer = Layer {
            name: "g".into(),
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
            is_group: true,
            background: false,
            ..Default::default()
        };
        assert!(layer.is_group());
    }

    #[test]
    fn default_layer_attribute_values() {
        let layer = Layer {
            name: "d".into(),
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
        };
        assert_eq!(layer.fill, 255);
        assert_eq!(layer.lock.bits(), 0);
        assert_eq!(layer.color, ColorLabel::None);
        assert!(!layer.background, "background defaults to false");

        let mut flagged = layer.clone();
        flagged.background = true;
        assert!(flagged.clone().background, "the flag is cloned");
    }

    #[test]
    fn color_label_byte_round_trip_and_out_of_range() {
        for v in 0u8..=7 {
            assert_eq!(ColorLabel::from_byte(v).to_byte(), v);
        }
        for v in 8u8..=255 {
            assert_eq!(ColorLabel::from_byte(v), ColorLabel::None);
        }
        assert_eq!(ColorLabel::Red.to_byte(), 1);
        assert_eq!(ColorLabel::Gray.to_byte(), 7);
    }

    #[test]
    fn lock_flags_bits_contains_with_and_all() {
        assert_eq!(LockFlags::all().bits(), 0x0F);
        assert!(LockFlags::all().is_all());
        assert!(!LockFlags::default().is_all());
        let t = LockFlags::default().with(LockFlags::TRANSPARENCY, true);
        assert!(t.contains(LockFlags::TRANSPARENCY));
        assert!(!t.contains(LockFlags::PIXELS));
        assert!(!t.is_all());
        assert_eq!(t.with(LockFlags::TRANSPARENCY, false).bits(), 0);
        assert!(LockFlags::all().contains(LockFlags::PIXELS));
        assert!(LockFlags::all().contains(LockFlags::POSITION));
        assert!(LockFlags::all().contains(LockFlags::NESTING));
        let n = LockFlags::default().with(LockFlags::NESTING, true);
        assert!(n.contains(LockFlags::NESTING));
        assert!(!n.is_all());
    }

    fn channel(layer: &Layer, id: i16) -> &[u8] {
        &layer
            .channels
            .iter()
            .find(|c| c.id == id)
            .expect("channel present")
            .data
    }

    #[test]
    fn from_rgba_sets_size_mode_depth_and_one_layer() {
        let doc = Document::from_rgba("photo", 2, 3, &[0u8; 24]);
        assert_eq!((doc.width, doc.height), (2, 3));
        assert_eq!(doc.mode, ColorMode::Rgb);
        assert_eq!(doc.depth, BitDepth::Eight);
        assert_eq!(doc.composite.channels, 4);
        assert_eq!(doc.layers.len(), 1);
        let layer = &doc.layers[0];
        assert_eq!(layer.name, "photo");
        assert_eq!(
            (
                layer.rect.top,
                layer.rect.left,
                layer.rect.bottom,
                layer.rect.right
            ),
            (0, 0, 3, 2)
        );
        assert!(layer.smart_object.is_none());
        assert!(layer.adjustment.is_none());
        assert!(!layer.is_group);
        assert_eq!(
            layer.channels.iter().map(|c| c.id).collect::<Vec<_>>(),
            vec![0, 1, 2, -1]
        );
    }

    #[test]
    fn from_rgba_preserves_rgba_including_alpha() {
        // 2x1: pixel 0 red opaque, pixel 1 green half-alpha.
        let rgba = [255, 0, 0, 255, 0, 255, 0, 128];
        let doc = Document::from_rgba("px", 2, 1, &rgba);
        let layer = &doc.layers[0];
        assert_eq!(channel(layer, 0), &[255, 0]);
        assert_eq!(channel(layer, 1), &[0, 255]);
        assert_eq!(channel(layer, 2), &[0, 0]);
        assert_eq!(channel(layer, -1), &[255, 128]);
        let plane = 2;
        assert_eq!(&doc.composite.data[0..2], &[255, 0]);
        assert_eq!(&doc.composite.data[plane..plane + 2], &[0, 255]);
        assert_eq!(&doc.composite.data[3 * plane..3 * plane + 2], &[255, 128]);
    }

    #[test]
    fn from_rgba_short_buffer_does_not_panic() {
        let doc = Document::from_rgba("short", 2, 2, &[1, 2, 3, 4, 5]);
        assert_eq!(doc.layers.len(), 1);
        let layer = &doc.layers[0];
        assert_eq!(channel(layer, 0)[0], 1);
        assert_eq!(channel(layer, -1)[0], 4);
        assert_eq!(channel(layer, 0)[1], 0, "missing pixels stay transparent");
        assert_eq!(channel(layer, -1)[3], 0);
        assert_eq!(doc.composite.data.len(), 16);
    }

    #[test]
    fn retains_source_depth_tracks_the_retained_store() {
        let mut doc = Document::new(1, 1, ColorMode::Rgb, BitDepth::Eight);
        assert!(!doc.retains_source_depth(), "8-bit retains nothing");
        // A converted mode records the depth for the notice but keeps no samples.
        doc.source_depth = Some(BitDepth::Sixteen);
        assert!(!doc.retains_source_depth());
        doc.source_planes = Some(SourcePlanes {
            depth: BitDepth::Sixteen,
            width: 1,
            height: 1,
            data: vec![0, 0],
        });
        assert!(doc.retains_source_depth());

        // A layer store alone also counts (a layered file with no composite).
        let mut layered = Document::new(1, 1, ColorMode::Rgb, BitDepth::Sixteen);
        layered.source_depth = Some(BitDepth::Sixteen);
        layered.layers.push(Layer {
            source_channels: Some(SourceChannels {
                depth: BitDepth::Sixteen,
                rect: PsdRect {
                    top: 0,
                    left: 0,
                    bottom: 1,
                    right: 1,
                },
                planes: vec![(0, vec![0, 0])],
            }),
            ..Default::default()
        });
        assert!(layered.retains_source_depth());
    }
}
