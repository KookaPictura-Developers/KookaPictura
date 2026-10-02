# Tasks: info-panel-grid-style

## 1. Info panel

- [x] 1.1 Table lines: replace the per-block `#3a3a3a` stylesheet with
      `gridRight`/`gridBottom` dynamic properties (inner cross only) and style
      them plus the `Doc:` rule from `theme.cpp` in `${border}`.
- [x] 1.2 Keys: split into a left-aligned key and a right-aligned colon
      sub-column (`keysHost` becomes a `QGridLayout`).
- [x] 1.3 Icon buttons: transparent and borderless at rest with hover and
      pressed shades, scoped `QWidget#infoBlock QToolButton` in `theme.cpp`.
- [x] 1.4 Menu placement: `InfoPanel::eventFilter` moves each readout menu
      beside its button on Show.
- [x] 1.5 Footer: pin to block row 1 with a row stretch.

## 2. Verification

- [x] 2.1 `tst_info_panel`: new `menuOpensBesideItsButton` case; all cases and
      the full `tst_` suite pass.
- [x] 2.2 `openspec validate --all --strict`, CMake build,
      `ctest -R '^tst_'`, `./build/pictura --headless --self-test`,
      `scripts/guard.sh`, `scripts/check-file-size.sh`.
