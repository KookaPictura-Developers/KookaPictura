## 1. File-drop event filter

- [x] 1.1 Add a `FileDropRouter` `QObject` in `crates/pictura-app/cpp/file_drop_router.{h,cpp}`: override `eventFilter` for `QEvent::DragEnter`, `QEvent::DragMove`, and `QEvent::Drop`.
- [x] 1.2 Add a classification helper `localPaths(const QMimeData*) -> QStringList` that keeps only `QMimeData::urls()` with `QUrl::isLocalFile()` **and** `QFileInfo::isFile()` (directories and non-file URLs discarded); a URL-less drag yields an empty list.
- [x] 1.3 Implement the hover contract: on `DragEnter`/`DragMove`, `acceptProposedAction()` when `localPaths` is non-empty, `ignore()` otherwise; never accept or consume a URL-less drag so the Layers MIME and tab reorder pass through.
- [x] 1.4 Install the filter on `tabs_`, `tabs_->tabBar()`, `menuBar()`, `optionsBar_`, `this`, and each `ImageView` created in `addDocument`; add the required `QDragEnterEvent`/`QDragMoveEvent`/`QDropEvent`/`QMimeData`/`QUrl` includes to `file_drop_router.cpp`.
- [x] 1.5 Resolve the drop route from the watched object: `qobject_cast<ImageView*>` → Place, every other target → Open; an `ImageView` route with no `activeView()` falls back to Open.

## 2. Per-file fan-out using the existing routers

- [x] 2.1 Implement the Open route: for each path, `isNativeDocumentPath(path) ? openPath(path) : openImagePath(path)`; a failed file is skipped.
- [x] 2.2 Implement the Place route: for each path on `activeView()`, `isNativeDocumentPath(path) ? place_smart_object(path) : place_image(path)`; each success records its own `"Place"` state, a failed file is skipped without recording.
- [x] 2.3 Call `refresh()` once when at least one file succeeded; do not add engine, bridge, or dependency code.

## 3. C++ self-test

- [x] 3.1 Extend `runFileDropChecks` in `crates/pictura-app/cpp/selftest_layers_smart_object.cpp` (declared in its header, called from `selftest_layers_controls.cpp`; no new test file): save two small solid-colour PNGs and a native PSD to temp paths and build a `QMimeData` via `setUrls({QUrl::fromLocalFile(...)})`.
- [x] 3.2 Synthesize `QDragEnterEvent` + `QDropEvent` and `QApplication::sendEvent` them to `frame.imageView()` (the canvas); assert two new topmost smart-object layers and exactly two `"Place"` history states (fan-out, canvas route); a good+bad drop places one object with one state; a PSD drop stays native (its exported payload equals the source bytes).
- [x] 3.3 Synthesize a drop on the tab bar (`frame.findChild<QTabBar*>("documentTabBar")`), the menu bar, and the options bar (`frame.findChild<QToolBar*>("optionsBar")`); assert one new tab per file (open route on all targets), and a PSD on the tab strip keeps its native file path.
- [x] 3.4 Synthesize a directory-only drag and a non-image file drag; assert the drag enter is ignored and no document, layer, or history state changes.
- [x] 3.5 With no document open, drop on the `QTabWidget` (`documentTabs`) and assert tabs open for each file (no-document fallback).
- [x] 3.6 Use the next free exit code **291** (290 is `lpr_image_import`; the current max) with `ST_BEGIN`/`ST_PASS`/`ST_SKIP`/`ST_FAIL`/`ST_FINISH`, keep each file within its `scripts/file-size-allowlist.txt` ceiling, and clean up temp files and every document the check opens.
- [x] 3.7 Give the options bar the stable `objectName("optionsBar")` and find it by that name in the test, so adding another toolbar cannot silently retarget the check.

## 4. Regression and gates

- [x] 4.1 Confirm `reorderDocumentsForTest` and the Layers-panel DnD self-tests still pass (URL-less drags are not intercepted).
- [x] 4.2 `cargo fmt --all` and `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] 4.3 `bash scripts/verify-full.sh`; build with CMake and run `./build/pictura --headless --self-test`.
- [x] 4.4 `openspec validate file-drop-routing --strict` and `openspec validate --all --strict`.
