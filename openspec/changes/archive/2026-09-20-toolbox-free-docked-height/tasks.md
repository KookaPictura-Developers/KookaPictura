## 1. Free docked/pane height

- [x] 1.1 `Toolbox::updateContentMetrics` pins the content height only while
  floating; docked/pane keeps min 0 / max unbounded.
- [x] 1.2 Release the dock layout's size constraint when not floating and
  override `minimumSizeHint()` to a free height.
- [x] 1.3 Recompute on `topLevelChanged` so re-docking releases the pinned height.

## 2. Coverage

- [x] 2.1 Update `tools_sizing_one` (167) to assert the docked width is locked
  and the height is free.
- [x] 2.2 Add `lss_tools_pane_height` (416): a pane-hosted panel keeps free
  height bounds and does not grow the central splitter.

## 3. Verification

- [x] 3.1 `./build/pictura --headless --self-test` — 351 passed, 0 failed.
- [x] 3.2 `bash scripts/verify-full.sh` — OK; `check-file-size` OK.
- [x] 3.3 `openspec validate --all --strict`.

## 4. Not done

- [ ] 4.1 Docked/pane content taller than the given height is clipped, not
  scrolled; a scroll host is out of scope here.
