//! Layer similarity transform: scale, rotation about the layer centre, and
//! translation, resampled with bilinear interpolation.
//!
//! The op is a pure `&mut Document` function alongside the other
//! `resolve_path`-based layer ops. It mutates the document only on success and
//! does **not** recomposite: like `translate_layer_rect`, the caller owns the
//! composite refresh.

use pictura_core::{Channel, Document, LayerMask, LockFlags, PsdRect};

use super::paths::{resolve_path, resolve_path_mut};

use pictura_core::Layer;

/// A similarity transform about the layer's (or mask's) rect centre.
#[derive(Debug, Clone, Copy)]
pub struct LayerTransform {
    pub scale_x: f64,
    pub scale_y: f64,
    pub angle_radians: f64,
    pub dx: f64,
    pub dy: f64,
}

/// Below this absolute scale factor a transform is refused as degenerate.
const SCALE_EPSILON: f64 = 1e-6;
/// Quarter-turn slack: a 90° `cos` is ~6e-17, not exactly 0, so an exact integer
/// corner must not round out to the next pixel. Same intent as the slack in
/// `pictura_ops::rotate_arbitrary`.
const BBOX_SLACK: f64 = 1e-9;

/// The document-space similarity map of one source rect.
struct Map {
    cx: f64,
    cy: f64,
    sx: f64,
    sy: f64,
    cos: f64,
    sin: f64,
    dx: f64,
    dy: f64,
}

impl Map {
    fn new(rect: PsdRect, t: LayerTransform, cos: f64, sin: f64) -> Self {
        Map {
            cx: (rect.left as f64 + rect.right as f64) / 2.0,
            cy: (rect.top as f64 + rect.bottom as f64) / 2.0,
            sx: t.scale_x,
            sy: t.scale_y,
            cos,
            sin,
            dx: t.dx,
            dy: t.dy,
        }
    }

    /// `p' = c + R(θ)·(S·(p − c)) + (dx, dy)`; positive θ is clockwise in the
    /// y-down screen convention.
    fn forward(&self, x: f64, y: f64) -> (f64, f64) {
        let ux = (x - self.cx) * self.sx;
        let uy = (y - self.cy) * self.sy;
        (
            self.cx + self.cos * ux - self.sin * uy + self.dx,
            self.cy + self.sin * ux + self.cos * uy + self.dy,
        )
    }

    /// `src = c + S⁻¹·R(−θ)·(q − c) − (dx, dy)`.
    fn inverse(&self, qx: f64, qy: f64) -> (f64, f64) {
        let vx = qx - self.cx - self.dx;
        let vy = qy - self.cy - self.dy;
        let rx = self.cos * vx + self.sin * vy;
        let ry = -self.sin * vx + self.cos * vy;
        (self.cx + rx / self.sx, self.cy + ry / self.sy)
    }
}

/// Largest result area the op will allocate for one plane, in pixels. A
/// similarity transform of a real layer is orders of magnitude smaller; this
/// only stops a pathological scale from attempting a multi-gigabyte allocation.
// ponytail: fixed 1 Gpx ceiling; thread the codec's allocation budget through
// here if transforms of near-PSB-limit layers ever matter.
const MAX_RESULT_PIXELS: u64 = 1 << 30;

/// Integer bounding box of the four transformed corners, or `None` when the
/// result is empty, out of `i32` range, or implausibly large.
fn bounding_box(map: &Map, rect: PsdRect) -> Option<(i32, i32, i32, i32)> {
    let corners = [
        (rect.left as f64, rect.top as f64),
        (rect.right as f64, rect.top as f64),
        (rect.right as f64, rect.bottom as f64),
        (rect.left as f64, rect.bottom as f64),
    ];
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for (x, y) in corners {
        let (px, py) = map.forward(x, y);
        min_x = min_x.min(px);
        min_y = min_y.min(py);
        max_x = max_x.max(px);
        max_y = max_y.max(py);
    }
    let left = (min_x + BBOX_SLACK).floor();
    let top = (min_y + BBOX_SLACK).floor();
    let right = (max_x - BBOX_SLACK).ceil();
    let bottom = (max_y - BBOX_SLACK).ceil();
    if !left.is_finite() || !top.is_finite() || !right.is_finite() || !bottom.is_finite() {
        return None;
    }
    if left < i32::MIN as f64
        || right > i32::MAX as f64
        || top < i32::MIN as f64
        || bottom > i32::MAX as f64
    {
        return None;
    }
    let (left, top, right, bottom) = (left as i32, top as i32, right as i32, bottom as i32);
    let width = (right as i64) - (left as i64);
    let height = (bottom as i64) - (top as i64);
    if width <= 0 || height <= 0 || (width as u64) * (height as u64) > MAX_RESULT_PIXELS {
        return None;
    }
    Some((left, top, right, bottom))
}

