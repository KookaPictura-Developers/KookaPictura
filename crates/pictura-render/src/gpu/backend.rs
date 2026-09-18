use std::borrow::Cow;
use std::sync::OnceLock;

use pictura_core::{BlendMode, ColorMode, Document, Layer, PixelBuffer};

use crate::{channel, decode_adjustment, mask_alpha, sample};

use super::shader::SHADER;
use super::{adjustment_params, mode_id, GpuError, NO_ADJ};

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
pub(super) struct Devices {
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

pub(super) fn devices() -> Result<&'static Devices, GpuError> {
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

pub(super) struct Gpu {
    pub(super) device: wgpu::Device,
    pub(super) queue: wgpu::Queue,
    res: &'static ComputeResources,
    params: wgpu::Buffer,
    /// Region origin in document pixels.
    x0: u32,
    y0: u32,
    /// Region dimensions; the canvas is `w * h` packed words.
    w: u32,
    h: u32,
    pub(super) n: u32,
}

impl Gpu {
    pub(super) fn new(x0: u32, y0: u32, w: u32, h: u32) -> Result<Self, GpuError> {
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
            size: 80,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Ok(Self {
            device,
            queue,
            res: shared.resources(),
            params,
            x0,
            y0,
            w,
            h,
            n,
        })
    }

    pub(super) fn region(&self) -> Region {
        Region {
            x0: self.x0,
            y0: self.y0,
            w: self.w,
            h: self.h,
        }
    }

