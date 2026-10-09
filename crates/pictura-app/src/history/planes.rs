//! The tracked planes of a document, their tile deltas, and the private copy
//! the history keeps of the state at its cursor.

use pictura_core::{fresh_stamp, Document, Layer, Plane, PsdRect};
use rayon::prelude::*;

/// Tile edge for the region deltas: a changed plane keeps only the 64×64 tiles
/// whose bytes differ, so a small edit costs a small delta rather than a whole
/// 61 MiB plane.
pub(super) const TILE: usize = 64;

pub(super) enum Dir {
    Before,
    After,
}

/// One changed tile of one plane, laid out row-major with the plane's stride.
pub(super) struct TileDelta {
    pub(super) x: usize,
    pub(super) y: usize,
    pub(super) w: usize,
    pub(super) h: usize,
    pub(super) before: Vec<u8>,
    pub(super) after: Vec<u8>,
}

/// The changed tiles of one tracked plane, anchored at the previous state.
pub(super) struct PlaneDelta {
    pub(super) index: usize,
    pub(super) width: usize,
    pub(super) tiles: Vec<TileDelta>,
}

/// What a tracked plane is, which decides what its change does to the picture.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum PlaneKind {
    /// The document composite: its tiles are the picture's change.
    Composite,
    /// A layer's color or alpha channel: shown only through the composite.
    LayerChannel,
    /// A document channel or a layer mask, which a channel or mask view shows
    /// directly.
    Other,
}

/// The kind of every tracked plane, in traversal order.
pub(super) fn plane_kinds(doc: &Document) -> Vec<PlaneKind> {
    fn visit(layer: &Layer, out: &mut Vec<PlaneKind>) {
        out.extend(layer.channels.iter().map(|_| PlaneKind::LayerChannel));
        if layer.mask.as_ref().is_some_and(|m| m.data.is_some()) {
            out.push(PlaneKind::Other);
        }
        for child in &layer.children {
            visit(child, out);
        }
    }
    let mut out = vec![PlaneKind::Composite];
    out.extend(doc.channels.iter().map(|_| PlaneKind::Other));
    for l in &doc.layers {
        visit(l, &mut out);
    }
    out
}

/// The changed tiles from `prev` to `next`, or `None` when the tracked-plane
/// geometry (count, stride, or height) differs and a full state is needed.
///
/// A pair with the same nonzero stamp, or one allocation, is equal and skipped
/// without a look at its bytes; every other pair is compared tile by tile on
/// every core.
pub(super) fn diff(prev: &Document, next: &Document) -> Option<Vec<PlaneDelta>> {
    let prev_planes = planes(prev);
    let next_planes = planes(next);
    if prev_planes.len() != next_planes.len()
        || prev_planes
            .iter()
            .zip(&next_planes)
            .any(|(a, b)| a.0.len() != b.0.len() || a.1 != b.1 || a.2 != b.2)
    {
        return None;
    }
    let mut out = Vec::new();
    for (index, ((before, width, _), (after, _, _))) in
        prev_planes.iter().zip(&next_planes).enumerate()
    {
        if known_equal(before, after) {
            continue;
        }
        let tiles = diff_tiles(before.as_slice(), after.as_slice(), *width);
        if !tiles.is_empty() {
            out.push(PlaneDelta {
                index,
                width: *width,
                tiles,
            });
        }
    }
    Some(out)
}

fn known_equal(a: &Plane<u8>, b: &Plane<u8>) -> bool {
    (a.stamp() != 0 && a.stamp() == b.stamp()) || (a.len() == b.len() && a.shares(b))
}

