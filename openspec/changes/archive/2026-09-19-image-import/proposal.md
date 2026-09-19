## Why

Kooka Pictura can only `Open` and `Place` PSD/PSB files. Photoshop's Open and
Place accept common raster images (PNG/JPEG/GIF/BMP/TIFF/WebP/…), so the primary
"get a picture into the app" paths are missing. Qt already decodes every format
the platform supports; the engine needs no decoder, only a boundary that hands
it pixels and construction code that builds its own structures from them.

## What Changes

- **New capability `image-import`**: a Qt-free probe + allocation budget in
  `pictura-codec`, an RGBA8888 decode edge at the app boundary, and engine
  construction of a document/layer from decoded pixels.
- **Engine**: `pictura-core::Document::from_rgba(name, width, height, rgba)` and
  `pictura-render::add_raster_layer_from_rgba(doc, name, width, height, rgba)`,
  both accepting the packed RGBA8888 layout the app already produces.
- **Probe**: `pictura_codec::probe_image(bytes, budget) -> Result<ImageProbe,
  ImportError>` parses declared format/width/height/bit-depth from the header
  and rejects over-budget files with a typed, provenance-carrying error *before*
  Qt decodes. The result is advisory (the header is attacker-controlled) and is
  paired with a cap on the actual decoded allocation.
- **App bridges**: `PictureView::open_image(path) -> bool` (decode →
  `Document::from_rgba` → replace the view) and `PictureView::place_image(path)
  -> QString` (decode → `add_raster_layer_from_rgba` →
  `convert_to_smart_object` → recomposite + one undo state → the new path).
- **Place reuses the existing smart-object path**: no new smart-object engine
  code. The placed layer keeps a raster proxy (unlike PSD Place), so it renders
  from the proxy and its embedded source is the authored PSD proxy, not the
  original image bytes.
- **Dialogs**: `File > Open` and `File > Place` add an `Images (…)` filter to
  the existing `Photoshop files (*.psd *.psb)` filter.
- **PSD stays native**: PSD/PSB continues through `pictura_codec::read_psd` and
  is never routed through Qt.
- **Modified `smart-object-layer-actions`**: the `File > Place…` requirement's
  dialog filter and bridge routing are broadened to include raster images.
- **No new dependencies** (no `image` crate; Qt supplies decoding).

## Capabilities

### New Capabilities

- `image-import`: Qt-free header probe + allocation budget, RGBA8888 decode at
  the Qt boundary, engine document/layer construction from decoded pixels, and
  the Open/Place entry points for common raster images.

### Modified Capabilities

- `smart-object-layer-actions`: the `File > Place…` command's dialog filter and
  bridge routing change from PSD/PSB-only to PSD/PSB-plus-supported-images;
  PSD/PSB keeps `place_smart_object`, other images route through the
  `image-import` path. The one-undo-state contract is unchanged.

## Impact

- `crates/pictura-codec/src/`: new image-header probe module, exported from
  `lib.rs`, with a typed `ImportError`.
- `crates/pictura-core/src/lib.rs`: `Document::from_rgba`.
- `crates/pictura-render/src/document_ops/layer_ops/create.rs` (or a sibling):
  `add_raster_layer_from_rgba`, re-exported from `layer_ops`/`document_ops`/`lib.rs`.
- `crates/pictura-app/src/cxxqt_object/{impl_core.rs,impl_layers_smart_object.rs}`:
  the two bridge methods and the decode-edge call.
- `crates/pictura-app/src/cxxqt_object.rs`: the cxx bridge declaration for the
  C++ decode helper (Qt `QImage`/`QImageReader` → packed RGBA8888 `Vec<u8>`).
- `crates/pictura-app/cpp/`: the decode helper translation unit, the Open/Place
  dialog filters (`frame.cpp`, `frame_menus.cpp`), and `CMakeLists.txt`
  registration.
- `crates/pictura-app/cpp/selftest_*.cpp`: one C++ self-test check (code 290).
- No new Rust dependencies.

## Non-Goals

Deferred deliberately; each is a separate phase or follow-up:

- **Phase 2** OS drag-and-drop, and **Phase 3** the interactive Free Transform
  placement session.
- **Original-file-bytes fidelity** on the placed smart object: `Export
  Contents` still yields the authored PSD proxy for images.
- **Replace Contents** for non-PSD images.
- **Multi-frame** (GIF/APNG) policy beyond frame 0; only frame 0 is imported.
- **ICC/EXIF/XMP** preservation; decoded pixels are imported without metadata.
- **Lazy / gigapixel image tabs** and downsampled previews.
- A labeled placeholder for unreadable formats: v1 refuses instead (see design).
- Any new decoder crate; the engine gains no format dependency.
