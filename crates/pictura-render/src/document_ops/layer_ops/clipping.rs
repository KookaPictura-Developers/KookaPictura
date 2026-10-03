//! Layer > Create / Release Clipping Mask, and the Layers panel's Alt-click on
//! the line between two layers. A clipped layer shows only where the layer
//! under it (its base) has content; a run of clipped layers shares one base.
//! The compositor already honours [`Layer::clipping`]; these edit the flag.
//! Ported from photorust's `core/src/compositor.rs` clipping rules.

use pictura_core::{Document, Layer};

use super::paths::{container_of, edit_paths, is_background, parse_path, resolve_path_mut};

/// Whether `path` can clip to the layer below it: not the bottom of its
/// container (nothing to clip to), not the Background, not already clipped.
pub fn can_create_clipping_mask(doc: &Document, path: &str) -> bool {
    parse_path(path).is_some_and(|s| s.last().is_some_and(|&i| i > 0))
        && !is_background(doc, path)
        && super::paths::resolve_path(doc, path).is_some_and(|l| !l.clipping)
}

/// Clip each of `paths` to the layer below it. Returns how many changed.
pub fn create_clipping_mask(doc: &mut Document, paths: &[&str]) -> usize {
    edit_paths(
        doc,
        paths,
        |doc, path, _| can_create_clipping_mask(doc, path),
        |layer: &mut Layer| {
            layer.clipping = true;
            true
        },
    )
}

/// The sibling indices Release Clipping Mask frees for `path`: a clipped
/// layer and every clipped layer above it in its run; a base, every layer
/// clipped to it. Empty when there is nothing to release.
fn release_set(doc: &Document, path: &str) -> Vec<String> {
    let Some(segments) = parse_path(path) else {
        return Vec::new();
    };
    let Some((&index, parent)) = segments.split_last() else {
        return Vec::new();
    };
    let Some(siblings) = container_of(doc, parent) else {
        return Vec::new();
    };
    let Some(layer) = siblings.get(index) else {
        return Vec::new();
    };
    let start = if layer.clipping { index } else { index + 1 };
    let prefix = parent
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join("/");
    (start..siblings.len())
        .take_while(|&i| siblings[i].clipping)
        .map(|i| {
            if prefix.is_empty() {
                i.to_string()
            } else {
                format!("{prefix}/{i}")
            }
        })
        .collect()
}

/// Whether Release Clipping Mask would free anything for `path`.
pub fn can_release_clipping_mask(doc: &Document, path: &str) -> bool {
    !release_set(doc, path).is_empty()
}

/// Release Clipping Mask for each of `paths`. Returns how many layers changed.
pub fn release_clipping_mask(doc: &mut Document, paths: &[&str]) -> usize {
    let mut freed: Vec<String> = paths.iter().flat_map(|p| release_set(doc, p)).collect();
    freed.sort();
    freed.dedup();
    let mut changed = 0;
    for path in &freed {
        if let Some(layer) = resolve_path_mut(doc, path) {
            layer.clipping = false;
            changed += 1;
        }
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document_ops::layer_ops::{add_group_in, add_layer, resolve_path};

    /// Background plus four layers, bottom-first "0".."4".
    fn stack() -> Document {
        let mut doc = Document::from_rgba("Background", 4, 4, &[255; 64]);
        doc.layers[0].background = true;
        for i in 0..4 {
            add_layer(&mut doc, i, &format!("L{}", i + 1));
        }
        doc
    }

    fn clipped(doc: &Document) -> Vec<bool> {
        doc.layers.iter().map(|l| l.clipping).collect()
    }

    #[test]
    fn create_clips_to_the_layer_below_but_never_the_bottom() {
        let mut doc = stack();
        assert!(!can_create_clipping_mask(&doc, "0"), "the Background");
        assert_eq!(create_clipping_mask(&mut doc, &["0", "2", "3"]), 2);
        assert_eq!(clipped(&doc), vec![false, false, true, true, false]);
        assert_eq!(create_clipping_mask(&mut doc, &["2"]), 0, "already clipped");

        let group = add_group_in(&mut doc, "4", "G");
        let child = format!("{group}/0");
        crate::add_layer_in(&mut doc, &group, "Inside");
        assert!(resolve_path(&doc, &child).is_some());
        assert!(
            !can_create_clipping_mask(&doc, &child),
            "bottom of its group"
        );
    }

    #[test]
    fn release_frees_the_run_above_a_clipped_layer_or_a_base() {
        let mut doc = stack();
        create_clipping_mask(&mut doc, &["2", "3", "4"]);
        assert_eq!(clipped(&doc), vec![false, false, true, true, true]);
        // From the middle of the run: it and the layers above.
        assert!(can_release_clipping_mask(&doc, "3"));
        assert_eq!(release_clipping_mask(&mut doc, &["3"]), 2);
        assert_eq!(clipped(&doc), vec![false, false, true, false, false]);
        // From the base: everything clipped to it.
        create_clipping_mask(&mut doc, &["3", "4"]);
        assert_eq!(release_clipping_mask(&mut doc, &["1"]), 3);
        assert_eq!(clipped(&doc), vec![false; 5]);
        assert!(!can_release_clipping_mask(&doc, "1"));
    }
}
