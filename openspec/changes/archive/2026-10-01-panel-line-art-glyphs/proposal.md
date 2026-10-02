# Proposal: panel-line-art-glyphs

## Why

Issue #81 adds eleven panel and path line-art glyphs ahead of the panels that will
use them. Kooka already ships most of the `layers.*` set, but four `LayerIcons`
glyphs and all seven `PathIcons` glyphs have no asset. Each new id resolves
through the same `icon(id)` lookup as every other asset, completing the set for
the Paths panel and the shape/smart-object kind mapping that will consume them.

No shipped code requests them yet: there is no Paths panel (the slot is still a
`PlaceholderPanel`, `frame_build.cpp`) and no search, reset, shape, or
smart-object buttons, so all eleven assets are currently unreferenced and none is
drawn by an existing widget.

## What Changes

- Add four `layers.*` SVGs: `layers.search`, `layers.kindShape`,
  `layers.kindSmartObject`, and `layers.reset`.
- Add seven `path.*` SVGs: `path.thumbnail`, `path.fill`, `path.stroke`,
  `path.loadSelection`, `path.makeWorkPath`, `path.newPath`, and `path.delete`.
- Register all eleven in `assets/pictura.qrc` (explicit list, no globbing).
- Record their upstream origin in `assets/PROVENANCE.md`.
- Add Qt Test suite `tst_icon_assets` to `PICTURA_QT_TESTS`; it asserts each new
  id resolves to a non-null icon that renders at 16 px, and that an unknown id
  stays null.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `ui/icon-assets`: add the panel and path glyph coverage.

## Impact

- `assets/` (`icons/*.svg`, `pictura.qrc`, `PROVENANCE.md`) and the `pictura-app`
  C++ test list (`cpp/tests/tst_icon_assets.cpp`, `cpp/tests/CMakeLists.txt`).
  No Rust, no bridge, no new dependency.

## Provenance

Extracted from the upstream photorust tree's `shell/src/panels/LayerIcons.cpp`
and `shell/src/panels/PathIcons.cpp`
(Source: https://github.com/perfecto25/photorust). Relicensing under
GPL-3.0-or-later is tracked by KookaPictura issue #1. The geometry is copied
verbatim; only the render-time `COLOR` placeholder is replaced with the shipped
`#c8c8c8` tint, matching the existing standalone assets.
