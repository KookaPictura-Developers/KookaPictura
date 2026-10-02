# Proposal: cs6-shared-widgets

## Why

Issue #80 ports the shared CS6 widgets as a reusable foundation for the
adjustment dialogs and the Properties panel. Discovery found six photorust
candidates, but two already exist in Kooka: the drop-down slider beside a field
is the existing `panels/numeric_field.*` popup, and the foreground/background
swatch is the existing `ForegroundBackgroundWidget` / `ColorCompare`.
Re-implementing either would duplicate working controls.

The other four have no full Kooka equivalent: the angle dial, the
colour-ramp-groove slider, the hue spectrum bar, and the curve editor. The
`SpectrumBar` overlaps the pre-existing non-interactive `ColorRamp`
(`cpp/color_picker_dialog.h`), which paints a spectrum but has no hue-shift
control, so the bar is a distinct interactive widget.

No shipped code consumes any of the four yet. The Properties, Hue/Saturation, and
Curves panels that will use them are deferred, so this change lands a reusable
foundation, not a working feature.

## What Changes

- `AngleDial` (`panels/angle_dial.{h,cpp}`): the circular drag-to-set-angle
  control, wrapping into `[0, 360)` and emitting `angleChanged`.
- `RampSlider` (`panels/ramp_slider.{h,cpp}`): a `JumpSlider` whose groove is a
  stylesheet colour ramp; fewer than two stops restores the ordinary groove, and
  press-drag tracking is inherited rather than re-implemented.
- `SpectrumBar` (`panels/spectrum_bar.{h,cpp}`): the rainbow strip whose colours
  rotate with the hue shift.
- `CurveWidget` (`panels/curve_widget.{h,cpp}`): the histogram, grid, baseline,
  and monotone cubic-spline curve through its control points, with a 256-entry
  `buildLut`.
- Register the four `.cpp` and `.h` pairs in the explicit `pictura_shell`
  source lists in the root `CMakeLists.txt` (no globbing).
- Qt Test suite `tst_shared_widgets`, added to `PICTURA_QT_TESTS`.

## Capabilities

### New Capabilities

- `ui/shared-widgets`: the angle dial, ramp slider, spectrum bar, curve widget,
  and the contract that makes each a self-contained reusable control.

## Impact

- `pictura-app` C++ shell (`cpp/panels/*`, `cpp/tests/tst_shared_widgets.cpp`,
  root `CMakeLists.txt`). No Rust, no bridge.
- No new dependency.

## Provenance

Ported from the upstream photorust tree at `/tmp/photorust` (AngleDial.h,
PropertiesPanel.cpp's `RampSlider`, HueSaturationDialog.cpp's `SpectrumBar`,
CurvesDialog.{h,cpp}'s `CurveWidget` including its Fritsch-Carlson
`splineInterpolate` and `buildLut`). Relicensing under GPL-3.0-or-later is
tracked by KookaPictura issue #1; the commit records the upstream source and
authors. `AngleDial` is header-only upstream and is split into a header and a
translation unit here to match Kooka's style.
