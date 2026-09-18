use pictura_core::{BlendMode, Channel, ColorLabel, Document, Layer, LockFlags, PsdRect};

use super::paths::{container_mut, container_of_mut, format_segments, parse_path, resolve_path};

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
pub(super) fn transparent_layer(width: u32, height: u32, name: &str) -> Layer {
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
pub(super) fn empty_group(name: &str) -> Layer {
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

/// Insert `node` inside `selection_path` when it is a group (as its top child),
/// otherwise directly above the selected node in its container, otherwise on
/// top of the document. Returns the new path, or empty on failure.
fn insert_node(doc: &mut Document, selection_path: &str, node: Layer) -> String {
    if let Some(segments) = parse_path(selection_path) {
        if resolve_path(doc, selection_path).is_some_and(|layer| layer.is_group) {
            let Some(container) = container_of_mut(doc, &segments) else {
                return String::new();
            };
            let index = container.len();
            container.push(node);
            let mut new_segments = segments;
            new_segments.push(index);
            return format_segments(&new_segments);
        }
        if let Some((container, index)) = container_mut(doc, &segments) {
            let at = insertion_index(container.len(), index as i32);
            container.insert(at, node);
            let mut new_segments = segments;
            if let Some(last) = new_segments.last_mut() {
                *last = at;
            }
            return format_segments(&new_segments);
        }
    }
    let at = insertion_index(doc.layers.len(), -1);
    doc.layers.insert(at, node);
    format_segments(&[at])
}

/// Insert a transparent raster layer for `name` (generated when empty) using
/// the tree-aware [`insert_node`] rule. Returns the new path, or empty for a
/// zero-dimension document.
pub fn add_layer_in(doc: &mut Document, selection_path: &str, name: &str) -> String {
    if doc.width == 0 || doc.height == 0 {
        return String::new();
    }
    let name = if name.is_empty() {
        next_layer_name(doc, "Layer")
    } else {
        name.to_string()
    };
    let layer = transparent_layer(doc.width, doc.height, &name);
    insert_node(doc, selection_path, layer)
}

/// Insert an empty group for `name` (generated when empty) using the
/// tree-aware [`insert_node`] rule. Returns the new path.
pub fn add_group_in(doc: &mut Document, selection_path: &str, name: &str) -> String {
    let name = if name.is_empty() {
        next_layer_name(doc, "Group")
    } else {
        name.to_string()
    };
    insert_node(doc, selection_path, empty_group(&name))
}
