//! A live shape's `vogk` (vector origination data) block: the parametric
//! rectangle, rounded rectangle, or ellipse a shape layer was drawn as, which
//! Photoshop CC keeps beside the `vmsk` outline until an edit turns the shape
//! into a regular path. Live shapes are a CC feature; CS6 keeps the block
//! opaque.
//!
//! Layout (inferred from ag-psd's and psd-tools' readers; no Adobe source):
//! a `u32` version 1, then a version-16 descriptor whose `keyDescriptorList`
//! holds one object per shape with `keyOriginType`, `keyOriginResolution`,
//! `keyOriginRRectRadii` (rounded rectangles), `keyOriginShapeBBox`,
//! `keyOriginBoxCorners`, `Trnf`, and `keyOriginIndex`.

use crate::descriptor::{get_object_item, write_descriptor, DescValue};

/// `keyOriginType` values.
pub const ORIGIN_RECTANGLE: i32 = 1;
pub const ORIGIN_ROUNDED_RECTANGLE: i32 = 2;
pub const ORIGIN_ELLIPSE: i32 = 5;

/// A decoded live shape: its origin type, its box as `(left, top, right,
/// bottom)` document pixels, and its corner radii (top-left, top-right,
/// bottom-right, bottom-left; zero unless a rounded rectangle).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LiveShape {
    pub origin_type: i32,
    pub bounds: (f64, f64, f64, f64),
    pub radii: [f64; 4],
}

impl LiveShape {
    /// The same shape moved by `(dx, dy)`.
    pub fn translated(self, dx: f64, dy: f64) -> LiveShape {
        let (l, t, r, b) = self.bounds;
        LiveShape {
            bounds: (l + dx, t + dy, r + dx, b + dy),
            ..self
        }
    }
}

fn key(k: &str) -> Vec<u8> {
    k.as_bytes().to_vec()
}

fn object(class: &str, items: Vec<(&str, DescValue)>) -> DescValue {
    DescValue::Object {
        name: String::new(),
        class_id: key(class),
        items: items.into_iter().map(|(k, v)| (key(k), v)).collect(),
    }
}

fn pixels(value: f64) -> DescValue {
    DescValue::UnitFloat {
        unit: *b"#Pxl",
        value,
    }
}

fn corner(x: f64, y: f64) -> DescValue {
    object(
        "Pnt ",
        vec![
            ("Hrzn", DescValue::Double(x)),
            ("Vrtc", DescValue::Double(y)),
        ],
    )
}

/// The `vogk` block bytes for `shape`.
pub fn encode_live_shape(shape: &LiveShape) -> Vec<u8> {
    let (l, t, r, b) = shape.bounds;
    let [tl, tr, br, bl] = shape.radii;
    let mut items = vec![
        ("keyOriginType", DescValue::Long(shape.origin_type)),
        ("keyOriginResolution", DescValue::Double(72.0)),
    ];
    if shape.origin_type == ORIGIN_ROUNDED_RECTANGLE {
        items.push((
            "keyOriginRRectRadii",
            object(
                "radii",
                vec![
                    ("unitValueQuadVersion", DescValue::Long(1)),
                    ("topRight", pixels(tr)),
                    ("topLeft", pixels(tl)),
                    ("bottomLeft", pixels(bl)),
                    ("bottomRight", pixels(br)),
                ],
            ),
        ));
    }
    items.extend([
        (
            "keyOriginShapeBBox",
            object(
                "unitRect",
                vec![
                    ("unitValueQuadVersion", DescValue::Long(1)),
                    ("Top ", pixels(t)),
                    ("Left", pixels(l)),
                    ("Btom", pixels(b)),
                    ("Rght", pixels(r)),
                ],
            ),
        ),
        (
            "keyOriginBoxCorners",
            object(
                "null",
                vec![
                    ("rectangleCornerA", corner(l, t)),
                    ("rectangleCornerB", corner(r, t)),
                    ("rectangleCornerC", corner(r, b)),
                    ("rectangleCornerD", corner(l, b)),
                ],
            ),
        ),
        (
            "Trnf",
            object(
                "Trnf",
                ["xx", "xy", "yx", "yy", "tx", "ty"]
                    .into_iter()
                    .zip([1.0, 0.0, 0.0, 1.0, 0.0, 0.0])
                    .map(|(k, v)| (k, DescValue::Double(v)))
                    .collect(),
            ),
        ),
        ("keyOriginIndex", DescValue::Long(0)),
    ]);
    let descriptor = object(
        "null",
        vec![(
            "keyDescriptorList",
            DescValue::List(vec![object("null", items)]),
        )],
    );
    let mut out = 1u32.to_be_bytes().to_vec();
    out.extend_from_slice(&write_descriptor(&descriptor));
    out
}

