## 1. Brief and interface freeze (orchestrator)

- [x] 1.1 Write `docs/dev/m17-document-lifecycle.md` milestone brief
- [x] 1.2 Freeze the bridge signatures (`new_document`, `save`, `is_dirty`, `file_path`), `new_document_dialog.h`, `dialogs.h`, and the new command ids (`file.new`, `file.save`, `file.saveAs`, `file.revert`, `file.close`, `file.closeAll`, `file.exit`, `file.openRecent.<path>`)
- [x] 1.3 Record sub-agent workstreams (R: bridge, D: dialogs, F: frame tabs + file handlers) with disjoint file ownership; orchestrator keeps `main.cpp`, `CMakeLists.txt`, `commands.h`/`command_tree.cpp` additions, self-test
- [x] 1.4 Commit brief + proposal with a `TASK-ALLOWS-DOCS` message

## 2. Rust bridge (workstream R)

- [x] 2.1 Add `dirty: bool` and `path: Option<String>` to `PictureViewRust`
- [x] 2.2 Add `new_document(width, height, mode, depth, background) -> bool` (8-bit grayscale/RGB; white/transparent; reset selection/history; clean)
- [x] 2.3 Add `save(path) -> bool` writing atomically via `write_psd` (temp + rename); record path and clear dirty
- [x] 2.4 Add `is_dirty() -> bool` and `file_path() -> QString`
- [x] 2.5 Mark dirty at every successful mutating command's history-capture site; `open()` clears dirty and records the path
- [x] 2.6 Add a Rust unit check for the dirty/path helper logic where testable without a QObject

## 3. Dialogs (workstream D)

- [x] 3.1 Implement `cpp/new_document_dialog.{h,cpp}`: name, width, height, color mode, bit depth, background; presets; disable unsupported modes/depths with a tooltip
- [x] 3.2 Implement `cpp/dialogs.{h,cpp}`: `askUnsaved(parent, name) -> Save|Discard|Cancel` and a non-interactive policy hook for the self-test
- [x] 3.3 Mark the 8-bit/mode ceiling with a `ponytail:` comment

## 4. Frame: tabs and file lifecycle (workstream F)

- [x] 4.1 Replace the single view with a `QTabWidget` document area and a document list (`pictureView`, `imageView`, `path`, `untitled` name)
- [x] 4.2 Add `activeView()` / `activeCanvas()` / `addDocument()` / `removeDocument()` and retarget all existing handlers, providers, dock, and status bar to the active document
- [x] 4.3 Connect tab `currentChanged` and `tabCloseRequested` to refresh and close-with-prompt
- [x] 4.4 Register handlers: `file.new`, `file.open`, `file.openRecent`, `file.save`, `file.saveAs`, `file.revert`, `file.close`, `file.closeAll`, `file.exit`
- [x] 4.5 Update tab titles with the modified marker and the window title with the active document name
- [x] 4.6 Prompt on modified close/revert/replace/quit; Cancel aborts
- [x] 4.7 Persist and rebuild the recent-files list from the session store; drop missing files

## 5. Integration and self-test (orchestrator)

- [x] 5.1 Update `commands.h`/`command_tree.cpp`: the file-lifecycle leaves become implemented with their ids
- [x] 5.2 Add the dialog sources to `CMakeLists.txt`
- [x] 5.3 Rewire `main.cpp`: open the CLI path into a frame document, or start empty; keep `render_gpu` fallback
- [x] 5.4 Extend `--self-test`: New creates a document; a save/open round-trip reloads matching pixels; save clears dirty and a mutating command sets it
- [x] 5.5 Extend `--self-test`: opening two documents makes two tabs and switching targets the active one
- [x] 5.6 Extend `--self-test`: closing a modified document with the prompt set to Cancel leaves it open; Discard closes it
- [x] 5.7 Assign self-test exit codes from 33 and keep M16 checks passing

## 6. Verification and close-out

- [x] 6.1 `cmake -S . -B build && cmake --build build`
- [x] 6.2 `xvfb-run -a ./build/pictura --self-test` and the fixture self-test pass
- [x] 6.3 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`
- [x] 6.4 `openspec validate --all --strict` and `bash scripts/guard.sh`
- [x] 6.5 Update `docs/dev/STATE.md` with a `TASK-ALLOWS-DOCS` message
- [x] 6.6 Archive the change and commit
