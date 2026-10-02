# Tasks: info-panel-menu-refinements

## 1. Info panel

- [x] 1.1 W/H block: remove its menu (`MenuKind::None`); `applyUnit` sets both
      position and size units, then refreshes.
- [x] 1.2 Footer: add at row `keys.size()`, column 1, span 2, with
      `Qt::AlignLeft | Qt::AlignBottom`.
- [x] 1.3 Table separators: zero grid spacing; per-block `#3a3a3a` right/bottom
      borders, plus top in row 0 and left in column 0; replace the `Doc:` rule
      with a 1px `#3a3a3a` widget.
- [x] 1.4 Full colour menu: Actual Color, Proof Color, | , Grayscale, RGB, HSB,
      CMYK, Lab, |, Total Ink, Opacity, |, 8-bit, 16-bit, 32-bit; checkable
      mode/depth entries; top-left Actual Color, top-right CMYK, both 8-bit.
- [x] 1.5 Value formatting and depth scaling: Actual/Proof/RGB/Grayscale scale
      by depth; HSB/CMYK/Lab/Total Ink/Opacity depth-independent; footer reads
      the block's depth.
- [x] 1.6 Test hooks: extended `setColorModeForTest`, `setBitDepthForTest`,
      `colorFooterForTest`, `colorModeForTest`, `sizeBlockHasMenuForTest`.

## 2. Verification

- [x] 2.1 Extend `tst_info_panel.cpp`: default Actual Color; menu modes; 16-bit
      scaling and footer; inherited W/H unit; no W/H menu; existing tests pass.
- [x] 2.2 `openspec validate --all --strict`, CMake build,
      `ctest -R '^tst_info_panel$'`, `ctest -R '^tst_'`,
      `scripts/verify-fast.sh`.
