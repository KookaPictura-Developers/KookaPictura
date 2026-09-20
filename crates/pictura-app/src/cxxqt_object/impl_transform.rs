use super::helpers::*;
use super::helpers_composite::*;
use super::qobject;
use super::state::TransformSession;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QImage, QString};
use pictura_core::{layer_move_locked, LockFlags, PsdRect};
use pictura_render::LayerTransform;

/// Screen-pixel tolerance for a scale-handle hit.
const HANDLE_TOLERANCE: f64 = 6.0;
/// Screen-pixel band outside a corner that starts a rotation.
const ROTATE_BAND: f64 = 20.0;
/// Handle indices above the eight scale handles.
const ROTATE_HANDLE: i32 = 8;
const MOVE_HANDLE: i32 = 9;

/// Centre of `rect` as a document-space point.
fn rect_center(rect: PsdRect) -> (f64, f64) {
    (
        (rect.left as f64 + rect.right as f64) / 2.0,
        (rect.top as f64 + rect.bottom as f64) / 2.0,
    )
}

/// Source-rect corners in order top-left, top-right, bottom-right, bottom-left.
fn source_corners(rect: PsdRect) -> [(f64, f64); 4] {
    [
        (rect.left as f64, rect.top as f64),
        (rect.right as f64, rect.top as f64),
        (rect.right as f64, rect.bottom as f64),
        (rect.left as f64, rect.bottom as f64),
    ]
}

/// `p' = c + R(θ)·(S·(p − c)) + (dx, dy)` for one point.
fn forward_point(
    rect: PsdRect,
    sx: f64,
    sy: f64,
    angle: f64,
    dx: f64,
    dy: f64,
    p: (f64, f64),
) -> (f64, f64) {
    let (c_x, c_y) = rect_center(rect);
    let (sin, cos) = angle.sin_cos();
    let ux = (p.0 - c_x) * sx;
    let uy = (p.1 - c_y) * sy;
    (
        c_x + cos * ux - sin * uy + dx,
        c_y + sin * ux + cos * uy + dy,
    )
}

/// The transformed quad corners in [`source_corners`] order.
fn transform_quad_points(
    rect: PsdRect,
    sx: f64,
    sy: f64,
    angle: f64,
    dx: f64,
    dy: f64,
) -> [(f64, f64); 4] {
    let mut out = source_corners(rect);
    for corner in &mut out {
        *corner = forward_point(rect, sx, sy, angle, dx, dy, *corner);
    }
    out
}

/// Source coordinate of handle `h` (4..=7 are edge midpoints).
fn handle_source_point(rect: PsdRect, h: i32) -> (f64, f64) {
    let (left, top) = (rect.left as f64, rect.top as f64);
    let (right, bottom) = (rect.right as f64, rect.bottom as f64);
    let (mx, my) = ((left + right) / 2.0, (top + bottom) / 2.0);
    match h {
        0 => (left, top),
        1 => (right, top),
        2 => (right, bottom),
        3 => (left, bottom),
        4 => (mx, top),
        5 => (right, my),
        6 => (mx, bottom),
        _ => (left, my),
    }
}

/// The anchor handle opposite `h` (the pivot a scale keeps fixed).
fn opposite_handle(h: i32) -> i32 {
    match h {
        0 => 2,
        1 => 3,
        2 => 0,
        3 => 1,
        4 => 6,
        5 => 7,
        6 => 4,
        _ => 5,
    }
}

/// The eight handle positions of the current quad.
fn handle_points(quad: &[(f64, f64); 4]) -> [(f64, f64); 8] {
    [
        quad[0],
        quad[1],
        quad[2],
        quad[3],
        ((quad[0].0 + quad[1].0) / 2.0, (quad[0].1 + quad[1].1) / 2.0),
        ((quad[1].0 + quad[2].0) / 2.0, (quad[1].1 + quad[2].1) / 2.0),
        ((quad[2].0 + quad[3].0) / 2.0, (quad[2].1 + quad[3].1) / 2.0),
        ((quad[3].0 + quad[0].0) / 2.0, (quad[3].1 + quad[0].1) / 2.0),
    ]
}

fn dist2(a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.0 - b.0) * (a.0 - b.0) + (a.1 - b.1) * (a.1 - b.1)
}

/// Translate the session by the pointer delta since press.
fn gesture_translate(session: &mut TransformSession, x: f64, y: f64) {
    session.dx = session.start[3] + (x - session.press_x);
    session.dy = session.start[4] + (y - session.press_y);
}