fn diff_tiles(before: &[u8], after: &[u8], width: usize) -> Vec<TileDelta> {
    if width == 0 {
        return Vec::new();
    }
    let height = before.len() / width;
    let columns = width.div_ceil(TILE);
    let bands: Vec<Vec<TileDelta>> = (0..height.div_ceil(TILE))
        .into_par_iter()
        .map(|band| {
            let y0 = band * TILE;
            let h = TILE.min(height - y0);
            let mut changed = vec![false; columns];
            for row in y0..y0 + h {
                let start = row * width;
                let (b, a) = (&before[start..start + width], &after[start..start + width]);
                if b == a {
                    continue;
                }
                for (c, flag) in changed.iter_mut().enumerate() {
                    let x0 = c * TILE;
                    let x1 = (x0 + TILE).min(width);
                    *flag = *flag || b[x0..x1] != a[x0..x1];
                }
            }
            let mut tiles = Vec::new();
            for (c, _) in changed.iter().enumerate().filter(|(_, f)| **f) {
                let x0 = c * TILE;
                let w = TILE.min(width - x0);
                let mut bb = Vec::with_capacity(w * h);
                let mut ab = Vec::with_capacity(w * h);
                for row in y0..y0 + h {
                    let start = row * width + x0;
                    bb.extend_from_slice(&before[start..start + w]);
                    ab.extend_from_slice(&after[start..start + w]);
                }
                tiles.push(TileDelta {
                    x: x0,
                    y: y0,
                    w,
                    h,
                    before: bb,
                    after: ab,
                });
            }
            tiles
        })
        .collect();
    bands.into_iter().flatten().collect()
}

/// Write `deltas` into `doc` in the given direction.
pub(super) fn apply_tiles(doc: &mut Document, deltas: &[PlaneDelta], dir: &Dir) {
    for delta in deltas {
        let mut index = 0;
        each_plane_mut(doc, &mut |plane, _, _| {
            if index == delta.index {
                for tile in &delta.tiles {
                    let bytes = match dir {
                        Dir::Before => &tile.before,
                        Dir::After => &tile.after,
                    };
                    write_tile(plane, delta.width, tile, bytes);
                }
            }
            index += 1;
        });
    }
}

fn write_tile(plane: &mut Plane<u8>, width: usize, tile: &TileDelta, bytes: &[u8]) {
    if width == 0 {
        return;
    }
    for row in 0..tile.h {
        let dst = (tile.y + row) * width + tile.x;
        let src = row * tile.w;
        if dst + tile.w > plane.len() {
            return;
        }
        plane[dst..dst + tile.w].copy_from_slice(&bytes[src..src + tile.w]);
    }
}

/// The document rectangle a composite tile covers.
pub(super) fn tile_rect(tile: &TileDelta) -> PsdRect {
    PsdRect {
        top: tile.y as i32,
        left: tile.x as i32,
        bottom: (tile.y + tile.h) as i32,
        right: (tile.x + tile.w) as i32,
    }
}

/// Replace `doc`'s metadata with `meta`'s, keeping `doc`'s tracked-plane bytes.
pub(super) fn adopt_metadata(doc: &mut Document, meta: &Document) {
    let mut planes: Vec<Plane<u8>> = Vec::new();
    each_plane(doc, &mut |p, _, _| planes.push(p.clone()));
    *doc = meta.clone();
    let mut index = 0;
    each_plane_mut(doc, &mut |p, _, _| {
        if let Some(src) = planes.get(index) {
            *p = src.clone();
        }
        index += 1;
    });
}

/// `doc` with every tracked plane cleared: its metadata alone.
pub(super) fn hollow(doc: &Document) -> Document {
    let mut meta = doc.clone();
    each_plane_mut(&mut meta, &mut |p, _, _| *p = Plane::default());
    meta
}

/// A copy of `source` in planes it does not share, each pair stamped equal.
pub(super) fn private_copy(source: &mut Document) -> Document {
    let mut copy = source.clone();
    let mut stamps = Vec::new();
    each_plane_mut(&mut copy, &mut |p, _, _| {
        let stamp = fresh_stamp();
        let mut fresh = copy_plane(p);
        fresh.set_stamp(stamp);
        *p = fresh;
        stamps.push(stamp);
    });
    let mut index = 0;
    each_plane_mut(source, &mut |p, _, _| {
        p.set_stamp(stamps[index]);
        index += 1;
    });
    copy
}

/// A uniquely held copy of `plane`, copied on every core.
pub(crate) fn copy_plane(plane: &Plane<u8>) -> Plane<u8> {
    Plane::build(plane.len(), |dst| copy_par(dst, plane.as_slice()))
}

fn copy_par(dst: &mut [u8], src: &[u8]) {
    const CHUNK: usize = 1 << 20;
    dst.par_chunks_mut(CHUNK)
        .zip(src.par_chunks(CHUNK))
        .for_each(|(d, s)| d.copy_from_slice(s));
}

/// Whether every tracked plane of `live` is known to equal `private`'s, so
/// tiles applied to both keep them equal.
pub(super) fn planes_agree(private: &Document, live: &Document) -> bool {
    let a = planes(private);
    let b = planes(live);
    a.len() == b.len() && a.iter().zip(&b).all(|(x, y)| known_equal(&x.0, &y.0))
}

