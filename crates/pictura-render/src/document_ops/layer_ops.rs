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

// ---------------------------------------------------------------------------
// M39 path/tree core. See `docs/dev/m39-panel-anatomy.md` §3.1–3.3.
// ---------------------------------------------------------------------------

/// Parse a frozen path (`"0"`, `"2/1"`, …) into bottom-first child indices.
///
/// Rejects the empty string, empty segments (leading/trailing/double `/`),
/// signs, non-digits, leading zeros, and overflow.
fn parse_path(path: &str) -> Option<Vec<usize>> {
    if path.is_empty() {
        return None;
    }
    let mut segments = Vec::new();
    for segment in path.split('/') {
        if segment.is_empty() || !segment.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        if segment.len() > 1 && segment.starts_with('0') {
            return None;
        }
        segments.push(segment.parse::<usize>().ok()?);
    }
    Some(segments)
}

/// The path of `path`'s containing node: `"2/1"` → `Some("2")`, `"0"` →
/// `None` (its container is the document). Malformed paths return `None`.
pub fn parent_path(path: &str) -> Option<&str> {
    parse_path(path)?;
    path.rsplit_once('/').map(|(parent, _)| parent)
}

/// Resolve a path to a layer, or `None` for a malformed or out-of-range path.
pub fn resolve_path<'a>(doc: &'a Document, path: &str) -> Option<&'a Layer> {
    let segments = parse_path(path)?;
    let mut layers: &[Layer] = &doc.layers;
    let mut node: Option<&Layer> = None;
    for &index in &segments {
        let layer = layers.get(index)?;
        node = Some(layer);
        layers = &layer.children;
    }
    node
}

/// Mutable [`resolve_path`].
pub fn resolve_path_mut<'a>(doc: &'a mut Document, path: &str) -> Option<&'a mut Layer> {
    let segments = parse_path(path)?;
    let (container, index) = container_mut(doc, &segments)?;
    container.get_mut(index)
}

/// Flatten the whole tree depth-first, topmost-first, as `(path, depth)` rows.
///
/// Each container is walked last index → 0 because `Layer.children` is
/// bottom-first; top-level rows are depth 0.
pub fn flatten_rows(doc: &Document) -> Vec<(String, u32)> {
    fn walk(layers: &[Layer], prefix: &str, depth: u32, rows: &mut Vec<(String, u32)>) {
        for (index, layer) in layers.iter().enumerate().rev() {
            let path = if prefix.is_empty() {
                index.to_string()
            } else {
                format!("{prefix}/{index}")
            };
            rows.push((path.clone(), depth));
            if layer.is_group {
                walk(&layer.children, &path, depth + 1, rows);
            }
        }
    }
    let mut rows = Vec::new();
    walk(&doc.layers, "", 0, &mut rows);
    rows
}

/// The M36 Background heuristic: top-level index 0, not a group, no adjustment
/// data, and named exactly `"Background"`.
pub fn is_background(doc: &Document, path: &str) -> bool {
    let Some(segments) = parse_path(path) else {
        return false;
    };
    if segments.len() != 1 || segments[0] != 0 {
        return false;
    }
    resolve_path(doc, path).is_some_and(|layer| {
        !layer.is_group && layer.adjustment.is_none() && layer.name == "Background"
    })
}

/// Resolve the mutable `children` vector holding the node at `segments` and the
/// node's index in it. `segments` must be non-empty.
fn container_mut<'a>(
    doc: &'a mut Document,
    segments: &[usize],
) -> Option<(&'a mut Vec<Layer>, usize)> {
    let (last, parent) = segments.split_last()?;
    let container = container_of_mut(doc, parent)?;
    if *last >= container.len() {
        return None;
    }
    Some((container, *last))
}

/// Resolve the mutable container named by a full parent path (empty = document).
fn container_of_mut<'a>(doc: &'a mut Document, parent: &[usize]) -> Option<&'a mut Vec<Layer>> {
    let mut layers: &'a mut Vec<Layer> = &mut doc.layers;
    for &index in parent {
        if index >= layers.len() {
            return None;
        }
        layers = &mut layers[index].children;
    }
    Some(layers)
}

