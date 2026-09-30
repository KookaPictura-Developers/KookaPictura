# Proposal

## Why

A brush dab costs O(its bounding box), and above roughly ⌀600 that box stops
fitting the 16 ms input-to-first-pixel Target. `large_brush_profile_4000`
(release, 4000²) puts a single 1000 px dab at 60 ms, a 2000 px dab at 201 ms and
a 5000 px dab at **933 ms** with a 133 ms region present — 56× the budget — while
`options_bar_paint.cpp` lets the Size field reach 5000 px. The two earlier
changes took everything that was cheap: the tip profile and the layer's plane
positions are hoisted out of the pixel loop, the document clones are refcount
bumps, and the in-stroke present is frame-bounded. What is left is the pixels
themselves, and no constant-factor win moves 56×.

## What Changes

- **Large strokes preview at a reduced level.** When a dab's bounding box
  exceeds the raster budget, the stroke runs against a reduced-resolution copy
  of the document at view-pyramid level ≥ 3, and the canvas presents that level
  instead of the zoom-selected one — the whole in-progress image drops to the
  preview resolution, exactly as Krita's Instant Preview does, and comes back to
  full resolution the moment the stroke ends.
- **The exact stroke still runs.** The reduced-resolution work is presentational
  only: the samples are recorded, and the full-resolution stroke is computed
  before the history state is recorded, so undo, the oracles and every golden
  stay bit-identical. What the user sees while the button is down is
  approximate; what lands is not.
- **The threshold is one measured constant.** Preview starts where the exact
  raster would leave the budget: 262 144 px of bounding box (⌀≈578), which is
  the 16 ms Target at the 58 ns/px the profile measures. The level starts at 3
  and only rises when the preview's own work would exceed the same budget.
- **BREAKING**: while a large stroke is in progress the canvas shows a
  reduced-resolution level rather than the zoom-selected level, and
  `regionBlitted` carries a preview-sized region. The commit restores the
  zoom-selected level and patches the exact pixels; consumers that read the
  display mid-stroke must expect the preview.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `tools/paint-engine`: the live stroke gains a reduced-resolution preview
  branch above the raster budget, with the exact full-resolution stroke required
  to complete before the history state is recorded.
- `ui/document-canvas`: while that preview is active the canvas crops a
  reduced view-pyramid level instead of the zoom-selected one, and the commit
  or cancel restores the zoom-selected level.

## Impact

- `crates/pictura-render/src/view_pyramid.rs` — a `patch_level` write that lets
  the preview repair one stored level without rebuilding it from level 0.
- `crates/pictura-app/src/cxxqt_object/{impl_paint.rs,state.rs,impl_core.rs}` —
  the threshold and level policy, the reduced document, the preview stroke, the
  sample log, the preview present, and the exact drain at commit.
- `crates/pictura-app/cpp/image_view.cpp` — clamp `presentLevelForZoom` to the
  preview level while a preview stroke is active.
- `crates/pictura-app/src/cxxqt_object.rs` — one `is_previewing` invokable so
  the canvas can clamp; it must stay within the file's 1227-line ceiling.
- New self-test check (next free exit code **545**) plus the region-parity
  suite; no new dependencies.