fn items(value: &DescValue) -> Option<&[(Vec<u8>, DescValue)]> {
    match value {
        DescValue::Object { items, .. } => Some(items),
        _ => None,
    }
}

fn number(items: &[(Vec<u8>, DescValue)], k: &str) -> Option<f64> {
    match get_object_item(items, k.as_bytes())? {
        DescValue::UnitFloat { value, .. } | DescValue::Double(value) => Some(*value),
        DescValue::Long(v) => Some(f64::from(*v)),
        _ => None,
    }
}

/// The first live shape in a `vogk` block, or `None` when the block is
/// malformed or describes no rectangle, rounded rectangle, or ellipse.
pub fn decode_live_shape(data: &[u8]) -> Option<LiveShape> {
    let version = u32::from_be_bytes(data.get(..4)?.try_into().ok()?);
    if version != 1 {
        return None;
    }
    let root = crate::read_descriptor(&data[4..]).ok()?;
    let DescValue::List(list) = get_object_item(items(&root)?, b"keyDescriptorList")? else {
        return None;
    };
    let shape = items(list.first()?)?;
    let DescValue::Long(origin_type) = get_object_item(shape, b"keyOriginType")? else {
        return None;
    };
    if ![ORIGIN_RECTANGLE, ORIGIN_ROUNDED_RECTANGLE, ORIGIN_ELLIPSE].contains(origin_type) {
        return None;
    }
    let bbox = items(get_object_item(shape, b"keyOriginShapeBBox")?)?;
    let bounds = (
        number(bbox, "Left")?,
        number(bbox, "Top ")?,
        number(bbox, "Rght")?,
        number(bbox, "Btom")?,
    );
    let radii = get_object_item(shape, b"keyOriginRRectRadii")
        .and_then(items)
        .map_or([0.0; 4], |r| {
            ["topLeft", "topRight", "bottomRight", "bottomLeft"]
                .map(|k| number(r, k).unwrap_or(0.0))
        });
    Some(LiveShape {
        origin_type: *origin_type,
        bounds,
        radii,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_live_shape_round_trips_and_moves() {
        let shape = LiveShape {
            origin_type: ORIGIN_ROUNDED_RECTANGLE,
            bounds: (10.0, 20.0, 110.0, 80.0),
            radii: [1.0, 2.0, 3.0, 4.0],
        };
        let data = encode_live_shape(&shape);
        assert_eq!(&data[..8], &[0, 0, 0, 1, 0, 0, 0, 16]);
        assert_eq!(decode_live_shape(&data), Some(shape));
        let moved = shape.translated(5.0, -5.0);
        assert_eq!(moved.bounds, (15.0, 15.0, 115.0, 75.0));

        let ellipse = LiveShape {
            origin_type: ORIGIN_ELLIPSE,
            radii: [0.0; 4],
            ..shape
        };
        assert_eq!(
            decode_live_shape(&encode_live_shape(&ellipse)),
            Some(ellipse)
        );
    }

    #[test]
    fn malformed_or_foreign_blocks_are_not_live() {
        assert_eq!(decode_live_shape(&[]), None);
        assert_eq!(decode_live_shape(&[0, 0, 0, 2]), None);
        let line = LiveShape {
            origin_type: 4,
            bounds: (0.0, 0.0, 1.0, 1.0),
            radii: [0.0; 4],
        };
        assert_eq!(decode_live_shape(&encode_live_shape(&line)), None);
    }
}
