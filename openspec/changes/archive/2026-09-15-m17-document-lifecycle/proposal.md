## Why

M16 gave the shell a real frame, a command registry, and a menu bar, but the
File menu is inert and the application still shows exactly one document loaded
from a command-line path. `write_psd` exists and is tested but no application
code calls it, so work cannot be saved; there is no New, no Open dialog, no
dirty tracking, and no second document. M17 makes the File menu live and turns
the single canvas into a multi-document tab area.

## What Changes

- Add a **document lifecycle**: New (with a New Document dialog), Open (file
  dialog), Open Recent, Save, Save As, Revert, Close, Close All, and Quit, with
  a Save/Discard/Cancel prompt when a modified document would be closed or
  replaced.
- Wire the existing `pictura-codec::write_psd` into Save/Save As, writing
  atomically (temp file + rename).
- Track **dirty state** per document: set by every successful mutating command,
  cleared by open and save; show it in the tab title and window title.
- Add **multi-document tabs**: each open document gets its own canvas tab; the
  menu commands, tools dock, and status bar act on the active document; tabs are
  closable and switchable.
- Add a minimal **New Document dialog** (name, width, height, color mode,
  bit depth, background) restricted to what the codec can represent
  (8-bit Grayscale/RGB; white or transparent background).
- Extend the **session store** with the recent-files list; opening a document
  adds it, and the list is bounded.
- Extend `--self-test` with lifecycle checks: new/round-trip save/open, dirty
  marking and clearing, multi-document tab switching, and unsaved-close
  rejection.

Out of scope: formats other than PSD/PSB, PDF/video import, Save a Copy/export,
placeholder documents, tab drag-reorder/float, and per-document guides/grid.

## Capabilities

### New Capabilities

- `document-lifecycle`: New/Open/Save/Save As/Revert/Close/Quit, dirty state and
  file path, the unsaved-changes prompt, atomic PSD writes, and the recent-files
  list.
- `document-tabs`: a tabbed document area, one canvas per document, the
  active-document indirection used by commands/dock/status, and per-tab title
  with a modified marker.

### Modified Capabilities

- `application-shell`: the central image canvas becomes a tabbed document area
  that hosts one canvas per open document.

## Impact

- Affected crate: `pictura-app`. Rust bridge gains `new_document`, `save`,
  `is_dirty`, and `file_path`, and marks the document dirty on successful
  mutating commands. The C++ shell gains a New Document dialog and a document-tab
  area, and the frame's command handlers move from a fixed document to the active
  one.
- Build: `CMakeLists.txt` gains the dialog sources. No new dependencies.
- Verification: `--self-test` grows checks and exit codes; the engine/PSD
  round-trip behavior is unchanged.
- Docs/specs: `application-shell` is modified; `document-lifecycle` and
  `document-tabs` are new.