/// Inverse-mapped bilinear sample of a `w×h` plane at source-local `(lx, ly)`.
///
/// The caller has already checked `0 <= lx < w` and `0 <= ly < h`; taps are
/// edge-clamped.
fn bilinear(src: &[u8], w: usize, h: usize, lx: f64, ly: f64) -> u8 {
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
    let p00 = src[y0 * w + x0] as f64;
    let p10 = src[y0 * w + x1] as f64;
    let p01 = src[y1 * w + x0] as f64;
    let p11 = src[y1 * w + x1] as f64;
    let top = p00 + (p10 - p00) * fx;
    let bot = p01 + (p11 - p01) * fx;
    (top + (bot - top) * fy).round().clamp(0.0, 255.0) as u8
}

/// Resample one `w×h` plane into the destination rect; out-of-source → 0.
fn resample_plane(
    src: &[u8],
    w: usize,
    h: usize,
    map: &Map,
    rect: PsdRect,
    dest: (i32, i32, i32, i32),
) -> Vec<u8> {
    let (dl, dt, dr, db) = dest;
    let dw = (dr - dl) as usize;
    let dh = (db - dt) as usize;
    let mut out = vec![0u8; dw * dh];
    let wf = w as f64;
    let hf = h as f64;
    for oy in 0..dh {
        for ox in 0..dw {
            let qx = dl as f64 + ox as f64 + 0.5;
            let qy = dt as f64 + oy as f64 + 0.5;
            let (sx, sy) = map.inverse(qx, qy);
            let lx = sx - rect.left as f64;
            let ly = sy - rect.top as f64;
            if lx < 0.0 || ly < 0.0 || lx >= wf || ly >= hf {
                continue;
            }
            out[oy * dw + ox] = bilinear(src, w, h, lx, ly);
        }
    }
    out
}

/// Whether `t`'s parameters are finite and non-degenerate.
fn params_ok(t: &LayerTransform) -> bool {
    t.scale_x.is_finite()
        && t.scale_y.is_finite()
        && t.angle_radians.is_finite()
        && t.dx.is_finite()
        && t.dy.is_finite()
        && t.scale_x.abs() >= SCALE_EPSILON
        && t.scale_y.abs() >= SCALE_EPSILON
}

/// Whether `layer` carries a materializable channel-less embedded smart object.
fn materialized_channels(doc: &Document, layer: &Layer) -> Option<Vec<Channel>> {
    let so = layer.smart_object.as_ref()?;
    let buf = crate::render_smart_source(so, layer.rect, layer.rect)?;
    if buf.width != layer.rect.width() as u32 || buf.height != layer.rect.height() as u32 {
        return None;
    }
    let plane = buf.width as usize * buf.height as usize;
    if buf.channels < 4 || buf.data.len() != plane * 4 {
        return None;
    }
    let mut channels: Vec<Channel> = (0..doc.mode.color_channels() as usize)
        .map(|c| Channel {
            id: c as i16,
            data: buf.data[c * plane..(c + 1) * plane].to_vec(),
        })
        .collect();
    channels.push(Channel {
        id: -1,
        data: buf.data[3 * plane..4 * plane].to_vec(),
    });
    Some(channels)
}

