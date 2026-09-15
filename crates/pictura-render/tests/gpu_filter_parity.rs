//! GPU filter parity for the M27 accelerated kernel set.
//!
//! `pictura_filters::apply` is the oracle. Every accelerated filter must match
//! it within ±1 LSB per colour channel and keep alpha bit-for-bit on 4-channel
//! buffers. A filter with no GPU kernel, or the whole path with GPU compute
//! disabled, must return `Backend::Cpu` with bytes identical to the oracle.
//!
//! Tests skip with a printed note when no Vulkan adapter is usable, so a
//! GPU-less CI stays green.

use pictura_core::PixelBuffer;
use pictura_filters::{apply, Filter};
use pictura_render::{apply_filter_active, filter_gpu_available, Backend};

const SIZE: u32 = 32;

/// Accelerated filters with representative parameters.
fn accelerated_filters() -> Vec<Filter> {
    vec![
        Filter::GaussianBlur { radius: 3.0 },
        Filter::BoxBlur { radius: 2 },
        Filter::MotionBlur {
            angle: 0.0,
            distance: 5,
        },
        Filter::Blur,
        Filter::BlurMore,
        Filter::Sharpen,
        Filter::SharpenMore,
        Filter::UnsharpMask {
            amount: 150.0,
            radius: 3.0,
            threshold: 0,
        },
        Filter::HighPass { radius: 3.0 },
    ]
}

/// A kernel-less filter (the painterly family stays CPU-only).
fn unsupported_filter() -> Filter {
    Filter::ColoredPencil {
        pencil_width: 6,
        stroke_pressure: 8,
        paper_brightness: 50,
        foreground: [0, 0, 0],
        background: [255, 255, 255],
        seed: 1,
    }
}

/// 3-channel RGB gradient.
fn gradient_rgb() -> PixelBuffer {
    let n = (SIZE * SIZE) as usize;
    let mut buf = PixelBuffer::new(SIZE, SIZE, 3);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let i = (y * SIZE + x) as usize;
            buf.data[i] = (x * 8) as u8;
            buf.data[n + i] = (y * 8) as u8;
            buf.data[2 * n + i] = ((x + y) * 4) as u8;
        }
    }
    buf
}

/// 4-channel gradient with a non-trivial alpha plane.
fn gradient_rgba() -> PixelBuffer {
    let n = (SIZE * SIZE) as usize;
    let mut buf = PixelBuffer::new(SIZE, SIZE, 4);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let i = (y * SIZE + x) as usize;
            buf.data[i] = (x * 8) as u8;
            buf.data[n + i] = (y * 8) as u8;
            buf.data[2 * n + i] = ((x + y) * 4) as u8;
            buf.data[3 * n + i] = ((x * 7 + y * 13) % 256) as u8;
        }
    }
    buf
}

/// Hard black/white/colour edges: stresses tie-rounding in the separable and
/// 3x3 kernels.
fn edge_rgb() -> PixelBuffer {
    let n = (SIZE * SIZE) as usize;
    let mut buf = PixelBuffer::new(SIZE, SIZE, 3);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let i = (y * SIZE + x) as usize;
            let v = if (x + y) % 4 < 2 { 0 } else { 255 };
            buf.data[i] = v;
            buf.data[n + i] = u8::MAX - v;
            buf.data[2 * n + i] = if x < SIZE / 2 { 0 } else { 255 };
        }
    }
    buf
}

/// A single flat colour: kernels must be a near no-op.
fn flat_rgb() -> PixelBuffer {
    let n = (SIZE * SIZE) as usize;
    let mut buf = PixelBuffer::new(SIZE, SIZE, 3);
    for i in 0..n {
        buf.data[i] = 37;
        buf.data[n + i] = 149;
        buf.data[2 * n + i] = 211;
    }
    buf
}

fn oracle(filter: &Filter, base: &PixelBuffer) -> PixelBuffer {
    let mut buf = base.clone();
    apply(filter, &mut buf).expect("oracle parameters are valid");
    buf
}

/// Colour-plane deltas and an alpha bit-for-bit check. Returns the max colour
/// delta.
fn assert_parity(got: &PixelBuffer, want: &PixelBuffer, label: &str) -> i32 {
    let n = want.pixel_count();
    let mut max_delta = 0i32;
    for c in 0..3 {
        for i in 0..n {
            let d = (want.data[c * n + i] as i32 - got.data[c * n + i] as i32).abs();
            max_delta = max_delta.max(d);
            assert!(
                d <= 1,
                "{label} channel {c} sample {i}: cpu {} vs gpu {}",
                want.data[c * n + i],
                got.data[c * n + i]
            );
        }
    }
    if want.channels == 4 {
        assert_eq!(
            &want.data[3 * n..4 * n],
            &got.data[3 * n..4 * n],
            "{label}: alpha must be untouched"
        );
    }
    max_delta
}

#[test]
fn accelerated_filters_match_cpu_within_one_lsb() {
    if !filter_gpu_available() {
        println!("no usable Vulkan GPU; skipping accelerated filter parity");
        return;
    }
    for (label, base) in [
        ("rgb", gradient_rgb()),
        ("rgba", gradient_rgba()),
        ("edge", edge_rgb()),
        ("flat", flat_rgb()),
    ] {
        for filter in accelerated_filters() {
            let want = oracle(&filter, &base);
            let mut got = base.clone();
            let backend = apply_filter_active(&filter, &mut got, true).expect("valid parameters");
            assert_eq!(backend, Backend::Gpu, "{label}/{filter:?} fell back to CPU");
            let delta = assert_parity(&got, &want, &format!("{label}/{filter:?}"));
            println!("{label}/{filter:?}: max delta {delta} LSB");
        }
    }
}

#[test]
fn unsupported_filter_falls_back_byte_identical() {
    let filter = unsupported_filter();
    let base = gradient_rgba();
    let want = oracle(&filter, &base);
    let mut got = base.clone();
    let backend = apply_filter_active(&filter, &mut got, true).expect("valid parameters");
    assert_eq!(backend, Backend::Cpu);
    assert_eq!(got.data, want.data, "fallback must be byte-identical");
}

#[test]
fn disabled_gpu_is_byte_identical() {
    let mut filters = accelerated_filters();
    filters.push(unsupported_filter());
    for base in [gradient_rgb(), gradient_rgba()] {
        for filter in &filters {
            let want = oracle(filter, &base);
            let mut got = base.clone();
            let backend = apply_filter_active(filter, &mut got, false).expect("valid parameters");
            assert_eq!(backend, Backend::Cpu, "{filter:?} with gpu disabled");
            assert_eq!(got.data, want.data, "{filter:?} disabled must match oracle");
        }
    }
}
