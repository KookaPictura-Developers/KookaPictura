## Context

M16 left `PicturaMainWindow` owning exactly one `pictura::PictureView*` and one
`ImageView`, passed in from `main.cpp`. The File menu is registered but has no
handlers, so New/Open/Save/Close are disabled. The Rust bridge `PictureView`
owns one `Document`, a `Selection`, a `History`, and a `QImage`, and its
`open()` already loads through `pictura-codec::read_psd`; `write_psd` is
exported and tested but never called by the app.

The shell self-test is the project's app-level check; it is driven from
`main.cpp` and asserts through public frame/bridge methods.

## Goals / Non-Goals

**Goals:**

- A live File menu: New, Open, Open Recent, Save, Save As, Revert, Close, Close
  All, Quit, with an unsaved-changes prompt.
- Per-document dirty state, file path, and title, cleared on open/save and set by
  every successful mutating command.
- Multiple documents, each in its own tab with its own canvas; commands, the
  layers dock, and the status bar act on the active document.
- Atomic PSD writes through the existing codec.

**Non-Goals:**

- Formats other than PSD/PSB; import/export/PDF/video.
- Save a Copy, export layers, or per-document view/guide metadata.
- Tab drag-out-to-float, tab reorder persistence, or stacked/tiled document
  groups (a plain movable/closable `QTabWidget` only).
- Placeholder/smart-object documents.

## Decisions

### One `PictureView` per document; the frame owns the tabs

Each document is a heap `PictureView` parented to the frame, hosted in a
`QTabWidget` tab with its own `ImageView`. The frame keeps
`struct DocEntry { PictureView* view; ImageView* canvas; QString path; }`.

- *Why:* `PictureView` already encapsulates exactly one document, selection, and
  history; the generated class has `PictureView(QObject* parent = nullptr)`, so
  N instances are cheap and isolated. This avoids a Rust-side document manager
  and keeps the bridge surface small.
- *Alternatives considered:* one `PictureView` holding `Vec<Document>` and an
  active index (rejected: rewrites the bridge and its history/selection model);
  MDI sub-windows (rejected: CS6 uses tabs, and `QMainWindow` docking already
  covers panels).

### Active-document indirection in the frame, not the registry

All existing handlers and providers change from `view_` to `activeView()`;
`currentChanged` triggers `refresh()`. The command registry is unchanged.

- *Why:* menus are application-wide; the target document is a frame concern.
  Keeps M16's registry contract intact.

### Dirty flag lives in the Rust bridge, set at history-capture points

`PictureViewRust` gains `dirty: bool` and `path: Option<String>`. Every
successful mutating command sets `dirty = true` where it already captures a
history snapshot; `open()` clears both; `save()` clears `dirty` and stores the
path.

- *Why:* the mutating commands are exactly the ones that capture history, so the
  two concepts share call sites and cannot drift. The frame only reads
  `is_dirty()`/`file_path()`.

### New Document restricted to what the codec can write

`new_document` accepts mode `grayscale|rgb`, depth `8` only, and background
`white|transparent`. Other modes/depths return `false`; the dialog disables them
with a tooltip naming the limitation.

- *Why:* `write_psd` rejects everything except 8-bit Grayscale/RGB, so offering
  16/32-bit or CMYK in New would create documents that cannot be saved. Honest
  ceiling, marked with a `ponytail:` note.

### Atomic save in the bridge

`save(path)` serializes with `write_psd`, writes to `path.tmp` in the same
directory, `fsync`s, then renames over `path` (mirroring the session store).

- *Why:* a failed write must never truncate an existing PSD.

### Unsaved prompt as a small helper

`askUnsaved(parent, name) -> Save|Discard|Cancel` wraps a `QMessageBox` used by
Close, Close All, Revert, Open-replacing, and Quit. In self-test mode a
non-interactive policy can be injected to avoid blocking.

- *Why:* one prompt path, testable.

### Recent files in the session store

`SessionState` gains `recent: QStringList` (bounded, default 20); the frame
records successful opens and rebuilds the Open Recent submenu.

## Risks / Trade-offs

- **Frame refactor breaks M16 behavior** → The M16 self-test checks are rerun
  unchanged; the frame keeps the same public hooks (`registry()`, `imageView()`,
  `topLevelMenuTitles()`, `screenMode()`), with `imageView()` retargeted to the
  active canvas.
- **Tabs complicate the dock** → The layers dock always reflects the active
  view; `currentChanged` calls `refresh()`.
- **Modal prompts block the self-test** → The self-test injects a policy
  (discard/accept) rather than driving real dialogs; the prompt helper reads a
  frame-level policy flag.
- **Two PictureViews share one GPU/`InteropState`** → Only `main` uses
  `render_gpu`/`gpu_interop_prepare` for the startup fallback; per-document GPU
  state is out of scope.
- **Path handling on save dialog** → Save As appends `.psd` when no extension is
  given; the self-test uses an explicit temp path.

## Migration Plan

Additive to the bridge and the shell. The File menu leaves that M16 registered
as disabled (`New…`, `Open…`, `Open Recent`, `Save`, `Save As…`, `Revert…`,
`Close`, `Close All`, `Exit`) gain handlers; the rest stay disabled. Rollback is
reverting the `pictura-app` sources, `CMakeLists.txt`, and the OpenSpec
artifacts. No document-model or codec change.

## Open Questions

- Whether Revert on an untitled document should no-op or prompt; plan: disabled
  for untitled.
- Recent-list pruning when a file disappears; plan: validate on menu build and
  drop missing entries.
- CS6's exact New-document preset list; M17 ships a minimal preset set (Clipboard
  is unavailable, so Default Photoshop Size + Custom).
