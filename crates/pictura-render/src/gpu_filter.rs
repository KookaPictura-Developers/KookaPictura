//! M27: apply `pictura_filters::Filter` kernels as wgpu compute passes.
//!
//! [`apply_filter_active`] runs a pointwise / separable-convolution filter on
//! the GPU when `gpu_enabled` is set and a usable adapter exists; otherwise it
//! delegates to the CPU oracle [`pictura_filters::apply`]. The accelerated set
//! is Gaussian Blur, Box Blur, Motion Blur, Blur, Blur More, Sharpen, Sharpen
//! More, Unsharp Mask and High Pass. Any other filter (or any GPU failure)
//! falls back to the CPU with a byte-identical result and [`Backend::Cpu`].
//!
//! One WGSL compute shader carries every kernel behind a mode selector and a
//! normalized weight buffer:
//!
//! - `KERNEL` — a KxK convolution (Sharpen family; Blur, repeated for Blur
//!   More) with one rounded 8-bit write per pass;
//! - `SEP_H`/`SEP_V` — a separable 1-D convolution keeping an unrounded `f32`
//!   intermediate between the row and column passes, exactly like
//!   `pictura_filters::kernel::gaussian_blur_planes` (Gaussian, Box);
//! - `MOTION` — the directional 1-D sum (Motion Blur);
//! - `COMBINE` — a pointwise mix of the original and a blurred copy (Unsharp
//!   Mask, High Pass).
//!
//! Samples are uploaded planar 8-bit, packed four bytes per `u32` word with the
//! same byte helpers as [`crate::gpu`]; one invocation owns a whole word, so
//! the read-modify-write of a shared word never races. Alpha is never uploaded
//! and never written back, so a 4-channel buffer keeps its alpha bit-for-bit.
//!
//! This module reuses the compositor's cached [`crate::gpu::shared_device`]
//! rather than opening a second Vulkan device, so filters and compositing share
//! one queue and one set of cached pipelines. The multi-pass stages each submit
//! separately (queue order is the barrier), which is fine while per-call setup
//! dominates; batch the encoder if filter dispatch ever shows up in a profile.

use std::borrow::Cow;
use std::sync::OnceLock;

use pictura_core::PixelBuffer;
use pictura_filters::kernel::{gaussian_kernel, sigma_from_radius};
use pictura_filters::{Filter, FilterError};

use crate::gpu::Backend;

const MODE_KERNEL: u32 = 0;
const MODE_SEP_H: u32 = 1;
const MODE_SEP_V: u32 = 2;
const MODE_MOTION: u32 = 3;
const MODE_COMBINE: u32 = 4;

const COMBINE_HIGH_PASS: u32 = 0;
const COMBINE_UNSHARP: u32 = 1;

/// Largest 1-D support the weight buffer accepts; anything larger falls back to
/// the CPU rather than allocating an unbounded kernel.
const MAX_WEIGHTS: usize = 512;

const BLUR_KERNEL: [[i32; 3]; 3] = [[1, 2, 1], [2, 4, 2], [1, 2, 1]];
const SHARPEN_KERNEL: [[i32; 3]; 3] = [[0, -1, 0], [-1, 5, -1], [0, -1, 0]];
const SHARPEN_MORE_KERNEL: [[i32; 3]; 3] = [[-1, -1, -1], [-1, 9, -1], [-1, -1, -1]];

/// Whether a usable adapter exists for the GPU filter kernels.
///
/// Reuses the compositor's shared device ([`crate::gpu::shared_device`]) and
/// requires the four storage buffers the kernel binds; cheap to call
/// repeatedly and never panics.
pub fn filter_gpu_available() -> bool {
    crate::gpu::shared_device()
        .is_some_and(|(d, _)| d.limits().max_storage_buffers_per_shader_stage >= 4)
}

/// Apply `filter` in place; return which backend produced the buffer.
///
/// Uses the GPU when `gpu_enabled && filter_gpu_available()` and a kernel
/// exists for this filter, otherwise the CPU oracle [`pictura_filters::apply`].
/// Any GPU failure or kernel-less filter silently degrades to the CPU. Invalid
/// parameters surface as `Err(FilterError)` from the CPU oracle; never panics.
pub fn apply_filter_active(
    filter: &Filter,
    buf: &mut PixelBuffer,
    gpu_enabled: bool,
) -> Result<Backend, FilterError> {
    if gpu_enabled && filter_gpu_available() && apply_filter_gpu(filter, buf).is_some() {
        return Ok(Backend::Gpu);
    }
    pictura_filters::apply(filter, buf)?;
    Ok(Backend::Cpu)
}

