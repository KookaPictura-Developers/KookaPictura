# Tasks: info-panel-expansion

## 1. Info panel

- [x] 1.1 Add a CMYK readout beside the RGB readout, derived from the sampled pixel with `QColor::toCmyk()`.
- [x] 1.2 Add ruler mode: the A/L block and the W/H block from `ruler_measurement`.
- [x] 1.3 Expose `setRulerMode`/`rulerMode` and test hooks; keep the samplers and the Position/Selection/Dimensions rows.

## 2. Wiring

- [x] 2.1 Wire `ToolController::activeToolChanged` (mode) and `rulerChanged` (re-read) from `frame_build.cpp`.

## 3. Verification

- [x] 3.1 `tst_info_panel` asserts the RGB/CMYK readouts and the ruler A/L + W/H.
- [x] 3.2 Register `tst_info_panel` in `PICTURA_QT_TESTS`.