/// Re-stamp `private` and `live`, whose tracked planes the caller has just made
/// equal byte for byte. A pair that shares one allocation is split by giving
/// `private` its own copy, so the next write to `live` happens in place.
pub(super) fn stamp_agreed(private: &mut Document, live: &mut Document) {
    let theirs = planes(live);
    let mut stamps = Vec::new();
    let mut index = 0;
    each_plane_mut(private, &mut |p, _, _| {
        let other = &theirs[index].0;
        index += 1;
        if p.len() == other.len() && p.shares(other) {
            *p = copy_plane(other);
        } else if p.stamp() != 0 && p.stamp() == other.stamp() {
            stamps.push(p.stamp());
            return;
        }
        let stamp = fresh_stamp();
        p.set_stamp(stamp);
        stamps.push(stamp);
    });
    drop(theirs);
    let mut index = 0;
    each_plane_mut(live, &mut |p, _, _| {
        p.set_stamp(stamps[index]);
        index += 1;
    });
}

fn planes(doc: &Document) -> Vec<(Plane<u8>, usize, usize)> {
    let mut out = Vec::new();
    each_plane(doc, &mut |p, w, h| out.push((p.clone(), w, h)));
    out
}

fn plane_height(len: usize, width: usize) -> usize {
    len.checked_div(width).unwrap_or(0)
}

/// Visit every tracked plane (composite, document channels, layer channels,
/// layer masks, depth-first) with its row stride and height. The traversal is
/// fixed by the document structure, not the plane lengths, so it enumerates the
/// same indices before and after a plane is cleared or written.
pub(super) fn each_plane(doc: &Document, f: &mut impl FnMut(&Plane<u8>, usize, usize)) {
    let width = doc.width as usize;
    let h = plane_height(doc.composite.data.len(), width);
    f(&doc.composite.data, width, h);
    for channel in &doc.channels {
        let h = plane_height(channel.data.len(), width);
        f(&channel.data, width, h);
    }
    for layer in &doc.layers {
        each_layer_plane(layer, f);
    }
}

fn each_layer_plane(layer: &Layer, f: &mut impl FnMut(&Plane<u8>, usize, usize)) {
    let layer_width = layer.rect.width().max(0) as usize;
    for channel in &layer.channels {
        let width = if layer_width > 0 {
            layer_width
        } else {
            channel.data.len()
        };
        let h = plane_height(channel.data.len(), width);
        f(&channel.data, width, h);
    }
    if let Some(mask) = &layer.mask {
        if let Some(data) = &mask.data {
            let mask_width = mask.rect.width().max(0) as usize;
            let width = if mask_width > 0 {
                mask_width
            } else {
                data.len()
            };
            let h = plane_height(data.len(), width);
            f(data, width, h);
        }
    }
    for child in &layer.children {
        each_layer_plane(child, &mut *f);
    }
}

pub(super) fn each_plane_mut(doc: &mut Document, f: &mut impl FnMut(&mut Plane<u8>, usize, usize)) {
    let width = doc.width as usize;
    let h = plane_height(doc.composite.data.len(), width);
    f(&mut doc.composite.data, width, h);
    for channel in doc.channels.iter_mut() {
        let h = plane_height(channel.data.len(), width);
        f(&mut channel.data, width, h);
    }
    for layer in doc.layers.iter_mut() {
        each_layer_plane_mut(layer, f);
    }
}

fn each_layer_plane_mut(layer: &mut Layer, f: &mut impl FnMut(&mut Plane<u8>, usize, usize)) {
    let layer_width = layer.rect.width().max(0) as usize;
    for channel in layer.channels.iter_mut() {
        let width = if layer_width > 0 {
            layer_width
        } else {
            channel.data.len()
        };
        let h = plane_height(channel.data.len(), width);
        f(&mut channel.data, width, h);
    }
    if let Some(mask) = layer.mask.as_mut() {
        if let Some(data) = mask.data.as_mut() {
            let mask_width = mask.rect.width().max(0) as usize;
            let width = if mask_width > 0 {
                mask_width
            } else {
                data.len()
            };
            let h = plane_height(data.len(), width);
            f(data, width, h);
        }
    }
    for child in layer.children.iter_mut() {
        each_layer_plane_mut(child, &mut *f);
    }
}
