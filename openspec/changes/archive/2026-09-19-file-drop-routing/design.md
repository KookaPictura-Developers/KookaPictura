## Context

Phase 1 (`image-import`) shipped the raster entry points on the app side:
`PictureView::open_image(path) -> bool` (decode → `Document::from_rgba` → one
`"Open"` state, path left empty), `PictureView::place_image(path) -> QString`
(decode → `add_raster_layer_from_rgba` → `convert_to_smart_object` → one `"Place"`
state), the C++ `decode_image_rgba` helper, and the frame-level routers
`PicturaMainWindow::openImagePath`/`openPath` plus the static
`isNativeDocumentPath` suffix test. `File > Open`/`Place…` already branch on that
suffix so PSD/PSB stays on `read_psd`/`place_smart_object` and every other image
goes through the Qt decode edge.

What is missing is the *trigger*: the frame accepts no external drop. The only
DnD in the tree is internal — the Layers panel's `LayersTreeView` drags rows with
its own `kLayerMimeType` (`crates/pictura-app/cpp/panels/layers_panel_internal.h`),
and `QTabWidget` reorders tabs. OS file drops carry `text/uri-list` (`QUrl`
list), a different payload that neither path claims.

The shell widgets a drop can land on:

- `ImageView` — each document's canvas. Crucially, `addDocument` makes the
  `ImageView` the `QTabWidget` page directly (`tabs_->addTab(entry.canvas, …)`,
  `frame.cpp:297`), and it paints the document plus the canvas colour around it,
  so it *is* both "the canvas" and "the space around the canvas".
- `QTabBar` — `tabs_->tabBar()`, objectName `documentTabBar`; the strip itself.
- `QMenuBar` — `menuBar()`, built by `CommandRegistry::buildMenuBar`.
- `OptionsBar` — a `QToolBar` (`options_bar.h`), the tool context bar.
- `QTabWidget` / main window — the empty document area when no document is open
  (there is then no `ImageView` to receive the drop) and the final fallback.

## Goals / Non-Goals

**Goals:**

- Accept OS file drags on the canvas, tab strip, menu bar, options bar, and the
  empty document area, and route by target: canvas → Place, the rest → Open.
- Fan out N dropped files to N placed objects or N tabs, one per file.
- Reuse the Phase 1 per-file routing (PSD native, other images through Qt) with
  no engine or bridge change.
- Ignore non-file, directory, and undecodable drops without touching a document
  or recording history.
- Leave the Layers-panel internal DnD and tab reordering exactly as they are.
- One runnable check: one C++ self-test that synthesizes real drop events.

**Non-Goals:**

- The Free Transform placement session (Phase 3), drop-position targeting, remote
  URLs, clipboard paste, per-target hover overlays beyond the drop cursor, any new
  dependency, any `docs/` change.

## Decisions

### D1. One event filter on the frame, installed per target

The router is a small `QObject` subclass (`FileDropRouter`) owned by
`PicturaMainWindow`, overriding `eventFilter` for `QEvent::DragEnter`,
`QEvent::DragMove`, and `QEvent::Drop`. The frame installs it with
`installEventFilter(router)` on `tabs_`, `tabs_->tabBar()`, `menuBar()`,
`optionsBar_`, the frame itself, and every `ImageView` created in `addDocument`.
It stores, per watched object, which route that target means — or derives it from
the object pointer (`qobject_cast<ImageView*>` → Place, else Open).

**Alternative considered: local subclasses** of `QTabBar`/`QMenuBar`/`QToolBar`/
`QWidget` with overridden `dragEnterEvent`/`dropEvent`. Rejected: four subclasses
and their construction sites for one behaviour; one `QObject` filter in its own
`file_drop_router.{h,cpp}` translation unit keeps it isolated (the new files are
listed in `CMakeLists.txt`).

**Alternative considered: `setAcceptDrops(true)` on every target plus a single
`QApplication`-level filter.** Rejected: `setAcceptDrops` on the `ImageView` would
also need the child stack to forward; the per-target install is explicit and
testable.

Qt delivers a drag to the deepest widget under the cursor and propagates it up the
parent chain only while it is ignored, so installing on `QTabWidget` and the
window as parents gives the empty-area fallback for free; the `QTabBar` and each
`ImageView` claim their own regions first.

### D2. Classification: local regular files only, non-URL drags pass through

`localPaths(const QMimeData*)` returns the `QMimeData::urls()` that are
`QUrl::isLocalFile()` **and** `QFileInfo::isFile()` (directories and non-file
URLs are dropped from the list). On `DragEnter`/`DragMove`:

- non-empty list → `event->acceptProposedAction()` (the copy cursor is the hover
  feedback; `Qt::CopyAction` is the default proposed action for OS drags);
- empty list → `event->ignore()`.

A drag with no URLs (the Layers panel `kLayerMimeType`, a Qt-internal tab
reorder, an application-internal drag) returns an empty list and is ignored by
this filter, so the widget's own handling runs untouched. The filter never calls
`accept` on such a drag and never consumes it.

