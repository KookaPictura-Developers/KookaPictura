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
//! layer and uploaded as planar 8-bit bytes; the shader does the unpack, blend
//! and source-over. Move channel sampling into WGSL if upload bandwidth ever
//! shows up in a profile.

use std::borrow::Cow;
use std::fmt;
use std::sync::OnceLock;

use pictura_adjust::Adjustment;
use pictura_core::{BlendMode, ColorMode, Document, Layer, PixelBuffer};

use crate::{channel, decode_adjustment, mask_alpha, sample};

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
    flags: u32,
    src_x0: u32,
    src_y0: u32,
    src_w: u32,
    src_h: u32,
    canvas_w: u32,
    stride: u32,
    _pad1: u32,
    _pad2: u32,
};

@group(0) @binding(0) var<storage, read_write> canvas: array<u32>;
@group(0) @binding(1) var<storage, read> src: array<u32>;
@group(0) @binding(2) var<storage, read> mask: array<u32>;
@group(0) @binding(3) var<uniform> params: Params;

// Both the packed canvas and a group's inner canvas are one u32 per pixel with
// bytes laid out R,G,B,A (little-endian word). Planar pixel-layer sources and
// the mask plane are packed four bytes per u32 and read through the byte
// helpers, so a sample's byte offset needs no per-plane alignment.
fn unpack_word(w: u32) -> vec4<f32> {
    return vec4<f32>(
        f32(w & 0xFFu),
        f32((w >> 8u) & 0xFFu),
        f32((w >> 16u) & 0xFFu),
        f32((w >> 24u) & 0xFFu),
    ) / 255.0;
}

fn src_byte(idx: u32) -> u32 {
    return (src[idx / 4u] >> ((idx % 4u) * 8u)) & 0xFFu;
}

fn mask_byte(idx: u32) -> u32 {
    return (mask[idx / 4u] >> ((idx % 4u) * 8u)) & 0xFFu;
}

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

fn pack_word(c: vec3<f32>, a: f32) -> u32 {
    return quant(c.x) | (quant(c.y) << 8u) | (quant(c.z) << 16u) | (quant(a) << 24u);
}

