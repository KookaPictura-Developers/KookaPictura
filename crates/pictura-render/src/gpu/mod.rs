//! GPU (wgpu/Vulkan) compositor for the separable and non-separable blend
//! modes and the supported adjustment layers.
//!
//! `composite_gpu` mirrors the CPU oracle [`crate::composite_rgba`] pass for
//! pass: the same per-layer loop, mask/opacity handling, pass-through/isolated
//! group semantics, and the W3C + Photoshop blend formulas, executed by a
//! wgpu compute shader. It is proven, not assumed — the parity test in
//! `tests/gpu_parity.rs` diffs it against the CPU compositor within ±1 LSB.
//!
//! # Region compositing
//!
//! [`composite_region_active`] runs the same kernel over a clamped document
//! rectangle: the canvas, each layer source, the mask, and group inner canvases
//! are region-sized, and each invocation maps a region-local index to document
//! coordinates for layer-rect and mask sampling. Only the region is dispatched
//! and read back, so the output is byte-identical to the same slice of the full
//! composite. `composite_gpu` is just the region kernel over the whole document.
//!
//! # CPU-only modes
//!
//! Dissolve is the only CPU-only blend mode. [`composite_gpu`] returns
//! [`GpuError::UnsupportedMode`] for a visible layer using it;
//! [`composite_gpu_or_cpu`] then falls back to the CPU compositor. DarkerColor
//! and LighterColor *are* handled (they are per-pixel whole-RGB comparisons, not
//! triplet math), and the four non-separable modes (Hue, Saturation, Color,
//! Luminosity) run the PDF/CSS triplet math on the GPU.
//!
//! # Adjustment layers
//!
//! Invert, Posterize, Threshold, Brightness/Contrast and Hue/Saturation
//! adjustment layers are applied on the GPU at the layer's stack position. Any
//! other adjustment (or an undecodable block) returns
//! [`GpuError::UnsupportedAdjustment`]; the caller falls back to CPU.
//!
//! # Never panics on a missing GPU
//!
//! Adapter/device creation, buffer sizing beyond the device limits, and the
//! readback all return [`GpuError`] instead of unwinding. The caller keeps the
//! CPU path.
//!
//! ponytail: source samples and mask coverage are assembled on the CPU per
//! layer and uploaded as planar 8-bit bytes; the shader does the unpack, blend
//! and source-over. Move channel sampling into WGSL if upload bandwidth ever
//! shows up in a profile.

use std::fmt;

use pictura_adjust::Adjustment;
use pictura_core::{BlendMode, Document, Layer, PixelBuffer, PsdRect};

use crate::decode_adjustment;

mod backend;
mod shader;

use backend::{devices, Gpu};
pub(crate) use backend::{grid_2d, shared_device};

#[cfg(test)]
use backend::{
    assemble_mask, assemble_mask_fill, assemble_mask_per_pixel, assemble_source,
    assemble_source_per_pixel, assemble_source_rowwise, mask_has_data, mask_influence_rect,
    source_geom, Region,
};

/// Why the GPU compositor could not run. The caller falls back to CPU.
#[derive(Debug)]
pub enum GpuError {
    /// No Vulkan adapter, or the device could not be created.
    Unavailable,
    /// The document needs buffers larger than the device limits.
    TooLarge,
    /// The GPU output could not be mapped back to the host.
    Readback,
    /// A mode with no GPU implementation is present in the stack.
    UnsupportedMode(BlendMode),
    /// An adjustment layer with no GPU implementation is present in the stack.
    UnsupportedAdjustment,
    /// A channel-less smart-object layer is present: it renders from its
    /// embedded source, which the GPU has no path for.
    UnsupportedSmartObject,
    /// A visible layer carries an enabled object-based layer effect.
    UnsupportedLayerEffect,
}