/// One GPU execution plan for a filter, mirroring the CPU kernel construction.
enum Plan {
    Kernel {
        weights: Vec<f32>,
        k: u32,
        norm: f32,
        repeat: u32,
    },
    Separable(SepPlan),
    Motion {
        angle: f32,
        taps: u32,
    },
}

enum Combine {
    None,
    HighPass,
    Unsharp { gain: f32, thr: f32 },
}

struct SepPlan {
    weights: Vec<f32>,
    support: i32,
    norm: f32,
    combine: Combine,
}

/// Mirror `pictura_filters::apply`'s parameter validation and kernel choice.
/// `None` means "no GPU kernel for this filter or these parameters": fall back
/// to the CPU so the oracle still decides the bytes.
fn plan(filter: &Filter) -> Option<Plan> {
    match filter {
        Filter::Blur => Some(kernel3(&BLUR_KERNEL, 16.0, 1)),
        Filter::BlurMore => Some(kernel3(&BLUR_KERNEL, 16.0, 3)),
        Filter::Sharpen => Some(kernel3(&SHARPEN_KERNEL, 1.0, 1)),
        Filter::SharpenMore => Some(kernel3(&SHARPEN_MORE_KERNEL, 1.0, 1)),
        Filter::GaussianBlur { radius } => {
            if !radius.is_finite() || *radius < 0.0 || *radius == 0.0 {
                return None;
            }
            separable_sigma(*radius, Combine::None)
        }
        Filter::BoxBlur { radius } => {
            if *radius == 0 {
                return None;
            }
            let r = *radius as usize;
            if 2 * r + 1 > MAX_WEIGHTS {
                return None;
            }
            Some(Plan::Separable(SepPlan {
                weights: vec![1.0; 2 * r + 1],
                support: r as i32,
                norm: (2 * r + 1) as f32,
                combine: Combine::None,
            }))
        }
        Filter::MotionBlur { angle, distance } => {
            if !angle.is_finite() || !(-360.0..=360.0).contains(angle) {
                return None;
            }
            if !(1..=999).contains(distance) {
                return None;
            }
            Some(Plan::Motion {
                angle: *angle as f32,
                taps: *distance,
            })
        }
        Filter::UnsharpMask {
            amount,
            radius,
            threshold,
        } => {
            if !(1.0..=500.0).contains(amount) {
                return None;
            }
            if !radius.is_finite() || *radius <= 0.0 {
                return None;
            }
            separable_sigma(
                *radius,
                Combine::Unsharp {
                    gain: (*amount / 100.0) as f32,
                    thr: f32::from(*threshold),
                },
            )
        }
        Filter::HighPass { radius } => {
            if !radius.is_finite() || *radius <= 0.0 {
                return None;
            }
            separable_sigma(*radius, Combine::HighPass)
        }
        _ => None,
    }
}

fn kernel3(kernel: &[[i32; 3]; 3], norm: f32, repeat: u32) -> Plan {
    Plan::Kernel {
        weights: kernel.iter().flatten().map(|&v| v as f32).collect(),
        k: 3,
        norm,
        repeat,
    }
}

fn separable_sigma(radius: f64, combine: Combine) -> Option<Plan> {
    let kernel = gaussian_kernel(sigma_from_radius(radius));
    let support = (kernel.len() / 2) as i32;
    if kernel.len() > MAX_WEIGHTS {
        return None;
    }
    Some(Plan::Separable(SepPlan {
        weights: kernel.into_iter().map(|w| w as f32).collect(),
        support,
        norm: 1.0,
        combine,
    }))
}

fn apply_filter_gpu(filter: &Filter, buf: &mut PixelBuffer) -> Option<()> {
    let plan = plan(filter)?;
    if buf.channels != 3 && buf.channels != 4 {
        return None;
    }
    let n = buf.pixel_count();
    if n == 0 || buf.data.len() != n * buf.channels as usize {
        return None;
    }
    let mut input = Vec::with_capacity(n * 3);
    for c in 0..3 {
        input.extend_from_slice(&buf.data[c * n..c * n + n]);
    }
    let (device, queue) = crate::gpu::shared_device()?;
    let out = run(device, queue, &plan, &input, buf.width, buf.height)?;
    for c in 0..3 {
        buf.data[c * n..c * n + n].copy_from_slice(&out[c * n..c * n + n]);
    }
    Some(())
}

