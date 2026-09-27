# Proposal: annotation-tools

## Why

The Color Sampler (issue #6), Ruler (issue #7), and Note (issue #8) tools were
catalogued but disabled. photorust (perfecto25/photorust) ships all three as
annotations: marks that read a document without changing its pixels. This
change ports them onto Kooka's document model, tool framework, and canvas.

## What Changes

- `pictura_core::annotations` (`Annotations`, `MarkerKind`, `Ruler`,
  `Measurement`, `MAX_COLOR_SAMPLERS`) and `Document::annotations`: color
  samplers (capped at CS6's four) and notes are document state; the Ruler's line
  and its X/Y/W/H/A/D1 readout, with a 45° Shift snap.
- A `cxxqt_object/annotations.rs` bridge: marker add/move/remove/clear/hit-test,
  note text, and the Ruler line (held on the view, never in history).
- `tool_annotations.cpp` (Color Sampler and Note share one marker handler),
  `tool_ruler.cpp`, an `ImageView` annotation overlay
  (`image_view_annotations.cpp`), options-bar rows (Ruler readout, Clear), the
  Info panel's sampler readouts, and a Notes panel replacing the placeholder.
- C++ self-test `annotation_tools` (code 535); the unimplemented-tool guard
  (code 98) now probes Count.

## Capabilities

### New Capabilities

- `tools/color-sampler`: color samplers and the Color Sampler tool.
- `tools/ruler-tool`: the measuring line and the Ruler tool.
- `tools/note-tool`: notes, the Note tool, and the Notes panel.

## Impact

- `pictura-core` (`annotations.rs`, `Document::annotations`), `pictura-codec`
  (constructors), `pictura-app` bridge and C++ as above.
- No new dependency.
