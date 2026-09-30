# Proposal: swatches-panel-expansion

## Why

Issue #78 expands the Swatches panel toward CS6. The current panel is a fixed
4×12 grid of `QPushButton`s: it never reflows, its swatches have no names, and
the only interaction is a plain click that sets the foreground. CS6's panel is a
reflowing grid of named swatches with ctrl-click for the background, alt-click to
delete, and a footer that adds and removes swatches.

The upstream photorust tree implements the whole grid as a custom-painted
widget (`SwatchesPanel.{h,cpp}`), which is the shape Kooka should take.

## What Changes

- Replace the fixed button grid with a custom-painted reflowing `SwatchGrid`:
  swatches reflow to the panel width, named swatches tooltip, click sets the
  foreground, ctrl-click the background, and alt-click deletes.
- Add a footer and a context menu offering New, Delete, and Reset.
- Keep Kooka's current default palette colours as the default library, with
  names attached for the tooltips.
- Keep both classes in `panels/swatches_panel.{h,cpp}` so no new translation
  unit is registered.
- Qt Test suite `tst_swatches_panel`, added to `PICTURA_QT_TESTS`.

## Non-Goals

- `.aco` library load and save is deliberately deferred. It is not in upstream
  (`SwatchesPanel.h` says so) and the docs place it in a not-yet-existing
  `pictura-presets` crate (`docs/07-color-painting/swatches-and-libraries.md`).
  `ponytail:` this is the named ceiling; the crate does not exist and MUST NOT
  be created for this change. No Load/Save entry is shown.
- CS6's new-swatch name prompt is omitted; a new swatch takes the foreground
  colour's hex as its name. `ponytail:` add the prompt when a name source is
  needed.

## Capabilities

### Modified Capabilities

- `ui/color-swatches-panel` (ADDED requirements): the reflowing swatch grid,
  the click modifiers, and the footer and context menu.

## Impact

- `pictura-app` C++ shell (`cpp/panels/swatches_panel.{h,cpp}`,
  `cpp/tests/tst_swatches_panel.cpp`, tests `CMakeLists.txt`). No Rust, no
  bridge, no new file in the root `CMakeLists.txt`.
- No new dependency.

## Provenance

Ported from the upstream photorust tree at `/tmp/photorust`
(`shell/src/panels/SwatchesPanel.{h,cpp}`). Relicensing under GPL-3.0-or-later is
tracked by KookaPictura issue #1.
