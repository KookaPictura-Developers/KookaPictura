# Design

## Context

See `proposal.md` for motivation. The starting state that shapes the approach:

- **Import is asymmetric with export.** `image-import` established a Qt decode
  edge: a C++ helper (`cpp/decode_image.cpp`) turns bytes into packed RGBA8888,
  and Rust (`impl_core.rs::open_image`/`place_image`) builds engine structures.
  There is no mirror for writing, so `PictureView::save` always calls
  `pictura_codec::write_psd` and ignores the target extension
  (`crates/pictura-app/src/cxxqt_object/impl_core.rs:240`).
- **Qt encode is already used in-tree.** `control_server.cpp` encodes a
  `QImage` to PNG (`encodePngBase64`, line 114) and several self-tests call
  `QImage::save(path, "PNG")`. The encoder is a proven, dependency-free seam.
- **The flattened composite is already a `QImage`.** `PictureView::image()`
  returns `Format_RGBA8888` built from `doc.composite` (via
  `helpers_composite.rs::buffer_to_image`), scale/alpha intact, and is what the
  control server already screenshots.
- **Save is atomic in Rust.** `save` writes `path.tmp` then renames, and sets
  `path`/`dirty`; that logic must stay the single owner of the document's saved
  state.
- **An imported raster is untitled.** `open_image` deliberately sets `path` to
  `None` so `Ctrl+S` cannot overwrite the source image. So "Save As defaults to
  the source format" must be remembered as state, not recovered from `path`.

## Goals / Non-Goals

**Goals:**

- One encode edge for every raster output, used by Save As, Export As, and Quick
  Export; PSD/PSB keeps the native codec.
- Remember the import source format so Save As preselects it, while the path
  extension remains the authoritative format on save.
- Keep the engine Qt-free and add no dependency.
- Every raster output is the flattened composite; a format that cannot hold
  layers warns before writing.

**Non-Goals:**

- Save for Web & Devices (#61), Export Layers to Files, Zoomify, Render Video,
  and the other `File > Export` stubs.
- GIF export, 16/32-bit raster output (the working model is 8-bit; the display
  image is 8-bit sRGB), ICC/EXIF/XMP embedding in exported rasters.
- Exporting a single layer or a selection; Export As always writes the document
  composite.
- Changing the smart-object editor save path (it is a PSD temp file).

## Decisions

### D1. Encode at the Qt boundary in C++, mirroring `decode_image.cpp`

Add `crates/pictura-app/cpp/encode_image.{h,cpp}` exposing, across the cxx
bridge:

```
encode_image_rgba(rgba: &[u8], width: i32, height: i32, path: &QString,
                  format: &QString, quality: i32, scale: i32) -> bool
```

It builds a `QImage` in `Format_RGBA8888` from the bytes (the same layout
`decode_image.cpp` produces), optionally scales it (percent, `KeepAspectRatio` +
`SmoothTransformation`) when `scale != 100`, flattens onto opaque white for the
formats whose writers cannot carry alpha (JPEG and BMP), and writes through a
`QSaveFile` + `QImageWriter` so the raster write is atomic (temp + rename) like
the PSD path. Registered explicitly in `CMakeLists.txt` (no globbing). Rust owns
file bytes for PSD and calls this for rasters, so no Qt type crosses the bridge —
the `image-import` precedent.

**Alternative considered:** Rust encoders (`png`/`jpeg-encoder`/…). Rejected:
new dependencies, duplicate of what Qt already ships, and inconsistent with the
decode edge the import path deliberately chose.

**Alternative considered:** export entirely in C++ via `view->image()`. Rejected
for Save: the atomic-write and `path`/`dirty` state live in Rust, and splitting
the two save kinds across the language boundary would duplicate that logic.

### D2. `save(path)` dispatches on the path extension; the extension is the format

`PictureView::save(path)` lowercases the suffix:

- `psd`, `psb`, empty, or unrecognized → the existing `write_psd`/`write_psb`
  atomic path (unchanged bytes, layers preserved).
- `png`/`jpg`/`jpeg`/`tif`/`tiff`/`webp`/`bmp` → flatten the composite to sRGB
  packed RGBA8888 and call `encode_image_rgba(..., quality = default 90,
  scale = 100)`.

On success, set `path` and `dirty = false`; on failure, mutate nothing. The
unrecognized/empty fallback to PSD preserves today's behavior (and keeps callers
that save to suffix-less temp paths working).

The Rust flatten step reuses the engine's display conversion: `current_buffer`
→ `buffer_to_srgb` → packed RGBA. Factor the interleaving out of
`buffer_to_image` into `buffer_to_rgba_bytes(&PixelBuffer) -> Vec<u8>` so the
`QImage` path and the export path share one plane-to-RGBA rule.