/// Rotate about the session centre; Shift snaps to 15° steps.
fn gesture_rotate(session: &mut TransformSession, x: f64, y: f64, shift: bool) {
    let (c_x, c_y) = rect_center(session.orig_rect);
    let pivot = (c_x + session.start[3], c_y + session.start[4]);
    let a0 = (session.press_y - pivot.1).atan2(session.press_x - pivot.0);
    let a1 = (y - pivot.1).atan2(x - pivot.0);
    let mut angle = session.start[2] + (a1 - a0);
    if shift {
        let step = std::f64::consts::PI / 12.0;
        angle = (angle / step).round() * step;
    }
    session.angle = angle;
}

/// Scale about the opposite handle so it stays fixed; Shift locks the aspect
/// ratio and the transformed rect is clamped to at least 1 px per axis.
fn gesture_scale(session: &mut TransformSession, x: f64, y: f64, shift: bool) -> bool {
    let rect = session.orig_rect;
    let [start_sx, start_sy, start_angle, start_dx, start_dy] = session.start;
    let (c_x, c_y) = rect_center(rect);
    let h = session.handle;
    let drag_src = handle_source_point(rect, h);
    let anchor_src = handle_source_point(rect, opposite_handle(h));
    let anchor_doc = forward_point(
        rect,
        start_sx,
        start_sy,
        start_angle,
        start_dx,
        start_dy,
        anchor_src,
    );
    let (sin, cos) = start_angle.sin_cos();
    let vx = x - anchor_doc.0;
    let vy = y - anchor_doc.1;
    let wx = cos * vx + sin * vy;
    let wy = -sin * vx + cos * vy;
    let ux = drag_src.0 - anchor_src.0;
    let uy = drag_src.1 - anchor_src.1;
    let mut new_sx = if ux.abs() > 1e-12 { wx / ux } else { start_sx };
    let mut new_sy = if uy.abs() > 1e-12 { wy / uy } else { start_sy };
    // Shift locks the aspect ratio on a corner only; an edge handle always
    // scales one axis.
    if shift && h <= 3 {
        let uniform = if new_sx.abs() > new_sy.abs() {
            new_sx
        } else {
            new_sy
        };
        new_sx = uniform;
        new_sy = uniform;
    }
    let min_sx = 1.0 / rect.width() as f64;
    let min_sy = 1.0 / rect.height() as f64;
    if new_sx.abs() < min_sx {
        new_sx = if new_sx < 0.0 { -min_sx } else { min_sx };
    }
    if new_sy.abs() < min_sy {
        new_sy = if new_sy < 0.0 { -min_sy } else { min_sy };
    }
    if !new_sx.is_finite() || !new_sy.is_finite() {
        return false;
    }
    let a00 = cos * new_sx;
    let a01 = -sin * new_sy;
    let a10 = sin * new_sx;
    let a11 = cos * new_sy;
    let anchor_mapped = (
        a00 * anchor_src.0 + a01 * anchor_src.1,
        a10 * anchor_src.0 + a11 * anchor_src.1,
    );
    let t_x = anchor_doc.0 - anchor_mapped.0;
    let t_y = anchor_doc.1 - anchor_mapped.1;
    session.scale_x = new_sx;
    session.scale_y = new_sy;
    session.angle = start_angle;
    session.dx = t_x - c_x + a00 * c_x + a01 * c_y;
    session.dy = t_y - c_y + a10 * c_x + a11 * c_y;
    true
}

/// Convex point-in-quad test using consistent cross-product signs.
fn point_in_quad(p: (f64, f64), quad: &[(f64, f64); 4]) -> bool {
    let mut sign = 0i32;
    for i in 0..4 {
        let a = quad[i];
        let b = quad[(i + 1) % 4];
        let cross = (b.0 - a.0) * (p.1 - a.1) - (b.1 - a.1) * (p.0 - a.0);
        if cross.abs() < 1e-9 {
            continue;
        }
        let s = if cross > 0.0 { 1 } else { -1 };
        if sign == 0 {
            sign = s;
        } else if sign != s {
            return false;
        }
    }
    true
}

/// Which part of the session `(x, y)` hits: 0..=7 handle, 8 rotate, 9 move, -1
/// nothing. Tolerances are in screen pixels and converted through `zoom`.
fn hit_test(
    rect: PsdRect,
    sx: f64,
    sy: f64,
    angle: f64,
    dx: f64,
    dy: f64,
    x: f64,
    y: f64,
    zoom: f64,
) -> i32 {
    if !x.is_finite() || !y.is_finite() {
        return -1;
    }
    let zoom = if zoom.is_finite() && zoom > 1e-9 {
        zoom
    } else {
        1.0
    };
    let quad = transform_quad_points(rect, sx, sy, angle, dx, dy);
    let p = (x, y);
    let tol = HANDLE_TOLERANCE / zoom;
    for (i, h) in handle_points(&quad).iter().enumerate() {
        if dist2(p, *h) <= tol * tol {
            return i as i32;
        }
    }
    if point_in_quad(p, &quad) {
        return MOVE_HANDLE;
    }
    let band = ROTATE_BAND / zoom;
    for corner in &quad {
        if dist2(p, *corner) <= band * band {
            return ROTATE_HANDLE;
        }
    }
    -1
}

