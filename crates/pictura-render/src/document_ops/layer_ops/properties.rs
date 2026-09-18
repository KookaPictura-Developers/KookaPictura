use pictura_core::{BlendMode, ColorLabel, Document, Layer, LockFlags};

use super::create::{empty_group, next_layer_name};
use super::paths::{
    container_mut, container_of_mut, edit_paths, format_segments, is_background, parse_path,
    resolve_path, resolve_path_mut, selected_paths, unique,
};

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
/// one container, or any is the Background, fully locked, or nesting locked.
/// Returns the new group's path.
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
            || resolve_path(doc, path)
                .is_some_and(|layer| layer.lock.is_all() || layer.lock.contains(LockFlags::NESTING))
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
/// the Background, fully-locked nodes, and nesting-locked nodes. Returns the
/// number of groups changed.
pub fn ungroup_paths(doc: &mut Document, paths: &[&str]) -> usize {
    let selected = selected_paths(doc, paths, false);
    let mut changed = 0;
    for (segments, path) in selected {
        let Some(layer) = resolve_path(doc, &path) else {
            continue;
        };
        if !layer.is_group || layer.lock.is_all() || layer.lock.contains(LockFlags::NESTING) {
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