// Resolve one canvas pixel's source sample. A packed group source is a full
// canvas word; a pixel layer is planar planes over its clamped rect, with the
// grayscale colour replicated from plane 0 and alpha defaulted by the host.
fn sample_src(i: u32) -> vec4<f32> {
    if ((params.flags & 1u) != 0u) {
        return unpack_word(src[i]);
    }
    let x = i % params.canvas_w;
    let y = i / params.canvas_w;
    if (x < params.src_x0 || x >= params.src_x0 + params.src_w
        || y < params.src_y0 || y >= params.src_y0 + params.src_h) {
        return vec4<f32>(0.0);
    }
    let li = (y - params.src_y0) * params.src_w + (x - params.src_x0);
    let n = params.src_w * params.src_h;
    var rgb = vec3<f32>(
        f32(src_byte(li)), f32(src_byte(li)), f32(src_byte(li)),
    ) / 255.0;
    if ((params.flags & 2u) == 0u) {
        rgb = vec3<f32>(
            f32(src_byte(li)),
            f32(src_byte(n + li)),
            f32(src_byte(2u * n + li)),
        ) / 255.0;
    }
    let a_plane = select(3u, 1u, (params.flags & 2u) != 0u);
    return vec4<f32>(rgb, f32(src_byte(a_plane * n + li)) / 255.0);
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
    let i = gid.x + gid.y * params.stride;
    if (i >= params.count) { return; }
    let mask_a = f32(mask_byte(i)) / 255.0;

    if (params.adj_kind != 0u) {
        let cb = unpack_word(canvas[i]);
        let ab = cb.a;
        if (ab <= 0.0) { return; }
        let as_ = ab * params.opacity * mask_a;
        if (as_ <= 0.0) { return; }
        let cs = adjust(params.adj_kind, cb.rgb, params.p0, params.p1, params.p2);
        let b = blend(params.mode, cb.rgb, cs);
        let ao = as_ + ab * (1.0 - as_);
        if (ao <= 0.0) {
            canvas[i] = 0u;
            return;
        }
        let co = ((1.0 - ab) * as_ * cs + as_ * ab * b + (1.0 - as_) * ab * cb.rgb) / ao;
        canvas[i] = pack_word(co, ao);
        return;
    }

    let s = sample_src(i);
    if (s.a <= 0.0) { return; }
    let as_ = s.a * params.opacity * mask_a;
    if (as_ <= 0.0) { return; }

    let cb = unpack_word(canvas[i]);
    let ab = cb.a;
    let b = blend(params.mode, cb.rgb, s.rgb);
    let ao = as_ + ab * (1.0 - as_);
    if (ao <= 0.0) {
        canvas[i] = 0u;
        return;
    }
    let co = ((1.0 - ab) * as_ * s.rgb + as_ * ab * b + (1.0 - as_) * ab * cb.rgb) / ao;
    canvas[i] = pack_word(co, ao);
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

/// The process-wide cached device and queue, shared by the compositor and the
/// M27 filter kernels. `None` when no Vulkan adapter is usable. Cheap to call
/// repeatedly and never panics.
pub(crate) fn shared_device() -> Option<(&'static wgpu::Device, &'static wgpu::Queue)> {
    devices().ok().map(|d| (&d.device, &d.queue))
}

/// 2-D compute grid for `n` single-invocation items at 64 per workgroup.
///
/// `gx` covers one row (capped at the device's per-dimension limit) and `gy`
/// stacks the remaining rows, so `gx * 64 * gy >= n`. Returns `None` when the
/// `gx * gy` workgroup product itself exceeds the limit — the device cannot
/// address that many workgroups (65535² × 64 ≈ 2.8×10¹⁴ items). `gx` is always
/// ≥ 1; a zero-item dispatch yields `gy == 0`.
///
/// ponytail: with `n: u32` and the standard 65535 limit, `limit² × 64` exceeds
/// `u32::MAX`, so the product guard can never fire on a compliant device; it
/// exists for adapters that report a smaller `max_compute_workgroups_per_dimension`.
pub(crate) fn grid_2d(n: u32, max_per_dim: u32) -> Option<(u32, u32)> {
    let gx = n.div_ceil(64).min(max_per_dim).max(1);
    let gy = u64::from(n).div_ceil(u64::from(gx) * 64);
    (gy <= u64::from(max_per_dim)).then_some((gx, gy as u32))
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
        let n = u64::from(w) * u64::from(h);
        let limits = device.limits();
        let bytes = n.saturating_mul(4);
        let limit = u64::from(limits.max_compute_workgroups_per_dimension);
        if bytes > limits.max_storage_buffer_binding_size
            || bytes > limits.max_buffer_size
            || n > limit.saturating_mul(limit).saturating_mul(64)
            || n > u64::from(u32::MAX)
        {
            return Err(GpuError::TooLarge);
        }
        let n = n as u32;
        let params = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("pictura-blend-params"),
            size: 64,
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
            &vec![0u8; self.n as usize * 4],
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

    /// Assemble the source for one pixel layer over its clamped canvas rect,
    /// matching `composite_pixels`: planar 8-bit channel planes, grayscale
    /// replicating channel 0 (one colour plane), and straight alpha defaulting
    /// to 255 when absent. The shader samples by canvas pixel; outside the rect
    /// the source alpha is 0.
    fn build_source(&self, layer: &Layer, doc: &Document) -> Option<(wgpu::Buffer, SrcLayout)> {
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
        let cw = (x1 - x0) as usize;
        let ch = (y1 - y0) as usize;
        let n = cw * ch;
        let planes = if gray { 2 } else { 4 };
        let ch0 = channel(layer, 0);
        let ch1 = channel(layer, 1).or(ch0);
        let ch2 = channel(layer, 2).or(ch0);
        let alpha = channel(layer, -1);

        let mut data = vec![0u8; planes * n];
        for y in y0..y1 {
            let row = (y - y0) as usize;
            for (col, x) in (x0..x1).enumerate() {
                let li = (y - layer.rect.top) as usize * lw + (x - layer.rect.left) as usize;
                let d = row * cw + col;
                let a = sample(alpha, li).unwrap_or(255);
                if gray {
                    data[d] = sample(ch0, li).unwrap_or(0);
                    data[n + d] = a;
                } else {
                    data[d] = sample(ch0, li).unwrap_or(0);
                    data[n + d] = sample(ch1, li).unwrap_or(0);
                    data[2 * n + d] = sample(ch2, li).unwrap_or(0);
                    data[3 * n + d] = a;
                }
            }
        }
        pad_to_4(&mut data);
        let buffer = self.make_buffer("pictura-src", wgpu::BufferUsages::STORAGE, &data);
        Some((
            buffer,
            SrcLayout {
                x0: x0 as u32,
                y0: y0 as u32,
                w: cw as u32,
                h: ch as u32,
                gray,
                packed: false,
            },
        ))
    }

    /// Per-canvas-pixel mask coverage as an 8-bit plane, reusing the CPU
    /// `mask_alpha`. A group's inner buffer and an adjustment layer act across
    /// the whole canvas; a pixel layer only over its clamped rect (outside it
    /// the source alpha is 0, so the coverage is irrelevant).
    ///
    /// ponytail: the plane is canvas-sized even for a rect-scoped layer; bound
    /// it by the rect if a many-small-layers mask profile ever shows up.
    fn build_mask(&self, layer: &Layer) -> wgpu::Buffer {
        let n = self.n as usize;
        let mut data = vec![0u8; n];
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
        if x1 > x0 && y1 > y0 {
            let stride = self.w as usize;
            for y in y0..y1 {
                let row = y as usize * stride;
                for x in x0..x1 {
                    data[row + x as usize] = mask_alpha(layer, x, y);
                }
            }
        }
        pad_to_4(&mut data);
        self.make_buffer("pictura-mask", wgpu::BufferUsages::STORAGE, &data)
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
            // The inner canvas is already packed 8-bit RGBA: bind it directly
            // as the source, no `f32` re-materialization.
            self.dispatch(
                canvas,
                &inner,
                &mask,
                PACKED_SRC,
                layer.blend,
                layer.opacity,
                NO_ADJ,
            );
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
                self.dispatch(
                    canvas,
                    &mask,
                    &mask,
                    PACKED_SRC,
                    layer.blend,
                    layer.opacity,
                    adj,
                );
            }
            return;
        }
        if let Some((src, layout)) = self.build_source(layer, doc) {
            let mask = self.build_mask(layer);
            self.dispatch(
                canvas,
                &src,
                &mask,
                layout,
                layer.blend,
                layer.opacity,
                NO_ADJ,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn dispatch(
        &self,
        canvas: &wgpu::Buffer,
        src: &wgpu::Buffer,
        mask: &wgpu::Buffer,
        layout: SrcLayout,
        mode: BlendMode,
        opacity: u8,
        adj: (u32, i32, i32, i32),
    ) {
        let flags = u32::from(layout.packed) | (u32::from(layout.gray) << 1);
        let (gx, gy) = grid_2d(
            self.n,
            self.device.limits().max_compute_workgroups_per_dimension,
        )
        .expect("Gpu::new rejected a canvas past the 2-D workgroup limit");
        let params = [
            mode_id(mode).to_le_bytes(),
            (opacity as f32 / 255.0).to_le_bytes(),
            self.n.to_le_bytes(),
            adj.0.to_le_bytes(),
            adj.1.to_le_bytes(),
            adj.2.to_le_bytes(),
            adj.3.to_le_bytes(),
            flags.to_le_bytes(),
            layout.x0.to_le_bytes(),
            layout.y0.to_le_bytes(),
            layout.w.to_le_bytes(),
            layout.h.to_le_bytes(),
            self.w.to_le_bytes(),
            (gx * 64).to_le_bytes(),
            0u32.to_le_bytes(),
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
            pass.dispatch_workgroups(gx, gy, 1);
        }
        self.queue.submit(Some(encoder.finish()));
    }

    fn read_canvas(&self, canvas: &wgpu::Buffer) -> Result<Vec<u8>, GpuError> {
        let size = u64::from(self.n) * 4;
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
        let out = mapped.to_vec();
        drop(mapped);
        staging.unmap();
        Ok(out)
    }

    /// De-interleave packed RGBA8 readback into the planar straight-alpha
    /// `PixelBuffer`, identical in shape to `composite_rgba`.
    fn to_pixel_buffer(&self, packed: &[u8]) -> PixelBuffer {
        let plane = self.n as usize;
        let mut out = PixelBuffer::new(self.w, self.h, 4);
        for i in 0..plane {
            let w = u32::from_le_bytes([
                packed[i * 4],
                packed[i * 4 + 1],
                packed[i * 4 + 2],
                packed[i * 4 + 3],
            ]);
            out.data[i] = (w & 0xFF) as u8;
            out.data[plane + i] = ((w >> 8) & 0xFF) as u8;
            out.data[2 * plane + i] = ((w >> 16) & 0xFF) as u8;
            out.data[3 * plane + i] = ((w >> 24) & 0xFF) as u8;
        }
        out
    }
}

/// How the shader should read the source binding for one dispatch.
#[derive(Clone, Copy)]
struct SrcLayout {
    x0: u32,
    y0: u32,
    w: u32,
    h: u32,
    gray: bool,
    packed: bool,
}

/// A group's inner canvas: one packed RGBA word per canvas pixel, full canvas.
const PACKED_SRC: SrcLayout = SrcLayout {
    x0: 0,
    y0: 0,
    w: 0,
    h: 0,
    gray: false,
    packed: true,
};

/// A byte-packed `array<u32>` binding's size and every upload must be a
/// multiple of 4; pad the tail rather than relying on the caller.
fn pad_to_4(data: &mut Vec<u8>) {
    while !data.len().is_multiple_of(4) {
        data.push(0);
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

#[cfg(test)]
mod tests {
    use super::grid_2d;

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
}
