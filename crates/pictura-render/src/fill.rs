//! Generative fill content: solid (`SoCo`) and gradient (`GdFl`).
//!
//! A fill layer owns no pixels; it is a descriptor that the compositor paints
//! over the layer's rect. This module is the single decoder for both fill keys
//! and the geometry that turns a gradient descriptor into pixels, following
//! psd-tools' `composite/paint.py::draw_gradient_fill`.
//!
//! ponytail: noise gradients (`ClNs`), transparency/opacity stops, stop
//! midpoints (`Mdpn`), non-linear interpolation (`Intr`), and non-RGB colour
//! models are not modelled; a fill that needs them renders as the nearest
//! custom-stop approximation. Raise these one at a time when a fixture shows a
//! real gap.

use pictura_adjust::{Adjustment, GradientFillParams, GradientKind, GradientStop};
use pictura_codec::DescValue;
use pictura_core::{AdjustmentData, Layer};

use crate::composite::{blend_into, desc_item, Canvas};

/// The decoded `GdFl` descriptor, or `None` when it is not a custom-stop
/// gradient this crate understands. Never panics; every failure is a no-op.
pub fn decode_gradient_fill(d: &[u8]) -> Option<Adjustment> {
    let obj = pictura_codec::read_descriptor(d).ok()?;
    if !matches!(obj, DescValue::Object { .. }) {
        return None;
    }
    let angle_deg = match desc_item(&obj, b"Angl") {
        Some(DescValue::Double(v)) if v.is_finite() => *v as f32,
        _ => return None,
    };
    let kind = match desc_item(&obj, b"Type") {
        Some(DescValue::Enum { value, .. }) => decode_kind(value)?,
        _ => return None,
    };
    let grad = desc_item(&obj, b"Grad")?;
    if !matches!(grad, DescValue::Object { .. }) {
        return None;
    }
    match desc_item(grad, b"GrdF") {
        Some(DescValue::Enum { value, .. }) if value.as_slice() == b"CstS" => {}
        _ => return None,
    }
    let stops = decode_stops(desc_item(grad, b"Clrs")?)?;
    let reverse = match desc_item(&obj, b"Rvrs") {
        Some(DescValue::Bool(b)) => *b,
        Some(_) => return None,
        None => false,
    };
    let scale = match desc_item(&obj, b"Scl ") {
        Some(DescValue::Double(v)) if v.is_finite() => *v as f32,
        Some(_) => return None,
        None => 100.0,
    };
    Some(Adjustment::GradientFill(GradientFillParams {
        stops,
        reverse,
        kind,
        angle_deg,
        scale,
    }))
}

fn decode_kind(value: &[u8]) -> Option<GradientKind> {
    Some(match value {
        b"Lnr " => GradientKind::Linear,
        b"Rdl " => GradientKind::Radial,
        b"Angl" => GradientKind::Angle,
        b"Rflc" => GradientKind::Reflected,
        b"Dmnd" => GradientKind::Diamond,
        _ => return None,
    })
}

fn kind_enum(kind: GradientKind) -> &'static [u8] {
    match kind {
        GradientKind::Linear => b"Lnr ",
        GradientKind::Radial => b"Rdl ",
        GradientKind::Angle => b"Angl",
        GradientKind::Reflected => b"Rflc",
        GradientKind::Diamond => b"Dmnd",
    }
}

fn decode_stops(value: &DescValue) -> Option<Vec<GradientStop>> {
    let DescValue::List(items) = value else {
        return None;
    };
    if items.len() < 2 {
        return None;
    }
    let mut stops = Vec::with_capacity(items.len());
    let mut previous: Option<u16> = None;
    for item in items {
        if !matches!(item, DescValue::Object { .. }) {
            return None;
        }
        let clr = desc_item(item, b"Clr ")?;
        if !matches!(clr, DescValue::Object { .. }) {
            return None;
        }
        let component = |key: &[u8]| match desc_item(clr, key) {
            Some(DescValue::Double(c)) if c.is_finite() => Some(c.round().clamp(0.0, 255.0) as u8),
            _ => None,
        };
        let color = [
            component(b"Rd  ")?,
            component(b"Grn ")?,
            component(b"Bl  ")?,
        ];
        let location = match desc_item(item, b"Lctn") {
            Some(DescValue::Double(v)) if v.is_finite() && (0.0..=4096.0).contains(v) => {
                v.round() as u16
            }
            Some(DescValue::Long(v)) if (0..=4096).contains(v) => *v as u16,
            _ => return None,
        };
        if previous.is_some_and(|p| location <= p) {
            return None;
        }
        previous = Some(location);
        stops.push(GradientStop { location, color });
    }
    Some(stops)
}