fn format_segments(segments: &[usize]) -> String {
    segments
        .iter()
        .map(|segment| segment.to_string())
        .collect::<Vec<_>>()
        .join("/")
}

/// Deduplicate a path list, preserving first-seen order.
fn unique<'a>(paths: &[&'a str]) -> Vec<&'a str> {
    let mut seen = std::collections::HashSet::new();
    let mut kept = Vec::new();
    for &path in paths {
        if seen.insert(path) {
            kept.push(path);
        }
    }
    kept
}

/// Structural selection: dedup, keep only paths that resolve, optionally drop a
/// path whose ancestor is also selected, and sort so descendants and
/// higher-index siblings come first. That order makes child-before-parent
/// removal safe (an index is never invalidated by a later edit).
fn selected_paths(
    doc: &Document,
    paths: &[&str],
    drop_descendants: bool,
) -> Vec<(Vec<usize>, String)> {
    let mut selected: Vec<(Vec<usize>, String)> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for &path in paths {
        let Some(segments) = parse_path(path) else {
            continue;
        };
        if !seen.insert(segments.clone()) || resolve_path(doc, path).is_none() {
            continue;
        }
        selected.push((segments, path.to_string()));
    }
    if drop_descendants {
        let snapshot: Vec<Vec<usize>> = selected.iter().map(|(s, _)| s.clone()).collect();
        selected.retain(|(segments, _)| {
            !snapshot
                .iter()
                .any(|other| other.len() < segments.len() && segments.starts_with(other))
        });
    }
    selected.sort_by(|a, b| b.0.iter().rev().cmp(a.0.iter().rev()));
    selected
}

/// Apply `update` to every distinct, resolving path for which `eligible` holds;
/// returns how many nodes actually changed.
fn edit_paths(
    doc: &mut Document,
    paths: &[&str],
    eligible: impl Fn(&Document, &str, &Layer) -> bool,
    update: impl Fn(&mut Layer) -> bool,
) -> usize {
    let mut changed = 0;
    for path in unique(paths) {
        let Some(layer) = resolve_path(doc, path) else {
            continue;
        };
        if !eligible(doc, path, layer) {
            continue;
        }
        if let Some(layer) = resolve_path_mut(doc, path) {
            if update(layer) {
                changed += 1;
            }
        }
    }
    changed
}

/// Set the eye on every listed path (the Background included). Returns the
/// number of nodes changed.
pub fn set_visible_paths(doc: &mut Document, paths: &[&str], visible: bool) -> usize {
    edit_paths(
        doc,
        paths,
        |_, _, _| true,
        |layer| {
            if layer.visible == visible {
                false
            } else {
                layer.visible = visible;
                true
            }
        },
    )
}

/// Solo visibility primitive: set exactly the listed paths visible and every
/// other node in the tree hidden. It is not additive, ancestors are not
/// implied (the caller passes them), an unresolvable path contributes nothing,
/// and a duplicate path counts once. Returns the number of nodes changed.
pub fn apply_visibility(doc: &mut Document, paths: &[&str]) -> usize {
    let visible: std::collections::HashSet<String> = unique(paths)
        .into_iter()
        .filter(|path| resolve_path(doc, path).is_some())
        .map(str::to_string)
        .collect();
    fn walk(
        layers: &mut [Layer],
        prefix: &str,
        visible: &std::collections::HashSet<String>,
    ) -> usize {
        let mut changed = 0;
        for (index, layer) in layers.iter_mut().enumerate() {
            let path = if prefix.is_empty() {
                index.to_string()
            } else {
                format!("{prefix}/{index}")
            };
            let want = visible.contains(&path);
            if layer.visible != want {
                layer.visible = want;
                changed += 1;
            }
            changed += walk(&mut layer.children, &path, visible);
        }
        changed
    }
    walk(&mut doc.layers, "", &visible)
}

/// Set the blend mode. Skips the Background and fully-locked nodes; groups are
/// eligible. Returns the number of nodes changed.
pub fn set_blend_paths(doc: &mut Document, paths: &[&str], mode: BlendMode) -> usize {
    edit_paths(
        doc,
        paths,
        |doc, path, layer| !is_background(doc, path) && !layer.lock.is_all(),
        |layer| {
            if layer.blend == mode {
                false
            } else {
                layer.blend = mode;
                true
            }
        },
    )
}

