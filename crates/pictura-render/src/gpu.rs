//! GPU (wgpu/Vulkan) compositor for the separable and non-separable blend
//! modes and the supported adjustment layers.
//!
//! `composite_gpu` mirrors the CPU oracle [`crate::composite_rgba`] pass for
//! pass: the same per-layer loop, mask/opacity handling, pass-through/isolated
//! group semantics, and the W3C + Photoshop blend formulas, executed by a
//! wgpu compute shader. It is proven, not assumed — the parity test in
//! `tests/gpu_parity.rs` diffs it against the CPU compositor within ±1 LSB.
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
//! layer and uploaded; the shader only does the blend + source-over. Move
//! channel sampling into WGSL if upload bandwidth ever shows up in a profile.

use std::borrow::Cow;
use std::fmt;
use std::sync::OnceLock;

use pictura_adjust::Adjustment;
use pictura_core::{BlendMode, ColorMode, Document, Layer, PixelBuffer};

use crate::{channel, decode_adjustment, mask_alpha, sample, to_u8};

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
}

impl fmt::Display for GpuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GpuError::Unavailable => write!(f, "no usable Vulkan adapter/device"),
            GpuError::TooLarge => write!(f, "document exceeds GPU buffer limits"),
            GpuError::Readback => write!(f, "GPU readback failed"),
            GpuError::UnsupportedMode(m) => write!(f, "blend mode {m:?} is CPU-only"),
            GpuError::UnsupportedAdjustment => write!(f, "adjustment kind is CPU-only"),
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
    check_supported(doc)?;
    if doc.width == 0 || doc.height == 0 {
        return Ok(PixelBuffer::new(doc.width, doc.height, 4));
    }
    let gpu = Gpu::new(doc.width, doc.height)?;
    let canvas = gpu.zero_canvas();
    for layer in &doc.layers {
        gpu.composite_layer(&canvas, layer, doc);
    }
    let pixels = gpu.read_canvas(&canvas)?;
    Ok(gpu.to_pixel_buffer(&pixels))
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