/// Build the transform overlay images for `path`: the document composited with
/// the target hidden, the target's own RGBA image (materializing a channel-less
/// embedded object into a clone), its document-space origin, and opacity.
fn build_transform_preview(
    doc: &pictura_core::Document,
    path: &str,
    gpu_compute: bool,
) -> Option<(QImage, QImage, i32, i32, i32)> {
    let layer = pictura_render::resolve_path(doc, path)?;
    let (x, y, opacity) = (layer.rect.left, layer.rect.top, layer.opacity as i32);
    let image = match layer_image(layer) {
        Some(image) => image,
        None => {
            let mut clone = doc.clone();
            if !pictura_render::rasterize_smart_object(&mut clone, path) {
                return None;
            }
            layer_image(pictura_render::resolve_path(&clone, path)?)?
        }
    };
    let mut base_doc = doc.clone();
    if let Some(target) = pictura_render::resolve_path_mut(&mut base_doc, path) {
        target.visible = false;
    }
    let rendered = current_buffer(&base_doc, gpu_compute);
    Some((buffer_to_image(&rendered), image, x, y, opacity))
}

/// Whether `path` resolves to a transformable target: a raster pixel layer, or
/// a channel-less embedded smart object whose source materializes. False for a
/// group, adjustment layer, Background layer, position-locked layer, or a
/// zero-area target.
fn can_free_transform(doc: &pictura_core::Document, path: &str) -> bool {
    let Some(layer) = pictura_render::resolve_path(doc, path) else {
        return false;
    };
    if layer.is_group
        || layer.adjustment.is_some()
        || layer.background
        || layer.lock.contains(LockFlags::POSITION)
        || layer.rect.width() <= 0
        || layer.rect.height() <= 0
    {
        return false;
    }
    if layer.channels.iter().any(|c| c.id == 0) {
        return true;
    }
    let mut clone = doc.clone();
    pictura_render::rasterize_smart_object(&mut clone, path)
}

impl qobject::PictureView {
    /// Whether `path` resolves to a transformable layer. Read-only.
    pub fn layer_can_free_transform(&self, path: &QString) -> bool {
        let path = path.to_string();
        self.rust()
            .doc
            .as_ref()
            .is_some_and(|doc| can_free_transform(doc, &path))
    }

    /// Begin a Free Transform session on the layer at `path`.
    ///
    /// Returns false without changing any session for an untransformable target.
    /// A second begin on the same active path is a no-op; a begin on a different
    /// path first cancels the active session.
    pub fn begin_free_transform(mut self: Pin<&mut Self>, path: &QString) -> bool {
        let path = path.to_string();
        {
            let rust = self.rust();
            if let Some(session) = rust.transform_session.as_ref() {
                if session.path == path {
                    return true;
                }
            }
        }
        let ok = self
            .rust()
            .doc
            .as_ref()
            .is_some_and(|doc| can_free_transform(doc, &path));
        if !ok {
            return false;
        }
        self.as_mut().rust_mut().transform_session = None;
        if !self.as_mut().compute_transform_preview(&path) {
            return false;
        }
        let rect = {
            let rust = self.rust();
            rust.doc
                .as_ref()
                .and_then(|doc| pictura_render::resolve_path(doc, &path))
                .map(|layer| layer.rect)
        };
        let Some(orig_rect) = rect else {
            return false;
        };
        self.as_mut().rust_mut().transform_session = Some(TransformSession {
            path,
            orig_rect,
            scale_x: 1.0,
            scale_y: 1.0,
            angle: 0.0,
            dx: 0.0,
            dy: 0.0,
            handle: -1,
            press_x: 0.0,
            press_y: 0.0,
            start: [1.0, 1.0, 0.0, 0.0, 0.0],
            dragging: false,
        });
        true
    }

