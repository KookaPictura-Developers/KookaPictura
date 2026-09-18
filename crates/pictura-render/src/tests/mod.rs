use super::*;
use pictura_adjust::{Adjustment, BrightnessContrastParams, HueSaturationParams, LevelsParams};
use pictura_core::{
    AdjustmentData, BitDepth, BlendMode, Channel, ColorLabel, ColorMode, Document, Layer,
    LayerMask, LockFlags, PixelBuffer, PsdRect,
};

mod adjustment;
mod blend;
mod composite;
mod rasterize;

fn rect(top: i32, left: i32, bottom: i32, right: i32) -> PsdRect {
    PsdRect {
        top,
        left,
        bottom,
        right,
    }
}

fn full(w: u32, h: u32) -> PsdRect {
    rect(0, 0, h as i32, w as i32)
}

fn solid(
    name: &str,
    r: PsdRect,
    rgb: (u8, u8, u8),
    alpha: u8,
    blend: BlendMode,
    opacity: u8,
) -> Layer {
    let w = r.width().max(0) as usize;
    let h = r.height().max(0) as usize;
    let n = w * h;
    Layer {
        name: name.into(),
        rect: r,
        blend,
        opacity,
        fill: 255,
        lock: LockFlags::default(),
        color: ColorLabel::None,
        clipping: false,
        visible: true,
        mask: None,
        adjustment: None,
        channels: vec![
            Channel {
                id: 0,
                data: vec![rgb.0; n],
            },
            Channel {
                id: 1,
                data: vec![rgb.1; n],
            },
            Channel {
                id: 2,
                data: vec![rgb.2; n],
            },
            Channel {
                id: -1,
                data: vec![alpha; n],
            },
        ],
        children: Vec::new(),
        is_group: false,
        background: false,
    }
}

fn group(
    name: &str,
    blend: BlendMode,
    opacity: u8,
    mask: Option<LayerMask>,
    children: Vec<Layer>,
) -> Layer {
    Layer {
        name: name.into(),
        rect: rect(0, 0, 0, 0),
        blend,
        opacity,
        fill: 255,
        lock: LockFlags::default(),
        color: ColorLabel::None,
        clipping: false,
        visible: true,
        mask,
        adjustment: None,
        channels: Vec::new(),
        children,
        is_group: true,
        background: false,
    }
}

fn doc(w: u32, h: u32, layers: Vec<Layer>) -> Document {
    let mut d = Document::new(w, h, ColorMode::Rgb, BitDepth::Eight);
    d.layers = layers;
    d
}

fn px(buf: &PixelBuffer, x: u32, y: u32) -> [u8; 4] {
    let plane = (buf.width * buf.height) as usize;
    let i = (y * buf.width + x) as usize;
    [
        buf.data[i],
        buf.data[plane + i],
        buf.data[2 * plane + i],
        buf.data[3 * plane + i],
    ]
}

fn assert_blend(mode: BlendMode, cb: [f32; 3], cs: [f32; 3], expected: [f32; 3]) {
    let got = blend(mode, cb, cs);
    for i in 0..3 {
        assert!(
            (got[i] - expected[i]).abs() <= 1.0 / 255.0 + 1e-6,
            "{mode:?}[{i}]: got {} want {}",
            got[i],
            expected[i]
        );
    }
}

fn adjdata(key: [u8; 4], data: Vec<u8>) -> AdjustmentData {
    AdjustmentData { key, data }
}

fn adjustment_layer(
    name: &str,
    key: [u8; 4],
    data: Vec<u8>,
    opacity: u8,
    mask: Option<LayerMask>,
) -> Layer {
    Layer {
        name: name.into(),
        rect: rect(0, 0, 0, 0),
        blend: BlendMode::Normal,
        opacity,
        fill: 255,
        lock: LockFlags::default(),
        color: ColorLabel::None,
        clipping: false,
        visible: true,
        mask,
        adjustment: Some(adjdata(key, data)),
        channels: Vec::new(),
        children: Vec::new(),
        is_group: false,
        background: false,
    }
}

fn rgb(buf: &PixelBuffer, x: u32, y: u32) -> [u8; 3] {
    let p = px(buf, x, y);
    [p[0], p[1], p[2]]
}
