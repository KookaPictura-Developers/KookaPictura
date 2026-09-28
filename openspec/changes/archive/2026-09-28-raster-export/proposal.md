# Proposal

## Why

Kooka Pictura opens PSD/PSB and common raster images, but Save and Save As always
`write_psd`: saving a `foo.png` target silently writes PSD bytes, and there is no
way to export a flattened PNG/JPEG/TIFF/WebP/BMP. `File > Save for Web…` and the
`File > Export` leaves are disabled stubs, and the Layers row context menu has no
export entries. Import already has a Qt decode edge (`image-import`); export has
no encode edge at all. This closes the round trip and the two tracking issues
#60 (File menu export) and #107 (layer row export).

## What Changes

- **New capability `interop/raster-export`**: a Qt-boundary encode edge that
  writes the flattened composite to PNG/JPEG/TIFF/WebP/BMP, the `File ▸ Export
  As…` dialog (format, quality, scale), and `File ▸ Quick Export as PNG`.
- **BREAKING — format-aware Save/Save As**: the document remembers the format it
  was imported from (an opened PNG defaults Save As to PNG), and Save/Save As
  write the format named by the path — PSD/PSB through the existing codec, a
  supported raster through the encode edge. A successful raster save sets the
  path and clears the modified state, exactly as the PSD save does today. The
  PSD/PSB native path is unchanged.
- **Layers row context menu (#107)**: add `Export As…` and `Quick Export as
  PNG`, enabled for pixel and smart-object rows and disabled for group and
  adjustment rows. Both act on the flattened composite, never on a single layer.
- **No new dependencies**: Qt's image writers are the encoder, mirroring Qt's
  image readers as the decoder for import.

## Capabilities

### New Capabilities

- `interop/raster-export`: Qt-boundary encoding of the flattened composite to
  the supported raster formats, the format-to-encoder dispatch, the Export As
  dialog (format/quality/scale), and Quick Export as PNG.

### Modified Capabilities

- `document/document-lifecycle`: the "Save and Save As" requirement changes from
  always writing the PSD codec to writing the document's current output format,
  remembering the import source format as the Save As default and flattening for
  formats that cannot hold layers.
- `ui/layers-panel`: the "Panel and row menus" requirement gains `Export As…`
  and `Quick Export as PNG` on pixel and smart-object rows.

## Impact

- **App C++**: a new `crates/pictura-app/cpp/encode_image.{h,cpp}` translation
  unit (registered explicitly in `CMakeLists.txt`; CMake does not glob); a new
  `export_as_dialog.{h,cpp}`; wiring in `frame_menus.cpp`, `frame.h`/`frame.cpp`,
  `commands.h`, `command_tree.cpp`, and `panels/layers_panel_menu.cpp`.
- **App Rust**: a new sibling bridge `crates/pictura-app/src/cxxqt_object/export.rs`
  (registered in `build.rs`, mirroring `magnetic.rs`) declares the Qt encode edge
  and the `output_format`/`export_image` free functions, so the
  `cxxqt_object.rs` declaration list — at its file-size ceiling — does not grow;
  `impl_core.rs` dispatches `save` on the path suffix; `state.rs` gains the
  remembered source format; `helpers_composite.rs` factors the packed-RGBA
  extraction out of `buffer_to_image`.
- **Self-test**: one C++ self-test check (append-only exit code) covering the
  export round trip, the format-aware default, and the no-history contract.
- **Docs**: `docs/dev/STATE.md` record (carries `TASK-ALLOWS-DOCS`). No behavior
  change to `docs/`; the long-form contract in `save-and-save-as.md` and
  `export-formats.md` is already written.
- **Dependencies**: none added.
