//! Layer creation and grouping at document scope (M37).
//!
//! Pure functions over `&mut Document`, following the bottom-first convention of
//! `Document::layers` (index 0 is the bottom of the stack).

use pictura_core::{BlendMode, Channel, ColorLabel, Document, Layer, LockFlags, PsdRect};

/// Insertion index "directly above `above`".
///
/// `above` is a bottom-first layer index. A negative sentinel (no selection)
/// or an out-of-range index places the new node at the top of the stack.
fn insertion_index(len: usize, above: i32) -> usize {
    if above < 0 {
        len
    } else {
        (above as usize + 1).min(len)
    }
}

/// A document-sized raster layer with fully transparent pixels.
///
/// ponytail: Photoshop stores no pixel data until the layer is painted; a
/// document-sized layer is the lazy stand-in and costs `w*h*4` bytes. Switch to
/// an empty-rect layer once the paint path can grow a layer on first dab.
fn transparent_layer(width: u32, height: u32, name: &str) -> Layer {
    let pixels = width as usize * height as usize;
    Layer {
        name: name.to_string(),
        rect: PsdRect {
            top: 0,
            left: 0,
            bottom: height as i32,
            right: width as i32,
        },
        blend: BlendMode::Normal,
        opacity: 255,
        fill: 255,
        lock: LockFlags::default(),
        color: ColorLabel::None,
        clipping: false,
        visible: true,
        mask: None,
        adjustment: None,
        channels: vec![
            Channel {
                id: 0,
                data: vec![0; pixels],
            },
            Channel {
                id: 1,
                data: vec![0; pixels],
            },
            Channel {
                id: 2,
                data: vec![0; pixels],
            },
            Channel {
                id: -1,
                data: vec![0; pixels],
            },
        ],
        children: Vec::new(),
        is_group: false,
    }
}

/// An empty group with a Normal blend and an empty bounds rectangle.
fn empty_group(name: &str) -> Layer {
    Layer {
        name: name.to_string(),
        rect: PsdRect {
            top: 0,
            left: 0,
            bottom: 0,
            right: 0,
        },
        blend: BlendMode::Normal,
        opacity: 255,
        fill: 255,
        lock: LockFlags::default(),
        color: ColorLabel::None,
        clipping: false,
        visible: true,
        mask: None,
        adjustment: None,
        channels: Vec::new(),
        children: Vec::new(),
        is_group: true,
    }
}

/// `Prefix N` where N is one more than the highest existing `Prefix <number>`
/// name in the whole tree, falling back to 1.
pub fn next_layer_name(doc: &Document, prefix: &str) -> String {
    let mut highest = 0u32;
    fn visit(layers: &[Layer], prefix: &str, highest: &mut u32) {
        for layer in layers {
            let suffix = layer
                .name
                .strip_prefix(prefix)
                .and_then(|rest| rest.strip_prefix(' '))
                .and_then(|n| n.parse::<u32>().ok());
            if let Some(n) = suffix {
                *highest = (*highest).max(n);
            }
            visit(&layer.children, prefix, highest);
        }
    }
    visit(&doc.layers, prefix, &mut highest);
    format!("{prefix} {}", highest + 1)
}

/// Insert a new transparent raster layer directly above `above`; returns its
/// index, or -1 for a document with a zero dimension.
pub fn add_layer(doc: &mut Document, above: i32, name: &str) -> i32 {
    if doc.width == 0 || doc.height == 0 {
        return -1;
    }
    let index = insertion_index(doc.layers.len(), above);
    doc.layers
        .insert(index, transparent_layer(doc.width, doc.height, name));
    index as i32
}

/// Insert an empty group directly above `above`; returns its index.
pub fn add_group(doc: &mut Document, above: i32, name: &str) -> i32 {
    let index = insertion_index(doc.layers.len(), above);
    doc.layers.insert(index, empty_group(name));
    index as i32
}

/// Deep-clone layer `index` (children, channels, mask, adjustment and all
/// attributes) and insert the copy directly above it. The copy is named
/// `"<name> copy"`. Returns the new index, or -1 when `index` is out of range.
pub fn duplicate_layer(doc: &mut Document, index: i32) -> i32 {
    if index < 0 || index as usize >= doc.layers.len() {
        return -1;
    }
    let source = index as usize;
    let mut copy = doc.layers[source].clone();
    copy.name = format!("{} copy", copy.name);
    doc.layers.insert(source + 1, copy);
    (source + 1) as i32
}

/// Wrap layer `index` in a new group at the same stack position: the group
/// takes the layer's slot and the layer becomes its only child. Returns the
/// group's index, or -1 when `index` is out of range.
pub fn group_layer(doc: &mut Document, index: i32) -> i32 {
    if index < 0 || index as usize >= doc.layers.len() {
        return -1;
    }
    let source = index as usize;
    let child = doc.layers.remove(source);
    let name = next_layer_name(doc, "Group");
    let mut group = empty_group(&name);
    group.children.push(child);
    doc.layers.insert(source, group);
    source as i32
}