impl fmt::Display for GpuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GpuError::Unavailable => write!(f, "no usable Vulkan adapter/device"),
            GpuError::TooLarge => write!(f, "document exceeds GPU buffer limits"),
            GpuError::Readback => write!(f, "GPU readback failed"),
            GpuError::UnsupportedMode(m) => write!(f, "blend mode {m:?} is CPU-only"),
            GpuError::UnsupportedAdjustment => write!(f, "adjustment kind is CPU-only"),
            GpuError::UnsupportedSmartObject => {
                write!(f, "channel-less smart-object source is CPU-only")
            }
            GpuError::UnsupportedLayerEffect => {
                write!(f, "layer effect is CPU-only")
            }
        }
    }
}

impl std::error::Error for GpuError {}

/// The compositing backend that produced a buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    /// The wgpu/Vulkan compute compositor.
    Gpu,
    /// The CPU oracle [`crate::composite_rgba`].
    Cpu,
}

/// Composite `doc` on the GPU for Normal + every separable mode.
///
/// Returns the same 4-channel planar straight-alpha 8-bit buffer as
/// [`crate::composite_rgba`], within ±1 LSB for the supported modes. Returns
/// [`GpuError`] when no Vulkan device is usable or the stack contains a
/// CPU-only mode; it never panics.
pub fn composite_gpu(doc: &Document) -> Result<PixelBuffer, GpuError> {
    composite_gpu_region(doc, 0, 0, doc.width, doc.height)
}

/// Composite a document-space region `[x0, x0+rw) × [y0, y0+rh)` on the GPU.
///
/// The same kernel as [`composite_gpu`], with every buffer (canvas, per-layer
/// source, mask, group inner canvases) sized to the region and each invocation
/// mapping a region-local index to document coordinates. `x0 + rw <= doc.width`
/// and `y0 + rh <= doc.height` are the caller's responsibility.
fn composite_gpu_region(
    doc: &Document,
    x0: u32,
    y0: u32,
    rw: u32,
    rh: u32,
) -> Result<PixelBuffer, GpuError> {
    check_supported(doc)?;
    if rw == 0 || rh == 0 {
        return Ok(PixelBuffer::new(rw, rh, 4));
    }
    let gpu = Gpu::new(x0, y0, rw, rh)?;
    let canvas = gpu.zero_canvas();
    for layer in &doc.layers {
        gpu.composite_layer(&canvas, layer, doc);
    }
    let pixels = gpu.read_canvas(&canvas)?;
    Ok(pixels)
}

/// Composite only `rect` (clamped to the document) through the active backend.
///
/// Returns a `rect`-sized buffer, byte-identical to the corresponding
/// sub-rectangle of `composite_active(doc, gpu_enabled).0`. An empty clamped
/// intersection returns a zero-dimension buffer and never panics.
pub fn composite_region_active(
    doc: &Document,
    rect: PsdRect,
    gpu_enabled: bool,
) -> (PixelBuffer, Backend) {
    let x0 = rect.left.max(0);
    let y0 = rect.top.max(0);
    let x1 = rect.right.min(doc.width as i32);
    let y1 = rect.bottom.min(doc.height as i32);
    if x1 <= x0 || y1 <= y0 {
        // Nothing ran, so report the fallback backend rather than a device.
        return (PixelBuffer::new(0, 0, 4), Backend::Cpu);
    }
    let w = (x1 - x0) as u32;
    let h = (y1 - y0) as u32;
    let (x0, y0) = (x0 as u32, y0 as u32);
    if gpu_enabled && gpu_available() {
        if let Ok(buf) = composite_gpu_region(doc, x0, y0, w, h) {
            return (buf, Backend::Gpu);
        }
    }
    (composite_cpu_region(doc, x0, y0, w, h), Backend::Cpu)
}