    /// Compute and store the transform overlay base/layer for `path`; keyed with
    /// the `-1` sentinel so the Move cache rebuilds when it next runs.
    fn compute_transform_preview(mut self: Pin<&mut Self>, path: &str) -> bool {
        let mut guard = self.as_mut().rust_mut();
        let rust = &mut *guard;
        let gpu_compute = rust.gpu_compute;
        let prepared = rust
            .doc
            .as_ref()
            .and_then(|doc| build_transform_preview(doc, path, gpu_compute));
        let Some((base, image, x, y, opacity)) = prepared else {
            return false;
        };
        rust.move_base = Some(base);
        rust.move_layer = Some(image);
        rust.move_x = x;
        rust.move_y = y;
        rust.move_opacity = opacity;
        rust.move_prepared_revision = rust.content_revision;
        rust.move_prepared_layer = -1;
        true
    }

    /// Clear the session without touching the document.
    pub fn cancel_transform(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().transform_session = None;
    }

    /// Commit the session: one `transform_layer` call, a recomposite, and one
    /// `"Free Transform"` history state on success. An identity transform records
    /// nothing; an engine refusal records nothing. The session always clears.
    pub fn commit_transform(mut self: Pin<&mut Self>) -> bool {
        let Some((path, transform)) = self.rust().transform_session.as_ref().map(|session| {
            (
                session.path.clone(),
                LayerTransform {
                    scale_x: session.scale_x,
                    scale_y: session.scale_y,
                    angle_radians: session.angle,
                    dx: session.dx,
                    dy: session.dy,
                },
            )
        }) else {
            return false;
        };
        let identity = transform.scale_x == 1.0
            && transform.scale_y == 1.0
            && transform.angle_radians == 0.0
            && transform.dx == 0.0
            && transform.dy == 0.0;
        if identity {
            self.as_mut().rust_mut().transform_session = None;
            return false;
        }
        let changed = match self.as_mut().rust_mut().doc.as_mut() {
            Some(doc) => pictura_render::transform_layer(doc, &path, transform),
            None => false,
        };
        self.as_mut().rust_mut().transform_session = None;
        if changed {
            self.as_mut().clear_link_sets();
            self.as_mut().recomposite();
            self.as_mut().record("Free Transform");
        }
        changed
    }

    /// Begin a drag at document-space `(x, y)`; returns the hit handle
    /// (0..=7 scale, 8 rotate, 9 move) or -1 when nothing was hit.
    pub fn transform_press(
        mut self: Pin<&mut Self>,
        x: f64,
        y: f64,
        zoom: f64,
        _shift: bool,
        _alt: bool,
    ) -> i32 {
        let hit = {
            let rust = self.rust();
            match rust.transform_session.as_ref() {
                Some(session) => hit_test(
                    session.orig_rect,
                    session.scale_x,
                    session.scale_y,
                    session.angle,
                    session.dx,
                    session.dy,
                    x,
                    y,
                    zoom,
                ),
                None => return -1,
            }
        };
        if hit < 0 {
            return -1;
        }
        let mut rust = self.as_mut().rust_mut();
        if let Some(session) = rust.transform_session.as_mut() {
            session.handle = hit;
            session.dragging = true;
            session.press_x = x;
            session.press_y = y;
            session.start = [
                session.scale_x,
                session.scale_y,
                session.angle,
                session.dx,
                session.dy,
            ];
        }
        hit
    }

    /// Hover hit-test for cursor selection; no session is mutated.
    pub fn transform_hit_test(&self, x: f64, y: f64, zoom: f64) -> i32 {
        match self.rust().transform_session.as_ref() {
            Some(session) => hit_test(
                session.orig_rect,
                session.scale_x,
                session.scale_y,
                session.angle,
                session.dx,
                session.dy,
                x,
                y,
                zoom,
            ),
            None => -1,
        }
    }

    /// Update the active drag from document-space `(x, y)`. Shift locks the
    /// aspect ratio on a corner and snaps rotation to 15°; the transformed rect
    /// is clamped to at least 1 px per axis. Returns false without an active drag.
    pub fn transform_move(
        mut self: Pin<&mut Self>,
        x: f64,
        y: f64,
        _zoom: f64,
        shift: bool,
        _alt: bool,
    ) -> bool {
        if !x.is_finite() || !y.is_finite() {
            return false;
        }
        let mut rust = self.as_mut().rust_mut();
        let Some(session) = rust.transform_session.as_mut() else {
            return false;
        };
        if !session.dragging {
            return false;
        }
        match session.handle {
            MOVE_HANDLE => {
                gesture_translate(session, x, y);
                true
            }
            ROTATE_HANDLE => {
                gesture_rotate(session, x, y, shift);
                true
            }
            h @ 0..=7 => {
                session.handle = h;
                gesture_scale(session, x, y, shift)
            }
            _ => false,
        }
    }