### D3. The source format is remembered only to preselect Save As

`PictureViewRust` gains `source_format: OutputFormat` (PSD by default). It is set
by `open_image` from the file suffix and reset to PSD by `open`,
`open_as_smart_object`, and `new_document`. `output_format(&PictureView) ->
QString` exposes it. The Save As dialog uses it to select the initial name
filter; the actual writer is chosen from the chosen path's suffix (D2), so there
is exactly one source of truth for what gets written.

**Alternative considered:** derive the Save As default from `view.path`.
Rejected: an imported raster is untitled (no path), which is precisely the case
the requirement is about.

### D4. Export As and Quick Export never touch the document

A second bridge file (`cxxqt_object/export.rs`, mirroring `magnetic.rs`) declares
the free function:

```
export_image(view: &PictureView, path: &QString, format: &QString, quality: i32, scale: i32) -> bool
```

Free functions, not `#[qinvokable]` methods: `cxxqt_object.rs` is at its
recorded file-size ceiling, and a sibling bridge is the established way to add
entry points without growing the `PictureView` declaration list.
`encode_image_rgba` (the extern "C++" encode) lives in that bridge too.

It flattens the composite, calls `encode_image_rgba`, and does **not** set
`path`, clear `dirty`, or record history. It returns `false` without writing on
a missing document or a failed encode. `Export As…` and `Quick Export as PNG`
both use it; this is also the "Save a Copy" behavior from #60.

The Export As dialog is a small modal `ExportAsDialog` (format combo
PNG/JPEG/TIFF/WebP/BMP, quality slider enabled for JPEG/WebP, scale percent) run
before a `QFileDialog`. A shared C++ helper (`exportAsFromView(parent, view)`)
runs the options dialog, then the file dialog, then `export_image`; the typed
path extension selects the encoder, exactly as Save's does, and is called from
both the File menu handler and the Layers row menu so the two entry points
cannot drift.

**Quick Export** computes `<source dir>/<source stem>.png`; when the document is
untitled it prompts once for a path. It records no history and does not re-point
the document.

### D5. Save As warns when the format cannot hold the document