/// Resample `layer.mask` by the same transform about the mask's own rect. A mask
/// without plane data still has its `rect` transformed (and keeps `data` `None`)
/// so the rect follows the layer.
fn transform_mask(mask: &LayerMask, t: LayerTransform, cos: f64, sin: f64) -> Option<LayerMask> {
    let rect = mask.rect;
    let map = Map::new(rect, t, cos, sin);
    let dest = bounding_box(&map, rect)?;
    let data = match mask.data.as_ref() {
        Some(data) => {
            let w = rect.width();
            let h = rect.height();
            if w <= 0 || h <= 0 || data.len() != (w as usize) * (h as usize) {
                return None;
            }
            Some(resample_plane(
                data, w as usize, h as usize, &map, rect, dest,
            ))
        }
        None => None,
    };
    Some(LayerMask {
        rect: PsdRect {
            top: dest.1,
            left: dest.0,
            bottom: dest.3,
            right: dest.2,
        },
        data,
        ..mask.clone()
    })
}

/// Apply a similarity transform to the layer at `path`.
///
/// On success every channel plane (and the mask, when present) is resampled into
/// the transformed bounding rect and `layer.rect` is updated. The op refuses —
/// leaving `doc` bit-identical — for a missing path, a group, an adjustment
/// layer, a Background layer, a position-locked layer, a zero-area source, any
/// non-finite parameter, a scale below [`SCALE_EPSILON`], a zero-area result, or
/// a channel-less target that does not materialize from its embedded source.
///
/// A channel-less embedded smart object is materialized from its payload and, on
/// success, consumed: the smart object, its preserved `SoLd`/`plLd` blocks, and
/// its linked record are dropped, so no stale untransformed source survives.
///
/// # Ceiling
///
/// Bilinear resampling only: no bicubic/nearest choice and no perspective/skew.
// ponytail: bilinear-only; add an interpolation parameter and bicubic kernel if
// a numeric options bar ever needs it.
pub fn transform_layer(doc: &mut Document, path: &str, transform: LayerTransform) -> bool {
    if !params_ok(&transform) {
        return false;
    }
    let Some(layer) = resolve_path(doc, path) else {
        return false;
    };
    if layer.is_group
        || layer.adjustment.is_some()
        || layer.background
        || layer.lock.contains(LockFlags::POSITION)
    {
        return false;
    }
    let rect = layer.rect;
    let w = rect.width();
    let h = rect.height();
    if w <= 0 || h <= 0 {
        return false;
    }
    let plane = (w as usize) * (h as usize);

    // A channel-bearing layer (e.g. a placed image) keeps its smart_object
    // payload and preserved blocks: the original source survives as the proxy's
    // cache and Replace re-renders from a new source. Only a channel-less target
    // is consumed below.
    let materialize = !layer.channels.iter().any(|c| c.id == 0);
    let source_channels: Vec<Channel> = if materialize {
        match materialized_channels(doc, layer) {
            Some(channels) => channels,
            None => return false,
        }
    } else {
        if layer.channels.iter().any(|c| c.data.len() != plane) {
            return false;
        }
        layer.channels.clone()
    };
    let old_uuid = layer
        .smart_object
        .as_ref()
        .map(|so| so.uuid.clone())
        .unwrap_or_default();

    let (sin, cos) = transform.angle_radians.sin_cos();
    let map = Map::new(rect, transform, cos, sin);
    let Some(dest) = bounding_box(&map, rect) else {
        return false;
    };
    let new_channels: Vec<Channel> = source_channels
        .iter()
        .map(|c| Channel {
            id: c.id,
            data: resample_plane(&c.data, w as usize, h as usize, &map, rect, dest),
        })
        .collect();
    let new_mask = layer
        .mask
        .as_ref()
        .and_then(|m| transform_mask(m, transform, cos, sin));

    let layer = resolve_path_mut(doc, path).expect("resolved above");
    layer.channels = new_channels;
    layer.rect = PsdRect {
        top: dest.1,
        left: dest.0,
        bottom: dest.3,
        right: dest.2,
    };
    if let Some(mask) = new_mask {
        layer.mask = Some(mask);
    }
    if materialize {
        layer
            .extra_blocks
            .retain(|block| !matches!(&block.key, b"SoLd" | b"SoLE" | b"plLd" | b"PlLd"));
        layer.smart_object = None;
        if !old_uuid.is_empty() {
            if let Some(cleaned) =
                pictura_codec::remove_linked_source(&doc.layer_section_extra, &old_uuid)
            {
                doc.layer_section_extra = cleaned;
            }
        }
    }
    true
}