This is the whole hover contract: no custom overlay, no per-target text. The drop
cursor already reads as "copy"; a richer overlay is deferred (see Non-Goals).
`ponytail:` no overlay, add when CS6 parity needs a Place/Open cue.

### D3. Routing table

| Target (deepest widget first) | No document open | Document open |
| --- | --- | --- |
| `ImageView` (canvas + surrounding space) | (no `ImageView` exists) | **Place** each file |
| `QTabBar` (strip), `QMenuBar`, `OptionsBar` | **Open** each file | **Open** each file |
| `QTabWidget` empty area / main window fallback | **Open** each file | **Open** each file |

Place with no active view is impossible on an `ImageView` (none exists when there
is no document), but the router still guards: an `ImageView` route with no
`activeView()` falls through to Open. That is the "drop with no open document
behaves like Open" clause.

### D4. Per-file fan-out reuses the Phase 1 router

Drop handling extracts `localPaths`, then for the resolved route:

- **Open**: for each path, `isNativeDocumentPath(path) ? openPath(path) :
  openImagePath(path)`; each successful call already adds one tab (untitled for
  images, path-named for PSD).
- **Place**: `PictureView* view = activeView()`; for each path,
  `isNativeDocumentPath(path) ? view->place_smart_object(path) :
  view->place_image(path)`; each success records its own `"Place"` state, so N
  files yield N objects and N states.

A failed path is skipped: the Phase 1 bridges return empty/`false` without
mutating, so a bad file in the batch neither records state nor disturbs the
others. `refresh()` is called once after the loop when anything succeeded. PSD/PSB
never touches the Qt decode edge; the branch is the same one `File > Open`/`Place`
already uses.

**Alternative considered: a new combined bridge method** `dropFiles(paths,
route)`. Rejected: it would duplicate the per-file routing that already exists in
C++ and add bridge surface for no gain.

### D5. No engine, bridge, or dependency change

Every action the router performs already exists. The only new code is the filter
(`file_drop_router.{h,cpp}` and its `CMakeLists.txt` entries), the classification
helper, the small fan-out loop, and the install calls. No Rust file is touched, so
the cxx bridge and the crate graph are unchanged.

### D6. Verification

- **C++ self-test** in `runFileDropChecks` (`selftest_layers_smart_object.{cpp,h}`,
  already in the build and called from `selftest_layers_controls.cpp`),
  allocating the next free exit code **291** (290 is `lpr_image_import`). It saves
  two small solid-colour PNGs and a native PSD to temp paths and builds a
  `QMimeData` with `setUrls({QUrl::fromLocalFile(...)})`, then:
  - sends `QDragEnterEvent` + `QDropEvent` to `frame.imageView()` (the active
    canvas) and asserts two new topmost smart-object layers and two `"Place"`
    states (fan-out, canvas route);
  - drops one supported image plus one undecodable file on the canvas and asserts
    one object and one `"Place"` state (the bad file is skipped);
  - drops the PSD on the canvas and asserts its exported embedded payload equals
    the source bytes (native `place_smart_object`, not the Qt decode edge);
  - sends the same to the tab bar (`frame.findChild<QTabBar*>(
    "documentTabBar")`), the menu bar, and the options bar
    (`frame.findChild<QToolBar*>("optionsBar")`), and asserts one new tab per
    file (open route) and that a PSD keeps its native file path;
  - sends a directory-only drag and a non-image file drag and asserts the
    drag-enter is ignored and no document/history changes;
  - with no document open, sends a drop to the `QTabWidget` (`documentTabs`) and
    asserts tabs open (no-document fallback);
  - cleans up the temp files and every document it opened.
- **Feasibility**: `QDragEnterEvent`/`QDropEvent` are plain `QEvent`s and
  `QApplication::sendEvent(target, &event)` runs the installed event filter, so no
  platform drag or input device is needed; the offscreen harness already resizes
  and shows the frame before `runSelfTest`, so the widgets exist. No geometry is
  required because the router routes by watched-object identity, not drop
  position.
- **Regression**: the existing `reorderDocumentsForTest` and the Layers-panel
  DnD self-tests stay green because the filter ignores URL-less drags.

## Risks / Trade-offs

- **The filter could swallow internal drags.** → It only acts when
  `QMimeData::hasUrls()` is true; the Layers MIME and tab reorder carry no URLs
  and pass through. Covered by an explicit self-test scenario for a URL-less drag.
- **`QToolBar`/`QMenuBar` have their own drag handling.** → The filter runs before
  the widget handler and claims only URL drags; toolbar docking and menu
  interaction are mouse-driven, not `text/uri-list`.
- **Drag propagation makes the target ambiguous.** → The route is derived from the
  watched object the filter is installed on; `QTabBar` and `ImageView` claim
  their regions first and the `QTabWidget`/window are only fallbacks.
- **Place on an inactive tab.** A drop lands on the visible `ImageView`, which is
  the active document's canvas (only the current page is visible), so Place
  targets the document the user sees.
- **Exit-code discipline.** Codes are append-only and identify failures. → Take
  291 (290 is the current max), keep the check within the suite's
  `scripts/file-size-allowlist.txt` ceiling.

## Migration

None. This is additive: no existing command, bridge method, requirement, or
persisted state changes.