/// Set opacity. Skips the Background and fully-locked nodes; groups are
/// eligible. Returns the number of nodes changed.
pub fn set_opacity_paths(doc: &mut Document, paths: &[&str], value: u8) -> usize {
    edit_paths(
        doc,
        paths,
        |doc, path, layer| !is_background(doc, path) && !layer.lock.is_all(),
        |layer| {
            if layer.opacity == value {
                false
            } else {
                layer.opacity = value;
                true
            }
        },
    )
}

/// Set fill opacity. Skips the Background, fully-locked nodes, and groups (a
/// group has no Fill). Returns the number of nodes changed.
pub fn set_fill_paths(doc: &mut Document, paths: &[&str], value: u8) -> usize {
    edit_paths(
        doc,
        paths,
        |doc, path, layer| !is_background(doc, path) && !layer.lock.is_all() && !layer.is_group,
        |layer| {
            if layer.fill == value {
                false
            } else {
                layer.fill = value;
                true
            }
        },
    )
}

/// Set (`on`) or clear a lock bit. Skips the Background; a fully-locked node is
/// still eligible (that is how a lock is released). Returns the number changed.
pub fn set_lock_paths(doc: &mut Document, paths: &[&str], flag: u8, on: bool) -> usize {
    edit_paths(
        doc,
        paths,
        |doc, path, _| !is_background(doc, path),
        |layer| {
            let next = layer.lock.with(flag, on);
            if next == layer.lock {
                false
            } else {
                layer.lock = next;
                true
            }
        },
    )
}

/// Set the color label. Skips the Background; fully-locked nodes and groups are
/// eligible. Returns the number of nodes changed.
pub fn set_color_paths(doc: &mut Document, paths: &[&str], color: ColorLabel) -> usize {
    edit_paths(
        doc,
        paths,
        |doc, path, _| !is_background(doc, path),
        |layer| {
            if layer.color == color {
                false
            } else {
                layer.color = color;
                true
            }
        },
    )
}

/// Delete every listed node. Skips the Background and fully-locked nodes; a
/// selected node whose ancestor is also selected is deleted once (with its
/// ancestor). Returns the number of deletions performed.
pub fn delete_paths(doc: &mut Document, paths: &[&str]) -> usize {
    let selected = selected_paths(doc, paths, true);
    let mut changed = 0;
    for (segments, path) in selected {
        if is_background(doc, &path) {
            continue;
        }
        if resolve_path(doc, &path).is_some_and(|layer| layer.lock.is_all()) {
            continue;
        }
        if let Some((container, index)) = container_mut(doc, &segments) {
            container.remove(index);
            changed += 1;
        }
    }
    changed
}

/// Deep-clone every listed node directly above itself, naming each copy
/// `"<name> copy"`. Eligible everywhere (Background and locked included).
/// Returns the new paths.
pub fn duplicate_paths(doc: &mut Document, paths: &[&str]) -> Vec<String> {
    let selected = selected_paths(doc, paths, true);
    let mut created = Vec::new();
    for (segments, _) in selected {
        let Some((container, index)) = container_mut(doc, &segments) else {
            continue;
        };
        let mut copy = container[index].clone();
        copy.name = format!("{} copy", copy.name);
        container.insert(index + 1, copy);
        let mut new_segments = segments;
        if let Some(last) = new_segments.last_mut() {
            *last = index + 1;
        }
        created.push(format_segments(&new_segments));
    }
    created
}