    /// End the active drag. The session stays open until commit/cancel.
    pub fn transform_release(mut self: Pin<&mut Self>) -> bool {
        let mut rust = self.as_mut().rust_mut();
        let Some(session) = rust.transform_session.as_mut() else {
            return false;
        };
        session.dragging = false;
        session.handle = -1;
        true
    }

    /// Whether a Free Transform session is active.
    pub fn transform_session_active(&self) -> bool {
        self.rust().transform_session.is_some()
    }

    /// The active session's target path, or an empty string.
    pub fn transform_session_path(&self) -> QString {
        self.rust()
            .transform_session
            .as_ref()
            .map(|session| QString::from(session.path.as_str()))
            .unwrap_or_default()
    }

    pub fn transform_scale_x(&self) -> f64 {
        self.rust()
            .transform_session
            .as_ref()
            .map_or(1.0, |session| session.scale_x)
    }

    pub fn transform_scale_y(&self) -> f64 {
        self.rust()
            .transform_session
            .as_ref()
            .map_or(1.0, |session| session.scale_y)
    }

    pub fn transform_angle(&self) -> f64 {
        self.rust()
            .transform_session
            .as_ref()
            .map_or(0.0, |session| session.angle)
    }

    pub fn transform_dx(&self) -> f64 {
        self.rust()
            .transform_session
            .as_ref()
            .map_or(0.0, |session| session.dx)
    }

    pub fn transform_dy(&self) -> f64 {
        self.rust()
            .transform_session
            .as_ref()
            .map_or(0.0, |session| session.dy)
    }

    /// The session quad as `"x,y x,y x,y x,y"` (four document-space corners).
    pub fn transform_quad(&self) -> QString {
        let Some(session) = self.rust().transform_session.as_ref() else {
            return QString::default();
        };
        let quad = transform_quad_points(
            session.orig_rect,
            session.scale_x,
            session.scale_y,
            session.angle,
            session.dx,
            session.dy,
        );
        let encoded: Vec<String> = quad.iter().map(|(x, y)| format!("{x:.2},{y:.2}")).collect();
        QString::from(encoded.join(" "))
    }

    /// The layer at `path`'s rect as `"left top right bottom"`, or empty.
    pub fn layer_rect(&self, path: &QString) -> QString {
        let rect = self
            .rust()
            .doc
            .as_ref()
            .and_then(|doc| pictura_render::resolve_path(doc, &path.to_string()))
            .map(|layer| layer.rect);
        rect.map_or_else(QString::default, |rect| {
            QString::from(format!(
                "{} {} {} {}",
                rect.left, rect.top, rect.right, rect.bottom
            ))
        })
    }
}

impl qobject::PictureView {
    pub fn translate_layer(mut self: Pin<&mut Self>, dx: i32, dy: i32) -> bool {
        let moved = {
            let mut rust = self.as_mut().rust_mut();
            let Some(doc) = rust.doc.as_mut() else {
                return false;
            };
            pictura_render::translate_layer(doc, dx, dy)
        };
        if moved {
            self.as_mut().recomposite();
            self.as_mut().record_move("Move Layer");
        }
        moved
    }

    pub fn move_preview(mut self: Pin<&mut Self>, dx: i32, dy: i32) -> bool {
        if dx == 0 && dy == 0 {
            return false;
        }
        let moved = {
            let mut rust = self.as_mut().rust_mut();
            let Some(doc) = rust.doc.as_mut() else {
                return false;
            };
            pictura_render::translate_layer(doc, dx, dy)
        };
        if moved {
            let gpu_compute = self.rust().gpu_compute;
            let image = self
                .rust()
                .doc
                .as_ref()
                .map(|doc| document_to_image(doc, gpu_compute));
            if let Some(image) = image {
                let mut rust = self.as_mut().rust_mut();
                rust.image = image;
                rust.display_dirty = false;
            }
            self.changed();
        }
        moved
    }

    /// Cache the base composite (topmost raster layer hidden), the layer image,
    /// its document-space origin, and opacity. One region composite at drag
    /// start, derived from the authoritative `doc.composite`.
    ///
    /// Reuses the cached base when the content revision, topmost layer, and
    /// clamped rect all match; otherwise it recomputes.
    pub fn begin_move_preview(mut self: Pin<&mut Self>) -> bool {
        let Some(index) = self.as_ref().move_cache_target() else {
            return false;
        };
        {
            let rust = self.rust();
            let Some(layer) = rust
                .doc
                .as_ref()
                .and_then(|doc| doc.layers.get(index as usize))
            else {
                return false;
            };
            if layer_move_locked(layer) {
                return false;
            }
        }
        if self.as_ref().move_cache_valid(index) {
            {
                let mut rust = self.as_mut().rust_mut();
                let (x, y, opacity) = rust
                    .doc
                    .as_ref()
                    .and_then(|doc| doc.layers.get(index as usize))
                    .map_or((0, 0, 0), |layer| {
                        (layer.rect.left, layer.rect.top, layer.opacity as i32)
                    });
                rust.move_x = x;
                rust.move_y = y;
                rust.move_opacity = opacity;
                rust.move_preview_cache_hit = true;
            }
            return true;
        }
        let computed = self.as_mut().compute_move_preview(index as usize);
        self.as_mut().rust_mut().move_preview_cache_hit = false;
        computed
    }

