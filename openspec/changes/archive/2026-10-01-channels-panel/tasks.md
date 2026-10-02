# Tasks: channels-panel

## 1. Panel

- [x] 1.1 `ChannelsPanel` widget with `setView(PictureView*)` and `refresh()`.
- [x] 1.2 Fixed RGB/Red/Green/Blue rows with a name label.
- [x] 1.3 Grayscale thumbnails from `view_->image()` (`qGray` composite, `qRed`/`qGreen`/`qBlue` channels).
- [x] 1.4 Local eye toggle per row using `layers.eyeOn`/`layers.eyeOff`.

## 2. Wiring

- [x] 2.1 Replace the Channels `PlaceholderPanel` with `ChannelsPanel`, keeping `objectName`.
- [x] 2.2 Register the `.cpp`/`.h` pair in the root `CMakeLists.txt` `pictura_shell` lists.
- [x] 2.3 Refresh from `retargetDock()`.
- [x] 2.4 Add `tst_channels_panel` to `PICTURA_QT_TESTS`.

## 3. Verification

- [x] 3.1 `tst_channels_panel` covers the four named rows, non-null thumbnails, local-only toggling, and stable count across refresh.
- [x] 3.2 `cmake --build build --parallel` and `ctest --test-dir build -R '^tst_'`.
