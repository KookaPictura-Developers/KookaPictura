//! GPU vs CPU parity for the separable blend modes.
//!
//! The CPU compositor is the oracle. Each scene drives one blend mode with a
//! gradient backdrop and a partially transparent gradient source, and the GPU
//! output must match `composite_rgba` within ±1 LSB per channel.
//!
//! Every test skips with a printed note when no Vulkan adapter is usable, so a
//! GPU-less CI stays green. On the target machine (RTX 3090, Vulkan) the
//! parity path actually runs.

use pictura_core::{BitDepth, BlendMode, Channel, ColorMode, Document, Layer, PsdRect};
use pictura_render::{composite_gpu, composite_gpu_or_cpu, composite_rgba, GpuError};

const SIZE: u32 = 8;

/// Normal + the separable modes the GPU implements, including the whole-RGB
/// DarkerColor/LighterColor comparisons.
const GPU_MODES: &[BlendMode] = &[
    BlendMode::Normal,
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
];

const CPU_ONLY: &[BlendMode] = &[
    BlendMode::Dissolve,
    BlendMode::Hue,
    BlendMode::Saturation,
    BlendMode::Color,
    BlendMode::Luminosity,
];

fn layer(name: &str, blend: BlendMode, sample: impl Fn(u32, u32) -> (u8, u8, u8, u8)) -> Layer {
    let pixels = (SIZE * SIZE) as usize;
    let (mut r, mut g, mut b, mut a) = (
        Vec::with_capacity(pixels),
        Vec::with_capacity(pixels),
        Vec::with_capacity(pixels),
        Vec::with_capacity(pixels),
    );
    for y in 0..SIZE {
        for x in 0..SIZE {
            let (rr, gg, bb, aa) = sample(x, y);
            r.push(rr);
            g.push(gg);
            b.push(bb);
            a.push(aa);
        }
    }
    Layer {
        name: name.into(),
        rect: PsdRect {
            top: 0,
            left: 0,
            bottom: SIZE as i32,
            right: SIZE as i32,
        },
        blend,
        opacity: 255,
        clipping: false,
        visible: true,
        mask: None,
        channels: vec![
            Channel { id: 0, data: r },
            Channel { id: 1, data: g },
            Channel { id: 2, data: b },
            Channel { id: -1, data: a },
        ],
        children: Vec::new(),
        is_group: false,
    }
}

/// Opaque gradient backdrop plus a half-transparent gradient source in `mode`.
fn scene(mode: BlendMode) -> Document {
    let mut doc = Document::new(SIZE, SIZE, ColorMode::Rgb, BitDepth::Eight);
    let base = layer("base", BlendMode::Normal, |x, y| {
        (x as u8 * 32, y as u8 * 32, (x + y) as u8 * 16, 255)
    });
    let top = layer("top", mode, |x, y| {
        (
            x as u8 * 16 + 8,
            y as u8 * 16 + 8,
            255 - x as u8 * 16,
            64 + x as u8 * 16,
        )
    });
    doc.layers = vec![base, top];
    doc
}

fn is_gpu_gone(err: &GpuError) -> bool {
    matches!(
        err,
        GpuError::Unavailable | GpuError::TooLarge | GpuError::Readback
    )
}

#[test]
fn gpu_matches_cpu_within_one_lsb() {
    let mut checked = 0;
    let mut max_delta = 0i32;
    for &mode in GPU_MODES {
        let doc = scene(mode);
        let cpu = composite_rgba(&doc);
        let gpu = match composite_gpu(&doc) {
            Ok(buf) => buf,
            Err(GpuError::UnsupportedMode(m)) => {
                panic!("{mode:?} dispatched to CPU-only mode {m:?}")
            }
            Err(e) if is_gpu_gone(&e) => {
                eprintln!("no usable Vulkan GPU ({e}); skipping parity test");
                return;
            }
            Err(e) => panic!("{mode:?}: {e}"),
        };
        assert_eq!((gpu.width, gpu.height, gpu.channels), (SIZE, SIZE, 4));
        for (i, (&a, &b)) in cpu.data.iter().zip(&gpu.data).enumerate() {
            let delta = (a as i32 - b as i32).abs();
            max_delta = max_delta.max(delta);
            assert!(delta <= 1, "{mode:?} byte {i}: cpu {a} vs gpu {b}");
        }
        checked += 1;
    }
    eprintln!("gpu parity: {checked} modes ok, max delta {max_delta} LSB");
}

#[test]
fn cpu_only_modes_error_without_panicking() {
    for &mode in CPU_ONLY {
        match composite_gpu(&scene(mode)) {
            Err(GpuError::UnsupportedMode(m)) => assert_eq!(m, mode),
            other => panic!("{mode:?}: expected UnsupportedMode, got {other:?}"),
        }
    }
}

#[test]
fn fallback_matches_cpu_for_cpu_only_mode() {
    let doc = scene(BlendMode::Luminosity);
    assert_eq!(composite_gpu_or_cpu(&doc).data, composite_rgba(&doc).data);
}
