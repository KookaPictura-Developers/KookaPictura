# Proposal: info-panel-readout-grid

## Why

Issue #80 reworks the Info panel from its flat label form into the readout grid
of Photoshop CS6: four blocks in a 2×2 arrangement, each with a small
menu-opening icon and `Key : value` rows, over a `Doc:` memory line. The current
panel shows one RGB line, one CMYK line, a cursor position, a selection pixel
count, and the document dimensions, with the only choice of readout being the
Ruler tool's mode.

The upstream photorust tree already builds the grid with `addReadout` /
`setValues` (`shell/src/panels/InfoPanel.{h,cpp}`), so Kooka ports that layout
faithfully and adds the per-block menus it lacks: a colour-mode menu on the two
colour blocks and a measurement-unit menu on the position and size blocks.

## What Changes

- Replace the form layout with a 2×2 grid of readout blocks, each an
  `InstantPopup` `QToolButton` (auto-raise) beside `Key : value` rows and an
  optional static footer.
- Top-left colour block defaults to RGB; top-right defaults to CMYK. Both open a
  colour-mode menu (Grayscale, RGB, HSB, CMYK, Lab) that rebuilds the block's key
  rows and formats its values.
- The position (X/Y) and size (W/H) blocks open a measurement-unit menu (Pixels,
  Inches, Centimeters, Millimeters, Points, Picas, Percent), each block keeping
  its own unit.
- Position X/Y is the cursor position; size W/H is the selection bounds
  (`selection_bounds`), blanked when nothing is selected. The document-dimensions
  row is removed.
- Ruler mode swaps the top-right block to A/L from `ruler_measurement` and points
  W/H at the ruler deltas, with a protractor glyph; leaving the Ruler restores
  the colour block.
- Add a `Doc: <memory>/<disk>` line from a new `document_size_bytes` bridge
  method returning the pixel-plane footprint and the file size.
- New house-style glyphs `info.crosshair`, `info.bounds`, `info.protractor`.

## Non-Goals

- **PPI is not stored on the document.** Unit conversion uses a fixed **72 PPI**.
  `ponytail:` this is the named ceiling; add a per-document resolution to the
  bridge before offering real print units.
- **The `8-bit` footer stays a static label.** `ponytail:` no bit-depth readout
  or menu exists yet; wire one when the document exposes a depth switch.
- No change to the Color Sampler, Note, or Count readouts; the samplers stay a
  text list rather than rebuilt grid blocks.
- No new dependency and no `runSelfTest()` change.

## Capabilities

### Modified Capabilities

- `ui/info-histogram-panel` (ADDED and MODIFIED requirements): the readout grid,
  the colour-mode and measurement-unit menus, the `Doc:` line, and the ruler-mode
  block swap.

## Impact

- `pictura-app` C++ shell: `cpp/panels/info_panel.{h,cpp}`,
  `cpp/tests/tst_info_panel.cpp`.
- `pictura-app` Rust bridge: `src/cxxqt_object.rs`,
  `src/cxxqt_object/impl_core.rs` (`document_size_bytes`).
- Assets: `assets/icons/info.{crosshair,bounds,protractor}.svg`,
  `assets/pictura.qrc`, `assets/PROVENANCE.md`.
- No new dependency.

## Provenance

Layout ported from the upstream photorust tree at `/tmp/photorust`
(`shell/src/panels/InfoPanel.{h,cpp}`), relicensed under GPL-3.0-or-later pending
KookaPictura issue #1. The three new glyphs are original project artwork.