/// Splice a group's children into the parent at the group's position. Returns
/// false (state unchanged) when `index` is out of range or not a group.
pub fn ungroup_layer(doc: &mut Document, index: i32) -> bool {
    if index < 0 || index as usize >= doc.layers.len() {
        return false;
    }
    let at = index as usize;
    if !doc.layers[at].is_group {
        return false;
    }
    let group = doc.layers.remove(at);
    for (offset, child) in group.children.into_iter().enumerate() {
        doc.layers.insert(at + offset, child);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use pictura_core::{BitDepth, Channel, ColorMode, Layer, LayerMask, PsdRect};

    fn rect(w: i32, h: i32) -> PsdRect {
        PsdRect {
            top: 0,
            left: 0,
            bottom: h,
            right: w,
        }
    }

    fn pixel_layer(name: &str, w: u32, h: u32, value: u8) -> Layer {
        let n = w as usize * h as usize;
        Layer {
            name: name.into(),
            rect: rect(w as i32, h as i32),
            blend: BlendMode::Normal,
            opacity: 255,
            fill: 255,
            lock: LockFlags::default(),
            color: ColorLabel::None,
            clipping: false,
            visible: true,
            mask: None,
            adjustment: None,
            channels: vec![
                Channel {
                    id: 0,
                    data: vec![value; n],
                },
                Channel {
                    id: 1,
                    data: vec![value; n],
                },
                Channel {
                    id: 2,
                    data: vec![value; n],
                },
                Channel {
                    id: -1,
                    data: vec![255; n],
                },
            ],
            children: Vec::new(),
            is_group: false,
        }
    }

    fn doc_with(layers: Vec<Layer>) -> Document {
        let mut doc = Document::new(4, 4, ColorMode::Rgb, BitDepth::Eight);
        doc.layers = layers;
        doc
    }

    #[test]
    fn add_layer_inserts_above_and_is_transparent() {
        let mut doc = doc_with(vec![pixel_layer("base", 4, 4, 40)]);
        let index = add_layer(&mut doc, 0, "Layer 1");

        assert_eq!(index, 1);
        assert_eq!(doc.layers.len(), 2);
        let added = &doc.layers[1];
        assert_eq!(added.name, "Layer 1");
        assert_eq!(added.rect, rect(4, 4));
        assert_eq!(added.blend, BlendMode::Normal);
        assert_eq!((added.opacity, added.fill), (255, 255));
        assert!(added.visible && !added.is_group);
        assert_eq!(added.channels.len(), 4);
        for (id, channel) in [(0i16, 0), (1, 1), (2, 2), (-1, 3)] {
            assert_eq!(added.channels[channel].id, id);
            assert_eq!(added.channels[channel].data, vec![0u8; 16]);
        }
    }

    #[test]
    fn add_layer_without_selection_goes_to_top() {
        let mut doc = doc_with(vec![pixel_layer("a", 4, 4, 1), pixel_layer("b", 4, 4, 2)]);
        assert_eq!(
            add_layer(&mut doc, -1, "top"),
            2,
            "no selection inserts on top"
        );
        assert_eq!(doc.layers[2].name, "top");

        let mut doc = doc_with(vec![pixel_layer("a", 4, 4, 1)]);
        assert_eq!(
            add_layer(&mut doc, 99, "top"),
            1,
            "out-of-range inserts on top"
        );
    }

    #[test]
    fn add_group_inserts_empty_group() {
        let mut doc = doc_with(vec![pixel_layer("base", 4, 4, 40)]);
        let index = add_group(&mut doc, 0, "Group 1");

        assert_eq!(index, 1);
        let group = &doc.layers[1];
        assert!(group.is_group);
        assert!(group.children.is_empty());
        assert_eq!(group.rect, rect(0, 0));
        assert_eq!(group.blend, BlendMode::Normal);
    }

    #[test]
    fn duplicate_layer_deep_copies_above_the_source() {
        let mask = LayerMask {
            rect: rect(4, 4),
            default_color: 255,
            disabled: false,
            flags: 0,
            data: Some(vec![128; 16]),
        };
        let mut original = pixel_layer("base", 4, 4, 40);
        original.mask = Some(mask);
        original.color = ColorLabel::Red;
        original.opacity = 200;
        let mut doc = doc_with(vec![original, pixel_layer("top", 4, 4, 80)]);

        let index = duplicate_layer(&mut doc, 0);
        assert_eq!(index, 1);
        assert_eq!(doc.layers.len(), 3);
        assert_eq!(doc.layers[1].name, "base copy");
        assert_eq!(doc.layers[1].mask, doc.layers[0].mask);
        assert_eq!(doc.layers[1].color, ColorLabel::Red);
        assert_eq!(doc.layers[1].opacity, 200);

        doc.layers[1].channels[0].data[0] = 7;
        assert_eq!(doc.layers[0].channels[0].data[0], 40, "deep copy");
    }

    #[test]
    fn duplicate_group_copies_children() {
        let mut group = empty_group("Group 1");
        group.children.push(pixel_layer("child", 4, 4, 30));
        let mut doc = doc_with(vec![group]);

        assert_eq!(duplicate_layer(&mut doc, 0), 1);
        assert_eq!(doc.layers[1].name, "Group 1 copy");
        assert!(doc.layers[1].is_group);
        assert_eq!(doc.layers[1].children.len(), 1);
        assert_eq!(doc.layers[1].children[0].name, "child");
    }

    #[test]
    fn duplicate_rejects_out_of_range() {
        let mut doc = doc_with(vec![pixel_layer("base", 4, 4, 1)]);
        assert_eq!(duplicate_layer(&mut doc, -1), -1);
        assert_eq!(duplicate_layer(&mut doc, 9), -1);
        assert_eq!(doc.layers.len(), 1);
    }

    #[test]
    fn group_layer_wraps_in_place() {
        let mut doc = doc_with(vec![
            pixel_layer("a", 4, 4, 1),
            pixel_layer("b", 4, 4, 2),
            pixel_layer("c", 4, 4, 3),
        ]);
        let index = group_layer(&mut doc, 1);

        assert_eq!(index, 1);
        assert_eq!(doc.layers.len(), 3);
        let group = &doc.layers[1];
        assert!(group.is_group);
        assert_eq!(group.name, "Group 1");
        assert_eq!(group.children.len(), 1);
        assert_eq!(group.children[0].name, "b");
        assert_eq!(doc.layers[0].name, "a");
        assert_eq!(doc.layers[2].name, "c");
    }

    #[test]
    fn group_layer_names_after_existing_groups() {
        let mut doc = doc_with(vec![pixel_layer("a", 4, 4, 1)]);
        group_layer(&mut doc, 0);
        // Wrap the new group again: the next name must skip the existing one.
        assert_eq!(doc.layers[0].name, "Group 1");
        doc.layers.push(pixel_layer("b", 4, 4, 2));
        group_layer(&mut doc, 1);
        assert_eq!(doc.layers[1].name, "Group 2");
    }

    #[test]
    fn ungroup_splices_children_in_order() {
        let mut group = empty_group("Group 1");
        group.children.push(pixel_layer("a", 4, 4, 1));
        group.children.push(pixel_layer("b", 4, 4, 2));
        let mut doc = doc_with(vec![
            pixel_layer("bottom", 4, 4, 0),
            group,
            pixel_layer("top", 4, 4, 3),
        ]);

        assert!(ungroup_layer(&mut doc, 1));
        assert_eq!(doc.layers.len(), 4);
        let names: Vec<&str> = doc.layers.iter().map(|l| l.name.as_str()).collect();
        assert_eq!(names, ["bottom", "a", "b", "top"]);
    }

    #[test]
    fn ungroup_rejects_non_group() {
        let mut doc = doc_with(vec![pixel_layer("a", 4, 4, 1)]);
        assert!(!ungroup_layer(&mut doc, 0));
        assert!(!ungroup_layer(&mut doc, -1));
        assert_eq!(doc.layers.len(), 1);
    }

    #[test]
    fn next_layer_name_uses_highest_suffix() {
        let mut doc = doc_with(vec![
            pixel_layer("Layer 1", 4, 4, 1),
            pixel_layer("Layer 5", 4, 4, 2),
            pixel_layer("Group 2", 4, 4, 3),
        ]);
        doc.layers[1].children.push(pixel_layer("Layer 9", 4, 4, 4));
        assert_eq!(next_layer_name(&doc, "Layer"), "Layer 10");
        assert_eq!(next_layer_name(&doc, "Group"), "Group 3");

        let empty = doc_with(Vec::new());
        assert_eq!(next_layer_name(&empty, "Layer"), "Layer 1");
    }

    #[test]
    fn transparent_layer_does_not_change_the_composite() {
        let mut doc = doc_with(vec![pixel_layer("base", 4, 4, 40)]);
        let before = crate::composite_rgba(&doc);

        let index = add_layer(&mut doc, 0, "Layer 1");
        assert!(index >= 0);
        let after = crate::composite_rgba(&doc);

        assert_eq!(before, after, "a fully transparent layer must be inert");
        assert!(after.data.iter().any(|&b| b != 0), "base layer is visible");
    }
}
