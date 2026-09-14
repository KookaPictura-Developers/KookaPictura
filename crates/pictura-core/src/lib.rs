//! Core document types for Kooka Pictura.
//!
//! M0 scope is deliberately tiny: an 8-bit RGB/gray composite image. The full
//! layer/channel/mask model is specified in `docs/01-architecture/document-model.md`
//! and is **not** implemented here yet.

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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub width: u32,
    pub height: u32,
    pub mode: ColorMode,
    pub depth: BitDepth,
    pub composite: PixelBuffer,
    pub layers: Vec<Layer>,
    /// Document-level extra channels (saved selections / spot channels), which
    /// live after the color channels in the PSD image-data section.
    pub channels: Vec<Channel>,
}

impl Document {
    pub fn new(width: u32, height: u32, mode: ColorMode, depth: BitDepth) -> Self {
        let channels = mode.color_channels();
        Self {
            width,
            height,
            mode,
            depth,
            composite: PixelBuffer::new(width, height, channels),
            layers: Vec::new(),
            channels: Vec::new(),
        }
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

/// A raster layer mask. `data` is `None` until the channel image is decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerMask {
    pub rect: PsdRect,
    pub default_color: u8,
    pub disabled: bool,
    pub flags: u8,
    pub data: Option<Vec<u8>>,
}

/// A pixel layer or a group (`is_group`). Groups carry `children`, bottom-first
/// like everything else. `rect` is the layer bounds; for groups it may be empty.
///
/// An adjustment layer carries `adjustment` and, in PSD, no color channels (its
/// mask still uses channel `-2`). `adjustment` is opaque to this crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layer {
    pub name: String,
    pub rect: PsdRect,
    pub blend: BlendMode,
    pub opacity: u8,
    pub clipping: bool,
    pub visible: bool,
    pub mask: Option<LayerMask>,
    pub adjustment: Option<AdjustmentData>,
    pub channels: Vec<Channel>,
    pub children: Vec<Layer>,
    pub is_group: bool,
}

impl Layer {
    pub fn is_group(&self) -> bool {
        self.is_group
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
        };
        assert!(!pixel.is_group());
        assert_eq!(pixel.channels.len(), 1);

        let group = Layer {
            name: "Group".into(),
            rect,
            blend: BlendMode::Normal,
            opacity: 255,
            clipping: false,
            visible: true,
            mask: None,
            adjustment: None,
            channels: Vec::new(),
            children: vec![pixel],
            is_group: true,
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
            clipping: false,
            visible: true,
            mask: None,
            adjustment: None,
            channels: Vec::new(),
            children: Vec::new(),
            is_group: false,
        };
        assert!(bare.mask.is_none());

        let masked = Layer {
            mask: Some(LayerMask {
                rect,
                default_color: 255,
                disabled: false,
                flags: 0,
                data: Some(vec![255; 4]),
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
            clipping: false,
            visible: true,
            mask: None,
            adjustment: None,
            channels: Vec::new(),
            children: Vec::new(),
            is_group: true,
        };
        assert!(layer.is_group());
    }
}
