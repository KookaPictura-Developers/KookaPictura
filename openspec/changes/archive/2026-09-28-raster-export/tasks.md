# Tasks

## 1. Qt encode edge

- [x] 1.1 Add `crates/pictura-app/cpp/encode_image.{h,cpp}` exposing `encode_image_rgba(rgba, width, height, path, format, quality, scale) -> bool`: build a `Format_RGBA8888` `QImage` from packed bytes, rescale by percent when `scale != 100` (`KeepAspectRatio` + `SmoothTransformation`), flatten onto opaque white for JPEG, and `QImage::save(path, format, quality)`; return false without a partial file on any failure. Register the `.cpp`/`.h` in `CMakeLists.txt` (no globbing). Verify `cmake --build build --parallel` compiles and links the new unit.
- [x] 1.2 Declare `encode_image_rgba` in the `#[cxx::bridge]` of `crates/pictura-app/src/cxxqt_object.rs`. Verify `cargo build -p pictura-app` succeeds.

## 2. Format-aware save and export bridge

- [x] 2.1 Factor the planar-to-RGBA8888 interleaving out of `helpers_composite.rs::buffer_to_image` into `buffer_to_rgba_bytes(&PixelBuffer) -> Vec<u8>`, and have `buffer_to_image` call it, so the export path shares one plane rule. Verify existing `pictura-app` tests pass and a new unit test asserts the packed bytes match `buffer_to_image`'s pixels for 1/2/3/4-channel buffers.
- [x] 2.2 Add `source_format` to `PictureViewRust` (`state.rs`, default PSD), set it from the file suffix in `open_image`, reset it to PSD in `reset_edit_state` (used by `open`/`open_as_smart_object`/`new_document`), and expose `output_format(&PictureView) -> QString` (a free function in the sibling `export.rs` bridge, so the `cxxqt_object.rs` declaration list does not grow). Verify a unit test: `format_for_path` maps a `.PNG` suffix to `"png"` and a missing suffix to `"psd"`, and `PictureViewRust::default().source_format` is `"psd"`.
- [x] 2.3 Dispatch `PictureView::save(path)` on the lowercased suffix: `psd`/`psb`/empty/unrecognized use the existing atomic `write_psd` path; `png`/`jpg`/`jpeg`/`tif`/`tiff`/`webp`/`bmp` flatten the composite (`current_buffer` → `buffer_to_srgb` → `buffer_to_rgba_bytes`) and call `encode_image_rgba(..., quality = 90, scale = 100)`. Set `path`/`dirty = false` only on success. Verify with the C++ self-test (task 5.1): save to `.psd` reads back through the codec, save to `.png` produces a PNG whose reopened pixels match, and a failed raster encode leaves `path`/`dirty` unchanged (the encode helper links only under CMake, so the bridge is self-tested, not Rust-unit-tested).
- [x] 2.4 Add the free function `export_image(view, path, format, quality, scale) -> bool` in the sibling `export.rs` bridge (with the `encode_image_rgba` extern) that flattens and encodes without setting `path`, clearing `dirty`, or recording history. Verify with the C++ self-test (task 5.1): a document with a non-empty `path` and `dirty = true` keeps both after a successful export, and history length is unchanged.

## 3. File menu commands and Export As dialog

- [x] 3.1 Add `file.exportAs` and `file.quickExportPng` to `commands.h` and register them in `command_tree.cpp` (File menu, beside Save As / Save for Web). Verify the command-registry self-test sees both by id.
- [x] 3.2 Add `crates/pictura-app/cpp/export_as_dialog.{h,cpp}` (format combo PNG/JPEG/TIFF/WebP/BMP, quality slider enabled for JPEG/WebP, percent scale) and a shared `exportAsFromView(parent, view)` helper that runs the file dialog then the options dialog then `view->export_image(path, format, quality, scale)`. Register in `CMakeLists.txt`. Verify the build compiles and links it.
- [x] 3.3 Wire the `File ▸ Export As…` and `File ▸ Quick Export as PNG` handlers in `frame_menus.cpp`: Export As uses the shared helper; Quick Export writes `<source stem>.png` beside `view->file_path()` or prompts once when empty, then calls `export_image`. Add their `hasDocument` enable providers. Verify the build succeeds and a self-test (task 5.1) exercises both.
- [x] 3.4 Make the Save As handler format-aware: offer PSD/PSB + PNG/JPEG/TIFF/WebP/BMP name filters, preselect `view->output_format()`, and when the chosen filter is a raster and the document has more than one layer, a group, or an adjustment, show the "format cannot hold the document's features" warning before writing. Route through `saveActiveAs`. Verify the build succeeds and task 5.1 covers the flattened re-save.