/// Build a standard Photoshop `GdFl` descriptor. `reverse` and `scale` are not
/// written, so a decoded copy is forward at scale 100 (the psd-tools defaults);
/// only custom-stop gradients are emitted, matching the decoder.
pub fn encode_gradient_fill(
    kind: GradientKind,
    stops: &[GradientStop],
    angle_deg: f32,
) -> AdjustmentData {
    let grad = DescValue::Object {
        name: String::new(),
        class_id: b"Grdn".to_vec(),
        items: vec![
            (b"Nm  ".to_vec(), DescValue::Text("Gradient".into())),
            (
                b"GrdF".to_vec(),
                DescValue::Enum {
                    kind: b"GrdF".to_vec(),
                    value: b"CstS".to_vec(),
                },
            ),
            (
                b"Intr".to_vec(),
                DescValue::Enum {
                    kind: b"Intp".to_vec(),
                    value: b"Lnr ".to_vec(),
                },
            ),
            (
                b"Clrs".to_vec(),
                DescValue::List(stops.iter().map(encode_stop).collect()),
            ),
        ],
    };
    let desc = DescValue::Object {
        name: String::new(),
        class_id: b"GdFl".to_vec(),
        items: vec![
            (b"Angl".to_vec(), DescValue::Double(angle_deg as f64)),
            (
                b"Type".to_vec(),
                DescValue::Enum {
                    kind: b"GrdT".to_vec(),
                    value: kind_enum(kind).to_vec(),
                },
            ),
            (b"Grad".to_vec(), grad),
        ],
    };
    AdjustmentData {
        key: *b"GdFl",
        data: pictura_codec::write_descriptor(&desc),
    }
}

fn encode_stop(stop: &GradientStop) -> DescValue {
    let clr = DescValue::Object {
        name: String::new(),
        class_id: b"RGBC".to_vec(),
        items: [b"Rd  ", b"Grn ", b"Bl  "]
            .into_iter()
            .zip(stop.color)
            .map(|(key, c)| (key.to_vec(), DescValue::Double(c as f64)))
            .collect(),
    };
    DescValue::Object {
        name: String::new(),
        class_id: b"Clrt".to_vec(),
        items: vec![
            (b"Clr ".to_vec(), clr),
            (
                b"Type".to_vec(),
                DescValue::Enum {
                    kind: b"Clry".to_vec(),
                    value: b"UsrS".to_vec(),
                },
            ),
            (b"Lctn".to_vec(), DescValue::Double(stop.location as f64)),
            (b"Mdpn".to_vec(), DescValue::Double(50.0)),
        ],
    }
}

/// Generate the opaque RGBA gradient over a `w × h` layer rect (row-major).
///
/// Mirrors psd-tools' `draw_gradient_fill` coordinate convention and the five
/// kind geometries; alpha is always 255.
pub(crate) fn gradient_rgba(w: i32, h: i32, params: &GradientFillParams) -> Vec<[u8; 4]> {
    let mut out = vec![[0u8, 0, 0, 255]; w.max(0) as usize * h.max(0) as usize];
    if w <= 0 || h <= 0 {
        return out;
    }
    let wf = w as f64;
    let hf = h as f64;
    let angle = params.angle_deg as f64;
    let ratio = angle.rem_euclid(90.0);
    let s = (params.scale as f64 / 100.0) * ((90.0 - ratio) / 90.0 * wf + (ratio / 90.0) * hf);
    if !s.is_finite() || s <= 0.0 {
        return out;
    }
    let theta = angle.rem_euclid(360.0).to_radians();
    let (sin_t, cos_t) = (theta.sin(), theta.cos());
    let x0 = -wf / s;
    let x_step = if w > 1 {
        2.0 * wf / s / (wf - 1.0)
    } else {
        0.0
    };
    let y0 = -hf / s;
    let y_step = if h > 1 {
        2.0 * hf / s / (hf - 1.0)
    } else {
        0.0
    };
    let row = w as usize;
    for j in 0..h as usize {
        let y = y0 + j as f64 * y_step;
        for i in 0..row {
            let x = x0 + i as f64 * x_step;
            let mut z = match params.kind {
                GradientKind::Linear => 0.5 * (cos_t * x - sin_t * y + 1.0),
                GradientKind::Radial => (x * x + y * y).sqrt(),
                GradientKind::Angle => {
                    let a = (180.0 * y.atan2(x) / std::f64::consts::PI) + angle;
                    a.rem_euclid(360.0) / 360.0
                }
                GradientKind::Reflected => (cos_t * x - sin_t * y).abs(),
                GradientKind::Diamond => {
                    (cos_t * x - sin_t * y).abs() + (sin_t * x + cos_t * y).abs()
                }
            };
            z = z.clamp(0.0, 1.0);
            if params.reverse {
                z = 1.0 - z;
            }
            out[j * row + i] = sample_stops(&params.stops, z);
        }
    }
    out
}

