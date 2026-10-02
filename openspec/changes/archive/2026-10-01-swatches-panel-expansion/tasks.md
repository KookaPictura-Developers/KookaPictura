# Tasks: swatches-panel-expansion

## 1. Swatches panel

- [x] 1.1 Replace the fixed button grid with a custom-painted reflowing `SwatchGrid`.
- [x] 1.2 Add click handling: plain sets foreground, ctrl sets background, alt deletes; named swatch tooltips.
- [x] 1.3 Keep the existing default palette colours as the default library, with names.
- [x] 1.4 Add the footer (New, Delete) and the context menu (New, Delete, Reset).

## 2. Wiring

- [x] 2.1 Connect the grid's foreground/background picks to the shared `ColorState`.
- [x] 2.2 Keep both classes in `panels/swatches_panel.{h,cpp}` (no new CMake file).

## 3. Verification

- [x] 3.1 `tst_swatches_panel` asserts reflow, `indexAt`, the three click modifiers, and footer New.
- [x] 3.2 Register `tst_swatches_panel` in `PICTURA_QT_TESTS`.