The Save As handler offers name filters for PSD/PSB + the five raster formats and
preselects PSD for any document that needs layers (otherwise the source format).
When the chosen format is a raster and the document has more than one layer, an
adjustment layer, a group, or a non-flat structure, it shows a modal warning
("the chosen format cannot hold all of the document's features; the flattened
composite will be written") before writing, then proceeds. The OpenSpec
requirement is "warn before writing"; the docs' bottom-of-dialog affordance is
reproduced as a message box, which is the deliberate divergence. PSD/PSB never
warn (they preserve layers).

### D6. Row-menu entries are gated by row kind

`LayersPanel::populateRowMenu` adds `Export As…` and `Quick Export as PNG` for
rows whose kind is `pixel` or `background` (a smart-object row reports the
`pixel` kind), and leaves them out of (or disabled on) group, adjustment, and
type rows. The actions call the D4 helper with the panel's `view_`; the export
always targets the whole composite, so the row kind only decides availability,
matching #107.

### D7. The file dialogs are hybrid: portal in a sandbox, built-in otherwise

Two delivery targets need different file dialogs:

- **Native distro builds** (source, `.deb`, AppImage) use the built-in Qt dialog
  (`DontUseNativeDialog`) so the "File type" combo is non-editable and rolls
  with a repeated key, and the sidebar is ours to fill. `fileDialogPlaces()`
  populates it: the root, `QStorageInfo::mountedVolumes()` (pseudo filesystems
  skipped), the `QStandardPaths` folders, the KDE `user-places.xbel` and GTK
  `gtk-3.0/bookmarks` files found on the XDG data/config paths, and the
  application's recent locations. Qt's `QUrlModel` accepts only existing
  directories and labels each by its folder basename, so KIO places (`trash:/`,
  `remote:/`, `recentlyused:/files|locations`) are not representable there.
- **Sandboxed builds** (Flatpak, Snap) cannot see host files; the supported
  mechanism is the XDG Desktop Portal FileChooser. `usesPortalFileDialog()` is
  true when `FLATPAK_ID`, `SNAP`, or `SNAP_NAME` is set (or the
  `PICTURA_PORTAL_FILE_DIALOG` override), and then `configureFileDialog()` does
  **not** force the built-in dialog, so `QFileDialog` uses the portal/native
  helper and the host chooser shows the host's real places. `main()` selects
  `QT_QPA_PLATFORMTHEME=xdgdesktopportal` before `QApplication` when sandboxed and
  the variable is unset, so the portal is used even without a manifest entry.

The sandbox cost is the toolkit's "File type" combo (no rolling) — the portal
chooser is host-rendered and cannot be customized. This is the deliberate trade:
rolling on native installs, correctness and host access under Flatpak.

The built-in dialogs are run without Qt's modal hint: input addressed to the
top-level window is filtered out for the duration instead of blocking it with a
modal window. KWin's "Dialog Parent" effect dims the parent of a *modal* window
(saturation 0.4, brightness 0.6), and the non-modal form avoids that while still
preventing interaction. The portal chooser is spawned by the host and is left to
its own modality. A user who also enables the separate "Dim Inactive" effect will
still see the window dimmed on focus loss; that is outside the application's
control.

The Flatpak manifest (roadmap) will need
`--env=QT_QPA_PLATFORMTHEME=xdgdesktopportal` and, for mounted/removable
locations, `--filesystem=xdg-run/gvfs` with `--talk-name=org.gtk.vfs.*` (or
`--filesystem=/media,/run/media,/mnt`).

### D8. Open As Smart Object accepts raster sources

CS6 accepts raster and vector sources there (`docs/05-layers/smart-objects.md`),
and `docs/10-workflow-io/open-and-new.md` states that a JPEG opened as a smart
object yields one editable smart-object layer. The dialog therefore uses the
shared Open filters, and `open_as_smart_object` grows a raster branch: when the
bytes do not parse as PSD/PSB, decode them with the Qt import edge and embed the
decoded layer with `convert_to_smart_object` — the same recipe `place_image`
already uses. The embedded payload is a PSD wrapper (Edit Contents works;
Export Contents emits PSD rather than the original JPEG, which the docs leave as
an open question at `open-and-new.md`). `Open As Smart Object` for a non-raster,
non-PSD file is refused without mutating the view.

### D9. Verification

- **Rust unit tests** (`pictura-app`) for the Qt-free pieces only: the suffix →
  writer classification (`format_for_path`/`raster_writer_for_suffix`), the
  `source_format` default, and `buffer_to_rgba_bytes` against `buffer_to_image`'s
  pixels for 1/2/3/4-channel buffers. The bridge methods call the C++ encode
  helper, which is linked by CMake and absent from a `cargo test` binary, so
  `save`/`export_image` themselves are covered by the C++ self-test (the same
  split the image-import bridge uses).
- **C++ self-test** in an existing suite: create a small document, Export As to
  PNG/JPEG/BMP, reopen each with `open_image` and compare pixels; Quick Export
  writes next to the source and records no history; a layered save to `.png`
  clears modified and re-saves as PNG on the next Save. Append-only exit code
  (next free code) and within the file-size ceiling.
- **Gates**: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D
  warnings`, `cargo nextest run --workspace`, doctests, `scripts/verify-full.sh`
  + headless self-test, and `openspec validate --all --strict`.

## Risks / Trade-offs

- **Transparent pixels in JPEG/BMP.** Neither carries alpha portably: Qt's BMP
  writer drops it (confirmed empirically in the self-test probe). → Flatten JPEG
  and BMP onto opaque white before writing; PNG, TIFF, and WebP keep alpha.
  Documented in the spec as observable behavior.
- **A layered document saved over a flat format loses layers on next Save.**
  `Save As foo.png` re-points the document to PNG, so the next `Ctrl+S` writes a
  flattened PNG again. → This matches Photoshop's Save As and is called out by
  D5's warning; the user keeps the PSD by using Export As/Save a Copy instead.
- **`QImageWriter` fails on a missing Qt image plugin** (e.g. no WebP plugin).
  → The handler cancels the `QSaveFile` (no partial file) and returns `false`;
  the app reports the failure. The format list is advisory and Qt's runtime
  writers are authoritative, exactly the import policy.
- **`image()` staleness.** Export uses `view->image()`. → Rust rebuilds it when
  `display_dirty`, and export goes through `current_buffer`, so the encoded
  bytes are the current composite.
- **New C++ files must be registered in CMake.** → The tasks name the explicit
  `CMakeLists.txt` edit; only `export_as_dialog` and `encode_image` are new.
- **Suffix is the format.** A file named `photo.jpg` holding PNG bytes is
  possible if the user forces a mismatched name filter. → The name filters drive
  the suffix (the file dialog appends/filters by suffix), and the writer reads
  the suffix, so the two can only disagree when the user types a mismatched name
  and accepts the warning; the result is a valid image with a misleading
  extension, not corruption.

## Open Questions

- **How Save As picks its default when the document has layers** is now settled:
  the format filter defaults to PSD (see D5 and the requirement's layered-default
  scenario). The Open/Place dialogs default to `All Formats`; only Save As keeps
  the smart default so a layered document is never silently flattened.