    pub(super) fn zero_canvas(&self) -> wgpu::Buffer {
        // ponytail: clear_buffer (GPU memset) over a host `vec![0; n*4]` upload;
        // COPY_DST is required by clear_buffer and the readback's COPY_SRC.
        let canvas = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("pictura-canvas"),
            size: u64::from(self.n) * 4,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("pictura-canvas-clear"),
            });
        encoder.clear_buffer(&canvas, 0, None);
        self.queue.submit(Some(encoder.finish()));
        canvas
    }

    pub(super) fn make_buffer(
        &self,
        label: &str,
        usage: wgpu::BufferUsages,
        bytes: &[u8],
    ) -> wgpu::Buffer {
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: bytes.len() as u64,
            usage: usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(&buffer, 0, bytes);
        buffer
    }

    /// Assemble the source for one pixel layer over its layer-rect intersection
    /// with the region, matching `composite_pixels`: planar 8-bit channel
    /// planes, grayscale replicating channel 0 (one colour plane), and straight
    /// alpha defaulting to 255 when absent. The shader samples by document
    /// pixel; outside the intersection the source alpha is 0.
    pub(super) fn build_source(
        &self,
        layer: &Layer,
        doc: &Document,
    ) -> Option<(wgpu::Buffer, SrcLayout)> {
        let (data, layout) = assemble_source(self.region(), layer, doc)?;
        let buffer = self.make_buffer("pictura-src", wgpu::BufferUsages::STORAGE, &data);
        Some((buffer, layout))
    }

    /// Per-region-pixel mask coverage as an 8-bit plane, reusing the CPU
    /// `mask_alpha`. A group's inner buffer and an adjustment layer act across
    /// the whole region; a pixel layer only over its layer-rect intersection
    /// with the region (outside it the source alpha is 0, so the coverage is
    /// irrelevant).
    ///
    /// ponytail: the plane is region-sized even for a rect-scoped layer; bound
    /// it by the rect if a many-small-layers mask profile ever shows up.
    pub(super) fn build_mask(&self, layer: &Layer) -> wgpu::Buffer {
        let data = assemble_mask(self.region(), layer);
        self.make_buffer("pictura-mask", wgpu::BufferUsages::STORAGE, &data)
    }

    pub(super) fn composite_layer(&self, canvas: &wgpu::Buffer, layer: &Layer, doc: &Document) {
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
                255,
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
                    layer.fill,
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
                layer.fill,
                NO_ADJ,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn dispatch(
        &self,
        canvas: &wgpu::Buffer,
        src: &wgpu::Buffer,
        mask: &wgpu::Buffer,
        layout: SrcLayout,
        mode: BlendMode,
        opacity: u8,
        fill: u8,
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
            (fill as f32 / 255.0).to_le_bytes(),
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
            self.x0.to_le_bytes(),
            self.y0.to_le_bytes(),
            self.w.to_le_bytes(),
            (gx * 64).to_le_bytes(),
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

    pub(super) fn read_canvas(&self, canvas: &wgpu::Buffer) -> Result<PixelBuffer, GpuError> {
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

        // De-interleave straight from the mapped slice: no host packed RGBA Vec.
        let mapped = slice.get_mapped_range().map_err(|_| GpuError::Readback)?;
        let out = self.to_pixel_buffer(&mapped);
        drop(mapped);
        staging.unmap();
        Ok(out)
    }

    /// De-interleave packed RGBA8 readback into the planar straight-alpha
    /// `PixelBuffer`, identical in shape to `composite_rgba`. Each LE `u32`
    /// unpacked and masked into four channels is a pure byte permutation, so
    /// walking 4-byte chunks writes the same bytes.
    pub(super) fn to_pixel_buffer(&self, packed: &[u8]) -> PixelBuffer {
        let plane = self.n as usize;
        let mut out = PixelBuffer::new(self.w, self.h, 4);
        for (i, px) in packed.as_chunks::<4>().0.iter().take(plane).enumerate() {
            out.data[i] = px[0];
            out.data[plane + i] = px[1];
            out.data[2 * plane + i] = px[2];
            out.data[3 * plane + i] = px[3];
        }
        out
    }
}

/// The region origin and dimensions a composite runs over. Pure assembly (no
/// device) so the row-wise and per-pixel paths are unit-testable without a GPU.
#[derive(Clone, Copy)]
pub(super) struct Region {
    pub(super) x0: u32,
    pub(super) y0: u32,
    pub(super) w: u32,
    pub(super) h: u32,
}

/// The clamped layer-rect ∩ region intersection plus the plane layout, the
/// shared product of both source-assembly paths.
pub(super) struct SourceGeom {
    pub(super) x0: i32,
    pub(super) y0: i32,
    pub(super) x1: i32,
    pub(super) y1: i32,
    pub(super) cw: usize,
    pub(super) ch: usize,
    pub(super) n: usize,
    pub(super) planes: usize,
    pub(super) gray: bool,
}

impl SourceGeom {
    fn layout(&self) -> SrcLayout {
        SrcLayout {
            x0: self.x0 as u32,
            y0: self.y0 as u32,
            w: self.cw as u32,
            h: self.ch as u32,
            gray: self.gray,
            packed: false,
        }
    }
}

pub(super) fn source_geom(region: Region, layer: &Layer, doc: &Document) -> Option<SourceGeom> {
    if layer.rect.width() <= 0 || layer.rect.height() <= 0 {
        return None;
    }
    let region_right = (region.x0 + region.w) as i32;
    let region_bottom = (region.y0 + region.h) as i32;
    let x0 = layer.rect.left.max(region.x0 as i32);
    let y0 = layer.rect.top.max(region.y0 as i32);
    let x1 = layer.rect.right.min(region_right);
    let y1 = layer.rect.bottom.min(region_bottom);
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    let gray = matches!(
        doc.mode,
        ColorMode::Grayscale | ColorMode::Bitmap | ColorMode::Duotone
    );
    let cw = (x1 - x0) as usize;
    let ch = (y1 - y0) as usize;
    Some(SourceGeom {
        x0,
        y0,
        x1,
        y1,
        cw,
        ch,
        n: cw * ch,
        planes: if gray { 2 } else { 4 },
        gray,
    })
}

/// The retained per-pixel reference assembly; the fast path must match it byte
/// for byte. Kept reachable for the equivalence tests and as the short/absent
/// plane fallback.
pub(super) fn assemble_source_per_pixel(layer: &Layer, g: &SourceGeom) -> Vec<u8> {
    let lw = layer.rect.width() as usize;
    let ch0 = channel(layer, 0);
    let ch1 = channel(layer, 1).or(ch0);
    let ch2 = channel(layer, 2).or(ch0);
    let alpha = channel(layer, -1);
    let n = g.n;
    let mut data = vec![0u8; g.planes * n];
    for y in g.y0..g.y1 {
        let row = (y - g.y0) as usize;
        for (col, x) in (g.x0..g.x1).enumerate() {
            let li = (y - layer.rect.top) as usize * lw + (x - layer.rect.left) as usize;
            let d = row * g.cw + col;
            let a = sample(alpha, li).unwrap_or(255);
            if g.gray {
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
    data
}

/// Whole-row copies when every required channel covers the clamped row
/// intersection; `None` (the fallback) when a present plane is short or channel
/// 0 is absent.
pub(super) fn assemble_source_rowwise(layer: &Layer, g: &SourceGeom) -> Option<Vec<u8>> {
    let lw = layer.rect.width() as usize;
    let base = (g.y0 - layer.rect.top) as usize * lw + (g.x0 - layer.rect.left) as usize;
    let last_end = base + (g.ch - 1) * lw + g.cw;
    let ch0 = channel(layer, 0)?;
    let ch1 = channel(layer, 1).or(Some(ch0))?;
    let ch2 = channel(layer, 2).or(Some(ch0))?;
    let alpha = channel(layer, -1);
    if ch0.len() < last_end || ch1.len() < last_end || ch2.len() < last_end {
        return None;
    }
    if alpha.is_some_and(|a| a.len() < last_end) {
        return None;
    }

    let n = g.n;
    let mut data = vec![0u8; g.planes * n];
    for row in 0..g.ch {
        let src = base + row * lw;
        let d = row * g.cw;
        data[d..d + g.cw].copy_from_slice(&ch0[src..src + g.cw]);
        if g.gray {
            match alpha {
                Some(a) => data[n + d..n + d + g.cw].copy_from_slice(&a[src..src + g.cw]),
                None => data[n + d..n + d + g.cw].fill(255),
            }
        } else {
            data[n + d..n + d + g.cw].copy_from_slice(&ch1[src..src + g.cw]);
            data[2 * n + d..2 * n + d + g.cw].copy_from_slice(&ch2[src..src + g.cw]);
            match alpha {
                Some(a) => {
                    data[3 * n + d..3 * n + d + g.cw].copy_from_slice(&a[src..src + g.cw]);
                }
                None => data[3 * n + d..3 * n + d + g.cw].fill(255),
            }
        }
    }
    pad_to_4(&mut data);
    Some(data)
}

pub(super) fn assemble_source(
    region: Region,
    layer: &Layer,
    doc: &Document,
) -> Option<(Vec<u8>, SrcLayout)> {
    let g = source_geom(region, layer, doc)?;
    let data =
        assemble_source_rowwise(layer, &g).unwrap_or_else(|| assemble_source_per_pixel(layer, &g));
    Some((data, g.layout()))
}

/// Whether `mask_alpha` has data to sample: absent, disabled, or data-less
/// masks are the constant-255 case the row fill covers.
pub(super) fn mask_has_data(layer: &Layer) -> bool {
    layer
        .mask
        .as_ref()
        .is_some_and(|m| !m.disabled && m.data.is_some())
}

/// The coverage influence rectangle: the whole region for a group or an
/// adjustment layer, the clamped layer rect for a pixel layer.
pub(super) fn mask_influence_rect(region: Region, layer: &Layer) -> (i32, i32, i32, i32) {
    let region_right = (region.x0 + region.w) as i32;
    let region_bottom = (region.y0 + region.h) as i32;
    if layer.is_group || layer.adjustment.is_some() {
        (
            region.x0 as i32,
            region.y0 as i32,
            region_right,
            region_bottom,
        )
    } else {
        (
            layer.rect.left.max(region.x0 as i32),
            layer.rect.top.max(region.y0 as i32),
            layer.rect.right.min(region_right),
            layer.rect.bottom.min(region_bottom),
        )
    }
}

/// Constant-255 coverage over the influence rect, 0 elsewhere.
pub(super) fn assemble_mask_fill(region: Region, r: (i32, i32, i32, i32)) -> Vec<u8> {
    let (x0, y0, x1, y1) = r;
    let mut data = vec![0u8; region.w as usize * region.h as usize];
    if x1 > x0 && y1 > y0 {
        let stride = region.w as usize;
        let left = (x0 - region.x0 as i32) as usize;
        let right = (x1 - region.x0 as i32) as usize;
        for y in y0..y1 {
            let row = (y - region.y0 as i32) as usize * stride;
            data[row + left..row + right].fill(255);
        }
    }
    pad_to_4(&mut data);
    data
}

/// The retained per-pixel `mask_alpha` reference, kept for data-carrying masks
/// and the equivalence tests.
pub(super) fn assemble_mask_per_pixel(
    region: Region,
    layer: &Layer,
    r: (i32, i32, i32, i32),
) -> Vec<u8> {
    let (x0, y0, x1, y1) = r;
    let mut data = vec![0u8; region.w as usize * region.h as usize];
    if x1 > x0 && y1 > y0 {
        let stride = region.w as usize;
        for y in y0..y1 {
            let row = (y - region.y0 as i32) as usize * stride;
            for x in x0..x1 {
                data[row + (x - region.x0 as i32) as usize] = mask_alpha(layer, x, y);
            }
        }
    }
    pad_to_4(&mut data);
    data
}

pub(super) fn assemble_mask(region: Region, layer: &Layer) -> Vec<u8> {
    let r = mask_influence_rect(region, layer);
    if mask_has_data(layer) {
        assemble_mask_per_pixel(region, layer, r)
    } else {
        assemble_mask_fill(region, r)
    }
}

/// How the shader should read the source binding for one dispatch.
#[derive(Clone, Copy)]
pub(super) struct SrcLayout {
    x0: u32,
    y0: u32,
    w: u32,
    h: u32,
    gray: bool,
    packed: bool,
}

/// A group's inner canvas: one packed RGBA word per region pixel.
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
