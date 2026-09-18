## 1. Header layout

- [x] 1.1 Reorder to `filterBar_` / [blend | Opacity label+field] / [five locks | Fill label+field] / tree / action strip; add `QLabel("Opacity")` and `QLabel("Fill")` (`layers_panel.cpp`).
- [x] 1.2 Remove `panelMenu_` and the local panel `QMenu`; drop `panelMenuTextsForTest` and its M39 assertion (`layers_panel.{h,cpp}`, `layers_panel_test.cpp`, `selftest.cpp`).
- [x] 1.3 Centre the `PercentField` popup horizontally under the field (`percent_field.cpp`).

## 2. Left-anchored row layout

- [x] 2.1 Add `LayersTreeView` (branch suppression); set indentation 0 / root not decorated; delegate anchors the eye at the left edge and indents thumbnail+name by depth, drawing a chevron for an expandable group (`layers_panel_internal.h`, `layers_panel.cpp`).
- [x] 2.2 Event-filter click on the chevron toggles expansion; keep the eye hit-test (`layers_panel.cpp`).

## 3. Filter lightswitch

- [x] 3.1 independent-creation `layers.filterOff.svg` / `layers.filterOn.svg`; qrc entries; the bar's toggle shows `layers.filterOn` checked by default and swaps the icon on toggle (`layers_filter_bar.{h,cpp}`, `assets/`, `assets/pictura.qrc`).

## 4. Self-tests

- [x] 4.1 New checks in a self-test TU (keeps `selftest.cpp` at its allowance): header order, Opacity/Fill labels present, no panel menu button, filter toggle is an icon and on by default, eye left-anchored for a nested row, chevron click expands.

## 5. Verification

- [x] 5.1 `cmake --build build --parallel`; `./build/pictura --headless --self-test` and the `.psd` run exit 0.
- [x] 5.2 `bash scripts/check-file-size.sh`, `python3 scripts/check-milestone-names.py`.
- [x] 5.3 `TASK_ALLOWS_DOCS=1 bash scripts/verify-full.sh` and `openspec validate --all --strict`.