    /// Warm the move-preview cache without entering preview mode.
    pub fn prepare_move_preview(mut self: Pin<&mut Self>) -> bool {
        let Some(index) = self.as_ref().move_cache_target() else {
            return false;
        };
        if self.as_ref().move_cache_valid(index) {
            return true;
        }
        self.as_mut().compute_move_preview(index as usize)
    }

    pub fn move_preview_cache_hit(&self) -> bool {
        self.rust().move_preview_cache_hit
    }

    /// The current topmost pixel layer index, or `None` without a document or
    /// raster layer.
    fn move_cache_target(&self) -> Option<i32> {
        Some(topmost_pixel_layer_index(self.rust().doc.as_ref()?)? as i32)
    }

    fn move_cache_valid(&self, index: i32) -> bool {
        let rust = self.rust();
        // The base is the document with the topmost layer hidden, so it does not
        // depend on that layer's position: a committed move leaves it valid.
        // `record_move` deliberately does not bump `content_revision` for this
        // reason; keying on the layer rect here would throw the base away on
        // every drag and force a full region recomposite on the next press.
        rust.move_base.is_some()
            && rust.move_prepared_revision == rust.content_revision
            && rust.move_prepared_layer == index
    }

    /// Compute and store the move-preview base/layer for the top-level layer
    /// `index` and record the revision and index they were built from.
    fn compute_move_preview(mut self: Pin<&mut Self>, index: usize) -> bool {
        let mut guard = self.as_mut().rust_mut();
        let rust = &mut *guard;
        let gpu_compute = rust.gpu_compute;
        let Some(doc) = rust.doc.as_mut() else {
            return false;
        };
        let Some(layer_image) = layer_image(&doc.layers[index]) else {
            return false;
        };
        let rect = doc.layers[index].rect;
        let (x, y, opacity) = (rect.left, rect.top, doc.layers[index].opacity as i32);
        // Build the base from the authoritative planar composite, never a
        // possibly-stale cached image: clone it and overwrite the moved layer's
        // rectangle with the region composited with the layer hidden.
        let composite_ok = doc.composite.width == doc.width
            && doc.composite.height == doc.height
            && !doc.composite.data.is_empty();
        let base = match move_preview_region(rect, doc.width, doc.height, composite_ok) {
            Some((x0, y0, ..)) => {
                doc.layers[index].visible = false;
                let (region, _backend) =
                    pictura_render::composite_region_active(doc, rect, gpu_compute);
                doc.layers[index].visible = true;
                let mut base_buffer = doc.composite.clone();
                patch_buffer_region(&mut base_buffer, &region, x0, y0);
                buffer_to_image(&base_buffer)
            }
            None => {
                // ponytail: full-composite fallback for a missing/mismatched
                // composite; the region path covers the common case.
                doc.layers[index].visible = false;
                let base = document_to_image(doc, gpu_compute);
                doc.layers[index].visible = true;
                base
            }
        };
        rust.move_base = Some(base);
        rust.move_layer = Some(layer_image);
        rust.move_x = x;
        rust.move_y = y;
        rust.move_opacity = opacity;
        rust.move_prepared_revision = rust.content_revision;
        rust.move_prepared_layer = index as i32;
        true
    }

    pub fn move_preview_base(&self) -> QImage {
        self.rust().move_base.clone().unwrap_or_default()
    }

    pub fn move_preview_layer(&self) -> QImage {
        self.rust().move_layer.clone().unwrap_or_default()
    }

    pub fn move_preview_x(&self) -> i32 {
        self.rust().move_x
    }

    pub fn move_preview_y(&self) -> i32 {
        self.rust().move_y
    }

    pub fn move_preview_opacity(&self) -> i32 {
        self.rust().move_opacity
    }

    pub fn end_move_preview(self: Pin<&mut Self>) {
        // Keep the cache: the next press reuses it when the revision, topmost
        // layer, and clamped rect are unchanged.
    }

