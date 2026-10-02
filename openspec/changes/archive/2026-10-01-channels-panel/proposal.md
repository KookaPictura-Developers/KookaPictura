# Proposal: channels-panel

## Why

Issue #71 replaces the empty `PlaceholderPanel` in the Channels slot with a
presentational Channels panel. CS6's Channels panel lists the composite and each
colour channel with a thumbnail and a visibility toggle, and lets those toggles
shape the canvas. The engine cannot back that yet: the model has no per-channel
visibility and the compositor has no channel-mask path, and `document_mode()`
collapses everything to grayscale/rgb at the bridge.

This change ships the presentational slice: the rows, their thumbnails built
from the existing composite, and a local eye toggle. It is deliberately a scoped
slice, not a fake full panel.

## What Changes

- `ChannelsPanel` (`panels/channels_panel.{h,cpp}`): a `QWidget` with
  `setView`/`refresh` that lays out an "RGB" composite row plus "Red", "Green",
  and "Blue" rows.
- Each row gets a grayscale thumbnail built in C++ from `view_->image()`: the
  composite uses luminance (`qGray`), the colour rows use `qRed`/`qGreen`/`qBlue`,
  each scaled to a small pixmap.
- Each row gets an eye toggle backed by `layers.eyeOn`/`layers.eyeOff`. The
  toggle is local UI state only; it makes no engine call.
- Replace the `PlaceholderPanel` at the Channels slot with `ChannelsPanel`,
  keeping `objectName` `channelsPanel`.
- Refresh from the shell's `retargetDock()`.
- Qt Test suite `tst_channels_panel`, added to `PICTURA_QT_TESTS`.

## Non-Goals

- **Toggling a channel does not change the canvas.** The model has no
  per-channel visibility and the compositor has no channel-mask path, so the eye
  is local UI state. Wire it when the model and compositor grow that path.
- **RGB rows only.** `document_mode()` collapses to grayscale/rgb at the bridge,
  so there is no per-mode channel set to enumerate.
- No extra/alpha-channel add or delete, no CMYK/Lab rows, and no saved-selection
  rows in this slice.

## Capabilities

### New Capabilities

- `ui/channels-panel`: the presentational Channels panel contract — composite and
  RGB rows with thumbnails and local-only visibility toggles.

## Impact

- `pictura-app` C++ shell (`cpp/panels/channels_panel.*`,
  `cpp/frame*.{h,cpp}`, `cpp/tests/tst_channels_panel.cpp`, root
  `CMakeLists.txt`). No Rust, no bridge.
- Reuses the existing `layers.eyeOn`/`layers.eyeOff` assets; no new dependency.
