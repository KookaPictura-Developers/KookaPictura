# Tasks: cs6-shared-widgets

## 0. Scope

No shipped code consumes the four widgets yet. The Properties, Hue/Saturation, and
Curves panels that will use them are deferred; this change delivers the reusable
controls and their direct Qt Test coverage only.

## 1. Widgets

- [x] 1.1 `AngleDial`: wrap into `[0, 360)`, drag to aim, emit `angleChanged`.
- [x] 1.2 `RampSlider` over `JumpSlider`: gradient groove; `< 2` stops restores the groove.
- [x] 1.3 `SpectrumBar`: rainbow strip rotated by `setHueShift`.
- [x] 1.4 `CurveWidget`: monotone spline, `buildLut`, default/reset points and editing.

## 2. Wiring

- [x] 2.1 Register the four `.cpp`/`.h` pairs in the root `CMakeLists.txt` `pictura_shell` lists.
- [x] 2.2 Add `tst_shared_widgets` to `PICTURA_QT_TESTS`.

## 3. Verification

- [x] 3.1 `tst_shared_widgets` covers wrap + signals, ramp fallback + tracking, hue repaint, and LUT monotonicity/reset.
- [x] 3.2 `cmake --build build --parallel` and `ctest --test-dir build -R '^tst_'`.