fn is_cpu_only(mode: BlendMode) -> bool {
    matches!(mode, BlendMode::Dissolve)
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
        if let Some(data) = &layer.adjustment {
            let supported =
                decode_adjustment(data).is_some_and(|adj| adjustment_params(&adj).is_some());
            if !supported {
                return Err(GpuError::UnsupportedAdjustment);
            }
        }
        if is_cpu_only(layer.blend) {
            return Err(GpuError::UnsupportedMode(layer.blend));
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

const SHADER: &str = r#"
struct Params {
    mode: u32,
    opacity: f32,
    count: u32,
    adj_kind: u32,
    p0: i32,
    p1: i32,
    p2: i32,
    pad: u32,
};

@group(0) @binding(0) var<storage, read_write> canvas: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> src: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> mask: array<f32>;
@group(0) @binding(3) var<uniform> params: Params;

fn color_burn(cb: f32, cs: f32) -> f32 {
    if (cb >= 1.0) { return 1.0; }
    if (cs <= 0.0) { return 0.0; }
    return 1.0 - min((1.0 - cb) / cs, 1.0);
}

fn color_dodge(cb: f32, cs: f32) -> f32 {
    if (cb <= 0.0) { return 0.0; }
    if (cs >= 1.0) { return 1.0; }
    return min(cb / (1.0 - cs), 1.0);
}

fn hard_light(cb: f32, cs: f32) -> f32 {
    if (cs <= 0.5) { return 2.0 * cb * cs; }
    return 1.0 - 2.0 * (1.0 - cb) * (1.0 - cs);
}

fn soft_light_d(x: f32) -> f32 {
    if (x <= 0.25) { return ((16.0 * x - 12.0) * x + 4.0) * x; }
    return sqrt(x);
}

fn soft_light(cb: f32, cs: f32) -> f32 {
    if (cs <= 0.5) { return cb - (1.0 - 2.0 * cs) * cb * (1.0 - cb); }
    return cb + (2.0 * cs - 1.0) * (soft_light_d(cb) - cb);
}

fn vivid_light(cb: f32, cs: f32) -> f32 {
    if (cs <= 0.5) { return color_burn(cb, 2.0 * cs); }
    return color_dodge(cb, 2.0 * cs - 1.0);
}

fn pin_light(cb: f32, cs: f32) -> f32 {
    if (cs <= 0.5) { return min(cb, 2.0 * cs); }
    return max(cb, 2.0 * cs - 1.0);
}

fn sep(mode: u32, cb: f32, cs: f32) -> f32 {
    switch mode {
        case 2u: { return min(cb, cs); }
        case 3u: { return cb * cs; }
        case 4u: { return color_burn(cb, cs); }
        case 5u: { return max(cb + cs - 1.0, 0.0); }
        case 7u: { return max(cb, cs); }
        case 8u: { return 1.0 - (1.0 - cb) * (1.0 - cs); }
        case 9u: { return color_dodge(cb, cs); }
        case 10u: { return min(cb + cs, 1.0); }
        case 12u: { return hard_light(cs, cb); }
        case 13u: { return soft_light(cb, cs); }
        case 14u: { return hard_light(cb, cs); }
        case 15u: { return vivid_light(cb, cs); }
        case 16u: { return clamp(cb + 2.0 * cs - 1.0, 0.0, 1.0); }
        case 17u: { return pin_light(cb, cs); }
        case 18u: {
            if (cb + cs >= 1.0) { return 1.0; }
            return 0.0;
        }
        case 19u: { return abs(cb - cs); }
        case 20u: { return cb + cs - 2.0 * cb * cs; }
        case 21u: { return max(cb - cs, 0.0); }
        case 22u: {
            if (cs == 0.0) { return 1.0; }
            return min(cb / cs, 1.0);
        }
        default: { return cs; }
    }
}

fn lum(c: vec3<f32>) -> f32 {
    return 0.3 * c.x + 0.59 * c.y + 0.11 * c.z;
}

fn sat(c: vec3<f32>) -> f32 {
    return max(c.x, max(c.y, c.z)) - min(c.x, min(c.y, c.z));
}

fn clip_color(c: vec3<f32>) -> vec3<f32> {
    let l = lum(c);
    let n = min(c.x, min(c.y, c.z));
    let x = max(c.x, max(c.y, c.z));
    var out = c;
    if (n < 0.0) {
        let d = l - n;
        if (d != 0.0) { out = vec3<f32>(l) + (out - vec3<f32>(l)) * (l / d); }
    }
    if (x > 1.0) {
        let d = x - l;
        if (d != 0.0) { out = vec3<f32>(l) + (out - vec3<f32>(l)) * ((1.0 - l) / d); }
    }
    return out;
}

fn set_lum(c: vec3<f32>, l: f32) -> vec3<f32> {
    let d = l - lum(c);
    return clip_color(c + vec3<f32>(d));
}

fn set_sat(c: vec3<f32>, s: f32) -> vec3<f32> {
    let mx = max(c.x, max(c.y, c.z));
    let mn = min(c.x, min(c.y, c.z));
    if (mx <= mn) { return vec3<f32>(0.0); }
    return (c - vec3<f32>(mn)) * s / (mx - mn);
}

fn blend(mode: u32, cb: vec3<f32>, cs: vec3<f32>) -> vec3<f32> {
    if (mode == 6u) {
        if (cb.x + cb.y + cb.z <= cs.x + cs.y + cs.z) { return cb; }
        return cs;
    }
    if (mode == 11u) {
        if (cb.x + cb.y + cb.z >= cs.x + cs.y + cs.z) { return cb; }
        return cs;
    }
    if (mode == 23u) { return set_lum(set_sat(cs, sat(cb)), lum(cb)); }
    if (mode == 24u) { return set_lum(set_sat(cb, sat(cs)), lum(cb)); }
    if (mode == 25u) { return set_lum(cs, lum(cb)); }
    if (mode == 26u) { return set_lum(cb, lum(cs)); }
    return vec3<f32>(sep(mode, cb.x, cs.x), sep(mode, cb.y, cs.y), sep(mode, cb.z, cs.z));
}

// --- adjustment helpers -----------------------------------------------------

fn rem_euclid(x: f32, m: f32) -> f32 {
    return x - m * floor(x / m);
}

fn quant(v: f32) -> u32 {
    return u32(round(clamp(v, 0.0, 1.0) * 255.0));
}

fn to_f(v: u32) -> f32 {
    return f32(v) / 255.0;
}

fn adj_posterize(v: u32, levels: i32) -> u32 {
    if (levels >= 255) { return v; }
    let d = f32(levels - 1);
    let q = round(f32(v) * d / 255.0);
    return u32(round(q * 255.0 / d));
}

fn adj_threshold(v: vec3<u32>, level: i32) -> u32 {
    let y = 0.299 * f32(v.x) + 0.587 * f32(v.y) + 0.114 * f32(v.z);
    if (y > f32(level)) { return 255u; }
    return 0u;
}

fn adj_bc(v: u32, brightness: i32, contrast: i32) -> u32 {
    let c = f32(contrast) / 100.0;
    let b = f32(brightness) / 150.0;
    let gamma = pow(2.0, b);
    let y = pow(f32(v) / 255.0, 1.0 / gamma);
    var res = 0.0;
    if (y <= 0.5) {
        let t = 2.0 * y;
        res = 0.5 * t + 0.5 * c * (t * t * t - t * t);
    } else {
        let t = 2.0 * y - 1.0;
        res = 0.5 * (1.0 + t) + 0.5 * c * (t * t * t - 2.0 * t * t + t);
    }
    return u32(round(clamp(res, 0.0, 1.0) * 255.0));
}

fn hue2rgb(p: f32, q: f32, t_in: f32) -> f32 {
    let t = rem_euclid(t_in, 1.0);
    if (t < 1.0 / 6.0) { return p + (q - p) * 6.0 * t; }
    if (t < 0.5) { return q; }
    if (t < 2.0 / 3.0) { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
    return p;
}

fn adj_hs(rgb: vec3<f32>, hue: i32, saturation: i32, lightness: i32) -> vec3<f32> {
    let ds = f32(saturation) / 100.0;
    let dl = f32(lightness) / 100.0;
    let mx = max(rgb.x, max(rgb.y, rgb.z));
    let mn = min(rgb.x, min(rgb.y, rgb.z));
    let l0 = (mx + mn) / 2.0;
    var h = 0.0;
    var s = 0.0;
    if (abs(mx - mn) >= 1e-12) {
        let d = mx - mn;
        if (l0 > 0.5) { s = d / (2.0 - mx - mn); } else { s = d / (mx + mn); }
        if (mx == rgb.x) { h = (rgb.y - rgb.z) / d; }
        else if (mx == rgb.y) { h = (rgb.z - rgb.x) / d + 2.0; }
        else { h = (rgb.x - rgb.y) / d + 4.0; }
        h = h * 60.0;
    }
    h = rem_euclid(h + f32(hue), 360.0);
    s = clamp(s * (1.0 + ds), 0.0, 1.0);
    var l = l0;
    if (dl >= 0.0) { l = l + dl * (1.0 - l); } else { l = l + dl * l; }
    l = clamp(l, 0.0, 1.0);
    if (s <= 0.0) { return vec3<f32>(l); }
    var q = l * (1.0 + s);
    if (l >= 0.5) { q = l + s - l * s; }
    let p = 2.0 * l - q;
    let hk = h / 360.0;
    return vec3<f32>(
        hue2rgb(p, q, hk + 1.0 / 3.0),
        hue2rgb(p, q, hk),
        hue2rgb(p, q, hk - 1.0 / 3.0),
    );
}

fn adjust(kind: u32, rgb: vec3<f32>, p0: i32, p1: i32, p2: i32) -> vec3<f32> {
    let q = vec3<u32>(quant(rgb.x), quant(rgb.y), quant(rgb.z));
    if (kind == 1u) {
        return vec3<f32>(to_f(255u - q.x), to_f(255u - q.y), to_f(255u - q.z));
    }
    if (kind == 2u) {
        return vec3<f32>(
            to_f(adj_posterize(q.x, p0)),
            to_f(adj_posterize(q.y, p0)),
            to_f(adj_posterize(q.z, p0)),
        );
    }
    if (kind == 3u) {
        let v = adj_threshold(q, p0);
        return vec3<f32>(to_f(v));
    }
    if (kind == 4u) {
        return vec3<f32>(
            to_f(adj_bc(q.x, p0, p1)),
            to_f(adj_bc(q.y, p0, p1)),
            to_f(adj_bc(q.z, p0, p1)),
        );
    }
    if (kind == 5u) {
        return adj_hs(vec3<f32>(to_f(q.x), to_f(q.y), to_f(q.z)), p0, p1, p2);
    }
    return rgb;
}

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.count) { return; }

    if (params.adj_kind != 0u) {
        let cb = canvas[i];
        let ab = cb.a;
        if (ab <= 0.0) { return; }
        let as_ = ab * params.opacity * mask[i];
        if (as_ <= 0.0) { return; }
        let cs = adjust(params.adj_kind, cb.rgb, params.p0, params.p1, params.p2);
        let b = blend(params.mode, cb.rgb, cs);
        let ao = as_ + ab * (1.0 - as_);
        if (ao <= 0.0) {
            canvas[i] = vec4<f32>(0.0);
            return;
        }
        let co = ((1.0 - ab) * as_ * cs + as_ * ab * b + (1.0 - as_) * ab * cb.rgb) / ao;
        canvas[i] = vec4<f32>(co, ao);
        return;
    }

    let s = src[i];
    if (s.a <= 0.0) { return; }
    let as_ = s.a * params.opacity * mask[i];
    if (as_ <= 0.0) { return; }

    let cb = canvas[i];
    let ab = cb.a;
    let b = blend(params.mode, cb.rgb, s.rgb);
    let ao = as_ + ab * (1.0 - as_);
    if (ao <= 0.0) {
        canvas[i] = vec4<f32>(0.0);
        return;
    }
    let co = ((1.0 - ab) * as_ * s.rgb + as_ * ab * b + (1.0 - as_) * ab * cb.rgb) / ao;
    canvas[i] = vec4<f32>(co, ao);
}
"#;

/// Bind-group layout and compute pipeline, both size-independent (buffer sizes
/// travel as bindings), so created once per shared device instead of rebuilt on
/// every composite. The params uniform lives on [`Gpu`] instead: it carries
/// per-dispatch state, so sharing one across concurrent composites raced.
struct ComputeResources {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
}

impl ComputeResources {
    fn new(device: &wgpu::Device) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("pictura-blend-layout"),
            entries: &[
                storage_entry(0, false),
                storage_entry(1, true),
                storage_entry(2, true),
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("pictura-blend"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(SHADER)),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pictura-blend"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("pictura-blend"),
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point: Some("cs_main"),
            compilation_options: Default::default(),
            cache: None,
        });
        Self { pipeline, layout }
    }
}

