# Tasks: blur-tool

## 1. Engine

- [x] 1.1 `pictura_paint::focus` (`BlurOptions`, `BlurMode`, `BlurBrush`) and `Stroke::begin_blur`; unit tests.

## 2. Bridge

- [x] 2.1 `begin_blur` in `cxxqt_object/paint_tools.rs`; one "Blur" state.

## 3. Tool

- [x] 3.1 `tool_blur.cpp` handler; bar in `options_bar_paint.cpp`; catalog row enabled; `CMakeLists.txt` entries.

## 4. Verification

- [x] 4.1 `blur_tool` self-test (554) in `selftest_retouch.cpp`; the guard (98) retargeted; `bash scripts/verify-fast.sh`.
