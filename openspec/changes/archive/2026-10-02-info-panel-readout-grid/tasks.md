# Tasks: info-panel-readout-grid

## 1. Bridge

- [x] 1.1 Add `document_size_bytes() -> Vec<f64>` (`[memory, disk]`) to the
      `PictureView` bridge and `impl_core.rs`; memory sums the composite, every
      layer channel (recursing groups), layer masks, and document extra channels.

## 2. Assets

- [x] 2.1 Add `info.crosshair`, `info.bounds`, `info.protractor` SVGs in the
      house style; register them in `assets/pictura.qrc` and note them in
      `assets/PROVENANCE.md`.

## 3. Info panel

- [x] 3.1 Rebuild `info_panel.{h,cpp}` as the 2×2 readout grid with per-block
      `QToolButton` menus.
- [x] 3.2 Colour-mode menu (Grayscale/RGB/HSB/CMYK/Lab) with a local sRGB→Lab
      helper; top-left defaults RGB, top-right defaults CMYK.
- [x] 3.3 Measurement-unit menu (Pixels/Inches/Centimeters/Millimeters/Points/
      Picas/Percent) per block, 72 PPI.
- [x] 3.4 Position from the cursor; W/H from `selection_bounds`, blank without a
      selection; drop the document-dimensions row.
- [x] 3.5 Ruler mode swaps the top-right block to A/L and points W/H at the
      ruler deltas; remove the separate Ruler row.
- [x] 3.6 `Doc:` line from `document_size_bytes`.

## 4. Verification

- [x] 4.1 Rewrite `tst_info_panel.cpp` for the grid: colour modes, measurement
      units, selection size, ruler swap, `Doc:` line, off-canvas blanks.
- [x] 4.2 `openspec validate --all --strict`, CMake build, `ctest -R '^tst_'`,
      `scripts/verify-fast.sh`.
