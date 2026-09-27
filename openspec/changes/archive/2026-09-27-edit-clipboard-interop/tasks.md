# Tasks: edit-clipboard-interop

## 1. Engine

- [x] 1.1 Add `Clip::from_rgba` and `Clip::masked_rgba`; route `paste_clip` through `masked_rgba`; tests for import, refusal, and the alpha fold.

## 2. Bridge and shell

- [x] 2.1 Add `PasteKind::InPlace`, `clipboard_export`, and `clipboard_import` to `cxxqt_object/clipboard.rs`.
- [x] 2.2 Add `command_ids::EditPasteInPlace` and the `Paste Special ▸ Paste in Place` command (no default shortcut).
- [x] 2.3 In `frame_menus_edit.cpp`, export after Copy/Cut/Copy Merged, import before a paste when another application owns the clipboard, clear our export on Purge, and enable Paste/Paste in Place from either clipboard.

## 3. Self-test

- [x] 3.1 Extend `edit_clipboard` (code 529): the copy is exported, Paste in Place restores the cut pixels at their source, a foreign image pastes in place at the origin, and Purge clears our export and disables Paste.

## 4. Verification

- [x] 4.1 `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `bash scripts/verify-fast.sh`, `cargo deny check`.
