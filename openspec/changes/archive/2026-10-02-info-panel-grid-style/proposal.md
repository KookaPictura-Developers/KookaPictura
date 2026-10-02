# Proposal: info-panel-grid-style

## Why

The readout grid renders its table lines, the `Doc:` rule, and the icon-button
chrome from hard-coded `#3a3a3a`-era constants, which come out *lighter* than
the panel pane at the default brightness level and freeze the panel at one
theme level (`theme.h` bans hard-coded frame colours). The grid also draws an
outer frame on top of the pane edge it already sits on, the right-aligned
`Key :` labels stagger under a proportional font, the readout icons keep a
button chip at rest, and QToolButton drops each menu in front of the button
that opened it. Issue #79's follow-up polish fixes all of this.

## What Changes

- Table lines become the theme's frame shade (`${border}`, the pane colour
  darkened), darker than the pane at every brightness level, drawn only as the
  inner cross of the 2×2 grid — the outer edge is the panel pane itself. The
  `Doc:` rule uses the same shade. Blocks flag their inner sides with
  `gridRight`/`gridBottom` dynamic properties styled from `theme.cpp`.
- The readout keys become two sub-columns: the key left-aligned, the `:`
  right-aligned in its own column, so proportional glyphs cannot stagger.
- The readout icon buttons rest bare against the pane (transparent, no border)
  and only shade on hover/press.
- A block's menu opens beside its button — to the right, or the left when the
  right does not fit — never over the button (`InfoPanel::eventFilter` on the
  menu's Show, after QToolButton has set its geometry).
- The bit-depth footer pins to the block's bottom (row 1 with a row stretch)
  instead of floating at `keys.size()`.

## Non-Goals

- `ponytail:` menu placement is LTR and single-screen: the sides are not
  flipped for RTL, and a menu that fits on neither side keeps QToolButton's
  own placement.
- `ponytail:` the `gridRight`/`gridBottom` edge flags assume the fixed 2×2
  grid.
- No change to readout values, menu entries, or the Rust bridge.
