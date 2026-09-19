## Context

`psd-smart-object-roundtrip` gave `Layer` a typed `SmartObject` and taught
`write_psd` to author a `SoLd` plus embedded `lnk2` when a layer carries an
embedded payload. The renderer draws a layer from its raster proxy and falls
back to decoding the embedded payload when the layer has no color channel
(`crates/pictura-render/src/composite.rs::composite_smart_source`).
`convert-to-smart-object` exposed the authoring path from an existing layer's
raster. `File > Place…` is a disabled leaf
(`crates/pictura-app/cpp/command_tree.cpp:60`); nothing decodes a source file
and appends it as a smart object, so Photoshop's primary create path is
unreachable.

## Goals / Non-Goals

**Goals:**

- One engine operation, `place_smart_object(doc, filename, bytes) ->
  Option<String>`, that decodes a PSD/PSB source and appends it as a new
  topmost, channel-less embedded smart-object layer sized to the source.
- The appended layer renders through the existing embedded-source path, and a
  save→load round-trip re-resolves the placed object.
- One app command with a stable id, correct availability, a PSD/PSB-filtered
  dialog, and exactly one undo state on success (none on refusal).

**Non-Goals:**

- `File > Open As Smart Object…`; the interactive Place transform session;
  non-PSD/PSB source formats; Edit Contents, Replace Contents, Export Contents;
  linked (external) objects; placement transforms of any kind.

## Decisions

### D1. The engine op lives beside `convert_to_smart_object`

`crates/pictura-render/src/document_ops/layer_ops/smart_object.rs` gains
`pub fn place_smart_object(doc: &mut Document, filename: &str, bytes: &[u8]) ->
Option<String>`, registered in `layer_ops/mod.rs` and re-exported from
`document_ops/mod.rs` and `lib.rs`, matching `convert_to_smart_object`. It is a
pure function over `&mut Document` and follows the existing refusal contract:
return `None` without mutating when the source cannot be decoded.

### D2. Placement is native size at the document origin, with no transform session

The placed layer is `rect = (0, 0, decoded.width, decoded.height)`, `visible`,
`BlendMode::Normal`, `opacity = 255`, and as many as `doc.mode.color_channels()`
channels left empty. There is **no** placement/transform session: Photoshop
drops the object in and starts Free Transform, but this change only inserts it.
Content outside the canvas is clipped at render by the existing compositor.

**Deliberate simplification (single format).** Only PSD/PSB sources are accepted,
imported through the codec's reader (`read_psd`); any other format refuses at the
parse step. Multi-format import is a separate follow-up.

### D3. The placed layer is channel-less and renders from its embedded source

The new layer has **no pixel channels**, so the renderer takes the
embedded-source branch of `composite_smart_source` and draws the decoded payload
scaled into the layer rect. This is the intended path: `convert-to-smart-object`
keeps a raster proxy because it already had pixels, whereas Place has no raster
to proxy and must render from the source. Consequently the placed layer's
`smart_object` is `SmartObject { kind: Embedded, payload: Some(bytes), filename:
filename, filetype: *b"8BPB", creator: *b"8BIM", ..Default::default() }`, with
`filename` being the caller-supplied display name. The op appends the layer at
the top of `doc.layers` and returns its path using the existing `format_segments`
convention (`"0"`, `"2/1"`).

### D4. The bridge owns file I/O and mirrors the convert/rasterize shape

- Bridge `place_smart_object(&self, path: &QString) -> QString` in
  `impl_layers_smart_object.rs`. It reads the file with `std::fs::read` (mirroring
  `impl_core.rs::open`), derives the display name from the file's base name, and
  calls the engine op on the current document. On success only it calls
  `clear_link_sets()`, `recomposite()`, and `record("Place")`, returning the new
  layer path; on refusal it returns an empty string and records nothing, leaving
  the document unchanged.
- `commands.h`: `inline constexpr char FilePlace[] = "file.place";`.
- `command_tree.cpp`: replace the disabled `{File, Place…}` leaf with
  `registry.add(command_ids::FilePlace, {"File", "Place…"},
  QStringLiteral("Place…"), QKeySequence(), true);`.
- App dialog: a `QFileDialog::getOpenFileName` filtered to
  `"Photoshop files (*.psd *.psb)"`, mirroring `showOpenDialog` in `frame.cpp`.
  The handler is enabled only when a document is open; on a non-empty path it
  calls the bridge and refreshes on success.

### D5. Verification

- **Rust engine test** in `crates/pictura-render/src/tests/smart_object.rs`:
  place a written solid-colour PSD into a document; assert `Some(path)`, a new
  topmost layer with `rect` equal to the source size, no pixel channels, and an
  `Embedded` smart object whose payload and filename match; assert
  `composite_rgba` shows the source colour over the layer rect; assert
  `write_psd` then `read_psd` re-resolves the placed object; assert malformed
  bytes return `None` with the document unchanged.
- **C++ self-test** extending `selftest_layers_controls.cpp` (no new file, so
  `CMakeLists.txt` is unchanged): write a small PSD to a temp path, place it,
  assert a new smart-object layer appears at the top and the composite changes,
  and assert a malformed file refuses with no history state. Use the next free
  exit code, 279 (277 and 278 are taken).

## Risks / Trade-offs

- **The placed object is opaque to Edit Contents.** It renders from the embedded
  source and carries no raster proxy, so Edit Contents/Replace Contents remain
  deferred. → Named here; this change only promises place + render + round-trip.
- **Placed content can exceed the canvas.** Native-size placement at `(0, 0)`
  clips at render. → Matches the specified simplification; the transform session
  that would reposition/scale it is a separate follow-up.
- **Non-PSD sources silently refuse.** A user may choose a PNG and see a no-op.
  → The dialog filter limits the choice to `*.psd *.psb`; the op's parse refusal
  is the backstop. Broadening formats is a separate follow-up.
- **Exit-code discipline.** The self-test uses an append-only code and a
  mis-assigned code breaks the failure identity. → Take 279, keep the check in
  the existing suite and within its file-size ceiling.
