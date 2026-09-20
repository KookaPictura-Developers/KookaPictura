## 1. Clamp panel widths

- [x] 1.1 Add `kMinNormalWidth`/`kMaxNormalWidth` (180/400) to `PanelColumn` and
  clamp `persistedWidth()` to them.
- [x] 1.2 `setPreferredWidth` clamps a normal column to `[kMinNormalWidth,
  kMaxNormalWidth]` and an iconic strip to `kIconStripMinWidth`;
  `setRestoredWidth` clamps the remembered normal width.
- [x] 1.3 Update `lpr_iconic_flip_width` (399) to assert the stored width is
  bounded and an oversized restore clamps to `kMaxNormalWidth`.

## 2. Verification

- [x] 2.1 `cmake --build build --parallel` and
  `./build/pictura --headless --self-test` — 334 passed, 0 failed.
- [x] 2.2 `bash scripts/verify-full.sh` — TOTAL 1637 passed, 11 skipped, 0 failed.
- [x] 2.3 `openspec validate --all --strict`.

## 3. Not done

- [x] 3.1 No change to the splitter's stretch behaviour or the drag-to-resize
  ceiling beyond the 400 px persistence bound.