    pub fn commit_move(mut self: Pin<&mut Self>, dx: i32, dy: i32) -> bool {
        if dx == 0 && dy == 0 {
            return false;
        }
        let before = {
            let rust = self.rust();
            let Some(doc) = rust.doc.as_ref() else {
                return false;
            };
            match topmost_pixel_layer_rect(doc) {
                Some(rect) => rect,
                None => return false,
            }
        };
        let moved = {
            let mut rust = self.as_mut().rust_mut();
            let Some(doc) = rust.doc.as_mut() else {
                return false;
            };
            pictura_render::translate_layer_rect(doc, dx, dy)
        };
        if !moved {
            return false;
        }
        let dirty = {
            let rust = self.rust();
            let Some(doc) = rust.doc.as_ref() else {
                return false;
            };
            union_rect(before, topmost_pixel_layer_rect(doc).unwrap_or(before))
        };
        self.as_mut().refresh_region(dirty);
        self.as_mut().record_move("Move Layer");
        true
    }

    pub fn commit_move_legacy(mut self: Pin<&mut Self>) -> bool {
        if self.rust().doc.is_none() {
            return false;
        }
        self.as_mut().recomposite();
        self.as_mut().record("Move Layer");
        true
    }

    pub fn resize_image(mut self: Pin<&mut Self>, kind: &QString, width: i32, height: i32) -> bool {
        let Some(resample) = parse_resample(&kind.to_string()) else {
            return false;
        };
        if width < 1 || height < 1 {
            return false;
        }
        let resized = {
            let mut rust = self.as_mut().rust_mut();
            let Some(doc) = rust.doc.as_mut() else {
                return false;
            };
            pictura_render::resize_document(doc, width as u32, height as u32, resample).is_ok()
        };
        if resized {
            self.as_mut().rust_mut().selection = None;
            self.as_mut().recomposite();
            self.as_mut().record("Image Size");
        }
        resized
    }

    pub fn resize_canvas(
        mut self: Pin<&mut Self>,
        anchor: &QString,
        width: i32,
        height: i32,
    ) -> bool {
        let Some(anchor) = parse_anchor(&anchor.to_string()) else {
            return false;
        };
        if width < 1 || height < 1 {
            return false;
        }
        let resized = {
            let mut rust = self.as_mut().rust_mut();
            let Some(doc) = rust.doc.as_mut() else {
                return false;
            };
            pictura_render::resize_canvas_document(doc, width as u32, height as u32, anchor).is_ok()
        };
        if resized {
            self.as_mut().rust_mut().selection = None;
            self.as_mut().recomposite();
            self.as_mut().record("Canvas Size");
        }
        resized
    }

    pub fn rotate_doc(mut self: Pin<&mut Self>, quarter_turns: i32) -> bool {
        if !(1..=3).contains(&quarter_turns) {
            return false;
        }
        let rotated = {
            let mut rust = self.as_mut().rust_mut();
            let Some(doc) = rust.doc.as_mut() else {
                return false;
            };
            pictura_render::rotate_document(doc, quarter_turns as u8).is_ok()
        };
        if rotated {
            self.as_mut().rust_mut().selection = None;
            self.as_mut().recomposite();
            self.as_mut().record("Rotate");
        }
        rotated
    }

