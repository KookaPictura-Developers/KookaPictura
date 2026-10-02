# Proposal: info-panel-menu-refinements

## Why

The 2×2 Info readout grid built by `info-panel-readout-grid` is faithful to
CS6's layout but simplified in four places: the W/H block carries its own unit
menu even though CS6 inherits the bottom-left unit, the `8-bit` footer is
centred under the whole block, the section separators are the default light
line, and the eyedropper menu offers only the five colour modes. Issue #80
refines the panel to the CS6 behaviour.

## What Changes

- The bottom-right W/H block loses its menu (`MenuKind::None`). `applyUnit` now
  sets the unit on both the position and size blocks, so the X/Y menu drives
  the W/H formatting too.
- The colour block's bit-depth footer moves into the content area to the right
  of the icon and is left- and bottom-aligned.
- The grid gets zero spacing and table-style separators: each block draws a
  single 1px `#3a3a3a` border on its right and bottom, plus top in row 0 and
  left in column 0, giving shared internal lines and one outer border. The
  rule above the `Doc:` line becomes a 1px `#3a3a3a` line.
- Both eyedropper blocks open the full CS6 menu: Actual Color, Proof Color,
  Grayscale, RGB, HSB, CMYK, Lab, Total Ink, Opacity, then the 8/16/32-bit
  depth entries, grouped by separators. Colour-mode and depth entries are
  checkable against the block's state. Top-left defaults to Actual Color,
  top-right to CMYK, both 8-bit. The footer reflects the block's depth.
- New test hooks: extended `setColorModeForTest`, `setBitDepthForTest`,
  `colorFooterForTest`, `colorModeForTest`, `sizeBlockHasMenuForTest`.

## Non-Goals

- **Proof Color has no proofing engine.** `ponytail:` it reads the same sRGB
  values as Actual Color; wire a soft-proof transform when one exists.
- **The engine is 8-bit.** `ponytail:` the 16/32-bit readouts scale the 8-bit
  sample (`v×257`, `v/255.0`) rather than reading a true deep-colour document.
- **PPI.** `ponytail:` unit conversion stays on the fixed 72 PPI ceiling.
- No change to the Rust bridge, assets, the Color Sampler list, or
  `runSelfTest()`.

## Capabilities

### Modified Capabilities

- `ui/info-histogram-panel` (MODIFIED and ADDED requirements): the full colour
  menu, the bit-depth footer and scaling, the inherited W/H unit, and the
  table-style separators.

## Impact

- `pictura-app` C++ shell: `cpp/panels/info_panel.{h,cpp}`,
  `cpp/tests/tst_info_panel.cpp`.
- No new dependency.