fn run(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    plan: &Plan,
    input: &[u8],
    w: u32,
    h: u32,
) -> Option<Vec<u8>> {
    let total = input.len();
    let padded = total.div_ceil(4) * 4;
    let u8_size = padded as u64;
    // Only the separable path needs the unrounded f32 intermediate; the other
    // modes bind a 4-byte dummy rather than allocating it.
    let mid_size = if matches!(plan, Plan::Separable(_)) {
        (total * 4) as u64
    } else {
        4
    };
    let limits = device.limits();
    if u8_size > limits.max_storage_buffer_binding_size
        || u8_size > limits.max_buffer_size
        || mid_size > limits.max_storage_buffer_binding_size
        || mid_size > limits.max_buffer_size
    {
        return None;
    }
    let count = (padded / 4) as u32;
    if count.div_ceil(64) > limits.max_compute_workgroups_per_dimension {
        return None;
    }

    let mut upload = input.to_vec();
    upload.resize(padded, 0);
    let a = make_u8(device, "pictura-filter-a", u8_size);
    let b = make_u8(device, "pictura-filter-b", u8_size);
    let mid = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("pictura-filter-mid"),
        size: mid_size.max(4),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let weights_bytes = weights_bytes(plan);
    let weights = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("pictura-filter-weights"),
        size: (weights_bytes.len() as u64).max(4),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&a, 0, &upload);
    queue.write_buffer(&weights, 0, &weights_bytes);
    let params = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("pictura-filter-params"),
        size: 64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let final_buf = match plan {
        Plan::Kernel {
            weights: _,
            k,
            norm,
            repeat,
        } => {
            let mut p = Params::new(MODE_KERNEL, w, h);
            p.ksize = *k;
            p.support = (*k / 2) as i32;
            p.norm = *norm;
            let (mut src, mut dst) = (&a, &b);
            for _ in 0..*repeat {
                dispatch(device, queue, &p, src, dst, &mid, &weights, &params, count);
                std::mem::swap(&mut src, &mut dst);
            }
            src
        }
        Plan::Separable(sp) => {
            let mut p = Params::new(MODE_SEP_H, w, h);
            p.ksize = sp.weights.len() as u32;
            p.support = sp.support;
            p.norm = sp.norm;
            dispatch(device, queue, &p, &a, &b, &mid, &weights, &params, count);
            p.mode = MODE_SEP_V;
            dispatch(device, queue, &p, &a, &b, &mid, &weights, &params, count);
            if let Combine::Unsharp { gain, thr } = sp.combine {
                p.mode = MODE_COMBINE;
                p.taps = COMBINE_UNSHARP;
                p.gain = gain;
                p.thr = thr;
                dispatch(device, queue, &p, &a, &b, &mid, &weights, &params, count);
            } else if matches!(sp.combine, Combine::HighPass) {
                p.mode = MODE_COMBINE;
                p.taps = COMBINE_HIGH_PASS;
                dispatch(device, queue, &p, &a, &b, &mid, &weights, &params, count);
            }
            &b
        }
        Plan::Motion { angle, taps } => {
            let mut p = Params::new(MODE_MOTION, w, h);
            p.angle = *angle;
            p.taps = *taps;
            dispatch(device, queue, &p, &a, &b, &mid, &weights, &params, count);
            &b
        }
    };

    let out = readback(device, queue, final_buf, u8_size)?;
    Some(out)
}

fn weights_bytes(plan: &Plan) -> Vec<u8> {
    let weights: &[f32] = match plan {
        Plan::Kernel { weights, .. } => weights,
        Plan::Separable(sp) => &sp.weights,
        Plan::Motion { .. } => &[],
    };
    weights
        .iter()
        .flat_map(|w| w.to_bits().to_le_bytes())
        .collect()
}