/// Wrap the selection in one new group inserted at the topmost selected node's
/// position. Refuses the whole operation (`None`) when the paths span more than
/// one container, or any is the Background or fully locked. Returns the new
/// group's path.
pub fn group_paths(doc: &mut Document, paths: &[&str]) -> Option<String> {
    let selected = selected_paths(doc, paths, true);
    let (first, _) = selected.first()?;
    let parent = first[..first.len() - 1].to_vec();
    if selected
        .iter()
        .any(|(segments, _)| segments[..segments.len() - 1] != parent[..])
    {
        return None;
    }
    for (_, path) in &selected {
        if is_background(doc, path)
            || resolve_path(doc, path).is_some_and(|layer| layer.lock.is_all())
        {
            return None;
        }
    }
    let name = next_layer_name(doc, "Group");
    let top = selected
        .iter()
        .map(|(segments, _)| *segments.last().expect("non-empty"))
        .max()
        .expect("selection is non-empty");
    let removed_below = selected
        .iter()
        .filter(|(segments, _)| *segments.last().expect("non-empty") < top)
        .count();
    let insert_at = top - removed_below;

    let mut selected = selected;
    selected.sort_by(|a, b| b.0.last().cmp(&a.0.last()));
    let mut children = Vec::new();
    for (segments, _) in selected {
        let (container, index) = container_mut(doc, &segments)?;
        children.push(container.remove(index));
    }
    children.reverse();

    let mut group = empty_group(&name);
    group.children = children;
    let container = container_of_mut(doc, &parent)?;
    let at = insert_at.min(container.len());
    container.insert(at, group);
    let mut new_segments = parent;
    new_segments.push(at);
    Some(format_segments(&new_segments))
}

/// Splice each listed group's children into its container. Skips non-groups,
/// the Background, and fully-locked nodes. Returns the number of groups changed.
pub fn ungroup_paths(doc: &mut Document, paths: &[&str]) -> usize {
    let selected = selected_paths(doc, paths, false);
    let mut changed = 0;
    for (segments, path) in selected {
        let Some(layer) = resolve_path(doc, &path) else {
            continue;
        };
        if !layer.is_group || layer.lock.is_all() {
            continue;
        }
        let Some((container, index)) = container_mut(doc, &segments) else {
            continue;
        };
        let group = container.remove(index);
        for (offset, child) in group.children.into_iter().enumerate() {
            container.insert(index + offset, child);
        }
        changed += 1;
    }
    changed
}

/// Rename the node at `path`. No refusal rule; returns false for a path that
/// does not resolve.
pub fn rename_path(doc: &mut Document, path: &str, name: &str) -> bool {
    match resolve_path_mut(doc, path) {
        Some(layer) => {
            layer.name = name.to_string();
            true
        }
        None => false,
    }
}

