# M17 — Document lifecycle and multi-document tabs

Goal: make the File menu live and turn the single canvas into a tabbed document
area. New/Open/Save/Save As/Revert/Close/Close All/Quit with dirty tracking and
an unsaved-changes prompt; one `PictureView` per document, one tab each; recent
files persisted. OpenSpec change: `m17-document-lifecycle` (MODIFIED
`application-shell`; new `document-lifecycle`, `document-tabs`).

## Scope

- Rust bridge: `PictureView` gains `new_document`, `save` (atomic PSD write via
  the existing codec), `is_dirty`, `file_path`; dirty is set at every successful
  mutating command and cleared by open/save.
- `cpp/new_document_dialog.{h,cpp}`: New Document dialog (name, size, mode,
  depth, background) restricted to 8-bit Grayscale/RGB, white/transparent.
- `cpp/dialogs.{h,cpp}`: `askUnsaved` Save/Discard/Cancel helper with a
  non-interactive policy for the self-test.
- `cpp/frame.{h,cpp}`: `QTabWidget` document area, one `PictureView` + `ImageView`
  per document, active-document indirection for all M16 handlers/providers, tab
  titles with a modified marker, File command handlers, recent-files submenu.
- `main.cpp`: construct the frame empty, open the CLI path into a document, or
  add a GPU-demo tab when no document loads; extend `--self-test`.

## Out of scope (later milestones)

- Formats other than PSD/PSB, import/export, Save a Copy, PDF/video.
- Tab drag-out-to-float, stacked/tiled groups, per-document guides/grid/view
  metadata. Tools (M18) and real panels remain future work.

## Process

Brief + OpenSpec proposal first (`TASK-ALLOWS-DOCS`), then freeze the bridge
signatures, `new_document_dialog.h`, `dialogs.h`, `frame.h`, and the file command
ids. Dispatch write-capable sub-agents on disjoint files: R (bridge), D (dialogs),
F (frame), then the orchestrator integrates `commands.h`/`command_tree.cpp`,
`CMakeLists.txt`, `main.cpp`, the self-test, verification, and the archive.

## Verification

- `cmake -S . -B build && cmake --build build`
- `xvfb-run -a ./build/pictura --self-test crates/pictura-codec/tests/fixtures/two_layers.psd`
  plus new checks (exit codes from 33): New creates a document; save/open
  round-trip reloads matching pixels; a mutating command sets dirty and save
  clears it; two documents make two tabs and switching targets the active one;
  closing a modified document with the prompt set to Cancel leaves it open.
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace`
- `openspec validate --all --strict`, `bash scripts/guard.sh`
