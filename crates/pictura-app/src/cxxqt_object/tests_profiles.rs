use super::helpers::rgba_from_argb;
use super::helpers_composite::buffer_to_image;
use super::tests::pixel_layer;
use pictura_core::{BitDepth, ColorMode, Document, PixelBuffer, PsdRect};
use pictura_render::{Planes, ViewPyramid};

/// Viewport the canvas presents into (a typical desktop canvas).
const VIEW: (u32, u32) = (1280, 800);

fn ms(label: &str, d: std::time::Duration) {
    println!("{label}: {:.2} ms", d.as_secs_f64() * 1000.0);
}

/// The C++ `ImageView::presentLevelForZoom` formula, so the profile crops the
/// level the canvas would. ponytail: mirrored, not shared across the FFI.
fn present_level(zoom: f64, level_count: usize) -> usize {
    let mut level = 0;
    while level + 1 < level_count && 1.0 / (1u64 << (level + 1)) as f64 >= zoom {
        level += 1;
    }
    level
}

/// A naive nearest-neighbour rescale of the full-resolution composite to a
/// viewport-sized buffer: the old present path that read every source row.
fn naive_rescale(src: &Planes<'_>, w: u32, h: u32) -> Vec<u8> {
    let mut out = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        let sy = (y as u64 * src.height as u64 / h as u64) as usize;
        for x in 0..w {
            let sx = (x as u64 * src.width as u64 / w as u64) as usize;
            let i = sy * src.width as usize + sx;
            let o = ((y * w + x) * 4) as usize;
            out[o] = src.r[i];
            out[o + 1] = src.g[i];
            out[o + 2] = src.b[i];
            out[o + 3] = src.a[i];
        }
    }
    out
}

fn scroll_zoom_pan_profile(n: u32) {
    let plane = n as usize * n as usize;
    let mut data = vec![0u8; plane * 4];
    // Sparse touch so the pages are resident and the premultiply/average is not
    // a no-op, without an O(n²) fill. ponytail: sparse fill, real pixels if the
    // timing ever needs texture.
    for i in (0..plane).step_by(4096) {
        data[i] = (i % 251) as u8;
        data[3 * plane + i] = 255;
    }
    let mut doc = Document::new(n, n, ColorMode::Rgb, BitDepth::Eight);
    doc.composite = PixelBuffer {
        width: n,
        height: n,
        channels: 4,
        data: data.into(),
    };
    let planes = Planes {
        width: n,
        height: n,
        r: &doc.composite.data[..plane],
        g: &doc.composite.data[plane..2 * plane],
        b: &doc.composite.data[2 * plane..3 * plane],
        a: &doc.composite.data[3 * plane..4 * plane],
    };

    let t = std::time::Instant::now();
    let pyramid = ViewPyramid::rebuild(planes);
    ms(&format!("scroll_zoom_pan_profile_{n} rebuild"), t.elapsed());

    let zoom = f64::min(VIEW.0 as f64 / n as f64, VIEW.1 as f64 / n as f64);
    let level = present_level(zoom, pyramid.level_count());
    let (lw, lh) = pyramid.level_size(level);
    let rect = PsdRect {
        top: 0,
        left: 0,
        bottom: lh as i32,
        right: lw as i32,
    };
    let t = std::time::Instant::now();
    let crop = pyramid.crop(planes, level, rect);
    ms(
        &format!(
            "scroll_zoom_pan_profile_{n} level_crop {}x{}",
            crop.width(),
            crop.height()
        ),
        t.elapsed(),
    );

    let t = std::time::Instant::now();
    let _rescaled = naive_rescale(&planes, VIEW.0, VIEW.1);
    ms(
        &format!("scroll_zoom_pan_profile_{n} full_rescale"),
        t.elapsed(),
    );
}

/// Print-only evidence that a viewport-sized pyramid-level crop is cheaper than
/// rescaling the full-resolution composite. No pass/fail budget (the reference
/// machine is not pinned).
#[test]
#[ignore = "4000x4000 crop-vs-rescale profile; run explicitly with --ignored --nocapture"]
fn scroll_zoom_pan_profile_4000() {
    scroll_zoom_pan_profile(4000);
}

