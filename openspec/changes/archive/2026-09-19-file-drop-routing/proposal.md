## Why

Phase 1 (`image-import`) made Open and Place accept common raster images, but the
only way in is a menu dialog. Photoshop users drag files from the OS onto the
app: onto the canvas to Place, onto the tab strip, menu bar, or options bar to
Open. Nothing in the frame accepts external drops today — the Layers panel
handles only its own internal MIME type — so the primary "get a picture in"
gesture is missing.

## What Changes

- **New capability `file-drop-routing`**: accept OS file drag-and-drop on the
  shell's targets and route each drop by where it lands.
  - **Canvas / the space around it** (the document `ImageView`) → **Place** each
    file into the current document, one placed object per file, reusing Phase 1's
    PSD-native vs Qt-decode routing. With no open document the drop falls back to
    the Open route.
  - **Tab strip, menu bar, options/tool context bar** → **Open** each file as
    its own new tab, one tab per file.
- **Single event filter** on the frame for `DragEnter`/`DragMove`/`Drop`, installed
  on each document `ImageView`, `QTabBar`/`QTabWidget`, `QMenuBar`, `OptionsBar`,
  and the main window as a fallback. No new widget hierarchy.
- **Hover feedback**: accept the proposed action when the drag carries at least
  one local regular file; ignore otherwise. Non-URL drags pass through so the
  Layers-panel internal DnD and tab reordering are unchanged.
- **Non-file / directory / undecodable drops** are ignored: no document change,
  no history, no crash.
- No new dependencies; no engine or Rust bridge changes — the routes call the
  existing `PictureView::open_image`/`place_image` and
  `PicturaMainWindow::openImagePath`/`openPath`.

## Capabilities

### New Capabilities

- `file-drop-routing`: the per-widget OS file drop targets, the route each one
  triggers (Place into the current document vs Open as a new tab), fan-out across
  multiple dropped files, hover acceptance, ignoring of non-file/undecodable
  drops, and preservation of the existing internal drag-and-drop.

### Modified Capabilities

<!-- None: the drop routes reuse existing image-import and document-tabs behavior;
     no existing requirement changes. -->

## Impact

- `crates/pictura-app/cpp/file_drop_router.{h,cpp}` (new translation unit, added
  to `CMakeLists.txt`): the `FileDropRouter` event filter (or
  `PicturaMainWindow::eventFilter`), the local-file classification helper, and
  the per-file place/open fan-out.
- `crates/pictura-app/cpp/frame.{h,cpp}`: the frame owns the router and installs
  the filter on `tabs_`, `tabs_->tabBar()`, `menuBar()`, `optionsBar_`, each
  `ImageView` created in `addDocument`, and `this`.
- `crates/pictura-app/cpp/frame_build.cpp`: the options bar gets a stable
  `optionsBar` object name so a drop target can be found by name.
- `crates/pictura-app/cpp/selftest_layers_smart_object.{cpp,h}` (declared there,
  called from `selftest_layers_controls.cpp`): one C++ self-test check (code 291)
  that synthesizes `QDragEnterEvent`/`QDropEvent`.
- No Rust source, no engine change, no new dependencies.

## Non-Goals

Deferred deliberately; each is a separate phase or follow-up:

- **Phase 3** the Free Transform placement session: dropped images land at native
  size, exactly as the existing Place command does.
- Targeting a drop at a specific layer or position; internal layer MIME stays the
  Layers panel's own.
- Remote URL downloads and clipboard paste.
- Per-target hover overlays beyond the drop cursor.