fn make_u8(device: &wgpu::Device, label: &str, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: size.max(4),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

#[derive(Clone, Copy)]
struct Params {
    mode: u32,
    w: u32,
    h: u32,
    ksize: u32,
    taps: u32,
    support: i32,
    norm: f32,
    gain: f32,
    thr: f32,
    angle: f32,
    count: u32,
}

impl Params {
    fn new(mode: u32, w: u32, h: u32) -> Self {
        Self {
            mode,
            w,
            h,
            ksize: 1,
            taps: 0,
            support: 0,
            norm: 1.0,
            gain: 0.0,
            thr: 0.0,
            angle: 0.0,
            count: 0,
        }
    }

    fn bytes(&self) -> [u8; 64] {
        let mut out = [0u8; 64];
        let fields = [
            self.mode,
            self.w,
            self.h,
            self.ksize,
            self.taps,
            self.support as u32,
            self.norm.to_bits(),
            self.gain.to_bits(),
            self.thr.to_bits(),
            self.angle.to_bits(),
            self.count,
        ];
        for (i, v) in fields.iter().enumerate() {
            out[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
        }
        out
    }
}

#[allow(clippy::too_many_arguments)]
fn dispatch(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    params: &Params,
    src: &wgpu::Buffer,
    dst: &wgpu::Buffer,
    mid: &wgpu::Buffer,
    weights: &wgpu::Buffer,
    params_buf: &wgpu::Buffer,
    count: u32,
) {
    queue.write_buffer(params_buf, 0, &params.bytes());
    let res = filter_resources(device);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("pictura-filter"),
        layout: &res.layout,
        entries: &[
            entry(0, dst),
            entry(1, src),
            entry(2, mid),
            entry(3, weights),
            wgpu::BindGroupEntry {
                binding: 4,
                resource: params_buf.as_entire_binding(),
            },
        ],
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("pictura-filter"),
    });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("pictura-filter"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&res.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(count.div_ceil(64), 1, 1);
    }
    queue.submit(Some(encoder.finish()));
}

fn entry(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: buffer.as_entire_binding(),
    }
}

fn readback(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    buffer: &wgpu::Buffer,
    size: u64,
) -> Option<Vec<u8>> {
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("pictura-filter-readback"),
        size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("pictura-filter-readback"),
    });
    encoder.copy_buffer_to_buffer(buffer, 0, &staging, 0, size);
    queue.submit(Some(encoder.finish()));

    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = tx.send(result);
    });
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    rx.recv().ok()?.ok()?;
    let mapped = slice.get_mapped_range().ok()?;
    let out = mapped.to_vec();
    drop(mapped);
    staging.unmap();
    Some(out)
}

struct FilterResources {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
}

impl FilterResources {
    fn new(device: &wgpu::Device) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("pictura-filter-layout"),
            entries: &[
                storage_entry(0, false),
                storage_entry(1, true),
                storage_entry(2, false),
                storage_entry(3, true),
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
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
            label: Some("pictura-filter"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(SHADER)),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pictura-filter"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("pictura-filter"),
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point: Some("cs_main"),
            compilation_options: Default::default(),
            cache: None,
        });
        Self { pipeline, layout }
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

/// The filter pipeline and bind-group layout, created once against the shared
/// device, so repeated applications reuse the compiled kernel.
fn filter_resources(device: &wgpu::Device) -> &'static FilterResources {
    static RESOURCES: OnceLock<FilterResources> = OnceLock::new();
    RESOURCES.get_or_init(|| FilterResources::new(device))
}

const SHADER: &str = r#"
struct FParams {
    mode: u32,
    w: u32,
    h: u32,
    ksize: u32,
    taps: u32,
    support: i32,
    norm: f32,
    gain: f32,
    thr: f32,
    angle: f32,
    count: u32,
    _p0: u32,
    _p1: u32,
    _p2: u32,
    _p3: u32,
    _p4: u32,
};

@group(0) @binding(0) var<storage, read_write> dst: array<u32>;
@group(0) @binding(1) var<storage, read> src: array<u32>;
@group(0) @binding(2) var<storage, read_write> mid: array<f32>;
@group(0) @binding(3) var<storage, read> weights: array<f32>;
@group(0) @binding(4) var<uniform> p: FParams;

// Rust's `f32::round` is half-away-from-zero; WGSL's `round` is half-to-even.
fn round_away(v: f32) -> f32 {
    if (v >= 0.0) { return floor(v + 0.5); }
    return ceil(v - 0.5);
}

fn q(v: f32) -> u32 {
    return u32(clamp(round_away(v), 0.0, 255.0));
}

fn src_byte(idx: u32) -> f32 {
    let w = src[idx / 4u];
    return f32((w >> ((idx % 4u) * 8u)) & 0xFFu);
}