/// Move the node `delta` places within its own container, clamped to the
/// container bounds. Refuses the Background and fully-locked nodes (LAY-002:
/// they do not reorder), and returns false (state unchanged) at a boundary or
/// for a path that does not resolve.
pub fn move_path(doc: &mut Document, path: &str, delta: i32) -> bool {
    let Some(segments) = parse_path(path) else {
        return false;
    };
    if is_background(doc, path) || resolve_path(doc, path).is_some_and(|layer| layer.lock.is_all())
    {
        return false;
    }
    let Some((container, index)) = container_mut(doc, &segments) else {
        return false;
    };
    let last = container.len() as i32 - 1;
    let target = (index as i32 + delta).clamp(0, last) as usize;
    if target == index {
        return false;
    }
    let layer = container.remove(index);
    container.insert(target, layer);
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

    fn locked_layer(name: &str) -> Layer {
        let mut layer = pixel_layer(name, 4, 4, 1);
        layer.lock = LockFlags::all();
        layer
    }

    fn sample_doc() -> Document {
        let mut group = empty_group("Group 1");
        group.children.push(pixel_layer("child", 4, 4, 2));
        doc_with(vec![
            pixel_layer("Background", 4, 4, 0),
            locked_layer("locked"),
            group,
            pixel_layer("normal", 4, 4, 3),
        ])
    }

    fn visible_names(doc: &Document) -> Vec<String> {
        fn walk(layers: &[Layer], out: &mut Vec<String>) {
            for layer in layers {
                if layer.visible {
                    out.push(layer.name.clone());
                }
                walk(&layer.children, out);
            }
        }
        let mut out = Vec::new();
        walk(&doc.layers, &mut out);
        out
    }

    #[test]
    fn path_resolve_nested() {
        let mut group = empty_group("Group 1");
        group.children.push(pixel_layer("a", 4, 4, 1));
        group.children.push(pixel_layer("b", 4, 4, 2));
        let mut doc = doc_with(vec![pixel_layer("bottom", 4, 4, 0), group]);

        assert_eq!(resolve_path(&doc, "0").unwrap().name, "bottom");
        assert_eq!(resolve_path(&doc, "1/0").unwrap().name, "a");
        assert_eq!(resolve_path(&doc, "1/1").unwrap().name, "b");
        assert!(resolve_path(&doc, "1/2").is_none(), "out of range child");
        assert_eq!(parent_path("1/1"), Some("1"));
        assert_eq!(parent_path("0"), None);

        resolve_path_mut(&mut doc, "1/0").unwrap().opacity = 100;
        assert_eq!(doc.layers[1].children[0].opacity, 100);
    }

    #[test]
    fn path_resolve_rejects() {
        let mut group = empty_group("Group 1");
        group.children.push(pixel_layer("a", 4, 4, 1));
        let mut doc = doc_with(vec![pixel_layer("bottom", 4, 4, 0), group]);

        for bad in [
            "", "01", "-1", "1/", "/1", "1//2", "1/a", "+1", " 1", "1/999", "999",
        ] {
            assert!(resolve_path(&doc, bad).is_none(), "resolve {bad:?}");
            assert!(
                resolve_path_mut(&mut doc, bad).is_none(),
                "resolve_mut {bad:?}"
            );
        }
        assert_eq!(parent_path(""), None);
        assert_eq!(parent_path("01"), None);
    }

    #[test]
    fn flatten_rows_is_topmost_first() {
        let mut group = empty_group("Group 1");
        group.children.push(pixel_layer("c0", 4, 4, 1));
        group.children.push(pixel_layer("c1", 4, 4, 2));
        let doc = doc_with(vec![
            pixel_layer("bottom", 4, 4, 0),
            group,
            pixel_layer("top", 4, 4, 3),
        ]);

        let rows = flatten_rows(&doc);
        let expected: Vec<(String, u32)> = vec![
            ("2".into(), 0),
            ("1".into(), 0),
            ("1/1".into(), 1),
            ("1/0".into(), 1),
            ("0".into(), 0),
        ];
        assert_eq!(rows, expected);
    }

    #[test]
    fn is_background_uses_m36_heuristic() {
        let doc = doc_with(vec![
            pixel_layer("Background", 4, 4, 0),
            pixel_layer("Layer 1", 4, 4, 1),
        ]);
        assert!(is_background(&doc, "0"));
        assert!(!is_background(&doc, "1"));
        assert!(!is_background(&doc, "9"), "missing path");

        let named_group = doc_with(vec![empty_group("Background")]);
        assert!(
            !is_background(&named_group, "0"),
            "a group is not Background"
        );

        let mut group = empty_group("Group 1");
        group.children.push(pixel_layer("Background", 4, 4, 0));
        let nested = doc_with(vec![group]);
        assert!(!is_background(&nested, "0/0"), "nested is not top-level 0");

        let mut adjusted = pixel_layer("Background", 4, 4, 0);
        adjusted.adjustment = Some(crate::encode_invert());
        let adjusted = doc_with(vec![adjusted]);
        assert!(
            !is_background(&adjusted, "0"),
            "an adjustment is not Background"
        );
    }

    #[test]
    fn visibility_applies_everywhere() {
        let mut doc = sample_doc();
        assert_eq!(set_visible_paths(&mut doc, &["0", "1", "2", "3"], false), 4);
        assert!(doc.layers.iter().all(|layer| !layer.visible));
        assert_eq!(set_visible_paths(&mut doc, &["0", "1", "2", "3"], false), 0);
    }

    #[test]
    fn blend_and_opacity_skip_background_and_locked() {
        let mut doc = sample_doc();
        assert_eq!(
            set_blend_paths(&mut doc, &["0", "1", "2", "3"], BlendMode::Multiply),
            2
        );
        assert_eq!(doc.layers[0].blend, BlendMode::Normal, "Background skipped");
        assert_eq!(doc.layers[1].blend, BlendMode::Normal, "locked skipped");
        assert_eq!(doc.layers[2].blend, BlendMode::Multiply, "group applies");
        assert_eq!(doc.layers[3].blend, BlendMode::Multiply);

        let mut doc = sample_doc();
        assert_eq!(set_opacity_paths(&mut doc, &["0", "1", "2", "3"], 100), 2);
        assert_eq!(doc.layers[0].opacity, 255);
        assert_eq!(doc.layers[1].opacity, 255);
        assert_eq!(doc.layers[2].opacity, 100);
        assert_eq!(doc.layers[3].opacity, 100);
    }

    #[test]
    fn fill_skips_groups() {
        let mut doc = sample_doc();
        assert_eq!(set_fill_paths(&mut doc, &["0", "1", "2", "3"], 100), 1);
        assert_eq!(doc.layers[0].fill, 255, "Background skipped");
        assert_eq!(doc.layers[1].fill, 255, "locked skipped");
        assert_eq!(doc.layers[2].fill, 255, "group has no Fill");
        assert_eq!(doc.layers[3].fill, 100);
    }

    #[test]
    fn lock_skips_background_but_applies_to_locked() {
        let mut doc = sample_doc();
        assert_eq!(
            set_lock_paths(&mut doc, &["0", "1"], LockFlags::PIXELS, false),
            1
        );
        assert_eq!(doc.layers[0].lock.bits(), 0, "Background lock untouched");
        assert!(!doc.layers[1].lock.contains(LockFlags::PIXELS));
    }

    #[test]
    fn color_skips_background_applies_to_locked() {
        let mut doc = sample_doc();
        assert_eq!(
            set_color_paths(&mut doc, &["0", "1", "3"], ColorLabel::Red),
            2
        );
        assert_eq!(doc.layers[0].color, ColorLabel::None, "Background skipped");
        assert_eq!(doc.layers[1].color, ColorLabel::Red, "locked applies");
        assert_eq!(doc.layers[3].color, ColorLabel::Red);
    }

    #[test]
    fn delete_skips_background_and_locked() {
        let mut doc = sample_doc();
        assert_eq!(delete_paths(&mut doc, &["0", "1"]), 0);
        assert_eq!(doc.layers.len(), 4);
        assert_eq!(delete_paths(&mut doc, &["2", "3"]), 2);
        assert_eq!(doc.layers.len(), 2);
    }

    #[test]
    fn duplicate_applies_everywhere() {
        let mut doc = sample_doc();
        let created = duplicate_paths(&mut doc, &["0", "1"]);
        assert_eq!(created.len(), 2, "Background and locked both duplicate");
        let names: Vec<&str> = doc.layers.iter().map(|layer| layer.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "Background",
                "Background copy",
                "locked",
                "locked copy",
                "Group 1",
                "normal"
            ]
        );
    }

    #[test]
    fn ungroup_skips_non_groups() {
        let mut doc = sample_doc();
        assert_eq!(ungroup_paths(&mut doc, &["0", "3"]), 0);
        assert_eq!(ungroup_paths(&mut doc, &["2"]), 1);
        assert_eq!(doc.layers.len(), 4);
        assert_eq!(doc.layers[2].name, "child");
    }

    #[test]
    fn group_paths_refuses_cross_container_and_succeeds_within_one() {
        let mut doc = sample_doc();
        assert_eq!(
            group_paths(&mut doc, &["0", "2/0"]),
            None,
            "cross container"
        );
        assert_eq!(
            group_paths(&mut doc, &["0", "3"]),
            None,
            "Background refuses"
        );
        assert_eq!(group_paths(&mut doc, &["1", "3"]), None, "locked refuses");

        let mut doc = doc_with(vec![
            pixel_layer("a", 4, 4, 0),
            pixel_layer("b", 4, 4, 1),
            pixel_layer("c", 4, 4, 2),
        ]);
        let path = group_paths(&mut doc, &["0", "2"]).unwrap();
        assert_eq!(path, "1");
        assert_eq!(doc.layers.len(), 2);
        assert!(doc.layers[1].is_group);
        let children: Vec<&str> = doc.layers[1]
            .children
            .iter()
            .map(|layer| layer.name.as_str())
            .collect();
        assert_eq!(children, ["a", "c"]);
        assert_eq!(doc.layers[0].name, "b");
    }

    #[test]
    fn apply_visibility_is_solo() {
        let mut group = empty_group("Group 1");
        group.children.push(pixel_layer("c", 4, 4, 1));
        group.children.push(pixel_layer("d", 4, 4, 2));
        let mut doc = doc_with(vec![
            pixel_layer("a", 4, 4, 0),
            pixel_layer("b", 4, 4, 3),
            group,
        ]);

        assert_eq!(apply_visibility(&mut doc, &["1"]), 4);
        assert_eq!(visible_names(&doc), vec!["b"]);
        assert!(!doc.layers[2].visible);
        assert!(doc.layers[2].children.iter().all(|child| !child.visible));

        assert_eq!(apply_visibility(&mut doc, &["2", "2/0"]), 3);
        assert_eq!(visible_names(&doc), vec!["Group 1", "c"]);
    }

    #[test]
    fn delete_paths_nested_pair_deletes_once() {
        let mut group = empty_group("Group 1");
        group.children.push(pixel_layer("child", 4, 4, 1));
        let mut doc = doc_with(vec![group]);

        assert_eq!(delete_paths(&mut doc, &["0", "0/0"]), 1);
        assert!(doc.layers.is_empty());
    }

    #[test]
    fn duplicate_paths_returns_new_paths() {
        let mut group = empty_group("Group 1");
        group.children.push(pixel_layer("child", 4, 4, 1));
        let mut doc = doc_with(vec![pixel_layer("bottom", 4, 4, 0), group]);

        let created = duplicate_paths(&mut doc, &["0", "1"]);
        assert_eq!(created, vec!["2".to_string(), "1".to_string()]);
        let names: Vec<&str> = doc.layers.iter().map(|layer| layer.name.as_str()).collect();
        assert_eq!(names, ["bottom", "bottom copy", "Group 1", "Group 1 copy"]);
        assert_eq!(doc.layers[3].children.len(), 1, "deep copy");
    }

    #[test]
    fn add_layer_in_inside_group_and_on_top() {
        let mut group = empty_group("Group 1");
        group.children.push(pixel_layer("child", 4, 4, 1));
        let mut doc = doc_with(vec![group]);

        let path = add_layer_in(&mut doc, "0", "Inside");
        assert_eq!(path, "0/1");
        assert_eq!(doc.layers[0].children.len(), 2);
        assert_eq!(doc.layers[0].children[1].name, "Inside");

        let path = add_group_in(&mut doc, "", "New Group");
        assert_eq!(path, "1", "empty selection lands on top");
        assert!(doc.layers[1].is_group);

        let path = add_layer_in(&mut doc, "99", "Top");
        assert_eq!(path, "2", "missing selection lands on top");
        assert_eq!(doc.layers[2].name, "Top");
    }

    #[test]
    fn move_path_clamps_and_rename_resolves() {
        let mut doc = doc_with(vec![pixel_layer("a", 4, 4, 0), pixel_layer("b", 4, 4, 1)]);
        assert!(!move_path(&mut doc, "0", -1), "boundary");
        assert!(move_path(&mut doc, "0", 1));
        assert_eq!(doc.layers[0].name, "b");
        assert_eq!(doc.layers[1].name, "a");
        assert!(!move_path(&mut doc, "1", 5), "clamped to itself");
        assert!(!move_path(&mut doc, "9", 1), "missing path");

        assert!(rename_path(&mut doc, "1", "renamed"));
        assert_eq!(doc.layers[1].name, "renamed");
        assert!(!rename_path(&mut doc, "9", "x"));
    }

    #[test]
    fn move_path_refuses_background_and_locked() {
        let mut doc = doc_with(vec![
            pixel_layer("Background", 4, 4, 0),
            locked_layer("locked"),
            pixel_layer("normal", 4, 4, 3),
        ]);

        assert!(!move_path(&mut doc, "0", 1), "Background refuses");
        assert_eq!(doc.layers[0].name, "Background", "Background unchanged");
        assert_eq!(doc.layers[1].name, "locked");
        assert!(!move_path(&mut doc, "1", -1), "fully locked refuses");
        assert_eq!(doc.layers[0].name, "Background", "locked state unchanged");
        assert_eq!(doc.layers[1].name, "locked");

        assert!(move_path(&mut doc, "2", -1), "ordinary layer moves");
        assert_eq!(doc.layers[1].name, "normal");
        assert_eq!(doc.layers[2].name, "locked");
    }
}