/// Keeps the wgpu objects that own the raw device alive for the process.
struct Devices {
    _instance: wgpu::Instance,
    _adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    resources: OnceLock<ComputeResources>,
}

impl Devices {
    fn resources(&'static self) -> &'static ComputeResources {
        self.resources
            .get_or_init(|| ComputeResources::new(&self.device))
    }
}

async fn request_device() -> Option<Devices> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
            apply_limit_buckets: false,
        })
        .await
        .ok()?;
    if adapter.limits().max_storage_buffers_per_shader_stage < 3 {
        return None;
    }
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("pictura-render-gpu"),
            required_features: wgpu::Features::empty(),
            required_limits: adapter.limits(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::default(),
            trace: wgpu::Trace::Off,
        })
        .await
        .ok()?;
    Some(Devices {
        _instance: instance,
        _adapter: adapter,
        device,
        queue,
        resources: OnceLock::new(),
    })
}

fn devices() -> Result<&'static Devices, GpuError> {
    static DEVICES: OnceLock<Option<Devices>> = OnceLock::new();
    DEVICES
        .get_or_init(|| pollster::block_on(request_device()))
        .as_ref()
        .ok_or(GpuError::Unavailable)
}

struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    res: &'static ComputeResources,
    params: wgpu::Buffer,
    w: u32,
    h: u32,
    n: u32,
}