    pub fn flip_doc(mut self: Pin<&mut Self>, horizontal: bool) -> bool {
        {
            let mut rust = self.as_mut().rust_mut();
            let Some(doc) = rust.doc.as_mut() else {
                return false;
            };
            pictura_render::flip_document(doc, horizontal);
            rust.selection = None;
        }
        self.as_mut().recomposite();
        self.as_mut().record("Flip");
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect4() -> PsdRect {
        PsdRect {
            top: 0,
            left: 0,
            bottom: 4,
            right: 4,
        }
    }

    fn rect100() -> PsdRect {
        PsdRect {
            top: 0,
            left: 0,
            bottom: 100,
            right: 100,
        }
    }

    fn session() -> TransformSession {
        TransformSession {
            path: "0".to_string(),
            orig_rect: rect4(),
            scale_x: 1.0,
            scale_y: 1.0,
            angle: 0.0,
            dx: 0.0,
            dy: 0.0,
            handle: -1,
            press_x: 0.0,
            press_y: 0.0,
            start: [1.0, 1.0, 0.0, 0.0, 0.0],
            dragging: true,
        }
    }

    #[test]
    fn hit_test_classifies_handles_move_and_rotate() {
        let r = rect100();
        assert_eq!(hit_test(r, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0), 0);
        assert_eq!(hit_test(r, 1.0, 1.0, 0.0, 0.0, 0.0, 100.0, 100.0, 1.0), 2);
        assert_eq!(hit_test(r, 1.0, 1.0, 0.0, 0.0, 0.0, 50.0, 0.0, 1.0), 4);
        assert_eq!(
            hit_test(r, 1.0, 1.0, 0.0, 0.0, 0.0, 50.0, 50.0, 1.0),
            MOVE_HANDLE
        );
        assert_eq!(
            hit_test(r, 1.0, 1.0, 0.0, 0.0, 0.0, -10.0, -10.0, 1.0),
            ROTATE_HANDLE
        );
        assert_eq!(hit_test(r, 1.0, 1.0, 0.0, 0.0, 0.0, 300.0, 300.0, 1.0), -1);
    }

    #[test]
    fn identity_quad_is_the_source_rect() {
        let q = transform_quad_points(rect4(), 1.0, 1.0, 0.0, 0.0, 0.0);
        assert_eq!(q, [(0.0, 0.0), (4.0, 0.0), (4.0, 4.0), (0.0, 4.0)]);
    }

    #[test]
    fn opposite_handles_are_paired() {
        assert_eq!(opposite_handle(0), 2);
        assert_eq!(opposite_handle(1), 3);
        assert_eq!(opposite_handle(4), 6);
        assert_eq!(opposite_handle(5), 7);
        for h in 0..8 {
            assert_eq!(opposite_handle(opposite_handle(h)), h);
        }
    }

    #[test]
    fn corner_scale_keeps_the_opposite_corner_fixed() {
        let mut s = session();
        s.handle = 0;
        s.press_x = 0.0;
        s.press_y = 0.0;
        assert!(gesture_scale(&mut s, -4.0, -4.0, false));
        assert!((s.scale_x - 2.0).abs() < 1e-9);
        assert!((s.scale_y - 2.0).abs() < 1e-9);
        let anchor = forward_point(
            s.orig_rect,
            s.scale_x,
            s.scale_y,
            s.angle,
            s.dx,
            s.dy,
            (4.0, 4.0),
        );
        assert!((anchor.0 - 4.0).abs() < 1e-9 && (anchor.1 - 4.0).abs() < 1e-9);
        let dragged = forward_point(
            s.orig_rect,
            s.scale_x,
            s.scale_y,
            s.angle,
            s.dx,
            s.dy,
            (0.0, 0.0),
        );
        assert!((dragged.0 + 4.0).abs() < 1e-9 && (dragged.1 + 4.0).abs() < 1e-9);
    }

    #[test]
    fn shift_on_a_corner_locks_the_two_scales_equal() {
        let mut s = session();
        s.handle = 0;
        assert!(gesture_scale(&mut s, -4.0, -8.0, true));
        assert!((s.scale_x - s.scale_y).abs() < 1e-12);
    }

    #[test]
    fn shift_on_an_edge_handle_leaves_the_other_axis() {
        let mut s = session();
        s.handle = 4;
        assert!(gesture_scale(&mut s, 2.0, -4.0, true));
        assert!((s.scale_x - 1.0).abs() < 1e-9, "x stays 1.0");
        assert!((s.scale_y - 2.0).abs() < 1e-9, "y scales to 2.0");
    }

    #[test]
    fn scale_clamps_to_at_least_one_pixel_per_axis() {
        let mut s = session();
        s.handle = 0;
        assert!(gesture_scale(&mut s, 4.0 - 0.001, 4.0 - 0.001, false));
        assert!((s.scale_x.abs() - 0.25).abs() < 1e-9);
        assert!((s.scale_y.abs() - 0.25).abs() < 1e-9);
    }

    #[test]
    fn rotation_tracks_the_pointer_and_snaps() {
        let mut s = session();
        s.handle = ROTATE_HANDLE;
        s.press_x = 4.0;
        s.press_y = 2.0;
        gesture_rotate(&mut s, 2.0, 4.0, false);
        assert!((s.angle - std::f64::consts::FRAC_PI_2).abs() < 1e-9);

        let mut snapped = session();
        snapped.handle = ROTATE_HANDLE;
        snapped.press_x = 4.0;
        snapped.press_y = 2.0;
        let target: f64 = 0.2;
        gesture_rotate(
            &mut snapped,
            2.0 + target.cos() * 2.0,
            2.0 + target.sin() * 2.0,
            true,
        );
        assert!((snapped.angle - std::f64::consts::PI / 12.0).abs() < 1e-9);
    }

    #[test]
    fn translation_accumulates_from_press() {
        let mut s = session();
        s.handle = MOVE_HANDLE;
        s.press_x = 1.0;
        s.press_y = 1.0;
        gesture_translate(&mut s, 3.0, 5.0);
        assert!((s.dx - 2.0).abs() < 1e-12 && (s.dy - 4.0).abs() < 1e-12);
    }
}
