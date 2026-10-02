# Proposal: info-panel-expansion

## Why

Issue #79 expands the Info panel toward its CS6 readout. The current panel shows
the cursor position, the RGB colour under the cursor, the selection size, and the
document dimensions, but CS6 also shows the colour's CMYK components and, while
the Ruler tool is active, the measuring line's angle, length, and deltas. Both
are pure readouts the panel can compute from state it already has.

The upstream photorust tree has both behaviours in `InfoPanel.{h,cpp}`; Kooka
already exposes the ruler measurement through the `ruler_measurement` bridge and
already consumes it in the options bar, so no new bridge or engine work is
needed.

## What Changes

- Add a CMYK readout beside the existing RGB readout, computed in C++ with
  `QColor::toCmyk()` on the sampled pixel.
- Add ruler mode: while the Ruler tool is active, show the angle/length (A/L)
  block and point the W/H block at the ruler's measurement.
- Wire the panel to `ToolController::activeToolChanged` (mode) and
  `rulerChanged` (re-read) from `frame_build.cpp`, the way the options bar is
  wired.
- Keep the existing Position, Selection, Dimensions, and sampler rows.
- Qt Test suite `tst_info_panel`, added to `PICTURA_QT_TESTS`.

## Non-Goals

- The `Doc: n/n` memory-footprint line is omitted: there is no document-size
  bridge to read it from. `ponytail:` this is the named ceiling; add it when a
  document-size bridge exists.
- No change to the Color Sampler, Note, or Count readouts.

## Capabilities

### Modified Capabilities

- `ui/info-histogram-panel` (ADDED requirements): the CMYK colour readout and
  the Ruler readout mode.

## Impact

- `pictura-app` C++ shell (`cpp/panels/info_panel.{h,cpp}`,
  `cpp/frame_build.cpp`, `cpp/tests/tst_info_panel.cpp`, tests `CMakeLists.txt`).
  No Rust, no bridge change.
- No new dependency.

## Provenance

Ported from the upstream photorust tree at `/tmp/photorust`
(`shell/src/panels/InfoPanel.{h,cpp}`, lines 161–244 for ruler mode and lines
232–243 for the CMYK readout). Relicensing under GPL-3.0-or-later is tracked by
KookaPictura issue #1.
