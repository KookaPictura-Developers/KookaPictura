use pictura_core::{BlendMode, ColorLabel, Document, Layer, LockFlags};

use super::create::{empty_group, next_layer_name};
use super::merge::is_visible_in_panel;
use super::paths::{
    container_mut, container_of_mut, edit_paths, flatten_rows, format_segments, is_background,
    parse_path, resolve_path, resolve_path_mut, selected_paths, unique,
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

/// Delete every layer whose effective visibility is off (its own eye and every
/// ancestor's), counting a hidden group and its nested hidden descendants as
/// one removal. Returns the number of nodes removed; a tree with no hidden
/// layer is left untouched. *(match inferred: CS6 does not state the nested
/// count or the Background/locked exemptions inherited from [`delete_paths`].)*
pub fn delete_hidden_layers(doc: &mut Document) -> usize {
    let hidden: Vec<String> = flatten_rows(doc)
        .into_iter()
        .filter(|(path, _)| !is_visible_in_panel(doc, path))
        .map(|(path, _)| path)
        .collect();
    if hidden.is_empty() {
        return 0;
    }
    let refs: Vec<&str> = hidden.iter().map(String::as_str).collect();
    delete_paths(doc, &refs)
}

/// The paths of every layer matching the layer at `path`, in panel
/// (depth-first, topmost-first) order and excluding the reference itself.
///
/// The match key is inferred (CS6's exact "similar" attributes are unsourced):
/// the same node class (group, adjustment, background, or pixel), for
/// adjustments the same PSD adjustment key, and the same blend mode. An
/// unknown path matches nothing.
pub fn select_similar(doc: &Document, path: &str) -> Vec<String> {
    let Some(reference) = resolve_path(doc, path) else {
        return Vec::new();
    };
    let key = match_key(reference);
    flatten_rows(doc)
        .into_iter()
        .filter(|(candidate, _)| candidate != path)
        .filter(|(candidate, _)| {
            resolve_path(doc, candidate).is_some_and(|layer| match_key(layer) == key)
        })
        .map(|(candidate, _)| candidate)
        .collect()
}

/// The [`select_similar`] match key: `(class, adjustment key, blend mode)`.
fn match_key(layer: &Layer) -> (u8, [u8; 4], BlendMode) {
    let class = if layer.is_group {
        0
    } else if layer.adjustment.is_some() {
        1
    } else if layer.background {
        2
    } else {
        3
    };
    let adjustment = layer.adjustment.as_ref().map_or([0; 4], |data| data.key);
    (class, adjustment, layer.blend)
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

/// Rename the node at `path`. The Background flag follows the PSD name+position
/// convention in-session: the bottom top-level non-group renamed to
/// `"Background"` becomes the Background, any other rename clears the flag.
/// Returns false for a path that does not resolve.
pub fn rename_path(doc: &mut Document, path: &str, name: &str) -> bool {
    let Some(segments) = parse_path(path) else {
        return false;
    };
    match resolve_path_mut(doc, path) {
        Some(layer) => {
            layer.name = name.to_string();
            layer.background =
                segments.len() == 1 && segments[0] == 0 && !layer.is_group && name == "Background";
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

/// Whether the container at `parent` still resolves after the node at
/// (`src_parent`, `src_index`) is removed. Mirrors the post-removal
/// `container_of_mut` lookup in [`move_path_to`], so the dry run and the
/// mutation agree on unknown-target refusals.
fn container_exists_after_removal(
    doc: &Document,
    parent: &[usize],
    src_parent: &[usize],
    src_index: usize,
) -> bool {
    let mut layers = &doc.layers;
    for depth in 0..parent.len() {
        let index = if &parent[..depth] == src_parent && parent[depth] >= src_index {
            parent[depth] + 1
        } else {
            parent[depth]
        };
        let Some(layer) = layers.get(index) else {
            return false;
        };
        layers = &layer.children;
    }
    true
}

/// Resolve where a `move_path_to` candidate would land: the destination parent
/// container and the insertion index (`None` = append). `None` means the move is
/// refused. `move_path_to` (mutation) and `can_move_path_to` (dry run) both
/// route through this, so the two can never diverge.
fn move_path_to_dest(
    doc: &Document,
    path: &str,
    target: &str,
    mode: i32,
) -> Option<(Vec<usize>, Option<usize>)> {
    let src = parse_path(path)?;
    if is_background(doc, path) {
        return None;
    }
    let layer = resolve_path(doc, path)?;
    if layer.lock.is_all() || layer.lock.contains(LockFlags::NESTING) {
        return None;
    }
    if !target.is_empty() && (target == path || target.starts_with(&format!("{path}/"))) {
        return None;
    }
    if target.is_empty() {
        return Some((Vec::new(), None));
    }
    let tgt = parse_path(target)?;
    let (dest_parent, dest_index) = if mode == 2 {
        if !resolve_path(doc, target).is_some_and(|layer| layer.is_group) {
            return None;
        }
        (tgt, None)
    } else {
        let (last, parent) = tgt.split_last()?;
        (parent.to_vec(), Some(last + usize::from(mode == 0)))
    };
    let src_parent = &src[..src.len() - 1];
    let src_index = *src.last().expect("non-empty");
    if !container_exists_after_removal(doc, &dest_parent, src_parent, src_index) {
        return None;
    }
    Some((dest_parent, dest_index))
}

/// Dry-run form of [`move_path_to`]: whether the move would be accepted. Performs
/// the same guards and never mutates the document.
pub fn can_move_path_to(doc: &Document, path: &str, target: &str, mode: i32) -> bool {
    move_path_to_dest(doc, path, target, mode).is_some()
}

/// Move the node at `path` relative to `target`: mode `0` = above, `1` = below,
/// `2` = into `target` (which must be a group). An empty target means the top of
/// the document. Refuses the Background, a fully- or nesting-locked source, a
/// malformed/unknown target, a drop onto the source or into its own descendant,
/// and an `Into` target that is not a group.
pub fn move_path_to(doc: &mut Document, path: &str, target: &str, mode: i32) -> bool {
    let Some(src) = parse_path(path) else {
        return false;
    };
    let Some((dest_parent, dest_index)) = move_path_to_dest(doc, path, target, mode) else {
        return false;
    };

    let src_parent = src[..src.len() - 1].to_vec();
    let src_index = *src.last().expect("non-empty");
    let removed = {
        let Some((container, index)) = container_mut(doc, &src) else {
            return false;
        };
        container.remove(index)
    };

    let mut dest_index = dest_index;
    if let Some(index) = dest_index.as_mut() {
        if dest_parent == src_parent && *index > src_index {
            *index -= 1;
        }
    }

    let Some(container) = container_of_mut(doc, &dest_parent) else {
        // Unreachable after the guards; re-insert rather than drop the node.
        if let Some(container) = container_of_mut(doc, &src_parent) {
            container.insert(src_index.min(container.len()), removed);
        }
        return false;
    };
    let at = dest_index.map_or(container.len(), |index| index.min(container.len()));
    container.insert(at, removed);
    true
}
