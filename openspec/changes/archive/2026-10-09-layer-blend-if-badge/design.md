# Design

## Context

See proposal.md - Why. The layer model already exposes the typed Blend If view
(`pictura_core::BlendIf`) parsed from the layer-record `blending_ranges` block
on read; the raw field is kept for unmodified round-trips. The panel already has
a projection pipeline for per-row booleans (mask, adjustment, style) surfaced as
`LayerRole`s and consumed by `LayerRowDelegate`.

## Goals / Non-Goals

**Goals:**
- One bridge read that answers "is this layer's Blend If customised?".
- A row projection + delegate chip that reuses the existing right-edge badge
  walk (`badgesRight` / `nameRect`) so lock, fx, mask, and name geometry stay in
  sync.
- A way to set a customised range from the Qt Test without a new icon asset or
  a PSD fixture.

**Non-Goals:**
- Editing Blend If from the UI (the chip is display-only, like the fx badge).
- Compositor changes: the Blend If gate math is already modeled elsewhere.
- Any change to the fx badge or the other row chips.

## Decisions

- **Customised predicate.** A layer is customised when its typed view is present
  and not `is_default()`, or when a raw `blending_ranges` block is present but
  no typed view could be parsed (malformed length). An empty raw field means
  "absent". This matches how `parse_blend_if` populates the model on read
  (empty/malformed -> `None`). Alternative (treat any `Some` view as
  customised) is wrong: a well-formed all-full body is the default and must not
  badge.
- **Bridge read mirrors `layer_row_has_style`.** A free function in the existing
  `layer_style` bridge, so `cxxqt_object.rs` (frozen at 1227 LOC) is untouched.
- **Text chip, not an icon.** CS6's affordance is reproduced as a small rounded
  rect with the words "Blend If". The chip width derives from a fixed bold font
  so `badgesRight`, `nameRect`, and paint all compute the same advance without
  threading a `QFont` through those helpers.
- **Test setter on the bridge.** No `blending.*` key writes the ranges today and
  no PSD fixture carries a custom range, so a minimal `layer_style_set_blend_if`
  bridge function sets/restores the composite-source range. It follows the
  live-edit semantics of `layer_style_set` (recomposite, no history).

## Risks / Trade-offs

- [Chip font derived from `QApplication::font()`, not the option font] → the
  chip is self-consistent (width from the same font it draws with), so it never
  clips; a view-level font override is the only mismatch, and the delegate
  already assumes the app font for other chrome.
- [Test setter is production-linkable API exercised only by tests] → kept tiny
  and documented; mirrors the existing `...ForTest` accessor convention on the
  C++ side.
- [Blend If chip can coexist with a shape row that suppresses the fx badge] →
  the chip is drawn on the `HasBlendIfRole` flag alone, independent of shape.

## Migration Plan

None: additive projection and paint; existing rows keep identical geometry when
`hasBlendIf` is false.
