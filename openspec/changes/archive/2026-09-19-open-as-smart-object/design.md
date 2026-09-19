## Context

`psd-smart-object-roundtrip` gave `Layer` a typed `SmartObject` and taught
`write_psd` to author a `SoLd` plus an embedded `lnk2`. The renderer draws a
layer from its raster proxy and falls back to decoding the embedded payload when
the layer has no color channel
(`crates/pictura-render/src/composite.rs::composite_smart_source`).
`convert-to-smart-object`, `place-smart-object`,
`rasterize-smart-object`, and `replace-smart-object-contents` cover the layer
actions, but `File > Open As Smart Object…` is still a disabled leaf
(`crates/pictura-app/cpp/command_tree.cpp:41`). Nothing turns a source file into
a brand-new document whose sole content is that file as a smart object, so
Photoshop's open-as-object entry point is unreachable.

## Goals / Non-Goals

**Goals:**

- One engine operation, `open_as_smart_object(filename, bytes) ->
  Option<Document>`, that decodes a PSD/PSB source, creates a new document at
  the source's size/mode/depth, seeds its composite from the decoded source, and
  appends the source as one channel-less embedded smart-object layer.
- One bridge and one app command with a stable id, correct availability, a
  PSD/PSB-filtered dialog, an untitled tab, and exactly one undo state on
  success (none on refusal).
- The new document is untitled (view path NONE) so Save cannot overwrite the
  source.

**Non-Goals:**

- Edit Contents, Export Contents, the interactive transform session, non-PSD/PSB
  source formats, and linked (external) objects. Only the PSD/PSB source path is
  covered.

## Decisions

### D1. The engine op lives beside `place_smart_object` and reuses it

`crates/pictura-render/src/document_ops/layer_ops/smart_object.rs` gains
`pub fn open_as_smart_object(filename: &str, bytes: &[u8]) -> Option<Document>`,
registered in `layer_ops/mod.rs` and re-exported from `document_ops/mod.rs` and
`lib.rs` beside `place_smart_object`. It decodes `bytes` with
`pictura_codec::read_psd`; on parse failure it returns `None`. On success it
builds `Document::new(src.width, src.height, src.mode, src.depth)`, copies the
decoded source's merged composite into the new document's composite (so the
pre-render image is correct), then calls the existing
`place_smart_object(&mut doc, filename, bytes)` to append exactly one top layer
(Embedded, channel-less, native size at `(0, 0)`). `place_smart_object` re-decodes
the same bytes and is guaranteed to succeed after the first decode; the op
returns `Some(doc)`.

### D2. The new document is untitled and holds exactly one layer

The returned document has one layer — the placed smart object — and its
composite seeded from the source. It carries no source path: the path lives on
the `PictureView`, not the `Document`. The bridge sets the view's path to NONE,
so the tab is untitled and `File > Save` routes through Save As rather than
overwriting the source file. This mirrors `PictureView::new_document`, which
also leaves `path` NONE.

### D3. The bridge mirrors `impl_core.rs::open` but early-returns on failure

Bridge `open_as_smart_object(self: Pin<&mut Self>, path: &QString) -> bool` in
`crates/pictura-app/src/cxxqt_object`. It reads `path` with `std::fs::read`,
derives the display name from the file (base name without extension, matching
`place_smart_object` in `impl_layers_smart_object.rs`), calls the engine op, and
returns `false` **before touching any state** when the read or decode fails. On
success it stores the composited image and document, calls `reset_edit_state()`,
captures exactly one `Snapshot` labelled `"Open As Smart Object"`, sets `path`
to NONE, and marks the view clean. This is `impl_core.rs::open`'s shape with the
path forced to NONE; unlike `open`, a failed load does not mutate the view.

### D4. The command creates a new tab and mirrors `openPath`

- `commands.h`: `inline constexpr char FileOpenAsSmartObject[] = "file.openAsSmartObject";`.
- `command_tree.cpp`: replace the disabled `{File, Open As Smart Object…}` leaf
  with `registry.add(command_ids::FileOpenAsSmartObject, {"File", "Open As Smart
  Object…"}, QStringLiteral("Open As Smart Object…"), QKeySequence(), true);`.
- Handler in `frame_menus.cpp`: a `QFileDialog::getOpenFileName` filtered to
  `"Photoshop files (*.psd *.psb)"`, always enabled (like `FileOpen`, no
  enabled provider). On a non-empty path it mirrors `openPath`
  (`frame.cpp:316`): construct a `PictureView`, call the bridge's
  `open_as_smart_object(path)`, and on success `addDocument(view, QString())` so
  the tab is untitled; on failure delete the view.
- `frame.h`: add the `openAsSmartObjectPath(const QString&)` declaration beside
  `openPath`, implemented in `frame.cpp`.

### D5. Verification

- **Rust engine test** in `crates/pictura-render/src/tests/smart_object.rs`: open
  a written solid-colour PSD as a smart object; assert `Some(doc)` with the
  source's dimensions and mode, exactly one layer that is an `Embedded` smart
  object carrying the exact payload, and that `composite_rgba` shows the source
  colour; assert a malformed source returns `None`.
- **C++ self-test** in the existing `selftest_layers_smart_object.{cpp,h}` (no
  new file, so `CMakeLists.txt` is unchanged): write a small solid-colour PSD to
  a temp path, open it as a smart object, assert exactly one layer that reports
  a smart object, the composite is non-empty and changed from blank, the tab is
  untitled (no path), and a malformed file fails without adding a document;
  clean up temp files. Use the next free exit code 281 (277-280 are taken) with
  `ST_BEGIN`/`ST_PASS`/`ST_SKIP`/`ST_FAIL`/`ST_FINISH`.

## Risks / Trade-offs

- **Double decode.** `open_as_smart_object` decodes once for the dimensions and
  `place_smart_object` decodes again for the layer. → Accepted; it reuses the
  existing op rather than adding a payload-passing variant, and the source is a
  single file opened once.
- **The opened object is opaque to Edit Contents.** It renders from the embedded
  source and carries no raster proxy. → Edit Contents, Export Contents, and the
  interactive transform session are named as deferred; this change only promises
  open + render + one layer.
- **Untitled means Save As.** A user who expects `Ctrl+S` to write the source
  file gets the Save As dialog. → This is the intended Photoshop behavior and the
  safety property that keeps the source from being overwritten.
- **Exit-code discipline.** The self-test uses an append-only code and a
  mis-assigned code breaks the failure identity. → Take 281, keep the check in
  the existing `selftest_layers_smart_object.cpp` within the file-size ceiling.
