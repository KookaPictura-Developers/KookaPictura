use std::borrow::Cow;
use std::sync::OnceLock;

use super::plan::{Combine, Plan};
use super::{
    COMBINE_HIGH_PASS, COMBINE_UNSHARP, MODE_COMBINE, MODE_KERNEL, MODE_MEDIAN, MODE_MORPH_H,
    MODE_MORPH_V, MODE_MOTION, MODE_OIL_AGG, MODE_OIL_HEIGHT, MODE_OIL_LUMA, MODE_OIL_SHADE,
    MODE_SEP_H, MODE_SEP_V, MODE_SURFACE,
};

pub(super) fn run(
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
    let npix = (w as usize) * (h as usize);
    // Only the separable/morphology paths need the unrounded f32 intermediate
    // (one f32 per byte); Oil Paint needs four f32 planes per pixel (lum +
    // three aggregated colour planes, with height reusing the lum plane). The
    // other modes bind a 4-byte dummy rather than allocating it.
    let mid_size = match plan {
        Plan::OilPaint { .. } => (npix * 4 * 4) as u64,
        Plan::Separable(_) | Plan::Morph { .. } => (total * 4) as u64,
        _ => 4,
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
    // Oil Paint's first passes dispatch one invocation per pixel (`npix`), so
    // the grid must cover the larger of the word count and the pixel count.
    crate::gpu::grid_2d(
        count.max(npix as u32),
        limits.max_compute_workgroups_per_dimension,
    )?;

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
        size: 128,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let final_buf = match plan {
        Plan::Kernel {
            weights: _,
            k,
            norm,
            offset,
            repeat,
        } => {
            let mut p = Params::new(MODE_KERNEL, w, h);
            p.ksize = *k;
            p.support = (*k / 2) as i32;
            p.norm = *norm;
            p.offset = *offset;
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
        Plan::Surface {
            radius,
            two_sig_sq,
            two_thr_sq,
        } => {
            let mut p = Params::new(MODE_SURFACE, w, h);
            p.support = *radius as i32;
            p.norm = *two_sig_sq;
            p.thr = *two_thr_sq;
            dispatch(device, queue, &p, &a, &b, &mid, &weights, &params, count);
            &b
        }
        Plan::Morph { radius, dilate } => {
            let mut p = Params::new(MODE_MORPH_H, w, h);
            p.support = *radius as i32;
            p.taps = u32::from(*dilate);
            dispatch(device, queue, &p, &a, &b, &mid, &weights, &params, count);
            p.mode = MODE_MORPH_V;
            dispatch(device, queue, &p, &a, &b, &mid, &weights, &params, count);
            &b
        }
        Plan::Median { radius } => {
            // Mirror `noise::median`'s cap: beyond max(w, h) every clamped
            // sample is interior, so a larger window would change the multiset.
            let r = (*radius).min(w.max(h));
            let mut p = Params::new(MODE_MEDIAN, w, h);
            p.support = r as i32;
            dispatch(device, queue, &p, &a, &b, &mid, &weights, &params, count);
            &b
        }
        Plan::OilPaint {
            along,
            perp,
            stop,
            scale_t,
            bristle,
            relief_mag,
            light,
            half,
            shine_t,
        } => {
            let mut p = Params::new(MODE_OIL_LUMA, w, h);
            p.along = *along as u32;
            p.perp = *perp as u32;
            p.stop = *stop;
            p.scale_t = *scale_t;
            p.bristle = *bristle;
            p.relief_mag = *relief_mag;
            p.light_x = light[0];
            p.light_y = light[1];
            p.light_z = light[2];
            p.half_x = half[0];
            p.half_y = half[1];
            p.half_z = half[2];
            p.shine_t = *shine_t;
            let npix = npix as u32;
            dispatch(device, queue, &p, &a, &b, &mid, &weights, &params, npix);
            p.mode = MODE_OIL_AGG;
            dispatch(device, queue, &p, &a, &b, &mid, &weights, &params, npix);
            p.mode = MODE_OIL_HEIGHT;
            dispatch(device, queue, &p, &a, &b, &mid, &weights, &params, npix);
            p.mode = MODE_OIL_SHADE;
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
        Plan::Motion { .. }
        | Plan::Surface { .. }
        | Plan::Morph { .. }
        | Plan::Median { .. }
        | Plan::OilPaint { .. } => &[],
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
    offset: f32,
    along: u32,
    perp: u32,
    light_x: f32,
    light_y: f32,
    light_z: f32,
    half_x: f32,
    half_y: f32,
    half_z: f32,
    scale_t: f32,
    bristle: f32,
    relief_mag: f32,
    shine_t: f32,
    stop: f32,
    stride: u32,
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
            offset: 0.0,
            along: 1,
            perp: 0,
            light_x: 0.0,
            light_y: 0.0,
            light_z: 1.0,
            half_x: 0.0,
            half_y: 0.0,
            half_z: 1.0,
            scale_t: 0.0,
            bristle: 0.0,
            relief_mag: 1.0,
            shine_t: 0.0,
            stop: 1.0,
            stride: 0,
        }
    }

    fn bytes(&self) -> [u8; 128] {
        let mut out = [0u8; 128];
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
            self.offset.to_bits(),
            self.along,
            self.perp,
            self.light_x.to_bits(),
            self.light_y.to_bits(),
            self.light_z.to_bits(),
            self.half_x.to_bits(),
            self.half_y.to_bits(),
            self.half_z.to_bits(),
            self.scale_t.to_bits(),
            self.bristle.to_bits(),
            self.relief_mag.to_bits(),
            self.shine_t.to_bits(),
            self.stop.to_bits(),
            self.stride,
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
    let (gx, gy) = crate::gpu::grid_2d(count, device.limits().max_compute_workgroups_per_dimension)
        .expect("run() rejected a count past the 2-D workgroup limit");
    let mut bytes = *params;
    bytes.stride = gx * 64;
    queue.write_buffer(params_buf, 0, &bytes.bytes());
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
        pass.dispatch_workgroups(gx, gy, 1);
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
    offset: f32,
    along: u32,
    perp: u32,
    light_x: f32,
    light_y: f32,
    light_z: f32,
    half_x: f32,
    half_y: f32,
    half_z: f32,
    scale_t: f32,
    bristle: f32,
    relief_mag: f32,
    shine_t: f32,
    stop: f32,
    stride: u32,
    _p1: u32,
    _p2: u32,
    _p3: u32,
    _p4: u32,
    _p5: u32,
    _p6: u32,
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

fn src_val(idx: u32) -> u32 {
    let w = src[idx / 4u];
    return (w >> ((idx % 4u) * 8u)) & 0xFFu;
}

// Rec.601 luma; `ci` is a pixel index within plane 0.
fn luma3(ci: u32) -> f32 {
    let n = p.w * p.h;
    return 0.299 * f32(src_val(ci))
        + 0.587 * f32(src_val(n + ci))
        + 0.114 * f32(src_val(2u * n + ci));
}

fn morph_best(a: u32, b: u32) -> u32 {
    if (p.taps != 0u) { return max(a, b); }
    return min(a, b);
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
    return acc / p.norm + p.offset;
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

// Direct bilateral: spatial Gaussian × range Gaussian on neighbour luma,
// mirroring `pictura_filters::blur::surface` (there is no hard threshold cut).
// ponytail: O(pixels·radius²) direct scan, same ceiling as the CPU oracle;
// swap for a guided/box-range approximation only if radius 100 ever gets hot.
fn surface_val(idx: u32) -> f32 {
    let n = p.w * p.h;
    let plane = idx / n;
    let rem = idx % n;
    let x = rem % p.w;
    let y = rem / p.w;
    let base = plane * n;
    let r = p.support;
    let center_luma = luma3(rem);
    var sum = 0.0;
    var acc = 0.0;
    for (var ky: i32 = 0; ky <= 2 * r; ky = ky + 1) {
        let dy = ky - r;
        let sy = clampi(i32(y) + dy, p.h);
        for (var kx: i32 = 0; kx <= 2 * r; kx = kx + 1) {
            let dx = kx - r;
            let sx = clampi(i32(x) + dx, p.w);
            let si = sy * p.w + sx;
            let dl = abs(luma3(si) - center_luma);
            var wgt = exp(-f32(dx * dx + dy * dy) / p.norm);
            wgt = wgt * exp(-(dl * dl) / p.thr);
            sum = sum + wgt;
            acc = acc + wgt * src_byte(base + si);
        }
    }
    return acc / sum;
}

fn morph_h_word(base_idx: u32, total: u32) {
    let n = p.w * p.h;
    let r = p.support;
    for (var bb: u32 = 0u; bb < 4u; bb = bb + 1u) {
        let idx = base_idx + bb;
        if (idx >= total) { break; }
        let plane = idx / n;
        let rem = idx % n;
        let x = rem % p.w;
        let y = rem / p.w;
        let row = plane * n + y * p.w;
        var best = src_val(row + clampi(i32(x) - r, p.w));
        for (var i: i32 = 0; i <= 2 * r; i = i + 1) {
            let sx = clampi(i32(x) + i - r, p.w);
            best = morph_best(best, src_val(row + sx));
        }
        mid[idx] = f32(best);
    }
}

fn morph_v_val(idx: u32) -> f32 {
    let n = p.w * p.h;
    let plane = idx / n;
    let rem = idx % n;
    let x = rem % p.w;
    let y = rem / p.w;
    let base = plane * n;
    let r = p.support;
    var best = u32(mid[base + y * p.w + x]);
    for (var i: i32 = 0; i <= 2 * r; i = i + 1) {
        let sy = clampi(i32(y) + i - r, p.h);
        best = morph_best(best, u32(mid[base + sy * p.w + x]));
    }
    return f32(best);
}

fn count_le_window(base: u32, x: u32, y: u32, v: u32) -> u32 {
    let r = p.support;
    var cnt = 0u;
    for (var dy: i32 = 0; dy <= 2 * r; dy = dy + 1) {
        let sy = clampi(i32(y) + dy - r, p.h);
        for (var dx: i32 = 0; dx <= 2 * r; dx = dx + 1) {
            let sx = clampi(i32(x) + dx - r, p.w);
            if (src_val(base + sy * p.w + sx) <= v) { cnt = cnt + 1u; }
        }
    }
    return cnt;
}

// Exact per-channel median: the window count is always odd, so the middle
// sorted element is the unique smallest value whose `<= v` count reaches
// `(k*k + 1) / 2`; an 8-step binary search over 0..=255 finds it.
fn median_val(idx: u32) -> u32 {
    let n = p.w * p.h;
    let plane = idx / n;
    let rem = idx % n;
    let x = rem % p.w;
    let y = rem / p.w;
    let base = plane * n;
    let k = 2u * u32(p.support) + 1u;
    let need = (k * k + 1u) / 2u;
    var lo = 0u;
    var hi = 255u;
    loop {
        if (lo >= hi) { break; }
        let mid = (lo + hi) / 2u;
        if (count_le_window(base, x, y, mid) >= need) {
            hi = mid;
        } else {
            lo = mid + 1u;
        }
    }
    return lo;
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

fn normalize3(v: vec3<f32>) -> vec3<f32> {
    let len = sqrt(dot(v, v));
    if (len < 1e-9) { return vec3<f32>(0.0, 0.0, 1.0); }
    return v / len;
}

// --- Oil Paint, mirroring `pictura_filters::oil_paint` stage for stage ------
// Layout of the `mid` f32 buffer while Oil Paint runs: [0, n) holds luma for
// the first pass and the height field for the last two; [n, 4n) holds the
// three aggregated colour planes. `weights` is unused.
//
// ponytail: the stages run in f32 where the CPU oracle uses f64, so a sample
// whose rotated-grid offset lands within ~1e-7 of a `.5` boundary can pick a
// different neighbour than the CPU. The parity corpus shows ≤1 LSB; if a
// parameter set ever exceeds it, precompute the orientation offsets on the CPU
// (still f64) and upload them instead of computing atan2/cos/sin here.

// 1. Rec.601 luma into mid[0..n).
fn oil_luma(wi: u32) {
    let n = p.w * p.h;
    if (wi >= n) { return; }
    mid[wi] = 0.299 * src_byte(wi)
        + 0.587 * src_byte(n + wi)
        + 0.114 * src_byte(2u * n + wi);
}

fn oil_at(x: i32, y: i32) -> f32 {
    return mid[clampi(y, p.h) * p.w + clampi(x, p.w)];
}

// 2. Structure orientation (Sobel tangent) + edge-stopping directional
// aggregation into mid[n..4n).
fn oil_agg(wi: u32) {
    let n = p.w * p.h;
    if (wi >= n) { return; }
    let x = wi % p.w;
    let y = wi / p.w;
    let xi = i32(x);
    let yi = i32(y);
    let li = mid[wi];
    let gx = (oil_at(xi + 1, yi - 1) + 2.0 * oil_at(xi + 1, yi) + oil_at(xi + 1, yi + 1))
        - (oil_at(xi - 1, yi - 1) + 2.0 * oil_at(xi - 1, yi) + oil_at(xi - 1, yi + 1));
    let gy = (oil_at(xi - 1, yi + 1) + 2.0 * oil_at(xi, yi + 1) + oil_at(xi + 1, yi + 1))
        - (oil_at(xi - 1, yi - 1) + 2.0 * oil_at(xi, yi - 1) + oil_at(xi + 1, yi - 1));
    let theta = atan2(gy, gx) + 1.5707963267948966;
    let tx = cos(theta);
    let ty = sin(theta);
    let nx = -ty;
    let ny = tx;
    let along = i32(p.along);
    let perp = i32(p.perp);
    var wsum = 0.0;
    var acc0 = 0.0;
    var acc1 = 0.0;
    var acc2 = 0.0;
    for (var d: i32 = -along; d <= along; d = d + 1) {
        for (var k: i32 = -perp; k <= perp; k = k + 1) {
            let ox = i32(round_away(tx * f32(d) + nx * f32(k)));
            let oy = i32(round_away(ty * f32(d) + ny * f32(k)));
            let sx = clampi(xi + ox, p.w);
            let sy = clampi(yi + oy, p.h);
            let s = sy * p.w + sx;
            let dl = mid[s] - li;
            let edge = 1.0 / (1.0 + (dl / p.stop) * (dl / p.stop));
            let space = 1.0 / (1.0 + f32(d * d + k * k) * 0.12);
            let wt = edge * space;
            wsum = wsum + wt;
            acc0 = acc0 + src_byte(s) * wt;
            acc1 = acc1 + src_byte(n + s) * wt;
            acc2 = acc2 + src_byte(2u * n + s) * wt;
        }
    }
    mid[n + wi] = acc0 / wsum;
    mid[2u * n + wi] = acc1 / wsum;
    mid[3u * n + wi] = acc2 / wsum;
}

// 3. Height field (luma thickness + bristle ridge) into mid[0..n), overwriting
// the luma no longer needed.
fn oil_height(wi: u32) {
    let n = p.w * p.h;
    if (wi >= n) { return; }
    let x = wi % p.w;
    let y = wi / p.w;
    let al = (0.299 * mid[n + wi]
        + 0.587 * mid[2u * n + wi]
        + 0.114 * mid[3u * n + wi]) / 255.0;
    let r = sin(f32(x * 37u + y * 17u) * 0.11);
    mid[wi] = al * (0.2 + 0.8 * p.scale_t) + p.bristle * 0.35 * r * r;
}

// 4. Lambert/Blinn-Phong relief shading for one output byte (plane = idx / n).
fn oil_shade(idx: u32) -> f32 {
    let n = p.w * p.h;
    let plane = idx / n;
    let px = idx % n;
    let x = px % p.w;
    let y = px / p.w;
    let row = y * p.w;
    let gx = mid[row + clampi(i32(x) + 1, p.w)] - mid[row + clampi(i32(x) - 1, p.w)];
    let gy = mid[clampi(i32(y) + 1, p.h) * p.w + x] - mid[clampi(i32(y) - 1, p.h) * p.w + x];
    let nrm = normalize3(vec3<f32>(-gx * p.relief_mag, -gy * p.relief_mag, 1.0));
    let lambert = max(dot(nrm, vec3<f32>(p.light_x, p.light_y, p.light_z)), 0.0);
    let spec = pow(max(dot(nrm, vec3<f32>(p.half_x, p.half_y, p.half_z)), 0.0), 28.0) * p.shine_t;
    let lightf = 0.55 + 0.45 * lambert;
    return mid[(plane + 1u) * n + px] * lightf + 255.0 * spec * 0.7;
}

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let wi = gid.x + gid.y * p.stride;
    let n = p.w * p.h;
    let total = 3u * n;

    if (p.mode == 20u) {
        oil_luma(wi);
        return;
    }
    if (p.mode == 21u) {
        oil_agg(wi);
        return;
    }
    if (p.mode == 22u) {
        oil_height(wi);
        return;
    }

    let base_idx = wi * 4u;
    if (base_idx >= total) { return; }

    if (p.mode == 1u) {
        sep_h_word(base_idx, total);
        return;
    }
    if (p.mode == 6u) {
        morph_h_word(base_idx, total);
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
        } else if (p.mode == 5u) {
            val = surface_val(idx);
        } else if (p.mode == 7u) {
            val = morph_v_val(idx);
        } else if (p.mode == 8u) {
            val = f32(median_val(idx));
        } else if (p.mode == 23u) {
            val = oil_shade(idx);
        } else {
            val = combine_val(idx, dstw, b);
        }
        outw = outw | (q(val) << (b * 8u));
    }
    dst[wi] = outw;
}
"#;
