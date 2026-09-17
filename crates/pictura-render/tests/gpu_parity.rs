//! GPU vs CPU parity for the separable blend modes.
//!
//! The CPU compositor is the oracle. Each scene drives one blend mode with a
//! gradient backdrop and a partially transparent gradient source, and the GPU
//! output must match `composite_rgba` within ±1 LSB per channel.
//!
//! Every test skips with a printed note when no Vulkan adapter is usable, so a
//! GPU-less CI stays green. On the target machine (RTX 3090, Vulkan) the
//! parity path actually runs.
//!
//! M31 adds region compositing: a region-limited GPU composite must be
//! byte-identical to the same slice of the full `composite_active` result,
//! because it is the same per-pixel kernel over a region-sized canvas.

use std::time::Instant;

use pictura_core::{
    AdjustmentData, BitDepth, BlendMode, Channel, ColorLabel, ColorMode, Document, Layer,
    LayerMask, LockFlags, PixelBuffer, PsdRect,
};
use pictura_render::{
    composite_active, composite_gpu, composite_gpu_or_cpu, composite_region_active, composite_rgba,
    encode_invert, gpu_available, Backend, GpuError,
};

const SIZE: u32 = 8;

/// Normal + every blend mode the GPU implements, including the whole-RGB
/// DarkerColor/LighterColor comparisons and the four non-separable modes.
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
    BlendMode::Hue,
    BlendMode::Saturation,
    BlendMode::Color,
    BlendMode::Luminosity,
];

const CPU_ONLY: &[BlendMode] = &[BlendMode::Dissolve];

/// Heavy hardware evidence tests are opt-in. Set `PICTURA_GPU_BENCH=1` to run the
/// 4000²/2048/1024 timing and large-document tests; the default `cargo test` on a
/// GPU box stays correctness-only.
fn bench_enabled() -> bool {
    std::env::var_os("PICTURA_GPU_BENCH").is_some()
}

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
        fill: 255,
        lock: LockFlags::default(),
        color: ColorLabel::None,
        clipping: false,
        visible: true,
        mask: None,
        adjustment: None,
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