/// Linearly sample `stops` at index `z ∈ [0, 1]` (`z * 4096`), clamping outside
/// the stop range. Stops are strictly increasing in location.
fn sample_stops(stops: &[GradientStop], z: f64) -> [u8; 4] {
    let Some(first) = stops.first() else {
        return [0, 0, 0, 255];
    };
    let last = stops.last().unwrap_or(first);
    let pos = z * 4096.0;
    if stops.len() == 1 || pos <= first.location as f64 {
        return [first.color[0], first.color[1], first.color[2], 255];
    }
    if pos >= last.location as f64 {
        return [last.color[0], last.color[1], last.color[2], 255];
    }
    for pair in stops.windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        if pos >= a.location as f64 && pos <= b.location as f64 {
            let span = (b.location - a.location) as f64;
            let t = if span <= 0.0 {
                0.0
            } else {
                (pos - a.location as f64) / span
            };
            let lerp = |i: usize| {
                (a.color[i] as f64 + (b.color[i] as f64 - a.color[i] as f64) * t)
                    .round()
                    .clamp(0.0, 255.0) as u8
            };
            return [lerp(0), lerp(1), lerp(2), 255];
        }
    }
    [last.color[0], last.color[1], last.color[2], 255]
}

/// Composite a solid fill across the layer's rect (clamped to the canvas).
///
/// Unlike a destructive adjustment, every in-rect pixel is source content at
/// the payload's alpha; the layer's mask, opacity, fill, and blend still apply
/// through [`blend_into`]. Pixels outside the rect are untouched.
pub(crate) fn composite_solid_fill(canvas: &mut Canvas, layer: &Layer, rgba: [u8; 4]) {
    let x0 = layer.rect.left.max(0);
    let y0 = layer.rect.top.max(0);
    let x1 = layer.rect.right.min(canvas.w as i32);
    let y1 = layer.rect.bottom.min(canvas.h as i32);
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    let cs = [
        rgba[0] as f32 / 255.0,
        rgba[1] as f32 / 255.0,
        rgba[2] as f32 / 255.0,
    ];
    let src_a = rgba[3] as f32 / 255.0;
    for y in y0..y1 {
        for x in x0..x1 {
            blend_into(canvas, layer, x as usize, y as usize, cs, src_a);
        }
    }
}

/// Composite a gradient fill across the layer's rect (clamped to the canvas),
/// generating an opaque colour per pixel and blending it through [`blend_into`].
pub(crate) fn composite_gradient_fill(
    canvas: &mut Canvas,
    layer: &Layer,
    params: &GradientFillParams,
) {
    let w = layer.rect.width();
    let h = layer.rect.height();
    if w <= 0 || h <= 0 {
        return;
    }
    let x0 = layer.rect.left.max(0);
    let y0 = layer.rect.top.max(0);
    let x1 = layer.rect.right.min(canvas.w as i32);
    let y1 = layer.rect.bottom.min(canvas.h as i32);
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    let px = gradient_rgba(w, h, params);
    let row = w as usize;
    for y in y0..y1 {
        for x in x0..x1 {
            let li = (y - layer.rect.top) as usize * row + (x - layer.rect.left) as usize;
            let c = px[li];
            blend_into(
                canvas,
                layer,
                x as usize,
                y as usize,
                [
                    c[0] as f32 / 255.0,
                    c[1] as f32 / 255.0,
                    c[2] as f32 / 255.0,
                ],
                1.0,
            );
        }
    }
}