#[test]
#[ignore = "16000x16000 crop-vs-rescale profile; run explicitly with --ignored --nocapture"]
fn scroll_zoom_pan_profile_16000() {
    scroll_zoom_pan_profile(16000);
}

/// Print-only split of the per-stroke cost into stroke start, rasterization and
/// region present, so the brush-latency work is ordered by measurement. The
/// existing `paint_dab_profile_4000` times only the region composite; this adds
/// the two unmeasured components. No pass/fail budget (the reference machine is
/// not pinned).
#[test]
#[ignore = "4000x4000 stroke split profile; run explicitly with --ignored --nocapture"]
fn stroke_split_profile_4000() {
    use pictura_paint::{spacing::SpacingMode, Stroke, StrokeConfig, StrokeSample};

    for layers in [1usize, 8] {
        let mut doc = Document::new(4000, 4000, ColorMode::Rgb, BitDepth::Eight);
        doc.layers = (0..layers)
            .map(|i| pixel_layer(&format!("L{i}"), 4000, 4000, (30, 60, 90)))
            .collect();
        // Prime the adapter, allocator and pipelines once, untimed.
        let _ = pictura_render::composite_active(&doc, true);
        let gpu = pictura_render::gpu_available();

        // `Document::clone` is a refcount bump since the planes became
        // copy-on-write: this is what a stroke start and a history state pay.
        let t = std::time::Instant::now();
        let cloned = doc.clone();
        let clone = t.elapsed();
        drop(cloned);

        for diameter in [64u32, 500] {
            let cfg = StrokeConfig {
                color: rgba_from_argb(0xFFFF0000),
                diameter,
                hardness: 100,
                spacing: SpacingMode::Fixed(25),
                ..StrokeConfig::default()
            };

            // Stroke start: two document clones (refcount bumps) plus the coverage buffers.
            let t = std::time::Instant::now();
            let mut stroke = Stroke::begin_at(&doc, "0", cfg).expect("stroke begins");
            let begin = t.elapsed();

            // Rasterization: the first sample places exactly one dab, unrefreshed.
            let t = std::time::Instant::now();
            let changed = stroke.sample(StrokeSample {
                x: 2000.0,
                y: 2000.0,
                pressure: 1.0,
            });
            let raster = t.elapsed();
            let rect = stroke.take_dirty().expect("a dab dirties a region");

            // Steady state: the second dab is a non-overlapping one on a stroke
            // whose painted planes have already forked off the document, so only
            // rasterization remains (the first dab also pays that one-time fork).
            let t = std::time::Instant::now();
            let steady_changed = stroke.sample(StrokeSample {
                x: 2600.0,
                y: 2000.0,
                pressure: 1.0,
            });
            let steady = t.elapsed();
            let steady_rect = stroke.take_dirty();

            // Present: the region composite and display conversion `refresh_region` runs.
            let t = std::time::Instant::now();
            let (buffer, _) = pictura_render::composite_region_active(stroke.document(), rect, gpu);
            let composite = t.elapsed();
            let t = std::time::Instant::now();
            let _ = buffer_to_image(&buffer);
            let display = t.elapsed();

            println!(
                "stroke_split_profile layers={layers} diameter={diameter} clone={:.3}ms \
                 begin={:.2}ms raster={:.2}ms steady={:.2}ms composite={:.2}ms \
                 display={:.2}ms present={:.2}ms region={}x{} changed={changed} \
                 steady_changed={steady_changed} steady_rect={steady_rect:?}",
                clone.as_secs_f64() * 1000.0,
                begin.as_secs_f64() * 1000.0,
                raster.as_secs_f64() * 1000.0,
                steady.as_secs_f64() * 1000.0,
                composite.as_secs_f64() * 1000.0,
                display.as_secs_f64() * 1000.0,
                (composite + display).as_secs_f64() * 1000.0,
                rect.width(),
                rect.height(),
            );
            drop(stroke);
        }
    }
}
