## 1. Platform-adaptive host decision

- [x] 1.1 Add `PanelFloat::overlayUsesTopLevel()` (static), false when
  `QGuiApplication::platformName()` contains `wayland` (case-insensitive).
- [x] 1.2 Add `PanelFloat::setForceChildOverlayForTest(bool)` (static, default
  false) so child mode is exercisable on the offscreen top-level platform.
- [x] 1.3 `PanelFloat` constructor: `Qt::Tool | Qt::FramelessWindowHint` when
  `overlayUsesTopLevel()`, otherwise `Qt::Widget` (in-window child); keep the
  styled background, minimum width, opacity effect, and grip.

## 2. Movement branches

- [x] 2.1 `PanelColumn::floatBounds`: screen available geometry in top-level
  mode, the owning frame rect in global coordinates in child mode.
- [x] 2.2 `PanelColumn::moveFloat`: screen clamp plus global `move()` in
  top-level mode; frame-rect clamp plus parent-relative
  `host->mapFromGlobal()` `move()` in child mode.
- [x] 2.3 Confirm `createFloat` and `floatColumn` still parent the overlay to
  the owning frame in both modes, and that `raise()` plus drag/follow work.

## 3. Coverage

- [x] 3.1 `float_child_overlay` (440): assert auto mode reports top-level on the
  current platform, force child mode, tear off a float, assert it is not a
  top-level but a child of the main window, that it moves parent-relative by the
  cursor delta, that it stays inside the frame, then restore auto mode.
- [x] 3.2 Existing top-level checks (136, 420, 422, 433-439) still pass
  unchanged.

## 4. Verification

- [x] 4.1 `openspec validate float-overlay-platform-adaptive --strict` and
  `openspec validate --all --strict`.
- [x] 4.2 `cmake --build build --parallel`.
- [x] 4.3 `QT_QPA_PLATFORM=offscreen ./build/pictura --headless --self-test` —
  zero failures, 375 checks.
