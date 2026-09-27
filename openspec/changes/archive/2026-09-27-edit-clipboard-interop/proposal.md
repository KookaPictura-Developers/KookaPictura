# Proposal: edit-clipboard-interop

## Why

`edit-clipboard` (#88) shipped an in-app clipboard only: pixels could not be
pasted to or from other applications, and Paste in Place was missing. photorust
implements both (the system clipboard as the transport, with the copy's document
origin remembered for Paste in Place); this change ports them.

## What Changes

- `pictura_render::Clip::{from_rgba, masked_rgba}`: import another
  application's image as a fully covered clip at the canvas origin, and export a
  clip as straight RGBA with the selection folded into alpha. `paste_clip` uses
  `masked_rgba`.
- Bridge: `PasteKind::InPlace`, `clipboard_export`, `clipboard_import`.
- Shell: Copy, Cut, and Copy Merged also put the image on the system clipboard;
  a paste first imports the system image when another application has replaced
  it since our last copy; Purge ▸ Clipboard also clears our own export (never
  another application's). Paste / Paste in Place are enabled by either
  clipboard.
- New `Edit ▸ Paste Special ▸ Paste in Place` command, placing the clip at its
  source document position (an imported image at the origin).
- The `edit_clipboard` self-test (code 529) grows to cover export, Paste in
  Place, a foreign import, and Purge of our export.

## Capabilities

### Modified Capabilities

- `document/edit-clipboard`: adds system-clipboard interop and Paste in Place.

## Impact

- `layer_ops/clipboard.rs` (+ tests), `cxxqt_object/clipboard.rs`,
  `commands.h`, `command_tree.cpp`, `frame.h`, `frame_menus_edit.cpp`,
  `selftest_clipboard.{h,cpp}`.
- No new dependency.
