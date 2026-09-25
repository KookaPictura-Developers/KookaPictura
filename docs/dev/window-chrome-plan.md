# Window chrome — modern merged title bar (plan)

Optional, unscheduled. This note captures the research so a future session can
implement it without re-doing the platform survey. No code was changed for it.

Goal: one row across the top of the application frame holding the app icon and
the menus on the left and the window buttons (minimise / maximise-restore /
close) on the right, with no native title bar. This is the modern Photoshop
(CC) look, not the CS6 chrome: `02-ui-ux/application-frame.md` documents a
separate slim title bar above the menu bar, so this is a deliberate deviation.

## Platform matrix

Qt's client-side-decoration API is `QWindow::startSystemMove()` and
`QWindow::startSystemResize(Qt::Edges)` (Qt 5.15+). Support differs, and the
whole design follows from it:

| | startSystemMove | startSystemResize | plan |
|---|---|---|---|
| Linux / X11 | yes | yes | frameless, custom chrome |
| Linux / Wayland | yes | yes | frameless, custom chrome |
| macOS | yes | **no** | keep native window, no custom chrome |

- `startSystemResize` is not implemented by the Cocoa platform plugin and
  returns `false` on macOS even in Qt 6 (`QTBUG-88218`; still reported in
  2025). A frameless macOS window would need a hand-rolled resize or a native
  `NSWindow` wrapper.
- On macOS Qt already routes a `QMainWindow`'s `QMenuBar` to the system-wide
  menu bar, merging About / Preferences / Quit by `QAction::menuRole()`
  (`QMenuBar` docs). Keeping the native window therefore gives the merged look
  for free: menus in the system bar, traffic lights and native resize in the
  window. Do **not** go frameless there.
- Kooka Pictura targets Linux only (`AGENTS.md`). The macOS branch is a guard,
  not work.

### Why not just edit the native title bar

Not possible. On Linux the decoration is drawn by the WM/compositor
(server-side); Qt has no API to place widgets into it. The merged row is only
reachable by removing the frame and drawing the row ourselves. Once
`Qt::FramelessWindowHint` is set there is no server-side decoration and no
runtime way back (a runtime switch needs a window recreate).

## Linux implementation

Touch points are in `crates/pictura-app/cpp/frame.cpp` (constructor,
`frame.cpp:71`), `frame_build.cpp`, and the screen-mode switch
(`frame.cpp:803`).

1. **Frameless, before show.** `setWindowFlag(Qt::FramelessWindowHint, true)`
   in `PicturaMainWindow`'s constructor, in the same block as the existing
   window setup. Guard it:
   `#if !defined(Q_OS_MACOS)` (on macOS the flag is skipped and Qt's native
   chrome is used unchanged).
2. **Keep the real `QMenuBar`.** Put the buttons in its top-right corner with
   `menuBar()->setCornerWidget(controls, Qt::TopRightCorner)` and the app icon
   at `Qt::TopLeftCorner`. This is the smallest change that yields the merged
   row, and it keeps `frame.menuBar()` returning a real `QMenuBar` — required
   by the file-drop router (`frame.cpp:132`), the smart-object drop self-test,
   and the `m42_menubar` geometry invariant (`selftest.cpp:3776`).
3. **`WindowControls` widget.** Three `QToolButton`s themed via `theme.h`,
   glyphs via `icons.h`. Wire to `showMinimized()`, maximize/restore toggle,
   `close()`. New `frame_chrome.cpp` — and remember new `.cpp`/`.h` files must
   be added to `CMakeLists.txt` explicitly (no globbing).
4. **Drag.** Event-filter the menu bar; on left-press where
   `menuBar()->actionAt(pos) == nullptr` (the empty gap before the buttons)
   call `windowHandle()->startSystemMove()`; on double-click toggle
   maximize/restore. Honour the `false` return from `startSystemMove` (tiling
   WMs may refuse).
5. **Resize.** Reserve a ~4px margin so the main window receives events at the
   edges, then call `startSystemResize(edges)` from `mousePressEvent` /
   hover handling. The top edge is handled on the menu-bar row. This is the
   fragile part; if a compositor misbehaves, QWindowKit
   (Apache-2.0, uses Qt private APIs) is the fallback dependency.
6. **System menu.** No portable native API. Build a small menu with
   Minimize / Maximize / Close on right-click of the drag area.
7. **Screen modes.** In `FullWithMenuBar` hide `WindowControls` (the spec
   hides the window buttons there) but keep the menu row; in `Full` hide the
   whole row, as today.

### Known ceilings (mark with `ponytail:`)

- **No shadow / rounded corners.** On Wayland a shadow must be part of the
  decoration surface and Qt cannot tell the compositor which pixels are
  shadow, so drawing one breaks tiling/snapping. On X11 a frameless window
  usually has no compositor shadow either. Flat window, or a fake 1px border.
- **Tiling WMs** (sway / i3 / Hyprland) own geometry and may ignore
  `startSystemMove`.
- **GNOME/Mutter** has no server-side decorations at all, so little is lost
  there; **KDE** does, so its native frame is replaced by ours (cosmetic
  only). Window title/icon stay set, so taskbar grouping and AT-SPI are
  unaffected.

## Verification

- Extend the shell self-test (`selftest_shell_round*.cpp`): frameless flag set,
  three buttons present, buttons sit to the right of the menu row, and the
  `m42_menubar` check still passes.
- `cmake -S . -B build -G Ninja -DCMAKE_EXE_LINKER_FLAGS=-fuse-ld=lld && cmake --build build --parallel`
- `bash scripts/verify-fast.sh`

## macOS reference (if ever ported)

Custom chrome on macOS is the opposite of frameless: keep the native
`NSWindow` and make its title bar transparent so content can sit under it —
`titlebarAppearsTransparent = YES`, `titleVisibility = hidden`,
`NSWindowStyleMaskFullSizeContentView`, and draw the row inset past the
traffic lights. That preserves native resize, move, snapping and the system
window buttons. Requires an Objective-C++ wrapper (what QWindowKit /
FramelessHelper provide). Since menus already live in the system menu bar,
there is no reason to draw them in the window.

## Sources

- Qt blog, *Custom client-side window decorations in Qt 5.15*
  (move/resize support matrix; no runtime CSD/SSD negotiation; Wayland shadow
  limitation): https://www.qt.io/blog/custom-window-decorations
- `QTBUG-88218` — `QWindow::startSystemResize()` is not implemented on macOS.
- Qt docs — `QMenuBar` as a global menu bar and macOS menu-role merging:
  https://doc.qt.io/qt-6/qmenubar.html
- `frameless-qt-poc` (Qt Widgets CSD proof): https://github.com/pedrolcl/frameless-qt-poc
- QWindowKit (frameless framework, Linux/macOS/Windows):
  https://github.com/stdware/qwindowkit
