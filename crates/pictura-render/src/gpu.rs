//! GPU (wgpu/Vulkan) compositor for the separable blend modes.
//!
//! `composite_gpu` mirrors the CPU oracle [`crate::composite_rgba`] pass for
//! pass: the same per-layer loop, mask/opacity handling, pass-through/isolated
//! group semantics, and the W3C + Photoshop blend formulas, executed by a
//! wgpu compute shader. It is proven, not assumed — the parity test in
//! `tests/gpu_parity.rs` diffs it against the CPU compositor within ±1 LSB.
//!
//! # CPU-only modes
//!
//! Hue, Saturation, Color, Luminosity and Dissolve are **not** implemented on
//! the GPU. [`composite_gpu`] returns [`GpuError::UnsupportedMode`] for any
//! visible layer using one; [`composite_gpu_or_cpu`] then falls back to the CPU
//! compositor. DarkerColor and LighterColor *are* handled (they are per-pixel
//! whole-RGB comparisons, not triplet math), matching the CPU path exactly.
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

use pictura_core::{BlendMode, ColorMode, Document, Layer, PixelBuffer};

use crate::{channel, mask_alpha, sample, to_u8};

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
    /// An adjustment layer is present; only the CPU compositor applies those.
    UnsupportedAdjustment,
}

impl fmt::Display for GpuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GpuError::Unavailable => write!(f, "no usable Vulkan adapter/device"),
            GpuError::TooLarge => write!(f, "document exceeds GPU buffer limits"),
            GpuError::Readback => write!(f, "GPU readback failed"),
            GpuError::UnsupportedMode(m) => write!(f, "blend mode {m:?} is CPU-only"),
            GpuError::UnsupportedAdjustment => write!(f, "adjustment layers are CPU-only"),
        }
    }
}

impl std::error::Error for GpuError {}

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

fn is_cpu_only(mode: BlendMode) -> bool {
    matches!(
        mode,
        BlendMode::Dissolve
            | BlendMode::Hue
            | BlendMode::Saturation
            | BlendMode::Color
            | BlendMode::Luminosity
    )
}

fn check_supported(doc: &Document) -> Result<(), GpuError> {
    fn walk(layer: &Layer) -> Result<(), GpuError> {
        if !layer.visible {
            return Ok(());
        }
        if layer.adjustment.is_some() {
            return Err(GpuError::UnsupportedAdjustment);
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
        // Rejected by `check_supported` before any dispatch.
        BlendMode::Dissolve
        | BlendMode::Hue
        | BlendMode::Saturation
        | BlendMode::Color
        | BlendMode::Luminosity => 0,
    }
}

const SHADER: &str = r#"
struct Params {
    mode: u32,
    opacity: f32,
    count: u32,
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

fn blend(mode: u32, cb: vec3<f32>, cs: vec3<f32>) -> vec3<f32> {
    if (mode == 6u) {
        if (cb.x + cb.y + cb.z <= cs.x + cs.y + cs.z) { return cb; }
        return cs;
    }
    if (mode == 11u) {
        if (cb.x + cb.y + cb.z >= cs.x + cs.y + cs.z) { return cb; }
        return cs;
    }
    return vec3<f32>(sep(mode, cb.x, cs.x), sep(mode, cb.y, cs.y), sep(mode, cb.z, cs.z));
}

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.count) { return; }

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

/// Keeps the wgpu objects that own the raw device alive for the process.
struct Devices {
    _instance: wgpu::Instance,
    _adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
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
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
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
        let params = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("pictura-blend-params"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Ok(Self {
            device,
            queue,
            pipeline,
            layout,
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

    /// Assemble the full-canvas source for one pixel layer, matching
    /// `composite_pixels`: zero outside the layer rect, otherwise the channel
    /// samples (grayscale replicates channel 0) and straight alpha.
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
        let mut data = vec![0f32; self.n as usize * 4];
        let stride = self.w as usize;
        for y in y0..y1 {
            for x in x0..x1 {
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
                let i = (y as usize * stride + x as usize) * 4;
                data[i] = r as f32 / 255.0;
                data[i + 1] = g as f32 / 255.0;
                data[i + 2] = b as f32 / 255.0;
                data[i + 3] = a as f32 / 255.0;
            }
        }
        Some(self.make_buffer(
            "pictura-src",
            wgpu::BufferUsages::STORAGE,
            &f32_bytes(&data),
        ))
    }

    /// Per-canvas-pixel mask coverage, reusing the CPU `mask_alpha`.
    fn build_mask(&self, layer: &Layer) -> wgpu::Buffer {
        let mut data = vec![0f32; self.n as usize];
        let stride = self.w as usize;
        for y in 0..self.h as usize {
            for x in 0..self.w as usize {
                data[y * stride + x] = mask_alpha(layer, x as i32, y as i32) as f32 / 255.0;
            }
        }
        self.make_buffer(
            "pictura-mask",
            wgpu::BufferUsages::STORAGE,
            &f32_bytes(&data),
        )
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
            self.dispatch(canvas, &inner, &mask, layer.blend, layer.opacity);
            return;
        }
        if let Some(src) = self.build_source(layer, doc) {
            let mask = self.build_mask(layer);
            self.dispatch(canvas, &src, &mask, layer.blend, layer.opacity);
        }
    }

    fn dispatch(
        &self,
        canvas: &wgpu::Buffer,
        src: &wgpu::Buffer,
        mask: &wgpu::Buffer,
        mode: BlendMode,
        opacity: u8,
    ) {
        let params = [
            mode_id(mode).to_le_bytes(),
            (opacity as f32 / 255.0).to_le_bytes(),
            self.n.to_le_bytes(),
            0u32.to_le_bytes(),
        ]
        .concat();
        self.queue.write_buffer(&self.params, 0, &params);

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("pictura-blend"),
            layout: &self.layout,
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
            pass.set_pipeline(&self.pipeline);
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

fn f32_bytes(values: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(values.len() * 4);
    for v in values {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}
