## 1. Resize grip resizes the overlay

- [x] 1.1 Replace the stock `QSizeGrip` in `PanelFloat` with a custom corner
  widget that resizes the owning overlay by the drag delta, clamped to its
  minimum, and never touches the main window.
- [x] 1.2 Keep a test accessor for the grip (`sizeGripForTest()`, returning
  `QWidget*`) and drive the real grip from the self-test with synthesized mouse
  events.

## 2. Non-resizable Tools overlay

- [x] 2.1 Add `PanelFloat::setResizable(bool)`: hide the grip and size the
  overlay to the minimum its content needs.
- [x] 2.2 `PanelColumn::floatColumn` detects `isToolsColumn()` and creates a
  non-resizable overlay sized to the tool grid's content width and the content's
  minimum height.

## 3. Floating column and icon sizing

- [x] 3.1 A floating widget column opens at about two thirds of its docked
  height, clamped to the overlay minimum.
- [x] 3.2 `PanelFloat::syncToContent` shrinks the width to the icon row's
  natural width when the group collapses to icons.
- [x] 3.3 A whole-column overlay snaps to the hosted column's content when the
  column is iconic, driven from `PanelColumn::setRailMode`.

## 4. Frameless tool window with screen bounds

- [x] 4.1 Give `PanelFloat` `Qt::Tool | Qt::FramelessWindowHint`, parented to
  the main window.
- [x] 4.2 Clamp `moveFloat` to the target point's screen available geometry
  (falling back to the frame's screen, then the primary screen) and move by
  global coordinates.

## 5. Coverage

- [x] 5.1 Update the pre-existing checks that encoded the in-window contract
  (136, 407, 411, 415, 419, 420, 422) to the frameless-tool-window contract.
- [x] 5.2 `float_grip_resize` (433): dragging the resize grip changes the
  overlay size and leaves the main window size unchanged.
- [x] 5.3 `tools_float_fixed` (434): the floating Tools column has no grip and
  is sized to its content minimum.
- [x] 5.4 `column_float_height` (435): a floating widget column's default height
  is strictly less than its docked height.
- [x] 5.5 `float_icon_width_snap` (436): a collapsed-to-icons floating group
  snaps to the icon-row height and shrinks its width.
- [x] 5.6 `float_screen_bounds` (437): a float can be moved outside the main
  window rect, clamped to the screen.

## 6. Verification

- [x] 6.1 `openspec validate floating-panel-sizing-and-bounds --strict` and
  `openspec validate --all --strict`.
- [x] 6.2 `cmake --build build --parallel`.
- [x] 6.3 `./build/pictura --headless --self-test` — zero failures, 372 checks.
