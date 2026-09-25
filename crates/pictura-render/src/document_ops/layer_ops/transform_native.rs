//! Native-depth resampling for the transform family. Dispatches on [`Samples`]
//! so a scaled, rotated, projectively mapped, or warped layer keeps its retained
//! store; the `u8` path delegates to the byte-identical 8-bit kernels.
//!
//! Split from `transform.rs` for the file-size cap.

use pictura_core::{PsdRect, Sample, Samples, SourceChannels};

use super::transform::{bilinear, resample_plane, PlaneMap};
use super::warp::{warp_plane, WarpMesh};

/// The `(left, top, right, bottom)` tuple of a rect.
pub(super) fn dest_tuple(rect: PsdRect) -> (i32, i32, i32, i32) {
    (rect.left, rect.top, rect.right, rect.bottom)
}

/// The rect of a destination tuple.
fn rect_of(dest: (i32, i32, i32, i32)) -> PsdRect {
    PsdRect {
        top: dest.1,
        left: dest.0,
        bottom: dest.3,
        right: dest.2,
    }
}

/// Resample one native plane through `map` into `dest`.
pub(super) fn resample_native<M: PlaneMap>(
    samples: &Samples,
    w: usize,
    h: usize,
    map: &M,
    rect: PsdRect,
    dest: (i32, i32, i32, i32),
) -> Samples {
    match samples {
        Samples::U8(v) => Samples::U8(resample_plane(v, w, h, map, rect, dest)),
        Samples::U16(v) => Samples::U16(resample_typed(v, w, h, map, rect, dest)),
        Samples::F32(v) => Samples::F32(resample_typed(v, w, h, map, rect, dest)),
    }
}

fn resample_typed<M: PlaneMap, T: Sample>(
    src: &[T],
    w: usize,
    h: usize,
    map: &M,
    rect: PsdRect,
    dest: (i32, i32, i32, i32),
) -> Vec<T> {
    let (dl, dt, dr, db) = dest;
    let dw = (dr - dl) as usize;
    let dh = (db - dt) as usize;
    let mut out = vec![T::from_unit(0.0); dw * dh];
    let wf = w as f64;
    let hf = h as f64;
    for oy in 0..dh {
        for ox in 0..dw {
            let qx = dl as f64 + ox as f64 + 0.5;
            let qy = dt as f64 + oy as f64 + 0.5;
            let Some((sx, sy)) = map.inverse(qx, qy) else {
                continue;
            };
            let lx = sx - rect.left as f64;
            let ly = sy - rect.top as f64;
            if lx < 0.0 || ly < 0.0 || lx >= wf || ly >= hf {
                continue;
            }
            out[oy * dw + ox] = bilinear_sample(src, w, h, lx, ly);
        }
    }
    out
}

/// Inverse-mapped bilinear sample of a native plane; the unit-domain analogue
/// of `transform::bilinear`.
pub(super) fn bilinear_sample<T: Sample>(src: &[T], w: usize, h: usize, lx: f64, ly: f64) -> T {
    let u = lx - 0.5;
    let v = ly - 0.5;
    let x0f = u.floor();
    let y0f = v.floor();
    let fx = u - x0f;
    let fy = v - y0f;
    let max_x = (w - 1) as f64;
    let max_y = (h - 1) as f64;
    let x0 = x0f.clamp(0.0, max_x) as usize;
    let y0 = y0f.clamp(0.0, max_y) as usize;
    let x1 = (x0f + 1.0).clamp(0.0, max_x) as usize;
    let y1 = (y0f + 1.0).clamp(0.0, max_y) as usize;
    let p00 = src[y0 * w + x0].to_unit();
    let p10 = src[y0 * w + x1].to_unit();
    let p01 = src[y1 * w + x0].to_unit();
    let p11 = src[y1 * w + x1].to_unit();
    let top = p00 + (p10 - p00) * fx;
    let bot = p01 + (p11 - p01) * fx;
    T::from_unit(top + (bot - top) * fy)
}

/// Resample a layer's retained store through `map`; the `-2` mask plane uses the
/// mask's own map and destination (`mask` is its old rect and new dest).
pub(super) fn resample_store<M: PlaneMap>(
    store: &SourceChannels,
    layer_w: usize,
    layer_h: usize,
    map: &M,
    layer_rect: PsdRect,
    dest: (i32, i32, i32, i32),
    mask: Option<(PsdRect, (i32, i32, i32, i32))>,
) -> SourceChannels {
    let planes = store
        .planes
        .iter()
        .map(|(id, samples)| {
            let resampled = if *id == -2 {
                match mask {
                    Some((mrect, mdest)) => {
                        let mmap = map.for_rect(mrect);
                        let mw = mrect.width().max(0) as usize;
                        let mh = mrect.height().max(0) as usize;
                        resample_native(samples, mw, mh, &mmap, mrect, mdest)
                    }
                    None => samples.clone(),
                }
            } else {
                resample_native(samples, layer_w, layer_h, map, layer_rect, dest)
            };
            (*id, resampled)
        })
        .collect();
    SourceChannels::new(store.depth, rect_of(dest), planes)
}

/// Warp a layer's retained store through the mesh; the `-2` mask plane uses the
/// mask-relative surface mapping and its own destination.
pub(super) fn warp_store(
    store: &SourceChannels,
    layer_w: usize,
    layer_h: usize,
    mesh: &WarpMesh,
    layer_rect: PsdRect,
    dest: (i32, i32, i32, i32),
    mask: Option<(PsdRect, (i32, i32, i32, i32))>,
) -> SourceChannels {
    let planes = store
        .planes
        .iter()
        .map(|(id, samples)| {
            let warped = if *id == -2 {
                match mask {
                    Some((mrect, mdest)) => {
                        let lw = layer_rect.width() as f64;
                        let lh = layer_rect.height() as f64;
                        let ox = layer_rect.left as f64 - mrect.left as f64;
                        let oy = layer_rect.top as f64 - mrect.top as f64;
                        let mw = mrect.width().max(0) as usize;
                        let mh = mrect.height().max(0) as usize;
                        warp_native(
                            samples,
                            mw,
                            mh,
                            mesh,
                            move |u, v| (u * lw + ox, v * lh + oy),
                            mdest,
                        )
                    }
                    None => samples.clone(),
                }
            } else {
                warp_native(
                    samples,
                    layer_w,
                    layer_h,
                    mesh,
                    |u, v| (u * layer_w as f64, v * layer_h as f64),
                    dest,
                )
            };
            (*id, warped)
        })
        .collect();
    SourceChannels::new(store.depth, rect_of(dest), planes)
}

fn warp_native(
    samples: &Samples,
    sw: usize,
    sh: usize,
    mesh: &WarpMesh,
    to_src: impl Fn(f64, f64) -> (f64, f64),
    dest: (i32, i32, i32, i32),
) -> Samples {
    match samples {
        Samples::U8(v) => Samples::U8(warp_plane(v, sw, sh, mesh, to_src, dest, bilinear)),
        Samples::U16(v) => Samples::U16(warp_plane(v, sw, sh, mesh, to_src, dest, bilinear_sample)),
        Samples::F32(v) => Samples::F32(warp_plane(v, sw, sh, mesh, to_src, dest, bilinear_sample)),
    }
}