/// An empty-rect layer with children, for the group dispatch paths.
fn group(name: &str, blend: BlendMode, opacity: u8, children: Vec<Layer>) -> Layer {
    Layer {
        name: name.into(),
        rect: PsdRect {
            top: 0,
            left: 0,
            bottom: 0,
            right: 0,
        },
        blend,
        opacity,
        fill: 255,
        lock: LockFlags::default(),
        color: ColorLabel::None,
        clipping: false,
        visible: true,
        mask: None,
        adjustment: None,
        channels: Vec::new(),
        children,
        is_group: true,
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

/// Opaque gradient base layer for the adjustment scenes.
fn base_layer() -> Layer {
    layer("base", BlendMode::Normal, |x, y| {
        (
            x as u8 * 30 + 5,
            y as u8 * 30 + 5,
            x as u8 * 16 + y as u8 * 8,
            255,
        )
    })
}

/// An adjustment layer owns no pixels: an empty rect and no channels.
fn adjustment_layer(name: &str, adjustment: AdjustmentData) -> Layer {
    Layer {
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
        mask: None,
        adjustment: Some(adjustment),
        channels: Vec::new(),
        children: Vec::new(),
        is_group: false,
    }
}

/// A full-size pixel layer for the timing scene (the `layer` helper is fixed at
/// `SIZE`). Seed 0 is the opaque backdrop; the rest are partially transparent.
fn timing_layer(size: u32, seed: u32, blend: BlendMode) -> Layer {
    large_layer(size, size, seed, blend)
}

/// A full-size `w×h` pixel layer, for the large-document (non-square) scenes.
fn large_layer(w: u32, h: u32, seed: u32, blend: BlendMode) -> Layer {
    let n = (w as usize) * (h as usize);
    let (mut r, mut g, mut b, mut a) = (vec![0u8; n], vec![0u8; n], vec![0u8; n], vec![0u8; n]);
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            r[i] = (x * 7 + seed * 31) as u8;
            g[i] = (y * 5 + seed * 17) as u8;
            b[i] = (x ^ y) as u8;
            a[i] = if seed == 0 { 255 } else { 160 };
        }
    }
    Layer {
        name: format!("layer{seed}"),
        rect: PsdRect {
            top: 0,
            left: 0,
            bottom: h as i32,
            right: w as i32,
        },
        blend,
        opacity: 255,
        fill: 255,
        lock: LockFlags::default(),
        color: ColorLabel::None,
        clipping: false,
        visible: true,
        mask: None,
        adjustment: None,
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

fn is_gpu_gone(err: &GpuError) -> bool {
    matches!(
        err,
        GpuError::Unavailable | GpuError::TooLarge | GpuError::Readback
    )
}

/// Assert one scene's GPU result matches the CPU oracle within ±1 LSB, or skip
/// with a printed note when no adapter is usable.
fn check_scene_parity(doc: &Document, max_delta: &mut i32) {
    let cpu = composite_rgba(doc);
    let gpu = match composite_gpu(doc) {
        Ok(buf) => buf,
        Err(e) if is_gpu_gone(&e) => {
            eprintln!("no usable Vulkan GPU ({e}); skipping scene parity");
            return;
        }
        Err(e) => panic!("{e}"),
    };
    assert_eq!(
        (gpu.width, gpu.height, gpu.channels),
        (doc.width, doc.height, 4)
    );
    for (i, (&a, &b)) in cpu.data.iter().zip(&gpu.data).enumerate() {
        let delta = (a as i32 - b as i32).abs();
        *max_delta = (*max_delta).max(delta);
        assert!(delta <= 1, "byte {i}: cpu {a} vs gpu {b}");
    }
}

/// The data path's mask plane and packed-group source are exercised here; the
/// blend/adjustment tests above never bind either.
#[test]
fn groups_and_masks_match_cpu() {
    let mut max_delta = 0i32;

    // Masked, reduced-opacity pixel layer over an opaque backdrop.
    let mut masked = layer("masked", BlendMode::Multiply, |x, y| {
        (x as u8 * 16 + 8, y as u8 * 16 + 8, 255 - x as u8 * 16, 200)
    });
    masked.opacity = 180;
    masked.mask = Some(LayerMask {
        rect: PsdRect {
            top: 0,
            left: 0,
            bottom: SIZE as i32,
            right: SIZE as i32,
        },
        default_color: 0,
        disabled: false,
        flags: 0,
        data: Some((0..SIZE * SIZE).map(|i| (i * 37 % 256) as u8).collect()),
    });
    let mut doc = Document::new(SIZE, SIZE, ColorMode::Rgb, BitDepth::Eight);
    doc.layers = vec![base_layer(), masked];
    check_scene_parity(&doc, &mut max_delta);

    // Isolated group (packed inner canvas source) over the backdrop.
    let mut doc = Document::new(SIZE, SIZE, ColorMode::Rgb, BitDepth::Eight);
    doc.layers = vec![
        base_layer(),
        group(
            "iso",
            BlendMode::Multiply,
            200,
            vec![layer("child", BlendMode::Normal, |x, y| {
                (255 - x as u8 * 20, 40 + y as u8 * 20, x as u8 * 32, 255)
            })],
        ),
    ];
    check_scene_parity(&doc, &mut max_delta);

    // Adjustment layer gated by a mask.
    let mut adj = adjustment_layer("invert", encode_invert());
    adj.mask = Some(LayerMask {
        rect: PsdRect {
            top: 0,
            left: 0,
            bottom: SIZE as i32,
            right: SIZE as i32,
        },
        default_color: 255,
        disabled: false,
        flags: 0,
        data: Some(
            (0..SIZE * SIZE)
                .map(|i| (255 - i * 11 % 256) as u8)
                .collect(),
        ),
    });
    let mut doc = Document::new(SIZE, SIZE, ColorMode::Rgb, BitDepth::Eight);
    doc.layers = vec![base_layer(), adj];
    check_scene_parity(&doc, &mut max_delta);

    eprintln!("group/mask parity: max delta {max_delta} LSB");
}

/// A reduced-fill layer must match the CPU oracle within ±1 LSB; over an opaque
/// backdrop the fill factor scales colour only, so the output alpha stays 255.
#[test]
fn fill_scene_matches_cpu() {
    let mut top = layer("fill", BlendMode::Multiply, |x, y| {
        (x as u8 * 16 + 8, y as u8 * 16 + 8, 255 - x as u8 * 16, 200)
    });
    top.opacity = 180;
    top.fill = 96;
    let mut doc = Document::new(SIZE, SIZE, ColorMode::Rgb, BitDepth::Eight);
    doc.layers = vec![base_layer(), top];

    let mut max_delta = 0i32;
    check_scene_parity(&doc, &mut max_delta);

    let cpu = composite_rgba(&doc);
    let plane = cpu.pixel_count();
    for i in 0..plane {
        assert_eq!(
            cpu.data[3 * plane + i],
            255,
            "fill must not change the alpha over an opaque backdrop at {i}"
        );
    }
    eprintln!("fill parity: max delta {max_delta} LSB, alpha unchanged");
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
    let doc = scene(BlendMode::Dissolve);
    assert_eq!(composite_gpu_or_cpu(&doc).data, composite_rgba(&doc).data);
}

#[test]
fn dissolve_still_cpu_only() {
    assert!(matches!(
        composite_gpu(&scene(BlendMode::Dissolve)),
        Err(GpuError::UnsupportedMode(BlendMode::Dissolve))
    ));
}

#[test]
fn non_separable_modes_match_cpu() {
    let mut checked = 0;
    let mut max_delta = 0i32;
    for &mode in &[
        BlendMode::Hue,
        BlendMode::Saturation,
        BlendMode::Color,
        BlendMode::Luminosity,
    ] {
        let doc = scene(mode);
        let cpu = composite_rgba(&doc);
        let gpu = match composite_gpu(&doc) {
            Ok(buf) => buf,
            Err(e) if is_gpu_gone(&e) => {
                eprintln!("no usable Vulkan GPU ({e}); skipping non-separable parity");
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
    eprintln!("non-separable parity: {checked} modes ok, max delta {max_delta} LSB");
}

#[test]
fn adjustment_layers_match_cpu() {
    use pictura_render::{
        encode_brightness_contrast, encode_hue_saturation, encode_invert, encode_posterize,
        encode_threshold,
    };
    let cases: &[(&str, AdjustmentData)] = &[
        ("invert", encode_invert()),
        ("posterize", encode_posterize(4)),
        ("threshold", encode_threshold(128)),
        ("brightness_contrast", encode_brightness_contrast(20, 0)),
        ("hue_saturation", encode_hue_saturation(30, 0, 0)),
    ];
    let mut checked = 0;
    let mut max_delta = 0i32;
    for (name, data) in cases {
        let mut doc = Document::new(SIZE, SIZE, ColorMode::Rgb, BitDepth::Eight);
        doc.layers = vec![base_layer(), adjustment_layer(name, data.clone())];
        let cpu = composite_rgba(&doc);
        let gpu = match composite_gpu(&doc) {
            Ok(buf) => buf,
            Err(e) if is_gpu_gone(&e) => {
                eprintln!("no usable Vulkan GPU ({e}); skipping adjustment parity");
                return;
            }
            Err(e) => panic!("{name}: {e}"),
        };
        assert_eq!((gpu.width, gpu.height, gpu.channels), (SIZE, SIZE, 4));
        let mut case_delta = 0i32;
        for (i, (&a, &b)) in cpu.data.iter().zip(&gpu.data).enumerate() {
            let delta = (a as i32 - b as i32).abs();
            case_delta = case_delta.max(delta);
            max_delta = max_delta.max(delta);
            assert!(delta <= 1, "{name} byte {i}: cpu {a} vs gpu {b}");
        }
        eprintln!("adjustment {name}: max delta {case_delta} LSB");
        checked += 1;
    }
    eprintln!("adjustment parity: {checked} kinds ok, max delta {max_delta} LSB");
}

#[test]
fn unsupported_adjustment_falls_back_or_errors() {
    // A key the renderer does not decode, and one that decodes but the GPU has
    // no kind for (Levels): both must error and fall back without panicking.
    let mut levels = vec![0, 2];
    for v in [5u16, 250, 10, 240, 120] {
        levels.extend_from_slice(&v.to_be_bytes());
    }
    let cases = [
        AdjustmentData {
            key: *b"clrL",
            data: vec![1, 2, 3],
        },
        AdjustmentData {
            key: *b"levl",
            data: levels,
        },
    ];
    for data in cases {
        let mut doc = Document::new(SIZE, SIZE, ColorMode::Rgb, BitDepth::Eight);
        doc.layers = vec![base_layer(), adjustment_layer("unsupported", data)];
        assert!(
            matches!(composite_gpu(&doc), Err(GpuError::UnsupportedAdjustment)),
            "expected UnsupportedAdjustment"
        );
        assert_eq!(
            composite_gpu_or_cpu(&doc).data,
            composite_rgba(&doc).data,
            "fallback must equal the CPU oracle"
        );
    }
}

#[test]
fn disabled_gpu_returns_cpu_backend() {
    let doc = scene(BlendMode::Normal);
    let (buf, backend) = composite_active(&doc, false);
    assert_eq!(backend, Backend::Cpu);
    assert_eq!(buf.data, composite_rgba(&doc).data);
}

#[test]
fn enabled_gpu_matches_availability() {
    let doc = scene(BlendMode::Normal);
    let cpu = composite_rgba(&doc);
    let (buf, backend) = composite_active(&doc, true);
    if gpu_available() {
        assert_eq!(backend, Backend::Gpu);
        assert_eq!((buf.width, buf.height, buf.channels), (SIZE, SIZE, 4));
        for (i, (&a, &b)) in cpu.data.iter().zip(&buf.data).enumerate() {
            assert!(
                (a as i32 - b as i32).abs() <= 1,
                "byte {i}: cpu {a} vs gpu {b}"
            );
        }
    } else {
        assert_eq!(backend, Backend::Cpu);
        assert_eq!(buf.data, cpu.data);
    }
}

/// The M29 regression: 4096×2048 is 8.4 MP ⇒ 131072 one-dimensional
/// workgroups, past the 65535 limit the pre-M29 stack rejected. The 2-D grid
/// must composite it on the GPU within ±1 LSB of the CPU oracle.
#[test]
fn large_document_over_1d_limit_composites_on_gpu() {
    if !bench_enabled() {
        eprintln!("skipping large-document parity: set PICTURA_GPU_BENCH=1 to run");
        return;
    }
    const W: u32 = 4096;
    const H: u32 = 2048;
    let mut doc = Document::new(W, H, ColorMode::Rgb, BitDepth::Eight);
    doc.layers = vec![
        large_layer(W, H, 0, BlendMode::Normal),
        large_layer(W, H, 1, BlendMode::Multiply),
    ];
    if !gpu_available() {
        eprintln!("no usable Vulkan GPU; skipping large-document parity");
        return;
    }
    let (gpu, backend) = composite_active(&doc, true);
    assert_eq!(
        backend,
        Backend::Gpu,
        "2-D dispatch must lift the 1-D ceiling"
    );
    assert_eq!((gpu.width, gpu.height, gpu.channels), (W, H, 4));
    let cpu = composite_rgba(&doc);
    let mut max_delta = 0i32;
    for (i, (&a, &b)) in cpu.data.iter().zip(&gpu.data).enumerate() {
        let d = (a as i32 - b as i32).abs();
        max_delta = max_delta.max(d);
        assert!(d <= 1, "byte {i}: cpu {a} vs gpu {b}");
    }
    eprintln!("large {W}x{H} parity: max delta {max_delta} LSB, backend {backend:?}");
}

/// 4000×4000 is 16.7 M px ⇒ 250000 one-dimensional workgroups. Asserts the GPU
/// backend, full ±1 LSB parity against the CPU oracle, and reports the
/// GPU-vs-CPU `composite_active` wall time.
#[test]
fn full_4000_document_composites_on_gpu() {
    if !bench_enabled() {
        eprintln!("skipping 4000² parity: set PICTURA_GPU_BENCH=1 to run");
        return;
    }
    const N: u32 = 4000;
    let mut doc = Document::new(N, N, ColorMode::Rgb, BitDepth::Eight);
    doc.layers = vec![
        large_layer(N, N, 0, BlendMode::Normal),
        large_layer(N, N, 1, BlendMode::Screen),
        large_layer(N, N, 2, BlendMode::Overlay),
    ];
    if !gpu_available() {
        eprintln!("no usable Vulkan GPU; skipping 4000² parity");
        return;
    }

    // One warm-up call keeps device/pipeline setup out of the GPU number; the
    // timed GPU buffer doubles as the parity buffer.
    let _ = composite_active(&doc, true);
    let t = Instant::now();
    let (gpu, backend) = composite_active(&doc, true);
    let gpu_ms = t.elapsed().as_millis();
    assert_eq!(backend, Backend::Gpu, "4000² must run on the GPU");
    let t = Instant::now();
    let cpu = composite_rgba(&doc);
    let cpu_ms = t.elapsed().as_millis();

    let mut max_delta = 0i32;
    for (i, (&a, &b)) in cpu.data.iter().zip(&gpu.data).enumerate() {
        let d = (a as i32 - b as i32).abs();
        max_delta = max_delta.max(d);
        assert!(d <= 1, "byte {i}: cpu {a} vs gpu {b}");
    }
    eprintln!("4000x4000x3layers: cpu {cpu_ms} ms, gpu {gpu_ms} ms, max delta {max_delta} LSB");
}

/// `limit² × 64` exceeds `u32::MAX`, so no real document reaches the 2-D product
/// guard; a struct-literal document with oversized dimensions exercises the
/// limit path with no large allocation. Must return `TooLarge` and not panic.
#[test]
fn document_past_2d_product_limit_returns_too_large() {
    if !gpu_available() {
        eprintln!("no usable Vulkan GPU; skipping TooLarge check");
        return;
    }
    let doc = Document {
        width: u32::MAX,
        height: u32::MAX,
        mode: ColorMode::Rgb,
        depth: BitDepth::Eight,
        composite: PixelBuffer::new(0, 0, 3),
        layers: Vec::new(),
        channels: Vec::new(),
    };
    assert!(matches!(composite_gpu(&doc), Err(GpuError::TooLarge)));
}

/// Evidence only: no ratio is asserted (timings vary by machine and load). One
/// warm-up call keeps one-time device/ pipeline setup out of the GPU number.
#[test]
fn gpu_vs_cpu_timing_1024() {
    if !bench_enabled() {
        println!("skipping m27 1024 timing: set PICTURA_GPU_BENCH=1 to run");
        return;
    }
    const N: u32 = 1024;
    let modes = [
        BlendMode::Normal,
        BlendMode::Multiply,
        BlendMode::Screen,
        BlendMode::Overlay,
    ];
    let mut doc = Document::new(N, N, ColorMode::Rgb, BitDepth::Eight);
    doc.layers = modes
        .iter()
        .enumerate()
        .map(|(i, &blend)| timing_layer(N, i as u32, blend))
        .collect();

    let t = Instant::now();
    let _ = composite_rgba(&doc);
    let cpu_ms = t.elapsed().as_millis();

    if !gpu_available() {
        println!("m27 timing {N}x{N}x4layers: cpu {cpu_ms} ms, gpu n/a");
        return;
    }
    let warm = composite_gpu(&doc);
    if let Err(e) = &warm {
        if is_gpu_gone(e) {
            println!("m27 timing {N}x{N}x4layers: cpu {cpu_ms} ms, gpu n/a");
            return;
        }
    }
    drop(warm);

    let t = Instant::now();
    match composite_gpu(&doc) {
        Ok(_) => {
            let gpu_ms = t.elapsed().as_millis();
            println!("m27 timing {N}x{N}x4layers: cpu {cpu_ms} ms, gpu {gpu_ms} ms");
        }
        Err(e) => println!("m27 timing {N}x{N}x4layers: cpu {cpu_ms} ms, gpu n/a ({e})"),
    }
}

// --- M31: region compositing -----------------------------------------------

fn whole_rect() -> PsdRect {
    PsdRect {
        top: 0,
        left: 0,
        bottom: SIZE as i32,
        right: SIZE as i32,
    }
}

fn region_rect(top: i32, left: i32, bottom: i32, right: i32) -> PsdRect {
    PsdRect {
        top,
        left,
        bottom,
        right,
    }
}

/// One document per scene kind the region path must preserve: separable,
/// non-separable, adjustment, masked, and an isolated group.
fn region_scenes() -> Vec<(&'static str, Document)> {
    let separable = scene(BlendMode::Multiply);
    let nonseparable = scene(BlendMode::Hue);

    let mut adjustment = Document::new(SIZE, SIZE, ColorMode::Rgb, BitDepth::Eight);
    adjustment.layers = vec![base_layer(), adjustment_layer("invert", encode_invert())];

    let mut masked_layer = layer("masked", BlendMode::Multiply, |x, y| {
        (x as u8 * 16 + 8, y as u8 * 16 + 8, 255 - x as u8 * 16, 200)
    });
    masked_layer.opacity = 180;
    masked_layer.mask = Some(LayerMask {
        rect: whole_rect(),
        default_color: 0,
        disabled: false,
        flags: 0,
        data: Some((0..SIZE * SIZE).map(|i| (i * 37 % 256) as u8).collect()),
    });
    let mut masked = Document::new(SIZE, SIZE, ColorMode::Rgb, BitDepth::Eight);
    masked.layers = vec![base_layer(), masked_layer];

    let mut iso = Document::new(SIZE, SIZE, ColorMode::Rgb, BitDepth::Eight);
    iso.layers = vec![
        base_layer(),
        group(
            "iso",
            BlendMode::Multiply,
            200,
            vec![layer("child", BlendMode::Normal, |x, y| {
                (255 - x as u8 * 20, 40 + y as u8 * 20, x as u8 * 32, 255)
            })],
        ),
    ];

    vec![
        ("separable_multiply", separable),
        ("nonseparable_hue", nonseparable),
        ("adjustment_invert", adjustment),
        ("masked_layer", masked),
        ("isolated_group", iso),
    ]
}

/// Assert `composite_region_active(doc, r, true)` equals the `r` slice of the
/// already-composited `full` buffer, byte for byte.
fn assert_region_slice(doc: &Document, full: &PixelBuffer, r: PsdRect, ctx: &str) {
    let (region, backend) = composite_region_active(doc, r, true);
    assert_eq!(backend, Backend::Gpu, "{ctx}: region must run on the GPU");
    let x0 = r.left.max(0) as usize;
    let y0 = r.top.max(0) as usize;
    let x1 = r.right.min(doc.width as i32).max(0) as usize;
    let y1 = r.bottom.min(doc.height as i32).max(0) as usize;
    let rw = x1 - x0;
    let rh = y1 - y0;
    assert_eq!(
        (region.width as usize, region.height as usize),
        (rw, rh),
        "{ctx}: region size"
    );

    let fplane = full.pixel_count();
    let rplane = region.pixel_count();
    let mut max_delta = 0i32;
    let mut samples = 0usize;
    for ry in 0..rh {
        for rx in 0..rw {
            let fi = (y0 + ry) * doc.width as usize + x0 + rx;
            let ri = ry * rw + rx;
            for c in 0..4 {
                let d =
                    (full.data[c * fplane + fi] as i32 - region.data[c * rplane + ri] as i32).abs();
                max_delta = max_delta.max(d);
                samples += 1;
            }
        }
    }
    assert_eq!(
        max_delta, 0,
        "{ctx}: region differs from the full-active slice by {max_delta} LSB over {samples} samples"
    );
}

#[test]
fn region_matches_full_slice() {
    if !gpu_available() {
        eprintln!("no usable Vulkan GPU; skipping region parity");
        return;
    }
    let rects = [
        region_rect(0, 0, 4, 4),
        region_rect(1, 2, 7, 6),
        region_rect(3, 5, SIZE as i32, SIZE as i32),
        region_rect(4, 0, SIZE as i32, SIZE as i32),
        region_rect(7, 7, SIZE as i32, SIZE as i32),
    ];
    let scenes = region_scenes();
    let count = scenes.len();
    for (name, doc) in scenes {
        let (full, backend) = composite_active(&doc, true);
        assert_eq!(backend, Backend::Gpu, "{name}: full must run on the GPU");
        for &r in &rects {
            assert_region_slice(&doc, &full, r, name);
        }
    }
    eprintln!("region parity: {count} scenes byte-identical to the full-active slice");
}

#[test]
fn full_region_equals_full_composite() {
    if !gpu_available() {
        eprintln!("no usable Vulkan GPU; skipping full-region parity");
        return;
    }
    for (name, doc) in region_scenes() {
        let (full, fb) = composite_active(&doc, true);
        let (region, rb) = composite_region_active(&doc, whole_rect(), true);
        assert_eq!(fb, Backend::Gpu, "{name}: full backend");
        assert_eq!(rb, fb, "{name}: region backend");
        assert_eq!((region.width, region.height), (full.width, full.height));
        assert_eq!(
            region.data, full.data,
            "{name}: whole-doc region != full composite"
        );
    }
    eprintln!("full-region parity: every scene byte-identical");
}

#[test]
fn empty_and_out_of_bounds_region_are_safe() {
    let doc = scene(BlendMode::Multiply);
    let (full, _) = composite_active(&doc, false);
    for (name, r) in [
        ("zero_area", region_rect(0, 0, 0, 0)),
        ("inverted", region_rect(5, 5, 2, 3)),
        ("fully_outside", region_rect(-10, -10, -5, -5)),
        ("past_bottom_right", region_rect(20, 20, 30, 30)),
    ] {
        let (buf, backend) = composite_region_active(&doc, r, false);
        assert_eq!(backend, Backend::Cpu, "{name}");
        assert_eq!(
            (buf.width, buf.height),
            (0, 0),
            "{name} must be zero-dimension"
        );
        assert!(buf.data.is_empty(), "{name} must have no data");
    }

    // Partial overlap clamps to the intersection and matches the full slice.
    let (buf, _) = composite_region_active(&doc, region_rect(-5, -5, 3, 3), false);
    assert_eq!((buf.width, buf.height), (3, 3));
    let fplane = full.pixel_count();
    let rplane = buf.pixel_count();
    for y in 0..3usize {
        for x in 0..3usize {
            for c in 0..4 {
                assert_eq!(
                    buf.data[c * rplane + y * 3 + x],
                    full.data[c * fplane + y * SIZE as usize + x]
                );
            }
        }
    }

    // Extending past the far edge clamps to what fits.
    let (buf, _) = composite_region_active(&doc, region_rect(6, 6, 20, 20), false);
    assert_eq!((buf.width, buf.height), (2, 2));
    eprintln!("empty/oob region: clamped and zero-dimension without panic");
}

/// Evidence only: records the region-vs-full win at 4000². No ratio is asserted
/// because timings vary by machine and load.
#[test]
fn region_vs_full_timing_4000() {
    if !bench_enabled() {
        eprintln!("skipping region timing: set PICTURA_GPU_BENCH=1 to run");
        return;
    }
    const N: u32 = 4000;
    if !gpu_available() {
        eprintln!("no usable Vulkan GPU; skipping region timing");
        return;
    }
    let mut doc = Document::new(N, N, ColorMode::Rgb, BitDepth::Eight);
    doc.layers = vec![
        large_layer(N, N, 0, BlendMode::Normal),
        large_layer(N, N, 1, BlendMode::Multiply),
    ];
    let _ = composite_active(&doc, true); // warm-up: keep device/pipeline setup out of the numbers

    let t = Instant::now();
    let (full, backend) = composite_active(&doc, true);
    let full_ms = t.elapsed().as_millis();
    assert_eq!(backend, Backend::Gpu);

    let dirty = region_rect(1200, 1500, 1712, 2012); // 512×512
    let t = Instant::now();
    let (region, rbackend) = composite_region_active(&doc, dirty, true);
    let region_ms = t.elapsed().as_millis();
    assert_eq!(rbackend, Backend::Gpu);
    assert_eq!((region.width, region.height), (512, 512));
    let ratio = if region_ms > 0 {
        format!("{}x", full_ms / region_ms.max(1))
    } else {
        "n/a".into()
    };
    eprintln!("M31 timing 4000²: region 512×512 {region_ms} ms vs full {full_ms} ms ({ratio})");
    drop(full);
}
