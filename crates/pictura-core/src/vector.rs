//! A derived view of a layer's `vmsk` vector mask.
//!
//! The raw `vmsk` tagged block stays in [`crate::Layer::extra_blocks`] and is
//! re-emitted verbatim; this view is what the compositor can sample.

/// How a subpath's interior is determined.
///
/// Even-odd is the documented PSD default; non-zero is read only from the
/// ag-psd fill-rule marker `2` and is not documented by the reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VectorFillRule {
    #[default]
    EvenOdd,
    NonZero,
}

/// One flattened subpath of a vector mask.
///
/// `operation` is the subpath's boolean operation (1 union, 2 not-or, 3
/// intersect, 0 xor, -1 continuation); it is recorded but not applied.
/// `points` are `[x, y]` document-pixel polyline vertices in 1/256-pixel units,
/// stored as integers so [`crate::Layer`] keeps its `Eq` derive.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VectorSubpath {
    pub closed: bool,
    pub operation: i16,
    pub fill_rule: VectorFillRule,
    pub points: Vec<[i32; 2]>,
}

/// A layer's decoded vector mask.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VectorMask {
    pub subpaths: Vec<VectorSubpath>,
    pub invert: bool,
    pub disabled: bool,
}

impl VectorMask {
    /// Whether the mask has a live closed subpath to sample. An absent,
    /// disabled, empty, or all-open mask is constant coverage (`255`).
    pub fn has_fill(&self) -> bool {
        !self.disabled
            && self
                .subpaths
                .iter()
                .any(|s| s.closed && s.points.len() >= 3)
    }
}