## 4. Layers row menu

- [x] 4.1 In `panels/layers_panel_menu.cpp::populateRowMenu`, add `Export As…` and `Quick Export as PNG` for pixel and smart-object rows (calling the shared export helper with the panel's `view_`), and omit them for group and adjustment rows. Verify by extending `panels/layers_panel_test.cpp`'s row-menu text assertions to require the two labels for a pixel/smart-object row and their absence for a group/adjustment row.

## 5. Integration check and record

- [x] 5.1 Add one C++ self-test check in an existing `crates/pictura-app/cpp/selftest_*.cpp` suite (append-only exit code, within its `scripts/file-size-allowlist.txt` ceiling): build a small document, Export As to PNG/JPEG/BMP (PNG reopened through the import edge with a pixel compare), assert PNG keeps alpha while JPEG and BMP flatten, refuse an unwritable target, refuse Export As on an empty view, check the flatten-warning predicate, revert a raster-recorded document, and run Quick Export beside the source. Verify with `./build/pictura --headless --self-test`.
- [x] 5.2 Update `docs/dev/STATE.md` with the change record. Commit carries `TASK-ALLOWS-DOCS`. Verify `scripts/guard.sh` passes.
- [x] 5.3 Run the full gates: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`, `cargo test --workspace --doc`, `bash scripts/verify-full.sh`, and `openspec validate --all --strict`.

## 6. Review remediation

- [x] 6.1 Route Open Recent and Revert through one `openDocumentAtPath` native/raster dispatch (also used by Open, drag-drop, and the control server), so a raster path no longer hits the PSD-only reader silently.
- [x] 6.2 Make the raster encode atomic: `encode_image_rgba` writes through `QSaveFile` + `QImageWriter` (temp + rename) and cancels on failure, matching the PSD path's guarantee.
- [x] 6.3 Make Export As extension-authoritative like Save and use a single uppercase per-format filter; replace an empty/unknown suffix with the chosen format.
- [x] 6.4 Restrict the Open As Smart Object dialog to PSD/PSB and report a refusal in the status bar.
- [x] 6.5 Guard Quick Export against writing over the document's own `.png` source by prompting for a path.
- [x] 6.6 Correct the BMP alpha assumption: flatten BMP onto opaque white and amend the interop requirement and the row-menu requirement (type rows excluded).
- [x] 6.7 Populate the built-in file dialog's Places sidebar (`fileDialogPlaces`) with the root, mounted volumes, XDG folders, KDE/GTK bookmark directories, and the application's recent locations, keeping the non-native rolling type combo (see design D7).
- [x] 6.8 Make the dialog hybrid: in a Flatpak/Snap sandbox (`usesPortalFileDialog`) hand off to the platform/portal chooser and select the `xdgdesktopportal` platform theme in `main()` when sandboxed, so host files stay reachable; keep the built-in dialog on native installs. Look KDE/GTK bookmarks up through `QStandardPaths` (XDG-correct).
- [x] 6.9 Fix the raster Revert path split: `open_image` preserves an existing path, so the view stays associated with the recorded file after a Revert while a fresh import stays untitled.
- [x] 6.10 Harden the dialog layer: dedup Places by canonical path, fall back to the dialog's only combo when the private `fileTypeCombo` name is absent, make the portal theme authoritative (one sandbox check shared with `main()`), and skip building the Places list under the portal.
- [x] 6.11 Open As Smart Object accepts raster sources (CS6 parity): use the shared filters and embed a decoded raster layer as a smart object, reusing the `place_image` recipe.
