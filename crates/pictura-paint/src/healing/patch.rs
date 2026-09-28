//! The Patch tool: repair the selection from a dragged-to area (Source), apply
//! the selection's content where it was dragged (Destination), or rebuild the
//! selection from its surroundings (Content-Aware). Every mode is one heal over
//! the selection's coverage, so it shares [`clone_region`] / [`heal_region`]
//! with the brushes.
//!
//! Ported from photorust's `Document::patch_selection`
//! (<https://github.com/perfecto25/photorust>).
//!
//! [`clone_region`]: super::clone_region
//! [`heal_region`]: super::heal_region

use super::layer::{heal_layer, HealError};
use super::{clone_region, heal_region, HealMode, Transfer};
use pictura_core::{Document, PsdRect};

/// The Patch options bar, plus the drag.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PatchOptions {
    /// The drag, in document pixels.
    pub dx: i32,
    pub dy: i32,
    /// Rebuild the selection from its surroundings and ignore the drag.
    pub content_aware: bool,
    /// The selection is good material applied where it was dragged, rather
    /// than the flaw repaired from there.
    pub destination: bool,
    /// Transfer texture only, keeping the patched area's own colour.
    pub transparent: bool,
}

/// Patch the pixel layer at `path` through `selection`, a document-sized
/// coverage mask (`doc.width × doc.height`, row-major, `0..=255`). Returns the
/// document rectangle changed, or `None` for an empty selection, a zero drag
/// outside Content-Aware, or a selection with no boundary to heal against.
pub fn patch_layer(
    doc: &mut Document,
    path: &str,
    selection: &[u8],
    options: PatchOptions,
) -> Result<Option<PsdRect>, HealError> {
    let (w, h) = (doc.width as i32, doc.height as i32);
    if selection.len() != (w.max(0) * h.max(0)) as usize {
        return Ok(None);
    }
    let Some(bounds) = bounds(selection, w, h) else {
        return Ok(None);
    };
    let coverage: Vec<f32> = (bounds.top..bounds.bottom)
        .flat_map(|y| (bounds.left..bounds.right).map(move |x| (y * w + x) as usize))
        .map(|i| f32::from(selection[i]) / 255.0)
        .collect();
    let transfer = if options.transparent {
        Transfer::TextureOnly
    } else {
        Transfer::Full
    };
    let (dx, dy) = (options.dx, options.dy);
    if options.content_aware {
        return heal_layer(doc, path, bounds, &coverage, |img, local, cov| {
            heal_region(img, local, cov, HealMode::ContentAware)
        });
    }
    if options.destination {
        // The selection's shape lands at the drag target and samples back from
        // where the selection sits.
        let target = PsdRect {
            top: bounds.top + dy,
            left: bounds.left + dx,
            bottom: bounds.bottom + dy,
            right: bounds.right + dx,
        };
        heal_layer(doc, path, target, &coverage, |img, local, cov| {
            clone_region(img, local, cov, (-dx, -dy), transfer)
        })
    } else {
        heal_layer(doc, path, bounds, &coverage, |img, local, cov| {
            clone_region(img, local, cov, (dx, dy), transfer)
        })
    }
}

/// The bounding box of the non-zero coverage, or `None` when there is none.
fn bounds(selection: &[u8], w: i32, h: i32) -> Option<PsdRect> {
    let mut r: Option<PsdRect> = None;
    for y in 0..h {
        let row = &selection[(y * w) as usize..((y + 1) * w) as usize];
        let (Some(first), Some(last)) = (
            row.iter().position(|&c| c > 0),
            row.iter().rposition(|&c| c > 0),
        ) else {
            continue;
        };
        let (left, right) = (first as i32, last as i32 + 1);
        r = Some(match r {
            None => PsdRect {
                top: y,
                left,
                bottom: y + 1,
                right,
            },
            Some(r) => PsdRect {
                top: r.top,
                left: r.left.min(left),
                bottom: y + 1,
                right: r.right.max(right),
            },
        });
    }
    r
}
