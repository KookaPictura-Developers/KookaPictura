# Proposal: open-recent-menu

## Why

File > Open Recent (issue #62) was built once at startup: a file opened this
session did not appear until a restart, the submenu sat at the end of the File
menu, every entry reopened through the PSD path (so a PNG/JPG failed), image
opens were never recorded, and there was no way to clear the list. photorust's
`refreshRecentMenu` rebuilds the submenu each time it opens and offers Clear
Recent File List.

## What Changes

- Open Recent sits after Open As Smart Object and is repopulated on
  `aboutToShow`: one row per existing file (name, full path as tooltip), then a
  separator and Clear Recent File List; a disabled "No Recent Files" row shows
  for an empty list.
- A recent entry opens through the same routing as File > Open (PSD/PSB through
  the codec, other files through the image importer), reporting a failure in
  the status bar; opening an image now records it.
- The recent-files code moves to `frame_recent.cpp`; `recentFiles` /
  `setRecentFiles` / `recentMenu` are public on the frame.
- C++ self-test `recent_files` (code 533), which restores the user's list.

## Capabilities

### Modified Capabilities

- `document/document-lifecycle`: adds the Open Recent menu requirement.

## Impact

- `frame.{h,cpp}`, new `frame_recent.cpp`, `frame_menus.cpp`,
  `command_tree.cpp`, new `selftest_recent_files.{h,cpp}`,
  `selftest_layers_controls.cpp`, `CMakeLists.txt`. No new dependency.