fn out_byte(word: u32, b: u32) -> f32 {
    return f32((word >> (b * 8u)) & 0xFFu);
}

fn clampi(v: i32, n: u32) -> u32 {
    if (v < 0) { return 0u; }
    if (v >= i32(n)) { return n - 1u; }
    return u32(v);
}

fn kernel_val(idx: u32) -> f32 {
    let n = p.w * p.h;
    let plane = idx / n;
    let rem = idx % n;
    let x = rem % p.w;
    let y = rem / p.w;
    let base = plane * n;
    let k = p.ksize;
    var acc = 0.0;
    for (var ky = 0u; ky < k; ky = ky + 1u) {
        let sy = clampi(i32(y) + i32(ky) - p.support, p.h);
        for (var kx = 0u; kx < k; kx = kx + 1u) {
            let sx = clampi(i32(x) + i32(kx) - p.support, p.w);
            acc = acc + weights[ky * k + kx] * src_byte(base + sy * p.w + sx);
        }
    }
    return acc / p.norm;
}

fn sep_h_word(base_idx: u32, total: u32) {
    let n = p.w * p.h;
    for (var b = 0u; b < 4u; b = b + 1u) {
        let idx = base_idx + b;
        if (idx >= total) { break; }
        let plane = idx / n;
        let rem = idx % n;
        let x = rem % p.w;
        let y = rem / p.w;
        let base = plane * n;
        var acc = 0.0;
        for (var i = 0u; i < p.ksize; i = i + 1u) {
            let sx = clampi(i32(x) + i32(i) - p.support, p.w);
            acc = acc + weights[i] * src_byte(base + y * p.w + sx);
        }
        mid[idx] = acc / p.norm;
    }
}

fn sep_v_val(idx: u32) -> f32 {
    let n = p.w * p.h;
    let plane = idx / n;
    let rem = idx % n;
    let x = rem % p.w;
    let y = rem / p.w;
    let base = plane * n;
    var acc = 0.0;
    for (var i = 0u; i < p.ksize; i = i + 1u) {
        let sy = clampi(i32(y) + i32(i) - p.support, p.h);
        acc = acc + weights[i] * mid[base + sy * p.w + x];
    }
    return acc / p.norm;
}

fn motion_val(idx: u32) -> f32 {
    let n = p.w * p.h;
    let plane = idx / n;
    let rem = idx % n;
    let x = rem % p.w;
    let y = rem / p.w;
    let base = plane * n;
    let ang = p.angle * 0.017453292519943295;
    let dx = cos(ang);
    let dy = sin(ang);
    let half = (f32(p.taps) - 1.0) / 2.0;
    var acc = 0.0;
    for (var k = 0u; k < p.taps; k = k + 1u) {
        let f = f32(k) - half;
        let ox = i32(round_away(f * dx));
        let oy = i32(round_away(f * dy));
        let sx = clampi(i32(x) + ox, p.w);
        let sy = clampi(i32(y) + oy, p.h);
        acc = acc + src_byte(base + sy * p.w + sx);
    }
    return acc / f32(p.taps);
}

fn combine_val(idx: u32, dstw: u32, b: u32) -> f32 {
    let o = src_byte(idx);
    let blurred = out_byte(dstw, b);
    if (p.taps == 0u) {
        return o - blurred + 128.0;
    }
    let diff = o - blurred;
    if (abs(diff) > p.thr) {
        return o + diff * p.gain;
    }
    return o;
}

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let wi = gid.x;
    let n = p.w * p.h;
    let total = 3u * n;
    let base_idx = wi * 4u;
    if (base_idx >= total) { return; }

    if (p.mode == 1u) {
        sep_h_word(base_idx, total);
        return;
    }

    let dstw = dst[wi];
    var outw = 0u;
    for (var b = 0u; b < 4u; b = b + 1u) {
        let idx = base_idx + b;
        if (idx >= total) { break; }
        var val = 0.0;
        if (p.mode == 0u) {
            val = kernel_val(idx);
        } else if (p.mode == 2u) {
            val = sep_v_val(idx);
        } else if (p.mode == 3u) {
            val = motion_val(idx);
        } else {
            val = combine_val(idx, dstw, b);
        }
        outw = outw | (q(val) << (b * 8u));
    }
    dst[wi] = outw;
}
"#;