impl Gpu {
    fn new(w: u32, h: u32) -> Result<Self, GpuError> {
        let shared = devices()?;
        let device = shared.device.clone();
        let queue = shared.queue.clone();
        let n = w * h;
        let bytes = u64::from(n) * 16;
        let limits = device.limits();
        if bytes > limits.max_storage_buffer_binding_size || bytes > limits.max_buffer_size {
            return Err(GpuError::TooLarge);
        }
        if n.div_ceil(64) > limits.max_compute_workgroups_per_dimension {
            return Err(GpuError::TooLarge);
        }
        let params = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("pictura-blend-params"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Ok(Self {
            device,
            queue,
            res: shared.resources(),
            params,
            w,
            h,
            n,
        })
    }

    fn zero_canvas(&self) -> wgpu::Buffer {
        self.make_buffer(
            "pictura-canvas",
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            &vec![0u8; self.n as usize * 16],
        )
    }

    fn make_buffer(&self, label: &str, usage: wgpu::BufferUsages, bytes: &[u8]) -> wgpu::Buffer {
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: bytes.len() as u64,
            usage: usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(&buffer, 0, bytes);
        buffer
    }

    /// Full-size storage buffer with no upload: wgpu zero-initializes it and the
    /// caller writes only the region it needs.
    fn make_zeroed(&self, label: &str, usage: wgpu::BufferUsages, size: u64) -> wgpu::Buffer {
        self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage: usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    /// Assemble the full-canvas source for one pixel layer, matching
    /// `composite_pixels`: zero outside the layer rect, otherwise the channel
    /// samples (grayscale replicates channel 0) and straight alpha. Only the
    /// clamped rect rows are uploaded; the shader indexes by canvas pixel.
    fn build_source(&self, layer: &Layer, doc: &Document) -> Option<wgpu::Buffer> {
        let lw = layer.rect.width();
        let lh = layer.rect.height();
        if lw <= 0 || lh <= 0 {
            return None;
        }
        let x0 = layer.rect.left.max(0);
        let y0 = layer.rect.top.max(0);
        let x1 = layer.rect.right.min(self.w as i32);
        let y1 = layer.rect.bottom.min(self.h as i32);
        if x1 <= x0 || y1 <= y0 {
            return None;
        }

        let gray = matches!(
            doc.mode,
            ColorMode::Grayscale | ColorMode::Bitmap | ColorMode::Duotone
        );
        let lw = lw as usize;
        let ch0 = channel(layer, 0);
        let ch1 = channel(layer, 1).or(ch0);
        let ch2 = channel(layer, 2).or(ch0);
        let alpha = channel(layer, -1);

        let buffer = self.make_zeroed(
            "pictura-src",
            wgpu::BufferUsages::STORAGE,
            u64::from(self.n) * 16,
        );
        let width = (x1 - x0) as usize;
        let stride = self.w as usize;
        let mut row = vec![0u8; width * 16];
        for y in y0..y1 {
            for (col, x) in (x0..x1).enumerate() {
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
                let i = col * 16;
                row[i..i + 4].copy_from_slice(&(r as f32 / 255.0).to_le_bytes());
                row[i + 4..i + 8].copy_from_slice(&(g as f32 / 255.0).to_le_bytes());
                row[i + 8..i + 12].copy_from_slice(&(b as f32 / 255.0).to_le_bytes());
                row[i + 12..i + 16].copy_from_slice(&(a as f32 / 255.0).to_le_bytes());
            }
            let offset = ((y as usize * stride + x0 as usize) * 16) as u64;
            self.queue.write_buffer(&buffer, offset, &row);
        }
        Some(buffer)
    }

    /// Per-canvas-pixel mask coverage, reusing the CPU `mask_alpha`.
    ///
    /// A pixel layer's source is zero outside its clamped rect, so the shader
    /// only reads the mask there and only that rect is uploaded. A group's inner
    /// buffer and an adjustment layer both act across the whole canvas (their own
    /// rect carries no content), so they keep the full-canvas write.
    /// ponytail: group/adjustment masks stay full-canvas; bound them by the inner
    /// buffer's content rect if a group-mask profile ever shows up.
    fn build_mask(&self, layer: &Layer) -> wgpu::Buffer {
        let buffer = self.make_zeroed(
            "pictura-mask",
            wgpu::BufferUsages::STORAGE,
            u64::from(self.n) * 4,
        );
        let (x0, y0, x1, y1) = if layer.is_group || layer.adjustment.is_some() {
            (0, 0, self.w as i32, self.h as i32)
        } else {
            (
                layer.rect.left.max(0),
                layer.rect.top.max(0),
                layer.rect.right.min(self.w as i32),
                layer.rect.bottom.min(self.h as i32),
            )
        };
        if x1 <= x0 || y1 <= y0 {
            return buffer;
        }
        let width = (x1 - x0) as usize;
        let stride = self.w as usize;
        let mut row = vec![0u8; width * 4];
        for y in y0..y1 {
            for (col, x) in (x0..x1).enumerate() {
                row[col * 4..col * 4 + 4]
                    .copy_from_slice(&(mask_alpha(layer, x, y) as f32 / 255.0).to_le_bytes());
            }
            let offset = ((y as usize * stride + x0 as usize) * 4) as u64;
            self.queue.write_buffer(&buffer, offset, &row);
        }
        buffer
    }

    fn composite_layer(&self, canvas: &wgpu::Buffer, layer: &Layer, doc: &Document) {
        if !layer.visible {
            return;
        }
        if layer.is_group {
            // True pass-through: recurse onto the running canvas (matches the
            // CPU exactness condition).
            if matches!(layer.blend, BlendMode::PassThrough)
                && layer.opacity == 255
                && layer.mask.is_none()
            {
                for child in &layer.children {
                    self.composite_layer(canvas, child, doc);
                }
                return;
            }
            let inner = self.zero_canvas();
            for child in &layer.children {
                self.composite_layer(&inner, child, doc);
            }
            let mask = self.build_mask(layer);
            self.dispatch(canvas, &inner, &mask, layer.blend, layer.opacity, NO_ADJ);
            return;
        }
        if let Some(data) = &layer.adjustment {
            // `check_supported` already rejected every adjustment that has no GPU
            // kind, so this only runs for the five supported kinds.
            if let Some(adj) = decode_adjustment(data).and_then(|a| adjustment_params(&a)) {
                let mask = self.build_mask(layer);
                // The shader ignores `src` in the adjustment branch; the mask
                // buffer stands in for it rather than aliasing the read-write
                // canvas binding.
                self.dispatch(canvas, &mask, &mask, layer.blend, layer.opacity, adj);
            }
            return;
        }
        if let Some(src) = self.build_source(layer, doc) {
            let mask = self.build_mask(layer);
            self.dispatch(canvas, &src, &mask, layer.blend, layer.opacity, NO_ADJ);
        }
    }

    fn dispatch(
        &self,
        canvas: &wgpu::Buffer,
        src: &wgpu::Buffer,
        mask: &wgpu::Buffer,
        mode: BlendMode,
        opacity: u8,
        adj: (u32, i32, i32, i32),
    ) {
        let params = [
            mode_id(mode).to_le_bytes(),
            (opacity as f32 / 255.0).to_le_bytes(),
            self.n.to_le_bytes(),
            adj.0.to_le_bytes(),
            adj.1.to_le_bytes(),
            adj.2.to_le_bytes(),
            adj.3.to_le_bytes(),
            0u32.to_le_bytes(),
        ]
        .concat();
        self.queue.write_buffer(&self.params, 0, &params);

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("pictura-blend"),
            layout: &self.res.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: canvas.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: src.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: mask.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.params.as_entire_binding(),
                },
            ],
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("pictura-blend"),
            });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("pictura-blend"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.res.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(self.n.div_ceil(64), 1, 1);
        }
        self.queue.submit(Some(encoder.finish()));
    }

    fn read_canvas(&self, canvas: &wgpu::Buffer) -> Result<Vec<f32>, GpuError> {
        let size = u64::from(self.n) * 16;
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("pictura-readback"),
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("pictura-readback"),
            });
        encoder.copy_buffer_to_buffer(canvas, 0, &staging, 0, size);
        self.queue.submit(Some(encoder.finish()));

        let slice = staging.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        let _ = self.device.poll(wgpu::PollType::wait_indefinitely());
        rx.recv()
            .map_err(|_| GpuError::Readback)?
            .map_err(|_| GpuError::Readback)?;

        let mapped = slice.get_mapped_range().map_err(|_| GpuError::Readback)?;
        let mut out = vec![0f32; self.n as usize * 4];
        for (i, chunk) in mapped.as_chunks::<4>().0.iter().enumerate() {
            out[i] = f32::from_le_bytes(*chunk);
        }
        drop(mapped);
        staging.unmap();
        Ok(out)
    }

    /// Planar RGBA8 output, identical in shape to `Canvas::into_pixel_buffer`.
    fn to_pixel_buffer(&self, pixels: &[f32]) -> PixelBuffer {
        let plane = self.n as usize;
        let mut out = PixelBuffer::new(self.w, self.h, 4);
        for i in 0..plane {
            out.data[i] = to_u8(pixels[i * 4]);
            out.data[plane + i] = to_u8(pixels[i * 4 + 1]);
            out.data[2 * plane + i] = to_u8(pixels[i * 4 + 2]);
            out.data[3 * plane + i] = to_u8(pixels[i * 4 + 3]);
        }
        out
    }
}

fn storage_entry(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
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