/// The CPU region fallback: composite the whole document, then slice the region.
///
/// ponytail: full CPU composite + slice; the CPU path is the non-default
/// fallback, so a genuinely region-limited accumulator is not worth it until a
/// profile asks for it. The oracle stays `composite_rgba`.
fn composite_cpu_region(doc: &Document, x0: u32, y0: u32, rw: u32, rh: u32) -> PixelBuffer {
    let full = crate::composite_rgba(doc);
    let fw = doc.width as usize;
    let fplane = fw * doc.height as usize;
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

/// Try [`composite_gpu`]; on any [`GpuError`] fall back to the CPU oracle.
pub fn composite_gpu_or_cpu(doc: &Document) -> PixelBuffer {
    composite_gpu(doc).unwrap_or_else(|_| crate::composite_rgba(doc))
}

/// Whether a usable Vulkan adapter/device exists.
///
/// Cached behind `devices()`'s `OnceLock`; cheap to call repeatedly and never
/// panics.
pub fn gpu_available() -> bool {
    devices().is_ok()
}

/// Composite `doc` on the GPU when `gpu_enabled && gpu_available()`, otherwise
/// on the CPU, reporting which backend ran.
///
/// Never panics: any [`GpuError`] (unavailable device, oversize document,
/// unsupported mode/adjustment, readback failure) falls back to the CPU
/// composite for that call.
pub fn composite_active(doc: &Document, gpu_enabled: bool) -> (PixelBuffer, Backend) {
    if gpu_enabled && gpu_available() {
        if let Ok(buf) = composite_gpu(doc) {
            return (buf, Backend::Gpu);
        }
    }
    (crate::composite_rgba(doc), Backend::Cpu)
}

/// No adjustment: the ordinary blend/source-over dispatch.
const NO_ADJ: (u32, i32, i32, i32) = (0, 0, 0, 0);

/// A COMPUTE-visible storage-buffer bind-group layout entry, shared by the
/// compositor and filter pipelines (both alias their storage as `array<u32>`).
pub(crate) fn storage_entry(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

/// Map a decoded adjustment to the shader's kind id plus up to three integer
/// params, or `None` when the GPU has no implementation for it.
fn adjustment_params(adjustment: &Adjustment) -> Option<(u32, i32, i32, i32)> {
    match adjustment {
        Adjustment::Invert => Some((1, 0, 0, 0)),
        Adjustment::Posterize(levels) => Some((2, i32::from(*levels), 0, 0)),
        Adjustment::Threshold(level) => Some((3, i32::from(*level), 0, 0)),
        Adjustment::BrightnessContrast(p) => {
            Some((4, i32::from(p.brightness), i32::from(p.contrast), 0))
        }
        Adjustment::HueSaturation(p) => Some((
            5,
            i32::from(p.hue),
            i32::from(p.saturation),
            i32::from(p.lightness),
        )),
        _ => None,
    }
}

fn check_supported(doc: &Document) -> Result<(), GpuError> {
    fn walk(layer: &Layer) -> Result<(), GpuError> {
        if !layer.visible {
            return Ok(());
        }
        // The effect check wins over the adjustment check: a fill layer with an
        // enabled shadow or glow must report `UnsupportedLayerEffect`, not
        // `UnsupportedAdjustment`.
        if crate::layer_effects::decode_drop_shadow(layer)
            .is_some_and(|shadow| shadow.enabled && shadow.present)
        {
            return Err(GpuError::UnsupportedLayerEffect);
        }
        if crate::layer_effects::decode_outer_glow(layer)
            .is_some_and(|glow| glow.enabled && glow.present)
        {
            return Err(GpuError::UnsupportedLayerEffect);
        }
        if crate::layer_effects::decode_inner_shadow(layer)
            .is_some_and(|shadow| shadow.enabled && shadow.present)
        {
            return Err(GpuError::UnsupportedLayerEffect);
        }
        if crate::layer_effects::decode_inner_glow(layer)
            .is_some_and(|glow| glow.enabled && glow.present)
        {
            return Err(GpuError::UnsupportedLayerEffect);
        }
        if crate::layer_effects::decode_satin(layer)
            .is_some_and(|satin| satin.enabled && satin.present)
        {
            return Err(GpuError::UnsupportedLayerEffect);
        }
        if crate::layer_effects::decode_stroke(layer)
            .is_some_and(|stroke| stroke.enabled && stroke.present)
        {
            return Err(GpuError::UnsupportedLayerEffect);
        }
        if crate::layer_effects::decode_color_overlay(layer)
            .is_some_and(|overlay| overlay.enabled && overlay.present)
        {
            return Err(GpuError::UnsupportedLayerEffect);
        }
        if crate::layer_effects::decode_gradient_overlay(layer)
            .is_some_and(|overlay| overlay.enabled && overlay.present)
        {
            return Err(GpuError::UnsupportedLayerEffect);
        }
        if crate::layer_effects::decode_pattern_overlay(layer)
            .is_some_and(|overlay| overlay.enabled && overlay.present)
        {
            return Err(GpuError::UnsupportedLayerEffect);
        }
        if let Some(data) = &layer.adjustment {
            let supported =
                decode_adjustment(data).is_some_and(|adj| adjustment_params(&adj).is_some());
            if !supported {
                return Err(GpuError::UnsupportedAdjustment);
            }
        }
        if matches!(layer.blend, BlendMode::Dissolve) {
            return Err(GpuError::UnsupportedMode(layer.blend));
        }
        if layer.smart_object.is_some() && !layer.channels.iter().any(|c| c.id == 0) {
            return Err(GpuError::UnsupportedSmartObject);
        }
        for child in &layer.children {
            walk(child)?;
        }
        Ok(())
    }
    for layer in &doc.layers {
        walk(layer)?;
    }
    Ok(())
}

/// Blend mode ids shared with the WGSL `switch`. `PassThrough` is Normal here
/// because the CPU isolated-group path treats it as Normal, and true
/// pass-through groups never reach a dispatch.
fn mode_id(mode: BlendMode) -> u32 {
    match mode {
        BlendMode::Normal | BlendMode::PassThrough => 1,
        BlendMode::Darken => 2,
        BlendMode::Multiply => 3,
        BlendMode::ColorBurn => 4,
        BlendMode::LinearBurn => 5,
        BlendMode::DarkerColor => 6,
        BlendMode::Lighten => 7,
        BlendMode::Screen => 8,
        BlendMode::ColorDodge => 9,
        BlendMode::LinearDodge => 10,
        BlendMode::LighterColor => 11,
        BlendMode::Overlay => 12,
        BlendMode::SoftLight => 13,
        BlendMode::HardLight => 14,
        BlendMode::VividLight => 15,
        BlendMode::LinearLight => 16,
        BlendMode::PinLight => 17,
        BlendMode::HardMix => 18,
        BlendMode::Difference => 19,
        BlendMode::Exclusion => 20,
        BlendMode::Subtract => 21,
        BlendMode::Divide => 22,
        BlendMode::Hue => 23,
        BlendMode::Saturation => 24,
        BlendMode::Color => 25,
        BlendMode::Luminosity => 26,
        // Rejected by `check_supported` before any dispatch.
        BlendMode::Dissolve => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pictura_core::{
        BitDepth, Channel, ColorLabel, ColorMode, Document, Layer, LayerMask, LockFlags, PsdRect,
        SmartObject, SmartObjectKind,
    };
    use std::time::Instant;

    #[test]
    fn grid_2d_tiles_without_gaps_or_overlap() {
        for n in [1u32, 63, 64, 65, 4096, 4_194_241, 8_388_608, 16_000_000] {
            let (gx, gy) = grid_2d(n, 65535).expect("65535 limit accepts every u32 count");
            assert!((1..=65535).contains(&gx), "gx {gx} out of range");
            assert!((1..=65535).contains(&gy), "gy {gy} out of range");
            // Invocation (x, y) maps to x + y * gx * 64, so the rows are
            // contiguous [y*gx*64, (y+1)*gx*64); covering n needs the product.
            assert!(gx as u64 * 64 * gy as u64 >= n as u64);
        }
    }

    #[test]
    fn grid_2d_rejects_product_past_limit_with_synthetic_small_limit() {
        // Synthetic 2-workgroup limit => 2*2*64 = 256 items.
        assert_eq!(grid_2d(256, 2), Some((2, 2)));
        assert_eq!(grid_2d(257, 2), None);
        // A limit of 1 rejects anything past a single 64-wide row.
        assert_eq!(grid_2d(64, 1), Some((1, 1)));
        assert_eq!(grid_2d(65, 1), None);
    }

    fn px_rect(w: u32, h: u32) -> PsdRect {
        PsdRect {
            top: 0,
            left: 0,
            bottom: h as i32,
            right: w as i32,
        }
    }

    fn pix_layer(w: u32, h: u32, channels: Vec<Channel>) -> Layer {
        Layer {
            name: "t".into(),
            rect: px_rect(w, h),
            blend: BlendMode::Normal,
            opacity: 255,
            fill: 255,
            lock: LockFlags::default(),
            color: ColorLabel::None,
            clipping: false,
            visible: true,
            mask: None,
            adjustment: None,
            channels,
            children: Vec::new(),
            is_group: false,
            background: false,
            ..Default::default()
        }
    }

    fn ramp(n: usize, seed: u8) -> Vec<u8> {
        (0..n)
            .map(|i| (i as u8).wrapping_mul(31).wrapping_add(seed))
            .collect()
    }

    #[test]
    fn rowwise_source_matches_per_pixel() {
        let (w, h) = (5u32, 3u32);
        let n = (w * h) as usize;
        let region = Region { x0: 0, y0: 0, w, h };
        let doc = Document::new(w, h, ColorMode::Rgb, BitDepth::Eight);
        let chans = |ids: &[i16]| -> Vec<Channel> {
            ids.iter()
                .map(|&id| Channel {
                    id,
                    data: ramp(n, id as u8),
                })
                .collect()
        };

        // RGB with all four channels: row-wise and per-pixel agree.
        let all = pix_layer(w, h, chans(&[0, 1, 2, -1]));
        let g = source_geom(region, &all, &doc).unwrap();
        let fast = assemble_source_rowwise(&all, &g).expect("covering RGB layer is row-wise");
        assert_eq!(fast, assemble_source_per_pixel(&all, &g));
        assert_eq!(assemble_source(region, &all, &doc).unwrap().0, fast);

        // Missing alpha: the alpha plane fills 255.
        let no_alpha = pix_layer(w, h, chans(&[0, 1, 2]));
        let g = source_geom(region, &no_alpha, &doc).unwrap();
        let fast = assemble_source_rowwise(&no_alpha, &g).expect("missing alpha is row-wise");
        assert_eq!(fast, assemble_source_per_pixel(&no_alpha, &g));
        assert!(
            fast[3 * n..4 * n].iter().all(|&b| b == 255),
            "absent alpha must read 255"
        );

        // Missing green/blue alias channel 0.
        let mono = pix_layer(w, h, chans(&[0, -1]));
        let g = source_geom(region, &mono, &doc).unwrap();
        let fast = assemble_source_rowwise(&mono, &g).expect("absent G/B alias channel 0");
        assert_eq!(fast, assemble_source_per_pixel(&mono, &g));
        assert_eq!(&fast[n..2 * n], &fast[..n]);
        assert_eq!(&fast[2 * n..3 * n], &fast[..n]);

        // Grayscale document: exactly two planes, colour from channel 0.
        let gdoc = Document::new(w, h, ColorMode::Grayscale, BitDepth::Eight);
        let gray = pix_layer(w, h, chans(&[0, -1]));
        let g = source_geom(region, &gray, &gdoc).unwrap();
        assert_eq!(g.planes, 2, "grayscale is two planes");
        let fast = assemble_source_rowwise(&gray, &g).expect("gray layer is row-wise");
        assert_eq!(fast, assemble_source_per_pixel(&gray, &g));

        // A short colour plane cannot cover the intersection -> fallback.
        let mut short = pix_layer(w, h, chans(&[0, 1, 2, -1]));
        short.channels[0].data.truncate(n - 1);
        let g = source_geom(region, &short, &doc).unwrap();
        assert!(
            assemble_source_rowwise(&short, &g).is_none(),
            "a short plane must take the per-pixel fallback"
        );
        assert_eq!(
            assemble_source(region, &short, &doc).unwrap().0,
            assemble_source_per_pixel(&short, &g)
        );
    }

    #[test]
    fn rowwise_mask_matches_per_pixel() {
        let (w, h) = (6u32, 4u32);
        let n = (w * h) as usize;
        let region = Region { x0: 0, y0: 0, w, h };

        // Maskless pixel layer -> constant-255 row fill.
        let mut l = pix_layer(w, h, Vec::new());
        assert!(!mask_has_data(&l));
        let r = mask_influence_rect(region, &l);
        assert_eq!(
            assemble_mask(region, &l),
            assemble_mask_per_pixel(region, &l, r)
        );
        assert_eq!(assemble_mask(region, &l), assemble_mask_fill(region, r));

        // A disabled mask still reads 255 everywhere.
        l.mask = Some(LayerMask {
            rect: px_rect(w, h),
            default_color: 9,
            disabled: true,
            flags: 0,
            data: Some(ramp(n, 5)),
            ..Default::default()
        });
        assert!(!mask_has_data(&l));
        let r = mask_influence_rect(region, &l);
        assert_eq!(
            assemble_mask(region, &l),
            assemble_mask_per_pixel(region, &l, r)
        );
        assert_eq!(assemble_mask(region, &l), assemble_mask_fill(region, r));

        // An enabled data-carrying mask keeps the per-pixel path.
        l.mask = Some(LayerMask {
            rect: px_rect(w, h),
            default_color: 9,
            disabled: false,
            flags: 0,
            data: Some(ramp(n, 5)),
            ..Default::default()
        });
        assert!(mask_has_data(&l));
        let r = mask_influence_rect(region, &l);
        assert_eq!(
            assemble_mask(region, &l),
            assemble_mask_per_pixel(region, &l, r)
        );
    }

    #[test]
    fn channel_less_smart_object_falls_back_to_cpu() {
        let (w, h) = (2u32, 2u32);
        let n = (w * h) as usize;
        let mut source = Document::new(w, h, ColorMode::Rgb, BitDepth::Eight);
        for i in 0..n {
            source.composite.data[i] = 10;
            source.composite.data[n + i] = 20;
            source.composite.data[2 * n + i] = 30;
        }
        let payload = pictura_codec::write_psd(&source).expect("payload writes");

        let mut doc = Document::new(w, h, ColorMode::Rgb, BitDepth::Eight);
        doc.layers = vec![Layer {
            name: "placed".into(),
            rect: px_rect(w, h),
            channels: Vec::new(),
            smart_object: Some(SmartObject {
                filename: "source.psd".into(),
                kind: SmartObjectKind::Embedded,
                payload: Some(payload),
                ..Default::default()
            }),
            ..Default::default()
        }];

        // `check_supported` rejects the stack before any device is touched, so
        // this holds on a device-less host too.
        assert!(matches!(
            composite_gpu(&doc),
            Err(GpuError::UnsupportedSmartObject)
        ));

        // The caller falls back to the CPU oracle, which renders the embedded
        // source rather than the channel-less layer's opaque black.
        let (out, backend) = composite_active(&doc, true);
        assert_eq!(backend, Backend::Cpu);
        for i in 0..n {
            assert_eq!(
                [
                    out.data[i],
                    out.data[n + i],
                    out.data[2 * n + i],
                    out.data[3 * n + i]
                ],
                [10, 20, 30, 255],
                "pixel {i}"
            );
        }
    }

    fn profile_layer(n: u32, seed: u32, blend: BlendMode) -> Layer {
        let px = (n as usize) * (n as usize);
        let (mut r, mut g, mut b, mut a) =
            (vec![0u8; px], vec![0u8; px], vec![0u8; px], vec![0u8; px]);
        for y in 0..n {
            for x in 0..n {
                let i = (y * n + x) as usize;
                r[i] = (x * 7 + seed * 31) as u8;
                g[i] = (y * 5 + seed * 17) as u8;
                b[i] = (x ^ y) as u8;
                a[i] = if seed == 0 { 255 } else { 160 };
            }
        }
        Layer {
            name: format!("layer{seed}"),
            rect: px_rect(n, n),
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
            background: false,
            ..Default::default()
        }
    }

    /// The transfer half of the readback only (staging + copy + poll + map), so
    /// the profile can separate it from the de-interleave.
    fn read_transfer(gpu: &Gpu, canvas: &wgpu::Buffer) -> Result<(), GpuError> {
        let size = u64::from(gpu.n) * 4;
        let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("pictura-profile-transfer"),
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("pictura-profile-transfer"),
            });
        encoder.copy_buffer_to_buffer(canvas, 0, &staging, 0, size);
        gpu.queue.submit(Some(encoder.finish()));
        let slice = staging.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        rx.recv()
            .map_err(|_| GpuError::Readback)?
            .map_err(|_| GpuError::Readback)?;
        let _mapped = slice.get_mapped_range().map_err(|_| GpuError::Readback)?;
        Ok(())
    }

    fn phase_ms(t: Instant) -> f64 {
        t.elapsed().as_secs_f64() * 1000.0
    }

    fn run_profile(n: u32) {
        if !gpu_available() {
            println!("gpu_phase {n}x{n} n/a: no usable Vulkan adapter");
            return;
        }
        let mut doc = Document::new(n, n, ColorMode::Rgb, BitDepth::Eight);
        doc.layers = vec![
            profile_layer(n, 0, BlendMode::Normal),
            profile_layer(n, 1, BlendMode::Multiply),
        ];
        // Warm-up keeps device/pipeline setup out of the phase numbers.
        let _ = composite_gpu_region(&doc, 0, 0, n, n);
        let gpu = match Gpu::new(0, 0, n, n) {
            Ok(g) => g,
            Err(e) => {
                println!("gpu_phase {n}x{n} n/a: {e}");
                return;
            }
        };

        let t = Instant::now();
        let canvas = gpu.zero_canvas();
        println!("gpu_phase zero_canvas {:.3} ms", phase_ms(t));

        let t = Instant::now();
        let srcs: Vec<_> = doc
            .layers
            .iter()
            .map(|l| gpu.build_source(l, &doc))
            .collect();
        println!("gpu_phase build_source {:.3} ms", phase_ms(t));

        let t = Instant::now();
        let masks: Vec<_> = doc.layers.iter().map(|l| gpu.build_mask(l)).collect();
        println!("gpu_phase build_mask {:.3} ms", phase_ms(t));

        let t = Instant::now();
        for (i, layer) in doc.layers.iter().enumerate() {
            if let Some((src, layout)) = &srcs[i] {
                gpu.dispatch(
                    &canvas,
                    src,
                    &masks[i],
                    *layout,
                    layer.blend,
                    layer.opacity,
                    layer.fill,
                    NO_ADJ,
                );
            }
        }
        println!("gpu_phase dispatch {:.3} ms", phase_ms(t));

        let t = Instant::now();
        let _ = read_transfer(&gpu, &canvas);
        println!("gpu_phase read_submit_poll_map {:.3} ms", phase_ms(t));

        // Standalone de-interleave micro-benchmark on a synthetic packed buffer;
        // it is not part of the composite total (whose de-interleave is now the
        // GPU planar pass plus plane copies in `read_canvas`, printed as
        // `readback` below). Kept to measure the retained host gather.
        let packed = vec![0u8; gpu.n as usize * 4];
        let t = Instant::now();
        let _ = gpu.to_pixel_buffer(&packed);
        println!(
            "gpu_phase to_pixel_buffer_micro (not in total) {:.3} ms",
            phase_ms(t)
        );

        let t = Instant::now();
        let _ = gpu.read_canvas(&canvas);
        println!("gpu_phase readback {:.3} ms", phase_ms(t));

        let t = Instant::now();
        let _ = composite_gpu_region(&doc, 0, 0, n, n);
        println!("gpu_phase total {:.3} ms", phase_ms(t));
    }

    #[test]
    #[ignore = "requires a Vulkan GPU; run with --ignored --nocapture"]
    fn composite_profile_4000() {
        run_profile(4000);
    }

    #[test]
    #[ignore = "requires a Vulkan GPU; run with --ignored --nocapture"]
    fn composite_profile_1024() {
        run_profile(1024);
    }
}
